use axum::body::Body;
use axum::extract::rejection::{FormRejection, JsonRejection};
use axum::extract::{FromRequest, Request};
use axum::http::header::CONTENT_TYPE;
use axum::http::{HeaderMap, Response};
use axum::response::IntoResponse;
use axum::{Extension, Form, Json};
use axum_extra::{
    headers::{authorization::Basic, Authorization},
    TypedHeader,
};

use crate::entity::accesstoken::{AccessTokenRequest, AccessTokenResponse};
use crate::entity::authorization::AuthorizationError;
use crate::service::cache::Cache;
use crate::service::jsonwebtoken::JsonWebToken;
use crate::service::oauth::Oauth;

use super::ApiError;

fn is_json_content(headers: &HeaderMap) -> bool {
    let Some(content_type) = headers.get(CONTENT_TYPE) else {
        return false;
    };

    let Ok(content_type) = content_type.to_str() else {
        return false;
    };

    content_type.starts_with("application/json")
}

#[derive(Debug)]
pub(crate) enum AccessTokenRequestParseError {
    Json(JsonRejection),
    Form(FormRejection),
}

impl IntoResponse for AccessTokenRequestParseError {
    fn into_response(self) -> Response<Body> {
        match self {
            Self::Form(inner) => inner.into_response(),
            Self::Json(inner) => inner.into_response(),
        }
    }
}

#[axum::async_trait]
impl<S> FromRequest<S> for AccessTokenRequest
where
    S: Send + Sized + Sync,
{
    type Rejection = AccessTokenRequestParseError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        if is_json_content(req.headers()) {
            Json::<AccessTokenRequest>::from_request(req, state)
                .await
                .map(|Json(inner)| inner)
                .map_err(AccessTokenRequestParseError::Json)
        } else {
            Form::<AccessTokenRequest>::from_request(req, state)
                .await
                .map(|Form(inner)| inner)
                .map_err(AccessTokenRequestParseError::Form)
        }
    }
}

pub(crate) async fn handler(
    Extension(oauth): Extension<Oauth>,
    Extension(cache): Extension<Cache>,
    Extension(jwt): Extension<JsonWebToken>,
    TypedHeader(Authorization(basic)): TypedHeader<Authorization<Basic>>,
    payload: AccessTokenRequest,
) -> Result<Json<AccessTokenResponse>, ApiError> {
    oauth
        .check_basic_token(basic.username(), basic.password())
        .map_err(ApiError::bad_request)?;

    let Some(auth_response) = cache.remove_authorization_response(&payload.code).await else {
        return Err(ApiError::bad_request(AuthorizationError {
            error: "code-not-found".into(),
            error_description: "The provided code was not found in our database.".into(),
            state: None,
        }));
    };

    // TODO run code challenge method on code verifier

    oauth
        .check_redirect_uri(&payload.redirect_uri, Some(auth_response.state))
        .map_err(ApiError::bad_request)?;

    let (access_token, expires_in) = jwt.encode(auth_response.user_id);

    Ok(Json(AccessTokenResponse {
        access_token,
        expires_in: Some(expires_in),
        token_type: "Bearer",
    }))
}
