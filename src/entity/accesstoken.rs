#[derive(Debug, serde::Deserialize)]
pub(crate) struct AccessTokenRequest {
    pub code: String,
    // pub code_verifier: String,
    // pub grant_type: String,
    pub redirect_uri: String,
}

#[derive(serde::Serialize)]
pub(crate) struct AccessTokenResponse {
    pub access_token: String,
    pub token_type: &'static str,
    pub expires_in: Option<u64>,
}
