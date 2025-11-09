//! A minimal build of the `apicentric` CLI.
//!
//! This build includes only the most essential commands, and is intended for
//! use in environments where a smaller binary size is desired.

use clap::{Parser, Subcommand};
use colored::*;

/// The command-line interface for the minimal build of `apicentric`.
#[derive(Parser)]
#[command(author, version, about = "apicentric CLI (minimal build)")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// The commands available in the minimal build of `apicentric`.
#[derive(Subcommand)]
enum Commands {
    /// Shows the version and build info.
    Version,
    /// Validates a YAML service definition.
    Validate {
        /// The path to the YAML file.
        #[arg(short, long)]
        path: String,
    },
}

/// The entry point for the minimal build of `apicentric`.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Version => {
            println!("{}", "apicentric CLI v0.1.1 (minimal build)".green().bold());
            println!("✅ All heavy dependencies removed for faster compilation!");
            println!("🚀 Core CLI functionality working");
        }
        Commands::Validate { path } => {
            println!("{} {}", "🔍 Validating:".blue().bold(), path);
            
            match std::fs::read_to_string(&path) {
                Ok(content) => {
                    match serde_yaml::from_str::<serde_json::Value>(&content) {
                        Ok(_) => println!("{}", "✅ Valid YAML structure".green()),
                        Err(e) => println!("{} {}", "❌ Invalid YAML:".red(), e),
                    }
                }
                Err(e) => println!("{} {}", "❌ Cannot read file:".red(), e),
            }
        }
    }

    Ok(())
}
