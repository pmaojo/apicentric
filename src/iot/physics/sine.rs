use crate::iot::model::VariableValue;
use crate::iot::traits::SimulationStrategy;
use crate::errors::{ApicentricResult, ApicentricError};
use async_trait::async_trait;
use std::time::SystemTime;

/// A simulation strategy that generates values based on a sine wave.
pub struct SineWaveStrategy {
    variable_name: String,
    min: f64,
    max: f64,
    frequency: f64, // Hz
}

impl SineWaveStrategy {
    /// Create a new sine wave strategy
    pub fn new(variable_name: String, min: f64, max: f64, frequency: f64) -> Self {
        Self {
            variable_name,
            min,
            max,
            frequency,
        }
    }
}

#[async_trait]
impl SimulationStrategy for SineWaveStrategy {
    async fn tick(&self, state: &mut crate::iot::model::DigitalTwinState) -> ApicentricResult<()> {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_err(|e| ApicentricError::Runtime {
                message: format!("Time calculation error: {}", e),
                suggestion: None
            })?
            .as_secs_f64();
        // A * sin(wt) + offset
        // Amplitude A = (max - min) / 2
        // Offset = (max + min) / 2
        // w = 2 * pi * f

        let amplitude = (self.max - self.min) / 2.0;
        let offset = (self.max + self.min) / 2.0;
        let w = 2.0 * std::f64::consts::PI * self.frequency;

        let value = amplitude * (w * now).sin() + offset;

        state
            .variables
            .insert(self.variable_name.clone(), VariableValue::Float(value));
        Ok(())
    }
}
