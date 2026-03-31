use std::env;

#[derive(Debug, Clone)]
pub struct WebConfig {
    pub api_base_url: String,
    pub host: String,
    pub port: u16,
}

impl WebConfig {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let env_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(".env");
        dotenvy::from_path(env_path).ok();

        Ok(Self {
            api_base_url: env::var("API_BASE_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:8088".to_string()),
            host: env::var("WEB_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: env::var("WEB_PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .unwrap_or(3000),
        })
    }
}
