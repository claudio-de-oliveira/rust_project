use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Full user model (maps to the `users` table).
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub role: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Public user DTO (excludes sensitive fields like `password_hash`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub role: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            role: user.role,
            is_active: user.is_active,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}

/// JWT authentication response.
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub user: UserResponse,
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_user_to_response_conversion() {
        let user = User {
            id: Uuid::new_v4(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password_hash: "hashed_password".to_string(),
            role: "user".to_string(),
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let response = UserResponse::from(user.clone());

        assert_eq!(response.id, user.id);
        assert_eq!(response.username, user.username);
        assert_eq!(response.email, user.email);
        assert_eq!(response.role, user.role);
        assert_eq!(response.is_active, user.is_active);
    }

    #[test]
    fn test_user_response_excludes_password_hash() {
        let user = User {
            id: Uuid::new_v4(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password_hash: "super_secret_hash".to_string(),
            role: "user".to_string(),
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let response = UserResponse::from(user);

        let json = serde_json::to_string(&response).unwrap();
        assert!(!json.contains("password_hash"));
        assert!(!json.contains("super_secret_hash"));
    }

    #[test]
    fn test_auth_response_serialization() {
        let user_id = Uuid::new_v4();
        let user_response = UserResponse {
            id: user_id,
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            role: "user".to_string(),
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let auth_response = AuthResponse {
            token: "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9".to_string(),
            token_type: "Bearer".to_string(),
            expires_in: 3600,
            user: user_response,
        };

        let json = serde_json::to_string(&auth_response).unwrap();
        assert!(json.contains("token"));
        assert!(json.contains("Bearer"));
    }

    #[test]
    fn test_user_response_contains_required_fields() {
        let user_id = Uuid::new_v4();
        let now = Utc::now();

        let response = UserResponse {
            id: user_id,
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            role: "user".to_string(),
            is_active: true,
            created_at: now,
            updated_at: now,
        };

        assert_eq!(response.id, user_id);
        assert_eq!(response.username, "testuser");
        assert_eq!(response.email, "test@example.com");
        assert_eq!(response.role, "user");
        assert!(response.is_active);
    }
}
