use super::ContractUseCaseError;
use crate::domain::contract_testing::{Contract, ContractId};
use crate::domain::ports::contract::ContractRepository;

pub struct ContractLoadingUseCase<T: ContractRepository> {
    contract_repo: T,
}

impl<T: ContractRepository> ContractLoadingUseCase<T> {
    pub fn new(contract_repo: T) -> Self {
        Self { contract_repo }
    }

    pub async fn execute(
        &self,
        contract_id: &ContractId,
    ) -> Result<Contract, ContractUseCaseError> {
        self.contract_repo
            .get(contract_id)
            .await
            .map_err(|e| ContractUseCaseError::RepositoryError(e.to_string()))?
            .ok_or_else(|| ContractUseCaseError::ContractNotFound(contract_id.to_string()))
    }
}
