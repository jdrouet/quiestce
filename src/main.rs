mod entity;
mod handler;
mod service;

use axum::Extension;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

fn init_logger() {
    use tracing::Level;
    use tracing_subscriber::FmtSubscriber;

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();
    if let Err(err) = tracing::subscriber::set_global_default(subscriber) {
        eprintln!("error: {err:?}");
    }
}

struct Server {
    address: SocketAddr,
    base_url: service::baseurl::BaseUrl,
    database_user: service::database::DatabaseUser,
    cache: service::cache::Cache,
    jsonwebtoken: service::jsonwebtoken::JsonWebToken,
    oauth: service::oauth::Oauth,
}

#[cfg(test)]
impl From<service::Config> for Server {
    fn from(config: service::Config) -> Self {
        let host = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        let port = 3010;

        Self {
            address: SocketAddr::from((host, port)),
            base_url: service::baseurl::BaseUrl::from_env_or_new(host, port),
            database_user: service::database::DatabaseUser::from(config.users),
            cache: service::cache::Cache::default(),
            jsonwebtoken: service::jsonwebtoken::JsonWebToken::from(config.jsonwebtoken),
            oauth: service::oauth::Oauth::from(config.oauth),
        }
    }
}

impl Server {
    fn from_env() -> Self {
        let config = crate::service::Config::from_env();

        let host = std::env::var("HOST")
            .ok()
            .and_then(|value| value.parse::<IpAddr>().ok())
            .unwrap_or(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
        let port = std::env::var("PORT")
            .ok()
            .and_then(|value| value.parse::<u16>().ok())
            .unwrap_or(3010);

        Self {
            address: SocketAddr::from((host, port)),
            base_url: service::baseurl::BaseUrl::from_env_or_new(host, port),
            database_user: service::database::DatabaseUser::from(config.users),
            cache: service::cache::Cache::default(),
            jsonwebtoken: service::jsonwebtoken::JsonWebToken::from(config.jsonwebtoken),
            oauth: service::oauth::Oauth::from(config.oauth),
        }
    }

    fn router(self) -> axum::Router {
        use axum::routing::{get, post};

        axum::Router::new()
            .route("/authorize", get(handler::authorize::handler))
            .route(
                "/api/redirect/:state/:user_id",
                get(handler::redirect::handler),
            )
            .route("/api/status", get(handler::status::handler))
            .route("/api/token", post(handler::token::handler))
            .route("/api/userinfo", get(handler::userinfo::handler))
            .layer(Extension(self.base_url))
            .layer(Extension(self.database_user))
            .layer(Extension(self.cache))
            .layer(Extension(self.jsonwebtoken))
            .layer(Extension(self.oauth))
            .layer(TraceLayer::new_for_http())
    }

    pub async fn listen(self) {
        tracing::debug!("starting server on {}", self.address);
        let listener = TcpListener::bind(self.address).await.unwrap();
        axum::serve(listener, self.router()).await.unwrap()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logger();

    Server::from_env().listen().await;

    Ok(())
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::{header, Request, StatusCode};
    use http_body_util::BodyExt;
    use oauth2::basic::BasicClient;
    use oauth2::{
        AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, HttpResponse,
        PkceCodeChallenge, RedirectUrl, TokenResponse, TokenUrl,
    };
    use tower::util::ServiceExt;

    use crate::entity::authorization::AuthorizationRedirect;
    use crate::service::Config;

    #[derive(Debug)]
    struct LocalHttpError;

    impl std::fmt::Display for LocalHttpError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Something went wrong with local http error")
        }
    }

    impl std::error::Error for LocalHttpError {}

    fn convert_request(request: oauth2::HttpRequest) -> axum::http::Request<Body> {
        request
            .headers
            .into_iter()
            .fold(Request::builder(), |req, entry| {
                if let Some(name) = entry.0 {
                    req.header(name.as_str(), entry.1.as_bytes())
                } else {
                    req
                }
            })
            .uri(request.url.path())
            .method(request.method.as_str())
            .body(Body::from(request.body))
            .unwrap()
    }

    fn convert_response(
        status: axum::http::StatusCode,
        headers: axum::http::HeaderMap,
        body: axum::body::Bytes,
    ) -> HttpResponse {
        let status_code = oauth2::http::StatusCode::from_u16(status.as_u16()).unwrap();
        let headers = headers.into_iter().fold(
            oauth2::http::HeaderMap::default(),
            |mut h, (name, value)| {
                if let Some(name) = name {
                    let name =
                        oauth2::http::HeaderName::from_bytes(name.as_str().as_bytes()).unwrap();
                    let value = oauth2::http::HeaderValue::from_bytes(value.as_bytes()).unwrap();
                    h.insert(name, value);
                }
                h
            },
        );

        HttpResponse {
            status_code,
            headers,
            body: body.into_iter().collect(),
        }
    }

    async fn local_async_http_client(
        app: axum::Router,
        request: oauth2::HttpRequest,
    ) -> Result<oauth2::HttpResponse, LocalHttpError> {
        let res = app.oneshot(convert_request(request)).await.unwrap();

        let status = res.status();
        let headers = res.headers().clone();
        let body = res.into_body().collect().await.unwrap().to_bytes();

        Ok(convert_response(status, headers, body))
    }

    #[tokio::test]
    async fn authentication_workflow() {
        super::init_logger();

        let client = BasicClient::new(
            ClientId::new("client-id".to_string()),
            Some(ClientSecret::new("client-secret".to_string())),
            AuthUrl::new("http://quiestce/authorize".to_string()).unwrap(),
            Some(TokenUrl::new("http://quiestce/api/token".to_string()).unwrap()),
        )
        // Set the URL the user will be redirected to after the authorization process.
        .set_redirect_uri(RedirectUrl::new("http://app/api/redirect".to_string()).unwrap());

        // Generate a PKCE challenge.
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        // Generate the full authorization URL.
        let (auth_url, csrf_token) = client
            .authorize_url(CsrfToken::new_random)
            // Set the PKCE code challenge.
            .set_pkce_challenge(pkce_challenge)
            .url();

        let auth_url = auth_url
            .to_string()
            .strip_prefix("http://quiestce")
            .unwrap()
            .to_owned();

        let config = Config::default();
        let server = super::Server::from(config);
        let app = server.router();

        let res = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(auth_url)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);

        let body = res.into_body().collect().await.unwrap().to_bytes();
        let body = String::from_utf8_lossy(body.as_ref());
        assert!(body.contains("Alice"));

        let re = regex::Regex::new("href=\"(/api/redirect/[^\"]+)\"").unwrap();
        let auth_url = &re.captures(&body).unwrap()[1];

        let res = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(auth_url)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert!(res.status().is_redirection());

        let location = res.headers().get(header::LOCATION).unwrap();
        let location = String::from_utf8_lossy(location.as_bytes()).to_string();
        assert!(location.starts_with("http://app/api/redirect"));

        let query_params = location.strip_prefix("http://app/api/redirect?").unwrap();
        let query_params: AuthorizationRedirect = serde_qs::from_str(query_params).unwrap();

        assert_eq!(&query_params.state, csrf_token.secret());

        let token_result = client
            .exchange_code(AuthorizationCode::new(query_params.code))
            .set_pkce_verifier(pkce_verifier)
            .request_async(|request| async { local_async_http_client(app.clone(), request).await })
            .await
            .unwrap();

        let res = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/userinfo")
                    .header(
                        "Authorization",
                        format!("Bearer {}", token_result.access_token().secret()),
                    )
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }
}
