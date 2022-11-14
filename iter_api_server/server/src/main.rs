use std::net::SocketAddr;

use server::create_app;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt::init();

    let app = create_app().await?;

    app.listen(SocketAddr::from(([0, 0, 0, 0], 80))).await
        .map_err(|e| anyhow::anyhow!(e))?;

    Ok(())
}

#[tokio::test]
async fn dev_server() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt::init();

    let app = create_app().await?;
    println!("Backend running!");

    app.listen(SocketAddr::from(([0, 0, 0, 0], 81))).await
        .map_err(|e| anyhow::anyhow!(e))?;

    Ok(())
}