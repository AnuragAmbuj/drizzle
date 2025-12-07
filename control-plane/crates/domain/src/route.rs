use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Route {
    pub id: Uuid,
    pub service_id: Uuid,

    #[validate(regex(path = *REGEX_NAME, message = "Name must be alphanumeric"))]
    pub name: String,

    pub priority: i32,

    // Matching rules
    pub match_methods: Vec<String>, // e.g., ["GET", "POST"]
    pub match_path: PathMatch,      // e.g., { type: "Prefix", value: "/v1" }

    // Simplified for now, just a map of header keys to values
    pub match_headers: HashMap<String, String>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum PathMatch {
    Prefix(String),
    Exact(String),
    Regex(String),
}

lazy_static::lazy_static! {
    static ref REGEX_NAME: regex::Regex = regex::Regex::new(r"^[a-z0-9][a-z0-9_-]{1,63}$").unwrap();
}

impl Route {
    pub fn new(service_id: Uuid, name: String, path: PathMatch) -> Self {
        Self {
            id: Uuid::new_v4(),
            service_id,
            name,
            priority: 0,
            match_methods: vec![],
            match_path: path,
            match_headers: HashMap::new(),
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
    fn test_route_creation() {
        let route = Route::new(
            Uuid::new_v4(),
            "checkout-flow".to_string(),
            PathMatch::Prefix("/checkout".to_string()),
        );
        assert!(route.validate().is_ok());
    }
}
