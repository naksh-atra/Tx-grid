# tmux-taskgrid

A Rust-based tmux plugin that opens a popup "task grid" showing all long-running commands and AI agents across tmux panes, with status, runtime, and one-keystroke jump/kill/restart.

## Features

- **Pane discovery**: Enumerates all panes across all tmux sessions
- **Process inspection**: Shows the running command, runtime, and state for each pane
- **Task classification**: Automatically classifies panes as running, idle, or exited
- **Interactive TUI**: Navigate, filter, sort, and act from a tmux popup
- **Actions**: Jump to pane, kill pane, restart task, all from one keystroke
- **Configurable**: Customize behavior via tmux options in `.tmux.conf`

## Requirements

- tmux 3.2+ (for `display-popup`)
- Linux (macOS support planned)
- Rust 1.70+ (for building from source)

## Installation

### Via TPM (coming soon)

Add to `.tmux.conf`:

```
set -g @plugin 'naksh-atra/tmux-taskgrid'
```

Then prefix+I to install.

### Manual

1. Download the latest binary from the [releases page](https://github.com/naksh-atra/Tx-grid/releases)
2. Place it on your PATH as `tmux-taskgrid`
3. Add to `.tmux.conf`:

```
bind-key C-t run-shell "tmux-taskgrid"
```

4. Reload tmux config: `tmux source-file ~/.tmux.conf`

## Keybindings

| Key | Action |
|-----|--------|
| `j`/`k` or arrows | Move selection up/down |
| `g`/`G` | Jump to first/last task |
| `PgUp`/`PgDn` | Page up/down |
| `/` | Enter filter mode |
| `s` | Cycle sort order (runtime → session → state) |
| `Enter` | Jump to selected pane |
| `x` | Kill selected pane (with confirmation) |
| `r` | Restart task in selected pane (best-effort) |
| `q`/`Esc` | Quit |

## Configuration

Add to `.tmux.conf`:

```
# Popup dimensions (percentage or absolute)
set -g @taskgrid-popup-width "80%"
set -g @taskgrid-popup-height "60%"

# Runtime threshold in seconds (tasks below this may be hidden)
set -g @taskgrid-runtime-threshold "5"

# Confirm before killing panes (0 or 1)
set -g @taskgrid-confirm-kill "1"

# Refresh interval in seconds
set -g @taskgrid-refresh-interval "5"
```

## CLI Modes

```
tmux-taskgrid          # Launch TUI popup
tmux-taskgrid --check  # Print task summary and exit
tmux-taskgrid --json   # Print JSON output
tmux-taskgrid --doctor # Run environment diagnostics
tmux-taskgrid --debug  # Enable verbose logging
```

## Building from Source

```bash
git clone https://github.com/naksh-atra/Tx-grid.git
cd tmux-taskgrid
cargo build --release
```

## Limitations

- Best-effort restart (uses Up+Enter in the shell)
- Linux-first; macOS not yet tested
- No live-refresh in v1 (5-second polling only)

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

MIT
