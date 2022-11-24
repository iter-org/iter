
use castle_api::async_trait;
use envoy_http::{Middleware, StatusCode, Response, Body};
pub struct NotFoundMiddleware;

#[async_trait]

impl Middleware for NotFoundMiddleware {
    async fn handle(&self, _ctx: &mut envoy_http::Context, _next: envoy_http::Next) -> envoy_http::Result {
        let mut response = Response::new(Body::from("Not Found"));
        *response.status_mut() = StatusCode::NOT_FOUND;
        Ok(response)
    }
}