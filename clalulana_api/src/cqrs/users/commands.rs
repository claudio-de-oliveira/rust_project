use async_trait::async_trait;
use chrono::Utc;
use serde::Deserialize;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use jsonwebtoken::{encode, EncodingKey, Header};
use mediatr::{Command, CommandHandler, Request, Result as MediatRResult};

use crate::domain::user::{AuthResponse, User, UserResponse};
use crate::errors::ApiError;
use crate::middleware::auth::Claims;

// ============================================================================
// CreateUserCommand
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateUserCommand {
    pub username: String,
    pub email: String,
    pub password: String,
}

pub struct CreateUserResult(pub std::result::Result<UserResponse, ApiError>);

impl Request for CreateUserCommand {
    type Response = CreateUserResult;
}

impl Command for CreateUserCommand {}

pub struct CreateUserHandler {
    pub pool: Arc<PgPool>,
}

#[async_trait]
impl CommandHandler<CreateUserCommand> for CreateUserHandler {
    async fn handle(&self, command: CreateUserCommand) -> MediatRResult<CreateUserResult> {
        let result = self.execute(command).await;
        Ok(CreateUserResult(result))
    }
}

impl CreateUserHandler {
    async fn execute(&self, command: CreateUserCommand) -> std::result::Result<UserResponse, ApiError> {
        // Validate input
        if command.username.trim().is_empty() {
            return Err(ApiError::BadRequest("Username is required".to_string()));
        }
        if command.email.trim().is_empty() {
            return Err(ApiError::BadRequest("Email is required".to_string()));
        }
        if command.password.len() < 8 {
            return Err(ApiError::BadRequest(
                "Password must be at least 8 characters".to_string(),
            ));
        }

        // Hash password
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(command.password.as_bytes(), &salt)
            .map_err(|e| ApiError::InternalError(format!("Failed to hash password: {}", e)))?
            .to_string();

        let now = Utc::now();

        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (id, username, email, password_hash, role, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(&command.username)
        .bind(&command.email)
        .bind(&password_hash)
        .bind("user")
        .bind(true)
        .bind(now)
        .bind(now)
        .fetch_one(self.pool.as_ref())
        .await
        .map_err(ApiError::from)?;

        Ok(UserResponse::from(user))
    }
}

// ============================================================================
// LoginCommand
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct LoginCommand {
    pub email: String,
    pub password: String,
}

pub struct LoginResult(pub std::result::Result<AuthResponse, ApiError>);

impl Request for LoginCommand {
    type Response = LoginResult;
}

impl Command for LoginCommand {}

pub struct LoginHandler {
    pub pool: Arc<PgPool>,
    pub jwt_secret: String,
    pub jwt_expiration: i64,
}

#[async_trait]
impl CommandHandler<LoginCommand> for LoginHandler {
    async fn handle(&self, command: LoginCommand) -> MediatRResult<LoginResult> {
        let result = self.execute(command).await;
        Ok(LoginResult(result))
    }
}

impl LoginHandler {
    async fn execute(&self, command: LoginCommand) -> std::result::Result<AuthResponse, ApiError> {
        // Find user by email
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE email = $1 AND is_active = true",
        )
        .bind(&command.email)
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::Unauthorized("Invalid credentials".to_string()))?;

        // Verify password
        let parsed_hash = PasswordHash::new(&user.password_hash)
            .map_err(|e| ApiError::InternalError(format!("Failed to parse hash: {}", e)))?;

        Argon2::default()
            .verify_password(command.password.as_bytes(), &parsed_hash)
            .map_err(|_| ApiError::Unauthorized("Invalid credentials".to_string()))?;

        // Generate JWT
        let now = Utc::now().timestamp();
        let claims = Claims {
            sub: user.id.to_string(),
            role: user.role.clone(),
            exp: (now + self.jwt_expiration) as usize,
            iat: now as usize,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .map_err(|e| ApiError::InternalError(format!("Failed to create token: {}", e)))?;

        Ok(AuthResponse {
            token,
            token_type: "Bearer".to_string(),
            expires_in: self.jwt_expiration,
            user: UserResponse::from(user),
        })
    }
}

// ============================================================================
// UpdateUserCommand
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct UpdateUserCommand {
    #[serde(skip)]
    pub user_id: Uuid,
    pub username: Option<String>,
    pub email: Option<String>,
}

pub struct UpdateUserResult(pub std::result::Result<UserResponse, ApiError>);

impl Request for UpdateUserCommand {
    type Response = UpdateUserResult;
}

impl Command for UpdateUserCommand {}

pub struct UpdateUserHandler {
    pub pool: Arc<PgPool>,
}

#[async_trait]
impl CommandHandler<UpdateUserCommand> for UpdateUserHandler {
    async fn handle(&self, command: UpdateUserCommand) -> MediatRResult<UpdateUserResult> {
        let result = self.execute(command).await;
        Ok(UpdateUserResult(result))
    }
}

impl UpdateUserHandler {
    async fn execute(&self, command: UpdateUserCommand) -> std::result::Result<UserResponse, ApiError> {
        let now = Utc::now();

        let user = sqlx::query_as::<_, User>(
            r#"
            UPDATE users
            SET username   = COALESCE($1, username),
                email      = COALESCE($2, email),
                updated_at = $3
            WHERE id = $4 AND is_active = true
            RETURNING *
            "#,
        )
        .bind(&command.username)
        .bind(&command.email)
        .bind(now)
        .bind(command.user_id)
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

        Ok(UserResponse::from(user))
    }
}

// ============================================================================
// DeleteUserCommand
// ============================================================================

#[derive(Debug)]
pub struct DeleteUserCommand {
    pub user_id: Uuid,
}

pub struct DeleteUserResult(pub std::result::Result<(), ApiError>);

impl Request for DeleteUserCommand {
    type Response = DeleteUserResult;
}

impl Command for DeleteUserCommand {}

pub struct DeleteUserHandler {
    pub pool: Arc<PgPool>,
}

#[async_trait]
impl CommandHandler<DeleteUserCommand> for DeleteUserHandler {
    async fn handle(&self, command: DeleteUserCommand) -> MediatRResult<DeleteUserResult> {
        let result = self.execute(command).await;
        Ok(DeleteUserResult(result))
    }
}

impl DeleteUserHandler {
    async fn execute(&self, command: DeleteUserCommand) -> std::result::Result<(), ApiError> {
        let rows_affected = sqlx::query(
            "UPDATE users SET is_active = false, updated_at = NOW() WHERE id = $1 AND is_active = true",
        )
        .bind(command.user_id)
        .execute(self.pool.as_ref())
        .await
        .map_err(ApiError::from)?
        .rows_affected();

        if rows_affected == 0 {
            return Err(ApiError::NotFound("User not found".to_string()));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_create_user_command_deserialization() {
        let json = json!({
            "username": "johndoe",
            "email": "john@example.com",
            "password": "secure_password123"
        });

        let command: CreateUserCommand = serde_json::from_value(json).unwrap();
        assert_eq!(command.username, "johndoe");
        assert_eq!(command.email, "john@example.com");
        assert_eq!(command.password, "secure_password123");
    }

    #[test]
    fn test_create_user_command_missing_fields() {
        let json = json!({
            "username": "johndoe",
            "email": "john@example.com"
        });

        let result: Result<CreateUserCommand, _> = serde_json::from_value(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_login_command_deserialization() {
        let json = json!({
            "email": "john@example.com",
            "password": "secure_password123"
        });

        let command: LoginCommand = serde_json::from_value(json).unwrap();
        assert_eq!(command.email, "john@example.com");
        assert_eq!(command.password, "secure_password123");
    }

    #[test]
    fn test_login_command_missing_email() {
        let json = json!({
            "password": "secure_password123"
        });

        let result: Result<LoginCommand, _> = serde_json::from_value(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_user_command_with_special_characters() {
        let json = json!({
            "username": "user@name",
            "email": "test+tag@example.com",
            "password": "P@ssw0rd!123"
        });

        let command: CreateUserCommand = serde_json::from_value(json).unwrap();
        assert_eq!(command.username, "user@name");
        assert_eq!(command.email, "test+tag@example.com");
    }

    #[test]
    fn test_update_user_command_deserialization() {
        let json = json!({
            "username": "newname",
            "email": "new@example.com"
        });

        let command: UpdateUserCommand = serde_json::from_value(json).unwrap();
        assert_eq!(command.username, Some("newname".to_string()));
        assert_eq!(command.email, Some("new@example.com".to_string()));
    }

    #[test]
    fn test_update_user_command_partial() {
        let json = json!({
            "username": "newname"
        });

        let command: UpdateUserCommand = serde_json::from_value(json).unwrap();
        assert_eq!(command.username, Some("newname".to_string()));
        assert!(command.email.is_none());
    }

    #[test]
    fn test_delete_user_command_creation() {
        let user_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let command = DeleteUserCommand { user_id };
        assert_eq!(command.user_id, user_id);
    }
}
