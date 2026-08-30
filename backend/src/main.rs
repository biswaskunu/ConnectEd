use axum::{routing::{get, post, patch, delete}, Router};
use sqlx::postgres::PgPoolOptions;
use std::env;
use dotenvy::dotenv;

// For tracing
use tower_http::trace::TraceLayer;
use tracing_subscriber;

pub mod models;
pub mod handlers;
pub mod middleware;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

     // for structured logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "iam_platform=debug,tower_http=debug".into())
        )
        .init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let app = Router::new()
        // Public, no auth
        .route("/auth/register", post(handlers::auth::register))
        .route("/auth/login", post(handlers::auth::login))
        .route("/auth/refresh", post(handlers::auth::refresh))

        // Any authenticated user, no role restriction
        .route("/auth/logout", post(handlers::auth::logout))
        .route("/users/me", get(handlers::auth::get_me).patch(handlers::auth::update_me))

        // Trainer-only — role visible in extractor, no body check needed
        .route("/courses", post(handlers::trainer::create_course))
        .route("/courses/:id/questionnaires", post(handlers::trainer::create_questionnaire))
        .route("/library", post(handlers::trainer::upload_library_content))

        // Trainee-only
        .route("/courses/:id/enroll", post(handlers::trainee::enroll))
        .route("/questionnaires/:id/submit", post(handlers::trainee::submit_questionnaire))
        .route("/courses/:id/feedback", post(handlers::trainee::submit_feedback))

        // Admin-only
        .route("/admin/users/pending", get(handlers::admin::list_pending))
        .route("/admin/users/:id/approve", post(handlers::admin::approve_user))
        .route("/admin/users/:id/role", patch(handlers::admin::change_role))
        .route("/admin/dashboard", get(handlers::admin::dashboard))
        .route("/competency-tags", post(handlers::admin::create_tag))

        // Multi-role (Trainer OR Admin) — plain AuthenticatedUser + inline check,
        .route("/courses/:id/enrollments", get(handlers::courses::list_enrollments))
        .route("/questionnaires/:id/results", get(handlers::trainer::view_results))
        .route("/courses/:id/tags", post(handlers::competency::tag_course))
        .route("/courses/:id/suggested-trainers", get(handlers::competency::suggested_trainers))

        .with_state(app_state);
    
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    println!("backend Gateway running on http://127.0.0.1:3000");
    axum::serve(listener, app).await?;

    Ok(())
}