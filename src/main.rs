use anyhow::Result;
use rmcp::{transport::stdio, ServiceExt};
use tracing_subscriber::EnvFilter;

use gitx_mcp::config::Config;
use gitx_mcp::server::GitxMcp;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing (logs go to stderr to keep stdout clean for MCP)
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    let config = Config::from_env()?;
    let service = GitxMcp::new(config)?;
    let server = service.serve(stdio()).await?;
    server.waiting().await?;

    Ok(())
}
