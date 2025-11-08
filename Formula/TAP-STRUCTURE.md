# Homebrew Tap Repository Structure

This document describes the structure of the `pmaojo/homebrew-tap` repository.

## Repository Layout

```
pmaojo/homebrew-tap/
├── Formula/
│   └── apicentric.rb          # Main Homebrew formula
├── README.md                   # User-facing documentation
├── SETUP.md                    # Setup guide for maintainers
├── CHECKLIST.md                # Release checklist
├── TAP-STRUCTURE.md            # This file
├── update-checksums.sh         # Script to update formula checksums
└── .github/
    └── workflows/
        └── test.yml            # Optional CI workflow
```

## File Descriptions

### Formula/apicentric.rb

The main Homebrew formula that defines how to install Apicentric. Contains:

- Package metadata (description, homepage, version, license)
- Platform-specific download URLs and checksums
- Installation instructions
- Test block to verify installation

**Key sections:**
- `on_macos`: macOS-specific installation (ARM64 and x64)
- `on_linux`: Linux-specific installation (x64)
- `install`: Installation logic (copies binary to bin directory)
- `test`: Verification that installation succeeded

### README.md

User-facing documentation that explains:
- How to install Apicentric via Homebrew
- Supported platforms
- How to update and uninstall
- Basic usage examples

**Target audience:** End users who want to install Apicentric

### SETUP.md

Maintainer guide that explains:
- How to set up the tap repository initially
- How to update the formula for new releases
- Testing procedures
- CI/CD integration options

**Target audience:** Repository maintainers

### CHECKLIST.md

Step-by-step checklist for:
- Initial repository setup
- Releasing new versions
- Testing procedures
- Troubleshooting common issues

**Target audience:** Maintainers performing releases

### TAP-STRUCTURE.md

This file - documents the repository structure and file purposes.

**Target audience:** New maintainers and contributors

### update-checksums.sh

Automated script that:
- Downloads release artifacts for a given version
- Calculates SHA256 checksums
- Updates the formula with new version and checksums

**Usage:** `./update-checksums.sh 0.1.1`

**Target audience:** Maintainers updating the formula

## Homebrew Tap Conventions

### Naming

- Repository must be named `homebrew-<tap-name>`
- For this tap: `homebrew-tap`
- Users reference it as: `pmaojo/tap`

### Formula Location

- Formulas must be in the `Formula/` directory
- Formula filename must match the package name: `apicentric.rb`
- Class name must match: `class Apicentric < Formula`

### URLs

- Tap URL: `https://github.com/pmaojo/homebrew-tap`
- Formula URL: `https://github.com/pmaojo/homebrew-tap/blob/main/Formula/apicentric.rb`
- User installs with: `brew install pmaojo/tap/apicentric`

## Workflow

### For Users

```bash
# Add tap
brew tap pmaojo/tap

# Install
brew install apicentric

# Update
brew update && brew upgrade apicentric

# Uninstall
brew uninstall apicentric
```

### For Maintainers

```bash
# Update formula for new release
./update-checksums.sh 0.1.1

# Test locally
brew audit --strict Formula/apicentric.rb
brew install --build-from-source Formula/apicentric.rb
brew test apicentric

# Commit and push
git add Formula/apicentric.rb
git commit -m "Update apicentric to v0.1.1"
git push origin main
```

## Platform Support

### macOS

- **Intel (x64)**: `apicentric-macos-x64.tar.gz`
- **Apple Silicon (ARM64)**: `apicentric-macos-arm64.tar.gz`

The formula automatically detects the architecture and downloads the correct binary.

### Linux

- **x64**: `apicentric-linux-x64.tar.gz`

Homebrew on Linux (Linuxbrew) is supported.

### Windows

Windows is not supported via Homebrew. Users should use:
- PowerShell install script
- Pre-built binaries from GitHub releases
- Cargo install

## Version Management

### Version Format

- Use semantic versioning: `MAJOR.MINOR.PATCH`
- Example: `0.1.1`, `1.0.0`, `1.2.3`

### Version Updates

1. Update `version` field in formula
2. Update download URLs (automatically uses `#{version}`)
3. Update SHA256 checksums for all platforms
4. Test installation
5. Commit and push

### Version Interpolation

The formula uses Ruby string interpolation:

```ruby
version "0.1.1"
url "https://github.com/pmaojo/apicentric/releases/download/v#{version}/apicentric-macos-arm64.tar.gz"
```

This automatically constructs the correct URL based on the version.

## Security

### Checksums

- All downloads must have SHA256 checksums
- Never use `sha256 :no_check` in production
- Checksums verify download integrity
- Mismatched checksums prevent installation

### HTTPS

- All URLs must use HTTPS
- GitHub releases provide HTTPS by default

### Binary Verification

The test block verifies:
- Binary is executable
- Binary responds to `--version`
- Output contains "apicentric"

## Best Practices

1. **Always test locally** before pushing
2. **Use the update script** to avoid manual errors
3. **Test on multiple platforms** when possible
4. **Keep documentation updated** with each release
5. **Follow semantic versioning** strictly
6. **Verify checksums** match release artifacts
7. **Test the upgrade path** from previous versions

## Troubleshooting

### Formula doesn't install

- Check release artifacts exist at URLs
- Verify checksums match
- Clear Homebrew cache: `rm -rf $(brew --cache)`
- Check logs: `brew install --verbose apicentric`

### Test fails

- Verify binary is executable
- Check `--version` flag works
- Test binary manually
- Review test block logic

### Checksum mismatch

- Re-download artifacts
- Recalculate checksums
- Update formula
- Clear cache and retry

## Resources

- [Homebrew Documentation](https://docs.brew.sh/)
- [Formula Cookbook](https://docs.brew.sh/Formula-Cookbook)
- [Acceptable Formulae](https://docs.brew.sh/Acceptable-Formulae)
- [Tap Documentation](https://docs.brew.sh/How-to-Create-and-Maintain-a-Tap)
- [Apicentric Repository](https://github.com/pmaojo/apicentric)
