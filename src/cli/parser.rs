//! CLI Argument Parser
//!
//! This module implements a manual command-line argument parser to replace `clap`.
//! It supports recursive descent parsing for subcommands and strongly-typed argument handling.

// We use `while let` on iterators to manually parse arguments which might consume
// extra items (e.g. flag values). Clippy prefers `for` loops but they borrow the iterator
// preventing manual `next()` calls inside the loop.
#![allow(clippy::while_let_on_iterator)]

use crate::cli::args::{
    AiAction, Cli, CliExecutionMode, Commands, ExportFormat, SimulatorAction,
};
#[cfg(feature = "iot")]
use crate::cli::args::{TwinCommands, TwinRunArgs};
#[cfg(feature = "mcp")]
use crate::cli::args::Mcp;
use std::env;

#[derive(Debug)]
pub enum ParseError {
    MissingSubcommand,
    UnknownSubcommand(String),
    MissingArgument(String),
    UnknownArgument(String),
    InvalidValue(String, String),
    HelpRequested(String),
    VersionRequested,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::MissingSubcommand => write!(f, "Missing subcommand"),
            ParseError::UnknownSubcommand(s) => write!(f, "Unknown subcommand: {}", s),
            ParseError::MissingArgument(s) => write!(f, "Missing required argument: {}", s),
            ParseError::UnknownArgument(s) => write!(f, "Unknown argument: {}", s),
            ParseError::InvalidValue(k, v) => write!(f, "Invalid value for {}: {}", k, v),
            ParseError::HelpRequested(s) => write!(f, "{}", s),
            ParseError::VersionRequested => write!(f, "apicentric {}", env!("CARGO_PKG_VERSION")),
        }
    }
}

impl std::error::Error for ParseError {}

pub fn parse() -> Cli {
    let args: Vec<String> = env::args().collect();
    // skip the first argument (program name)
    let args = if args.len() > 1 {
        args[1..].to_vec()
    } else {
        vec![]
    };

    match parse_args(&args) {
        Ok(cli) => cli,
        Err(e) => {
            match e {
                ParseError::HelpRequested(msg) => {
                    println!("{}", msg);
                    std::process::exit(0);
                }
                ParseError::VersionRequested => {
                    println!("apicentric {}", env!("CARGO_PKG_VERSION"));
                    std::process::exit(0);
                }
                _ => {
                    eprintln!("Error: {}", e);
                    print_help();
                    std::process::exit(1);
                }
            }
        }
    }
}

fn print_help() {
    println!(r#"Usage: apicentric [OPTIONS] <COMMAND>

Options:
  -c, --config <FILE>    Path to config file (default: apicentric.json)
      --mode <MODE>      Execution mode (CI, Development, Debug)
      --dry-run          Enable dry-run mode
  -v, --verbose          Enable verbose output
      --db-path <PATH>   Path to SQLite database (default: apicentric.db)
  -h, --help             Print help
  -V, --version          Print version

Commands:
  simulator (sim)        Manage API simulator
  ai                     AI-assisted generation
  tui                    Launch terminal UI
  gui                    Launch graphical UI
  cloud                  Launch cloud server
  new                    Create new service from template
  doctor                 Diagnose environment
  open                   Open WebUI
  twin                   Manage IoT Digital Twins
"#);
}

pub fn parse_args(args: &[String]) -> Result<Cli, ParseError> {
    let mut cli = Cli::default();
    let mut iter = args.iter().peekable();

    // Parse global flags
    while let Some(arg) = iter.peek() {
        if !arg.starts_with('-') {
            break;
        }
        let arg = iter.next().unwrap();
        match arg.as_str() {
            "--config" | "-c" => {
                cli.config = iter.next().ok_or(ParseError::MissingArgument("--config".into()))?.clone();
            }
            "--mode" => {
                let mode_str = iter.next().ok_or(ParseError::MissingArgument("--mode".into()))?;
                cli.mode = Some(match mode_str.to_lowercase().as_str() {
                    "ci" => CliExecutionMode::CI,
                    "development" | "dev" => CliExecutionMode::Development,
                    "debug" => CliExecutionMode::Debug,
                    _ => return Err(ParseError::InvalidValue("--mode".into(), mode_str.clone())),
                });
            }
            "--dry-run" => cli.dry_run = true,
            "--verbose" | "-v" => cli.verbose = true,
            "--db-path" => {
                cli.db_path = iter.next().ok_or(ParseError::MissingArgument("--db-path".into()))?.clone();
            }
            "--help" | "-h" => return Err(ParseError::HelpRequested("Full help text here...".into())), // Simplified
            "--version" | "-V" => return Err(ParseError::VersionRequested),
            _ => return Err(ParseError::UnknownArgument(arg.clone())),
        }
    }

    // Parse subcommand
    if let Some(cmd) = iter.next() {
        match cmd.as_str() {
            "simulator" | "sim" => {
                cli.command = Commands::Simulator {
                    action: parse_simulator_action(&mut iter)?,
                };
            }
            "ai" => {
                cli.command = Commands::Ai {
                    action: parse_ai_action(&mut iter)?,
                };
            }
            #[cfg(feature = "tui")]
            "tui" => cli.command = Commands::Tui,
            #[cfg(feature = "gui")]
            "gui" => cli.command = Commands::Gui,
            #[cfg(feature = "webui")]
            "cloud" => cli.command = Commands::Cloud,
            "new" => {
                let name = iter.next().ok_or(ParseError::MissingArgument("name".into()))?.clone();
                let mut template = None;
                while let Some(arg) = iter.next() {
                    match arg.as_str() {
                        "--template" | "-t" => {
                            template = Some(iter.next().ok_or(ParseError::MissingArgument("--template".into()))?.clone());
                        }
                        _ => return Err(ParseError::UnknownArgument(arg.clone())),
                    }
                }
                cli.command = Commands::New { name, template };
            }
            #[cfg(feature = "mcp")]
            "mcp" => {
                let mut test = false;
                while let Some(arg) = iter.next() {
                    match arg.as_str() {
                         "--test" => test = true,
                         _ => return Err(ParseError::UnknownArgument(arg.clone())),
                    }
                }
                cli.command = Commands::Mcp(Mcp { test });
            }
            "doctor" => cli.command = Commands::Doctor,
            "open" => {
                let mut port = None;
                while let Some(arg) = iter.next() {
                     match arg.as_str() {
                        "--port" | "-p" => {
                            let p = iter.next().ok_or(ParseError::MissingArgument("--port".into()))?;
                            port = Some(p.parse().map_err(|_| ParseError::InvalidValue("--port".into(), p.clone()))?);
                        }
                        _ => return Err(ParseError::UnknownArgument(arg.clone())),
                     }
                }
                cli.command = Commands::Open { port };
            }
            #[cfg(feature = "iot")]
            "twin" => {
                cli.command = Commands::Twin {
                    command: parse_twin_command(&mut iter)?,
                };
            }
            _ => return Err(ParseError::UnknownSubcommand(cmd.clone())),
        }
    } else {
        // Default behavior if no command
        return Err(ParseError::MissingSubcommand);
    }

    // Ensure all arguments were consumed
    if let Some(arg) = iter.next() {
        return Err(ParseError::UnknownArgument(arg.clone()));
    }

    Ok(cli)
}

fn parse_simulator_action<'a, I>(iter: &mut I) -> Result<Option<SimulatorAction>, ParseError>
where
    I: Iterator<Item = &'a String>,
{
    if let Some(action) = iter.next() {
        match action.as_str() {
            "start" | "s" => {
                let mut services_dir = "services".to_string();
                let mut force = false;
                let mut p2p = false;
                let mut template = None;
                while let Some(arg) = iter.next() {
                    match arg.as_str() {
                        "--services-dir" | "-s" => services_dir = iter.next().ok_or(ParseError::MissingArgument("--services-dir".into()))?.clone(),
                        "--force" => force = true,
                        "--p2p" => p2p = true,
                        "--template" => template = Some(iter.next().ok_or(ParseError::MissingArgument("--template".into()))?.clone()),
                        _ => return Err(ParseError::UnknownArgument(arg.clone())),
                    }
                }
                Ok(Some(SimulatorAction::Start { services_dir, force, p2p, template }))
            }
            "stop" | "x" => {
                let mut force = false;
                while let Some(arg) = iter.next() {
                    match arg.as_str() {
                        "--force" => force = true,
                         _ => return Err(ParseError::UnknownArgument(arg.clone())),
                    }
                }
                Ok(Some(SimulatorAction::Stop { force }))
            }
            "status" | "st" => {
                 let mut detailed = false;
                 while let Some(arg) = iter.next() {
                    match arg.as_str() {
                        "--detailed" | "-d" => detailed = true,
                         _ => return Err(ParseError::UnknownArgument(arg.clone())),
                    }
                 }
                 Ok(Some(SimulatorAction::Status { detailed }))
            }
            "validate" | "v" => {
                let mut file = "services".to_string();
                let mut recursive = false;
                let mut verbose = false;
                 while let Some(arg) = iter.next() {
                    match arg.as_str() {
                        "--file" | "--path" => file = iter.next().ok_or(ParseError::MissingArgument("--file".into()))?.clone(),
                        "--recursive" | "-r" => recursive = true,
                        "--verbose" => verbose = true,
                         _ => {
                             // Handle positional arg as file if not already set or starts with -
                             if !arg.starts_with('-') {
                                 file = arg.clone();
                             } else {
                                 return Err(ParseError::UnknownArgument(arg.clone()));
                             }
                         }
                    }
                 }
                 Ok(Some(SimulatorAction::Validate { file, recursive, verbose }))
            }
            "logs" | "l" => {
                let service = iter.next().ok_or(ParseError::MissingArgument("service".into()))?.clone();
                let mut limit = 20;
                let mut method = None;
                let mut route = None;
                let mut status = None;
                let mut output = None;

                while let Some(arg) = iter.next() {
                     match arg.as_str() {
                        "--limit" | "-l" => {
                             let l = iter.next().ok_or(ParseError::MissingArgument("--limit".into()))?;
                             limit = l.parse().map_err(|_| ParseError::InvalidValue("--limit".into(), l.clone()))?;
                        }
                        "--method" => method = Some(iter.next().ok_or(ParseError::MissingArgument("--method".into()))?.clone()),
                        "--route" => route = Some(iter.next().ok_or(ParseError::MissingArgument("--route".into()))?.clone()),
                        "--status" => {
                             let s = iter.next().ok_or(ParseError::MissingArgument("--status".into()))?;
                             status = Some(s.parse().map_err(|_| ParseError::InvalidValue("--status".into(), s.clone()))?);
                        }
                        "--output" => output = Some(iter.next().ok_or(ParseError::MissingArgument("--output".into()))?.clone()),
                        _ => return Err(ParseError::UnknownArgument(arg.clone())),
                     }
                }
                Ok(Some(SimulatorAction::Logs { service, limit, method, route, status, output }))
            }
            "monitor" | "m" => {
                let mut service = None;
                let mut json = false;
                let mut interval = None;
                while let Some(arg) = iter.next() {
                    match arg.as_str() {
                        "--service" => service = Some(iter.next().ok_or(ParseError::MissingArgument("--service".into()))?.clone()),
                        "--json" => json = true,
                        "--interval" => {
                             let i = iter.next().ok_or(ParseError::MissingArgument("--interval".into()))?;
                             interval = Some(i.parse().map_err(|_| ParseError::InvalidValue("--interval".into(), i.clone()))?);
                        }
                         _ => return Err(ParseError::UnknownArgument(arg.clone())),
                    }
                }
                Ok(Some(SimulatorAction::Monitor { service, json, interval }))
            }
            "set-scenario" => {
                let scenario = iter.next().ok_or(ParseError::MissingArgument("scenario".into()))?.clone();
                Ok(Some(SimulatorAction::SetScenario { scenario }))
            }
            "import" => {
                let mut file = String::new();
                let mut output = String::new();
                while let Some(arg) = iter.next() {
                    match arg.as_str() {
                        "--file" | "--input" | "-i" => file = iter.next().ok_or(ParseError::MissingArgument("--file".into()))?.clone(),
                        "--output" | "-o" => output = iter.next().ok_or(ParseError::MissingArgument("--output".into()))?.clone(),
                        _ => return Err(ParseError::UnknownArgument(arg.clone())),
                    }
                }
                if file.is_empty() { return Err(ParseError::MissingArgument("--file".into())); }
                if output.is_empty() { return Err(ParseError::MissingArgument("--output".into())); }
                Ok(Some(SimulatorAction::Import { file, output }))
            }
            "export" => {
                let mut file = String::new();
                let mut output = String::new();
                let mut format = ExportFormat::Openapi;
                while let Some(arg) = iter.next() {
                    match arg.as_str() {
                        "--file" | "--input" | "-i" => file = iter.next().ok_or(ParseError::MissingArgument("--file".into()))?.clone(),
                        "--output" | "-o" => output = iter.next().ok_or(ParseError::MissingArgument("--output".into()))?.clone(),
                        "--format" | "-f" => {
                             let f = iter.next().ok_or(ParseError::MissingArgument("--format".into()))?;
                             format = match f.to_lowercase().as_str() {
                                 "openapi" => ExportFormat::Openapi,
                                 "postman" => ExportFormat::Postman,
                                 _ => return Err(ParseError::InvalidValue("--format".into(), f.clone())),
                             };
                        }
                        _ => return Err(ParseError::UnknownArgument(arg.clone())),
                    }
                }
                if file.is_empty() { return Err(ParseError::MissingArgument("--file".into())); }
                if output.is_empty() { return Err(ParseError::MissingArgument("--output".into())); }
                Ok(Some(SimulatorAction::Export { file, output, format }))
            }
             "generate-types" => {
                let mut file = String::new();
                let mut output = String::new();
                while let Some(arg) = iter.next() {
                    match arg.as_str() {
                        "--file" | "--input" | "-i" => file = iter.next().ok_or(ParseError::MissingArgument("--file".into()))?.clone(),
                        "--output" | "-o" => output = iter.next().ok_or(ParseError::MissingArgument("--output".into()))?.clone(),
                        _ => return Err(ParseError::UnknownArgument(arg.clone())),
                    }
                }
                if file.is_empty() { return Err(ParseError::MissingArgument("--file".into())); }
                if output.is_empty() { return Err(ParseError::MissingArgument("--output".into())); }
                Ok(Some(SimulatorAction::GenerateTypes { file, output }))
            }
            "generate-query" => {
                let mut file = String::new();
                let mut output = String::new();
                while let Some(arg) = iter.next() {
                    match arg.as_str() {
                        "--file" | "--input" | "-i" => file = iter.next().ok_or(ParseError::MissingArgument("--file".into()))?.clone(),
                        "--output" | "-o" => output = iter.next().ok_or(ParseError::MissingArgument("--output".into()))?.clone(),
                        _ => return Err(ParseError::UnknownArgument(arg.clone())),
                    }
                }
                if file.is_empty() { return Err(ParseError::MissingArgument("--file".into())); }
                if output.is_empty() { return Err(ParseError::MissingArgument("--output".into())); }
                Ok(Some(SimulatorAction::GenerateQuery { file, output }))
            }
            "generate-view" => {
                let mut file = String::new();
                let mut output = String::new();
                while let Some(arg) = iter.next() {
                    match arg.as_str() {
                        "--file" | "--input" | "-i" => file = iter.next().ok_or(ParseError::MissingArgument("--file".into()))?.clone(),
                        "--output" | "-o" => output = iter.next().ok_or(ParseError::MissingArgument("--output".into()))?.clone(),
                        _ => return Err(ParseError::UnknownArgument(arg.clone())),
                    }
                }
                 if file.is_empty() { return Err(ParseError::MissingArgument("--file".into())); }
                if output.is_empty() { return Err(ParseError::MissingArgument("--output".into())); }
                Ok(Some(SimulatorAction::GenerateView { file, output }))
            }
            "new" => {
                let mut output = "services".to_string();
                while let Some(arg) = iter.next() {
                    match arg.as_str() {
                        "--output" | "-o" => output = iter.next().ok_or(ParseError::MissingArgument("--output".into()))?.clone(),
                        _ => return Err(ParseError::UnknownArgument(arg.clone())),
                    }
                }
                Ok(Some(SimulatorAction::New { output }))
            }
            "new-graphql" => {
                 let name = iter.next().ok_or(ParseError::MissingArgument("name".into()))?.clone();
                 let mut output = "services".to_string();
                 while let Some(arg) = iter.next() {
                    match arg.as_str() {
                        "--output" | "-o" => output = iter.next().ok_or(ParseError::MissingArgument("--output".into()))?.clone(),
                        _ => return Err(ParseError::UnknownArgument(arg.clone())),
                    }
                }
                 Ok(Some(SimulatorAction::NewGraphql { name, output }))
            }
            "edit" => {
                 let mut file = String::new();
                 while let Some(arg) = iter.next() {
                    match arg.as_str() {
                        "--file" | "--input" | "-i" => file = iter.next().ok_or(ParseError::MissingArgument("--file".into()))?.clone(),
                        _ => {
                             if !arg.starts_with('-') && file.is_empty() {
                                 file = arg.clone();
                             } else {
                                return Err(ParseError::UnknownArgument(arg.clone()));
                             }
                        }
                    }
                }
                if file.is_empty() { return Err(ParseError::MissingArgument("--file".into())); }
                Ok(Some(SimulatorAction::Edit { file }))
            }
            "record" => {
                let mut output = "services".to_string();
                let mut url = None;
                 while let Some(arg) = iter.next() {
                    match arg.as_str() {
                        "--output" | "-o" => output = iter.next().ok_or(ParseError::MissingArgument("--output".into()))?.clone(),
                        "--url" => url = Some(iter.next().ok_or(ParseError::MissingArgument("--url".into()))?.clone()),
                         _ => return Err(ParseError::UnknownArgument(arg.clone())),
                    }
                 }
                 Ok(Some(SimulatorAction::Record { output, url }))
            }
            "share" => {
                 let service = iter.next().ok_or(ParseError::MissingArgument("service".into()))?.clone();
                 Ok(Some(SimulatorAction::Share { service }))
            }
            "connect" => {
                 let peer = iter.next().ok_or(ParseError::MissingArgument("peer".into()))?.clone();
                 let mut service = String::new();
                 let mut port = 0;
                 let mut token = None;

                 while let Some(arg) = iter.next() {
                    match arg.as_str() {
                        "--service" => service = iter.next().ok_or(ParseError::MissingArgument("--service".into()))?.clone(),
                        "--port" => {
                             let p = iter.next().ok_or(ParseError::MissingArgument("--port".into()))?;
                             port = p.parse().map_err(|_| ParseError::InvalidValue("--port".into(), p.clone()))?;
                        }
                        "--token" => token = Some(iter.next().ok_or(ParseError::MissingArgument("--token".into()))?.clone()),
                         _ => return Err(ParseError::UnknownArgument(arg.clone())),
                    }
                 }
                 if service.is_empty() { return Err(ParseError::MissingArgument("--service".into())); }
                 if port == 0 { return Err(ParseError::MissingArgument("--port".into())); }
                 Ok(Some(SimulatorAction::Connect { peer, service, port, token }))
            }
            "dockerize" => {
                 let mut files = Vec::new();
                 let mut output = ".".to_string();
                 while let Some(arg) = iter.next() {
                    match arg.as_str() {
                        "--services" | "--file" | "-f" | "-s" => {
                            // Can be multiple? clap says Vec<String>.
                            // Simple version: assume one per flag or consecutive
                            files.push(iter.next().ok_or(ParseError::MissingArgument("--services".into()))?.clone());
                        },
                        "--output" | "-o" => output = iter.next().ok_or(ParseError::MissingArgument("--output".into()))?.clone(),
                        _ => return Err(ParseError::UnknownArgument(arg.clone())),
                    }
                 }
                 if files.is_empty() { return Err(ParseError::MissingArgument("--services".into())); }
                 Ok(Some(SimulatorAction::Dockerize { file: files, output }))
            }
            "test" => {
                 let mut path = String::new();
                 let mut url = String::new();
                 let mut env = "default".to_string();
                 while let Some(arg) = iter.next() {
                     match arg.as_str() {
                        "--path" | "-p" => path = iter.next().ok_or(ParseError::MissingArgument("--path".into()))?.clone(),
                        "--url" | "-u" => url = iter.next().ok_or(ParseError::MissingArgument("--url".into()))?.clone(),
                        "--env" => env = iter.next().ok_or(ParseError::MissingArgument("--env".into()))?.clone(),
                         _ => return Err(ParseError::UnknownArgument(arg.clone())),
                     }
                 }
                 if path.is_empty() { return Err(ParseError::MissingArgument("--path".into())); }
                 if url.is_empty() { return Err(ParseError::MissingArgument("--url".into())); }
                 Ok(Some(SimulatorAction::Test { path, url, env }))
            }
            _ => Err(ParseError::UnknownSubcommand(action.clone())),
        }
    } else {
        Ok(None)
    }
}

fn parse_ai_action<'a, I>(iter: &mut I) -> Result<AiAction, ParseError>
where
    I: Iterator<Item = &'a String>,
{
    if let Some(action) = iter.next() {
        match action.as_str() {
            "generate" => {
                 let prompt = iter.next().ok_or(ParseError::MissingArgument("prompt".into()))?.clone();
                 Ok(AiAction::Generate { prompt })
            }
             _ => Err(ParseError::UnknownSubcommand(action.clone())),
        }
    } else {
         Err(ParseError::MissingSubcommand)
    }
}

#[cfg(feature = "iot")]
fn parse_twin_command<'a, I>(iter: &mut I) -> Result<TwinCommands, ParseError>
where
    I: Iterator<Item = &'a String>,
{
    if let Some(cmd) = iter.next() {
        match cmd.as_str() {
            "run" => {
                 let mut device = String::new();
                 let mut override_config = None;
                 let mut library = "./assets/library".to_string();

                 while let Some(arg) = iter.next() {
                     match arg.as_str() {
                         "--device" | "-d" => device = iter.next().ok_or(ParseError::MissingArgument("--device".into()))?.clone(),
                         "--override-config" => override_config = Some(iter.next().ok_or(ParseError::MissingArgument("--override-config".into()))?.clone()),
                         "--library" => library = iter.next().ok_or(ParseError::MissingArgument("--library".into()))?.clone(),
                         _ => {
                             // Allow positional device if not set?
                             if !arg.starts_with('-') && device.is_empty() {
                                 device = arg.clone();
                             } else {
                                return Err(ParseError::UnknownArgument(arg.clone()));
                             }
                         }
                     }
                 }
                 if device.is_empty() { return Err(ParseError::MissingArgument("--device".into())); }
                 Ok(TwinCommands::Run(TwinRunArgs { device, override_config, library }))
            }
             _ => Err(ParseError::UnknownSubcommand(cmd.clone())),
        }
    } else {
        Err(ParseError::MissingSubcommand)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn to_args(args: &str) -> Vec<String> {
        args.split_whitespace().map(|s| s.to_string()).collect()
    }

    #[test]
    fn test_global_flags() {
        let args = to_args("--config my_config.json --verbose --dry-run doctor");
        let cli = parse_args(&args).unwrap();
        assert_eq!(cli.config, "my_config.json");
        assert!(cli.verbose);
        assert!(cli.dry_run);
        assert!(matches!(cli.command, Commands::Doctor));
    }

    #[test]
    fn test_simulator_start() {
        let args = to_args("simulator start --services-dir ./myservices --force --p2p");
        let cli = parse_args(&args).unwrap();
        match cli.command {
            Commands::Simulator { action } => {
                match action.unwrap() {
                    SimulatorAction::Start { services_dir, force, p2p, template } => {
                        assert_eq!(services_dir, "./myservices");
                        assert!(force);
                        assert!(p2p);
                        assert!(template.is_none());
                    }
                    _ => panic!("Wrong action"),
                }
            }
            _ => panic!("Wrong command"),
        }
    }

    #[test]
    fn test_simulator_start_short_alias() {
        let args = to_args("sim s -s ./myservices");
        let cli = parse_args(&args).unwrap();
        match cli.command {
            Commands::Simulator { action } => {
                match action.unwrap() {
                    SimulatorAction::Start { services_dir, .. } => {
                        assert_eq!(services_dir, "./myservices");
                    }
                    _ => panic!("Wrong action"),
                }
            }
            _ => panic!("Wrong command"),
        }
    }

    #[test]
    fn test_new_command() {
        let args = to_args("new my-service --template stripe");
        let cli = parse_args(&args).unwrap();
        match cli.command {
            Commands::New { name, template } => {
                assert_eq!(name, "my-service");
                assert_eq!(template, Some("stripe".to_string()));
            }
            _ => panic!("Wrong command"),
        }
    }

    #[test]
    fn test_missing_argument() {
        let args = to_args("new");
        let err = parse_args(&args).unwrap_err();
        assert!(matches!(err, ParseError::MissingArgument(_)));
    }

    #[test]
    fn test_unknown_argument() {
        let args = to_args("doctor --what");
        let err = parse_args(&args).unwrap_err();
        assert!(matches!(err, ParseError::UnknownArgument(_)));
    }

    #[test]
    fn test_open_port() {
        let args = to_args("open --port 3000");
        let cli = parse_args(&args).unwrap();
        match cli.command {
            Commands::Open { port } => {
                assert_eq!(port, Some(3000));
            }
            _ => panic!("Wrong command"),
        }
    }

    #[test]
    #[cfg(feature = "iot")]
    fn test_twin_run() {
         let args = to_args("twin run my_device --library ./lib");
         let cli = parse_args(&args).unwrap();
         match cli.command {
             Commands::Twin { command } => {
                 match command {
                     TwinCommands::Run(TwinRunArgs { device, library, .. }) => {
                         assert_eq!(device, "my_device");
                         assert_eq!(library, "./lib");
                     }
                 }
             }
             _ => panic!("Wrong command"),
         }
    }

    #[test]
    fn test_validate_positional() {
        let args = to_args("simulator validate services/api.yaml -r");
        let cli = parse_args(&args).unwrap();
         match cli.command {
            Commands::Simulator { action } => {
                match action.unwrap() {
                    SimulatorAction::Validate { file, recursive, .. } => {
                        assert_eq!(file, "services/api.yaml");
                        assert!(recursive);
                    }
                    _ => panic!("Wrong action"),
                }
            }
            _ => panic!("Wrong command"),
        }
    }
}
