#!/usr/bin/env bash
# tmux-taskgrid plugin script
# Source this from ~/.tmux.conf using:
#   run-shell /path/to/taskgrid.tmux
#
# Or install via TPM:
#   set -g @plugin 'naksh-atra/tmux-taskgrid'

# Default keybinding: Prefix + Shift+p (P)
# To override in .tmux.conf:
#   set -g @taskgrid-key 'P'

CURRENT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Set default options
tmux set-option -g @taskgrid-popup-width "80%" 2>/dev/null || true
tmux set-option -g @taskgrid-popup-height "60%" 2>/dev/null || true
tmux set-option -g @taskgrid-key "P" 2>/dev/null || true

# Get the configured key
KEY="$(tmux show-option -gv @taskgrid-key 2>/dev/null || echo 'P')"

# Bind the key
tmux bind-key "$KEY" run-shell "bash ${CURRENT_DIR}/open_taskgrid.sh"
