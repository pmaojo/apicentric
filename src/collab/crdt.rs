use automerge::{transaction::Transactable, AutoCommit, Value, ScalarValue, ReadDoc, ROOT};
use serde::{Deserialize, Serialize};

use crate::simulator::config::ServiceDefinition;

/// Wrapper around [`ServiceDefinition`] backed by an Automerge CRDT document.
pub struct ServiceCrdt {
    doc: AutoCommit,
}

impl ServiceCrdt {
    /// Create a new CRDT document from an initial service definition.
    pub fn new(service: ServiceDefinition) -> Self {
        let mut doc = AutoCommit::new();
        let json = serde_json::to_string(&service).expect("serialize service");
        // Store entire service definition as a single string field. This keeps the
        // implementation simple while still allowing Automerge to merge changes
        // originating from different peers.
        doc.put(ROOT, "service", json).expect("put service");
        Self { doc }
    }

    /// Construct from an existing encoded CRDT document.
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        AutoCommit::load(data).ok().map(|doc| Self { doc })
    }

    /// Encode the current CRDT state into a byte vector that can be sent over
    /// the network to other peers.
    pub fn encode(&mut self) -> Vec<u8> {
        self.doc.save()
    }

    /// Merge another CRDT document (provided as raw bytes) into this one.
    pub fn merge_bytes(&mut self, data: &[u8]) {
        if let Ok(mut other) = AutoCommit::load(data) {
            let _ = self.doc.merge(&mut other);
        }
    }

    /// Replace the underlying service definition with a local change.
    pub fn apply_local_change(&mut self, service: ServiceDefinition) {
        let json = serde_json::to_string(&service).expect("serialize service");
        let _ = self.doc.put(ROOT, "service", json);
    }

    /// Extract the current [`ServiceDefinition`] value from the CRDT document.
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

/// Message exchanged between peers containing CRDT data for a service.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrdtMessage {
    /// Name of the service being updated.
    pub name: String,
    /// Encoded Automerge document bytes.
    pub data: Vec<u8>,
}
