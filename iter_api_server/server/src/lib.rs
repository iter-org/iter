
pub mod middleware;
mod graph;
pub mod models;

pub async fn create_app() -> Result<envoy_http::Server, anyhow::Error> {
    let mut app = envoy_http::new();

    app.with(middleware::ErrorMiddleware);
    app.with(middleware::LogMiddleware);
    app.with(middleware::NamespaceMiddleware::create().await);
    app.with(middleware::SecretMiddleware::create().await?);
    app.with(middleware::MongoDBMiddleware::new());
    app.with(middleware::IndexMiddleware::new());

    app.at("/")
        .post(graph::CastleEndpoint::create().await);
    // 404
    app.at("*")
        .with(middleware::NotFoundMiddleware);

    Ok(app)
}
