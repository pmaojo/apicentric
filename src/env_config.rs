use std::env;
use std::path::PathBuf;

use crate::config::ApicentricConfig;
use crate::errors::{ApicentricError, ApicentricResult};
#[cfg(feature = "simulator")]
use crate::simulator::config::{PortRange, SimulatorConfig};

/// Environment-driven configuration overrides for Apicentric.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvConfig {
    pub config_path: PathBuf,
    pub services_dir: Option<PathBuf>,
    pub port_start: Option<u16>,
    pub port_end: Option<u16>,
    pub db_path: Option<PathBuf>,
    pub admin_port: Option<u16>,
    pub enable_simulator: Option<bool>,
}

impl EnvConfig {
    /// Load environment variables (respecting .env files) into a typed structure.
    pub fn load(ignore_env_file: bool) -> ApicentricResult<Self> {
        if !ignore_env_file {
            let _ = dotenvy::dotenv();
        }
        let config_path = env::var("APICENTRIC_CONFIG_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("apicentric.json"));

        let services_dir = env::var("APICENTRIC_SERVICES_DIR").ok().map(PathBuf::from);
        let port_start = env::var("APICENTRIC_PORT_START")
            .ok()
            .and_then(|v| v.parse().ok());
        let port_end = env::var("APICENTRIC_PORT_END")
            .ok()
            .and_then(|v| v.parse().ok());
        let db_path = env::var("APICENTRIC_DB_PATH").ok().map(PathBuf::from);
        let admin_port = env::var("APICENTRIC_ADMIN_PORT")
            .ok()
            .and_then(|v| v.parse().ok());
        let enable_simulator = env::var("APICENTRIC_SIMULATOR_ENABLED")
            .ok()
            .and_then(|v| v.parse().ok());

        if let (Some(start), Some(end)) = (port_start, port_end) {
            if start >= end {
                return Err(ApicentricError::validation_error(
                    "APICENTRIC_PORT_START must be less than APICENTRIC_PORT_END",
                    Some("port_range"),
                    Some("Adjust the environment values to represent an increasing range"),
                ));
            }
        }

        Ok(Self {
            config_path,
            services_dir,
            port_start,
            port_end,
            db_path,
            admin_port,
            enable_simulator,
        })
    }

    /// Apply the environment overrides to an in-memory configuration.
    pub fn apply(&self, config: &mut ApicentricConfig) {
        #[cfg(feature = "simulator")]
        {
            let simulator = config
                .simulator
                .get_or_insert_with(SimulatorConfig::default_config);

            if let Some(dir) = &self.services_dir {
                simulator.services_dir = dir.clone();
            }
            if let (Some(start), Some(end)) = (self.port_start, self.port_end) {
                simulator.port_range = PortRange { start, end };
            }
            if let Some(path) = &self.db_path {
                simulator.db_path = path.clone();
            }
            if let Some(port) = self.admin_port {
                simulator.admin_port = Some(port);
            }
            if let Some(enabled) = self.enable_simulator {
                simulator.enabled = enabled;
            }
        }
        #[cfg(not(feature = "simulator"))]
        {
            // Suppress unused variable warnings when simulator is disabled
            let _ = config;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clear_env() {
        for key in [
            "APICENTRIC_CONFIG_PATH",
            "APICENTRIC_SERVICES_DIR",
            "APICENTRIC_PORT_START",
            "APICENTRIC_PORT_END",
            "APICENTRIC_DB_PATH",
            "APICENTRIC_ADMIN_PORT",
            "APICENTRIC_SIMULATOR_ENABLED",
        ] {
            env::remove_var(key);
        }
    }

    #[test]
    fn test_env_loading_and_logic() {
        // Since environment variables are global to the process, we run these sequentially
        // in a single test to avoid race conditions with parallel execution.

        // 1. Test defaults
        clear_env();
        let env_cfg = EnvConfig::load(true).unwrap();
        assert_eq!(env_cfg.config_path, PathBuf::from("apicentric.json"));
        assert!(env_cfg.services_dir.is_none());

        // 2. Test overrides and apply
        clear_env();
        env::set_var("APICENTRIC_SERVICES_DIR", "/tmp/services");
        env::set_var("APICENTRIC_PORT_START", "8100");
        env::set_var("APICENTRIC_PORT_END", "8200");
        env::set_var("APICENTRIC_DB_PATH", "./test.db");
        env::set_var("APICENTRIC_ADMIN_PORT", "9999");
        env::set_var("APICENTRIC_SIMULATOR_ENABLED", "true");

        let env_cfg = EnvConfig::load(true).unwrap();
        let mut cfg = ApicentricConfig::default();
        env_cfg.apply(&mut cfg);

        #[cfg(feature = "simulator")]
        {
            let simulator = cfg.simulator.unwrap();
            assert_eq!(simulator.services_dir, PathBuf::from("/tmp/services"));
            assert_eq!(simulator.port_range.start, 8100);
            assert_eq!(simulator.port_range.end, 8200);
            assert_eq!(simulator.db_path, PathBuf::from("./test.db"));
            assert_eq!(simulator.admin_port, Some(9999));
            assert!(simulator.enabled);
        }

        // 3. Test invalid range
        clear_env();
        env::set_var("APICENTRIC_PORT_START", "9000");
        env::set_var("APICENTRIC_PORT_END", "8000");
        let result = EnvConfig::load(true);
        assert!(result.is_err());

        // Final cleanup
        clear_env();
    }
}
