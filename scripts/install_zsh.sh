#!/bin/bash

# Note: This script is run with bash, but installs for zsh.

set -e

REPO="zhangzhenxiang666/env-manager"
INSTALL_DIR="$HOME/.config/env-manage/bin"
TARGET_BIN="$INSTALL_DIR/env-manage"
RC_FILE="$HOME/.zshrc"

# Detect OS and Arch
OS="$(uname -s)"
ARCH="$(uname -m)"
BINARY_SUFFIX=""

if [ "$OS" = "Linux" ]; then
    if [ "$ARCH" = "x86_64" ]; then
        BINARY_SUFFIX="linux-amd64"
    elif [ "$ARCH" = "aarch64" ] || [ "$ARCH" = "arm64" ]; then
        BINARY_SUFFIX="linux-aarch64"
    else
        echo "Unsupported Linux architecture: $ARCH"
        exit 1
    fi
elif [ "$OS" = "Darwin" ]; then
    if [ "$ARCH" = "x86_64" ]; then
        BINARY_SUFFIX="darwin-amd64"
    elif [ "$ARCH" = "arm64" ]; then
        BINARY_SUFFIX="darwin-aarch64"
    else
        echo "Unsupported MacOS architecture: $ARCH"
        exit 1
    fi
else
    echo "Unsupported OS: $OS"
    exit 1
fi

BINARY="env-manage-${BINARY_SUFFIX}"

echo "Detected OS: $OS, Arch: $ARCH"
echo "Installing $BINARY to $TARGET_BIN ..."

# 1. Create directory
mkdir -p "$INSTALL_DIR"

# 2. Download binary
echo "Fetching latest version..."
LATEST_TAG=$(curl -s https://api.github.com/repos/$REPO/releases/latest | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')

if [ -z "$LATEST_TAG" ]; then
    echo "Error: Could not determine latest version."
    exit 1
fi

echo "Latest version: $LATEST_TAG"
ASSET_NAME="env-manage-${LATEST_TAG}-${BINARY_SUFFIX}.tar.gz"
DOWNLOAD_URL="https://github.com/$REPO/releases/latest/download/$ASSET_NAME"

echo "Downloading from: $DOWNLOAD_URL"
TEMP_DIR=$(mktemp -d)
curl -L "$DOWNLOAD_URL" -o "$TEMP_DIR/$ASSET_NAME"

echo "Extracting..."
tar -xzf "$TEMP_DIR/$ASSET_NAME" -C "$TEMP_DIR"

# Find binary
FIND_BIN=$(find "$TEMP_DIR" -type f -name "env-manage" | head -n 1)

if [ -z "$FIND_BIN" ]; then
    echo "Error: Could not find binary in archive"
    rm -rf "$TEMP_DIR"
    exit 1
fi

mv "$FIND_BIN" "$TARGET_BIN"
rm -rf "$TEMP_DIR"

# 3. Make executable
chmod +x "$TARGET_BIN"

# 4. Add to shell config
echo "Configuring $RC_FILE..."

INIT_CMD="eval \"\$($TARGET_BIN init zsh)\""

# Check if already configured
if [ -f "$RC_FILE" ] && grep -q "env-manage init zsh" "$RC_FILE"; then
    echo "env-manage init already configured in $RC_FILE"
else
    echo "" >> "$RC_FILE"
    echo "# env-manage" >> "$RC_FILE"
    echo "$INIT_CMD" >> "$RC_FILE"
    echo "Added env-manage init to $RC_FILE"
fi

echo "Installation complete!"
echo "Please restart your shell or run 'source $RC_FILE' to use 'em'."
