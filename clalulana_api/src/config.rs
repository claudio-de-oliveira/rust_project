use std::env;

/// Application configuration loaded from environment variables.
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_expiration: i64,
    pub host: String,
    pub port: u16,
}

impl AppConfig {
    /// Load configuration from environment variables.
    ///
    /// Requires `DATABASE_URL` and `JWT_SECRET` to be set.
    /// Other values have sensible defaults.
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let env_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(".env");
        dotenvy::from_path(env_path).ok();

        Ok(Self {
            database_url: env::var("DATABASE_URL")
                .map_err(|_| "DATABASE_URL must be set")?,
            jwt_secret: env::var("JWT_SECRET")
                .map_err(|_| "JWT_SECRET must be set")?,
            jwt_expiration: env::var("JWT_EXPIRATION")
                .unwrap_or_else(|_| "3600".to_string())
                .parse()
                .unwrap_or(3600),
            host: env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "8088".to_string())
                .parse()
                .unwrap_or(8088),
        })
    }
}
