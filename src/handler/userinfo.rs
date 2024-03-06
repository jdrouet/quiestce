use axum::{Extension, Json};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};

use crate::entity::authorization::AuthorizationError;
use crate::entity::user::User;
use crate::service::database::DatabaseUser;
use crate::service::jsonwebtoken::JsonWebToken;

use super::ApiError;

pub(crate) async fn handler(
    Extension(database): Extension<DatabaseUser>,
    Extension(jwt): Extension<JsonWebToken>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
) -> Result<Json<User>, ApiError> {
    let user_id = jwt.decode(bearer.token()).ok_or_else(|| {
        ApiError::unauthorized(AuthorizationError {
            error: "invalid-bearer".into(),
            error_description: "Unable to decode bearer token.".into(),
            state: None,
        })
    })?;
    let Some(user) = database.as_ref().get(&user_id) else {
        return Err(ApiError::bad_request(AuthorizationError {
            error: "user-not-found".into(),
            error_description: "Unable to find user.".into(),
            state: None,
        }));
    };

    Ok(Json(user.clone()))
}
