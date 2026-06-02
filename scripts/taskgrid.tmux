#!/usr/bin/env bash
# tmux-taskgrid plugin script
# Source this from ~/.tmux.conf using:
#   run-shell /path/to/taskgrid.tmux
#
# Or install via TPM:
#   set -g @plugin 'naksh-atra/tmux-taskgrid'

# Default keybinding: Prefix + C-t
# To override in .tmux.conf:
#   set -g @taskgrid-key 'C-t'

CURRENT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Set default options
tmux set-option -g @taskgrid-popup-width "80%" 2>/dev/null || true
tmux set-option -g @taskgrid-popup-height "60%" 2>/dev/null || true
tmux set-option -g @taskgrid-key "C-t" 2>/dev/null || true

# Get the configured key
KEY="$(tmux show-option -gv @taskgrid-key 2>/dev/null || echo 'C-t')"

# Bind the key
tmux bind-key "$KEY" run-shell "${CURRENT_DIR}/open_taskgrid.sh"

# Alternative: bind to t as well (without prefix)
# tmux bind-key -n C-t run-shell "${CURRENT_DIR}/open_taskgrid.sh"
