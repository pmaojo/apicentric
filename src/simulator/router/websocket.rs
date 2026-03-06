use bytes::Bytes;
#[cfg(feature = "websockets")]
use futures_util::SinkExt;
use http_body_util::Full;
use hyper::{Request, Response, StatusCode};
#[cfg(feature = "websockets")]
use hyper_util::rt::TokioIo;
use std::sync::Arc;
use tokio::time::{interval, Duration};
#[cfg(feature = "websockets")]
use tokio_tungstenite::{accept_async, tungstenite::Message};

use crate::simulator::config::EndpointDefinition;
use crate::simulator::template::{TemplateContext, TemplateEngine};

/// Handle a WebSocket upgrade and send templated messages
#[cfg(feature = "websockets")]
pub async fn handle_websocket_connection(
    req: Request<hyper::body::Incoming>,
    endpoint: &EndpointDefinition,
    engine: Arc<TemplateEngine>,
    context: TemplateContext,
) -> Response<Full<Bytes>> {
    let (parts, body) = req.into_parts();
    let req_head = Request::from_parts(parts.clone(), ());
    // Create handshake response
    let response = tokio_tungstenite::tungstenite::handshake::server::create_response(&req_head)
        .map(|resp| {
            let (parts_resp, _) = resp.into_parts();
            Response::from_parts(parts_resp, Full::new(Bytes::new()))
        })
        .unwrap_or_else(|_| {
            Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Full::new(Bytes::new()))
                .unwrap()
        });

    let upgrade = hyper::upgrade::on(Request::from_parts(parts, body));
    let endpoint_clone = endpoint.clone();
    tokio::spawn(async move {
        if let Ok(upgraded) = upgrade.await {
            if let Ok(mut ws) = accept_async(TokioIo::new(upgraded)).await {
                if let Some(cfg) = endpoint_clone.stream.as_ref() {
                    for tpl in &cfg.initial {
                        if let Ok(msg) = engine.render(tpl, &context) {
                            let _ = ws.send(Message::Text(msg)).await;
                        }
                    }
                    if let Some(periodic) = &cfg.periodic {
                        let mut ticker = interval(Duration::from_millis(periodic.interval_ms));
                        loop {
                            ticker.tick().await;
                            if let Ok(msg) = engine.render(&periodic.message, &context) {
                                if ws.send(Message::Text(msg)).await.is_err() {
                                    break;
                                }
                            }
                        }
                    }
                }
                let _ = ws.close(None).await;
            }
        }
    });

    response
}
