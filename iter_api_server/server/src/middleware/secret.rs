

use std::{sync::Arc, env};
use castle_api::async_trait;
use envoy_http::Middleware;
use k8s_openapi::{api::core::v1::Secret, serde::{de::DeserializeOwned}};
use kube::{Api, Client};
use serde::{Deserialize, Serialize};

pub const SECRET_NAME: &str = "backend-secrets";

pub struct SecretMiddleware {
    pub secret: Arc<BackendSecrets>
}

#[derive(Deserialize, Serialize)]
pub struct BackendSecrets {

}


/// Get a kubernetes client
async fn get_client() -> Client {
    let client = Client::try_default().await.unwrap();
    client
}

/// Get our pods current namespace
pub async fn get_namespace() -> String {
    let config = kube::config::Config::infer().await.unwrap();
    config.default_namespace
}

/// Panics if the secret does not exist
pub async fn get_secret<D: Serialize + DeserializeOwned>(secret_name: &str, namespace: &str) -> Result<Option<D>, anyhow::Error> {
    let client = get_client().await;
    let secret_api: Api<Secret> = Api::namespaced(client, &namespace);

    match secret_api.get_opt(secret_name).await {
        Ok(Some(Secret { data: Some(data), .. })) => serde_json::from_slice(
            &data.get("secret")
                .ok_or(anyhow::anyhow!("Could not get secret data"))?.0
            ).map_err(|e| anyhow::anyhow!(e)),
        Err(e) => Err(anyhow::anyhow!(e)),
        _ => Ok(None)
    }
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