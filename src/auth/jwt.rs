//! Utilities for working with JSON Web Tokens (JWTs).
//!
//! This module provides functions for generating and validating JWTs.

<<<<<<< HEAD
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// The claims in a JWT.
#[derive(Debug, Clone, Serialize, Deserialize)]
=======
use jsonwebtoken::{encode, Header, EncodingKey, DecodingKey, Validation, decode};
use serde::{Serialize, Deserialize};
use std::time::{SystemTime, UNIX_EPOCH, Duration};

/// The claims in a JWT.
#[derive(Debug, Serialize, Deserialize)]
>>>>>>> origin/main
pub struct Claims {
    /// The subject of the token.
    pub sub: String,
    /// The expiration time of the token.
    pub exp: usize,
}

/// The keys used for encoding and decoding JWTs.
pub struct JwtKeys {
    /// The key used for encoding.
    pub encoding: EncodingKey,
    /// The key used for decoding.
    pub decoding: DecodingKey,
}

impl JwtKeys {
    /// Creates a new `JwtKeys` from a secret.
    ///
    /// # Arguments
    ///
    /// * `secret` - The secret to use for encoding and decoding.
    pub fn from_secret(secret: &str) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret.as_bytes()),
            decoding: DecodingKey::from_secret(secret.as_bytes()),
        }
    }
}

/// Generates a JWT.
///
/// # Arguments
///
/// * `username` - The username to include in the token.
/// * `keys` - The keys to use for encoding the token.
/// * `ttl_hours` - The time-to-live of the token in hours.
///
/// # Returns
///
/// The generated JWT.
<<<<<<< HEAD
pub fn generate_token(
    username: &str,
    keys: &JwtKeys,
    ttl_hours: u64,
) -> Result<String, jsonwebtoken::errors::Error> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0));
    let exp = now + Duration::from_secs(ttl_hours * 3600);
    let claims = Claims {
        sub: username.to_string(),
        exp: exp.as_secs() as usize,
    };
=======
pub fn generate_token(username: &str, keys: &JwtKeys, ttl_hours: u64) -> Result<String, jsonwebtoken::errors::Error> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or(Duration::from_secs(0));
    let exp = now + Duration::from_secs(ttl_hours * 3600);
    let claims = Claims { sub: username.to_string(), exp: exp.as_secs() as usize };
>>>>>>> origin/main
    encode(&Header::default(), &claims, &keys.encoding)
}

/// Validates a JWT.
///
/// # Arguments
///
/// * `token` - The token to validate.
/// * `keys` - The keys to use for decoding the token.
///
/// # Returns
///
/// The claims in the token if the token is valid.
pub fn validate_token(token: &str, keys: &JwtKeys) -> Result<Claims, jsonwebtoken::errors::Error> {
    let data = decode::<Claims>(token, &keys.decoding, &Validation::default())?;
    Ok(data.claims)
}
