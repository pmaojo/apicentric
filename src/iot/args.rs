use clap::{Args, Subcommand};

#[derive(Subcommand)]
pub enum TwinCommands {
    /// Run a digital twin simulation
    Run(TwinRunArgs),
}

#[derive(Args)]
pub struct TwinRunArgs {
    /// Path to the device definition file (YAML)
    #[arg(short, long)]
    pub device: String,

    /// Path to an override configuration file
    #[arg(long)]
    pub override_config: Option<String>,

    /// Path to the library directory containing device definitions
    #[arg(long, default_value = "./assets/library")]
    pub library: String,
}
