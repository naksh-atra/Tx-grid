# Decision Log

## Why Rust?

- Performance: fast startup even with many panes
- Safety: no segfaults in a tool that manages your terminal
- Distribution: single static binary
- Ecosystem: excellent TUI (ratatui) and procfs crates

## Why ratatui?

- Most mature Rust TUI framework
- Used by many popular CLI tools (ripgrep, etc.)
- Good documentation and active maintenance

## Why raw `tmux` commands instead of `tmux_interface` crate?

- `tmux_interface` adds complexity for minimal benefit
- Direct `Command::new("tmux")` is simpler and more transparent
- Easy to debug: visible in process list

## Why best-effort restart in v1?

- Reliable restart requires shell integration (history, etc.)
- Up+Enter heuristic works for most interactive shells
- Can be improved in v2 with shell integration

## Why Linux-first?

- `/proc` gives rich process info
- Primary target is dev servers and workstations
- macOS can use `sysinfo` fallback

## Why not live-refresh via tmux events?

- Polling is simpler and more reliable
- tmux event hooks are complex to set up
- 5-second polling is sufficient for the use case
