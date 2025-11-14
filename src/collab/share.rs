//! A service for sharing and synchronizing service definitions.
//!
//! This module provides a `share_service` function that can be used to share a
//! service over libp2p, and a `connect_service` function that can be used to
//! connect to a remote peer and proxy requests locally.
//!
//! This module is only available when the `p2p` feature flag is enabled.

use std::{collections::HashMap, error::Error, sync::Arc};

use bytes::Bytes;
use libp2p::futures::{AsyncReadExt, AsyncWriteExt, StreamExt};
use hyper::{Request, Response};
use hyper::body::Incoming;
use http_body_util::{Full, BodyExt};
use hyper_util::{client::legacy::{connect::HttpConnector, Client}, rt::TokioExecutor};
use libp2p::{
    identity,
    mdns,
    request_response::{self, Codec, OutboundRequestId, ProtocolSupport},
    swarm::SwarmEvent,
    PeerId, SwarmBuilder,
};
use libp2p::swarm::NetworkBehaviour;
use serde::{Deserialize, Serialize};
use tokio::{net::TcpListener, sync::{mpsc, oneshot, RwLock}};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use std::convert::Infallible;

/// A message representing an HTTP request sent over libp2p.
#[derive(Debug, Serialize, Deserialize)]
struct HttpRequestMsg {
    token: String,
    method: String,
    path: String,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

/// A message representing an HTTP response sent over libp2p.
#[derive(Debug, Serialize, Deserialize)]
struct HttpResponseMsg {
    status: u16,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

const PROTOCOL: &str = "/apicentric/http/1.0.0";

#[derive(Clone, Default)]
struct HttpCodec;

#[async_trait::async_trait]
impl Codec for HttpCodec {
    type Protocol = &'static str;
    type Request = Bytes;
    type Response = Bytes;

    async fn read_request<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
    ) -> std::io::Result<Self::Request>
    where
        T: libp2p::futures::AsyncRead + Unpin + Send,
    {
        let mut buf = Vec::new();
        io.read_to_end(&mut buf).await?;
        Ok(Bytes::from(buf))
    }

    async fn read_response<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
    ) -> std::io::Result<Self::Response>
    where
        T: libp2p::futures::AsyncRead + Unpin + Send,
    {
        let mut buf = Vec::new();
        io.read_to_end(&mut buf).await?;
        Ok(Bytes::from(buf))
    }

    async fn write_request<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
        data: Self::Request,
    ) -> std::io::Result<()>
    where
        T: libp2p::futures::AsyncWrite + Unpin + Send,
    {
        io.write_all(&data).await?;
        io.close().await
    }

    async fn write_response<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
        data: Self::Response,
    ) -> std::io::Result<()>
    where
        T: libp2p::futures::AsyncWrite + Unpin + Send,
    {
        io.write_all(&data).await?;
        io.close().await
    }
}

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "ShareBehaviourEvent")]
struct ShareBehaviour {
    request_response: request_response::Behaviour<HttpCodec>,
    mdns: mdns::tokio::Behaviour,
}

#[derive(Debug)]
pub enum ShareBehaviourEvent {
    RequestResponse(request_response::Event<Bytes, Bytes>),
    Mdns(mdns::Event),
}

impl From<request_response::Event<Bytes, Bytes>> for ShareBehaviourEvent {
    fn from(event: request_response::Event<Bytes, Bytes>) -> Self {
        ShareBehaviourEvent::RequestResponse(event)
    }
}

impl From<mdns::Event> for ShareBehaviourEvent {
    fn from(event: mdns::Event) -> Self {
        ShareBehaviourEvent::Mdns(event)
    }
}

/// Starts hosting a service over libp2p.
///
/// # Arguments
///
/// * `port` - The port of the service to share.
///
/// # Returns
///
/// A `Result` containing the peer ID and auth token.
pub async fn share_service(port: u16) -> Result<(PeerId, String), Box<dyn Error>> {
    let local_key = identity::Keypair::generate_ed25519();
    let peer_id = PeerId::from(local_key.public());
    let token = uuid::Uuid::new_v4().to_string();

    let protocols = std::iter::once((PROTOCOL, ProtocolSupport::Full));
    let behaviour = ShareBehaviour {
        request_response: request_response::Behaviour::new(
            protocols,
            request_response::Config::default(),
        ),
        mdns: mdns::tokio::Behaviour::new(mdns::Config::default(), peer_id)?,
    };

    let mut swarm = SwarmBuilder::with_existing_identity(local_key)
        .with_tokio()
        .with_quic()
        .with_behaviour(|_| behaviour)?
        .build();

    swarm.listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse()?)?;

    let client: Client<HttpConnector, Full<Bytes>> =
        Client::builder(TokioExecutor::new()).build(HttpConnector::new());

    let token_clone = token.clone();

    tokio::spawn(async move {
        loop {
            match swarm.select_next_some().await {
                SwarmEvent::Behaviour(ShareBehaviourEvent::RequestResponse(ev)) => match ev {
                    request_response::Event::Message { peer: _, message, .. } => {
                        if let request_response::Message::Request { request, channel, .. } = message {
                            if let Ok(req_msg) = serde_json::from_slice::<HttpRequestMsg>(&request) {
                                if req_msg.token != token_clone {
                                    let _ = swarm
                                        .behaviour_mut()
                                        .request_response
                                        .send_response(channel, Bytes::new());
                                    continue;
                                }
                                let method = req_msg
                                    .method
                                    .parse()
                                    .unwrap_or(hyper::Method::GET);
                                let mut builder =
                                    Request::builder().method(method).uri(format!(
                                        "http://127.0.0.1:{}{}",
                                        port, req_msg.path
                                    ));
                                for (k, v) in req_msg.headers {
                                    builder = builder.header(&k, v);
                                }
                                let req = builder
                                    .body(Full::from(req_msg.body))
                                    .expect("valid request");
                                let resp_msg = match client.request(req).await {
                                    Ok(resp) => {
                                        let status = resp.status().as_u16();
                                        let headers = resp
                                            .headers()
                                            .iter()
                                            .map(|(k, v)| {
                                                (
                                                    k.to_string(),
                                                    v.to_str().unwrap_or_default().to_string(),
                                                )
                                            })
                                            .collect();
                                        let body = resp
                                            .into_body()
                                            .collect()
                                            .await
                                            .unwrap_or_default()
                                            .to_bytes()
                                            .to_vec();
                                        HttpResponseMsg { status, headers, body }
                                    }
                                    Err(_) => HttpResponseMsg {
                                        status: 500,
                                        headers: vec![],
                                        body: vec![],
                                    },
                                };
                                if let Ok(data) = serde_json::to_vec(&resp_msg) {
                                    let _ = swarm
                                        .behaviour_mut()
                                        .request_response
                                        .send_response(channel, Bytes::from(data));
                                }
                            }
                        }
                    }
                    _ => {}
                },
                SwarmEvent::Behaviour(ShareBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                    for (peer, addr) in list {
                        swarm.add_peer_address(peer, addr);
                    }
                }
                SwarmEvent::Behaviour(ShareBehaviourEvent::Mdns(mdns::Event::Expired(_))) => {}
                _ => {}
            }
        }
    });

    Ok((peer_id, token))
}

/// Connects to a remote peer and proxies requests locally.
///
/// # Arguments
///
/// * `peer_id` - The ID of the peer to connect to.
/// * `token` - The authentication token for the peer.
/// * `service` - The name of the service to connect to.
/// * `port` - The local port to listen on.
pub async fn connect_service(
    peer_id: PeerId,
    token: String,
    service: String,
    port: u16,
) -> Result<(), Box<dyn Error>> {
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer = PeerId::from(local_key.public());

    let protocols = std::iter::once((PROTOCOL, ProtocolSupport::Full));
    let behaviour = ShareBehaviour {
        request_response: request_response::Behaviour::new(
            protocols,
            request_response::Config::default(),
        ),
        mdns: mdns::tokio::Behaviour::new(mdns::Config::default(), local_peer)?,
    };

    let (tx_req, mut rx_req) = mpsc::unbounded_channel::<(Bytes, oneshot::Sender<Bytes>)>();
    let pending: Arc<RwLock<HashMap<OutboundRequestId, oneshot::Sender<Bytes>>>> =
        Arc::new(RwLock::new(HashMap::new()));
    let pending_swarm = pending.clone();

    let mut swarm = SwarmBuilder::with_existing_identity(local_key)
        .with_tokio()
        .with_quic()
        .with_behaviour(|_| behaviour)?
        .build();

    swarm.listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse()?)?;

    let peer_clone = peer_id.clone();

    tokio::spawn(async move {
        loop {
            tokio::select! {
                Some((data, sender)) = rx_req.recv() => {
                    let id = swarm.behaviour_mut().request_response.send_request(&peer_clone, data);
                    pending_swarm.write().await.insert(id, sender);
                }
                event = swarm.select_next_some() => {
                    match event {
                        SwarmEvent::Behaviour(ShareBehaviourEvent::RequestResponse(ev)) => match ev {
                            request_response::Event::Message { message, .. } => {
                                if let request_response::Message::Response { request_id, response, .. } = message {
                                    if let Some(tx) = pending_swarm.write().await.remove(&request_id) {
                                        let _ = tx.send(response);
                                    }
                                }
                            }
                            request_response::Event::OutboundFailure { request_id, .. } => {
                                if let Some(tx) = pending_swarm.write().await.remove(&request_id) {
                                    let _ = tx.send(Bytes::new());
                                }
                            }
                            _ => {}
                        },
                        SwarmEvent::Behaviour(ShareBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                            for (peer, addr) in list {
                                swarm.add_peer_address(peer, addr);
                            }
                        }
                        SwarmEvent::Behaviour(ShareBehaviourEvent::Mdns(mdns::Event::Expired(_))) => {}
                        _ => {}
                    }
                }
            }
        }
    });

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr).await?;
    let tx_http = tx_req.clone();

    tokio::spawn(async move {
        loop {
            let (stream, _) = listener.accept().await.expect("accept");
            let tx = tx_http.clone();
            let token = token.clone();
            tokio::spawn(async move {
                let io = TokioIo::new(stream);
                if let Err(e) = http1::Builder::new()
                    .serve_connection(
                        io,
                        service_fn(move |req: Request<Incoming>| {
                            let tx = tx.clone();
                            let token = token.clone();
                            async move {
                                let (parts, body_stream) = req.into_parts();
                                let body = body_stream
                                    .collect()
                                    .await
                                    .unwrap_or_default()
                                    .to_bytes()
                                    .to_vec();
                                let headers = parts
                                    .headers
                                    .iter()
                                    .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or_default().to_string()))
                                    .collect();
                                let msg = HttpRequestMsg {
                                    token: token.clone(),
                                    method: parts.method.to_string(),
                                    path: parts.uri.to_string(),
                                    headers,
                                    body,
                                };
                                let data = Bytes::from(serde_json::to_vec(&msg).expect("serialize"));
                                let (resp_tx, resp_rx) = oneshot::channel();
                                tx.send((data, resp_tx)).expect("send req");
                                if let Ok(resp_data) = resp_rx.await {
                                    if let Ok(resp_msg) = serde_json::from_slice::<HttpResponseMsg>(&resp_data) {
                                        let mut builder = Response::builder().status(resp_msg.status);
                                        for (k, v) in resp_msg.headers {
                                            builder = builder.header(&k, v);
                                        }
                                        let body = Full::<Bytes>::from(Bytes::from(resp_msg.body));
                                        let resp = builder.body(body).unwrap();
                                        return Ok::<_, Infallible>(resp);
                                    }
                                }
                                let resp =
                                    Response::builder().status(500).body(Full::from(Vec::new())).unwrap();
                                Ok::<_, Infallible>(resp)
                            }
                        }),
                    )
                    .await
                {
                    eprintln!("proxy error: {}", e);
                }
            });
        }
    });

    println!(
        "Proxy for service '{}' listening on http://localhost:{} (peer: {})",
        service, port, peer_id
    );

    tokio::signal::ctrl_c().await?;

    Ok(())
}
