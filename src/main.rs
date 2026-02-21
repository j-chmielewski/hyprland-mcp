use hyprland_mcp::cmd;

fn main() -> anyhow::Result<()> {
    // let out = cmd("dispatch workspace 1")?;
    let out = cmd("activewindow")?;
    println!("OUT: {out}");

    Ok(())
}
