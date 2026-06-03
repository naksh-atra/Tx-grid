#!/usr/bin/env bash
set -euo pipefail

# tmux-taskgrid installer
# Usage: curl -fsSL https://raw.githubusercontent.com/naksh-atra/Tx-grid/main/scripts/install.sh | bash

REPO="naksh-atra/Tx-grid"
BINARY_NAME="tmux-taskgrid"
INSTALL_DIR="${HOME}/.local/bin"

echo "=== tmux-taskgrid installer ==="

# Detect OS and architecture
OS="$(uname -s)"
ARCH="$(uname -m)"
case "$OS" in
    Linux*)  PLATFORM="linux";;
    Darwin*) PLATFORM="macos";;
    *)       echo "Unsupported OS: $OS"; exit 1;;
esac
case "$ARCH" in
    x86_64|amd64)  ARCH="x64";;
    aarch64|arm64) ARCH="arm64";;
    *)             echo "Unsupported arch: $ARCH"; exit 1;;
esac

SUFFIX="${PLATFORM}-${ARCH}"

# Check if we should build from source or download
if command -v cargo &>/dev/null; then
    echo "Rust detected — building from source..."
    if ! command -v git &>/dev/null; then
        echo "git is required to clone the repo" >&2
        exit 1
    fi
    TMP_DIR="$(mktemp -d)"
    trap "rm -rf $TMP_DIR" EXIT
    git clone --depth 1 "https://github.com/${REPO}.git" "$TMP_DIR"
    cd "$TMP_DIR"
    cargo build --release
    mkdir -p "$INSTALL_DIR"
    cp target/release/tmux-taskgrid "$INSTALL_DIR/$BINARY_NAME"
    chmod +x "$INSTALL_DIR/$BINARY_NAME"
    echo "Built and installed to $INSTALL_DIR/$BINARY_NAME"
else
    echo "Downloading pre-built binary (${SUFFIX})..."
    LATEST_URL="https://api.github.com/repos/${REPO}/releases/latest"
    TAG="$(curl -fsSL "$LATEST_URL" | grep '"tag_name"' | head -1 | sed 's/.*"tag_name": "\(.*\)".*/\1/')"
    if [ -z "$TAG" ]; then
        echo "Could not determine latest release. Install Rust and build from source:" >&2
        echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh" >&2
        echo "  git clone https://github.com/${REPO}.git && cd Tx-grid && cargo build --release" >&2
        exit 1
    fi

    DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${TAG}/tmux-taskgrid-${SUFFIX}"
    TMP_FILE="$(mktemp)"
    trap "rm -f $TMP_FILE" EXIT

    if ! curl -fsSL -o "$TMP_FILE" "$DOWNLOAD_URL"; then
        echo "Download failed. Your platform (${SUFFIX}) may not have a pre-built binary yet." >&2
        echo "Install Rust and build from source:" >&2
        echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh" >&2
        echo "  git clone https://github.com/${REPO}.git && cd Tx-grid && cargo build --release" >&2
        exit 1
    fi

    mkdir -p "$INSTALL_DIR"
    mv "$TMP_FILE" "$INSTALL_DIR/$BINARY_NAME"
    chmod +x "$INSTALL_DIR/$BINARY_NAME"
    echo "Installed to $INSTALL_DIR/$BINARY_NAME"
fi

# Verify installation
if ! command -v "$BINARY_NAME" &>/dev/null; then
    echo "WARNING: $INSTALL_DIR is not in your PATH." >&2
    echo "Add this to your shell profile:" >&2
    echo '  export PATH="$HOME/.local/bin:$PATH"' >&2
fi

# Add keybinding to .tmux.conf if not already present
TMUX_CONF="${HOME}/.tmux.conf"
if [ -f "$TMUX_CONF" ]; then
    if grep -q "tmux-taskgrid" "$TMUX_CONF"; then
        echo "Keybinding already present in ~/.tmux.conf"
    else
        echo "" >> "$TMUX_CONF"
        echo "# tmux-taskgrid popup keybinding" >> "$TMUX_CONF"
        echo 'bind-key -T prefix P display-popup -w 80% -h 60% -E "tmux-taskgrid"' >> "$TMUX_CONF"
        echo "Added keybinding to ~/.tmux.conf"
    fi
else
    echo "# tmux-taskgrid popup keybinding" > "$TMUX_CONF"
    echo 'bind-key -T prefix P display-popup -w 80% -h 60% -E "tmux-taskgrid"' >> "$TMUX_CONF"
    echo "Created ~/.tmux.conf with keybinding"
fi

echo ""
echo "=== Installation complete ==="
echo "Reload tmux config: tmux source-file ~/.tmux.conf"
echo "Open task grid:    Prefix + Shift+P"
echo ""
echo "Run '$BINARY_NAME --doctor' to verify your setup."
