use castle_api::async_trait;
use envoy_http::Middleware;
use tracing::{span, Level, Instrument};

pub struct LogMiddleware;

#[async_trait]
impl Middleware for LogMiddleware {
    async fn handle(&self, ctx: &mut envoy_http::Context, next: envoy_http::Next) -> envoy_http::Result {
        let span = span!(Level::INFO, "req", uuid = %uuid::Uuid::new_v4().to_string());
        {
            let _enter = span.enter();
            tracing::event!(Level::INFO, "{} {}", ctx.borrow::<envoy_http::Method>(), ctx.borrow::<envoy_http::Uri>());
        }
        next.run(ctx).instrument(span).await
    }
}