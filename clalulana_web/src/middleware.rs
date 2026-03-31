use axum::{
    extract::FromRequestParts,
    http::request::Parts,
    response::Redirect,
};
use axum_extra::extract::cookie::CookieJar;

use crate::models::AuthInfo;

/// Extractor that reads the auth token and user info from cookies.
/// If the cookies are missing, it redirects to `/login`.
pub struct AuthUser {
    pub token: String,
    pub info: AuthInfo,
}


#[axum::async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = Redirect;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let jar = CookieJar::from_request_parts(parts, state)
            .await
            .map_err(|_| Redirect::to("/login"))?;

        let token = jar
            .get("auth_token")
            .map(|c| c.value().to_string())
            .ok_or_else(|| Redirect::to("/login"))?;

        let info = jar
            .get("auth_user")
            .and_then(|c| serde_json::from_str::<AuthInfo>(c.value()).ok())
            .ok_or_else(|| Redirect::to("/login"))?;

        Ok(Self { token, info })
    }
}
