use super::{AiConfig, AiProviderKind, ExecutionConfig, NpmConfig, ApicentricConfig, ServerConfig};
use crate::errors::ValidationError;
use crate::validation::{ConfigValidator, ValidationUtils};

impl ConfigValidator for AiConfig {
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();
        match self.provider {
            AiProviderKind::Local => {
                if self.model_path.as_deref().unwrap_or("").is_empty() {
                    errors.push(ValidationError::new(
                        "ai.model_path",
                        "model_path is required for local provider",
                    ));
                }
            }
            AiProviderKind::Openai => {
                if self.api_key.as_deref().unwrap_or("").is_empty() {
                    errors.push(ValidationError::new(
                        "ai.api_key",
                        "api_key is required for openai provider",
                    ));
                }
            }
        }
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

impl ConfigValidator for ApicentricConfig {
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();
        if let Err(e) = ValidationUtils::validate_url(&self.base_url, "base_url") {
            errors.push(e);
        }
        if let Err(e) = ValidationUtils::validate_non_empty_string(
            &self.cypress_config_path,
            "cypress_config_path",
        ) {
            errors.push(e);
        }
        if let Err(e) = ValidationUtils::validate_non_empty_string(&self.specs_pattern, "specs_pattern") {
            errors.push(e);
        }
        if let Err(e) = ValidationUtils::validate_non_empty_string(&self.reports_dir, "reports_dir") {
            errors.push(e);
        }
        if let Err(e) = ValidationUtils::validate_glob_pattern(&self.specs_pattern, "specs_pattern") {
            errors.push(e);
        }
        if let Err(e) = ValidationUtils::validate_directory(&self.routes_dir, "routes_dir", true) {
            errors.push(e);
        }
        if let Err(e) = ValidationUtils::validate_directory(&self.specs_dir, "specs_dir", true) {
            errors.push(e);
        }
        if let Err(e) =
            ValidationUtils::validate_parent_directory(&self.index_cache_path, "index_cache_path")
        {
            errors.push(e);
        }
        if let Err(e) = ValidationUtils::validate_numeric_range(
            self.default_timeout,
            1000,
            300000,
            "default_timeout",
        ) {
            errors.push(e);
        }
        if let Err(mut server_errors) = self.server.validate() {
            errors.append(&mut server_errors);
        }
        if let Err(mut exec_errors) = self.execution.validate() {
            errors.append(&mut exec_errors);
        }
        if let Err(mut npm_errors) = self.npm.validate() {
            errors.append(&mut npm_errors);
        }
        if let Some(ref ai) = self.ai {
            if let Err(mut ai_errors) = ai.validate() {
                errors.append(&mut ai_errors);
            }
        }
        if let Some(ref simulator) = self.simulator {
            if let Err(mut simulator_errors) = simulator.validate() {
                errors.append(&mut simulator_errors);
            }
        }
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

impl ConfigValidator for ServerConfig {
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();
        if let Err(e) =
            ValidationUtils::validate_non_empty_string(&self.start_command, "server.start_command")
        {
            errors.push(e);
        }
        if let Err(e) = ValidationUtils::validate_numeric_range(
            self.startup_timeout_ms,
            5000,
            120000,
            "server.startup_timeout_ms",
        ) {
            errors.push(e);
        }
        if let Err(e) = ValidationUtils::validate_numeric_range(
            self.health_check_retries,
            1,
            20,
            "server.health_check_retries",
        ) {
            errors.push(e);
        }
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

impl ConfigValidator for ExecutionConfig {
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        Ok(())
    }
}

impl ConfigValidator for NpmConfig {
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();
        if let Err(e) =
            ValidationUtils::validate_non_empty_string(&self.apicentric_script, "npm.apicentric_script")
        {
            errors.push(e);
        }
        if let Err(e) = ValidationUtils::validate_non_empty_string(
            &self.apicentric_watch_script,
            "npm.apicentric_watch_script",
        ) {
            errors.push(e);
        }
        if let Err(e) = ValidationUtils::validate_non_empty_string(&self.dev_script, "npm.dev_script") {
            errors.push(e);
        }
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_config_validation() {
        let mut cfg = ServerConfig::default();
        assert!(cfg.validate().is_ok());
        cfg.start_command = String::new();
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn npm_config_validation() {
        let mut cfg = NpmConfig::default();
        cfg.dev_script.clear();
        assert!(cfg.validate().is_err());
    }
}

