use anyhow::{Context, Result};
use std::process::{Command, Output};

pub fn run_command(cmd: &str, args: &[&str]) -> Result<Output> {
    Command::new(cmd)
        .args(args)
        .output()
        .with_context(|| format!("Failed to execute command: {} {:?}", cmd, args))
}

pub fn run_command_with_shell(cmd: &str) -> Result<Output> {
    if cfg!(target_os = "windows") {
        Command::new("cmd").args(["/C", cmd]).output()
    } else {
        Command::new("sh").args(["-c", cmd]).output()
    }
    .with_context(|| format!("Failed to execute shell command: {}", cmd))
}

pub fn get_command_output(output: Output) -> Result<String> {
    if output.status.success() {
        String::from_utf8(output.stdout).context("Failed to decode command output")
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Command failed: {}", stderr)
    }
}
