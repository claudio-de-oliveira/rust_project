use askama::Template;
use axum::{
    extract::State,
    response::{Html, IntoResponse, Redirect, Response},
    Form,
};
use axum_extra::extract::cookie::{Cookie, CookieJar};

use crate::api_client::ApiClient;
use crate::models::*;

// ============================================================================
// Templates
// ============================================================================

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate {
    auth: Option<AuthInfo>,
    error: Option<String>,
}

#[derive(Template)]
#[template(path = "register.html")]
struct RegisterTemplate {
    auth: Option<AuthInfo>,
    error: Option<String>,
    success: Option<String>,
}

// ============================================================================
// Handlers
// ============================================================================

pub async fn login_page() -> impl IntoResponse {
    render(LoginTemplate {
        auth: None,
        error: None,
    })
}

pub async fn login_submit(
    State(client): State<ApiClient>,
    jar: CookieJar,
    Form(form): Form<LoginForm>,
) -> Response {
    match client.login(&form.email, &form.password).await {
        Ok(auth_resp) => {
            let info = AuthInfo {
                id: auth_resp.user.id.clone(),
                username: auth_resp.user.username.clone(),
                role: auth_resp.user.role.clone(),
            };
            let user_json = serde_json::to_string(&info).unwrap_or_default();

            let jar = jar
                .add(
                    Cookie::build(("auth_token", auth_resp.token))
                        .path("/")
                        .http_only(true)
                        .same_site(axum_extra::extract::cookie::SameSite::Lax)
                        .build(),
                )
                .add(
                    Cookie::build(("auth_user", user_json))
                        .path("/")
                        .same_site(axum_extra::extract::cookie::SameSite::Lax)
                        .build(),
                );

            (jar, Redirect::to("/dashboard")).into_response()
        }
        Err(msg) => render(LoginTemplate {
            auth: None,
            error: Some(msg),
        })
        .into_response(),
    }
}

pub async fn register_page() -> impl IntoResponse {
    render(RegisterTemplate {
        auth: None,
        error: None,
        success: None,
    })
}

pub async fn register_submit(
    State(client): State<ApiClient>,
    Form(form): Form<RegisterForm>,
) -> Response {
    match client.register(&form.username, &form.email, &form.password).await {
        Ok(_) => render(RegisterTemplate {
            auth: None,
            error: None,
            success: Some("Conta criada com sucesso! Faça login.".to_string()),
        })
        .into_response(),
        Err(msg) => render(RegisterTemplate {
            auth: None,
            error: Some(msg),
            success: None,
        })
        .into_response(),
    }
}

pub async fn logout(jar: CookieJar) -> impl IntoResponse {
    let jar = jar
        .remove(Cookie::build("auth_token").path("/").build())
        .remove(Cookie::build("auth_user").path("/").build());

    (jar, Redirect::to("/login"))
}

// ============================================================================
// Helper
// ============================================================================

fn render<T: Template>(tmpl: T) -> Html<String> {
    Html(tmpl.render().unwrap_or_else(|e| format!("Erro no template: {}", e)))
}
