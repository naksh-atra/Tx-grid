# tmux-taskgrid

A Rust-based tmux plugin that opens a popup "task grid" showing all long-running
commands and AI agents across your tmux panes — with status, runtime, filtering,
and one-keystroke actions.

## Features

- **Pane discovery**: Enumerates all panes across all tmux sessions and windows
- **Window grouping**: Panes grouped by window with visual headers
  (`▸ session:window`)
- **Process inspection**: Shows command, runtime, and state for each pane
- **Notes per pane**: Press `n` to open a side-by-side split view editor
- **Rename panes**: Press `r` to set the pane title
- **Interactive TUI**: Navigate, filter, sort, and act from a tmux popup
- **Actions**: Jump to pane, kill pane, rename, add notes
- **CLI modes**: `--check`, `--json`, `--doctor`, `--install-keybinding`
- **Configurable**: Customize popup dimensions via tmux options

## Requirements

- **tmux 3.2+** (for `display-popup`)
- **Linux** (macOS not yet tested)

## Quick Install

### One-liner

```bash
curl -fsSL https://raw.githubusercontent.com/naksh-atra/Tx-grid/main/scripts/install.sh | bash
```

Then reload tmux:

```bash
tmux source-file ~/.tmux.conf
```

### Manual (pre-built binary)

```bash
curl -fsSL -o ~/.local/bin/tmux-taskgrid \
  https://github.com/naksh-atra/Tx-grid/releases/latest/download/tmux-taskgrid-linux-x64
chmod +x ~/.local/bin/tmux-taskgrid
```

Add to `~/.tmux.conf`:

```
bind-key -T prefix P display-popup -w 80% -h 60% -E "tmux-taskgrid"
```

### Build from source

```bash
git clone https://github.com/naksh-atra/Tx-grid.git
cd Tx-grid
cargo build --release
cp target/release/tmux-taskgrid ~/.local/bin/tmux-taskgrid
```

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
| `s` | Cycle sort (runtime → session → state) |
| `Enter` | Jump to selected pane |
| `x` | Kill selected pane (with confirmation) |
| `r` | Rename selected pane |
| `n` | Open notes editor (split view) |
| `q` / `Esc` / `Ctrl+C` | Quit |

### Rename mode

Press `r` on a selected pane. The current pane title (or `session:window` as
default) appears in the footer. Type the new name, `Enter` to apply, `Esc` to
cancel.

### Notes mode

Press `n` on a selected pane. The popup splits:

- **Left**: Task grid (narrowed to #, Pane, Command)
- **Right**: Full notes editor with word wrapping

Type freely. `Enter` for new lines. `Esc` to save and exit. Notes are stored
in `~/.tmux-taskgrid-notes`.

### CLI

```bash
tmux-taskgrid                      # Open TUI popup
tmux-taskgrid --check              # Print task summary
tmux-taskgrid --json               # Print JSON output
tmux-taskgrid --doctor             # Environment diagnostics
tmux-taskgrid --install-keybinding # Add keybinding to ~/.tmux.conf
tmux-taskgrid --debug              # Verbose logging
tmux-taskgrid --version            # Print version
```

## Configuration

Add to `~/.tmux.conf`:

```
# Popup dimensions (percentage or absolute)
set -g @taskgrid-popup-width "80%"
set -g @taskgrid-popup-height "60%"

# Refresh interval in seconds (default: 5)
set -g @taskgrid-refresh-interval "5"

# Confirm before killing panes (default: 1)
set -g @taskgrid-confirm-kill "1"
```

## Platform Support

| Platform | Support |
|----------|---------|
| Linux x64 | Full |
| Linux arm64 | Full |
| macOS | Not yet tested |
| Windows (WSL) | Full |
| Windows (PSMux) | Not supported |

## Data

- Notes are stored in `~/.tmux-taskgrid-notes` (tab-separated: `pane_id\ttext`)
- No other persistent data is created

## Architecture

```
src/
├── main.rs              # Entry point, CLI, event loop
├── cli.rs               # clap argument definitions
├── config.rs            # Config from tmux options
├── logging.rs           # env_logger init
├── models/
│   ├── pane.rs          # PaneId, PaneInfo
│   ├── process.rs       # Process inspection via /proc
│   └── task.rs          # Task classification
├── services/
│   ├── tmux_service.rs  # tmux command wrapper
│   └── task_service.rs  # Task discovery with caching
└── ui/
    ├── state.rs         # App state machine (5 modes)
    ├── render.rs        # ratatui rendering
    ├── layout.rs        # Layout helpers (split view for notes)
    └── events.rs        # crossterm event polling
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

MIT
