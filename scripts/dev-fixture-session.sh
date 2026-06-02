#!/usr/bin/env bash
set -euo pipefail

# Create a test tmux session with various workloads for manual testing

SESSION="taskgrid-test"

# Kill existing test session
tmux kill-session -t "$SESSION" 2>/dev/null || true

# Create new session
tmux new-session -d -s "$SESSION" -n "main"

# Pane 0: just a shell
tmux send-keys -t "$SESSION":0.0 "echo 'idle shell'" Enter

# Split and run something
tmux split-window -h -t "$SESSION":0.0
tmux send-keys -t "$SESSION":0.1 "sleep 3600" Enter

# Split vertically
tmux split-window -v -t "$SESSION":0.0
tmux send-keys -t "$SESSION":0.2 "top" Enter

# New window for AI agents
tmux new-window -t "$SESSION" -n "agents"
tmux send-keys -t "$SESSION":1.0 "echo 'simulated AI agent'" Enter

tmux split-window -h -t "$SESSION":1.0
tmux send-keys -t "$SESSION":1.1 "python3 -c 'import time; time.sleep(3600)'" Enter

echo "Test session '$SESSION' created."
echo "Attach with: tmux attach -t $SESSION"
echo "Then run: tmux-taskgrid"
