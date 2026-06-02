#!/usr/bin/env bash
set -euo pipefail

# Open the taskgrid popup
# This is called by the tmux keybinding

BINARY="${TMUX_TASKGRID_BINARY:-tmux-taskgrid}"

# Check if binary exists
if ! command -v "$BINARY" &>/dev/null; then
    echo "tmux-taskgrid binary not found in PATH" >&2
    exit 1
fi

# Get popup dimensions from tmux options
WIDTH="${TMUX_TASKGRID_WIDTH:-80%}"
HEIGHT="${TMUX_TASKGRID_HEIGHT:-60%}"

# Open popup with taskgrid
tmux display-popup -w "$WIDTH" -h "$HEIGHT" -E "$BINARY"
