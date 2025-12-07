use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Service {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub environment_id: Uuid, // References an Environment entity

    #[validate(regex(path = *REGEX_NAME, message = "Name must be alphanumeric with dashes"))]
    pub name: String,

    #[validate(length(min = 1, message = "Must have at least one host"))]
    pub hosts: Vec<String>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

lazy_static::lazy_static! {
    static ref REGEX_NAME: regex::Regex = regex::Regex::new(r"^[a-z0-9][a-z0-9_-]{1,63}$").unwrap();
}

impl Service {
    pub fn new(tenant_id: Uuid, environment_id: Uuid, name: String, hosts: Vec<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            tenant_id,
            environment_id,
            name,
            hosts,
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
    fn test_valid_service() {
        let svc = Service::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "billing-api".to_string(),
            vec!["api.acme.com".to_string()],
        );
        assert!(svc.validate().is_ok());
    }

    #[test]
    fn test_invalid_hosts() {
        let svc = Service::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "billing-api".to_string(),
            vec![], // Empty hosts
        );
        assert!(svc.validate().is_err());
    }
}
