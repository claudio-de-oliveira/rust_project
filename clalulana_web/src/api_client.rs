use crate::models::*;

/// HTTP client that wraps all calls to the clalulana_api backend.
#[derive(Clone)]
pub struct ApiClient {
    client: reqwest::Client,
    base_url: String,
}

impl ApiClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}/api/v1{}", self.base_url, path)
    }

    // ========================================================================
    // Auth
    // ========================================================================

    pub async fn login(&self, email: &str, password: &str) -> Result<AuthApiResponse, String> {
        let body = serde_json::json!({ "email": email, "password": password });

        let resp = self
            .client
            .post(self.url("/auth/login"))
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Erro de conexão: {}", e))?;

        if resp.status().is_success() {
            let api: ApiResponse<AuthApiResponse> =
                resp.json().await.map_err(|e| format!("Erro ao ler resposta: {}", e))?;
            Ok(api.data)
        } else {
            Err(self.extract_error(resp).await)
        }
    }

    pub async fn register(
        &self,
        username: &str,
        email: &str,
        password: &str,
    ) -> Result<UserResponse, String> {
        let body =
            serde_json::json!({ "username": username, "email": email, "password": password });

        let resp = self
            .client
            .post(self.url("/auth/register"))
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Erro de conexão: {}", e))?;

        if resp.status().is_success() {
            let api: ApiResponse<UserResponse> =
                resp.json().await.map_err(|e| format!("Erro ao ler resposta: {}", e))?;
            Ok(api.data)
        } else {
            Err(self.extract_error(resp).await)
        }
    }

    // ========================================================================
    // Users
    // ========================================================================

    pub async fn get_current_user(&self, token: &str) -> Result<UserResponse, String> {
        let resp = self
            .client
            .get(self.url("/users/me"))
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| format!("Erro de conexão: {}", e))?;

        if resp.status().is_success() {
            let api: ApiResponse<UserResponse> =
                resp.json().await.map_err(|e| format!("Erro ao ler resposta: {}", e))?;
            Ok(api.data)
        } else {
            Err(self.extract_error(resp).await)
        }
    }

    pub async fn get_all_users(
        &self,
        token: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<UserResponse>, String> {
        let resp = self
            .client
            .get(self.url("/users"))
            .bearer_auth(token)
            .query(&[("limit", limit), ("offset", offset)])
            .send()
            .await
            .map_err(|e| format!("Erro de conexão: {}", e))?;

        if resp.status().is_success() {
            let api: ApiResponse<Vec<UserResponse>> =
                resp.json().await.map_err(|e| format!("Erro ao ler resposta: {}", e))?;
            Ok(api.data)
        } else {
            Err(self.extract_error(resp).await)
        }
    }

    pub async fn get_user_by_id(&self, token: &str, id: &str) -> Result<UserResponse, String> {
        let resp = self
            .client
            .get(self.url(&format!("/users/{}", id)))
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| format!("Erro de conexão: {}", e))?;

        if resp.status().is_success() {
            let api: ApiResponse<UserResponse> =
                resp.json().await.map_err(|e| format!("Erro ao ler resposta: {}", e))?;
            Ok(api.data)
        } else {
            Err(self.extract_error(resp).await)
        }
    }

    pub async fn update_user(
        &self,
        token: &str,
        id: &str,
        username: Option<&str>,
        email: Option<&str>,
    ) -> Result<UserResponse, String> {
        let mut body = serde_json::Map::new();
        if let Some(u) = username {
            body.insert("username".into(), serde_json::Value::String(u.to_string()));
        }
        if let Some(e) = email {
            body.insert("email".into(), serde_json::Value::String(e.to_string()));
        }

        let resp = self
            .client
            .put(self.url(&format!("/users/{}", id)))
            .bearer_auth(token)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Erro de conexão: {}", e))?;

        if resp.status().is_success() {
            let api: ApiResponse<UserResponse> =
                resp.json().await.map_err(|e| format!("Erro ao ler resposta: {}", e))?;
            Ok(api.data)
        } else {
            Err(self.extract_error(resp).await)
        }
    }

    pub async fn delete_user(&self, token: &str, id: &str) -> Result<(), String> {
        let resp = self
            .client
            .delete(self.url(&format!("/users/{}", id)))
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| format!("Erro de conexão: {}", e))?;

        if resp.status().is_success() {
            Ok(())
        } else {
            Err(self.extract_error(resp).await)
        }
    }

    // ========================================================================
    // Helpers
    // ========================================================================

    async fn extract_error(&self, resp: reqwest::Response) -> String {
        resp.json::<ApiErrorResponse>()
            .await
            .map(|e| e.error.message)
            .unwrap_or_else(|_| "Erro desconhecido na API".to_string())
    }
}
