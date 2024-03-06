use std::{net::IpAddr, sync::Arc};

#[derive(Clone, Debug)]
pub(crate) struct BaseUrl(Arc<String>);

impl From<String> for BaseUrl {
    fn from(value: String) -> Self {
        Self(Arc::new(value))
    }
}

impl AsRef<str> for BaseUrl {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl BaseUrl {
    pub fn from_env() -> Option<Self> {
        Some(Self(Arc::new(std::env::var("BASE_URL").ok()?)))
    }

    pub fn new(host: IpAddr, port: u16) -> Self {
        Self(Arc::new(format!("http://{host}:{port}")))
    }

    pub fn from_env_or_new(host: IpAddr, port: u16) -> Self {
        Self::from_env().unwrap_or_else(|| Self::new(host, port))
    }
}
