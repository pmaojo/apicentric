use async_trait::async_trait;
use std::path::PathBuf;
use tokio::fs;

use crate::domain::contract_testing::ContractValidationResult;
use crate::domain::ports::contract::{ContractReportSink, ReportError, ReportFormat};

/// Simple file-based implementation of `ContractReportSink`.
///
/// The adapter writes reports under the provided output directory. For
/// demonstration purposes only JSON output is fully implemented; other
/// formats are treated as no-ops. When a badge is requested a placeholder
/// file is written and its path returned.
pub struct FileReportSink {
    output_dir: PathBuf,
}

impl FileReportSink {
    pub fn new(output_dir: PathBuf) -> Self {
        Self { output_dir }
    }
}

#[async_trait]
impl ContractReportSink for FileReportSink {
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
