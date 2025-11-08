use super::super::{BehaviorConfig, ServiceDefinition, SimulatorConfig};
use crate::errors::{ApicentricError, ApicentricResult, ValidationError};
use crate::validation::{ConfigValidator, ValidationUtils};
use std::collections::HashSet;

/// Validate the basic structure of a service definition
pub fn validate_service_schema(service: &ServiceDefinition) -> ApicentricResult<()> {
    if let Err(validation_errors) = service.validate() {
        let error_message =
            crate::errors::ErrorFormatter::format_validation_errors(&validation_errors);
        return Err(ApicentricError::config_error(
            format!("Service validation failed for '{}':\n{}", service.name, error_message),
            Some("Fix the validation errors listed above"),
        ));
    }
    Ok(())
}

/// Ensure service names are unique
pub fn validate_unique_name(
    service: &ServiceDefinition,
    names: &mut HashSet<String>,
) -> ApicentricResult<()> {
    if !names.insert(service.name.clone()) {
        return Err(ApicentricError::config_error(
            format!("Duplicate service name '{}'", service.name),
            Some("Ensure each service has a unique name"),
        ));
    }
    Ok(())
}

impl ConfigValidator for ServiceDefinition {
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        if let Err(e) = ValidationUtils::validate_non_empty_string(&self.name, "name") {
            errors.push(e);
        }

        if let Err(mut server_errors) = self.server.validate() {
            errors.append(&mut server_errors);
        }

        if self.endpoints.is_empty() {
            errors.push(ValidationError {
                field: "endpoints".to_string(),
                message: "Service must have at least one endpoint".to_string(),
                suggestion: Some("Add at least one endpoint definition".to_string()),
            });
        }

        for (i, endpoint) in self.endpoints.iter().enumerate() {
            if let Err(mut endpoint_errors) = endpoint.validate() {
                for error in &mut endpoint_errors {
                    error.field = format!("endpoints[{}].{}", i, error.field);
                }
                errors.append(&mut endpoint_errors);
            }
        }

        if let Some(ref behavior) = self.behavior {
            if let Err(mut behavior_errors) = behavior.validate() {
                errors.append(&mut behavior_errors);
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl ConfigValidator for BehaviorConfig {
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        if let Some(ref latency) = self.latency {
            if latency.min_ms > latency.max_ms {
                errors.push(ValidationError {
                    field: "behavior.latency".to_string(),
                    message: "Minimum latency must be less than or equal to maximum latency"
                        .to_string(),
                    suggestion: Some("Ensure min_ms <= max_ms".to_string()),
                });
            }
        }

        if let Some(ref error_sim) = self.error_simulation {
            if error_sim.rate < 0.0 || error_sim.rate > 1.0 {
                errors.push(ValidationError {
                    field: "behavior.error_simulation.rate".to_string(),
                    message: "Error rate must be between 0.0 and 1.0".to_string(),
                    suggestion: Some(
                        "Use a decimal value between 0.0 and 1.0 (e.g., 0.05 for 5%)".to_string(),
                    ),
                });
            }
        }

        if let Some(ref rate_limit) = self.rate_limiting {
            if rate_limit.requests_per_minute == 0 {
                errors.push(ValidationError {
                    field: "behavior.rate_limiting.requests_per_minute".to_string(),
                    message: "Requests per minute must be greater than 0".to_string(),
                    suggestion: Some("Set a positive number of requests per minute".to_string()),
                });
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl ConfigValidator for SimulatorConfig {
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        if let Err(e) =
            ValidationUtils::validate_directory(&self.services_dir, "services_dir", false)
        {
            errors.push(e);
        }

        if self.port_range.start >= self.port_range.end {
            errors.push(ValidationError {
                field: "port_range".to_string(),
                message: "Port range start must be less than end".to_string(),
                suggestion: Some("Ensure start port is less than end port".to_string()),
            });
        }

        if self.port_range.start < 1024 {
            errors.push(ValidationError {
                field: "port_range.start".to_string(),
                message: "Port range start should be >= 1024 to avoid system ports".to_string(),
                suggestion: Some("Use ports 1024 or higher".to_string()),
            });
        }

        if let Some(ref behavior) = self.global_behavior {
            if let Err(mut behavior_errors) = behavior.validate() {
                errors.append(&mut behavior_errors);
            }
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
    use crate::simulator::config::{EndpointDefinition, ServerConfig};

    #[test]
    fn duplicate_name_validator_fails() {
        let server = ServerConfig { port: None, base_path: "/api".into(), proxy_base_url: None, cors: None };
        let endpoint = EndpointDefinition {
            kind: Default::default(),
            method: "GET".into(),
            path: "/health".into(),
            header_match: None,
            description: None,
            parameters: None,
            request_body: None,
            responses: std::collections::HashMap::new(),
            scenarios: None,
            stream: None,
        };
        let service = ServiceDefinition { name: "svc".into(), version: None, description: None, server, models: None, fixtures: None, bucket: None, endpoints: vec![endpoint], graphql: None, behavior: None };
        let mut names = HashSet::new();
        validate_unique_name(&service, &mut names).unwrap();
        let err = validate_unique_name(&service, &mut names).unwrap_err();
        assert!(format!("{}", err).contains("Duplicate"));
    }
}
