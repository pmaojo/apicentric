use crate::config;

/// Representa el contexto de ejecución derivado de la configuración y banderas CLI.
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub mode: config::ExecutionMode,
    pub dry_run: bool,
    pub verbose: bool,
    pub continue_on_failure: bool,
}

impl ExecutionContext {
    pub fn new(config: &config::PulseConfig) -> Self {
        Self {
            mode: config.execution.mode.clone(),
            dry_run: config.execution.dry_run,
            verbose: config.execution.verbose,
            continue_on_failure: config.execution.continue_on_failure,
        }
    }
    pub fn with_mode(mut self, mode: config::ExecutionMode) -> Self {
        self.mode = mode;
        self
    }
    pub fn with_dry_run(mut self, dry_run: bool) -> Self {
        self.dry_run = dry_run;
        self
    }
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }
    pub fn is_ci_mode(&self) -> bool {
        matches!(self.mode, config::ExecutionMode::CI)
    }
    pub fn is_development_mode(&self) -> bool {
        matches!(self.mode, config::ExecutionMode::Development)
    }
    pub fn is_debug_mode(&self) -> bool {
        matches!(self.mode, config::ExecutionMode::Debug)
    }
    pub fn should_skip_server_check(&self) -> bool {
        self.is_ci_mode()
    }
    pub fn should_show_progress(&self) -> bool {
        self.is_development_mode() || self.is_debug_mode()
    }
    pub fn should_log_debug(&self) -> bool {
        self.is_debug_mode() || self.verbose
    }
}
