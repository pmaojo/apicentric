#![cfg(feature = "webui")]
use apicentric::cloud::recording_session::RecordingSessionManager;

#[tokio::test]
async fn test_recording_ssrf_prevention() {
    let manager = RecordingSessionManager::new();

    // 1. Loopback (IPv4)
    // This should fail with an SSRF validation error.
    let result = manager.start_recording("http://127.0.0.1:8080".to_string(), 0).await;
    assert!(result.is_err(), "Should block 127.0.0.1");
    if result.is_ok() {
        let _ = manager.stop_recording().await;
    }

    // 2. Loopback (IPv6)
    let result = manager.start_recording("http://[::1]:8080".to_string(), 0).await;
    assert!(result.is_err(), "Should block [::1]");
    if result.is_ok() {
        let _ = manager.stop_recording().await;
    }

    // 3. Private IP (Class C)
    let result = manager.start_recording("http://192.168.1.50:8080".to_string(), 0).await;
    assert!(result.is_err(), "Should block 192.168.x.x");
    if result.is_ok() {
        let _ = manager.stop_recording().await;
    }

    // 4. Link Local
    let result = manager.start_recording("http://169.254.169.254:80".to_string(), 0).await;
    assert!(result.is_err(), "Should block 169.254.x.x");
    if result.is_ok() {
        let _ = manager.stop_recording().await;
    }
}
