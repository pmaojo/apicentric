//! A CRDT for service definitions.
//!
//! This module provides a `ServiceCrdt` that can be used to synchronize service
//! definitions between peers.
<<<<<<< HEAD
//!
//! This module is only available when the `p2p` feature flag is enabled.

use automerge::{transaction::Transactable, AutoCommit, ReadDoc, ScalarValue, Value, ROOT};
=======

use automerge::{transaction::Transactable, AutoCommit, Value, ScalarValue, ReadDoc, ROOT};
>>>>>>> origin/main
use serde::{Deserialize, Serialize};

use crate::simulator::config::ServiceDefinition;

/// A wrapper around a `ServiceDefinition` that is backed by an Automerge CRDT
/// document.
pub struct ServiceCrdt {
    doc: AutoCommit,
}

impl ServiceCrdt {
    /// Creates a new CRDT document from an initial service definition.
    ///
    /// # Arguments
    ///
    /// * `service` - The initial service definition.
    pub fn new(service: ServiceDefinition) -> Self {
        let mut doc = AutoCommit::new();
        let json = serde_json::to_string(&service).expect("serialize service");
        // Store entire service definition as a single string field. This keeps the
        // implementation simple while still allowing Automerge to merge changes
        // originating from different peers.
        doc.put(ROOT, "service", json).expect("put service");
        Self { doc }
    }

    /// Constructs a `ServiceCrdt` from an existing encoded CRDT document.
    ///
    /// # Arguments
    ///
    /// * `data` - The encoded CRDT document.
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        AutoCommit::load(data).ok().map(|doc| Self { doc })
    }

    /// Encodes the current CRDT state into a byte vector that can be sent over
    /// the network to other peers.
    pub fn encode(&mut self) -> Vec<u8> {
        self.doc.save()
    }

    /// Merges another CRDT document (provided as raw bytes) into this one.
    ///
    /// # Arguments
    ///
    /// * `data` - The encoded CRDT document to merge.
    pub fn merge_bytes(&mut self, data: &[u8]) {
        if let Ok(mut other) = AutoCommit::load(data) {
            let _ = self.doc.merge(&mut other);
        }
    }

    /// Replaces the underlying service definition with a local change.
    ///
    /// # Arguments
    ///
    /// * `service` - The new service definition.
    pub fn apply_local_change(&mut self, service: ServiceDefinition) {
        let json = serde_json::to_string(&service).expect("serialize service");
        let _ = self.doc.put(ROOT, "service", json);
    }

    /// Extracts the current `ServiceDefinition` value from the CRDT document.
    pub fn to_service(&self) -> ServiceDefinition {
        match self.doc.get(ROOT, "service") {
            Ok(Some((Value::Scalar(sv), _))) => {
                if let ScalarValue::Str(s) = sv.as_ref() {
                    serde_json::from_str(s).expect("valid service json")
                } else {
                    panic!("service field not string")
                }
            }
            _ => panic!("service field missing in CRDT"),
        }
    }
}

/// A message exchanged between peers containing CRDT data for a service.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrdtMessage {
    /// The name of the service being updated.
    pub name: String,
    /// The encoded Automerge document bytes.
    pub data: Vec<u8>,
}
