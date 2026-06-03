#!/usr/bin/env bash
set -euo pipefail

# Open the taskgrid popup
# This is called by the tmux keybinding

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

# Prefer installed binary, fall back to local release/debug build
BINARY="tmux-taskgrid"
if ! command -v "$BINARY" &>/dev/null; then
    if [ -x "$PROJECT_DIR/target/release/tmux-taskgrid" ]; then
        BINARY="$PROJECT_DIR/target/release/tmux-taskgrid"
    elif [ -x "$PROJECT_DIR/target/debug/tmux-taskgrid" ]; then
        BINARY="$PROJECT_DIR/target/debug/tmux-taskgrid"
    fi
fi

# Check if binary exists
if ! command -v "$BINARY" &>/dev/null && [ ! -x "$BINARY" ]; then
    tmux display-message "tmux-taskgrid binary not found. Run: cargo build --release"
    exit 1
fi

# Get popup dimensions from tmux options
WIDTH="${TMUX_TASKGRID_WIDTH:-80%}"
HEIGHT="${TMUX_TASKGRID_HEIGHT:-60%}"

# Open popup with taskgrid
tmux display-popup -w "$WIDTH" -h "$HEIGHT" -E "$BINARY"
