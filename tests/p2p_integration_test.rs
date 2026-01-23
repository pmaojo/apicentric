#[cfg(feature = "p2p")]
#[tokio::test]
async fn test_p2p_service_sharing() {
    use apicentric::collab::share;

    // Attempt to share a service on a dummy port
    let port = 8080;
    let result = share::share_service(port).await;

    // We expect this to succeed and return a PeerId and a Token
    assert!(
        result.is_ok(),
        "Failed to share service: {:?}",
        result.err()
    );

    let (peer_id, token) = result.unwrap();
    println!(
        "P2P Share successful. PeerId: {}, Token: {}",
        peer_id, token
    );

    assert!(!token.is_empty(), "Token should not be empty");
}

#[cfg(not(feature = "p2p"))]
#[test]
fn test_p2p_disabled_by_default() {
    println!("P2P feature is not enabled. Skipping P2P tests.");
}
