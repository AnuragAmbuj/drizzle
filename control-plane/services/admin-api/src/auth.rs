use crate::AppState;
use axum::{
    extract::{Json, Request, State},
    http::{header::AUTHORIZATION, HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use storage::UserRepository;

const SECRET: &str = "super_secret_jwt_key_change_me_in_prod"; // TODO: Env var

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String, // username
    pub role: String,
    pub exp: usize,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub role: String,
}

pub async fn login_handler(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, Json<serde_json::Value>)> {
    // 1. Fetch user
    let user_opt = state
        .user_repo
        .get_by_username(&payload.username)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "DB error"})),
            )
        })?;

    let user = match user_opt {
        Some(u) => u,
        None => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "Invalid credentials"})),
            ))
        }
    };

    // 2. Verify password
    if !verify(&payload.password, &user.password_hash).unwrap_or(false) {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Invalid credentials"})),
        ));
    }

    // 3. Generate JWT
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .expect("valid timestamp")
        .timestamp();

    let role_str = match user.role {
        domain::user::UserRole::Admin => "admin",
        domain::user::UserRole::Editor => "editor",
        domain::user::UserRole::Viewer => "viewer",
    };

    let claims = Claims {
        sub: user.username.clone(),
        role: role_str.to_string(),
        exp: expiration as usize,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(SECRET.as_ref()),
    )
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Token creation failed"})),
        )
    })?;

    Ok(Json(LoginResponse {
        token,
        role: role_str.to_string(),
    }))
}

// Helper to seed initial user
pub fn hash_password(password: &str) -> String {
    hash(password, DEFAULT_COST).unwrap()
}

pub async fn auth_middleware(
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = headers
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if !auth_header.starts_with("Bearer ") {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let token = &auth_header[7..];

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(SECRET.as_ref()),
        &Validation::default(),
    )
    .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Ideally verify user exists in DB, but for stateless JWT this is enough + role check
    // We can inject claims into extension if needed: request.extensions_mut().insert(token_data.claims);

    let response = next.run(request).await;
    Ok(response)
}
