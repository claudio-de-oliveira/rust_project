use actix_web::{web, HttpResponse};
use mediatr::Mediator;
use std::sync::Arc;

use crate::cqrs::users::commands::{CreateUserCommand, LoginCommand};
use crate::errors::ApiError;
use crate::response::ApiResponse;

/// POST /api/v1/auth/register
pub async fn register(
    mediator: web::Data<Arc<Mediator>>,
    body: web::Json<CreateUserCommand>,
) -> Result<HttpResponse, ApiError> {
    let command = body.into_inner();

    let result = mediator
        .send(command)
        .await
        .map_err(|e| ApiError::InternalError(format!("Mediator error: {}", e)))?;

    match result.0 {
        Ok(user) => Ok(HttpResponse::Created().json(ApiResponse::success(user))),
        Err(e) => Err(e),
    }
}

/// POST /api/v1/auth/login
pub async fn login(
    mediator: web::Data<Arc<Mediator>>,
    body: web::Json<LoginCommand>,
) -> Result<HttpResponse, ApiError> {
    let command = body.into_inner();

    let result = mediator
        .send(command)
        .await
        .map_err(|e| ApiError::InternalError(format!("Mediator error: {}", e)))?;

    match result.0 {
        Ok(auth_response) => Ok(HttpResponse::Ok().json(ApiResponse::success(auth_response))),
        Err(e) => Err(e),
    }
}
