//! A minimal build of the `apicentric` CLI.
//!
//! This build includes only the most essential commands, and is intended for
//! use in environments where a smaller binary size is desired.

use colored::*;
use std::env;
use std::error::Error;

/// The entry point for the minimal build of `apicentric`.
fn main() {
    if let Err(e) = run() {
        eprintln!("{} {}", "❌ Error:".red(), e);
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let mut iter = args.iter().skip(1); // Skip program name

    let command = match iter.next() {
        Some(cmd) => cmd,
        None => {
            print_help();
            return Ok(());
        }
    };

    match command.as_str() {
        "version" => {
            println!("{}", "apicentric CLI (minimal build)".green().bold());
            println!("✅ All heavy dependencies removed for faster compilation!");
            println!("🚀 Core CLI functionality working");
        }
        "validate" => {
            let mut path = None;
            while let Some(arg) = iter.next() {
                match arg.as_str() {
                    "--path" | "-p" => {
                        if let Some(p) = iter.next() {
                            path = Some(p.clone());
                        } else {
                            return Err("Missing value for --path".into());
                        }
                    }
                    _ => {
                        return Err(format!("Unknown argument: {}", arg).into());
                    }
                }
            }

            if let Some(p) = path {
                validate_file(&p)?;
            } else {
                return Err("Missing required argument: --path".into());
            }
        }
        "--help" | "-h" => print_help(),
        _ => {
            return Err(format!("Unknown command: {}", command).into());
        }
    }

    Ok(())
}

fn print_help() {
    println!("Usage: apicentric-minimal <COMMAND>");
    println!();
    println!("Commands:");
    println!("  version      Show version info");
    println!("  validate     Validate a YAML service definition (--path <FILE>)");
}

fn validate_file(path: &str) -> Result<(), Box<dyn Error>> {
    println!("{} {}", "🔍 Validating:".blue().bold(), path);

    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Cannot read file: {}", e))?;

    serde_yaml::from_str::<serde_json::Value>(&content)
        .map_err(|e| format!("Invalid YAML: {}", e))?;

    println!("{}", "✅ Valid YAML structure".green());
    Ok(())
}
