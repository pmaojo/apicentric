//! A simple file-based implementation of the `ContractReportSink` port.
//!
//! This module provides a `FileReportSink` that writes contract validation
//! reports to the file system.

use async_trait::async_trait;
use std::path::PathBuf;
use tokio::fs;

use crate::domain::contract_testing::ContractValidationResult;
use crate::domain::ports::contract::{ContractReportSink, ReportError, ReportFormat};

/// A simple file-based implementation of `ContractReportSink`.
///
/// The adapter writes reports under the provided output directory. For
/// demonstration purposes only JSON output is fully implemented; other
/// formats are treated as no-ops. When a badge is requested a placeholder
/// file is written and its path returned.
pub struct FileReportSink {
    output_dir: PathBuf,
}

impl FileReportSink {
    /// Creates a new `FileReportSink`.
    ///
    /// # Arguments
    ///
    /// * `output_dir` - The directory where reports will be written.
    pub fn new(output_dir: PathBuf) -> Self {
        Self { output_dir }
    }
}

#[async_trait]
impl ContractReportSink for FileReportSink {
    /// Writes a contract validation report to the file system.
    ///
    /// # Arguments
    ///
    /// * `format` - The format of the report to write.
    /// * `result` - The contract validation result to write.
    async fn write_report(
        &self,
        format: ReportFormat,
        result: &ContractValidationResult,
    ) -> Result<Option<String>, ReportError> {
        fs::create_dir_all(&self.output_dir)
            .await
            .map_err(|e| ReportError::FileSystemError(e.to_string()))?;

        match format {
            ReportFormat::Json => {
                let path = self.output_dir.join("report.json");
                let data = serde_json::to_string_pretty(result)
                    .map_err(|e| ReportError::FormatError(e.to_string()))?;
                fs::write(&path, data)
                    .await
                    .map_err(|e| ReportError::WriteError(e.to_string()))?;
                Ok(None)
            }
            ReportFormat::JUnit => Ok(None),
            ReportFormat::Html => Ok(None),
            ReportFormat::Badge => {
                let path = self.output_dir.join("badge.svg");
                fs::write(&path, "<svg></svg>")
                    .await
                    .map_err(|e| ReportError::WriteError(e.to_string()))?;
                Ok(Some(path.to_string_lossy().to_string()))
            }
        }
    }
}
