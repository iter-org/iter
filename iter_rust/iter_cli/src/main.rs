//! # Iter CLI Tool
//!
//! This is a CLI tool for the interation of the iter platform
//!
//! ## Commands
//! - [ ] iter setup - requires a valid kubeconfig (choose from a list of available kubeconfigs)
//!

mod cli_kube;
mod commands;
mod utils;
mod config;

use commands::{init::InitCommand, install::InstallCommand, RunnableCommand, ping::PingCommand};

use clap::{Parser, Subcommand};
use dialoguer::console::style;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    cli(std::env::args_os()
        .into_iter()
        .map(|x| x.to_string_lossy().into_owned()))
    .await
}

async fn cli(args: impl Iterator<Item = String>) -> Result<(), anyhow::Error> {
    let cmd = IterCLI::parse_from(args);

    match cmd.command.run().await {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!(
                "{} {}",
                style("✖").red().bold(),
                style(e.to_string()).bold()
            );
            // if we are in debug mode, panic with the error and stack trace
            if cfg!(debug_assertions) || cmd.debug {
                panic!("{:?}", e);
            }

            std::process::exit(1);
        }
    }
}
#[derive(Parser, Debug, Clone)]
#[command(
    author = "Iter",
    version = "0.0.1",
    about = "auto setups your kubernetes and github projects managable by through the frontend, API or the CLI",
    long_about = None
)]
pub struct IterCLI {
    #[command(subcommand)]
    pub command: Command,
    #[clap(short, long)]
    pub debug: bool,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    /// setups secrets and kubls
    Install(InstallCommand),
    Init(InitCommand),
    Ping(PingCommand)
}

#[async_trait::async_trait]
impl RunnableCommand for Command {
    async fn run(self) -> Result<(), anyhow::Error> {
        match self {
            Command::Install(command) => command.run().await,
            Command::Init(command) => command.run().await,
            Command::Ping(command) => command.run().await,
        }
    }
}
