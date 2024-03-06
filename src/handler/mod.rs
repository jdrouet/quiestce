use axum::{http::StatusCode, response::IntoResponse};

use crate::entity::authorization::AuthorizationError;

pub(crate) mod authorize;
pub(crate) mod redirect;
pub(crate) mod status;
pub(crate) mod token;
pub(crate) mod userinfo;

pub(crate) struct ApiError {
    code: StatusCode,
    inner: AuthorizationError,
}

impl ApiError {
    pub fn bad_request(inner: AuthorizationError) -> Self {
        Self {
            code: StatusCode::BAD_REQUEST,
            inner,
        }
    }

    pub fn unauthorized(inner: AuthorizationError) -> Self {
        Self {
            code: StatusCode::UNAUTHORIZED,
            inner,
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        (self.code, axum::Json(self.inner)).into_response()
    }
}
