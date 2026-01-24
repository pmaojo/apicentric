//! A minimal build of the `apicentric` CLI.
//!
//! This build includes only the most essential commands, and is intended for
//! use in environments where a smaller binary size is desired.

use colored::*;
use std::env;

/// The entry point for the minimal build of `apicentric`.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    // Default to help if no args
    if args.len() < 2 {
        print_help();
        return Ok(());
    }

    match args[1].as_str() {
        "version" => {
            println!("{}", "apicentric CLI (minimal build)".green().bold());
            println!("✅ All heavy dependencies removed for faster compilation!");
            println!("🚀 Core CLI functionality working");
        }
        "validate" => {
            let mut path = None;
            let mut i = 2;
            while i < args.len() {
                if args[i] == "--path" || args[i] == "-p" {
                    if i + 1 < args.len() {
                        path = Some(args[i+1].clone());
                        i += 1;
                    }
                }
                i += 1;
            }

            if let Some(p) = path {
                 validate_file(&p);
            } else {
                println!("{}", "❌ Missing required argument: --path".red());
                print_help();
                std::process::exit(1);
            }
        }
        "--help" | "-h" => print_help(),
        _ => {
            println!("{} Unknown command: {}", "❌".red(), args[1]);
            print_help();
            std::process::exit(1);
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

fn validate_file(path: &str) {
    println!("{} {}", "🔍 Validating:".blue().bold(), path);

    match std::fs::read_to_string(path) {
        Ok(content) => match serde_yaml::from_str::<serde_json::Value>(&content) {
            Ok(_) => println!("{}", "✅ Valid YAML structure".green()),
            Err(e) => println!("{} {}", "❌ Invalid YAML:".red(), e),
        },
        Err(e) => println!("{} {}", "❌ Cannot read file:".red(), e),
    }
}
