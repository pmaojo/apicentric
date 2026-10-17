# Changelog

All notable changes to this project are documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and the project follows [Semantic Versioning](https://semver.org/).

## [1.0.0] - 2026-10-17

First stable release. Changes since v0.7.0.1:

### Added
- SOAP/XML support in the simulator: XML request bodies (`text/xml`,
  `application/xml`, `application/soap+xml`) are parsed into the same tree as
  JSON, so templates can read `{{request.body.Envelope.Body.<Op>.<Param>}}`.
  Namespace prefixes are stripped, attributes are exposed as `@name`.
- Amadeus REST and SOAP mock services (`services/amadeus.yaml`,
  `services/amadeus-soap.yaml`) in the marketplace, plus
  `Dockerfile.amadeus-mock`.
- MCP `update_service` tool: replaces a running service with an updated YAML
  definition and persists it to disk.
- Release asset `apicentric-simulator-linux-x64.tar.gz`: a Linux x64 binary
  built with only the `simulator`, `scripting` and `file-watch` features, for
  CI pipelines and other projects that only need the simulator.
- A `.sha256` file for every release archive, alongside `checksums.txt`.

### Fixed
- File watcher reload loop (`file-watch` feature): on Linux, reading a service
  YAML was reported as a modification, so each reload triggered the next one
  and services kept restarting and refusing connections. Read events are now
  ignored, and a burst of events from a single save causes one reload.
- The recording proxy reports `http://127.0.0.1:<port>` instead of the bind
  address `http://0.0.0.0:<port>`, which clients cannot connect to on Windows.
- MCP `get_service_logs` returns the real request logs from SQLite instead of
  placeholder data, takes an optional `limit` (default 50) and returns an error
  for unknown services.
- Security: the admin server compares tokens in constant time (timing attack
  fix), using `subtle::Choice` without unwrapping to a raw `u8`.
- Security: the cloud config handlers no longer leak file paths in error
  responses, and `validate_config` restores redacted secrets before validating.

### Security
- Dependency upgrades for RustSec advisories: `reqwest` 0.11 → 0.12 (drops
  the vulnerable rustls 0.21 / rustls-webpki 0.101 / h2 0.3 stack),
  `quick-xml` 0.36 → 0.42 (DoS fixes in the parser used for SOAP request
  bodies), `rumqttc` 0.24 → 0.25 without its unused TLS stack, plus patch
  updates of rustls, rustls-webpki, h2, thin-vec, webbrowser, anyhow,
  event-listener, memmap2, rand and spin. `cargo audit` reports no
  vulnerabilities; the advisories in `.cargo/audit.toml` affect code that
  apicentric does not compile or only runs at build time.

### Changed
- MCP integration tests cover list, status, logs, delete and update of
  services.
- Release workflow: binaries are built with `--locked`, the GitHub release is
  created only after every platform build succeeds, and assets are uploaded
  with `softprops/action-gh-release` instead of the archived
  `actions/upload-release-asset`.

[1.0.0]: https://github.com/pmaojo/apicentric/compare/v0.7.0.1...v1.0.0
