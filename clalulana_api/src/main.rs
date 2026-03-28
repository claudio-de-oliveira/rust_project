mod config;
mod db;
mod errors;
mod response;
mod domain;
mod cqrs;
mod middleware;
mod handlers;
mod routes;

use actix_cors::Cors;
use actix_web::{web, App, HttpServer, ResponseError};
use mediatr::MediatorBuilder;
use sqlx::PgPool;
use std::sync::Arc;
use tracing_actix_web::TracingLogger;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::cqrs::users::commands::*;
use crate::cqrs::users::queries::*;
use crate::middleware::performance::PerformanceMonitor;

/// Application state shared across all handlers.
pub struct AppState {
    pub mediator: Arc<mediatr::Mediator>,
    pub config: config::AppConfig,
}

/// Build the Mediator with all CQRS handlers, injecting the database pool.
fn build_mediator(pool: Arc<PgPool>, config: &config::AppConfig) -> mediatr::Mediator {
    MediatorBuilder::new()
        // Commands
        .register_command_handler(CreateUserHandler {
            pool: pool.clone(),
        })
        .register_command_handler(LoginHandler {
            pool: pool.clone(),
            jwt_secret: config.jwt_secret.clone(),
            jwt_expiration: config.jwt_expiration,
        })
        .register_command_handler(UpdateUserHandler {
            pool: pool.clone(),
        })
        .register_command_handler(DeleteUserHandler {
            pool: pool.clone(),
        })
        // Queries
        .register_query_handler(GetUserByIdHandler {
            pool: pool.clone(),
        })
        .register_query_handler(GetAllUsersHandler {
            pool: pool.clone(),
        })
        .register_query_handler(GetCurrentUserHandler {
            pool: pool.clone(),
        })
        .build()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load configuration
    let config = config::AppConfig::from_env().expect("Failed to load configuration");

    // Initialize tracing
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            EnvFilter::new("clalulana_api=debug,actix_web=info,sqlx=warn")
        }))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!(
        "Starting clalulana_api v{} on {}:{}",
        env!("CARGO_PKG_VERSION"),
        config.host,
        config.port
    );

    // Initialize database pool and run migrations
    let pool = db::init_db(&config.database_url)
        .await
        .expect("Failed to connect to database");

    let pool = Arc::new(pool);

    // Build the Mediator with all CQRS handlers
    let mediator = Arc::new(build_mediator(pool.clone(), &config));

    let host = config.host.clone();
    let port = config.port;
    let jwt_secret = config.jwt_secret.clone();

    // Start HTTP server
    HttpServer::new(move || {
        println!("Executando Start HTTP server ...");

        // CORS configuration
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            // Middleware
            .wrap(TracingLogger::default())
            .wrap(PerformanceMonitor)
            .wrap(cors)
            // Shared state
            .app_data(web::Data::new(mediator.clone()))
            .app_data(web::Data::new(jwt_secret.clone()))
            .app_data(web::Data::new(pool.clone()))
            // JSON configuration
            .app_data(
                web::JsonConfig::default()
                    .limit(4096)
                    .error_handler(|err, _req| {
                        let error = errors::ApiError::BadRequest(format!(
                            "Invalid JSON: {}",
                            err
                        ));
                        actix_web::error::InternalError::from_response(
                            err,
                            error.error_response(),
                        )
                        .into()
                    }),
            )
            // Routes
            .configure(routes::configure_routes)
    })
    .bind((host.as_str(), port))?
    .run()
    .await
}
