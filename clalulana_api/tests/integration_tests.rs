// Integration tests for clalulana_api
// These tests verify the complete flow of the application

use clalulana_api::{
    response::ApiResponse,
    domain::user::UserResponse,
};
use serde_json::json;

#[test]
fn test_user_response_serialization() {
    use uuid::Uuid;
    use chrono::Utc;

    let user_response = UserResponse {
        id: Uuid::new_v4(),
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
        role: "user".to_string(),
        is_active: true,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let json = serde_json::to_string(&user_response).unwrap();
    assert!(json.contains("testuser"));
    assert!(json.contains("test@example.com"));
}

#[test]
fn test_api_response_serialization() {
    use uuid::Uuid;
    use chrono::Utc;

    let user_response = UserResponse {
        id: Uuid::new_v4(),
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
        role: "user".to_string(),
        is_active: true,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let api_response = ApiResponse::success(user_response);
    let json = serde_json::to_string(&api_response).unwrap();

    assert!(json.contains("\"success\":true"));
    assert!(json.contains("testuser"));
}

#[test]
fn test_error_response_structures() {
    use clalulana_api::errors::ApiError;

    let errors = vec![
        ApiError::BadRequest("Invalid input".to_string()),
        ApiError::Unauthorized("Missing token".to_string()),
        ApiError::Forbidden("Access denied".to_string()),
        ApiError::NotFound("Resource not found".to_string()),
        ApiError::Conflict("Email already exists".to_string()),
        ApiError::InternalError("Database error".to_string()),
    ];

    for error in errors {
        let msg = error.to_string();
        assert!(!msg.is_empty());
    }
}

#[test]
fn test_create_user_command_validation() {
    use clalulana_api::cqrs::users::commands::CreateUserCommand;

    // Test all fields present
    let json = json!({
        "username": "johndoe",
        "email": "john@example.com",
        "password": "secure123"
    });

    let command: Result<CreateUserCommand, _> = serde_json::from_value(json);
    assert!(command.is_ok());

    // Test missing password
    let json = json!({
        "username": "johndoe",
        "email": "john@example.com"
    });

    let command: Result<CreateUserCommand, _> = serde_json::from_value(json);
    assert!(command.is_err());
}

#[test]
fn test_login_command_validation() {
    use clalulana_api::cqrs::users::commands::LoginCommand;

    let json = json!({
        "email": "john@example.com",
        "password": "secure123"
    });

    let command: Result<LoginCommand, _> = serde_json::from_value(json);
    assert!(command.is_ok());

    // Test missing email
    let json = json!({
        "password": "secure123"
    });

    let command: Result<LoginCommand, _> = serde_json::from_value(json);
    assert!(command.is_err());
}

#[test]
fn test_jwt_claims_structure() {
    use clalulana_api::middleware::auth::Claims;
    use uuid::Uuid;

    let user_id = Uuid::new_v4();
    let claims = Claims {
        sub: user_id.to_string(),
        role: "user".to_string(),
        exp: 9999999999,
        iat: 1234567890,
    };

    // Should be serializable
    let json = serde_json::to_string(&claims).unwrap();
    let deserialized: Claims = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.sub, user_id.to_string());
    assert_eq!(deserialized.role, "user");
}

#[test]
fn test_authenticated_user_structure() {
    use clalulana_api::middleware::auth::AuthenticatedUser;
    use uuid::Uuid;

    let user_id = Uuid::new_v4();
    let user = AuthenticatedUser {
        id: user_id,
        role: "admin".to_string(),
    };

    let cloned = user.clone();
    assert_eq!(user.id, cloned.id);
    assert_eq!(user.role, cloned.role);
}

#[test]
fn test_bearer_token_extraction() {
    use clalulana_api::middleware::auth::extract_bearer_token;

    // Valid token
    let header = "Bearer token123abc";
    assert!(extract_bearer_token(header).is_ok());
    assert_eq!(extract_bearer_token(header).unwrap(), "token123abc");

    // Invalid format
    let header = "Basic token123abc";
    assert!(extract_bearer_token(header).is_err());

    // Missing space
    let header = "Bearertoken123abc";
    assert!(extract_bearer_token(header).is_err());
}

#[test]
fn test_get_all_users_query_pagination() {
    use clalulana_api::cqrs::users::queries::GetAllUsersQuery;

    let query = GetAllUsersQuery {
        limit: 50,
        offset: 100,
    };

    assert_eq!(query.limit, 50);
    assert_eq!(query.offset, 100);
}

#[test]
fn test_type_conversions() {
    use clalulana_api::domain::user::{User, UserResponse};
    use uuid::Uuid;
    use chrono::Utc;

    let user = User {
        id: Uuid::new_v4(),
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
        password_hash: "hashed".to_string(),
        role: "user".to_string(),
        is_active: true,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let response = UserResponse::from(user.clone());
    
    assert_eq!(response.id, user.id);
    assert_eq!(response.username, user.username);
    assert_eq!(response.email, user.email);

    // Ensure password hash is not exposed
    let json = serde_json::to_string(&response).unwrap();
    assert!(!json.contains("password_hash"));
}
