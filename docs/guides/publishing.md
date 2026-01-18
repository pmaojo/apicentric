# Publishing Guide

This guide details how to publish new versions of Apicentric to the various package registries.

## Prerequisites

- **Version Bump**: Ensure you have bumped the version in:
  - `Cargo.toml`
  - `npm/package.json`
  - `npm/install.js` (constant `PACKAGE_VERSION`)
- **Git Tag**: Create a new git tag (e.g., `v0.2.9`).

## 1. Crates.io (Rust)

Publish the core binary and library to the Rust community.

1.  **Login** (first time only):

    ```bash
    cargo login <your-api-token>
    ```

2.  **Publish**:
    ```bash
    cargo publish
    ```
    _Note: This will automatically package the source code and upload it to crates.io._

## 2. NPM (Node.js)

Publish the wrapper package to the npm registry.

1.  ** Navigate to the npm directory**:

    ```bash
    cd npm/
    ```

2.  **Login** (first time only):

    ```bash
    npm login
    ```

3.  **Publish**:
    ```bash
    npm publish --access public
    ```

## 3. Docker Hub

Publish the production container image.

1.  **Login**:

    ```bash
    docker login
    ```

2.  **Build and Push**:

    ```bash
    # Build cleanly
    docker build -f Dockerfile.production -t pmaojo/apicentric:latest -t pmaojo/apicentric:v0.2.8 .

    # Push tags
    docker push pmaojo/apicentric:latest
    docker push pmaojo/apicentric:v0.2.8
    ```

## 4. GitHub Releases (Binaries)

The NPM wrapper relies on pre-built binaries hosted on GitHub Releases.

1.  **Build Binaries**:
    You need to cross-compile for supported targets (Linux, macOS, Windows).
    _Recommended: Use the GitHub Actions Release workflow._

2.  **Upload Assets**:
    Attach the `.tar.gz` and `.zip` files to the GitHub Release `v0.2.9`.
    - Naming convention must match `npm/install.js`: `apicentric-{platform}-{arch}.{ext}`

## Release Checklist

- [ ] Run `./scripts/health_check.sh`
- [ ] Update version numbers
- [ ] Push to GitHub and wait for CI
- [ ] `cargo publish`
- [ ] Create GitHub Release (Upload binaries)
- [ ] `npm publish`
- [ ] Docker push
