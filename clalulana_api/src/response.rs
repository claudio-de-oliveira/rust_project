use chrono::Utc;
use serde::Serialize;

/// Standardized API response envelope.
#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub data: T,
    pub meta: ResponseMeta,
}

/// Metadata included in every API response.
#[derive(Debug, Serialize)]
pub struct ResponseMeta {
    pub timestamp: String,
    pub version: String,
}

impl<T: Serialize> ApiResponse<T> {
    /// Create a successful response with data.
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data,
            meta: ResponseMeta {
                timestamp: Utc::now().to_rfc3339(),
                version: "v1".to_string(),
            },
        }
    }
}

/// Empty data placeholder for responses without a body.
#[derive(Debug, Serialize)]
pub struct EmptyData;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_success_response_creation() {
        let data = json!({"id": 1, "name": "test"});
        let response = ApiResponse::success(data.clone());

        assert!(response.success);
        assert_eq!(response.data, data);
        assert_eq!(response.meta.version, "v1");
    }

    #[test]
    fn test_response_meta_contains_timestamp() {
        let response = ApiResponse::success(json!({"test": "data"}));

        assert!(!response.meta.timestamp.is_empty());
        assert!(response.meta.timestamp.contains('T'));
    }

    #[test]
    fn test_response_serialization() {
        let response = ApiResponse::success(json!({"id": 123, "name": "John"}));
        let json_str = serde_json::to_string(&response).unwrap();

        assert!(json_str.contains("\"success\":true"));
        assert!(json_str.contains("\"version\":\"v1\""));
    }

    #[test]
    fn test_empty_data_response() {
        let response = ApiResponse::success(EmptyData);

        assert!(response.success);
        assert_eq!(response.meta.version, "v1");
    }

    #[test]
    fn test_response_version_is_v1() {
        let response = ApiResponse::success(json!(null));
        assert_eq!(response.meta.version, "v1");
    }
}
