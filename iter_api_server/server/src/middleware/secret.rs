

use std::{sync::Arc, env};

use castle_api::async_trait;
use envoy_http::Middleware;
use secrets::{BackendSecrets, get_secret, SECRET_NAME};

pub struct SecretMiddleware {
    pub secret: Arc<BackendSecrets>
}

impl SecretMiddleware {
    pub async fn create() -> Result<Self, anyhow::Error> {
        let namespace = env::var("SECRET_NAMESPACE").unwrap_or_else(|_| String::from("dev"));

        Ok(Self {
            secret: Arc::new(get_secret::<BackendSecrets>(SECRET_NAME, &namespace).await?
                .expect("Expected secret to exist"))
        })
    }
}

#[async_trait]
impl Middleware for SecretMiddleware {
    async fn handle(&self, ctx: &mut envoy_http::Context, next: envoy_http::Next) -> envoy_http::Result {
        ctx.insert(self.secret.clone());

        next.run(ctx).await
    }
}