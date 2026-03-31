mod api_client;
mod config;
mod middleware;
mod models;
mod routes;

use axum::{
    routing::{get, post},
    Router,
};
use tower_http::services::ServeDir;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use api_client::ApiClient;
use config::WebConfig;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            EnvFilter::new("clalulana_web=debug,tower_http=info")
        }))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = WebConfig::from_env().expect("Falha ao carregar configuração");
    tracing::info!("Iniciando clalulana_web em {}:{}", config.host, config.port);
    tracing::info!("API backend: {}", config.api_base_url);

    // Build shared state
    let api_client = ApiClient::new(&config.api_base_url);

    // Static files
    let static_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/static");

    // Build router
    let app = Router::new()
        // Public routes
        .route("/", get(routes::dashboard::index))
        .route("/login", get(routes::auth::login_page).post(routes::auth::login_submit))
        .route("/register", get(routes::auth::register_page).post(routes::auth::register_submit))
        .route("/logout", post(routes::auth::logout))
        // Authenticated routes
        .route("/dashboard", get(routes::dashboard::dashboard))
        .route("/users", get(routes::users::list_users))
        .route("/users/{id}/edit", get(routes::users::edit_user_page).post(routes::users::edit_user_submit))
        .route("/users/{id}/delete", post(routes::users::delete_user))
        // Static assets
        .nest_service("/static", ServeDir::new(static_dir))
        // Shared state
        .with_state(api_client);

    // Start server
    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Falha ao iniciar servidor");

    tracing::info!("Servidor rodando em http://{}", addr);

    axum::serve(listener, app)
        .await
        .expect("Falha no servidor");
}
