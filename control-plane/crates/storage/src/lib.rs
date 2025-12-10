use async_trait::async_trait;
use domain::policy::{LimitPolicy, Policy};
use domain::route::Route;
use domain::service::Service;
use domain::tenant::Tenant;
use sqlx::PgPool;

#[async_trait]
#[async_trait]
pub trait TenantRepository: Send + Sync {
    async fn create(&self, tenant: &Tenant) -> anyhow::Result<()>;
    async fn get_all(&self) -> anyhow::Result<Vec<Tenant>>;
    async fn update(&self, id: uuid::Uuid, display_name: &str) -> anyhow::Result<()>;
    async fn delete(&self, id: uuid::Uuid) -> anyhow::Result<()>;
}

#[async_trait]
pub trait ServiceRepository: Send + Sync {
    async fn create(&self, service: &Service) -> anyhow::Result<()>;
    async fn get_all(&self) -> anyhow::Result<Vec<Service>>;
    async fn update(&self, id: uuid::Uuid, name: &str, hosts: &[String]) -> anyhow::Result<()>;
    async fn delete(&self, id: uuid::Uuid) -> anyhow::Result<()>;
}

// ... RouteRepository ...

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

    async fn update(&self, id: uuid::Uuid, display_name: &str) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            UPDATE tenants 
            SET display_name = $1, updated_at = NOW()
            WHERE id = $2 AND deleted_at IS NULL
            "#,
            display_name,
            id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn delete(&self, id: uuid::Uuid) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            UPDATE tenants 
            SET deleted_at = NOW()
            WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
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

    async fn update(&self, id: uuid::Uuid, name: &str, hosts: &[String]) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            UPDATE services
            SET name = $1, hosts = $2, updated_at = NOW()
            WHERE id = $3 AND deleted_at IS NULL
            "#,
            name,
            hosts,
            id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn delete(&self, id: uuid::Uuid) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            UPDATE services
            SET deleted_at = NOW()
            WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

#[async_trait]
pub trait RouteRepository: Send + Sync {
    async fn create(&self, route: &Route) -> anyhow::Result<()>;
    async fn get_all(&self) -> anyhow::Result<Vec<Route>>;
    async fn delete(&self, id: uuid::Uuid) -> anyhow::Result<()>;
    // Simplified update for now (Name only or similar? Or full replacement?)
    // In a real app we'd have a specific DTO.
    // For now let's support deleting and re-creating as the primary "Edit" flow on Frontend if structure changes too much,
    // OR just support soft delete.
    // User asked for "Edit".
    // I will add delete. Editing complex routes via SQL in this file might be verbose without a DTO.
    // Let's add `update` that takes name and priority for now, as path might be complex JSON.
    // Actually, I can accept `match_path` json value?
    // Let's implement `delete` first.
}

// ... in impl ...
#[async_trait]
impl RouteRepository for PgRouteRepository {
    async fn create(&self, route: &Route) -> anyhow::Result<()> {
        // ... (CREATE) ...
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
        // ... (GET ALL) ...
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
                    )?,
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

    async fn delete(&self, id: uuid::Uuid) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            UPDATE routes 
            SET deleted_at = NOW()
            WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

// ... ApiKeys ...

#[async_trait]
pub trait ApiKeyRepository: Send + Sync {
    async fn create(
        &self,
        id: uuid::Uuid,
        key_value: &str,
        tenant_id: uuid::Uuid,
    ) -> anyhow::Result<()>;
    /// Returns (Key, TenantID) tuples
    async fn get_all(&self) -> anyhow::Result<Vec<(String, String)>>;
}

pub struct PgApiKeyRepository {
    pool: PgPool,
}

impl PgApiKeyRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ApiKeyRepository for PgApiKeyRepository {
    async fn create(
        &self,
        id: uuid::Uuid,
        key_value: &str,
        tenant_id: uuid::Uuid,
    ) -> anyhow::Result<()> {
        let now = chrono::Utc::now();
        sqlx::query!(
            r#"
            INSERT INTO api_keys (id, key_value, tenant_id, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            id,
            key_value,
            tenant_id,
            now,
            now
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get_all(&self) -> anyhow::Result<Vec<(String, String)>> {
        let recs = sqlx::query!(
            r#"
            SELECT key_value, tenant_id
            FROM api_keys
            WHERE deleted_at IS NULL
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        // Map Tenant UUID to String for the Snapshot logic
        Ok(recs
            .into_iter()
            .map(|r| (r.key_value, r.tenant_id.to_string()))
            .collect())
    }
}

#[async_trait]
pub trait PolicyRepository: Send + Sync {
    async fn create(&self, policy: &Policy) -> anyhow::Result<()>;
    async fn get_all(&self) -> anyhow::Result<Vec<Policy>>;
    async fn update(&self, id: uuid::Uuid, name: &str, content: &str) -> anyhow::Result<()>;
    async fn delete(&self, id: uuid::Uuid) -> anyhow::Result<()>;
}

pub struct PgPolicyRepository {
    pool: PgPool,
}

impl PgPolicyRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

// ... impl ...
#[async_trait]
impl PolicyRepository for PgPolicyRepository {
    async fn create(&self, policy: &Policy) -> anyhow::Result<()> {
        // ... (CREATE) ...
        sqlx::query!(
            r#"
            INSERT INTO policies (id, tenant_id, name, content, created_at, updated_at, deleted_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
            policy.id,
            policy.tenant_id,
            policy.name,
            policy.content,
            policy.created_at,
            policy.updated_at,
            policy.deleted_at
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get_all(&self) -> anyhow::Result<Vec<Policy>> {
        let recs = sqlx::query!(
            r#"
            SELECT id, tenant_id, name, content, created_at, updated_at, deleted_at
            FROM policies
            WHERE deleted_at IS NULL
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let policies = recs
            .into_iter()
            .map(|r| Policy {
                id: r.id,
                tenant_id: r.tenant_id.unwrap_or_default(),
                name: r.name,
                content: r.content,
                created_at: r.created_at,
                updated_at: r.updated_at,
                deleted_at: r.deleted_at,
            })
            .collect();
        Ok(policies)
    }

    async fn update(&self, id: uuid::Uuid, name: &str, content: &str) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            UPDATE policies
            SET name = $1, content = $2, updated_at = NOW()
            WHERE id = $3 AND deleted_at IS NULL
            "#,
            name,
            content,
            id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn delete(&self, id: uuid::Uuid) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            UPDATE policies
            SET deleted_at = NOW()
            WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

#[async_trait]
pub trait LimitPolicyRepository: Send + Sync {
    async fn create(&self, policy: &LimitPolicy) -> anyhow::Result<()>;
    async fn get_all(&self) -> anyhow::Result<Vec<LimitPolicy>>;
}

pub struct PgLimitPolicyRepository {
    pool: PgPool,
}

impl PgLimitPolicyRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl LimitPolicyRepository for PgLimitPolicyRepository {
    async fn create(&self, policy: &LimitPolicy) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO limit_policies (id, tenant_id, name, rate, burst)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            policy.id,
            policy.tenant_id,
            policy.name,
            policy.rate as i32,
            policy.burst as i32
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get_all(&self) -> anyhow::Result<Vec<LimitPolicy>> {
        let recs = sqlx::query!(
            r#"
            SELECT id, tenant_id, name, rate, burst
            FROM limit_policies
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let policies = recs
            .into_iter()
            .map(|r| LimitPolicy {
                id: r.id,
                tenant_id: r.tenant_id.unwrap_or_default(),
                name: r.name,
                rate: r.rate as u32,
                burst: r.burst as u32,
            })
            .collect();
        Ok(policies)
    }
}

use domain::user::{User, UserRole};

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(&self, user: &User) -> anyhow::Result<()>;
    async fn get_by_username(&self, username: &str) -> anyhow::Result<Option<User>>;
}

pub struct PgUserRepository {
    pool: PgPool,
}

impl PgUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PgUserRepository {
    async fn create(&self, user: &User) -> anyhow::Result<()> {
        let role_str = match user.role {
            UserRole::Admin => "admin",
            UserRole::Editor => "editor",
            UserRole::Viewer => "viewer",
        };

        sqlx::query!(
            r#"
            INSERT INTO users (id, username, password_hash, role, created_at, updated_at)
            VALUES ($1, $2, $3, $4::user_role, NOW(), NOW())
            "#,
            user.id,
            user.username,
            user.password_hash,
            role_str as _
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get_by_username(&self, username: &str) -> anyhow::Result<Option<User>> {
        let rec = sqlx::query!(
            r#"
            SELECT id, username, password_hash, role as "role: String", created_at, updated_at
            FROM users
            WHERE username = $1 AND deleted_at IS NULL
            "#,
            username
        )
        .fetch_optional(&self.pool)
        .await?;

        match rec {
            Some(r) => {
                let role = match r.role.as_str() {
                    "admin" => UserRole::Admin,
                    "editor" => UserRole::Editor,
                    "viewer" => UserRole::Viewer,
                    _ => return Err(anyhow::anyhow!("Invalid role in DB")),
                };

                Ok(Some(User {
                    id: r.id,
                    username: r.username,
                    password_hash: r.password_hash,
                    role,
                }))
            }
            None => Ok(None),
        }
    }
}

use domain::security::SecurityConfig;

#[async_trait]
pub trait SecurityRepository: Send + Sync {
    async fn get(&self) -> anyhow::Result<SecurityConfig>;
    async fn update(&self, config: &SecurityConfig) -> anyhow::Result<()>;
}

pub struct PgSecurityRepository {
    pool: PgPool,
}

impl PgSecurityRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SecurityRepository for PgSecurityRepository {
    async fn get(&self) -> anyhow::Result<SecurityConfig> {
        let rec = sqlx::query!(
            r#"
            SELECT id, global_rate_limit, global_burst
            FROM security_config
            WHERE id = 1
            "#
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(SecurityConfig {
            id: rec.id,
            global_rate_limit: rec.global_rate_limit as u32,
            global_burst: rec.global_burst as u32,
        })
    }

    async fn update(&self, config: &SecurityConfig) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            UPDATE security_config
            SET global_rate_limit = $1, global_burst = $2
            WHERE id = 1
            "#,
            config.global_rate_limit as i32,
            config.global_burst as i32
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
