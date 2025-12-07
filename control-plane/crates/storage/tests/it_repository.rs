use domain::tenant::Tenant;
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use storage::{
    PgRouteRepository, PgServiceRepository, PgTenantRepository, RouteRepository, ServiceRepository,
    TenantRepository,
};
use uuid::Uuid;

#[tokio::test]
async fn test_tenant_repository() {
    dotenv().ok();
    // Assuming init_db.sh has been run and Postgres is available at default location or DATABASE_URL
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@localhost:5442/drizzle".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await
        .expect("Failed to connect to DB");

    let tenant_repo = PgTenantRepository::new(pool.clone());
    let service_repo = PgServiceRepository::new(pool.clone());
    let route_repo = PgRouteRepository::new(pool.clone());

    // 1. Create Tenant
    let slug = format!("test-tenant-{}", Uuid::new_v4());
    let tenant = Tenant::new(slug.clone(), "Integration Test".to_string());

    tenant_repo
        .create(&tenant)
        .await
        .expect("Failed to create tenant");

    // 2. Create Service
    let service = domain::service::Service::new(
        tenant.id,
        Uuid::new_v4(), // Env ID
        "test-service".to_string(),
        vec!["example.com".to_string()],
    );
    service_repo
        .create(&service)
        .await
        .expect("Failed to create service");

    // 3. Create Route
    let route = domain::route::Route::new(
        service.id,
        "test-route".to_string(),
        domain::route::PathMatch::Prefix("/test".to_string()),
    );
    route_repo
        .create(&route)
        .await
        .expect("Failed to create route");

    // 4. Verify All
    let tenants = tenant_repo.get_all().await.expect("Failed to get tenants");
    assert!(tenants.iter().any(|t| t.slug == slug));

    let services = service_repo
        .get_all()
        .await
        .expect("Failed to get services");
    assert!(services.iter().any(|s| s.name == "test-service"));

    let routes = route_repo.get_all().await.expect("Failed to get routes");
    assert!(routes.iter().any(|r| r.name == "test-route"));
}
