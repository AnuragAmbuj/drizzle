use async_trait::async_trait;
use domain::route::Route;
use domain::service::Service;
use domain::tenant::Tenant;
use sqlx::PgPool;

#[async_trait]
pub trait TenantRepository: Send + Sync {
    async fn create(&self, tenant: &Tenant) -> anyhow::Result<()>;
    async fn get_all(&self) -> anyhow::Result<Vec<Tenant>>;
}

#[async_trait]
pub trait ServiceRepository: Send + Sync {
    async fn create(&self, service: &Service) -> anyhow::Result<()>;
    async fn get_all(&self) -> anyhow::Result<Vec<Service>>;
}

#[async_trait]
pub trait RouteRepository: Send + Sync {
    async fn create(&self, route: &Route) -> anyhow::Result<()>;
    async fn get_all(&self) -> anyhow::Result<Vec<Route>>;
}

pub struct PgTenantRepository {
    pool: PgPool,
}

impl PgTenantRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

pub struct PgServiceRepository {
    pool: PgPool,
}

impl PgServiceRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

pub struct PgRouteRepository {
    pool: PgPool,
}

impl PgRouteRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TenantRepository for PgTenantRepository {
    async fn create(&self, tenant: &Tenant) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO tenants (id, slug, display_name, settings, created_at, updated_at, deleted_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
            tenant.id,
            tenant.slug,
            tenant.display_name,
            serde_json::to_value(&tenant.settings)?,
            tenant.created_at,
            tenant.updated_at,
            tenant.deleted_at
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get_all(&self) -> anyhow::Result<Vec<Tenant>> {
        let recs = sqlx::query!(
            r#"
            SELECT id, slug, display_name, settings, created_at, updated_at, deleted_at
            FROM tenants
            WHERE deleted_at IS NULL
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let tenants = recs
            .into_iter()
            .map(|r| {
                Ok(Tenant {
                    id: r.id,
                    slug: r.slug,
                    display_name: r.display_name,
                    settings: serde_json::from_value(r.settings)?,
                    created_at: r.created_at,
                    updated_at: r.updated_at,
                    deleted_at: r.deleted_at,
                })
            })
            .collect::<anyhow::Result<Vec<Tenant>>>()?;

        Ok(tenants)
    }
}

#[async_trait]
impl ServiceRepository for PgServiceRepository {
    async fn create(&self, service: &Service) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO services (id, tenant_id, environment_id, name, hosts, created_at, updated_at, deleted_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
            service.id,
            service.tenant_id,
            service.environment_id,
            service.name,
            &service.hosts,
            service.created_at,
            service.updated_at,
            service.deleted_at
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get_all(&self) -> anyhow::Result<Vec<Service>> {
        let recs = sqlx::query!(
            r#"
            SELECT id, tenant_id, environment_id, name, hosts, created_at, updated_at, deleted_at
            FROM services
            WHERE deleted_at IS NULL
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let services = recs
            .into_iter()
            .map(|r| Service {
                id: r.id,
                tenant_id: r.tenant_id,
                environment_id: r.environment_id,
                name: r.name,
                hosts: r.hosts,
                created_at: r.created_at,
                updated_at: r.updated_at,
                deleted_at: r.deleted_at,
            })
            .collect();

        Ok(services)
    }
}

#[async_trait]
impl RouteRepository for PgRouteRepository {
    async fn create(&self, route: &Route) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO routes (id, service_id, name, priority, match_methods, match_path, match_headers, created_at, updated_at, deleted_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
            route.id,
            route.service_id,
            route.name,
            route.priority,
            &route.match_methods,
            serde_json::to_value(&route.match_path)?,
            serde_json::to_value(&route.match_headers)?,
            route.created_at,
            route.updated_at,
            route.deleted_at
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get_all(&self) -> anyhow::Result<Vec<Route>> {
        let recs = sqlx::query!(
            r#"
            SELECT id, service_id, name, priority, match_methods, match_path, match_headers, created_at, updated_at, deleted_at
            FROM routes
            WHERE deleted_at IS NULL
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let routes = recs
            .into_iter()
            .map(|r| {
                Ok(Route {
                    id: r.id,
                    service_id: r.service_id,
                    name: r.name,
                    priority: r.priority,
                    match_methods: r.match_methods.unwrap_or_default(),
                    match_path: serde_json::from_value(
                        r.match_path.unwrap_or(serde_json::json!({})),
                    )?, // Should not be null if migrated correctly but for safety
                    match_headers: serde_json::from_value(
                        r.match_headers.unwrap_or(serde_json::json!({})),
                    )?,
                    created_at: r.created_at,
                    updated_at: r.updated_at,
                    deleted_at: r.deleted_at,
                })
            })
            .collect::<anyhow::Result<Vec<Route>>>()?;

        Ok(routes)
    }
}
