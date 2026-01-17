use apicentric::ApicentricResult;
use colored::*;
use std::process::Command;

pub async fn open_command(port: Option<u16>) -> ApicentricResult<()> {
    // Default to 9002 if not provided
    let target_port = port.unwrap_or(9002);
    let url = format!("http://localhost:{}", target_port);

    println!("🌐 Opening WebUI at {}...", url.cyan());

    let open_result = if cfg!(target_os = "windows") {
        Command::new("cmd").args(["/C", "start", &url]).status()
    } else if cfg!(target_os = "macos") {
        Command::new("open").arg(&url).status()
    } else {
        Command::new("xdg-open").arg(&url).status()
    };

    match open_result {
        Ok(status) if status.success() => {
            println!("✅ Launched browser successfully");
        }
        _ => {
            println!("{} Could not open browser automatically.", "⚠️".yellow());
            println!("Please visit: {}", url.underline());
        }
    }

    Ok(())
}
