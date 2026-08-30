use std::marker::PhantomData;

use axum::{
    extract::FromRequestParts,
    http::request::Parts,
    RequestPartsExt,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use jsonwebtoken::{decode, DecodingKey, Validation};

use crate::models::{AppError, AuthenticatedUser, Claims, Role};


#[axum::async_trait]
impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AppError::Unauthorized)?;

        let secret = std::env::var("JWT_SECRET").map_err(|_| {
            AppError::Internal(anyhow::anyhow!("JWT_SECRET not set"))
        })?;

        let token_data = decode::<Claims>(
            bearer.token(),
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| AppError::Unauthorized)?;

        let claims = token_data.claims;

        if claims.token_type != "access" {
            return Err(AppError::Unauthorized);
        }

        Ok(AuthenticatedUser {
            id: claims.sub,
            role: claims.role,
        })
    }
}


pub trait RoleMarker {
    const ROLE: Role;
}

pub struct Admin;
impl RoleMarker for Admin {
    const ROLE: Role = Role::Admin;
}

pub struct Trainer;
impl RoleMarker for Trainer {
    const ROLE: Role = Role::Trainer;
}

pub struct Trainee;
impl RoleMarker for Trainee {
    const ROLE: Role = Role::Trainee;
}

pub struct RequireRole<R: RoleMarker> {
    pub user: AuthenticatedUser,
    _marker: PhantomData<R>,
}

#[axum::async_trait]
impl<S, R: RoleMarker> FromRequestParts<S> for RequireRole<R>
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // Reuses AuthenticatedUser's extractor above — a bad/missing
        // token correctly surfaces as 401 from this inner call; only a
        // valid-but-wrong-role token becomes 403 below.
        let user = AuthenticatedUser::from_request_parts(parts, state).await?;
        if user.role != R::ROLE {
            return Err(AppError::Forbidden);
        }
        Ok(RequireRole {
            user,
            _marker: PhantomData,
        })
    }
}

// ---------------------------------------------------------------------
// NOTE: no handler bodies belong in this file. approve_user, tag_course,
// etc. live in handlers/admin.rs, handlers/competency.rs, and so on,
// per ARCHITECTURE.md. This file only owns the extractor + role guards.
// ---------------------------------------------------------------------
