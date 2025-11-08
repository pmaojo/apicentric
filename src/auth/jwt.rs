use jsonwebtoken::{encode, Header, EncodingKey, DecodingKey, Validation, decode};
use serde::{Serialize, Deserialize};
use std::time::{SystemTime, UNIX_EPOCH, Duration};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

pub struct JwtKeys {
    pub encoding: EncodingKey,
    pub decoding: DecodingKey,
}

impl JwtKeys {
    pub fn from_secret(secret: &str) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret.as_bytes()),
            decoding: DecodingKey::from_secret(secret.as_bytes()),
        }
    }
}

pub fn generate_token(username: &str, keys: &JwtKeys, ttl_hours: u64) -> Result<String, jsonwebtoken::errors::Error> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or(Duration::from_secs(0));
    let exp = now + Duration::from_secs(ttl_hours * 3600);
    let claims = Claims { sub: username.to_string(), exp: exp.as_secs() as usize };
    encode(&Header::default(), &claims, &keys.encoding)
}

pub fn validate_token(token: &str, keys: &JwtKeys) -> Result<Claims, jsonwebtoken::errors::Error> {
    let data = decode::<Claims>(token, &keys.decoding, &Validation::default())?;
    Ok(data.claims)
}
