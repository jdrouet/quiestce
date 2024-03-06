use std::{borrow::Cow, sync::Arc};

use crate::entity::authorization::{AuthorizationError, AuthorizationRequest};

#[derive(serde::Deserialize)]
pub(crate) struct Config {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

#[cfg(test)]
impl Default for Config {
    fn default() -> Self {
        Self {
            client_id: String::from("client-id"),
            client_secret: String::from("client-secret"),
            redirect_uri: String::from("http://app/api/redirect"),
        }
    }
}

#[derive(Clone)]
pub(crate) struct Oauth(Arc<Config>);

impl Oauth {
    pub fn check_basic_token(
        &self,
        client_id: &str,
        client_secret: &str,
    ) -> Result<(), AuthorizationError> {
        if !self.0.client_id.eq(client_id) {
            Err(AuthorizationError {
                error: "invalid_client_id".into(),
                error_description: "Unable to find an application with the provided client_id."
                    .into(),
                state: None,
            })
        } else if !self.0.client_secret.eq(client_secret) {
            Err(AuthorizationError {
                error: "invalid_client_secret".into(),
                error_description: "The provided client secret is invalid.".into(),
                state: None,
            })
        } else {
            Ok(())
        }
    }

    pub fn check_redirect_uri(
        &self,
        uri: &str,
        state: Option<String>,
    ) -> Result<(), AuthorizationError> {
        if !self.0.redirect_uri.eq(uri) {
            return Err(AuthorizationError {
                error: Cow::Borrowed("redirect_uri_mismatch"),
                error_description: Cow::Borrowed(
                    "The redirect_uri MUST match the registered callback URL for this application.",
                ),
                state,
            });
        }

        Ok(())
    }

    pub fn check(&self, req: &AuthorizationRequest) -> Result<(), AuthorizationError> {
        if !self.0.client_id.eq(&req.client_id) {
            return Err(AuthorizationError {
                error: "invalid_client_id".into(),
                error_description: "Unable to find an application with the provided client_id."
                    .into(),
                state: Some(req.state.clone()),
            });
        }
        if !self.0.redirect_uri.eq(&req.redirect_uri) {
            return Err(AuthorizationError {
                error: Cow::Borrowed("redirect_uri_mismatch"),
                error_description: Cow::Borrowed(
                    "The redirect_uri MUST match the registered callback URL for this application.",
                ),
                state: Some(req.state.clone()),
            });
        }

        Ok(())
    }
}

impl From<Config> for Oauth {
    fn from(value: Config) -> Self {
        Self(Arc::new(value))
    }
}
