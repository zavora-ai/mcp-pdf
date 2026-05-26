//! mcp-pdf binary entry point.

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use rmcp::{ServiceExt, transport::stdio};
    mcp_pdf::server::PdfServer.serve(stdio()).await?.waiting().await?;
    Ok(())
}
