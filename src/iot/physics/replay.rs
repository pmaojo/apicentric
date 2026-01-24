use crate::iot::model::{DigitalTwinState, VariableValue};
use crate::iot::traits::SimulationStrategy;
use crate::errors::{ApicentricResult, ApicentricError};
use async_trait::async_trait;
use log::warn;
use std::path::Path;
use std::sync::Mutex;

/// A simulation strategy that replays data from a CSV file.
pub struct ReplayStrategy {
    variable_name: String,
    data: Vec<f64>,
    index: Mutex<usize>,
    loop_data: bool,
}

impl ReplayStrategy {
    /// Create a new replay strategy from a CSV file
    pub fn new(
        file_path: &Path,
        variable_name: String,
        column_name: Option<String>,
        loop_data: bool,
    ) -> ApicentricResult<Self> {
        let mut data = Vec::new();
        let mut rdr = csv::Reader::from_path(file_path)?;

        // Check headers if column_name is provided
        let headers = rdr.headers()?.clone();
        let col_idx = if let Some(col) = column_name {
            headers
                .iter()
                .position(|h| h == col)
                .ok_or_else(|| ApicentricError::Data {
                    message: format!("Column '{}' not found in CSV file", col),
                    suggestion: Some("Check CSV headers".to_string())
                })?
        } else {
            0 // Default to first column
        };

        for result in rdr.records() {
            let record = result?;
            if let Some(field) = record.get(col_idx) {
                // Parse float, default to 0.0 if parsing fails
                // We might want to be stricter or log errors, but for now be robust
                let val: f64 = match field.parse() {
                    Ok(v) => v,
                    Err(_) => {
                        warn!(
                            "Failed to parse value '{}' in CSV. Defaulting to 0.0",
                            field
                        );
                        0.0
                    }
                };
                data.push(val);
            }
        }

        if data.is_empty() {
            return Err(ApicentricError::Data {
                message: "CSV file contains no data".to_string(),
                suggestion: Some("Ensure the CSV file is not empty".to_string())
            });
        }

        Ok(Self {
            variable_name,
            data,
            index: Mutex::new(0),
            loop_data,
        })
    }
}

#[async_trait]
impl SimulationStrategy for ReplayStrategy {
    async fn tick(&self, state: &mut DigitalTwinState) -> ApicentricResult<()> {
        let mut idx = self.index.lock().map_err(|_| ApicentricError::Runtime {
            message: "Failed to lock replay index".to_string(),
            suggestion: None
        })?;

        if *idx >= self.data.len() {
            if self.loop_data {
                *idx = 0;
            } else {
                // Hold the last value
                *idx = self.data.len() - 1;
            }
        }

        let value = self.data[*idx];
        state
            .variables
            .insert(self.variable_name.clone(), VariableValue::Float(value));

        *idx += 1;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_replay_strategy() {
        // Create temp CSV
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "val,other").unwrap();
        writeln!(file, "10.0,1").unwrap();
        writeln!(file, "20.0,2").unwrap();
        writeln!(file, "30.0,3").unwrap();

        let path = file.path();

        let strategy =
            ReplayStrategy::new(path, "temp".to_string(), Some("val".to_string()), true).unwrap();

        let mut state = DigitalTwinState::default();

        // Tick 1
        strategy.tick(&mut state).await.unwrap();
        assert_eq!(state.variables.get("temp").unwrap().as_f64().unwrap(), 10.0);

        // Tick 2
        strategy.tick(&mut state).await.unwrap();
        assert_eq!(state.variables.get("temp").unwrap().as_f64().unwrap(), 20.0);

        // Tick 3
        strategy.tick(&mut state).await.unwrap();
        assert_eq!(state.variables.get("temp").unwrap().as_f64().unwrap(), 30.0);

        // Tick 4 (Loop)
        strategy.tick(&mut state).await.unwrap();
        assert_eq!(state.variables.get("temp").unwrap().as_f64().unwrap(), 10.0);
    }

    #[tokio::test]
    async fn test_replay_strategy_no_loop() {
        // Create temp CSV
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "val").unwrap();
        writeln!(file, "10.0").unwrap();
        writeln!(file, "20.0").unwrap();

        let path = file.path();

        let strategy = ReplayStrategy::new(
            path,
            "temp".to_string(),
            None, // Default to first col
            false,
        )
        .unwrap();

        let mut state = DigitalTwinState::default();

        // Tick 1
        strategy.tick(&mut state).await.unwrap(); // 10.0
                                                  // Tick 2
        strategy.tick(&mut state).await.unwrap(); // 20.0

        // Tick 3 (End reached, should hold 20.0)
        strategy.tick(&mut state).await.unwrap();
        assert_eq!(state.variables.get("temp").unwrap().as_f64().unwrap(), 20.0);
    }
}
