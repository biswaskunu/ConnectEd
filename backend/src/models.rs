use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::Type;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Trainee,
    Trainer,
    Admin,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,        // user id
    pub role: Role,
    pub token_type: String, // "access" | "refresh"
    pub exp: usize,
}

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub id: Uuid,
    pub role: Role,
}

// AppError
pub enum AppError {
    Unauthorized,                 // 401 - missing/invalid token
    Forbidden,                    // 403 - authenticated, wrong role
    NotFound,                     // 404
    UnprocessableEntity(String),  // 422 - validation
    Internal(anyhow::Error),      // 500
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized".to_string()),
            AppError::Forbidden => (StatusCode::FORBIDDEN, "forbidden".to_string()),
            AppError::NotFound => (StatusCode::NOT_FOUND, "not found".to_string()),
            AppError::UnprocessableEntity(msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg),
            AppError::Internal(e) => {
                tracing::error!(?e, "internal error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal error".to_string(),
                )
            }
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    /// Ignored (forced to Admin) for the very first user in the table.
    /// Rejected with 422 if a non-bootstrap request tries to self-select Admin.
    pub role: Role,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub role: Role,
    pub approved: bool,
}

#[derive(Debug, Serialize)]
pub struct TokenPairResponse {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct UserRow {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub role: Role,
    pub approved: bool,
}

impl From<UserRow> for UserResponse {
    fn from(row: UserRow) -> Self {
        UserResponse {
            id: row.id,
            email: row.email,
            role: row.role,
            approved: row.approved,
        }
    }
}

// ---------------------------------------------------------------------
// NOTE: FromRequestParts for AuthenticatedUser (JWT decode / validation)
// is implemented in middleware.rs — this file only owns the data shapes,
// per ARCHITECTURE.md's models.rs scope.
// ---------------------------------------------------------------------
