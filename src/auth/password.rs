//! Utilities for hashing and verifying passwords.
//!
//! This module provides functions for hashing and verifying passwords using
//! the Argon2 password hashing algorithm.

use argon2::password_hash::{Error as PasswordError, SaltString};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use rand::rngs::OsRng;

/// Hashes a password using the Argon2 password hashing algorithm.
///
/// # Arguments
///
/// * `plain` - The password to hash.
///
/// # Returns
///
/// The hashed password.
pub fn hash_password(plain: &str) -> Result<String, PasswordError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2.hash_password(plain.as_bytes(), &salt)?;
    Ok(hash.to_string())
}

/// Verifies a password against a hash.
///
/// # Arguments
///
/// * `hash` - The hash to verify against.
/// * `attempt` - The password to verify.
///
/// # Returns
///
/// `true` if the password is correct, `false` otherwise.
pub fn verify_password(hash: &str, attempt: &str) -> Result<bool, PasswordError> {
    let parsed = PasswordHash::new(hash)?;
    let argon2 = Argon2::default();
    Ok(argon2.verify_password(attempt.as_bytes(), &parsed).is_ok())
}
