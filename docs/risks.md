# Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| Incorrect process attribution from `pane_pid` | Wrong task info displayed | Graceful fallback to "unknown" |
| Restart unreliable across shells | Task doesn't restart | Document as "best-effort"; show clear error |
| Popup sizing varies across terminals | UI clipped or too small | Use percentage-based sizing |
| macOS process inspection differences | Missing or wrong data | Linux-first; macOS fallback planned |
| tmux version drift | Popup unavailable | Version check at startup with clear message |
| Terminal rendering quirks | Garbled UI | Standard ratatui widgets, tested sizes |
| User expects full task manager | Disappointed by scope | Clear README on what this is/isn't |

## Assumptions

- tmux 3.2+ is installed and running
- `/proc` filesystem is available (Linux)
- Binary is on `$PATH` or called via full path
- User is inside a tmux session
