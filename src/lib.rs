use std::{
    io::{Read, Write},
    net::Shutdown,
    os::unix::net::UnixStream,
};

use rmcp::{
    ServerHandler,
    handler::server::{tool::ToolRouter, wrapper::Parameters},
    model::{Implementation, ServerCapabilities, ServerInfo},
    schemars::JsonSchema,
    tool, tool_handler, tool_router,
};
use serde::Deserialize;

#[derive(Clone)]
pub struct HyprlandMcpServer {
    sock: String,
    tool_router: ToolRouter<Self>,
}

#[derive(Deserialize, JsonSchema)]
pub struct DispatchRequest {
    /// Raw hyprctl socket command passed through verbatim.
    ///
    /// Examples:
    /// - "dispatch exec kitty"
    /// - "activewindow"
    command: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct WorkspaceRequest {
    /// Target workspace number (1-based).
    n: usize,
}

impl HyprlandMcpServer {
    pub fn new() -> Result<Self, std::env::VarError> {
        let xdg = std::env::var("XDG_RUNTIME_DIR")?;
        let instance = std::env::var("HYPRLAND_INSTANCE_SIGNATURE")?;
        let sock = format!("{xdg}/hypr/{instance}/.socket.sock");

        Ok(Self {
            sock,
            tool_router: Self::tool_router(),
        })
    }

    pub fn cmd(&self, cmd: &str) -> Result<String, std::io::Error> {
        let mut sock = UnixStream::connect(&self.sock)?;
        sock.write_all(cmd.as_bytes())?;
        sock.shutdown(Shutdown::Write)?;

        let mut out = String::new();
        sock.read_to_string(&mut out)?;

        Ok(out)
    }
}

#[tool_router]
impl HyprlandMcpServer {
    #[tool(description = "Run a raw hyprctl-style command via Hyprland socket")]
    async fn hyprctl(
        &self,
        Parameters(DispatchRequest { command }): Parameters<DispatchRequest>,
    ) -> Result<String, String> {
        self.cmd(&command).map_err(|e| e.to_string())
    }

    #[tool(description = "Switch to a numbered workspace")]
    async fn workspace(
        &self,
        Parameters(WorkspaceRequest { n }): Parameters<WorkspaceRequest>,
    ) -> Result<String, String> {
        if n == 0 {
            return Err("workspace must be >= 1".to_string());
        }

        let command = format!("dispatch workspace {n}");
        self.cmd(&command).map_err(|e| e.to_string())
    }
}

#[tool_handler]
impl ServerHandler for HyprlandMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: env!("CARGO_PKG_NAME").to_string(),
                title: Some("Hyprland MCP".to_string()),
                version: env!("CARGO_PKG_VERSION").to_string(),
                description: Some(
                    "Local MCP server for controlling Hyprland through its unix socket.".to_string(),
                ),
                icons: None,
                website_url: Some("https://wiki.hypr.land".to_string()),
            },
            instructions: Some(
                "Use `workspace` to switch workspaces (1-based). Use `hyprctl` for raw hyprctl-style socket commands such as `dispatch exec kitty`, `activewindow`, or `clients`."
                    .to_string(),
            ),
            ..ServerInfo::default()
        }
    }
}
