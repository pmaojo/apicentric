//! Real-time collaboration features.
//!
//! This module provides the building blocks for real-time collaboration,
//! including a CRDT for service definitions, a peer-to-peer network layer,
//! and a service for sharing and synchronizing service definitions.
<<<<<<< HEAD
//!
//! # Feature Flag
//!
//! This module is only available when the `p2p` feature flag is enabled.
//! When disabled, stub implementations are provided that indicate P2P
//! functionality is unavailable.
//!
//! To enable P2P features, build with:
//! ```bash
//! cargo build --features p2p
//! ```

#[cfg(feature = "p2p")]
pub mod crdt;
#[cfg(feature = "p2p")]
pub mod p2p;
#[cfg(feature = "p2p")]
pub mod share;

/// Check if P2P collaboration features are available at runtime.
///
/// Returns `true` if the binary was compiled with the `p2p` feature flag,
/// `false` otherwise.
#[cfg(feature = "p2p")]
pub fn is_available() -> bool {
    true
}

/// Check if P2P collaboration features are available at runtime.
///
/// Returns `true` if the binary was compiled with the `p2p` feature flag,
/// `false` otherwise.
#[cfg(not(feature = "p2p"))]
pub fn is_available() -> bool {
    false
}

/// Returns a user-friendly message explaining P2P availability status.
#[cfg(feature = "p2p")]
pub fn availability_message() -> &'static str {
    "P2P collaboration features are enabled"
}

/// Returns a user-friendly message explaining P2P availability status.
#[cfg(not(feature = "p2p"))]
pub fn availability_message() -> &'static str {
    "P2P collaboration features are not available. Rebuild with --features p2p to enable."
}
=======

pub mod crdt;
pub mod p2p;
pub mod share;
>>>>>>> origin/main
