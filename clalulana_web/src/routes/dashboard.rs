use askama::Template;
use axum::{
    extract::{Query, State},
    response::{Html, IntoResponse, Redirect, Response},
};

use crate::api_client::ApiClient;
use crate::middleware::AuthUser;
use crate::models::*;

#[derive(Template)]
#[template(path = "dashboard.html")]
struct DashboardTemplate {
    auth: Option<AuthInfo>,
    user: UserResponse,
    message: Option<String>,
    error: Option<String>,
}

/// GET / — redirect to dashboard or login.
pub async fn index(auth: Option<AuthUser>) -> Redirect {
    match auth {
        Some(_) => Redirect::to("/dashboard"),
        None => Redirect::to("/login"),
    }
}

/// GET /dashboard
pub async fn dashboard(
    State(client): State<ApiClient>,
    auth: AuthUser,
    Query(flash): Query<FlashQuery>,
) -> Response {
    match client.get_current_user(&auth.token).await {
        Ok(user) => Html(
            DashboardTemplate {
                auth: Some(auth.info),
                user,
                message: flash.msg,
                error: flash.error,
            }
            .render()
            .unwrap_or_else(|e| format!("Erro no template: {}", e)),
        )
        .into_response(),
        Err(_) => Redirect::to("/login").into_response(),
    }
}
