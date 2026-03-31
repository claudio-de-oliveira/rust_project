use askama::Template;
use axum::{
    extract::{Path, Query, State},
    response::{Html, IntoResponse, Redirect, Response},
    Form,
};

use crate::api_client::ApiClient;
use crate::middleware::AuthUser;
use crate::models::*;

// ============================================================================
// Templates
// ============================================================================

#[derive(Template)]
#[template(path = "users_list.html")]
struct UsersListTemplate {
    auth: Option<AuthInfo>,
    users: Vec<UserResponse>,
    message: Option<String>,
    error: Option<String>,
}

#[derive(Template)]
#[template(path = "user_edit.html")]
struct UserEditTemplate {
    auth: Option<AuthInfo>,
    user: UserResponse,
    error: Option<String>,
}

// ============================================================================
// Handlers
// ============================================================================

/// GET /users — list all users (admin only).
pub async fn list_users(
    State(client): State<ApiClient>,
    auth: AuthUser,
    Query(flash): Query<FlashQuery>,
) -> Response {
    match client.get_all_users(&auth.token, 50, 0).await {
        Ok(users) => render(UsersListTemplate {
            auth: Some(auth.info),
            users,
            message: flash.msg,
            error: flash.error,
        })
        .into_response(),
        Err(msg) => render(UsersListTemplate {
            auth: Some(auth.info),
            users: vec![],
            message: None,
            error: Some(msg),
        })
        .into_response(),
    }
}

/// GET /users/:id/edit — edit user form.
pub async fn edit_user_page(
    State(client): State<ApiClient>,
    auth: AuthUser,
    Path(id): Path<String>,
) -> Response {
    match client.get_user_by_id(&auth.token, &id).await {
        Ok(user) => render(UserEditTemplate {
            auth: Some(auth.info),
            user,
            error: None,
        })
        .into_response(),
        Err(msg) => Redirect::to(&format!("/users?error={}", urlencoded(&msg))).into_response(),
    }
}

/// POST /users/:id/edit — submit user edit.
pub async fn edit_user_submit(
    State(client): State<ApiClient>,
    auth: AuthUser,
    Path(id): Path<String>,
    Form(form): Form<UserEditForm>,
) -> Response {
    match client
        .update_user(
            &auth.token,
            &id,
            form.username.as_deref(),
            form.email.as_deref(),
        )
        .await
    {
        Ok(_) => {
            // If editing own profile, redirect to dashboard
            if id == auth.info.id {
                Redirect::to("/dashboard?msg=Perfil+atualizado+com+sucesso").into_response()
            } else {
                Redirect::to("/users?msg=Usuário+atualizado+com+sucesso").into_response()
            }
        }
        Err(msg) => {
            // Re-fetch user to re-render the form with error
            match client.get_user_by_id(&auth.token, &id).await {
                Ok(user) => render(UserEditTemplate {
                    auth: Some(auth.info),
                    user,
                    error: Some(msg),
                })
                .into_response(),
                Err(_) => {
                    Redirect::to(&format!("/users?error={}", urlencoded(&msg))).into_response()
                }
            }
        }
    }
}

/// POST /users/:id/delete — delete user.
pub async fn delete_user(
    State(client): State<ApiClient>,
    auth: AuthUser,
    Path(id): Path<String>,
) -> Response {
    match client.delete_user(&auth.token, &id).await {
        Ok(_) => {
            Redirect::to("/users?msg=Usuário+excluído+com+sucesso").into_response()
        }
        Err(msg) => {
            Redirect::to(&format!("/users?error={}", urlencoded(&msg))).into_response()
        }
    }
}

// ============================================================================
// Helpers
// ============================================================================

fn render<T: Template>(tmpl: T) -> Html<String> {
    Html(tmpl.render().unwrap_or_else(|e| format!("Erro no template: {}", e)))
}

fn urlencoded(s: &str) -> String {
    s.replace(' ', "+")
        .replace('&', "%26")
        .replace('=', "%3D")
}
