use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use mediatr::{Query, QueryHandler, Request, Result as MediatRResult};

use crate::domain::user::{User, UserResponse};
use crate::errors::ApiError;

// ============================================================================
// GetUserByIdQuery
// ============================================================================

#[derive(Debug)]
pub struct GetUserByIdQuery {
    pub user_id: Uuid,
}

pub struct GetUserByIdResult(pub std::result::Result<UserResponse, ApiError>);

impl Request for GetUserByIdQuery {
    type Response = GetUserByIdResult;
}

impl Query for GetUserByIdQuery {}

pub struct GetUserByIdHandler {
    pub pool: Arc<PgPool>,
}

#[async_trait]
impl QueryHandler<GetUserByIdQuery> for GetUserByIdHandler {
    async fn handle(&self, query: GetUserByIdQuery) -> MediatRResult<GetUserByIdResult> {
        let result = self.execute(query).await;
        Ok(GetUserByIdResult(result))
    }
}

impl GetUserByIdHandler {
    async fn execute(&self, query: GetUserByIdQuery) -> std::result::Result<UserResponse, ApiError> {
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE id = $1 AND is_active = true",
        )
        .bind(query.user_id)
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

        Ok(UserResponse::from(user))
    }
}

// ============================================================================
// GetAllUsersQuery
// ============================================================================

#[derive(Debug)]
pub struct GetAllUsersQuery {
    pub limit: i64,
    pub offset: i64,
}

pub struct GetAllUsersResult(pub std::result::Result<Vec<UserResponse>, ApiError>);

impl Request for GetAllUsersQuery {
    type Response = GetAllUsersResult;
}

impl Query for GetAllUsersQuery {}

pub struct GetAllUsersHandler {
    pub pool: Arc<PgPool>,
}

#[async_trait]
impl QueryHandler<GetAllUsersQuery> for GetAllUsersHandler {
    async fn handle(&self, query: GetAllUsersQuery) -> MediatRResult<GetAllUsersResult> {
        let result = self.execute(query).await;
        Ok(GetAllUsersResult(result))
    }
}

impl GetAllUsersHandler {
    async fn execute(&self, query: GetAllUsersQuery) -> std::result::Result<Vec<UserResponse>, ApiError> {
        let users = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE is_active = true ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(query.limit)
        .bind(query.offset)
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(ApiError::from)?;

        Ok(users.into_iter().map(UserResponse::from).collect())
    }
}

// ============================================================================
// GetCurrentUserQuery
// ============================================================================

#[derive(Debug)]
pub struct GetCurrentUserQuery {
    pub user_id: Uuid,
}

pub struct GetCurrentUserResult(pub std::result::Result<UserResponse, ApiError>);

impl Request for GetCurrentUserQuery {
    type Response = GetCurrentUserResult;
}

impl Query for GetCurrentUserQuery {}

pub struct GetCurrentUserHandler {
    pub pool: Arc<PgPool>,
}

#[async_trait]
impl QueryHandler<GetCurrentUserQuery> for GetCurrentUserHandler {
    async fn handle(&self, query: GetCurrentUserQuery) -> MediatRResult<GetCurrentUserResult> {
        let result = self.execute(query).await;
        Ok(GetCurrentUserResult(result))
    }
}

impl GetCurrentUserHandler {
    async fn execute(&self, query: GetCurrentUserQuery) -> std::result::Result<UserResponse, ApiError> {
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE id = $1 AND is_active = true",
        )
        .bind(query.user_id)
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

        Ok(UserResponse::from(user))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_user_by_id_query_creation() {
        let user_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let query = GetUserByIdQuery { user_id };

        assert_eq!(query.user_id, user_id);
    }

    #[test]
    fn test_get_user_by_id_query_debug() {
        let user_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let query = GetUserByIdQuery { user_id };

        let debug_str = format!("{:?}", query);
        assert!(debug_str.contains("GetUserByIdQuery"));
    }

    #[test]
    fn test_get_all_users_query_creation() {
        let query = GetAllUsersQuery {
            limit: 50,
            offset: 0,
        };

        assert_eq!(query.limit, 50);
        assert_eq!(query.offset, 0);
    }

    #[test]
    fn test_get_all_users_query_with_pagination() {
        let query = GetAllUsersQuery {
            limit: 25,
            offset: 100,
        };

        assert_eq!(query.limit, 25);
        assert_eq!(query.offset, 100);
    }

    #[test]
    fn test_get_all_users_query_debug() {
        let query = GetAllUsersQuery {
            limit: 50,
            offset: 10,
        };

        let debug_str = format!("{:?}", query);
        assert!(debug_str.contains("GetAllUsersQuery"));
    }

    #[test]
    fn test_get_current_user_query_creation() {
        let user_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let query = GetCurrentUserQuery { user_id };

        assert_eq!(query.user_id, user_id);
    }

    #[test]
    fn test_get_current_user_query_debug() {
        let user_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let query = GetCurrentUserQuery { user_id };

        let debug_str = format!("{:?}", query);
        assert!(debug_str.contains("GetCurrentUserQuery"));
    }

    #[test]
    fn test_get_user_by_id_query_with_multiple_ids() {
        let id1 = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let id2 = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap();

        let query1 = GetUserByIdQuery { user_id: id1 };
        let query2 = GetUserByIdQuery { user_id: id2 };

        assert_ne!(query1.user_id, query2.user_id);
    }
}
