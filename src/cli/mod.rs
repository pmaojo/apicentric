//! The command-line interface for `apicentric`.
//!
//! This module provides the command-line interface for `apicentric`, replacing
//! the previous `clap`-based implementation with a custom parser for reduced dependencies.

pub mod args;
pub mod parser;

#[cfg(feature = "iot")]
pub use args::TwinCommands;
pub use args::{AiAction, Cli, Commands, SimulatorAction};

/// Parses the command-line arguments into a `Cli` instance.
pub fn parse() -> Cli {
    parser::parse()
}
