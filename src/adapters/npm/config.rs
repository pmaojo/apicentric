use crate::errors::ValidationError;
use crate::validation::{ConfigValidator, ValidationUtils};
use serde::{Deserialize, Serialize};

/// NPM integration configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NpmConfig {
    #[serde(default = "default_pulse_script")]
    pub pulse_script: String,

    #[serde(default = "default_pulse_watch_script")]
    pub pulse_watch_script: String,

    #[serde(default = "default_dev_script")]
    pub dev_script: String,
}

impl Default for NpmConfig {
    fn default() -> Self {
        Self {
            pulse_script: default_pulse_script(),
            pulse_watch_script: default_pulse_watch_script(),
            dev_script: default_dev_script(),
        }
    }
}

fn default_pulse_script() -> String {
    "cargo run --manifest-path utils/mockforge/Cargo.toml --".to_string()
}

fn default_pulse_watch_script() -> String {
    "cargo run --manifest-path utils/mockforge/Cargo.toml -- watch".to_string()
}

fn default_dev_script() -> String {
    "npm run dev".to_string()
}

impl ConfigValidator for NpmConfig {
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        if let Err(e) =
            ValidationUtils::validate_non_empty_string(&self.pulse_script, "npm.pulse_script")
        {
            errors.push(e);
        }

        if let Err(e) = ValidationUtils::validate_non_empty_string(
            &self.pulse_watch_script,
            "npm.pulse_watch_script",
        ) {
            errors.push(e);
        }

        if let Err(e) =
            ValidationUtils::validate_non_empty_string(&self.dev_script, "npm.dev_script")
        {
            errors.push(e);
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_valid() {
        let cfg = NpmConfig::default();
        assert!(cfg.validate().is_ok());
        assert_eq!(cfg.dev_script, "npm run dev");
    }

    #[test]
    fn empty_scripts_fail_validation() {
        let cfg = NpmConfig {
            pulse_script: String::new(),
            pulse_watch_script: String::new(),
            dev_script: String::new(),
        };
        assert!(cfg.validate().is_err());
    }
}
