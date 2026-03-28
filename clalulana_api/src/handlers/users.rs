use actix_web::{web, HttpRequest, HttpResponse};
use mediatr::Mediator;
use std::sync::Arc;
use uuid::Uuid;

use crate::cqrs::users::commands::{DeleteUserCommand, UpdateUserCommand};
use crate::cqrs::users::queries::{GetAllUsersQuery, GetCurrentUserQuery, GetUserByIdQuery};
use crate::errors::ApiError;
use crate::middleware::auth::{self, AuthenticatedUser};
use crate::response::ApiResponse;

/// GET /api/v1/users
/// Requires admin role.
pub async fn get_all_users(
    req: HttpRequest,
    mediator: web::Data<Arc<Mediator>>,
    jwt_secret: web::Data<String>,
    query_params: web::Query<PaginationParams>,
) -> Result<HttpResponse, ApiError> {
    let user = authenticate(&req, &jwt_secret)?;
    auth::require_role(&user, "admin")?;

    let query = GetAllUsersQuery {
        limit: query_params.limit.unwrap_or(50).min(100),
        offset: query_params.offset.unwrap_or(0),
    };

    let result = mediator
        .send(query)
        .await
        .map_err(|e| ApiError::InternalError(format!("Mediator error: {}", e)))?;

    match result.0 {
        Ok(users) => Ok(HttpResponse::Ok().json(ApiResponse::success(users))),
        Err(e) => Err(e),
    }
}

/// GET /api/v1/users/me
/// Requires authentication.
pub async fn get_current_user(
    req: HttpRequest,
    mediator: web::Data<Arc<Mediator>>,
    jwt_secret: web::Data<String>,
) -> Result<HttpResponse, ApiError> {
    let user = authenticate(&req, &jwt_secret)?;

    let query = GetCurrentUserQuery { user_id: user.id };

    let result = mediator
        .send(query)
        .await
        .map_err(|e| ApiError::InternalError(format!("Mediator error: {}", e)))?;

    match result.0 {
        Ok(user_response) => Ok(HttpResponse::Ok().json(ApiResponse::success(user_response))),
        Err(e) => Err(e),
    }
}

/// GET /api/v1/users/{id}
/// Requires authentication.
pub async fn get_user_by_id(
    req: HttpRequest,
    mediator: web::Data<Arc<Mediator>>,
    jwt_secret: web::Data<String>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let _user = authenticate(&req, &jwt_secret)?;
    let user_id = path.into_inner();

    let query = GetUserByIdQuery { user_id };

    let result = mediator
        .send(query)
        .await
        .map_err(|e| ApiError::InternalError(format!("Mediator error: {}", e)))?;

    match result.0 {
        Ok(user_response) => Ok(HttpResponse::Ok().json(ApiResponse::success(user_response))),
        Err(e) => Err(e),
    }
}

/// PUT /api/v1/users/{id}
/// Requires authentication (owner or admin).
pub async fn update_user(
    req: HttpRequest,
    mediator: web::Data<Arc<Mediator>>,
    jwt_secret: web::Data<String>,
    path: web::Path<Uuid>,
    body: web::Json<UpdateUserCommand>,
) -> Result<HttpResponse, ApiError> {
    let auth_user = authenticate(&req, &jwt_secret)?;
    let target_id = path.into_inner();

    // Only allow owner or admin to update
    if auth_user.id != target_id && auth_user.role != "admin" {
        return Err(ApiError::Forbidden("Cannot update another user's profile".to_string()));
    }

    let mut command = body.into_inner();
    command.user_id = target_id;

    let result = mediator
        .send(command)
        .await
        .map_err(|e| ApiError::InternalError(format!("Mediator error: {}", e)))?;

    match result.0 {
        Ok(user_response) => Ok(HttpResponse::Ok().json(ApiResponse::success(user_response))),
        Err(e) => Err(e),
    }
}

/// DELETE /api/v1/users/{id}
/// Requires admin role.
pub async fn delete_user(
    req: HttpRequest,
    mediator: web::Data<Arc<Mediator>>,
    jwt_secret: web::Data<String>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let user = authenticate(&req, &jwt_secret)?;
    auth::require_role(&user, "admin")?;

    let target_id = path.into_inner();

    let command = DeleteUserCommand {
        user_id: target_id,
    };

    let result = mediator
        .send(command)
        .await
        .map_err(|e| ApiError::InternalError(format!("Mediator error: {}", e)))?;

    match result.0 {
        Ok(()) => Ok(HttpResponse::NoContent().finish()),
        Err(e) => Err(e),
    }
}

// ============================================================================
// Helpers
// ============================================================================

/// Pagination query parameters.
#[derive(Debug, serde::Deserialize)]
pub struct PaginationParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Extract and validate JWT from request headers.
fn authenticate(req: &HttpRequest, jwt_secret: &str) -> Result<AuthenticatedUser, ApiError> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| ApiError::Unauthorized("Missing Authorization header".to_string()))?;

    let token = auth::extract_bearer_token(auth_header)?;
    let token_data = auth::decode_jwt(token, jwt_secret)?;
    let claims = token_data.claims;

    Ok(AuthenticatedUser {
        id: claims.user_id()?,
        role: claims.role,
    })
}
