use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    author = "The Federation of Framework",
    version = "1.0.0",
    about = "auto setups your kubernetes and github projects managable by a frontend URL or the CLI",
    long_about = None
)]
pub struct IterCLI {


    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// setups secrets and kubls
    Install {
        /// the domain of the project
        #[arg(short, long)]
        domain: Option<String>,
        /// github app secret
        #[arg(short, long)]
        github_secret: Option<String>,
    },
    /// deploys a project
    Deploy {

    },




}

// iter setup {domain} {github secrets}