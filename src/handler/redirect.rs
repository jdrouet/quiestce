use axum::{extract::Path, response::Redirect, Extension};
use uuid::Uuid;

use crate::{
    entity::authorization::{AuthorizationError, AuthorizationRedirect, AuthorizationResponse},
    service::{cache::Cache, database::DatabaseUser},
};

use super::ApiError;

pub(crate) async fn handler(
    Extension(database): Extension<DatabaseUser>,
    Extension(cache): Extension<Cache>,
    Path((state, user_id)): Path<(String, Uuid)>,
) -> Result<Redirect, ApiError> {
    let Some(request) = cache.remove_authorization_request(&state).await else {
        return Err(ApiError::bad_request(AuthorizationError {
            error: "state_unknown".into(),
            error_description: "Unable to find authorization request with the provided state."
                .into(),
            state: Some(state),
        }));
    };
    if !database.as_ref().contains_key(&user_id) {
        return Err(ApiError::bad_request(AuthorizationError {
            error: "user_not_found".into(),
            error_description: "Unable to find the requested user.".into(),
            state: Some(state),
        }));
    };

    cache
        .insert_authorization_response(AuthorizationResponse {
            // client_id: request.client_id,
            code_challenge: request.code_challenge.clone(),
            // code_challenge_method: request.code_challenge_method,
            // redirect_uri: request.redirect_uri.clone(),
            // response_type: request.response_type,
            state: request.state,
            user_id,
        })
        .await;

    Ok(Redirect::temporary(
        &AuthorizationRedirect::new(request.code_challenge, state)
            .as_redirect_url(&request.redirect_uri),
    ))
}
