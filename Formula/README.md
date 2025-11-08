# Apicentric Homebrew Tap

This is the official Homebrew tap for [Apicentric](https://github.com/pmaojo/apicentric), a CLI tool and API simulator platform for developers.

## Installation

### Using Homebrew

```bash
# Add the tap
brew tap pmaojo/tap

# Install apicentric
brew install apicentric
```

Or install directly:

```bash
brew install pmaojo/tap/apicentric
```

### Verify Installation

```bash
apicentric --version
```

## Supported Platforms

- macOS (Intel x64)
- macOS (Apple Silicon ARM64)
- Linux (x64)

## Updating

```bash
brew update
brew upgrade apicentric
```

## Uninstalling

```bash
brew uninstall apicentric
brew untap pmaojo/tap
```

## Formula Maintenance

### Updating the Formula

When a new version is released:

1. Update the `version` field in `Formula/apicentric.rb`
2. Download the release artifacts and calculate SHA256 checksums:

```bash
# For macOS ARM64
curl -L https://github.com/pmaojo/apicentric/releases/download/v0.1.1/apicentric-macos-arm64.tar.gz -o apicentric-macos-arm64.tar.gz
shasum -a 256 apicentric-macos-arm64.tar.gz

# For macOS x64
curl -L https://github.com/pmaojo/apicentric/releases/download/v0.1.1/apicentric-macos-x64.tar.gz -o apicentric-macos-x64.tar.gz
shasum -a 256 apicentric-macos-x64.tar.gz

# For Linux x64
curl -L https://github.com/pmaojo/apicentric/releases/download/v0.1.1/apicentric-linux-x64.tar.gz -o apicentric-linux-x64.tar.gz
shasum -a 256 apicentric-linux-x64.tar.gz
```

3. Update the SHA256 checksums in the formula
4. Test the formula locally:

```bash
brew install --build-from-source Formula/apicentric.rb
brew test apicentric
```

5. Commit and push the changes

### Testing the Formula

```bash
# Audit the formula
brew audit --strict Formula/apicentric.rb

# Test installation
brew install --build-from-source Formula/apicentric.rb

# Run the test block
brew test apicentric

# Verify the binary works
apicentric --version
```

## License

MIT License - see the main [Apicentric repository](https://github.com/pmaojo/apicentric) for details.
