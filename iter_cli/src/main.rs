//! # Iter CLI Tool
//!
//! This is a CLI tool for the interation of the iter platform
//!
//! ## Commands
//! - [ ] iter setup - requires a valid kubeconfig (choose from a list of available kubeconfigs)
//! 

use clap::Parser;
mod cli_types;


fn main() {
    cli(std::env::args_os().into_iter().map(|x| x.to_string_lossy().into_owned()));
}

fn cli(args: impl Iterator<Item = String>) {
    let cli = cli_types::IterCLI::parse_from(args);
    // match cli {

    // }
    println!("{:#?}", cli);
}

#[test]
fn test_cli() {
    let args = vec!["iter".to_string(), "install".to_string(), "-d".to_string(), "domain".to_string(), "-g".to_string(), "github_secret".to_string()];
    cli(args.into_iter());
}