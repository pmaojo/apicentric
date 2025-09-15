use std::time::SystemTime;

use super::ContractManagementError;
use crate::domain::contract_testing::{Contract, ContractEvent, ContractId};
use crate::domain::ports::contract::{
    ContractEventPublisher, ContractIdGenerator, ContractRepository, ServiceSpecLoader,
};
use tracing::{info, warn};

pub struct ManageContractsUseCase<T: ContractRepository, S: ServiceSpecLoader> {
    contract_repo: T,
    spec_loader: S,
    id_generator: Box<dyn ContractIdGenerator>,
    event_publisher: Box<dyn ContractEventPublisher>,
}

impl<T, S> ManageContractsUseCase<T, S>
where
    T: ContractRepository,
    S: ServiceSpecLoader,
{
    pub fn new(
        contract_repo: T,
        spec_loader: S,
        id_generator: Box<dyn ContractIdGenerator>,
        event_publisher: Box<dyn ContractEventPublisher>,
    ) -> Self {
        Self {
            contract_repo,
            spec_loader,
            id_generator,
            event_publisher,
        }
    }

    pub async fn register_contract(
        &self,
        service_name: String,
        spec_path: String,
        description: Option<String>,
    ) -> Result<Contract, ContractManagementError> {
        info!("Registering contract for service: {}", service_name);

        let spec = self
            .spec_loader
            .load(&spec_path)
            .await
            .map_err(|e| ContractManagementError::InvalidSpec(e.to_string()))?;

        self.spec_loader
            .validate(&spec)
            .await
            .map_err(|e| ContractManagementError::InvalidSpec(e.to_string()))?;

        let contract_id = self.id_generator.generate_contract_id();
        let contract = Contract::new(contract_id.clone(), service_name, spec_path, description)
            .map_err(|e| ContractManagementError::InvalidSpec(e.to_string()))?;

        self.contract_repo
            .save(&contract)
            .await
            .map_err(|e| ContractManagementError::RepositoryError(e.to_string()))?;

        let event = ContractEvent::ContractRegistered {
            contract_id: contract_id.clone(),
            service_name: contract.service_name.clone(),
            timestamp: SystemTime::now(),
        };

        if let Err(e) = self.event_publisher.publish(event).await {
            warn!("Failed to publish contract registration event: {}", e);
        }

        info!("Successfully registered contract: {}", contract_id);
        Ok(contract)
    }

    pub async fn list_contracts(&self) -> Result<Vec<Contract>, ContractManagementError> {
        self.contract_repo
            .list()
            .await
            .map_err(|e| ContractManagementError::RepositoryError(e.to_string()))
    }

    pub async fn get_contract(
        &self,
        contract_id: ContractId,
    ) -> Result<Option<Contract>, ContractManagementError> {
        self.contract_repo
            .get(&contract_id)
            .await
            .map_err(|e| ContractManagementError::RepositoryError(e.to_string()))
    }

    pub async fn delete_contract(
        &self,
        contract_id: ContractId,
    ) -> Result<(), ContractManagementError> {
        info!("Deleting contract: {}", contract_id);

        self.contract_repo
            .delete(&contract_id)
            .await
            .map_err(|e| ContractManagementError::RepositoryError(e.to_string()))?;

        let event = ContractEvent::ContractDeleted {
            contract_id: contract_id.clone(),
            timestamp: SystemTime::now(),
        };

        if let Err(e) = self.event_publisher.publish(event).await {
            warn!("Failed to publish contract deletion event: {}", e);
        }

        info!("Successfully deleted contract: {}", contract_id);
        Ok(())
    }
}
