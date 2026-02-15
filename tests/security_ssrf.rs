#![cfg(feature = "webui")]

use apicentric::cloud::recording_session::RecordingSessionManager;

#[tokio::test]
async fn test_ssrf_prevention_recording() {
    // Ensure we are enforcing security
    std::env::remove_var("APICENTRIC_ALLOW_PRIVATE_IPS");

    let manager = RecordingSessionManager::new();

    // 1. Loopback (IPv4)
    // This should fail with "Host '127.0.0.1' resolves to a private, loopback, or invalid IP address"
    let result = manager
        .start_recording("http://127.0.0.1:8080".to_string(), 0)
        .await;

    // Clean up if it succeeded (vulnerable behavior)
    if result.is_ok() {
        let _ = manager.stop_recording().await;
    }

    assert!(
        result.is_err(),
        "Should block 127.0.0.1 (Loopback IPv4)"
    );

    // 2. Loopback (IPv6)
    let result = manager
        .start_recording("http://[::1]:8080".to_string(), 0)
        .await;

    if result.is_ok() {
        let _ = manager.stop_recording().await;
    }

    assert!(
        result.is_err(),
        "Should block [::1] (Loopback IPv6)"
    );

    // 3. Private IP
    let result = manager
        .start_recording("http://192.168.1.50:8080".to_string(), 0)
        .await;

    if result.is_ok() {
        let _ = manager.stop_recording().await;
    }

    assert!(
        result.is_err(),
        "Should block 192.168.x.x (Private IPv4)"
    );

    // 4. Link Local
    let result = manager
        .start_recording("http://169.254.169.254:80".to_string(), 0)
        .await;

    if result.is_ok() {
        let _ = manager.stop_recording().await;
    }

    assert!(
        result.is_err(),
        "Should block 169.254.x.x (Link-Local)"
    );
}
