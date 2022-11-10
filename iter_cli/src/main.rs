//! # Iter CLI Tool
//!
//! This is a CLI tool for the interation of the iter platform
//!
//! ## Commands
//! - [ ] iter setup - requires a valid kubeconfig (choose from a list of available kubeconfigs)
//!

mod cli_types;
mod cli_kube;
mod commands;
mod utils;

use clap::Parser;
use commands::install::install_command;


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