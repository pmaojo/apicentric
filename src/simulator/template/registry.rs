use handlebars::Handlebars;

use crate::errors::ApicentricResult;
use crate::simulator::template::helpers;

/// Registry for template helpers
pub struct HelperRegistry;

impl HelperRegistry {
    /// Register all built-in template helpers
    pub fn register_helpers(handlebars: &mut Handlebars) -> ApicentricResult<()> {
        helpers::faker::register(handlebars);
        helpers::math::register(handlebars);
        helpers::text::register(handlebars);
        helpers::core::register_core_helpers(handlebars);
        Ok(())
    }
}
