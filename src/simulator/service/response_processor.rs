use crate::errors::{ApicentricError, ApicentricResult};
use crate::simulator::service::state::ServiceState;
use crate::simulator::template::{TemplateContext, TemplateEngine};
use serde_json::Value;

/// Process response body template with robust error handling and validation
pub fn process_response_body_template(
    response_body: &str,
    template_context: &TemplateContext,
    template_engine: &TemplateEngine,
    service_name: &str,
    method: &str,
    path: &str,
) -> ApicentricResult<String> {
    let processed_body = if response_body.contains("{{") {
        // Template contains Handlebars placeholders, attempt to render
        match template_engine.render(response_body, template_context) {
            Ok(rendered) => {
                // Validate that rendered body is not empty when template was expected to produce content
                let trimmed = rendered.trim();
                if trimmed.is_empty() {
                    log::error!(
                            "Template rendering produced empty body for {} {} in service '{}': Original template: '{}'",
                            method,
                            path,
                            service_name,
                            response_body
                        );
                    return Err(ApicentricError::runtime_error(
                        "Template rendering produced empty body",
                        Some("Check template logic and ensure fixtures contain required data"),
                    ));
                }

                // Log successful template rendering for debugging
                log::info!(
                    "Successfully processed template for {} {} in service '{}': '{}'",
                    method,
                    path,
                    service_name,
                    trimmed
                );

                rendered
            }
            Err(e) => {
                // Handle template rendering errors explicitly
                log::error!(
                    "Template rendering failed for {} {} in service '{}': {}",
                    method,
                    path,
                    service_name,
                    e
                );

                // Try to provide more specific error information
                let (error_type, suggestion) = if response_body.contains("{{ fixtures") {
                    (
                        "Fixture reference error",
                        "Ensure fixtures contain the referenced data",
                    )
                } else if response_body.contains("{{ params") {
                    (
                        "Parameter reference error",
                        "Ensure URL path parameters are properly defined",
                    )
                } else if response_body.contains("{{ request") {
                    (
                        "Request context error",
                        "Check request context availability",
                    )
                } else {
                    ("Template syntax error", "Check Handlebars template syntax")
                };

                return Err(ApicentricError::runtime_error(
                    format!("Template rendering failed: {}", e),
                    Some(format!("{}: {}", error_type, suggestion)),
                ));
            }
        }
    } else {
        // No Handlebars placeholders, return as-is
        response_body.to_string()
    };

    Ok(processed_body)
}

/// Process a side effect from a response
pub fn process_side_effect(
    side_effect: &crate::simulator::config::SideEffect,
    state: &mut ServiceState,
    template_context: &TemplateContext,
    template_engine: &TemplateEngine,
) -> ApicentricResult<()> {
    // Render the side effect value template
    let rendered_value = template_engine.render(&side_effect.value, template_context)?;

    // Parse the rendered value as JSON
    let value: Value = serde_json::from_str(&rendered_value).map_err(|e| {
        ApicentricError::runtime_error(
            format!("Failed to parse side effect value as JSON: {}", e),
            Some("Ensure side effect value templates produce valid JSON"),
        )
    })?;

    match side_effect.action.as_str() {
        "add_to_fixture" => {
            state.add_to_fixture_array(&side_effect.target, value)?;
        }
        "update_fixture" => {
            state.set_fixture(side_effect.target.clone(), value);
        }
        "remove_from_fixture" => {
            state.remove_fixture(&side_effect.target);
        }
        "set_runtime_data" => {
            state.set_runtime_data(side_effect.target.clone(), value);
        }
        "remove_runtime_data" => {
            state.remove_runtime_data(&side_effect.target);
        }
        _ => {
            return Err(ApicentricError::runtime_error(
                    format!("Unknown side effect action: {}", side_effect.action),
                    Some("Use supported actions: add_to_fixture, update_fixture, remove_from_fixture, set_runtime_data, remove_runtime_data")
                ));
        }
    }

    Ok(())
}
