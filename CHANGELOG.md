# Changelog

## [0.1.0] - 2026-06-01

### Added
- Initial project structure and Rust crate
- Pane discovery via `tmux list-panes -a`
- Process inspection using `/proc` on Linux
- Task classification (Running, Idle, Exited, Unknown)
- TUI with ratatui: header, task grid, footer
- Keyboard navigation (j/k, g/G, PgUp/PgDn)
- Filter mode with case-insensitive substring matching
- Sort by runtime, session, or state
- Jump to pane (Enter)
- Kill pane with confirmation (x)
- Best-effort restart (r)
- Configuration via tmux options
- `--check` and `--json` non-interactive modes
- `--doctor` environment diagnostics
- CI/CD with GitHub Actions (fmt, clippy, test, build, release)
- Unit tests for parsing, classification, and UI state
- Integration tests with mock process provider
- Full documentation set
