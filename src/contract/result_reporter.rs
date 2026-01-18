use crate::domain::contract_testing::ContractValidationResult;
use crate::domain::ports::contract::{ContractReportSink, ReportError, ReportFormat};
use std::sync::Arc;

/// Reports contract validation results using a `ContractReportSink` port.
pub struct ResultReporter<S: ContractReportSink> {
    sink: Arc<S>,
}

impl<S: ContractReportSink> ResultReporter<S> {
    /// Create a new reporter with the given sink.
    pub fn new(sink: Arc<S>) -> Self {
        Self { sink }
    }

    /// Delegate the report generation to the sink.
    pub async fn report(
        &self,
        format: ReportFormat,
        result: &ContractValidationResult,
    ) -> Result<Option<String>, ReportError> {
        self.sink.write_report(format, result).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::sync::{Arc, Mutex};
    use std::time::SystemTime;

    use crate::domain::contract_testing::{ContractId, ContractValidationResult};

    struct FakeSink {
        called: Mutex<bool>,
    }

    #[async_trait]
    impl ContractReportSink for FakeSink {
        async fn write_report(
            &self,
            _format: ReportFormat,
            _result: &ContractValidationResult,
        ) -> Result<Option<String>, ReportError> {
            *self.called.lock().unwrap() = true;
            Ok(Some("ok".to_string()))
        }
    }

    #[tokio::test]
    async fn delegates_reporting_to_sink() {
<<<<<<< HEAD
        let sink = Arc::new(FakeSink {
            called: Mutex::new(false),
        });
=======
        let sink = Arc::new(FakeSink { called: Mutex::new(false) });
>>>>>>> origin/main
        let reporter = ResultReporter::new(sink.clone());
        let result = ContractValidationResult {
            contract_id: ContractId::new("c1".into()).unwrap(),
            validation_timestamp: SystemTime::now(),
            compliance_score: 1.0,
            is_compatible: true,
            issues: vec![],
            scenario_results: vec![],
            environment: "test".into(),
        };

<<<<<<< HEAD
        let output = reporter.report(ReportFormat::Json, &result).await.unwrap();
=======
        let output = reporter
            .report(ReportFormat::Json, &result)
            .await
            .unwrap();
>>>>>>> origin/main
        assert_eq!(output, Some("ok".to_string()));
        assert!(*sink.called.lock().unwrap());
    }
}
<<<<<<< HEAD
=======

>>>>>>> origin/main
