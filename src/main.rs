mod server;

use rmcp::{ServiceExt, transport::stdio};
use server::PdfServer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_env_filter(tracing_subscriber::EnvFilter::from_default_env()).init();
    tracing::info!("mcp-pdf starting");
    let service = PdfServer.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
