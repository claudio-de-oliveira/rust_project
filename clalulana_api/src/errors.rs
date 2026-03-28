use actix_web::{HttpResponse, ResponseError};
use serde::Serialize;

/// Unified API error type.
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Internal server error: {0}")]
    InternalError(String),
}

/// JSON error envelope returned to clients.
#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: ErrorBody,
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    code: u16,
    message: String,
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        let (status, message) = match self {
            ApiError::BadRequest(msg) => (actix_web::http::StatusCode::BAD_REQUEST, msg.clone()),
            ApiError::Unauthorized(msg) => (actix_web::http::StatusCode::UNAUTHORIZED, msg.clone()),
            ApiError::Forbidden(msg) => (actix_web::http::StatusCode::FORBIDDEN, msg.clone()),
            ApiError::NotFound(msg) => (actix_web::http::StatusCode::NOT_FOUND, msg.clone()),
            ApiError::Conflict(msg) => (actix_web::http::StatusCode::CONFLICT, msg.clone()),
            ApiError::InternalError(msg) => {
                tracing::error!("Internal error: {}", msg);
                (
                    actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "An internal error occurred".to_string(),
                )
            }
        };

        HttpResponse::build(status).json(ErrorResponse {
            error: ErrorBody {
                code: status.as_u16(),
                message,
            },
        })
    }
}

// Conversions from common error types
impl From<sqlx::Error> for ApiError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => ApiError::NotFound("Resource not found".to_string()),
            sqlx::Error::Database(db_err) => {
                if let Some(code) = db_err.code() {
                    // PostgreSQL unique violation
                    if code == "23505" {
                        return ApiError::Conflict("Resource already exists".to_string());
                    }
                }
                ApiError::InternalError(db_err.to_string())
            }
            _ => ApiError::InternalError(err.to_string()),
        }
    }
}

impl From<jsonwebtoken::errors::Error> for ApiError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        ApiError::Unauthorized(format!("Invalid token: {}", err))
    }
}

impl From<mediatr::Error> for ApiError {
    fn from(err: mediatr::Error) -> Self {
        ApiError::InternalError(format!("Mediator error: {}", err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bad_request_error() {
        let error = ApiError::BadRequest("Invalid input".to_string());
        assert_eq!(error.to_string(), "Bad request: Invalid input");
    }

    #[test]
    fn test_unauthorized_error() {
        let error = ApiError::Unauthorized("Missing token".to_string());
        assert_eq!(error.to_string(), "Unauthorized: Missing token");
    }

    #[test]
    fn test_forbidden_error() {
        let error = ApiError::Forbidden("Access denied".to_string());
        assert_eq!(error.to_string(), "Forbidden: Access denied");
    }

    #[test]
    fn test_not_found_error() {
        let error = ApiError::NotFound("User not found".to_string());
        assert_eq!(error.to_string(), "Not found: User not found");
    }

    #[test]
    fn test_conflict_error() {
        let error = ApiError::Conflict("Email already exists".to_string());
        assert_eq!(error.to_string(), "Conflict: Email already exists");
    }

    #[test]
    fn test_internal_error() {
        let error = ApiError::InternalError("Database connection failed".to_string());
        assert_eq!(
            error.to_string(),
            "Internal server error: Database connection failed"
        );
    }

    #[test]
    fn test_error_debug_output() {
        let error = ApiError::BadRequest("Test error".to_string());
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("BadRequest"));
        assert!(debug_str.contains("Test error"));
    }
}
