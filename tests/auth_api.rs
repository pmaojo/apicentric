//! Integration tests for authentication API endpoints.

use apicentric::auth::{
    handlers::AuthState,
    jwt::JwtKeys,
    repository::AuthRepository,
    blacklist::TokenBlacklist,
};
use std::sync::Arc;

/// Helper function to create a test auth state.
fn create_test_auth_state() -> Arc<AuthState> {
    let repo = AuthRepository::new(":memory:").expect("Failed to create in-memory repository");
    let keys = JwtKeys::from_secret("test-secret-key");
    let blacklist = TokenBlacklist::new();
    
    Arc::new(AuthState {
        repo: Arc::new(repo),
        keys,
        blacklist,
    })
}

#[tokio::test]
async fn test_token_blacklist_functionality() {
    let auth_state = create_test_auth_state();
    
    // Generate a token
    let token = apicentric::auth::jwt::generate_token("testuser", &auth_state.keys, 24)
        .expect("Failed to generate token");
    
    // Token should not be blacklisted initially
    assert!(!auth_state.blacklist.is_blacklisted(&token).await);
    
    // Add token to blacklist
    auth_state.blacklist.add(&token).await;
    
    // Token should now be blacklisted
    assert!(auth_state.blacklist.is_blacklisted(&token).await);
}

#[tokio::test]
async fn test_jwt_token_generation_and_validation() {
    let auth_state = create_test_auth_state();
    
    // Generate a token
    let token = apicentric::auth::jwt::generate_token("testuser", &auth_state.keys, 24)
        .expect("Failed to generate token");
    
    // Validate the token
    let claims = apicentric::auth::jwt::validate_token(&token, &auth_state.keys)
        .expect("Failed to validate token");
    
    assert_eq!(claims.sub, "testuser");
}

#[tokio::test]
async fn test_jwt_token_validation_fails_with_wrong_secret() {
    let auth_state = create_test_auth_state();
    let wrong_keys = JwtKeys::from_secret("wrong-secret");
    
    // Generate a token with one secret
    let token = apicentric::auth::jwt::generate_token("testuser", &auth_state.keys, 24)
        .expect("Failed to generate token");
    
    // Try to validate with different secret - should fail
    let result = apicentric::auth::jwt::validate_token(&token, &wrong_keys);
    assert!(result.is_err());
}

#[test]
fn test_password_hashing_and_verification() {
    let password = "test_password_123";
    
    // Hash the password
    let hash = apicentric::auth::password::hash_password(password)
        .expect("Failed to hash password");
    
    // Verify correct password
    let valid = apicentric::auth::password::verify_password(&hash, password)
        .expect("Failed to verify password");
    assert!(valid);
    
    // Verify incorrect password
    let invalid = apicentric::auth::password::verify_password(&hash, "wrong_password")
        .expect("Failed to verify password");
    assert!(!invalid);
}

#[tokio::test]
async fn test_user_registration_and_retrieval() {
    let repo = AuthRepository::new(":memory:").expect("Failed to create repository");
    
    // Create a user
    let username = "testuser";
    let password_hash = "hashed_password";
    
    repo.create_user(username.to_string(), password_hash.to_string()).await
        .expect("Failed to create user");
    
    // Retrieve the user
    let user = repo.find_by_username(username.to_string()).await
        .expect("Failed to find user")
        .expect("User not found");
    
    assert_eq!(user.username, username);
    assert_eq!(user.password_hash, password_hash);
}

#[tokio::test]
async fn test_duplicate_user_registration_fails() {
    let repo = AuthRepository::new(":memory:").expect("Failed to create repository");
    
    let username = "testuser";
    let password_hash = "hashed_password";
    
    // Create first user
    repo.create_user(username.to_string(), password_hash.to_string()).await
        .expect("Failed to create first user");
    
    // Try to create duplicate user - should fail
    let result = repo.create_user(username.to_string(), password_hash.to_string()).await;
    assert!(result.is_err());
}
