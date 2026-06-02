# Architecture

## Components

```
tmux.conf keybinding
    │
    ▼
display-popup -E "tmux-taskgrid"
    │
    ▼
┌─────────────────────────────────┐
│         Rust Binary             │
│                                 │
│  ┌───────────┐  ┌────────────┐  │
│  │ tmux.rs   │  │ process.rs │  │
│  │ (tmux cmd)│  │ (/proc)    │  │
│  └─────┬─────┘  └──────┬─────┘  │
│        │               │        │
│        ▼               ▼        │
│  ┌───────────┐  ┌────────────┐  │
│  │ task.rs   │  │ config.rs  │  │
│  │(classify) │  │(tmux opts) │  │
│  └─────┬─────┘  └────────────┘  │
│        │                        │
│        ▼                        │
│  ┌──────────────────────────┐   │
│  │ ui/ (ratatui TUI)        │   │
│  │  state.rs, render.rs,    │   │
│  │  events.rs, layout.rs    │   │
│  └──────────────────────────┘   │
└─────────────────────────────────┘
```

## Data Flow

1. `tmux list-panes -a` → `PaneInfo` structs
2. `/proc/<pid>/cmdline` + `/proc/<pid>/stat` → `ProcessInfo` structs
3. `PaneInfo` + `ProcessInfo` → `Task` (classified)
4. TUI renders tasks, handles input, sends commands back to tmux

## Module Responsibilities

- `models/`: Core data types (Pane, Process, Task)
- `services/`: Business logic (tmux commands, process inspection, task building)
- `ui/`: TUI rendering and event handling
- `config/`: Configuration from tmux options
