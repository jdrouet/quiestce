use std::borrow::Cow;

use uuid::Uuid;

#[derive(Clone, serde::Deserialize)]
pub(crate) struct AuthorizationRequest {
    pub client_id: String,
    pub code_challenge: String,
    pub code_challenge_method: String,
    pub redirect_uri: String,
    pub response_type: String,
    pub state: String,
}

#[derive(Clone)]
pub(crate) struct AuthorizationResponse {
    pub client_id: String,
    pub code_challenge: String,
    pub code_challenge_method: String,
    pub redirect_uri: String,
    pub response_type: String,
    pub state: String,
    //
    pub user_id: Uuid,
}

#[derive(Debug, serde::Serialize)]
#[cfg_attr(test, derive(serde::Deserialize))]
pub(crate) struct AuthorizationRedirect {
    pub code: String,
    pub state: String,
}

impl AuthorizationRedirect {
    #[inline]
    pub fn new(code: String, state: String) -> Self {
        Self { code, state }
    }

    pub fn as_redirect_url(&self, url: &str) -> String {
        format!("{url}?{}", serde_qs::to_string(&self).unwrap())
    }
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct AuthorizationError {
    pub error: Cow<'static, str>,
    pub error_description: Cow<'static, str>,
    pub state: Option<String>,
}

impl AuthorizationError {
    pub fn as_redirect_url(&self, url: &str) -> String {
        format!("{url}?{}", serde_qs::to_string(&self).unwrap())
    }
}
