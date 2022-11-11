use std::sync::Arc;

use castle_api::{async_trait};
use envoy_http::Middleware;
use mongodb::Client;
use secrets::BackendSecrets;
use tokio::sync::RwLock;

pub struct MongoDBMiddleware {
    client: RwLock<Option<Client>>,
}

#[async_trait]
impl Middleware for MongoDBMiddleware {
    async fn handle(
        &self,
        ctx: &mut envoy_http::Context,
        next: envoy_http::Next,
    ) -> envoy_http::Result {
        let client = self.client.read().await.clone();
        let client = match client {
            Some(client) => client,
            None => {
                // prevent multiple concurrent requests from creating the client
                // check if another thread has already created the client in the time between the read
                // and exclusive lock on write
                let mut write_guard = self.client.write().await;
                match &*write_guard {
                    None => {
                        let secret = &ctx.borrow::<Arc<BackendSecrets>>().mongo;

                        let uri = format!(
                            "mongodb+srv://{}:{}@{}/test?retryWrites=true&w=majority",
                            secret.username, secret.password, secret.host,
                        );

                        let client = Client::with_uri_str(&uri)
                            .await
                            .map_err(|err| anyhow::anyhow!("{}", err))?;

                        *write_guard = Some(client.clone());

                        client
                    }
                    Some(client) => client.clone(),
                }
            }
        };

        ctx.insert(client);

        next.run(ctx).await
    }
}

impl MongoDBMiddleware {
    pub fn new() -> Self {
        Self {
            client: RwLock::new(None),
        }
    }
}