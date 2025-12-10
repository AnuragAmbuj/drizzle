use axum::{
    extract::{Json, State},
    http::StatusCode,
    routing::{get, post},
    Router,
};
use domain::policy::Policy;
use domain::route::{PathMatch, Route};
use domain::service::Service;
use domain::tenant::Tenant;
use dotenvy::dotenv;
use serde_json::json;
use snapshot::{builder::SnapshotBuilder, Snapshot};
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use std::sync::Arc;
use storage::{
    ApiKeyRepository, LimitPolicyRepository, PgApiKeyRepository, PgLimitPolicyRepository,
    PgPolicyRepository, PgRouteRepository, PgSecurityRepository, PgServiceRepository,
    PgTenantRepository, PgUserRepository, PolicyRepository, RouteRepository, SecurityRepository,
    ServiceRepository, TenantRepository, UserRepository,
};
pub mod auth;
use domain::policy::LimitPolicy;
use domain::security::SecurityConfig;
use domain::user::{User, UserRole}; // For seeding
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    tenant_repo: Arc<PgTenantRepository>,
    service_repo: Arc<PgServiceRepository>,
    route_repo: Arc<PgRouteRepository>,
    policy_repo: Arc<PgPolicyRepository>,
    limit_policy_repo: Arc<PgLimitPolicyRepository>,
    api_key_repo: Arc<PgApiKeyRepository>,
    user_repo: Arc<PgUserRepository>,
    security_repo: Arc<PgSecurityRepository>,
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to Postgres");

    let tenant_repo = Arc::new(PgTenantRepository::new(pool.clone()));
    let service_repo = Arc::new(PgServiceRepository::new(pool.clone()));
    let route_repo = Arc::new(PgRouteRepository::new(pool.clone()));
    let policy_repo = Arc::new(PgPolicyRepository::new(pool.clone()));
    let limit_policy_repo = Arc::new(PgLimitPolicyRepository::new(pool.clone()));
    let api_key_repo = Arc::new(PgApiKeyRepository::new(pool.clone()));
    let user_repo = Arc::new(PgUserRepository::new(pool.clone())); // Used pool
    let security_repo = Arc::new(PgSecurityRepository::new(pool));

    let state = AppState {
        tenant_repo,
        service_repo,
        route_repo,
        policy_repo,
        limit_policy_repo,
        api_key_repo,
        user_repo: user_repo.clone(),
        security_repo,
    };

    // Seed Admin
    let admin_exists = user_repo.get_by_username("admin").await.unwrap_or(None);
    if admin_exists.is_none() {
        let admin = User::new(
            "admin".to_string(),
            auth::hash_password("admin123"),
            UserRole::Admin,
        );
        println!("Seeding default admin user: admin / admin123");
        if let Err(e) = user_repo.create(&admin).await {
            println!("Failed to seed admin user: {:?}", e);
        }
    }

    // build our application with a route
    // Protected Routes
    let api_routes = Router::new()
        .route("/tenants", post(create_tenant))
        .route(
            "/tenants/:id",
            axum::routing::put(update_tenant).delete(delete_tenant),
        )
        .route("/services", post(create_service))
        .route(
            "/services/:id",
            axum::routing::put(update_service).delete(delete_service),
        )
        .route("/routes", post(create_route))
        .route("/routes/:id", axum::routing::delete(delete_route))
        .route("/policies", post(create_policy))
        .route(
            "/policies/:id",
            axum::routing::put(update_policy).delete(delete_policy),
        )
        .route("/limits", post(create_limit_policy))
        .route("/api-keys", post(create_api_key))
        .route(
            "/security",
            get(get_security_config).put(update_security_config),
        )
        .route_layer(axum::middleware::from_fn(auth::auth_middleware));

    // Public Routes & Merge
    let app = Router::new()
        .route("/", get(root))
        .route("/auth/login", post(auth::login_handler))
        .route("/snapshot", get(get_snapshot))
        .merge(api_routes)
        .with_state(state);

    // run our app with hyper
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "Drizzle Control Plane API"
}

async fn get_snapshot(
    State(state): State<AppState>,
) -> Result<Json<Snapshot>, (StatusCode, String)> {
    // Fetch all data in parallel (could use join!, but sequential await is fine for MVP)
    let tenants = state.tenant_repo.get_all().await.unwrap_or_default();
    let services = state.service_repo.get_all().await.unwrap_or_default();
    let routes = state.route_repo.get_all().await.unwrap_or_default();
    let policies = state.policy_repo.get_all().await.unwrap_or_default();
    let limits = state.limit_policy_repo.get_all().await.unwrap_or_default();
    let api_keys_list = state.api_key_repo.get_all().await.unwrap_or_default();
    // Default if missing
    let security = state.security_repo.get().await.unwrap_or_default();

    let api_keys: std::collections::HashMap<String, String> = api_keys_list.into_iter().collect();

    let version = "v1".to_string(); // Defined version for the builder
    let snapshot = SnapshotBuilder::new(version, "admin-api".to_string())
        .with_tenants(tenants)
        .with_services(services)
        .with_routes(routes)
        .with_policies(policies)
        .with_limits(limits)
        .with_api_keys(api_keys)
        .with_security(security)
        .build()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(snapshot))
}

// --- Tenant Handlers ---

async fn create_tenant(
    State(state): State<AppState>,
    Json(payload): Json<CreateTenantRequest>,
) -> Json<serde_json::Value> {
    let tenant = Tenant::new(payload.slug, payload.display_name);
    match state.tenant_repo.create(&tenant).await {
        Ok(_) => Json(json!(tenant)),
        Err(e) => {
            eprintln!("Failed to create tenant: {:?}", e);
            Json(json!({ "error": "Failed to create tenant" }))
        }
    }
}

async fn update_tenant(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
    Json(payload): Json<UpdateTenantRequest>,
) -> Json<serde_json::Value> {
    match state.tenant_repo.update(id, &payload.display_name).await {
        Ok(_) => Json(json!({"status": "updated"})),
        Err(e) => Json(json!({"error": e.to_string()})),
    }
}

async fn delete_tenant(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> Json<serde_json::Value> {
    match state.tenant_repo.delete(id).await {
        Ok(_) => Json(json!({"status": "deleted"})),
        Err(e) => Json(json!({"error": e.to_string()})),
    }
}

// --- Service Handlers ---

async fn create_service(
    State(state): State<AppState>,
    Json(payload): Json<CreateServiceRequest>,
) -> Json<serde_json::Value> {
    let service = Service::new(
        payload.tenant_id,
        Uuid::new_v4(),
        payload.name,
        vec![payload.host],
    );
    match state.service_repo.create(&service).await {
        Ok(_) => Json(json!(service)),
        Err(e) => {
            eprintln!("Failed to create service: {:?}", e);
            Json(json!({ "error": "Failed to create service" }))
        }
    }
}

async fn update_service(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
    Json(payload): Json<UpdateServiceRequest>,
) -> Json<serde_json::Value> {
    match state
        .service_repo
        .update(id, &payload.name, &payload.hosts)
        .await
    {
        Ok(_) => Json(json!({"status": "updated"})),
        Err(e) => Json(json!({"error": e.to_string()})),
    }
}

async fn delete_service(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> Json<serde_json::Value> {
    match state.service_repo.delete(id).await {
        Ok(_) => Json(json!({"status": "deleted"})),
        Err(e) => Json(json!({"error": e.to_string()})),
    }
}

// --- Route Handlers ---

async fn create_route(
    State(state): State<AppState>,
    Json(payload): Json<CreateRouteRequest>,
) -> Json<serde_json::Value> {
    let route = Route::new(
        payload.service_id,
        payload.name,
        PathMatch::Prefix(payload.path),
    );
    match state.route_repo.create(&route).await {
        Ok(_) => Json(json!(route)),
        Err(e) => {
            eprintln!("Failed to create route: {:?}", e);
            Json(json!({ "error": "Failed to create route" }))
        }
    }
}

async fn delete_route(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> Json<serde_json::Value> {
    match state.route_repo.delete(id).await {
        Ok(_) => Json(json!({"status": "deleted"})),
        Err(e) => Json(json!({"error": e.to_string()})),
    }
}

// ... snapshot ...

// --- Policy Handlers ---

async fn create_policy(
    State(state): State<AppState>,
    Json(payload): Json<CreatePolicyRequest>,
) -> Json<serde_json::Value> {
    let policy = Policy::new(payload.tenant_id, payload.name, payload.content);
    match state.policy_repo.create(&policy).await {
        Ok(_) => Json(json!(policy)),
        Err(e) => {
            eprintln!("Failed to create policy: {:?}", e);
            Json(json!({ "error": "Failed to create policy" }))
        }
    }
}

async fn update_policy(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
    Json(payload): Json<UpdatePolicyRequest>,
) -> Json<serde_json::Value> {
    match state
        .policy_repo
        .update(id, &payload.name, &payload.content)
        .await
    {
        Ok(_) => Json(json!({"status": "updated"})),
        Err(e) => Json(json!({"error": e.to_string()})),
    }
}

async fn delete_policy(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> Json<serde_json::Value> {
    match state.policy_repo.delete(id).await {
        Ok(_) => Json(json!({"status": "deleted"})),
        Err(e) => Json(json!({"error": e.to_string()})),
    }
}

// --- DTOs ---

#[derive(serde::Deserialize)]
struct CreateTenantRequest {
    slug: String,
    display_name: String,
}

#[derive(serde::Deserialize)]
struct UpdateTenantRequest {
    display_name: String,
}

#[derive(serde::Deserialize)]
struct CreateServiceRequest {
    tenant_id: Uuid,
    name: String,
    host: String, // Kept for compat with create logic
}

#[derive(serde::Deserialize)]
struct UpdateServiceRequest {
    name: String,
    hosts: Vec<String>,
}

#[derive(serde::Deserialize)]
struct CreateRouteRequest {
    service_id: Uuid,
    name: String,
    path: String,
}

#[derive(serde::Deserialize)]
struct CreateLimitPolicyRequest {
    tenant_id: Uuid,
    name: String,
    rate: u32,
    burst: u32,
}

async fn create_limit_policy(
    State(state): State<AppState>,
    Json(payload): Json<CreateLimitPolicyRequest>,
) -> Json<serde_json::Value> {
    let policy = LimitPolicy::new(payload.tenant_id, payload.name, payload.rate, payload.burst);
    match state.limit_policy_repo.create(&policy).await {
        Ok(_) => Json(json!(policy)),
        Err(e) => {
            eprintln!("Failed to create limit policy: {:?}", e);
            Json(json!({ "error": "Failed to create limit policy" }))
        }
    }
}

#[derive(serde::Deserialize)]
struct CreatePolicyRequest {
    tenant_id: Uuid,
    name: String,
    content: String,
}

#[derive(serde::Deserialize)]
struct UpdatePolicyRequest {
    name: String,
    content: String,
}

async fn create_api_key(
    State(state): State<AppState>,
    Json(payload): Json<CreateApiKeyRequest>,
) -> Json<serde_json::Value> {
    let id = Uuid::new_v4();
    match state
        .api_key_repo
        .create(id, &payload.key, payload.tenant_id)
        .await
    {
        Ok(_) => Json(json!({ "id": id, "key": payload.key, "tenant_id": payload.tenant_id })),
        Err(e) => {
            eprintln!("Failed to create API key: {:?}", e);
            Json(json!({ "error": "Failed to create API key" }))
        }
    }
}

#[derive(serde::Deserialize)]
struct CreateApiKeyRequest {
    tenant_id: Uuid,
    key: String,
}

// --- Security ---

async fn get_security_config(State(state): State<AppState>) -> Json<serde_json::Value> {
    match state.security_repo.get().await {
        Ok(config) => Json(json!(config)),
        Err(e) => {
            eprintln!("Failed to get security config: {:?}", e);
            // If missing, return default? Behave nicely.
            Json(json!(SecurityConfig::default()))
        }
    }
}

#[derive(serde::Deserialize)]
struct UpdateSecurityRequest {
    global_rate_limit: u32,
    global_burst: u32,
}

async fn update_security_config(
    State(state): State<AppState>,
    Json(payload): Json<UpdateSecurityRequest>,
) -> Json<serde_json::Value> {
    let config = SecurityConfig {
        id: 1,
        global_rate_limit: payload.global_rate_limit,
        global_burst: payload.global_burst,
    };

    match state.security_repo.update(&config).await {
        Ok(_) => Json(json!({"status": "updated", "config": config})),
        Err(e) => Json(json!({"error": e.to_string()})),
    }
}
