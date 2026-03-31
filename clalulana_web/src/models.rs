use serde::{Deserialize, Serialize};

// ============================================================================
// API Response Envelope
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: T,
}

#[derive(Debug, Deserialize)]
pub struct ApiErrorResponse {
    pub error: ApiErrorDetail,
}

#[derive(Debug, Deserialize)]
pub struct ApiErrorDetail {
    pub code: u16,
    pub message: String,
}

// ============================================================================
// Domain DTOs (mirror the API responses)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: String,
    pub username: String,
    pub email: String,
    pub role: String,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl UserResponse {
    /// Returns just the date portion (YYYY-MM-DD) for display.
    pub fn created_date(&self) -> &str {
        self.created_at.get(..10).unwrap_or(&self.created_at)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthApiResponse {
    pub token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub user: UserResponse,
}

// ============================================================================
// Auth info stored in cookie
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthInfo {
    pub id: String,
    pub username: String,
    pub role: String,
}

// ============================================================================
// Form DTOs
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct LoginForm {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterForm {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct UserEditForm {
    pub username: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FlashQuery {
    pub msg: Option<String>,
    pub error: Option<String>,
}
