use axum::{
    extract::{Json, State},
    routing::{get, post},
    Router,
};
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
    PgRouteRepository, PgServiceRepository, PgTenantRepository, RouteRepository, ServiceRepository,
    TenantRepository,
};
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    tenant_repo: Arc<PgTenantRepository>,
    service_repo: Arc<PgServiceRepository>,
    route_repo: Arc<PgRouteRepository>,
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
    let route_repo = Arc::new(PgRouteRepository::new(pool)); // Last use of pool

    let state = AppState {
        tenant_repo,
        service_repo,
        route_repo,
    };

    // build our application with a route
    let app = Router::new()
        .route("/", get(root))
        .route("/tenants", post(create_tenant))
        .route("/services", post(create_service))
        .route("/routes", post(create_route))
        .route("/snapshot", get(get_snapshot))
        .with_state(state);

    // run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Admin API listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "Drizzle Control Plane API"
}

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

async fn create_service(
    State(state): State<AppState>,
    Json(payload): Json<CreateServiceRequest>,
) -> Json<serde_json::Value> {
    let service = Service::new(
        payload.tenant_id,
        Uuid::new_v4(), // Placeholder Environment ID
        payload.name,
        vec![payload.host], // Single host for now
    );

    match state.service_repo.create(&service).await {
        Ok(_) => Json(json!(service)),
        Err(e) => {
            eprintln!("Failed to create service: {:?}", e);
            Json(json!({ "error": "Failed to create service" }))
        }
    }
}

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

async fn get_snapshot(State(state): State<AppState>) -> Json<Snapshot> {
    // Fetch all data in parallel (could use join!, but sequential await is fine for MVP)
    let tenants = state.tenant_repo.get_all().await.unwrap_or_default();
    let services = state.service_repo.get_all().await.unwrap_or_default();
    let routes = state.route_repo.get_all().await.unwrap_or_default();

    let builder = SnapshotBuilder::new("v1".to_string(), "admin-api".to_string())
        .with_tenants(tenants)
        .with_services(services)
        .with_routes(routes);

    let snapshot = builder.build().unwrap();
    Json(snapshot)
}

#[derive(serde::Deserialize)]
struct CreateTenantRequest {
    slug: String,
    display_name: String,
}

#[derive(serde::Deserialize)]
struct CreateServiceRequest {
    tenant_id: Uuid,
    name: String,
    host: String,
}

#[derive(serde::Deserialize)]
struct CreateRouteRequest {
    service_id: Uuid,
    name: String,
    path: String,
}
