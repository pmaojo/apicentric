use bytes::Bytes;
use http_body_util::StreamBody;
use hyper::{Response, StatusCode};
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::time::{interval, Duration};
use tokio_stream::wrappers::UnboundedReceiverStream;

use crate::simulator::config::EndpointDefinition;
use crate::simulator::template::{TemplateContext, TemplateEngine};

/// Create a Server-Sent Events response with optional periodic messages
pub fn handle_sse_connection(
    endpoint: &EndpointDefinition,
    engine: Arc<TemplateEngine>,
    context: TemplateContext,
) -> Response<StreamBody<UnboundedReceiverStream<Result<Bytes, Infallible>>>> {
    let (tx, rx) = mpsc::unbounded_channel();

    if let Some(cfg) = endpoint.stream.as_ref() {
        for tpl in &cfg.initial {
            if let Ok(msg) = engine.render(tpl, &context) {
                let _ = tx.send(Ok(Bytes::from(format!("data: {}\n\n", msg))));
            }
        }

        if let Some(periodic) = &cfg.periodic {
            let mut ticker = interval(Duration::from_millis(periodic.interval_ms));
            let tx_clone = tx.clone();
            let msg_tpl = periodic.message.clone();
            let engine_clone = engine.clone();
            let ctx_clone = context.clone();
            tokio::spawn(async move {
                loop {
                    ticker.tick().await;
                    if let Ok(msg) = engine_clone.render(&msg_tpl, &ctx_clone) {
                        if tx_clone
                            .send(Ok(Bytes::from(format!("data: {}\n\n", msg))))
                            .is_err()
                        {
                            break;
                        }
                    }
                }
            });
        }
    }

    let stream = UnboundedReceiverStream::new(rx);
    let body = StreamBody::new(stream);
    Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "text/event-stream")
        .body(body)
        .unwrap()
}
