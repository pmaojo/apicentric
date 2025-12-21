#!/bin/bash
set -e

# --- Configuration ---
BACKEND_DIR="webui/backend"
BINARY_NAME="apicentric"
REPO="pmaojo/apicentric"

# --- Script Logic ---

# Ensure the backend directory exists
mkdir -p "$BACKEND_DIR"

# Check if the binary already exists
if [ -f "$BACKEND_DIR/$BINARY_NAME" ]; then
  echo "Backend binary already exists."
else
  echo "Backend binary not found. Downloading the latest release..."

  # Determine the OS and architecture to download the correct binary
  OS=$(uname -s | tr '[:upper:]' '[:lower:]')
  ARCH=$(uname -m)
  ASSET_NAME=""

  if [ "$OS" == "linux" ]; then
    if [ "$ARCH" == "x86_64" ]; then
      ASSET_NAME="apicentric-x86_64-unknown-linux-gnu.tar.gz"
    elif [ "$ARCH" == "aarch64" ]; then
      ASSET_NAME="apicentric-aarch64-unknown-linux-gnu.tar.gz"
    fi
  elif [ "$OS" == "darwin" ]; then
    if [ "$ARCH" == "x86_64" ]; then
      ASSET_NAME="apicentric-x86_64-apple-darwin.tar.gz"
    elif [ "$ARCH" == "arm64" ]; then
      ASSET_NAME="apicentric-aarch64-apple-darwin.tar.gz"
    fi
  fi

  if [ -z "$ASSET_NAME" ]; then
    echo "Unsupported OS or architecture: $OS-$ARCH"
    # Don't fail the build if OS is unsupported (e.g. Windows in some environments), just warn
    echo "Warning: Could not determine correct binary for this environment. Backend will not be installed."
    exit 0
  fi

  # Fetch the URL of the asset from the latest release
  # Note: This requires internet access during build
  DOWNLOAD_URL=$(curl -s "https://api.github.com/repos/$REPO/releases/latest" | grep "browser_download_url.*$ASSET_NAME" | cut -d '"' -f 4)

  if [ -z "$DOWNLOAD_URL" ]; then
    echo "Could not find asset '$ASSET_NAME' in the latest release."
    # Don't fail hard, just warn
    echo "Warning: Failed to find binary download URL."
    exit 0
  fi

  echo "Downloading from $DOWNLOAD_URL..."
  curl -L "$DOWNLOAD_URL" | tar -xz -C "$BACKEND_DIR"

  # Ensure the binary is executable
  chmod +x "$BACKEND_DIR/$BINARY_NAME"

  echo "Backend binary installed successfully to $BACKEND_DIR/$BINARY_NAME"
fi
