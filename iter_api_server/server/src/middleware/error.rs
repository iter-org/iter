use castle_api::async_trait;

use envoy_http::{Middleware, Response, StatusCode, http::HeaderValue};
use serde_json::json;
pub struct ErrorMiddleware;

#[async_trait]
impl Middleware for ErrorMiddleware {
    async fn handle(&self, ctx: &mut envoy_http::Context, next: envoy_http::Next) -> envoy_http::Result {
        let mut res = match next.run(ctx).await {
            Ok(response) => response,
            Err(error) => {
                let res = serde_json::to_string_pretty(&json!({
                    "data": {},
                    "errors": vec![error.to_string()]
                }))?; // this ? could fail but there's no real way to handle it

                let mut response = Response::new(res.into());
                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;

                response
            }
        };

        res.headers_mut().append(
            envoy_http::http::header::ACCESS_CONTROL_ALLOW_ORIGIN,
            HeaderValue::from_str("*")?,
        );

        Ok(res)
    }
}