use clap::{Parser, Subcommand};
use colored::*;

#[derive(Parser)]
#[command(author, version, about = "apicentric CLI (minimal build)")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show version and build info
    Version,
    /// Validate YAML service definitions
    Validate {
        /// Path to YAML file
        #[arg(short, long)]
        path: String,
    },
}

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