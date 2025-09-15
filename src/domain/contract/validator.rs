use super::ContractUseCaseError;
use crate::domain::contract_testing::{Contract, ValidationScenario};
use crate::domain::ports::contract::ServiceSpecLoader;

pub struct SpecValidationUseCase<S: ServiceSpecLoader> {
    spec_loader: S,
}

impl<S: ServiceSpecLoader> SpecValidationUseCase<S> {
    pub fn new(spec_loader: S) -> Self {
        Self { spec_loader }
    }

    pub async fn execute(
        &self,
        contract: &Contract,
    ) -> Result<Vec<ValidationScenario>, ContractUseCaseError> {
        let spec = self
            .spec_loader
            .load(&contract.spec_path)
            .await
            .map_err(|e| ContractUseCaseError::SpecLoadError(e.to_string()))?;

        self.spec_loader
            .validate(&spec)
            .await
            .map_err(|e| ContractUseCaseError::SpecValidationError(e.to_string()))?;

        let scenarios = self
            .spec_loader
            .extract_scenarios(&spec)
            .map_err(|e| ContractUseCaseError::SpecLoadError(e.to_string()))?;

        Ok(scenarios)
    }
}
