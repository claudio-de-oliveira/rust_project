use actix_web::{dev::ServiceRequest, HttpMessage};
use jsonwebtoken::{decode, DecodingKey, TokenData, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::errors::ApiError;

/// JWT claims embedded in every token.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID as UUID string)
    pub sub: String,
    /// User role
    pub role: String,
    /// Expiration time (Unix timestamp)
    pub exp: usize,
    /// Issued at (Unix timestamp)
    pub iat: usize,
}

impl Claims {
    /// Parse `sub` back into a `Uuid`.
    pub fn user_id(&self) -> Result<Uuid, ApiError> {
        Uuid::parse_str(&self.sub)
            .map_err(|_| ApiError::Unauthorized("Invalid user ID in token".to_string()))
    }
}

/// Authenticated user information extracted from the JWT.
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub id: Uuid,
    pub role: String,
}

/// Decode and validate a JWT token.
pub fn decode_jwt(token: &str, secret: &str) -> Result<TokenData<Claims>, ApiError> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|e| ApiError::Unauthorized(format!("Invalid token: {}", e)))
}

/// Extract the Bearer token from an Authorization header value.
pub fn extract_bearer_token(auth_header: &str) -> Result<&str, ApiError> {
    if !auth_header.starts_with("Bearer ") {
        return Err(ApiError::Unauthorized(
            "Invalid authorization header format. Expected 'Bearer <token>'".to_string(),
        ));
    }
    Ok(&auth_header[7..])
}

/// Validate the JWT from a service request and insert `AuthenticatedUser` into extensions.
pub fn validate_request(
    req: &ServiceRequest,
    jwt_secret: &str,
) -> Result<AuthenticatedUser, ApiError> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| ApiError::Unauthorized("Missing Authorization header".to_string()))?;

    let token = extract_bearer_token(auth_header)?;
    let token_data = decode_jwt(token, jwt_secret)?;
    let claims = token_data.claims;

    let user = AuthenticatedUser {
        id: claims.user_id()?,
        role: claims.role,
    };

    // Insert authenticated user into request extensions for downstream handlers
    req.extensions_mut().insert(user.clone());

    Ok(user)
}

/// Check if the authenticated user has the required role.
pub fn require_role(user: &AuthenticatedUser, required_role: &str) -> Result<(), ApiError> {
    if user.role != required_role && user.role != "admin" {
        return Err(ApiError::Forbidden(
            "Insufficient permissions".to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claims_user_id_parsing() {
        let user_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let claims = Claims {
            sub: user_id.to_string(),
            role: "user".to_string(),
            exp: 1234567890,
            iat: 1234567000,
        };

        let parsed_id = claims.user_id().unwrap();
        assert_eq!(parsed_id, user_id);
    }

    #[test]
    fn test_claims_invalid_user_id() {
        let claims = Claims {
            sub: "invalid-uuid".to_string(),
            role: "user".to_string(),
            exp: 1234567890,
            iat: 1234567000,
        };

        assert!(claims.user_id().is_err());
    }

    #[test]
    fn test_extract_bearer_token_success() {
        let auth_header = "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9";
        let token = extract_bearer_token(auth_header).unwrap();
        assert_eq!(token, "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9");
    }

    #[test]
    fn test_extract_bearer_token_invalid_format() {
        let auth_header = "Basic dXNlcm5hbWU6cGFzc3dvcmQ=";
        assert!(extract_bearer_token(auth_header).is_err());
    }

    #[test]
    fn test_extract_bearer_token_no_space() {
        let auth_header = "Bearertoken123";
        assert!(extract_bearer_token(auth_header).is_err());
    }

    #[test]
    fn test_authenticated_user_creation() {
        let user_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let user = AuthenticatedUser {
            id: user_id,
            role: "admin".to_string(),
        };

        assert_eq!(user.id, user_id);
        assert_eq!(user.role, "admin");
    }

    #[test]
    fn test_require_role_admin_access() {
        let user = AuthenticatedUser {
            id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            role: "admin".to_string(),
        };

        assert!(require_role(&user, "admin").is_ok());
        assert!(require_role(&user, "user").is_ok());
        assert!(require_role(&user, "moderator").is_ok());
    }

    #[test]
    fn test_require_role_user_access() {
        let user = AuthenticatedUser {
            id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            role: "user".to_string(),
        };

        assert!(require_role(&user, "user").is_ok());
        assert!(require_role(&user, "admin").is_err());
    }

    #[test]
    fn test_require_role_insufficient_permissions() {
        let user = AuthenticatedUser {
            id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            role: "user".to_string(),
        };

        let result = require_role(&user, "moderator");
        assert!(result.is_err());
    }

    #[test]
    fn test_claims_serialization() {
        let user_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let claims = Claims {
            sub: user_id.to_string(),
            role: "admin".to_string(),
            exp: 1234567890,
            iat: 1234567000,
        };

        let json = serde_json::to_string(&claims).unwrap();
        assert!(json.contains("admin"));
    }

    #[test]
    fn test_authenticated_user_cloneable() {
        let user = AuthenticatedUser {
            id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            role: "user".to_string(),
        };

        let user_clone = user.clone();
        assert_eq!(user.id, user_clone.id);
        assert_eq!(user.role, user_clone.role);
    }
}
