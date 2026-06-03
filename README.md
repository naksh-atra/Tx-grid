# tmux-taskgrid

A Rust-based tmux plugin that opens a popup "task grid" showing all long-running commands and AI agents across your tmux panes — with status, runtime, filtering, and one-keystroke actions.

![tmux-taskgrid popup showing a grid of panes with commands, runtimes, and states](docs/screenshot.png)

## Features

- **Pane discovery**: Enumerates all panes across all tmux sessions and windows
- **Window grouping**: Panes grouped by window with visual headers (`▸ session:window`)
- **Process inspection**: Shows command, runtime, and state (running/idle/exited) for each pane
- **Notes per pane**: Press `n` to open a side-by-side notes editor; notes persist across sessions
- **Rename panes**: Press `r` to rename any pane (sets the tmux pane title)
- **Interactive TUI**: Navigate, filter, sort, and act from a centered tmux popup
- **Actions**: Jump to pane, kill pane, rename, add notes — all from one keystroke
- **CLI modes**: `--check`, `--json`, `--doctor`, `--install-keybinding`
- **Configurable**: Customize via tmux options in `~/.tmux.conf`

## Requirements

- **tmux 3.2+** (for `display-popup` support)
- **Linux** or **macOS**
- Rust 1.70+ (only if building from source)

## Quick Install

### One-liner (downloads pre-built binary or builds from source)

```bash
curl -fsSL https://raw.githubusercontent.com/naksh-atra/Tx-grid/main/scripts/install.sh | bash
```

Then reload tmux:
```bash
tmux source-file ~/.tmux.conf
```

### Manual install (pre-built binary)

1. Download the latest binary for your platform from the [releases page](https://github.com/naksh-atra/Tx-grid/releases)
2. Place it on your PATH:

```bash
# Linux x64
curl -fsSL -o ~/.local/bin/tmux-taskgrid \
  https://github.com/naksh-atra/Tx-grid/releases/latest/download/tmux-taskgrid-linux-x64
chmod +x ~/.local/bin/tmux-taskgrid
```

3. Add to `~/.tmux.conf`:

```
bind-key -T prefix P display-popup -w 80% -h 60% -E "tmux-taskgrid"
```

4. Reload: `tmux source-file ~/.tmux.conf`

### Build from source

```bash
git clone https://github.com/naksh-atra/Tx-grid.git
cd Tx-grid
cargo build --release
cp target/release/tmux-taskgrid ~/.local/bin/tmux-taskgrid
```

### TPM (Tmux Plugin Manager)

Add to `~/.tmux.conf`:

```
set -g @plugin 'naksh-atra/tmux-taskgrid'
```

Then press `Prefix + I` to install.

## Usage

Open the task grid popup:

```
Ctrl+B, then Shift+P
```

### Keybindings

| Key | Action |
|-----|--------|
| `j` / `k` or `↑` / `↓` | Navigate up/down |
| `g` / `G` | Jump to first/last pane |
| `PgUp` / `PgDn` | Page up/down |
| `/` | Filter panes by name/command |
| `s` | Cycle sort order (runtime → session → state) |
| `Enter` | Jump to selected pane |
| `x` | Kill selected pane (with confirmation) |
| `r` | Rename selected pane |
| `n` | Open notes editor for selected pane (split view) |
| `q` / `Esc` / `Ctrl+C` | Quit |

### Rename mode

Press `r` on a selected pane. The current pane title (or `session:window` as default) appears in the footer. Type the new name, `Enter` to apply, `Esc` to cancel.

### Notes mode

Press `n` on a selected pane. The popup splits:
- **Left**: Task grid (narrowed to #, Pane, Command)
- **Right**: Full notes editor with word wrapping

Type freely. `Enter` for new lines. `Esc` to save and exit. Notes are stored in `~/.tmux-taskgrid-notes`.

### CLI modes

```bash
tmux-taskgrid                # Open TUI popup
tmux-taskgrid --check        # Print task summary (non-interactive)
tmux-taskgrid --json         # Print JSON output
tmux-taskgrid --doctor       # Environment diagnostics
tmux-taskgrid --install-keybinding  # Auto-configure ~/.tmux.conf
tmux-taskgrid --debug        # Verbose logging
tmux-taskgrid --version      # Print version
```

## Configuration

Add to `~/.tmux.conf`:

```
# Popup dimensions
set -g @taskgrid-popup-width "80%"
set -g @taskgrid-popup-height "60%"

# Keybinding (default: P = Shift+P)
set -g @taskgrid-key "P"

# Refresh interval in seconds
set -g @taskgrid-refresh-interval "5"

# Confirm before killing panes
set -g @taskgrid-confirm-kill "1"
```

## Platform Support

| Platform | Popup | Notes |
|----------|-------|-------|
| Linux (x64, arm64) | ✅ Full support | |
| macOS (x64, arm64) | ✅ Full support | |
| Windows (PSMux) | ❌ Popup doesn't support TUI | Run in pane mode only |
| Windows (WSL) | ✅ Full support | |

## Architecture

```
src/
├── main.rs          # Entry point, CLI parsing, event loop
├── cli.rs           # clap argument definitions
├── config.rs        # Config from tmux options
├── logging.rs       # env_logger init
├── models/
│   ├── pane.rs      # PaneId, PaneInfo structs
│   ├── process.rs   # Process inspection via /proc
│   └── task.rs      # Task classification, build_tasks
├── services/
│   ├── tmux_service.rs  # tmux command wrapper
│   └── task_service.rs  # Task discovery with caching
└── ui/
    ├── state.rs     # App state machine (Normal/Filter/Confirm/Rename/Notes)
    ├── render.rs    # ratatui rendering
    ├── layout.rs    # Layout helpers (split view for notes)
    └── events.rs    # crossterm event polling
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

MIT
