use std::{
    io::{Read, Write},
    net::Shutdown,
    os::unix::net::UnixStream,
};

use rmcp::{
    ServerHandler,
    handler::server::{tool::ToolRouter, wrapper::Parameters},
    model::{ServerCapabilities, ServerInfo},
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
    command: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct WorkspaceRequest {
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
    #[tool(description = "Dispatch hyprland command")]
    async fn dispatch(
        &self,
        Parameters(DispatchRequest { command }): Parameters<DispatchRequest>,
    ) -> Result<String, String> {
        self.cmd(&command).map_err(|e| e.to_string())
    }

    #[tool(description = "Go to workspace n")]
    async fn workspace(
        &self,
        Parameters(WorkspaceRequest { n }): Parameters<WorkspaceRequest>,
    ) -> Result<String, String> {
        let command = format!("dispatch workspace {n}");
        self.cmd(&command).map_err(|e| e.to_string())
    }
}

#[tool_handler]
impl ServerHandler for HyprlandMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..ServerInfo::default()
        }
    }
}
