use crate::{PulseError, PulseResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct AllureTestResult {
    pub uuid: String,
    pub name: String,
    pub full_name: String,
    pub status: AllureStatus,
    pub start: u64,
    pub stop: u64,
    pub duration: u64,
    pub labels: Vec<AllureLabel>,
    pub parameters: Vec<AllureParameter>,
    pub steps: Vec<AllureStep>,
    pub attachments: Vec<AllureAttachment>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_details: Option<AllureStatusDetails>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum AllureStatus {
    #[serde(rename = "passed")]
    Passed,
    #[serde(rename = "failed")]
    Failed,
    #[serde(rename = "broken")]
    Broken,
    #[serde(rename = "skipped")]
    Skipped,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AllureLabel {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AllureParameter {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AllureStep {
    pub name: String,
    pub status: AllureStatus,
    pub start: u64,
    pub stop: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AllureAttachment {
    pub name: String,
    pub source: String,
    #[serde(rename = "type")]
    pub attachment_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AllureStatusDetails {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace: Option<String>,
}

pub struct AllureAdapter {
    report_dir: PathBuf,
    current_test: Option<AllureTestResult>,
}

impl AllureAdapter {
    pub fn new(report_dir: PathBuf) -> PulseResult<Self> {
        // Create report directory if it doesn't exist
        fs::create_dir_all(&report_dir).map_err(|e| {
            PulseError::fs_error(
                format!("Cannot create Allure report directory: {}", e),
                Some("Check write permissions for the reports directory"),
            )
        })?;

        Ok(Self {
            report_dir,
            current_test: None,
        })
    }

    pub fn start_test(&mut self, name: &str, full_name: &str) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        self.current_test = Some(AllureTestResult {
            uuid: Uuid::new_v4().to_string(),
            name: name.to_string(),
            full_name: full_name.to_string(),
            status: AllureStatus::Passed,
            start: now,
            stop: now,
            duration: 0,
            labels: vec![
                AllureLabel {
                    name: "suite".to_string(),
                    value: full_name.to_string(),
                },
                AllureLabel {
                    name: "framework".to_string(),
                    value: "cypress".to_string(),
                },
            ],
            parameters: Vec::new(),
            steps: Vec::new(),
            attachments: Vec::new(),
            status_details: None,
        });
    }

    pub fn end_test(
        &mut self,
        status: AllureStatus,
        error: Option<String>,
        duration: Duration,
    ) -> PulseResult<()> {
        if let Some(ref mut test) = self.current_test {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64;

            test.status = status;
            test.stop = now;
            test.duration = duration.as_millis() as u64;

            if let Some(error_msg) = error {
                test.status_details = Some(AllureStatusDetails {
                    message: error_msg,
                    trace: None,
                });
            }

            // Write test result to file
            let filename = format!("{}-result.json", test.uuid);
            let filepath = self.report_dir.join(filename);

            let json = serde_json::to_string_pretty(test).map_err(|e| {
                PulseError::fs_error(
                    format!("Cannot serialize Allure test result: {}", e),
                    None::<String>,
                )
            })?;

            let mut file = File::create(&filepath).map_err(|e| {
                PulseError::fs_error(
                    format!("Cannot create Allure result file: {}", e),
                    Some("Check write permissions for the Allure report directory"),
                )
            })?;

            file.write_all(json.as_bytes()).map_err(|e| {
                PulseError::fs_error(
                    format!("Cannot write Allure result: {}", e),
                    Some("Check disk space and write permissions"),
                )
            })?;
        }

        self.current_test = None;
        Ok(())
    }

    pub fn add_step(&mut self, name: &str, status: AllureStatus) {
        if let Some(ref mut test) = self.current_test {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64;

            test.steps.push(AllureStep {
                name: name.to_string(),
                status,
                start: now,
                stop: now,
            });
        }
    }

    pub fn add_parameter(&mut self, name: &str, value: &str) {
        if let Some(ref mut test) = self.current_test {
            test.parameters.push(AllureParameter {
                name: name.to_string(),
                value: value.to_string(),
            });
        }
    }

    pub fn attach_screenshot(&mut self, name: &str, screenshot_path: &PathBuf) -> PulseResult<()> {
        if let Some(ref mut test) = self.current_test {
            if screenshot_path.exists() {
                // Copy screenshot to allure attachments directory
                let attachments_dir = self.report_dir.join("attachments");
                fs::create_dir_all(&attachments_dir).map_err(|e| {
                    PulseError::fs_error(
                        format!("Cannot create attachments directory: {}", e),
                        Some("Check write permissions"),
                    )
                })?;

                let attachment_filename = format!(
                    "{}-{}",
                    Uuid::new_v4(),
                    screenshot_path.file_name().unwrap().to_string_lossy()
                );
                let attachment_path = attachments_dir.join(&attachment_filename);

                fs::copy(screenshot_path, &attachment_path).map_err(|e| {
                    PulseError::fs_error(
                        format!("Cannot copy screenshot: {}", e),
                        Some("Check file permissions and disk space"),
                    )
                })?;

                test.attachments.push(AllureAttachment {
                    name: name.to_string(),
                    source: attachment_filename,
                    attachment_type: "image/png".to_string(),
                });
            }
        }
        Ok(())
    }

    pub fn generate_environment_info(&self) -> PulseResult<()> {
        let env_info = HashMap::from([
            ("Framework".to_string(), "Cypress".to_string()),
            ("Language".to_string(), "TypeScript".to_string()),
            ("Test Runner".to_string(), "Pulse".to_string()),
            ("OS".to_string(), std::env::consts::OS.to_string()),
            (
                "Architecture".to_string(),
                std::env::consts::ARCH.to_string(),
            ),
        ]);

        let env_path = self.report_dir.join("environment.properties");
        let mut file = File::create(&env_path).map_err(|e| {
            PulseError::fs_error(
                format!("Cannot create environment file: {}", e),
                Some("Check write permissions"),
            )
        })?;

        for (key, value) in env_info {
            writeln!(file, "{}={}", key, value).map_err(|e| {
                PulseError::fs_error(
                    format!("Cannot write environment info: {}", e),
                    None::<String>,
                )
            })?;
        }

        Ok(())
    }
}
