//! Implements the `ContractRepository` port for persisting contracts.
//!
//! This module provides two implementations of the `ContractRepository` trait:
//!
//! - `FileSystemContractRepository`: A file-based implementation that stores
//!   contracts as JSON files in a specified directory.
//! - `InMemoryContractRepository`: An in-memory implementation for testing and
//!   development.

use crate::domain::contract_testing::*;
use crate::domain::ports::contract::{ContractRepository, RepositoryError};
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// A file-based implementation of the `ContractRepository` trait.
///
/// This repository stores contracts as JSON files in a specified directory.
/// It also maintains an in-memory cache for better performance.
pub struct FileSystemContractRepository {
    storage_path: PathBuf,
    // In-memory cache for better performance
    cache: RwLock<HashMap<String, Contract>>,
}

impl FileSystemContractRepository {
    /// Creates a new `FileSystemContractRepository`.
    ///
    /// # Arguments
    ///
    /// * `storage_path` - The path to the directory where contracts will be
    ///   stored.
    ///
    /// # Returns
    ///
    /// A new `FileSystemContractRepository` instance.
    pub async fn new<P: AsRef<Path>>(storage_path: P) -> Result<Self, RepositoryError> {
        let storage_path = storage_path.as_ref().to_path_buf();

        // Ensure the storage directory exists
        if !storage_path.exists() {
            fs::create_dir_all(&storage_path).await.map_err(|e| {
                RepositoryError::StorageError(format!("Failed to create storage directory: {}", e))
            })?;
            info!("Created storage directory: {}", storage_path.display());
        }

        let repository = Self {
            storage_path,
            cache: RwLock::new(HashMap::new()),
        };

        // Load existing contracts into cache
        repository.load_cache().await?;

        Ok(repository)
    }

    async fn load_cache(&self) -> Result<(), RepositoryError> {
        let mut cache = self.cache.write().await;

        let mut entries = fs::read_dir(&self.storage_path).await.map_err(|e| {
            RepositoryError::StorageError(format!("Failed to read storage directory: {}", e))
        })?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| RepositoryError::StorageError(e.to_string()))?
        {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
                match self.load_contract_from_file(&path).await {
                    Ok(contract) => {
                        cache.insert(contract.id.to_string(), contract);
                    }
                    Err(e) => {
                        warn!("Failed to load contract from {}: {}", path.display(), e);
                    }
                }
            }
        }

        info!("Loaded {} contracts into cache", cache.len());
        Ok(())
    }

    async fn load_contract_from_file(&self, path: &Path) -> Result<Contract, RepositoryError> {
        let content = fs::read_to_string(path).await.map_err(|e| {
            RepositoryError::StorageError(format!("Failed to read file {}: {}", path.display(), e))
        })?;

        let contract: Contract = serde_json::from_str(&content).map_err(|e| {
            RepositoryError::SerializationError(format!(
                "Failed to deserialize contract from {}: {}",
                path.display(),
                e
            ))
        })?;

        Ok(contract)
    }

    async fn save_contract_to_file(&self, contract: &Contract) -> Result<(), RepositoryError> {
        let filename = format!("{}.json", contract.id);
        let file_path = self.storage_path.join(filename);

        let json = serde_json::to_string_pretty(contract).map_err(|e| {
            RepositoryError::SerializationError(format!("Failed to serialize contract: {}", e))
        })?;

        fs::write(&file_path, json).await.map_err(|e| {
            RepositoryError::StorageError(format!(
                "Failed to write contract to {}: {}",
                file_path.display(),
                e
            ))
        })?;

        debug!("Saved contract {} to {}", contract.id, file_path.display());
        Ok(())
    }

    async fn delete_contract_file(&self, contract_id: &ContractId) -> Result<(), RepositoryError> {
        let filename = format!("{}.json", contract_id);
        let file_path = self.storage_path.join(filename);

        if file_path.exists() {
            fs::remove_file(&file_path).await.map_err(|e| {
                RepositoryError::StorageError(format!(
                    "Failed to delete contract file {}: {}",
                    file_path.display(),
                    e
                ))
            })?;
            debug!("Deleted contract file: {}", file_path.display());
        }

        Ok(())
    }
}

#[async_trait]
impl ContractRepository for FileSystemContractRepository {
    /// Saves a contract to the repository.
    ///
    /// # Arguments
    ///
    /// * `contract` - The contract to save.
    async fn save(&self, contract: &Contract) -> Result<(), RepositoryError> {
        debug!("Saving contract: {}", contract.id);

        // Save to file
        self.save_contract_to_file(contract).await?;

        // Update cache
        let mut cache = self.cache.write().await;
        cache.insert(contract.id.to_string(), contract.clone());

        info!("Successfully saved contract: {}", contract.id);
        Ok(())
    }

    /// Gets a contract from the repository by its ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the contract to get.
    ///
    /// # Returns
    ///
    /// An `Option` containing the contract if it was found, or `None` if it was
    /// not.
    async fn get(&self, id: &ContractId) -> Result<Option<Contract>, RepositoryError> {
        debug!("Retrieving contract: {}", id);

        // Check cache first
        let cache = self.cache.read().await;
        if let Some(contract) = cache.get(&id.to_string()) {
            debug!("Contract found in cache: {}", id);
            return Ok(Some(contract.clone()));
        }
        drop(cache);

        // Try to load from file if not in cache
        let filename = format!("{}.json", id);
        let file_path = self.storage_path.join(filename);

        if !file_path.exists() {
            debug!("Contract not found: {}", id);
            return Ok(None);
        }

        match self.load_contract_from_file(&file_path).await {
            Ok(contract) => {
                // Update cache
                let mut cache = self.cache.write().await;
                cache.insert(id.to_string(), contract.clone());

                debug!("Contract loaded from file: {}", id);
                Ok(Some(contract))
            }
            Err(e) => {
                error!("Failed to load contract {} from file: {}", id, e);
                Err(e)
            }
        }
    }

    /// Lists all contracts in the repository.
    ///
    /// # Returns
    ///
    /// A `Vec` of all contracts in the repository.
    async fn list(&self) -> Result<Vec<Contract>, RepositoryError> {
        debug!("Listing all contracts");

        let cache = self.cache.read().await;
        let contracts: Vec<Contract> = cache.values().cloned().collect();

        debug!("Found {} contracts", contracts.len());
        Ok(contracts)
    }

    /// Deletes a contract from the repository by its ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the contract to delete.
    async fn delete(&self, id: &ContractId) -> Result<(), RepositoryError> {
        debug!("Deleting contract: {}", id);

        // Remove from cache
        let mut cache = self.cache.write().await;
        let existed = cache.remove(&id.to_string()).is_some();
        drop(cache);

        if !existed {
            return Err(RepositoryError::NotFound(id.to_string()));
        }

        // Delete file
        self.delete_contract_file(id).await?;

        info!("Successfully deleted contract: {}", id);
        Ok(())
    }

    /// Finds all contracts for a given service.
    ///
    /// # Arguments
    ///
    /// * `service_name` - The name of the service to find contracts for.
    ///
    /// # Returns
    ///
    /// A `Vec` of all contracts for the given service.
    async fn find_by_service(&self, service_name: &str) -> Result<Vec<Contract>, RepositoryError> {
        debug!("Finding contracts for service: {}", service_name);

        let cache = self.cache.read().await;
        let contracts: Vec<Contract> = cache
            .values()
            .filter(|contract| contract.service_name == service_name)
            .cloned()
            .collect();

        debug!(
            "Found {} contracts for service: {}",
            contracts.len(),
            service_name
        );
        Ok(contracts)
    }
}

/// An in-memory implementation of the `ContractRepository` trait for testing
/// and development.
pub struct InMemoryContractRepository {
    contracts: RwLock<HashMap<String, Contract>>,
}

impl InMemoryContractRepository {
    /// Creates a new `InMemoryContractRepository`.
    pub fn new() -> Self {
        Self {
            contracts: RwLock::new(HashMap::new()),
        }
    }

    /// Clears all contracts from the repository.
    pub async fn clear(&self) {
        let mut contracts = self.contracts.write().await;
        contracts.clear();
    }

    /// Returns the number of contracts in the repository.
    pub async fn count(&self) -> usize {
        let contracts = self.contracts.read().await;
        contracts.len()
    }
}

#[async_trait]
impl ContractRepository for InMemoryContractRepository {
    /// Saves a contract to the repository.
    ///
    /// # Arguments
    ///
    /// * `contract` - The contract to save.
    async fn save(&self, contract: &Contract) -> Result<(), RepositoryError> {
        debug!("Saving contract to memory: {}", contract.id);

        let mut contracts = self.contracts.write().await;
        contracts.insert(contract.id.to_string(), contract.clone());

        info!("Successfully saved contract to memory: {}", contract.id);
        Ok(())
    }

    /// Gets a contract from the repository by its ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the contract to get.
    ///
    /// # Returns
    ///
    /// An `Option` containing the contract if it was found, or `None` if it was
    /// not.
    async fn get(&self, id: &ContractId) -> Result<Option<Contract>, RepositoryError> {
        debug!("Retrieving contract from memory: {}", id);

        let contracts = self.contracts.read().await;
        let contract = contracts.get(&id.to_string()).cloned();

        if contract.is_some() {
            debug!("Contract found in memory: {}", id);
        } else {
            debug!("Contract not found in memory: {}", id);
        }

        Ok(contract)
    }

    /// Lists all contracts in the repository.
    ///
    /// # Returns
    ///
    /// A `Vec` of all contracts in the repository.
    async fn list(&self) -> Result<Vec<Contract>, RepositoryError> {
        debug!("Listing all contracts from memory");

        let contracts = self.contracts.read().await;
        let contract_list: Vec<Contract> = contracts.values().cloned().collect();

        debug!("Found {} contracts in memory", contract_list.len());
        Ok(contract_list)
    }

    /// Deletes a contract from the repository by its ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the contract to delete.
    async fn delete(&self, id: &ContractId) -> Result<(), RepositoryError> {
        debug!("Deleting contract from memory: {}", id);

        let mut contracts = self.contracts.write().await;
        let existed = contracts.remove(&id.to_string()).is_some();

        if !existed {
            return Err(RepositoryError::NotFound(id.to_string()));
        }

        info!("Successfully deleted contract from memory: {}", id);
        Ok(())
    }

    /// Finds all contracts for a given service.
    ///
    /// # Arguments
    ///
    /// * `service_name` - The name of the service to find contracts for.
    ///
    /// # Returns
    ///
    /// A `Vec` of all contracts for the given service.
    async fn find_by_service(&self, service_name: &str) -> Result<Vec<Contract>, RepositoryError> {
        debug!("Finding contracts for service in memory: {}", service_name);

        let contracts = self.contracts.read().await;
        let matching_contracts: Vec<Contract> = contracts
            .values()
            .filter(|contract| contract.service_name == service_name)
            .cloned()
            .collect();

        debug!(
            "Found {} contracts for service in memory: {}",
            matching_contracts.len(),
            service_name
        );
        Ok(matching_contracts)
    }
}

impl Default for InMemoryContractRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_in_memory_repository() {
        let repo = InMemoryContractRepository::new();

        // Test empty repository
        assert_eq!(repo.count().await, 0);
        let contracts = repo.list().await.unwrap();
        assert!(contracts.is_empty());

        // Create a test contract
        let contract = Contract::new(
            ContractId::new("test-contract-1".to_string()).unwrap(),
            "test-service".to_string(),
            "/path/to/spec.yaml".to_string(),
            Some("Test contract".to_string()),
        )
        .unwrap();

        // Test save
        repo.save(&contract).await.unwrap();
        assert_eq!(repo.count().await, 1);

        // Test get
        let retrieved = repo.get(&contract.id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, contract.id);

        // Test find by service
        let service_contracts = repo.find_by_service("test-service").await.unwrap();
        assert_eq!(service_contracts.len(), 1);

        // Test delete
        repo.delete(&contract.id).await.unwrap();
        assert_eq!(repo.count().await, 0);

        // Test delete non-existent
        let result = repo.delete(&contract.id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_filesystem_repository() {
        let temp_dir = TempDir::new().unwrap();
        let repo = FileSystemContractRepository::new(temp_dir.path())
            .await
            .unwrap();

        // Create a test contract
        let contract = Contract::new(
            ContractId::new("test-contract-fs".to_string()).unwrap(),
            "fs-test-service".to_string(),
            "/path/to/fs-spec.yaml".to_string(),
            Some("Filesystem test contract".to_string()),
        )
        .unwrap();

        // Test save
        repo.save(&contract).await.unwrap();

        // Test get
        let retrieved = repo.get(&contract.id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().service_name, "fs-test-service");

        // Test persistence (create new repository instance)
        let repo2 = FileSystemContractRepository::new(temp_dir.path())
            .await
            .unwrap();
        let retrieved2 = repo2.get(&contract.id).await.unwrap();
        assert!(retrieved2.is_some());

        // Test delete
        repo2.delete(&contract.id).await.unwrap();
        let retrieved3 = repo2.get(&contract.id).await.unwrap();
        assert!(retrieved3.is_none());
    }
}
