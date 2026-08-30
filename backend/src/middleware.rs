pub struct RequireRole<R: RoleMarker> {
    pub user: AuthenticatedUser,
    _marker: PhantomData<R>,
}
pub trait RoleMarker {
    const ROLE: Role;
}
// single role handler
async fn approve_user(
    RequireRole::<Admin> { user, .. }: RequireRole<Admin>,
    State(pool): State<PgPool>,
    Path(target_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    sqlx::query!("UPDATE users SET approved = true WHERE id = $1", target_id)
        .execute(&pool)
        .await
        .map_err(AppError::Internal)?;
    Ok(StatusCode::OK)
}
// multi role handler
async fn tag_course(
    user: AuthenticatedUser,
    State(pool): State<PgPool>,
    Path(course_id): Path<Uuid>,
    Json(body): Json<TagCourseRequest>,
) -> Result<StatusCode, AppError> {
    if !matches!(user.role, Role::Trainer | Role::Admin) {
        return Err(AppError::Forbidden);
    }
    // ... proceed
}




pub struct Admin;
impl RoleMarker for Admin {
    const ROLE: Role = Role::Admin;
}
// same for Trainer, Trainee

impl<S, R: RoleMarker> FromRequestParts<S> for RequireRole<R>
where
    AuthenticatedUser: FromRequestParts<S>,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let user = AuthenticatedUser::from_request_parts(parts, state).await?;
        if user.role != R::ROLE {
            return Err(AppError::Forbidden);
        }
        Ok(RequireRole { user, _marker: PhantomData })
    }
}