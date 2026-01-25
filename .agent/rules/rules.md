# Project Rules

## General Principles

- **Always fix warnings**: Maintain a warning-free codebase. Any new code or modifications should not introduce compiler warnings.
- **Low Dependency Tendency**: Prefer standard library or simple solutions over adding new external dependencies unless strictly necessary.
- **Hexagonal Architecture**:
  - Maintain clear separation between the **Domain** (core logic), **Ports** (interfaces), and **Adapters** (infrastructure/external integrations).
  - Domain should not depend on external frameworks.
  - Infrastructure/Adapters should depend on domain ports.

## Rust Specifics

- Follow idiomatic Rust patterns.
- Use `cargo fmt` and `cargo clippy` regularly.
- Ensure all tests pass before completing a task.
