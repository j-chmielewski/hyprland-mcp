use std::{
    io::{Read, Write},
    net::Shutdown,
    os::unix::net::UnixStream,
    sync::LazyLock,
};

use thiserror::Error;

static SOCK: LazyLock<String> = LazyLock::new(|| {
    let xdg = std::env::var("XDG_RUNTIME_DIR").expect("Failed to read $XDG_RUNTIME_DIR");
    let instance = std::env::var("HYPRLAND_INSTANCE_SIGNATURE")
        .expect("Failed to read $HYPRLAND_INSTANCE_SIGNATURE");
    format!("{xdg}/hypr/{instance}/.socket.sock")
});

#[derive(Error, Debug)]
pub enum McpError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

type Result<T> = std::result::Result<T, McpError>;

fn open() -> Result<UnixStream> {
    Ok(UnixStream::connect(&*SOCK)?)
}

pub fn cmd(cmd: &str) -> Result<String> {
    let mut f = open()?;
    f.write_all(cmd.as_bytes())?;
    f.shutdown(Shutdown::Write)?;

    let mut out = String::new();
    f.read_to_string(&mut out)?;

    Ok(out)
}
