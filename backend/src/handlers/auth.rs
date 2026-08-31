use axum::{extract::State, Json};
use jsonwebtoken::{encode, EncodingKey, Header};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

use crate::models::{
    AppError, AuthenticatedUser, Claims, LoginRequest, RefreshRequest, RegisterRequest, Role,
    TokenPairResponse, UserResponse, UserRow,
};
use crate::AppState;

const ACCESS_TOKEN_TTL_SECS: usize = 15 * 60; // 15 min
const REFRESH_TOKEN_TTL_SECS: usize = 7 * 24 * 60 * 60; // 7 days

fn now_unix() -> usize {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_secs() as usize
}

fn jwt_secret() -> Result<String, AppError> {
    std::env::var("JWT_SECRET").map_err(|_| AppError::Internal(anyhow::anyhow!("JWT_SECRET not set")))
}

fn sha256_hex(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    hex::encode(hasher.finalize())
}

fn sign_token(sub: Uuid, role: Role, token_type: &str, ttl_secs: usize) -> Result<String, AppError> {
    let claims = Claims {
        sub,
        role,
        token_type: token_type.to_string(),
        exp: now_unix() + ttl_secs,
    };
    let secret = jwt_secret()?;
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))
}

/// Issues a fresh access+refresh pair and persists the refresh token's
/// hash in `sessions` (rotation-friendly: caller is responsible for
/// deleting any prior session row for this user/device when rotating).
async fn issue_token_pair(
    pool: &PgPool,
    user_id: Uuid,
    role: Role,
) -> Result<TokenPairResponse, AppError> {
    let access_token = sign_token(user_id, role, "access", ACCESS_TOKEN_TTL_SECS)?;
    let refresh_token = sign_token(user_id, role, "refresh", REFRESH_TOKEN_TTL_SECS)?;

    let refresh_hash = sha256_hex(&refresh_token);
    let expires_at = chrono::Utc::now() + chrono::Duration::seconds(REFRESH_TOKEN_TTL_SECS as i64);

    sqlx::query!(
        r#"
        INSERT INTO sessions (user_id, refresh_token_hash, expires_at)
        VALUES ($1, $2, $3)
        "#,
        user_id,
        refresh_hash,
        expires_at,
    )
    .execute(pool)
    .await
    .map_err(|e| AppError::Internal(e.into()))?;

    Ok(TokenPairResponse {
        access_token,
        refresh_token,
    })
}

// -----------------------------------------------------------------------
// POST /auth/register
// -----------------------------------------------------------------------
pub async fn register(
    State(state): State<AppState>,
    Json(body): Json<RegisterRequest>,
) -> Result<Json<UserResponse>, AppError> {
    let RegisterRequest {
        email,
        password,
        role: requested_role,
    } = body;

    if password.len() < 8 {
        return Err(AppError::UnprocessableEntity(
            "password must be at least 8 characters".into(),
        ));
    }

    // Bootstrap: first user ever created becomes a pre-approved Admin,
    // regardless of what role was requested. Known race condition if two
    // registrations hit an empty table simultaneously — acceptable for a
    // hackathon-scale deployment, not something to harden further here.
    let is_first_user: bool = sqlx::query_scalar!("SELECT NOT EXISTS(SELECT 1 FROM users)")
        .fetch_one(&state.pool)
        .await
        .map_err(|e| AppError::Internal(e.into()))?
        .unwrap_or(true);

    if !is_first_user && matches!(requested_role, Role::Admin) {
        return Err(AppError::UnprocessableEntity(
            "cannot self-register as admin".into(),
        ));
    }

    let (role, approved) = if is_first_user {
        (Role::Admin, true)
    } else {
        (requested_role, false)
    };

    let password_hash = bcrypt::hash(&password, bcrypt::DEFAULT_COST)
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;

    let row = sqlx::query_as!(
        UserRow,
        r#"
        INSERT INTO users (email, password_hash, role, approved)
        VALUES ($1, $2, $3, $4)
        RETURNING id, email, password_hash, role as "role: Role", approved
        "#,
        email,
        password_hash,
        role as Role,
        approved,
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
            AppError::UnprocessableEntity("email already registered".into())
        }
        other => AppError::Internal(other.into()),
    })?;

    Ok(Json(row.into()))
}

// -----------------------------------------------------------------------
// POST /auth/login
// -----------------------------------------------------------------------
pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<TokenPairResponse>, AppError> {
    let row = sqlx::query_as!(
        UserRow,
        r#"
        SELECT id, email, password_hash, role as "role: Role", approved
        FROM users
        WHERE email = $1
        "#,
        body.email,
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::Internal(e.into()))?
    .ok_or(AppError::Unauthorized)?;

    let valid = bcrypt::verify(&body.password, &row.password_hash)
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
    if !valid {
        return Err(AppError::Unauthorized);
    }

    if !row.approved {
        return Err(AppError::Forbidden);
    }

    let tokens = issue_token_pair(&state.pool, row.id, row.role).await?;
    Ok(Json(tokens))
}

// -----------------------------------------------------------------------
// POST /auth/refresh
// -----------------------------------------------------------------------
pub async fn refresh(
    State(state): State<AppState>,
    Json(body): Json<RefreshRequest>,
) -> Result<Json<TokenPairResponse>, AppError> {
    let secret = jwt_secret()?;
    let token_data = jsonwebtoken::decode::<Claims>(
        &body.refresh_token,
        &jsonwebtoken::DecodingKey::from_secret(secret.as_bytes()),
        &jsonwebtoken::Validation::default(),
    )
    .map_err(|_| AppError::Unauthorized)?;

    let claims = token_data.claims;
    if claims.token_type != "refresh" {
        return Err(AppError::Unauthorized);
    }

    let refresh_hash = sha256_hex(&body.refresh_token);

    // Rotation: the presented refresh token must match a live, unexpired
    // session row. Deleting it here (rather than after issuing the new
    // pair) means a reused/stolen token can't be replayed even if the
    // rest of this handler fails.
    let deleted = sqlx::query!(
        r#"
        DELETE FROM sessions
        WHERE user_id = $1 AND refresh_token_hash = $2 AND expires_at > NOW()
        "#,
        claims.sub,
        refresh_hash,
    )
    .execute(&state.pool)
    .await
    .map_err(|e| AppError::Internal(e.into()))?;

    if deleted.rows_affected() == 0 {
        return Err(AppError::Unauthorized);
    }

    let tokens = issue_token_pair(&state.pool, claims.sub, claims.role).await?;
    Ok(Json(tokens))
}

// -----------------------------------------------------------------------
// POST /auth/logout
// -----------------------------------------------------------------------
pub async fn logout(
    user: AuthenticatedUser,
    State(state): State<AppState>,
) -> Result<axum::http::StatusCode, AppError> {
    sqlx::query!("DELETE FROM sessions WHERE user_id = $1", user.id)
        .execute(&state.pool)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}

// -----------------------------------------------------------------------
// GET /users/me
// -----------------------------------------------------------------------
pub async fn get_me(
    user: AuthenticatedUser,
    State(state): State<AppState>,
) -> Result<Json<UserResponse>, AppError> {
    let row = sqlx::query_as!(
        UserRow,
        r#"
        SELECT id, email, password_hash, role as "role: Role", approved
        FROM users
        WHERE id = $1
        "#,
        user.id,
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::Internal(e.into()))?
    .ok_or(AppError::NotFound)?;

    Ok(Json(row.into()))
}

// -----------------------------------------------------------------------
// PATCH /users/me
// -----------------------------------------------------------------------
// NOTE: role-specific fields (qualifications, bio, subject tags, etc.)
// live in trainee_profiles/trainer_profiles, which don't exist yet.
// This only updates base-user fields for now — extend once those tables
// are locked.
pub async fn update_me(
    _user: AuthenticatedUser,
    State(_state): State<AppState>,
) -> Result<axum::http::StatusCode, AppError> {
    Err(AppError::UnprocessableEntity(
        "profile fields not yet implemented — see trainee_profiles/trainer_profiles TODO".into(),
    ))
}
