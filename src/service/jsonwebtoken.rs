use std::{
    sync::Arc,
    time::{Duration, SystemTime},
};

use uuid::Uuid;

#[derive(serde::Deserialize)]
pub(crate) struct Config {
    pub duration: Option<u64>,
    pub secret: String,
}

#[cfg(test)]
impl Default for Config {
    fn default() -> Self {
        Self {
            duration: None,
            secret: String::from("secret"),
        }
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct JsonWebTokenClaim {
    exp: usize, // Required (validate_exp defaults to true in validation). Expiration time (as UTC timestamp)
    sub: Uuid,  // Optional. Subject (whom token refers to)
}

impl JsonWebTokenClaim {
    pub fn new(user_id: Uuid, expiration: Duration) -> Self {
        Self {
            exp: expiration.as_secs() as usize,
            sub: user_id,
        }
    }
}

struct JsonWebTokenInner {
    duration: Duration,
    decoding_key: jsonwebtoken::DecodingKey,
    encoding_key: jsonwebtoken::EncodingKey,
    header: jsonwebtoken::Header,
    validation: jsonwebtoken::Validation,
}

#[derive(Clone)]
pub(crate) struct JsonWebToken(Arc<JsonWebTokenInner>);

impl From<Config> for JsonWebToken {
    fn from(value: Config) -> Self {
        Self(Arc::new(JsonWebTokenInner {
            duration: Duration::from_secs(value.duration.unwrap_or(60 * 60)),
            decoding_key: jsonwebtoken::DecodingKey::from_secret(value.secret.as_bytes()),
            encoding_key: jsonwebtoken::EncodingKey::from_secret(value.secret.as_bytes()),
            header: jsonwebtoken::Header::new(jsonwebtoken::Algorithm::HS512),
            validation: jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS512),
        }))
    }
}

impl JsonWebToken {
    pub fn encode(&self, user_id: Uuid) -> (String, u64) {
        use std::ops::Add;

        let expiration = SystemTime::now()
            .add(self.0.duration)
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();

        let claim = JsonWebTokenClaim::new(user_id, expiration);
        (
            jsonwebtoken::encode(&self.0.header, &claim, &self.0.encoding_key).unwrap(),
            expiration.as_secs(),
        )
    }

    pub fn decode(&self, token: &str) -> Option<Uuid> {
        jsonwebtoken::decode::<JsonWebTokenClaim>(token, &self.0.decoding_key, &self.0.validation)
            .map_err(|err| {
                tracing::error!("unable to decode jwt token: {err:?}");
                err
            })
            .ok()
            .map(|payload| payload.claims.sub)
    }
}
