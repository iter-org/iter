//! # Iter CLI Tool
//!
//! This is a CLI tool for the interation of the iter platform
//!
//! ## Commands
//! - [ ] iter setup - requires a valid kubeconfig (choose from a list of available kubeconfigs)
//! 


use clap::Parser;

use cli_kube::create_or_update_kube_secrets;
use dialoguer::{theme::ColorfulTheme, Input, console::style};
use serde_json::json;
mod cli_types;
mod cli_kube;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> { 
    cli(std::env::args_os().into_iter().map(|x| x.to_string_lossy().into_owned())).await
}

async fn cli(args: impl Iterator<Item = String>) -> Result<(), anyhow::Error> {
    match cli_types::IterCLI::parse_from(args).command {
        cli_types::Command::Install(install_cmd) => install_command(install_cmd).await,
        cli_types::Command::Deploy {  } => unimplemented!()
    }
}

async fn install_command(cli_types::InstallCommand { domain, github_secret }: cli_types::InstallCommand) -> Result<(), anyhow::Error> {
    create_or_update_kube_secrets(json!(
        {
            "domain": unwrap_or_prompt(domain, "Iter domain")?,
            "github_secret": unwrap_or_prompt(github_secret, "Github App secret")?
        }
    ), "iter-secrets", "iter").await?;

    println!("{} {}",
        style("✔").green().bold(),
        style("Iter Install Completed").blue().bold(),
    );
    
    Ok(())
}

fn unwrap_or_prompt(arg: Option<String>, prompt: &str) -> Result<String, anyhow::Error> {
    match arg {
        Some(arg) => Ok(arg),
        None => request_missing_arg(prompt),
    }
}

fn request_missing_arg(prompt: &str) -> Result<String, anyhow::Error> {
    return Input::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .interact()
        .map_err(|e| anyhow::anyhow!(e));
}

#[tokio::test]
async fn test_cli() -> Result<(), anyhow::Error> {
    let args = vec![
        "iter".to_string(),
        "install".to_string(),
        "-d".to_string(),
        "domain".to_string(),
        "-g".to_string(),
        "github_secret".to_string()];

    cli(args.into_iter()).await
}
#[tokio::test]
async fn test_cli_without_args() -> Result<(), anyhow::Error> {
    let args = vec![
        "iter".to_string(),
        "install".to_string()];
        
    cli(args.into_iter()).await
}