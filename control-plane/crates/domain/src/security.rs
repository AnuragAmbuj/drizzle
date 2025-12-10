use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SecurityConfig {
    pub id: i32,
    #[validate(range(min = 1))]
    pub global_rate_limit: u32,
    #[validate(range(min = 1))]
    pub global_burst: u32,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            id: 1,
            global_rate_limit: 100,
            global_burst: 50,
        }
    }
}
