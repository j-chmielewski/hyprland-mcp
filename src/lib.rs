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
pub struct HyprctlRequest {
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

#[derive(Deserialize, JsonSchema)]
pub struct MonitorsRequest {
    /// Include inactive monitors.
    all: Option<bool>,
}

#[derive(Deserialize, JsonSchema)]
pub struct DecorationsRequest {
    /// Window regex.
    window_regex: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct DismissNotifyRequest {
    /// Optional number of notifications to dismiss.
    amount: Option<usize>,
}

#[derive(Deserialize, JsonSchema)]
pub struct DispatchCommandRequest {
    /// Dispatcher and arguments, e.g. "workspace 1" or "exec kitty".
    args: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct OptionNameRequest {
    /// Config option name.
    option: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct KeywordRequest {
    /// Keyword name.
    name: String,
    /// Keyword value.
    value: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct NotifyRequest {
    /// Icon ID used by Hyprland notifications.
    icon: i32,
    /// Timeout in milliseconds.
    timeout_ms: u32,
    /// Color string, e.g. "rgb(7fd16a)".
    color: String,
    /// Notification message body.
    message: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct RawArgsRequest {
    /// Raw arguments passed to the subcommand.
    args: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct ReloadRequest {
    /// If true, run "reload config-only".
    config_only: Option<bool>,
}

#[derive(Deserialize, JsonSchema)]
pub struct RollingLogRequest {
    /// Follow rolling log stream.
    follow: Option<bool>,
}

#[derive(Deserialize, JsonSchema)]
pub struct SetCursorRequest {
    theme: String,
    size: u32,
}

#[derive(Deserialize, JsonSchema)]
pub struct SetErrorRequest {
    color: String,
    message: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct SwitchXkbLayoutRequest {
    keyboard: String,
    command: String,
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

    fn quote(arg: &str) -> String {
        format!("\"{}\"", arg.replace('"', "\\\""))
    }
}

#[tool_router]
impl HyprlandMcpServer {
    #[tool(description = "Run a raw hyprctl-style command via Hyprland socket")]
    async fn hyprctl(
        &self,
        Parameters(HyprctlRequest { command }): Parameters<HyprctlRequest>,
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

        self.hyprctl(Parameters(HyprctlRequest {
            command: format!("dispatch workspace {n}"),
        }))
        .await
    }

    #[tool(description = "Get active window info")]
    async fn activewindow(&self) -> Result<String, String> {
        self.hyprctl(Parameters(HyprctlRequest {
            command: "activewindow".to_string(),
        }))
        .await
    }

    #[tool(description = "Get active workspace info")]
    async fn activeworkspace(&self) -> Result<String, String> {
        self.hyprctl(Parameters(HyprctlRequest {
            command: "activeworkspace".to_string(),
        }))
        .await
    }

    #[tool(description = "Get animation and bezier info")]
    async fn animations(&self) -> Result<String, String> {
        self.hyprctl(Parameters(HyprctlRequest {
            command: "animations".to_string(),
        }))
        .await
    }

    #[tool(description = "List registered binds")]
    async fn binds(&self) -> Result<String, String> {
        self.hyprctl(Parameters(HyprctlRequest {
            command: "binds".to_string(),
        }))
        .await
    }

    #[tool(description = "List clients")]
    async fn clients(&self) -> Result<String, String> {
        self.hyprctl(Parameters(HyprctlRequest {
            command: "clients".to_string(),
        }))
        .await
    }

    #[tool(description = "List config errors")]
    async fn configerrors(&self) -> Result<String, String> {
        self.hyprctl(Parameters(HyprctlRequest {
            command: "configerrors".to_string(),
        }))
        .await
    }

    #[tool(description = "Get cursor position")]
    async fn cursorpos(&self) -> Result<String, String> {
        self.hyprctl(Parameters(HyprctlRequest {
            command: "cursorpos".to_string(),
        }))
        .await
    }

    #[tool(description = "List decorations for a window regex")]
    async fn decorations(
        &self,
        Parameters(DecorationsRequest { window_regex }): Parameters<DecorationsRequest>,
    ) -> Result<String, String> {
        self.hyprctl(Parameters(HyprctlRequest {
            command: format!("decorations {window_regex}"),
        }))
        .await
    }

    #[tool(description = "Dismiss all or a specific amount of notifications")]
    async fn dismissnotify(
        &self,
        Parameters(DismissNotifyRequest { amount }): Parameters<DismissNotifyRequest>,
    ) -> Result<String, String> {
        let command = match amount {
            Some(n) => format!("dismissnotify {n}"),
            None => "dismissnotify".to_string(),
        };

        self.hyprctl(Parameters(HyprctlRequest { command })).await
    }

    #[tool(description = "Issue a Hyprland dispatch command")]
    async fn dispatch(
        &self,
        Parameters(DispatchCommandRequest { args }): Parameters<DispatchCommandRequest>,
    ) -> Result<String, String> {
        self.hyprctl(Parameters(HyprctlRequest {
            command: format!("dispatch {args}"),
        }))
        .await
    }

    #[tool(description = "Get config option value")]
    async fn getoption(
        &self,
        Parameters(OptionNameRequest { option }): Parameters<OptionNameRequest>,
    ) -> Result<String, String> {
        self.hyprctl(Parameters(HyprctlRequest {
            command: format!("getoption {option}"),
        }))
        .await
    }

    #[tool(description = "List global shortcuts")]
    async fn globalshortcuts(&self) -> Result<String, String> {
        self.hyprctl(Parameters(HyprctlRequest {
            command: "globalshortcuts".to_string(),
        }))
        .await
    }

    #[tool(description = "List Hyprland instances")]
    async fn instances(&self) -> Result<String, String> {
        self.hyprctl(Parameters(HyprctlRequest {
            command: "instances".to_string(),
        }))
        .await
    }

    #[tool(description = "Dynamically set a Hyprland keyword")]
    async fn keyword(
        &self,
        Parameters(KeywordRequest { name, value }): Parameters<KeywordRequest>,
    ) -> Result<String, String> {
        self.hyprctl(Parameters(HyprctlRequest {
            command: format!("keyword {name} {value}"),
        }))
        .await
    }

    #[tool(description = "Enter Hyprland kill mode")]
    async fn kill(&self) -> Result<String, String> {
        self.hyprctl(Parameters(HyprctlRequest {
            command: "kill".to_string(),
        }))
        .await
    }

    #[tool(description = "List layer surfaces")]
    async fn layers(&self) -> Result<String, String> {
        self.hyprctl(Parameters(HyprctlRequest {
            command: "layers".to_string(),
        }))
        .await
    }

    #[tool(description = "List available layouts")]
    async fn layouts(&self) -> Result<String, String> {
        self.hyprctl(Parameters(HyprctlRequest {
            command: "layouts".to_string(),
        }))
        .await
    }

    #[tool(description = "List monitors")]
    async fn monitors(
        &self,
        Parameters(MonitorsRequest { all }): Parameters<MonitorsRequest>,
    ) -> Result<String, String> {
        let command = if all.unwrap_or(false) {
            "monitors all".to_string()
        } else {
            "monitors".to_string()
        };

        self.hyprctl(Parameters(HyprctlRequest { command })).await
    }

    #[tool(description = "Send a Hyprland notification")]
    async fn notify(
        &self,
        Parameters(NotifyRequest {
            icon,
            timeout_ms,
            color,
            message,
        }): Parameters<NotifyRequest>,
    ) -> Result<String, String> {
        let command = format!(
            "notify {icon} {timeout_ms} {color} {}",
            Self::quote(&message)
        );
        self.hyprctl(Parameters(HyprctlRequest { command })).await
    }

    #[tool(description = "Issue an output subcommand")]
    async fn output(
        &self,
        Parameters(RawArgsRequest { args }): Parameters<RawArgsRequest>,
    ) -> Result<String, String> {
        self.hyprctl(Parameters(HyprctlRequest {
            command: format!("output {args}"),
        }))
        .await
    }

    #[tool(description = "Issue a plugin subcommand")]
    async fn plugin(
        &self,
        Parameters(RawArgsRequest { args }): Parameters<RawArgsRequest>,
    ) -> Result<String, String> {
        self.hyprctl(Parameters(HyprctlRequest {
            command: format!("plugin {args}"),
        }))
        .await
    }

    #[tool(description = "Reload Hyprland config")]
    async fn reload(
        &self,
        Parameters(ReloadRequest { config_only }): Parameters<ReloadRequest>,
    ) -> Result<String, String> {
        let command = if config_only.unwrap_or(false) {
            "reload config-only".to_string()
        } else {
            "reload".to_string()
        };

        self.hyprctl(Parameters(HyprctlRequest { command })).await
    }

    #[tool(description = "Get rolling log")]
    async fn rollinglog(
        &self,
        Parameters(RollingLogRequest { follow }): Parameters<RollingLogRequest>,
    ) -> Result<String, String> {
        let command = if follow.unwrap_or(false) {
            "rollinglog -f".to_string()
        } else {
            "rollinglog".to_string()
        };

        self.hyprctl(Parameters(HyprctlRequest { command })).await
    }

    #[tool(description = "Set cursor theme and size")]
    async fn setcursor(
        &self,
        Parameters(SetCursorRequest { theme, size }): Parameters<SetCursorRequest>,
    ) -> Result<String, String> {
        self.hyprctl(Parameters(HyprctlRequest {
            command: format!("setcursor {theme} {size}"),
        }))
        .await
    }

    #[tool(description = "Set Hyprland error string")]
    async fn seterror(
        &self,
        Parameters(SetErrorRequest { color, message }): Parameters<SetErrorRequest>,
    ) -> Result<String, String> {
        let command = format!("seterror {color} {}", Self::quote(&message));
        self.hyprctl(Parameters(HyprctlRequest { command })).await
    }

    #[tool(description = "Set window property")]
    async fn setprop(
        &self,
        Parameters(RawArgsRequest { args }): Parameters<RawArgsRequest>,
    ) -> Result<String, String> {
        self.hyprctl(Parameters(HyprctlRequest {
            command: format!("setprop {args}"),
        }))
        .await
    }

    #[tool(description = "Get window property")]
    async fn getprop(
        &self,
        Parameters(RawArgsRequest { args }): Parameters<RawArgsRequest>,
    ) -> Result<String, String> {
        self.hyprctl(Parameters(HyprctlRequest {
            command: format!("getprop {args}"),
        }))
        .await
    }

    #[tool(description = "Get splash text")]
    async fn splash(&self) -> Result<String, String> {
        self.hyprctl(Parameters(HyprctlRequest {
            command: "splash".to_string(),
        }))
        .await
    }

    #[tool(description = "Switch keyboard layout")]
    async fn switchxkblayout(
        &self,
        Parameters(SwitchXkbLayoutRequest { keyboard, command }): Parameters<SwitchXkbLayoutRequest>,
    ) -> Result<String, String> {
        self.hyprctl(Parameters(HyprctlRequest {
            command: format!("switchxkblayout {keyboard} {command}"),
        }))
        .await
    }

    #[tool(description = "Get system info")]
    async fn systeminfo(&self) -> Result<String, String> {
        self.hyprctl(Parameters(HyprctlRequest {
            command: "systeminfo".to_string(),
        }))
        .await
    }

    #[tool(description = "Get Hyprland version")]
    async fn version(&self) -> Result<String, String> {
        self.hyprctl(Parameters(HyprctlRequest {
            command: "version".to_string(),
        }))
        .await
    }

    #[tool(description = "List workspace rules")]
    async fn workspacerules(&self) -> Result<String, String> {
        self.hyprctl(Parameters(HyprctlRequest {
            command: "workspacerules".to_string(),
        }))
        .await
    }

    #[tool(description = "List workspaces")]
    async fn workspaces(&self) -> Result<String, String> {
        self.hyprctl(Parameters(HyprctlRequest {
            command: "workspaces".to_string(),
        }))
        .await
    }

    #[tool(description = "Issue a hyprpaper request")]
    async fn hyprpaper(
        &self,
        Parameters(RawArgsRequest { args }): Parameters<RawArgsRequest>,
    ) -> Result<String, String> {
        self.hyprctl(Parameters(HyprctlRequest {
            command: format!("hyprpaper {args}"),
        }))
        .await
    }

    #[tool(description = "Issue a hyprsunset request")]
    async fn hyprsunset(
        &self,
        Parameters(RawArgsRequest { args }): Parameters<RawArgsRequest>,
    ) -> Result<String, String> {
        self.hyprctl(Parameters(HyprctlRequest {
            command: format!("hyprsunset {args}"),
        }))
        .await
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
                "Use dedicated tools for each hyprctl subcommand. For advanced behavior, use `hyprctl` with a raw command string."
                    .to_string(),
            ),
            ..ServerInfo::default()
        }
    }
}
