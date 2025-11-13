# Contributing to Apicentric

Thank you for your interest in contributing to Apicentric! This document provides guidelines and instructions for contributing to the project.

## Getting Started

### Prerequisites

- Rust 1.70 or higher
- Git
- A GitHub account

### Setting Up Your Development Environment

1. Fork the repository on GitHub
2. Clone your fork locally:

```bash
git clone https://github.com/YOUR_USERNAME/apicentric.git
cd apicentric
```

3. Add the upstream repository:

```bash
git remote add upstream https://github.com/pmaojo/apicentric.git
```

4. Build the project:

```bash
cargo build
```

5. Run tests to ensure everything works:

```bash
cargo test
```

## Development Workflow

### Creating a Branch

Create a new branch for your work:

```bash
git checkout -b feature/your-feature-name
```

Use descriptive branch names:
- `feature/` for new features
- `fix/` for bug fixes
- `docs/` for documentation changes
- `refactor/` for code refactoring

### Making Changes

1. Make your changes in your branch
2. Write or update tests as needed
3. Ensure all tests pass: `cargo test`
4. Format your code: `cargo fmt`
5. Run clippy: `cargo clippy -- -D warnings`

### Commit Messages

Follow these conventions for commit messages:

- Use the present tense ("Add feature" not "Added feature")
- Use the imperative mood ("Move cursor to..." not "Moves cursor to...")
- Limit the first line to 72 characters or less
- Reference issues and pull requests liberally after the first line

Examples:

```
Add TUI filtering functionality

Implement log filtering in the TUI with support for method,
status code, and service name filters.

Fixes #123
```

### Submitting a Pull Request

1. Push your changes to your fork:

```bash
git push origin feature/your-feature-name
```

2. Go to the GitHub repository and create a Pull Request
3. Fill out the PR template with:
   - Description of changes
   - Motivation and context
   - Testing performed
   - Related issues

4. Wait for review and address any feedback

## Coding Standards

### Rust Style Guide

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` to format code
- Use `cargo clippy` to catch common mistakes
- Write idiomatic Rust code

### Code Organization

- Keep functions small and focused
- Use meaningful variable and function names
- Add comments for complex logic
- Document public APIs with doc comments

### Error Handling

- Use the `ApicentricError` type for errors
- Provide helpful error messages with suggestions
- Use `?` operator for error propagation
- Avoid unwrap() in production code

### Testing

- Write unit tests for new functionality
- Add integration tests for major features
- Ensure tests are deterministic
- Use descriptive test names

Example:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_status_parsing() {
        // Test implementation
    }
}
```

## Testing Guidelines

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture

# Run tests for specific feature
cargo test --features tui
```

### Writing Tests

- Test one thing per test
- Use descriptive assertions
- Clean up resources in tests
- Mock external dependencies

## Documentation

### Code Documentation

- Add doc comments to public APIs
- Include examples in doc comments
- Document panics and errors
- Keep documentation up to date

Example:

```rust
/// Starts the API simulator with the given configuration.
///
/// # Arguments
///
/// * `config` - The simulator configuration
///
/// # Returns
///
/// Returns `Ok(())` on success, or an error if the simulator fails to start.
///
/// # Examples
///
/// ```
/// let config = SimulatorConfig::default();
/// start_simulator(config)?;
/// ```
pub fn start_simulator(config: SimulatorConfig) -> ApicentricResult<()> {
    // Implementation
}
```

### User Documentation

- Update README.md for user-facing changes
- Add examples for new features
- Update guides in `docs/guides/`
- Keep documentation clear and concise

## Review Process

### What to Expect

1. A maintainer will review your PR within a few days
2. You may be asked to make changes
3. Once approved, a maintainer will merge your PR
4. Your contribution will be included in the next release

### Review Criteria

- Code quality and style
- Test coverage
- Documentation completeness
- Backward compatibility
- Performance impact

## Good First Issues

Looking for a place to start? Check out issues labeled:

- `good first issue` - Simple issues for newcomers
- `help wanted` - Issues where we need help
- `documentation` - Documentation improvements

## Feature Requests

Have an idea for a new feature?

1. Check if it's already been requested
2. Open a new issue with the `feature request` label
3. Describe the feature and its use case
4. Discuss with maintainers before implementing

## Bug Reports

Found a bug?

1. Check if it's already been reported
2. Open a new issue with the `bug` label
3. Include:
   - Steps to reproduce
   - Expected behavior
   - Actual behavior
   - System information
   - Error messages

## Community Guidelines

- Be respectful and inclusive
- Follow the [Code of Conduct](CODE_OF_CONDUCT.md)
- Help others in discussions
- Share knowledge and experience

## Questions?

- Open a [Discussion](https://github.com/pmaojo/apicentric/discussions)
- Ask in an issue
- Check existing documentation

Thank you for contributing to Apicentric!
