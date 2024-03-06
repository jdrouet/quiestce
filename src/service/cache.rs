use std::{sync::Arc, time::Duration};

use crate::entity::authorization::{AuthorizationRequest, AuthorizationResponse};

#[derive(Clone, Default)]
pub(crate) struct Cache(Arc<CacheInner>);

impl Cache {
    pub async fn insert_authorization_request(&self, req: AuthorizationRequest) {
        self.0
            .authorization_request
            .insert(req.state.clone(), req)
            .await;
    }

    pub async fn remove_authorization_request(&self, state: &str) -> Option<AuthorizationRequest> {
        self.0.authorization_request.remove(state).await
    }

    pub async fn insert_authorization_response(&self, res: AuthorizationResponse) {
        self.0
            .authorization_response
            .insert(res.code_challenge.clone(), res)
            .await;
    }

    pub async fn remove_authorization_response(&self, code: &str) -> Option<AuthorizationResponse> {
        self.0.authorization_response.remove(code).await
    }
}

struct CacheInner {
    authorization_request: moka::future::Cache<String, AuthorizationRequest>,
    authorization_response: moka::future::Cache<String, AuthorizationResponse>,
}

impl Default for CacheInner {
    fn default() -> Self {
        Self {
            authorization_request: moka::future::Cache::builder()
                .max_capacity(100)
                .time_to_live(Duration::from_secs(120))
                .build(),
            authorization_response: moka::future::Cache::builder()
                .max_capacity(100)
                .time_to_live(Duration::from_secs(120))
                .build(),
        }
    }
}
