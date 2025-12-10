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
echo "Downloading latest release from $REPO..."
DOWNLOAD_URL="https://github.com/$REPO/releases/latest/download/$BINARY"
echo "Downloading from: $DOWNLOAD_URL"
curl -L "$DOWNLOAD_URL" -o "$TARGET_BIN"

# 3. Make executable
chmod +x "$TARGET_BIN"

# 4. Add to shell config
echo "Configuring $RC_FILE..."

# Define the function block (Zsh compatible)
read -r -d '' FUNC_BLOCK <<EOF || true
function em() {
    local output
    local exit_code

    output=\$(CLICOLOR_FORCE=1 EM_SHELL=zsh $TARGET_BIN "\$@")
    exit_code=\$?

    if [[ \$exit_code -ne 0 ]]; then
        return \$exit_code
    fi

    if [[ -z "\$output" ]]; then
        return 0
    fi

    if [[ "\$output" =~ ^__SHELL_CMD__ ]]; then
        eval "\${output#__SHELL_CMD__}"
    else
        echo "\$output"
    fi

    return \$exit_code
}

em init
EOF

# Check if function already exists in RC_FILE
if [ -f "$RC_FILE" ] && grep -q "function em()" "$RC_FILE"; then
    echo "Function 'em' already exists in $RC_FILE"
     if ! grep -q "em init" "$RC_FILE"; then
         echo "Adding 'em init' to $RC_FILE"
         echo "em init" >> "$RC_FILE"
    fi
else
    echo "" >> "$RC_FILE"
    echo "# env-manage" >> "$RC_FILE"
    echo "$FUNC_BLOCK" >> "$RC_FILE"
    echo "Added 'em' function and init to $RC_FILE."
fi

echo "Installation complete!"
echo "Please restart your shell or run 'source $RC_FILE' to use 'em'."
