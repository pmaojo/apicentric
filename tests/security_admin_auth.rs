#[cfg(test)]
mod tests {
    use apicentric::simulator::admin_server::AdminServer;
    use apicentric::simulator::registry::ServiceRegistry;
    use apicentric::storage::sqlite::SqliteStorage;
    use apicentric::simulator::config::PortRange;
    use std::sync::Arc;
    use tokio::sync::{RwLock, broadcast};
    use std::time::Duration;
    use reqwest::StatusCode;

    // Run scenarios sequentially to avoid env var race conditions
    #[tokio::test]
    async fn test_admin_server_security_scenarios() {
        let (log_sender, _) = broadcast::channel(100);
        let storage = Arc::new(SqliteStorage::init_db(":memory:").unwrap());
        let registry = Arc::new(RwLock::new(ServiceRegistry::new(
            PortRange { start: 10000, end: 11000 },
            storage,
            log_sender,
        )));

        // --- Scenario 1: Fail Secure (No Token) ---
        println!("Running Scenario 1: No Token");
        std::env::remove_var("APICENTRIC_ADMIN_TOKEN");

        let mut server = AdminServer::new(registry.clone());
        let port1 = 15001;
        server.start(port1).await;
        tokio::time::sleep(Duration::from_millis(100)).await;

        let client = reqwest::Client::new();
        let resp = client.get(format!("http://127.0.0.1:{}/apicentric-admin/logs", port1))
            .send()
            .await
            .expect("Failed to connect");

        assert_eq!(resp.status(), StatusCode::FORBIDDEN, "Should be Forbidden when no token is set");
        server.stop().await;


        // --- Scenario 2: Auth Success (Correct Token) ---
        println!("Running Scenario 2: Correct Token");
        std::env::set_var("APICENTRIC_ADMIN_TOKEN", "test-secret-123");

        let mut server = AdminServer::new(registry.clone());
        let port2 = 15002;
        server.start(port2).await;
        tokio::time::sleep(Duration::from_millis(100)).await;

        let resp = client.get(format!("http://127.0.0.1:{}/apicentric-admin/logs", port2))
            .bearer_auth("test-secret-123")
            .send()
            .await
            .expect("Failed to connect");

        assert_eq!(resp.status(), StatusCode::OK, "Should be OK with correct token");
        server.stop().await;


        // --- Scenario 3: Auth Failure (Wrong Token) ---
        println!("Running Scenario 3: Wrong Token");
        // Token is still set to "test-secret-123"

        let mut server = AdminServer::new(registry.clone());
        let port3 = 15003;
        server.start(port3).await;
        tokio::time::sleep(Duration::from_millis(100)).await;

        let resp = client.get(format!("http://127.0.0.1:{}/apicentric-admin/logs", port3))
            .bearer_auth("wrong-token")
            .send()
            .await
            .expect("Failed to connect");

        assert_eq!(resp.status(), StatusCode::FORBIDDEN, "Should be Forbidden with wrong token");
        server.stop().await;

        // Cleanup
        std::env::remove_var("APICENTRIC_ADMIN_TOKEN");
    }
}
