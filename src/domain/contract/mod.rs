<<<<<<< HEAD
pub mod executor;
pub mod facade;
pub mod loader;
pub mod management;
pub mod reporter;
pub mod validator;

pub use executor::ScenarioExecutionUseCase;
pub use facade::{ContractTestingFacade, WorkflowResult};
pub use loader::ContractLoadingUseCase;
pub use management::ManageContractsUseCase;
pub use reporter::{ReportPublicationResult, ReportingUseCase};
pub use validator::SpecValidationUseCase;
=======
pub mod loader;
pub mod validator;
pub mod executor;
pub mod reporter;
pub mod facade;
pub mod management;

pub use loader::ContractLoadingUseCase;
pub use validator::SpecValidationUseCase;
pub use executor::ScenarioExecutionUseCase;
pub use reporter::{ReportingUseCase, ReportPublicationResult};
pub use facade::{ContractTestingFacade, WorkflowResult};
pub use management::ManageContractsUseCase;
>>>>>>> origin/main

#[derive(Debug, thiserror::Error)]
pub enum ContractUseCaseError {
    #[error("Contract not found: {0}")]
    ContractNotFound(String),

    #[error("Repository error: {0}")]
    RepositoryError(String),

    #[error("Spec load error: {0}")]
    SpecLoadError(String),

    #[error("Spec validation error: {0}")]
    SpecValidationError(String),

    #[error("Mock API error: {0}")]
    MockApiError(String),

    #[error("HTTP client error: {0}")]
    HttpClientError(String),

    #[error("Template error: {0}")]
    TemplateError(String),
}

#[derive(Debug, thiserror::Error)]
pub enum ContractManagementError {
    #[error("Repository error: {0}")]
    RepositoryError(String),

    #[error("Invalid spec: {0}")]
    InvalidSpec(String),

    #[error("Contract creation error: {0}")]
    ContractCreationError(String),
}

impl From<crate::domain::contract_testing::ContractValidationError> for ContractManagementError {
    fn from(err: crate::domain::contract_testing::ContractValidationError) -> Self {
        ContractManagementError::ContractCreationError(err.to_string())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ReportingError {
    #[error("Report generation error: {0}")]
    ReportGenerationError(String),

    #[error("Notification error: {0}")]
    NotificationError(String),

    #[error("Template error: {0}")]
    TemplateError(String),

    #[error("File system error: {0}")]
    FileSystemError(String),
}

#[derive(Debug, thiserror::Error)]
pub enum OrchestrationError {
    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Reporting error: {0}")]
    ReportingError(String),

    #[error("Workflow error: {0}")]
    WorkflowError(String),
}
