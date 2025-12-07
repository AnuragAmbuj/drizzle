use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Policy {
    pub id: Uuid,
    pub tenant_id: Uuid,

    #[validate(length(min = 1))]
    pub name: String,

    /// The actual Cedar policy text
    #[validate(length(min = 1))]
    pub content: String,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct LimitPolicy {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,

    // Simple verification for now
    pub rate: u32,
    pub burst: u32,
}

impl Policy {
    pub fn new(tenant_id: Uuid, name: String, content: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            tenant_id,
            name,
            content,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        }
    }
}
