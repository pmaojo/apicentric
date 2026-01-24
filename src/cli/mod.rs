//! The command-line interface for `apicentric`.
//!
//! This module provides the command-line interface for `apicentric`, replacing
//! the previous `clap`-based implementation with a custom parser for reduced dependencies.

pub mod args;
pub mod parser;

pub use args::{Cli, Commands, SimulatorAction, AiAction};
#[cfg(feature = "iot")]
pub use args::TwinCommands;

/// Parses the command-line arguments into a `Cli` instance.
pub fn parse() -> Cli {
    parser::parse()
}
