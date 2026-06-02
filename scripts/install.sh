#!/usr/bin/env bash
set -euo pipefail

# Install tmux-taskgrid as a tmux plugin (manual install)
# Usage: ./scripts/install.sh [install_dir]

INSTALL_DIR="${1:-$HOME/.tmux/plugins/tmux-taskgrid}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$(dirname "$SCRIPT_DIR")"

echo "Installing tmux-taskgrid to $INSTALL_DIR"

# Create directory
mkdir -p "$INSTALL_DIR/scripts"

# Copy plugin script
cp "$SCRIPT_DIR/open_taskgrid.sh" "$INSTALL_DIR/scripts/"
cp "$SCRIPT_DIR/taskgrid.tmux" "$INSTALL_DIR/"

# Build binary if cargo is available
if command -v cargo &>/dev/null; then
    echo "Building tmux-taskgrid..."
    cd "$REPO_DIR"
    cargo build --release 2>&1
    cp target/release/tmux-taskgrid "$INSTALL_DIR/scripts/"
    echo "Binary built and installed."
else
    echo "Cargo not found. Please build manually:"
    echo "  cd $REPO_DIR && cargo build --release"
    echo "  cp target/release/tmux-taskgrid $INSTALL_DIR/scripts/"
fi

echo ""
echo "Add to ~/.tmux.conf:"
echo "  run-shell '$INSTALL_DIR/taskgrid.tmux'"
echo ""
echo "Then reload: tmux source-file ~/.tmux.conf"
