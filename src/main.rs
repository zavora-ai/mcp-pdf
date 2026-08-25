//! mcp-pdf binary entry point.

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use rmcp::{transport::stdio, ServiceExt};
    mcp_pdf::server::PdfServer
        .serve(stdio())
        .await?
        .waiting()
        .await?;
    Ok(())
}
