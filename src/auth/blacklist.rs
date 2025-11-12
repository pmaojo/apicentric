//! Token blacklist for JWT revocation.
//!
//! This module provides a token blacklist mechanism for invalidating JWTs
//! before their expiration time. This is used for logout functionality.

use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;

/// A thread-safe token blacklist.
#[derive(Clone)]
pub struct TokenBlacklist {
    /// The set of blacklisted token IDs (using token hash for efficiency).
    tokens: Arc<RwLock<HashSet<String>>>,
}

impl TokenBlacklist {
    /// Creates a new empty token blacklist.
    pub fn new() -> Self {
        Self {
            tokens: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    /// Adds a token to the blacklist.
    ///
    /// # Arguments
    ///
    /// * `token` - The token to blacklist.
    pub async fn add(&self, token: &str) {
        let token_hash = Self::hash_token(token);
        let mut tokens = self.tokens.write().await;
        tokens.insert(token_hash);
    }

    /// Checks if a token is blacklisted.
    ///
    /// # Arguments
    ///
    /// * `token` - The token to check.
    ///
    /// # Returns
    ///
    /// `true` if the token is blacklisted, `false` otherwise.
    pub async fn is_blacklisted(&self, token: &str) -> bool {
        let token_hash = Self::hash_token(token);
        let tokens = self.tokens.read().await;
        tokens.contains(&token_hash)
    }

    /// Removes expired tokens from the blacklist.
    ///
    /// This should be called periodically to prevent memory growth.
    /// In a production system, you might want to store expiration times
    /// with tokens and clean them up automatically.
    pub async fn cleanup_expired(&self) {
        // For now, we'll keep all tokens until server restart
        // In production, you'd want to track expiration times
        // and remove tokens that have expired
    }

    /// Hashes a token for storage (to save memory).
    fn hash_token(token: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        token.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}

impl Default for TokenBlacklist {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_blacklist_add_and_check() {
        let blacklist = TokenBlacklist::new();
        let token = "test.token.here";

        assert!(!blacklist.is_blacklisted(token).await);
        
        blacklist.add(token).await;
        
        assert!(blacklist.is_blacklisted(token).await);
    }

    #[tokio::test]
    async fn test_blacklist_different_tokens() {
        let blacklist = TokenBlacklist::new();
        let token1 = "token1";
        let token2 = "token2";

        blacklist.add(token1).await;
        
        assert!(blacklist.is_blacklisted(token1).await);
        assert!(!blacklist.is_blacklisted(token2).await);
    }
}
