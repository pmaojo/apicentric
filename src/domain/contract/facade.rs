use super::{
    ContractLoadingUseCase, OrchestrationError, ReportPublicationResult, ReportingUseCase,
    ScenarioExecutionUseCase, SpecValidationUseCase,
};
use crate::domain::contract_testing::{ContractId, ContractValidationResult};
use crate::domain::ports::contract::{
    ContractConfigurationProvider, ContractHttpClient, ContractMockApiRunner, ContractReportSink,
    ContractRepository, ServiceSpecLoader,
};

pub struct ContractTestingFacade<T, S, H, M, R>
where
    T: ContractRepository,
    S: ServiceSpecLoader,
    H: ContractHttpClient,
    M: ContractMockApiRunner,
    R: ContractReportSink,
{
    loader: ContractLoadingUseCase<T>,
    validator: SpecValidationUseCase<S>,
    executor: ScenarioExecutionUseCase<H, M>,
    reporter: ReportingUseCase<R>,
    config_provider: Box<dyn ContractConfigurationProvider>,
}

impl<T, S, H, M, R> ContractTestingFacade<T, S, H, M, R>
where
    T: ContractRepository,
    S: ServiceSpecLoader,
    H: ContractHttpClient,
    M: ContractMockApiRunner,
    R: ContractReportSink,
{
    pub fn new(
        loader: ContractLoadingUseCase<T>,
        validator: SpecValidationUseCase<S>,
        executor: ScenarioExecutionUseCase<H, M>,
        reporter: ReportingUseCase<R>,
        config_provider: Box<dyn ContractConfigurationProvider>,
    ) -> Self {
        Self {
            loader,
            validator,
            executor,
            reporter,
            config_provider,
        }
    }

    pub async fn execute_full_workflow(
        &self,
        contract_id: ContractId,
        environment: String,
        policy_name: String,
    ) -> Result<WorkflowResult, OrchestrationError> {
        let real_api_config = self
            .config_provider
            .get_real_api_config(&environment)
            .map_err(|e| OrchestrationError::ConfigurationError(e.to_string()))?;

        let compatibility_policy = self
            .config_provider
            .get_compatibility_policy(&policy_name)
            .map_err(|e| OrchestrationError::ConfigurationError(e.to_string()))?;

        let notification_config = self
            .config_provider
            .get_notification_config()
            .map_err(|e| OrchestrationError::ConfigurationError(e.to_string()))?;

        let contract = self
            .loader
            .execute(&contract_id)
            .await
            .map_err(|e| OrchestrationError::ValidationError(e.to_string()))?;

        let scenarios = self
            .validator
            .execute(&contract)
            .await
            .map_err(|e| OrchestrationError::ValidationError(e.to_string()))?;

        let validation_result = self
            .executor
            .execute(&contract, &real_api_config, &scenarios, &compatibility_policy)
            .await
            .map_err(|e| OrchestrationError::ValidationError(e.to_string()))?;

        let report_result = self
            .reporter
            .publish_validation_report(&validation_result)
            .await
            .map_err(|e| OrchestrationError::ReportingError(e.to_string()))?;

        self.reporter
            .send_notifications(&validation_result, &notification_config)
            .await
            .map_err(|e| OrchestrationError::ReportingError(e.to_string()))?;

        Ok(WorkflowResult {
            validation_result,
            report_result,
            success: true,
        })
    }
}

pub struct WorkflowResult {
    pub validation_result: ContractValidationResult,
    pub report_result: ReportPublicationResult,
    pub success: bool,
}
