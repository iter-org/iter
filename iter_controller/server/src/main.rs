use std::net::SocketAddr;

use octorust::{Client, auth::{Credentials, JWTCredentials, InstallationTokenGenerator}};
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

#[tokio::test]
async fn github_test() -> Result<(), anyhow::Error> {
    // let pem = include_bytes!("./lumina-iter-pk.pem").to_vec();
    // let der = nom_pem::decode_block(&pem).unwrap().data;

    // let credentials = JWTCredentials::new(256286, der)?;

    // let app_client = Client::new("iter", Credentials::JWT(credentials.clone()))?;
    // let first_installation = app_client.apps()
    //     .list_installations(10, 1, None, "")
    //     .await?
    //     .into_iter()
    //     .next()
    //     .unwrap();

    // let token_generator = InstallationTokenGenerator::new(first_installation.id as u64, credentials);

    // let github = Client::new("iter", Credentials::InstallationToken(token_generator))?;

    // let repo = github.repos().get("AlbertMarashi", "framework").await?;

    // let user = github.users().get_by_username("yifan117").await?;

    // println!("{:#?}", repo);
    // println!("{:#?}", user);

    Ok(())
}