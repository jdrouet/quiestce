use axum::{
    extract::Query,
    response::{Html, Redirect},
    Extension,
};
use std::fmt::Write;

use crate::{
    entity::authorization::AuthorizationRequest,
    service::{cache::Cache, database::DatabaseUser, oauth::Oauth},
};

pub(crate) async fn handler(
    Extension(database): Extension<DatabaseUser>,
    Extension(cache): Extension<Cache>,
    Extension(oauth): Extension<Oauth>,
    Query(params): Query<AuthorizationRequest>,
) -> Result<Html<String>, Redirect> {
    if let Err(error) = oauth.check(&params) {
        return Err(Redirect::temporary(
            &error.as_redirect_url(&params.redirect_uri),
        ));
    }
    let links = database
        .as_ref()
        .values()
        .fold(String::default(), |mut res, user| {
            write!(
                &mut res,
                "<p><a href=\"/api/redirect/{}/{}\">Login as {}</a></p>",
                params.state, user.id, user.name
            )
            .unwrap();
            res
        });
    let page = format!(
        "<!DOCTYPE html><html><head><title>Authorization</title></head><body>{links}</body></html>"
    );
    cache.insert_authorization_request(params).await;
    Ok(Html(page))
}
