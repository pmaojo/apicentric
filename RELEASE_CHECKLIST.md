# Apicentric Release Checklist

This document provides a comprehensive checklist for preparing and executing releases of Apicentric.

## Pre-Release Verification

### Code Quality

- [ ] All CI checks pass on main branch
  - [ ] Format check (`cargo fmt --check`)
  - [ ] Lint check (`cargo clippy -- -D warnings`)
  - [ ] Tests pass on Linux, macOS, and Windows
  - [ ] Security audit passes (`cargo audit`)
  - [ ] Code coverage meets threshold (>70%)

- [ ] All feature combinations build successfully
  - [ ] Minimal build: `cargo build --no-default-features --features minimal`
  - [ ] Default build: `cargo build`
  - [ ] CLI tools build: `cargo build --features cli-tools`
  - [ ] Full build: `cargo build --all-features`

- [ ] No compiler warnings
  - [ ] Run `cargo build --all-features` and verify zero warnings
  - [ ] Run `cargo clippy --all-features` and verify zero warnings

### Documentation

- [ ] README.md is up to date
  - [ ] Version numbers are current
  - [ ] Installation instructions are accurate
  - [ ] Examples work correctly
  - [ ] Links are valid

- [ ] CHANGELOG.md is updated
  - [ ] All changes since last release are documented
  - [ ] Breaking changes are clearly marked
  - [ ] Migration guide provided for breaking changes
  - [ ] Contributors are credited

- [ ] Documentation guides are current
  - [ ] Quick start guide works end-to-end
  - [ ] TUI guide reflects current features
  - [ ] Feature flags guide is accurate
  - [ ] Architecture document is up to date

- [ ] API documentation is complete
  - [ ] Run `cargo doc --all-features --no-deps`
  - [ ] Verify all public APIs are documented
  - [ ] Examples in doc comments work

### Testing

- [ ] Manual testing completed
  - [ ] Install from source works
  - [ ] Basic simulator functionality works
  - [ ] TUI launches and functions correctly
  - [ ] Contract testing works
  - [ ] Code generation works

- [ ] Platform-specific testing
  - [ ] Test on Linux (Ubuntu 22.04 or later)
  - [ ] Test on macOS (Intel and ARM)
  - [ ] Test on Windows 11

- [ ] Installation methods tested
  - [ ] Cargo install works
  - [ ] Install script works (Unix)
  - [ ] Install script works (Windows PowerShell)
  - [ ] Pre-built binaries work

### Version Management

- [ ] Version numbers updated
  - [ ] Cargo.toml version matches release version
  - [ ] README.md version badges updated
  - [ ] CHANGELOG.md has new version section

- [ ] Git repository is clean
  - [ ] All changes committed
  - [ ] Working directory is clean
  - [ ] On main branch
  - [ ] Synced with remote

## Release Execution

### Create Release Tag

- [ ] Create annotated git tag
  ```bash
  git tag -a v0.X.Y -m "Release v0.X.Y"
  ```

- [ ] Push tag to trigger release workflow
  ```bash
  git push origin v0.X.Y
  ```

### Monitor Release Workflow

- [ ] GitHub Actions release workflow starts
- [ ] All platform builds complete successfully
  - [ ] Linux x64 binary created
  - [ ] macOS x64 binary created
  - [ ] macOS ARM64 binary created
  - [ ] Windows x64 binary created

- [ ] Checksums generated correctly
- [ ] GitHub release created with all assets
- [ ] Release notes generated

### Verify Release Assets

- [ ] Download and verify each binary
  - [ ] Linux x64: `apicentric-linux-x64.tar.gz`
  - [ ] macOS x64: `apicentric-macos-x64.tar.gz`
  - [ ] macOS ARM64: `apicentric-macos-arm64.tar.gz`
  - [ ] Windows x64: `apicentric-windows-x64.zip`

- [ ] Verify checksums match
  ```bash
  sha256sum -c checksums.txt
  ```

- [ ] Test each binary runs
  ```bash
  ./apicentric --version
  ./apicentric --help
  ```

### Update Distribution Channels

- [ ] Publish to crates.io
  ```bash
  cargo publish --all-features
  ```

- [ ] Update Homebrew formula
  - [ ] Update version in `Formula/apicentric.rb`
  - [ ] Update SHA256 checksums for each platform
  - [ ] Test formula installation
    ```bash
    brew install --build-from-source Formula/apicentric.rb
    brew test apicentric
    ```
  - [ ] Commit and push to homebrew-tap repository

- [ ] Update installation scripts
  - [ ] Verify `scripts/install.sh` downloads correct version
  - [ ] Verify `scripts/install.ps1` downloads correct version
  - [ ] Test scripts on clean systems

## Post-Release Activities

### Announcements

- [ ] Update GitHub release notes
  - [ ] Add highlights and key features
  - [ ] Include upgrade instructions
  - [ ] Link to CHANGELOG
  - [ ] Thank contributors

- [ ] Announce on social media
  - [ ] Twitter/X post with key features
  - [ ] LinkedIn post (if applicable)
  - [ ] Mastodon post (if applicable)

- [ ] Announce to Rust community
  - [ ] Post to r/rust on Reddit
  - [ ] Post to This Week in Rust (if significant release)
  - [ ] Post to Rust Users forum (if major release)

- [ ] Announce on GitHub Discussions
  - [ ] Create announcement post
  - [ ] Highlight new features
  - [ ] Invite feedback

### Monitoring

- [ ] Monitor for issues
  - [ ] Watch GitHub issues for bug reports
  - [ ] Monitor discussions for questions
  - [ ] Check CI status on main branch

- [ ] Verify metrics
  - [ ] Check crates.io download stats
  - [ ] Monitor GitHub stars/forks
  - [ ] Review installation analytics (if available)

### Documentation Updates

- [ ] Update website (if applicable)
- [ ] Update any external documentation
- [ ] Update comparison tables or benchmarks

## Rollback Procedure

If critical issues are discovered after release:

1. **Immediate Actions**
   - [ ] Create GitHub issue documenting the problem
   - [ ] Add warning to release notes
   - [ ] Pin issue to repository

2. **Yank Release (if necessary)**
   - [ ] Yank from crates.io: `cargo yank --vers 0.X.Y`
   - [ ] Mark GitHub release as pre-release
   - [ ] Update Homebrew formula to previous version

3. **Fix and Re-release**
   - [ ] Create hotfix branch
   - [ ] Fix the issue
   - [ ] Follow release checklist for patch release
   - [ ] Communicate fix to users

## Release Templates

### Git Tag Message Template

```
Release v0.X.Y

## Highlights
- Feature 1
- Feature 2
- Bug fix 1

## Breaking Changes
- Change 1 (migration: ...)

See CHANGELOG.md for full details.
```

### GitHub Release Notes Template

```markdown
# Release v0.X.Y

## 🎉 Highlights

- **Feature 1**: Description
- **Feature 2**: Description
- **Improvement**: Description

## 🐛 Bug Fixes

- Fixed issue #123: Description
- Fixed issue #456: Description

## ⚠️ Breaking Changes

- **Change 1**: Description
  - **Migration**: How to update your code

## 📦 Installation

### Homebrew (macOS/Linux)
\`\`\`bash
brew install pmaojo/tap/apicentric
\`\`\`

### Cargo
\`\`\`bash
cargo install apicentric --features cli-tools
\`\`\`

### Pre-built Binaries
Download from the assets below for your platform.

## 📝 Full Changelog

See [CHANGELOG.md](CHANGELOG.md) for complete details.

## 🙏 Contributors

Thanks to all contributors who made this release possible!

- @contributor1
- @contributor2
```

### Social Media Post Template

```
🚀 Apicentric v0.X.Y is out!

New features:
✨ Feature 1
✨ Feature 2
🐛 Bug fixes and improvements

Install: brew install pmaojo/tap/apicentric

Full release notes: [link]

#rust #cli #api #testing #opensource
```

### Reddit Post Template (r/rust)

```markdown
# Apicentric v0.X.Y Released - CLI Tool and API Simulator Platform

Hi r/rust! I'm excited to announce the release of Apicentric v0.X.Y.

## What is Apicentric?

Apicentric is a Rust-based CLI tool and API simulator platform that helps developers:
- Mock APIs with simple YAML configuration
- Test API contracts between services
- Generate code (TypeScript types, React Query hooks)
- Manage services with an interactive TUI

## What's New in v0.X.Y

- **Feature 1**: Description
- **Feature 2**: Description
- **Improvement**: Description

## Installation

```bash
# Homebrew
brew install pmaojo/tap/apicentric

# Cargo
cargo install apicentric --features cli-tools
```

## Links

- GitHub: https://github.com/pmaojo/apicentric
- Documentation: [link]
- Release Notes: [link]

Feedback and contributions welcome!
```

## Notes

- This checklist should be reviewed and updated after each release
- Adjust timelines based on release size (major/minor/patch)
- Consider automating more steps in future releases
- Keep a release retrospective document to improve the process

## Version History

- v0.1.0: Initial release checklist created
