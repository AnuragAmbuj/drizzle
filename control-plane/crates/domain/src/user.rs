use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum UserRole {
    Admin,
    Editor,
    Viewer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub role: UserRole,
}

impl User {
    pub fn new(username: String, password_hash: String, role: UserRole) -> Self {
        Self {
            id: Uuid::new_v4(),
            username,
            password_hash,
            role,
        }
    }
}
