mod server;
mod tools;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use rmcp::{ServiceExt, transport::stdio};
    server::PdfServer.serve(stdio()).await?.waiting().await?;
    Ok(())
}
