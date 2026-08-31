use axum::{
    routing::{delete, get, patch, post},
    Router,
};
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::env;

use tower_http::trace::TraceLayer;
use tracing_subscriber;

pub mod handlers;
pub mod middleware;
pub mod models;

#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::PgPool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    // structured logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "connected=debug,tower_http=debug".into()),
        )
        .init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let app_state = AppState { pool };

    let app = Router::new()
        // Public, no auth
        .route("/auth/register", post(handlers::auth::register))
        .route("/auth/login", post(handlers::auth::login))
        .route("/auth/refresh", post(handlers::auth::refresh))

        // Any authenticated user, no role restriction
        .route("/auth/logout", post(handlers::auth::logout))
        .route("/users/me",get(handlers::auth::get_me).patch(handlers::auth::update_me),)


        // The following paths will be added eventually by GOD, cuz I cant


        // Trainer-only — role visible in extractor, no body check needed
        // .route("/courses", post(handlers::trainer::create_course))
        // .route("/courses/:id/questionnaires",post(handlers::trainer::create_questionnaire),)
        // .route("/library", post(handlers::trainer::upload_library_content))
        
        // If this grows, promote to its own handlers/courses.rs and
        // update ARCHITECTURE.md's module list to match.
        // .route("/courses/:id/enrollments",get(handlers::trainer::list_enrollments),)
        // .route("/questionnaires/:id/results",get(handlers::trainer::view_results),)

        // Trainee-only
        // .route("/courses/:id/enroll", post(handlers::trainee::enroll))
        // .route("/questionnaires/:id/submit",post(handlers::trainee::submit_questionnaire),)
        // .route("/courses/:id/feedback",post(handlers::trainee::submit_feedback),)

        // Admin-only
        // .route("/admin/users/pending", get(handlers::admin::list_pending))
        // .route("/admin/users/:id/approve",post(handlers::admin::approve_user),)
        // .route("/admin/users/:id/role",patch(handlers::admin::change_role),)
        // .route("/admin/dashboard", get(handlers::admin::dashboard))
        // .route("/competency-tags", post(handlers::admin::create_tag))


        // Multi-role (Trainer OR Admin) — plain AuthenticatedUser + inline
        // check inside the handler, per the minority-case decision.
        // .route("/courses/:id/tags",post(handlers::competency::tag_course),)
        // .route("/courses/:id/suggested-trainers",get(handlers::competency::suggested_trainers),)
        
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    println!("backend gateway running on http://127.0.0.1:3000");
    axum::serve(listener, app).await?;

    Ok(())
}
