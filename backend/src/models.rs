struct RequireRole<const R: Role>(AuthenticatedUser);

impl<S> FromRequestParts<S> for RequireRole<R> {
    async fn from_request_parts(parts, state) -> Result<Self, Rejection> {
        let user = AuthenticatedUser::from_request_parts(parts, state).await?; // reuse existing logic
        if user.role != R {
            return Err(Rejection::Forbidden);
        }
        Ok(RequireRole(user))
    }
}


// Errors for testing - may change later (for sachi to handle)

pub enum AppError {
    Unauthorized,      // 401 - missing/invalid token
    Forbidden,          // 403 - authenticated, wrong role
    NotFound,           // 404
    UnprocessableEntity(String), // 422 - validation
    Internal(anyhow::Error),     // 500
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized".into()),
            AppError::Forbidden => (StatusCode::FORBIDDEN, "forbidden".into()),
            AppError::NotFound => (StatusCode::NOT_FOUND, "not found".into()),
            AppError::UnprocessableEntity(msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg),
            AppError::Internal(e) => {
                tracing::error!(?e, "internal error");
                (StatusCode::INTERNAL_SERVER_ERROR, "internal error".into())
            }
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}