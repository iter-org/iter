use clap::Parser;

use super::RunnableCommand;

#[derive(Parser, Debug, Clone)]
pub struct PingCommand;

#[async_trait::async_trait]
impl RunnableCommand for PingCommand {
    async fn run(self) -> Result<(), anyhow::Error> {
        let config = crate::utils::load_config()?;

        let client = iter_lib::Client::new(config.endpoint);

        let response = client.ping().await?;

        println!("{}", response);
        
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
            "ping".to_string(),
        ];

        cli(args.into_iter()).await
    }
}
