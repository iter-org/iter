

use std::sync::Arc;

use castle_api::{async_trait, Castle, types::State};
use envoy_http::{Endpoint, Response, Body};
use mongodb::Client as MongoClient;
use secrets::BackendSecrets;
use serde_json::json;

use crate::{models::{Root, User}, middleware::KubernetesNamespace};

pub struct CastleEndpoint {
    pub castle: Castle,
}

#[async_trait]
impl Endpoint for CastleEndpoint {
    async fn call(&self, ctx: &mut envoy_http::Context) -> envoy_http::Result {
        Ok(Response::new(Body::from(graph_handler(ctx, &self.castle).await?)))
    }
}

impl CastleEndpoint {
    pub async fn create() -> Self {
        Self {
            castle: castle_api::castle::CastleBuilder::new(Root)
                .build()
                .unwrap(),
        }
    }
}

#[derive(serde::Deserialize)]
struct GraphRequest {
    pub query: String,
    pub auth_token: Option<String>,
}

pub async fn graph_handler(
    ctx: &mut envoy_http::Context,
    castle: &Castle,
) -> Result<String, anyhow::Error> {
    let body = envoy_http::body::to_bytes(ctx.take::<Body>()).await?;
    let json = String::from_utf8_lossy(&body);
    let req: GraphRequest = serde_json::from_str(&json)?;

    // Set up the castle state
    let mut state = create_state_from_context(ctx);

    // Authenticate the user
    if let Some(auth_token) = req.auth_token {
        let secret = state.borrow::<Arc<BackendSecrets>>();
        let user = User::authenticate_from_token(&auth_token, &secret.jwt_secret, &state).await?;
        state.insert(user);
    }

    let json = match castle.run_message(&req.query, &state).await {
        Ok((data, errors)) => json!({
            "data": data,
            "errors": errors
                .into_iter()
                .map(|e| format!("{}", e))
                .collect::<Vec<_>>()
        }),
        Err(e) => json!({
            "data": {},
            "errors": vec![e.to_string()]
        }),
    };

    match serde_json::to_string_pretty(&json) {
        Ok(json) => Ok(json.into()),
        Err(e) => Err(e.into()),
    }
}

pub fn create_state_from_context(ctx: &mut envoy_http::Context) -> State {
    let mut state = State::new();
    ctx.try_borrow::<Arc<BackendSecrets>>().cloned().map(|secret| state.insert(secret));
    ctx.try_borrow::<MongoClient>().cloned().map(|client| state.insert(client));
    ctx.try_borrow::<KubernetesNamespace>().cloned().map(|secret| state.insert(secret));
    state
}