use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Tenant {
    pub id: Uuid,

    #[validate(regex(path = *REGEX_SLUG, message = "Slug must be alphanumeric with dashes"))]
    pub slug: String,

    #[validate(length(min = 1, message = "Display name cannot be empty"))]
    pub display_name: String,

    pub settings: TenantSettings,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TenantSettings {
    pub mtls_required: bool,
    pub default_limits: Option<String>,
}

lazy_static::lazy_static! {
    static ref REGEX_SLUG: regex::Regex = regex::Regex::new(r"^[a-z0-9][a-z0-9_-]{1,63}$").unwrap();
}

impl Tenant {
    pub fn new(slug: String, display_name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            slug,
            display_name,
            settings: TenantSettings::default(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_tenant() {
        let tenant = Tenant::new("acme-corp".to_string(), "Acme Corp".to_string());
        assert!(tenant.validate().is_ok());
    }

    #[test]
    fn test_invalid_slug() {
        let tenant = Tenant::new("Acme Corp".to_string(), "Acme Corp".to_string()); // Capital letters not allowed in slug
        assert!(tenant.validate().is_err());
    }
}
