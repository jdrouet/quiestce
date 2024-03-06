use std::path::PathBuf;

pub(crate) mod baseurl;
pub(crate) mod cache;
pub(crate) mod database;
pub(crate) mod jsonwebtoken;
pub(crate) mod oauth;

#[derive(serde::Deserialize)]
pub(crate) struct Config {
    pub oauth: oauth::Config,
    pub jsonwebtoken: jsonwebtoken::Config,
    pub users: Vec<crate::entity::user::User>,
}

#[cfg(test)]
impl Default for Config {
    fn default() -> Self {
        let content = include_str!("../../config.toml");
        toml::from_str(content).expect("couldn't parse default configuration file")
    }
}

impl Config {
    pub fn from_env() -> Self {
        let path = std::env::var("CONFIG_PATH")
            .ok()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("config.toml"));
        let content = std::fs::read_to_string(path).expect("configuration not found");
        toml::from_str(&content).expect("couldn't parse configuration file")
    }
}
