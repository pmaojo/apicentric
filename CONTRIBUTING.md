# Contributing to Apicentric

Welcome to the team! We follow a strict "CTO-approved" workflow to ensure high quality and velocity.

## Quick Start for Developers

1.  **Install Prerequisites**:
    - Rust (stable)
    - Docker (optional, for container tests)

2.  **Run the Health Check**:
    Before submitting _any_ code, run the automated health check:
    ```bash
    ./scripts/health_check.sh
    ```
    This script runs:
    - `cargo fmt`: Ensures code style consistency.
    - `cargo clippy`: Catches common mistakes and improvements.
    - `cargo test`: Runs the test suite.
    - `cargo build`: Verifies compilation.

## Code Style

- We use standard Rust formatting (`rustfmt`).
- We use `clippy` for linting. We aim for zero warnings, but the CI is currently configured to allow non-critical warnings to keep velocity high.
- **No "unwrap" in production code**: Handle errors gracefully using `Result` and our `ApicentricError` types.

## Pull Request Process

1.  Create a feature branch.
2.  Make your changes.
3.  Run `./scripts/health_check.sh`. **If it fails, fix it.**
4.  Open a PR.
5.  Wait for CI to pass.

## Architecture Guidelines

- **Hexagonal Architecture**: Keep domain logic pure and separate from adapters (CLI, HTTP, FileSystem).
- **Errors**: Use the `ApicentricError` enum in `src/errors.rs`. Do not introduce ad-hoc string errors.
- **DX First**: Always consider the Developer Experience. If a command fails, tell the user _why_ and _how to fix it_.
