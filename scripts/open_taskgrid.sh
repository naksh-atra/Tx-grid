#!/usr/bin/env bash
set -euo pipefail

# Open the taskgrid popup
# This is called by the tmux keybinding

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

# Verify tmux is available
if ! command -v tmux &>/dev/null; then
    echo "tmux not found in PATH" >&2
    exit 1
fi

# Prefer installed binary, fall back to local release/debug build
BINARY=""
if command -v tmux-taskgrid &>/dev/null; then
    BINARY="tmux-taskgrid"
elif [ -x "$PROJECT_DIR/target/release/tmux-taskgrid" ]; then
    BINARY="$PROJECT_DIR/target/release/tmux-taskgrid"
elif [ -x "$PROJECT_DIR/target/debug/tmux-taskgrid" ]; then
    BINARY="$PROJECT_DIR/target/debug/tmux-taskgrid"
fi

if [ -z "$BINARY" ]; then
    tmux display-message "tmux-taskgrid: binary not found. Build with: cargo build --release" 2>/dev/null || true
    echo "tmux-taskgrid binary not found." >&2
    echo "Either install it on PATH or build: cargo build --release" >&2
    exit 1
fi

# Get popup dimensions from tmux options
WIDTH="$(tmux show-option -gv @taskgrid-popup-width 2>/dev/null || echo '80%')"
HEIGHT="$(tmux show-option -gv @taskgrid-popup-height 2>/dev/null || echo '60%')"

# Open popup with taskgrid
tmux display-popup -w "${WIDTH}" -h "${HEIGHT}" -E "${BINARY}"
