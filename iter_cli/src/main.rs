//! # Iter CLI Tool
//!
//! This is a CLI tool for the interation of the iter platform
//!
//! ## Commands
//! - [ ] iter setup - requires a valid kubeconfig (choose from a list of available kubeconfigs)
//! 


use clap::Parser;
use cli_kube::create_install_secrets;
use dialoguer::{theme::ColorfulTheme, Input};
use serde_json::json;
mod cli_types;
mod cli_kube;


fn main() {
    cli(std::env::args_os().into_iter().map(|x| x.to_string_lossy().into_owned()));
}

fn cli(args: impl Iterator<Item = String>) {
    let cli = cli_types::IterCLI::parse_from(args);
    match cli {
        cli_types::IterCLI { command } => {
            match command {
                cli_types::Command::Install(install_cmd) => install_command(install_cmd),
                cli_types::Command::Deploy {  } => {
                    unimplemented!()
                },
            }
        }
    };
}

async fn install_command(cli_types::InstallCommand { domain, github_secret }: cli_types::InstallCommand) {
    let domain = unwrap_or_prompt(domain, "Iter domain");
    let github_secret = unwrap_or_prompt(github_secret, "Github App secret");
    let json_secret = json!(
        {
            "domain": domain,
            "github_secret": github_secret
        }
    );
    let parsed_secret = create_install_secrets(json_secret, "iter-secrets", "iter").await;

    // println!("domain: {}", domain);
    // println!("github_secret: {}", github_secret);
}

fn unwrap_or_prompt(arg: Option<String>, prompt: &str) -> String {
    match arg {
        Some(arg) => arg,
        None => request_missing_arg(prompt),
    }
}

fn request_missing_arg(prompt: &str) -> String {
    return Input::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .interact()
        .unwrap();
}

#[test]
fn test_cli() {
    let args = vec![
        "iter".to_string(),
        "install".to_string(),
        "-d".to_string(),
        "domain".to_string(),
        "-g".to_string(),
        "github_secret".to_string()];
    cli(args.into_iter());
}