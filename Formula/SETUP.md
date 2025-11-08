# Homebrew Tap Setup Guide

This guide explains how to set up the `pmaojo/homebrew-tap` repository for distributing Apicentric via Homebrew.

## Repository Setup

### 1. Create the Tap Repository

Create a new GitHub repository named `homebrew-tap` under the `pmaojo` organization or user account.

```bash
# Repository URL should be:
https://github.com/pmaojo/homebrew-tap
```

### 2. Initialize the Repository

```bash
# Clone the repository
git clone https://github.com/pmaojo/homebrew-tap.git
cd homebrew-tap

# Copy the formula from the main repository
cp /path/to/apicentric/Formula/apicentric.rb Formula/apicentric.rb
cp /path/to/apicentric/Formula/README.md README.md
cp /path/to/apicentric/Formula/update-checksums.sh update-checksums.sh
cp /path/to/apicentric/Formula/SETUP.md SETUP.md

# Make the update script executable
chmod +x update-checksums.sh

# Commit and push
git add .
git commit -m "Initial tap setup with apicentric formula"
git push origin main
```

### 3. Repository Structure

```
homebrew-tap/
├── Formula/
│   └── apicentric.rb
├── README.md
├── SETUP.md
├── update-checksums.sh
└── .github/
    └── workflows/
        └── test.yml (optional)
```

## Updating the Formula for New Releases

### Automated Method (Recommended)

Use the provided script to automatically fetch checksums and update the formula:

```bash
./update-checksums.sh 0.1.1
```

This will:
1. Download the release artifacts
2. Calculate SHA256 checksums
3. Update the formula with the new version and checksums

### Manual Method

1. Update the version in `Formula/apicentric.rb`:
   ```ruby
   version "0.1.1"
   ```

2. Download each release artifact and calculate checksums:
   ```bash
   curl -L https://github.com/pmaojo/apicentric/releases/download/v0.1.1/apicentric-macos-arm64.tar.gz -o apicentric-macos-arm64.tar.gz
   shasum -a 256 apicentric-macos-arm64.tar.gz
   ```

3. Update the SHA256 values in the formula

4. Test the formula locally

5. Commit and push

## Testing the Formula

### Local Testing

```bash
# Audit the formula
brew audit --strict Formula/apicentric.rb

# Install from local formula
brew install --build-from-source Formula/apicentric.rb

# Run the test block
brew test apicentric

# Verify it works
apicentric --version

# Uninstall
brew uninstall apicentric
```

### Testing from the Tap

```bash
# Add the tap
brew tap pmaojo/tap

# Install
brew install apicentric

# Test
brew test apicentric

# Verify
apicentric --version
```

## CI/CD Integration (Optional)

You can add a GitHub Actions workflow to automatically test the formula:

```yaml
# .github/workflows/test.yml
name: Test Formula

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

jobs:
  test:
    strategy:
      matrix:
        os: [macos-latest, ubuntu-latest]
    runs-on: ${{ matrix.os }}
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Set up Homebrew
        uses: Homebrew/actions/setup-homebrew@master
      
      - name: Audit formula
        run: brew audit --strict Formula/apicentric.rb
      
      - name: Test formula
        run: |
          brew install --build-from-source Formula/apicentric.rb
          brew test apicentric
```

## Troubleshooting

### Formula Fails to Install

1. Check that the release artifacts exist at the URLs
2. Verify the SHA256 checksums match
3. Ensure the binary is named `apicentric` in the tarball
4. Check Homebrew logs: `brew install --verbose apicentric`

### Checksum Mismatch

If you get a checksum mismatch error:

1. Re-download the artifact
2. Recalculate the checksum
3. Update the formula
4. Clear Homebrew cache: `rm -rf $(brew --cache)`

### Test Failures

If `brew test apicentric` fails:

1. Check that the binary is executable
2. Verify `--version` flag works
3. Check the test block in the formula

## Best Practices

1. **Always test locally** before pushing to the tap
2. **Use semantic versioning** for releases
3. **Keep checksums up to date** with each release
4. **Document breaking changes** in release notes
5. **Test on both macOS and Linux** if possible

## Resources

- [Homebrew Formula Cookbook](https://docs.brew.sh/Formula-Cookbook)
- [Homebrew Acceptable Formulae](https://docs.brew.sh/Acceptable-Formulae)
- [How to Create and Maintain a Tap](https://docs.brew.sh/How-to-Create-and-Maintain-a-Tap)
