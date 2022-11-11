use std::{sync::Arc};

use castle_api::async_trait;
use secrets::get_namespace;
use envoy_http::Middleware;


pub struct NamespaceMiddleware(Arc<String>);
#[derive(Clone, Debug)]
pub struct KubernetesNamespace(pub Arc<String>);

#[async_trait]
impl Middleware for NamespaceMiddleware {
    async fn handle(&self, ctx: &mut envoy_http::Context, next: envoy_http::Next) -> envoy_http::Result {
        ctx.insert(KubernetesNamespace(self.0.clone()));
        next.run(ctx).await
    }
}

impl NamespaceMiddleware {
    pub async fn create() -> Self {
        Self(Arc::new(get_namespace().await))
    }
}

impl AsRef<str> for KubernetesNamespace {
    fn as_ref(&self) -> &str {
        &self.0
    }
}