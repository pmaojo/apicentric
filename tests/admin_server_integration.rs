use apicentric::simulator::admin_server::AdminServer;
use apicentric::simulator::config::PortRange;
use apicentric::simulator::registry::ServiceRegistry;
use apicentric::storage::sqlite::SqliteStorage;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::sync::RwLock;

#[tokio::test]
async fn test_admin_server_lifecycle_and_auth() {
    // Prepare shared resources
    let storage = Arc::new(SqliteStorage::init_db(":memory:").expect("Failed to init db"));
    let (tx, _rx) = broadcast::channel(100);
    let registry = Arc::new(RwLock::new(ServiceRegistry::new(
        PortRange {
            start: 9000,
            end: 9100,
        },
        storage,
        tx,
    )));

    // --- Scenario 1: No Auth ---
    // Ensure no token is set
    unsafe {
        std::env::remove_var("APICENTRIC_ADMIN_TOKEN");
    }

    let mut server = AdminServer::new(registry.clone());
    let port = 9050;
    server.start(port).await;

    // Allow startup
    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = reqwest::Client::new();
    let resp = client
        .get(format!("http://127.0.0.1:{}/apicentric-admin/logs", port))
        .send()
        .await
        .expect("Request failed");

    assert_eq!(resp.status(), 200, "Should be 200 OK without token set");

    server.stop().await;
    // Allow shutdown
    tokio::time::sleep(Duration::from_millis(100)).await;

    // --- Scenario 2: With Auth ---
    let token = "secret_token_123";
    unsafe {
        std::env::set_var("APICENTRIC_ADMIN_TOKEN", token);
    }

    let mut server = AdminServer::new(registry.clone());

    // Unset immediately to avoid leaking to other tests,
    // as AdminServer caches it in `new`.
    unsafe {
        std::env::remove_var("APICENTRIC_ADMIN_TOKEN");
    }

    let port = 9051; // Use different port just in case
    server.start(port).await;

    tokio::time::sleep(Duration::from_millis(100)).await;

    // 2a. No header -> 401
    let resp = client
        .get(format!("http://127.0.0.1:{}/apicentric-admin/logs", port))
        .send()
        .await
        .expect("Request failed");
    assert_eq!(
        resp.status(),
        401,
        "Should be 401 Unauthorized without header"
    );

    // 2b. Wrong header -> 403 (or 401 depending on logic flow, let's verify)
    let resp = client
        .get(format!("http://127.0.0.1:{}/apicentric-admin/logs", port))
        .header("Authorization", "Bearer wrong")
        .send()
        .await
        .expect("Request failed");

    // Based on implementation: if header exists but token mismatch -> Forbidden (403)
    assert_eq!(
        resp.status(),
        403,
        "Should be 403 Forbidden with wrong token"
    );

    // 2c. Correct header -> 200
    let resp = client
        .get(format!("http://127.0.0.1:{}/apicentric-admin/logs", port))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Request failed");
    assert_eq!(resp.status(), 200, "Should be 200 OK with correct token");

    server.stop().await;
}
