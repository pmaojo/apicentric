use apicentric::cloud::recording_session::RecordingSessionManager;
use bytes::Bytes;
use http_body_util::Full;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use std::convert::Infallible;
use std::env;
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[tokio::test]
async fn test_recording_proxy_functionality() {
    // 1. Start a dummy target server
    let addr = SocketAddr::from(([127, 0, 0, 1], 0));
    let listener = TcpListener::bind(addr).await.unwrap();
    let target_port = listener.local_addr().unwrap().port();
    let target_url = format!("http://127.0.0.1:{}", target_port);

    tokio::spawn(async move {
        loop {
            if let Ok((stream, _)) = listener.accept().await {
                let io = TokioIo::new(stream);
                tokio::task::spawn(async move {
                    let service = service_fn(|req: Request<hyper::body::Incoming>| async move {
                        let path = req.uri().path();
                        if path == "/echo" {
                            Ok::<_, Infallible>(Response::new(Full::new(Bytes::from(
                                "echo-response",
                            ))))
                        } else {
                            Ok::<_, Infallible>(Response::new(Full::new(Bytes::from(
                                "hello-world",
                            ))))
                        }
                    });
                    if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
                        eprintln!("Error serving connection: {:?}", err);
                    }
                });
            }
        }
    });

    // 2. Allow private IPs for the test
    unsafe {
        env::set_var("APICENTRIC_ALLOW_PRIVATE_IPS", "true");
    }

    // 3. Start recording proxy
    let manager = RecordingSessionManager::new();

    // Find a free port for the proxy
    let proxy_listener = TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 0)))
        .await
        .unwrap();
    let proxy_port = proxy_listener.local_addr().unwrap().port();
    drop(proxy_listener); // Free the port

    // start_recording
    let (session_id, proxy_url, actual_proxy_port) = manager
        .start_recording(target_url.clone(), proxy_port)
        .await
        .expect("Failed to start recording");

    assert_eq!(proxy_port, actual_proxy_port);

    // 4. Send request to proxy
    // Wait a bit for proxy to start listening
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{}/echo", proxy_url))
        .send()
        .await
        .expect("Failed to send request to proxy");

    assert_eq!(resp.status(), 200);
    let body = resp.text().await.unwrap();
    assert_eq!(body, "echo-response");

    // 5. Stop recording and verify capture
    let (stopped_session_id, endpoints) = manager
        .stop_recording()
        .await
        .expect("Failed to stop recording");
    assert_eq!(session_id, stopped_session_id);
    assert!(!endpoints.is_empty());

    // Find the endpoint
    let endpoint = endpoints
        .iter()
        .find(|e| e.path == "/echo")
        .expect("Endpoint not captured");
    assert_eq!(endpoint.method, "GET");

    // 6. Test SSRF protection (disable env var)
    unsafe {
        env::remove_var("APICENTRIC_ALLOW_PRIVATE_IPS");
    }

    // Use a NEW manager to avoid any state issues (though manager is stateless regarding env var)
    // But validate_ssrf_url reads env var dynamically.

    // We need to stop the previous proxy session first? It was stopped in step 5.

    // Reuse port? better find new one.
    let proxy_listener = TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 0)))
        .await
        .unwrap();
    let proxy_port_2 = proxy_listener.local_addr().unwrap().port();
    drop(proxy_listener);

    let result = manager
        .start_recording(target_url.clone(), proxy_port_2)
        .await;
    assert!(
        result.is_err(),
        "Should block private IP when env var is unset"
    );
}
