use actix_web::HttpResponse;
use serde::Serialize;

use crate::response::ApiResponse;

#[derive(Debug, Serialize)]
pub struct HealthStatus {
    pub status: String,
    pub version: String,
}

/// GET /api/v1/health
pub async fn health_check() -> HttpResponse {
    let health = HealthStatus {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    };

    HttpResponse::Ok().json(ApiResponse::success(health))
}
