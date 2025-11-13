//! A peer-to-peer network layer for collaboration.
//!
//! This module provides a `spawn` function that creates a new libp2p node for
//! collaboration.
//!
//! This module is only available when the `p2p` feature flag is enabled.

use std::error::Error;

use libp2p::{
    futures::StreamExt,
    gossipsub::{self, IdentTopic, MessageAuthenticity, ValidationMode},
    identity, mdns,
    swarm::SwarmEvent,
    PeerId, SwarmBuilder,
};
use libp2p::swarm::NetworkBehaviour;
use tokio::sync::mpsc;

/// The combined network behaviour for collaboration, consisting of gossipsub
/// and mDNS.
#[derive(NetworkBehaviour)]
struct CollabBehaviour {
    gossipsub: gossipsub::Behaviour,
    mdns: mdns::tokio::Behaviour,
}

/// Spawns a libp2p node that publishes and receives raw CRDT operations.
///
/// # Returns
///
/// A `Result` containing a sender for local messages and a receiver for
/// messages coming from peers. Messages are plain byte vectors that higher
/// layers interpret as [`crate::collab::crdt::CrdtMessage`].
pub async fn spawn()
    -> Result<(mpsc::UnboundedSender<Vec<u8>>, mpsc::UnboundedReceiver<Vec<u8>>), Box<dyn Error + Send + Sync>>
{
    // Generate a random peer id based on an Ed25519 key pair.
    let local_key = identity::Keypair::generate_ed25519();
    let peer_id = PeerId::from(local_key.public());

    // Build gossipsub behaviour.
    let gossipsub_config =
        gossipsub::ConfigBuilder::default().validation_mode(ValidationMode::None).build()?;
    let gossipsub = gossipsub::Behaviour::new(
        MessageAuthenticity::Signed(local_key.clone()),
        gossipsub_config,
    )?;

    // mDNS for local peer discovery.
    let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), peer_id)?;

    let behaviour = CollabBehaviour { gossipsub, mdns };

    // Build swarm using Quic transport for simplicity.
    let mut swarm = SwarmBuilder::with_existing_identity(local_key)
        .with_tokio()
        .with_quic()
        .with_behaviour(|_| behaviour)?
        .build();

    // Subscribe to a global topic for service CRDT messages.
    let topic = IdentTopic::new("apicentric-service-crdt");
    swarm
        .behaviour_mut()
        .gossipsub
        .subscribe(&topic)
        .expect("subscribe to topic");

    // Channels used to communicate with the task.
    let (tx_publish, mut rx_publish) = mpsc::unbounded_channel::<Vec<u8>>();
    let (tx_events, rx_events) = mpsc::unbounded_channel::<Vec<u8>>();

    tokio::spawn(async move {
        loop {
            tokio::select! {
                Some(data) = rx_publish.recv() => {
                    let _ = swarm.behaviour_mut().gossipsub.publish(topic.clone(), data);
                }
                event = swarm.select_next_some() => {
                    match event {
                        SwarmEvent::Behaviour(CollabBehaviourEvent::Mdns(ev)) => {
                            match ev {
                                mdns::Event::Discovered(list) => {
                                    for (peer, _addr) in list {
                                        swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer);
                                    }
                                }
                                mdns::Event::Expired(list) => {
                                    for (peer, _addr) in list {
                                        swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer);
                                    }
                                }
                            }
                        }
                        SwarmEvent::Behaviour(CollabBehaviourEvent::Gossipsub(ev)) => {
                            if let gossipsub::Event::Message { message, .. } = ev {
                                let _ = tx_events.send(message.data);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    });

    Ok((tx_publish, rx_events))
}
