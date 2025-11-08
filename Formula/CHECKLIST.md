# Homebrew Tap Setup Checklist

Use this checklist when setting up or updating the `pmaojo/homebrew-tap` repository.

## Initial Setup

- [ ] Create GitHub repository `pmaojo/homebrew-tap`
- [ ] Set repository visibility to Public
- [ ] Add repository description: "Homebrew tap for Apicentric"
- [ ] Add topics: `homebrew`, `homebrew-tap`, `apicentric`, `cli`
- [ ] Clone the repository locally
- [ ] Copy formula files from main repository:
  - [ ] `Formula/apicentric.rb`
  - [ ] `Formula/README.md` → `README.md`
  - [ ] `Formula/SETUP.md` → `SETUP.md`
  - [ ] `Formula/update-checksums.sh` → `update-checksums.sh`
  - [ ] `Formula/CHECKLIST.md` → `CHECKLIST.md`
- [ ] Make `update-checksums.sh` executable
- [ ] Create `Formula/` directory in tap repository
- [ ] Move `apicentric.rb` to `Formula/apicentric.rb`
- [ ] Commit and push initial setup

## For Each New Release

### Pre-Release

- [ ] Ensure release artifacts are built and uploaded to GitHub releases
- [ ] Verify artifacts are named correctly:
  - [ ] `apicentric-macos-arm64.tar.gz`
  - [ ] `apicentric-macos-x64.tar.gz`
  - [ ] `apicentric-linux-x64.tar.gz`
- [ ] Verify each tarball contains the `apicentric` binary

### Update Formula

- [ ] Run `./update-checksums.sh <version>` (e.g., `./update-checksums.sh 0.1.1`)
- [ ] Review the changes: `git diff Formula/apicentric.rb`
- [ ] Verify version number is correct
- [ ] Verify all three SHA256 checksums are updated

### Testing

- [ ] Test formula syntax: `brew audit --strict Formula/apicentric.rb`
- [ ] Clear Homebrew cache: `rm -rf $(brew --cache)`
- [ ] Test installation: `brew install --build-from-source Formula/apicentric.rb`
- [ ] Run formula tests: `brew test apicentric`
- [ ] Verify binary works: `apicentric --version`
- [ ] Check version output matches expected version
- [ ] Test basic functionality: `apicentric --help`
- [ ] Uninstall: `brew uninstall apicentric`

### Linux Testing (if available)

- [ ] Test on Linux system or container
- [ ] Install: `brew install --build-from-source Formula/apicentric.rb`
- [ ] Run tests: `brew test apicentric`
- [ ] Verify: `apicentric --version`
- [ ] Uninstall: `brew uninstall apicentric`

### Commit and Release

- [ ] Commit changes: `git add Formula/apicentric.rb`
- [ ] Create descriptive commit message: `git commit -m "Update apicentric to v<version>"`
- [ ] Push to main: `git push origin main`
- [ ] Create GitHub release tag (optional): `git tag v<version> && git push origin v<version>`

### Post-Release Verification

- [ ] Wait a few minutes for tap to sync
- [ ] Test from tap: `brew tap pmaojo/tap`
- [ ] Install from tap: `brew install apicentric`
- [ ] Verify installation: `apicentric --version`
- [ ] Test upgrade path: `brew upgrade apicentric`
- [ ] Uninstall: `brew uninstall apicentric`

## Troubleshooting

### Common Issues

**Checksum mismatch:**
- [ ] Re-download artifacts
- [ ] Recalculate checksums
- [ ] Clear Homebrew cache
- [ ] Update formula

**Binary not found:**
- [ ] Check tarball contents: `tar -tzf apicentric-*.tar.gz`
- [ ] Ensure binary is at root of tarball
- [ ] Verify binary name is exactly `apicentric`

**Test failures:**
- [ ] Check binary is executable
- [ ] Verify `--version` flag exists
- [ ] Test binary manually
- [ ] Review test block in formula

**Installation fails:**
- [ ] Check release URLs are accessible
- [ ] Verify GitHub release is published (not draft)
- [ ] Check network connectivity
- [ ] Review Homebrew logs: `brew install --verbose apicentric`

## Maintenance

### Regular Tasks

- [ ] Monitor GitHub issues for installation problems
- [ ] Keep formula in sync with releases
- [ ] Update documentation as needed
- [ ] Test on new macOS/Linux versions
- [ ] Review and update dependencies

### Quarterly Review

- [ ] Review Homebrew best practices for changes
- [ ] Check for deprecated Homebrew APIs
- [ ] Update CI/CD workflows if needed
- [ ] Review and update documentation
- [ ] Test on latest macOS and Linux versions

## Resources

- [Homebrew Documentation](https://docs.brew.sh/)
- [Formula Cookbook](https://docs.brew.sh/Formula-Cookbook)
- [Tap Documentation](https://docs.brew.sh/How-to-Create-and-Maintain-a-Tap)
- [Apicentric Releases](https://github.com/pmaojo/apicentric/releases)
