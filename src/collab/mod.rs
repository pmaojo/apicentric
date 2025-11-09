//! Real-time collaboration features.
//!
//! This module provides the building blocks for real-time collaboration,
//! including a CRDT for service definitions, a peer-to-peer network layer,
//! and a service for sharing and synchronizing service definitions.

pub mod crdt;
pub mod p2p;
pub mod share;
