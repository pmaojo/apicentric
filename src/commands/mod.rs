pub mod setup_npm;
pub mod contract;
// pub mod docs; // (pendiente de implementación real)
pub mod simulator;
pub mod shared;
pub mod contract_demo;
pub mod gui;
pub mod ai;

pub use setup_npm::SetupNpmHandler;
pub use contract::{contract_command, ContractAction};
pub use simulator::{simulator_command, SimulatorAction};
pub use ai::{ai_command, AiAction};
pub use shared::{find_yaml_files, validate_yaml_file};
pub use contract_demo::run_full_demo;
pub use gui::gui_command;
