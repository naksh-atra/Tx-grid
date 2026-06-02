# Product Scope

## One-Sentence Vision

A Rust-based tmux plugin that opens a popup task grid showing all long-running commands and AI agents across tmux panes.

## The 3 Most Important User Actions in v1

1. See all running/Idle/exited tasks across all sessions
2. Jump to any task's pane with Enter
3. Kill or restart tasks from the grid

## Explicitly Out of Scope for v1

- No AI assistant functionality
- No YAML session layouts (tmuxp replacement)
- No statusline integration
- No live-refresh via tmux events (polling only)
- No macOS-specific optimizations
- No remote tmux session management beyond discovery

## Quality Bar for v1

- Binary builds on Linux
- All unit tests pass
- Manual acceptance checks pass
- README install steps work from a clean machine
- Error messages are understandable
- No panics or confusing crashes
