#!/bin/bash
set -e

# Apicentric Installation Script
# This script downloads and installs the latest version of Apicentric

# Configuration
REPO="pmaojo/apicentric"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"
BINARY_NAME="apicentric"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
print_info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Detect operating system
detect_os() {
    local os
    os=$(uname -s | tr '[:upper:]' '[:lower:]')
    
    case "$os" in
        linux*)
            echo "linux"
            ;;
        darwin*)
            echo "macos"
            ;;
        *)
            print_error "Unsupported operating system: $os"
            print_info "Supported systems: Linux, macOS"
            exit 1
            ;;
    esac
}

# Detect architecture
detect_arch() {
    local arch
    arch=$(uname -m)
    
    case "$arch" in
        x86_64|amd64)
            echo "x64"
            ;;
        aarch64|arm64)
            echo "arm64"
            ;;
        *)
            print_error "Unsupported architecture: $arch"
            print_info "Supported architectures: x86_64 (x64), aarch64/arm64"
            exit 1
            ;;
    esac
}

# Check for required tools
check_requirements() {
    local missing_tools=()
    
    if ! command_exists curl; then
        missing_tools+=("curl")
    fi
    
    if ! command_exists tar; then
        missing_tools+=("tar")
    fi
    
    if ! command_exists sha256sum && ! command_exists shasum; then
        missing_tools+=("sha256sum or shasum")
    fi
    
    if [ ${#missing_tools[@]} -ne 0 ]; then
        print_error "Missing required tools: ${missing_tools[*]}"
        print_info "Please install the missing tools and try again"
        exit 1
    fi
}

# Get latest release version from GitHub
get_latest_version() {
    local version
    version=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
    
    if [ -z "$version" ]; then
        print_error "Failed to fetch latest version from GitHub"
        exit 1
    fi
    
    echo "$version"
}

# Download file with progress
download_file() {
    local url="$1"
    local output="$2"
    
    if ! curl -fsSL --progress-bar "$url" -o "$output"; then
        print_error "Failed to download from $url"
        return 1
    fi
    
    return 0
}

# Verify checksum
verify_checksum() {
    local file="$1"
    local checksums_file="$2"
    local filename
    filename=$(basename "$file")
    
    print_info "Verifying checksum..."
    
    # Extract the checksum for our file
    local expected_checksum
    expected_checksum=$(grep "$filename" "$checksums_file" | awk '{print $1}')
    
    if [ -z "$expected_checksum" ]; then
        print_warning "Checksum not found in checksums file, skipping verification"
        return 0
    fi
    
    # Calculate actual checksum
    local actual_checksum
    if command_exists sha256sum; then
        actual_checksum=$(sha256sum "$file" | awk '{print $1}')
    elif command_exists shasum; then
        actual_checksum=$(shasum -a 256 "$file" | awk '{print $1}')
    else
        print_warning "No checksum tool available, skipping verification"
        return 0
    fi
    
    if [ "$expected_checksum" != "$actual_checksum" ]; then
        print_error "Checksum verification failed!"
        print_error "Expected: $expected_checksum"
        print_error "Got:      $actual_checksum"
        return 1
    fi
    
    print_success "Checksum verified"
    return 0
}

# Check if already installed
check_existing_installation() {
    if [ -f "${INSTALL_DIR}/${BINARY_NAME}" ]; then
        local current_version
        current_version=$("${INSTALL_DIR}/${BINARY_NAME}" --version 2>/dev/null | awk '{print $2}' || echo "unknown")
        print_info "Apicentric is already installed (version: $current_version)"
        print_info "This will upgrade/reinstall the binary"
        return 0
    fi
    return 1
}

# Main installation function
main() {
    echo ""
    print_info "Apicentric Installation Script"
    echo ""
    
    # Check requirements
    check_requirements
    
    # Detect platform
    print_info "Detecting platform..."
    OS=$(detect_os)
    ARCH=$(detect_arch)
    print_success "Platform detected: $OS-$ARCH"
    
    # Check existing installation
    check_existing_installation
    
    # Get latest version
    print_info "Fetching latest version..."
    VERSION=$(get_latest_version)
    print_success "Latest version: $VERSION"
    
    # Construct download URLs
    ASSET_NAME="${BINARY_NAME}-${OS}-${ARCH}.tar.gz"
    DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${VERSION}/${ASSET_NAME}"
    CHECKSUMS_URL="https://github.com/${REPO}/releases/download/${VERSION}/checksums.txt"
    
    # Create temporary directory
    TMP_DIR=$(mktemp -d)
    trap 'rm -rf "$TMP_DIR"' EXIT
    
    # Download binary archive
    print_info "Downloading ${ASSET_NAME}..."
    if ! download_file "$DOWNLOAD_URL" "${TMP_DIR}/${ASSET_NAME}"; then
        print_error "Download failed. Please check your internet connection and try again."
        exit 1
    fi
    print_success "Downloaded successfully"
    
    # Download checksums
    print_info "Downloading checksums..."
    if ! download_file "$CHECKSUMS_URL" "${TMP_DIR}/checksums.txt"; then
        print_warning "Failed to download checksums, skipping verification"
    else
        # Verify checksum
        if ! verify_checksum "${TMP_DIR}/${ASSET_NAME}" "${TMP_DIR}/checksums.txt"; then
            print_error "Installation aborted due to checksum mismatch"
            exit 1
        fi
    fi
    
    # Extract archive
    print_info "Extracting archive..."
    if ! tar -xzf "${TMP_DIR}/${ASSET_NAME}" -C "$TMP_DIR"; then
        print_error "Failed to extract archive"
        exit 1
    fi
    print_success "Extracted successfully"
    
    # Install binary
    print_info "Installing to ${INSTALL_DIR}..."
    
    # Check if we need sudo
    if [ -w "$INSTALL_DIR" ]; then
        mv "${TMP_DIR}/${BINARY_NAME}" "${INSTALL_DIR}/${BINARY_NAME}"
        chmod +x "${INSTALL_DIR}/${BINARY_NAME}"
    else
        print_info "Requesting sudo access to install to ${INSTALL_DIR}..."
        sudo mv "${TMP_DIR}/${BINARY_NAME}" "${INSTALL_DIR}/${BINARY_NAME}"
        sudo chmod +x "${INSTALL_DIR}/${BINARY_NAME}"
    fi
    
    print_success "Installed successfully"
    
    # Verify installation
    print_info "Verifying installation..."
    if command_exists "$BINARY_NAME"; then
        INSTALLED_VERSION=$("$BINARY_NAME" --version 2>/dev/null | awk '{print $2}' || echo "unknown")
        print_success "Apicentric ${INSTALLED_VERSION} is now installed!"
    else
        print_warning "Installation completed, but ${BINARY_NAME} is not in PATH"
        print_info "You may need to add ${INSTALL_DIR} to your PATH"
        print_info "Add this to your shell profile: export PATH=\"${INSTALL_DIR}:\$PATH\""
    fi
    
    echo ""
    print_success "Installation complete!"
    print_info "Run '${BINARY_NAME} --help' to get started"
    echo ""
}

# Run main function
main "$@"
