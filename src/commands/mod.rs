pub mod contract;
pub mod simulator;
pub mod shared;
pub mod contract_demo;
pub mod gui;
pub mod ai;

#[cfg(feature = "tui")]
pub mod tui;
#[cfg(feature = "tui")]
pub mod tui_state;
#[cfg(feature = "tui")]
pub mod tui_render;
#[cfg(feature = "tui")]
pub mod tui_events;

pub use contract::{contract_command, ContractAction};
pub use simulator::{simulator_command, SimulatorAction};
pub use ai::{ai_command, AiAction};
pub use shared::{find_yaml_files, validate_yaml_file};
pub use contract_demo::run_full_demo;
pub use gui::gui_command;
pub mod cloud;
pub use cloud::cloud_command;

#[cfg(feature = "tui")]
pub use tui::tui_command;
