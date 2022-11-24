pub mod install;
pub mod init;
pub mod ping;

#[async_trait::async_trait]
pub trait RunnableCommand {
    async fn run(self) -> Result<(), anyhow::Error>;
}