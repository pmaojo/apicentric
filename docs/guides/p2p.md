# P2P Collaboration Guide

The P2P feature in Apicentric allows developers to collaborate on service definitions and share running services securely over a peer-to-peer network.

## Feature Availability

**Note on Availability**: The P2P functionality is **not enabled by default** in the `apicentric` crate published on crates.io to keep the base installation lightweight.

- **Pre-built Binaries**: The official release binaries (Linux, macOS, Windows) available on GitHub **include** the P2P feature (built with `--features full`).
- **Crates.io**: When installing via `cargo install apicentric`, you must explicitly enable the feature: `cargo install apicentric --features p2p` (or `--features full`).

## Usage

### Starting the Simulator with P2P

To enable collaboration on service definitions, start the simulator with the `--p2p` flag:

```bash
apicentric simulator start --services-dir ./services --p2p
```

This will join the default discovery swarm and look for peers on the local network (mDNS) and configured bootstrap nodes.

### Sharing a Running Service

You can securely share a local service with a remote peer (tunneling):

```bash
apicentric simulator share --service my-service-name
```

This command will output:
- A **Peer ID**
- An **Authentication Token**

### Connecting to a Shared Service

A remote developer can connect to your shared service using the Peer ID and Token:

```bash
apicentric simulator connect \
  --peer <PEER_ID> \
  --service my-service-name \
  --port 9005 \
  --token <TOKEN>
```

The service will now be available locally at `http://localhost:9005`, proxying requests securely to the host machine.

## Troubleshooting

- **Firewalls**: Ensure UDP traffic is allowed for QUIC (libp2p default transport).
- **Network**: mDNS discovery works only on the local network. For internet-wide discovery, you may need to configure bootstrap nodes (advanced configuration).
