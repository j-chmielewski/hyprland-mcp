use tokio::io::{stdin, stdout};

use rmcp::ServiceExt;
use hyprland_mcp::server::HyprlandMcpServer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let transport = (stdin(), stdout());
    let server = HyprlandMcpServer::new()?;
    server.serve(transport).await?;

    Ok(())
}
