use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug, Clone)]
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

#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    /// setups secrets and kubls
    Install(InstallCommand),
    /// deploys a project
    Deploy {

    },
}


#[derive(Parser, Debug, Clone)]
pub struct InstallCommand { 
    /// the domain of the project
    #[arg(short, long)]
    pub domain: Option<String>,
    /// github app secret
    #[arg(short, long)]
    pub github_secret: Option<String>,
}

// iter setup {domain} {github secrets}