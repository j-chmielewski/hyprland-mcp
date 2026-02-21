use tokio::io::{stdin, stdout};

use hyprland_mcp::server::HyprlandMcpServer;
use rmcp::ServiceExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let transport = (stdin(), stdout());
    let server = HyprlandMcpServer::new()?;
    let running = server.serve(transport).await?;
    running.waiting().await?;

    Ok(())
}
