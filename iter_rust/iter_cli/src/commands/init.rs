use std::path::PathBuf;

use clap::Parser;
use json_value_merge::Merge;
use path_clean::PathClean;

use crate::utils::unwrap_or_prompt;

use super::RunnableCommand;



#[derive(Parser, Debug, Clone)]
pub struct InitCommand {
    /// the endpoint of the iter api server
    #[arg(short, long)]
    pub endpoint: Option<String>,

    #[arg(value_name = "DIRECTORY")]
    pub directory: Option<String>,
}


#[async_trait::async_trait]
impl RunnableCommand for InitCommand {
    async fn run(self) -> Result<(), anyhow::Error> {
        let InitCommand { endpoint, directory } = self;
        let endpoint = unwrap_or_prompt(endpoint, "iter endpoint")?;
        let mut directory = PathBuf::from(directory.unwrap_or(".".to_string()));

        // resolve the directory to an absolute path
        // check if the directory is an absolute path
        if !directory.is_absolute() {
            let current_dir = std::env::current_dir()?;
            directory = current_dir.join(directory).clean();
        }
        
        // create the directory if it doesn't exist
        if !directory.exists() {
            std::fs::create_dir_all(&directory)?;
        }

        // create the iter.json file
        let mut iter_json = serde_json::json!({
            "endpoint": endpoint,
        });

        // read the existing iter.json file if it exists, and merge it with the new iter.json file
        let iter_json_file = directory.join("iter.json");
        if iter_json_file.exists() {
            let iter_json_file = std::fs::File::open(&iter_json_file)?;
            let mut existing_json: serde_json::Value = serde_json::from_reader(iter_json_file)?;
            existing_json.merge(iter_json);
            iter_json = existing_json;
        }

        // write the iter.json file
        let iter_json_file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(iter_json_file)?;

        serde_json::to_writer_pretty(iter_json_file, &iter_json)?;

        Ok(())
    }
}


#[cfg(test)]
mod test {
    use crate::cli;
    #[tokio::test]
    async fn test_command() -> Result<(), anyhow::Error> {
        let args = vec![
            "iter".to_string(),
            "init".to_string(),
            "-e".to_string(),
            "endpoint_url".to_string(),
        ];

        cli(args.into_iter()).await
    }
    #[tokio::test]
    async fn test_command_without_args() -> Result<(), anyhow::Error> {
        let args = vec![
            "iter".to_string(),
            "init".to_string()];

        cli(args.into_iter()).await
    }
}
