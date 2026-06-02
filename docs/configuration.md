# Configuration

All configuration is via tmux options (set in `~/.tmux.conf`).

| Option | Default | Description |
|--------|---------|-------------|
| `@taskgrid-popup-width` | `80%` | Popup width (percentage or columns) |
| `@taskgrid-popup-height` | `60%` | Popup height (percentage or rows) |
| `@taskgrid-runtime-threshold` | `5` | Minimum runtime in seconds |
| `@taskgrid-confirm-kill` | `1` | Confirm before killing panes |
| `@taskgrid-refresh-interval` | `5` | Auto-refresh interval in seconds |

## Example

```
set -g @taskgrid-popup-width "90%"
set -g @taskgrid-popup-height "70%"
set -g @taskgrid-confirm-kill "0"
set -g @taskgrid-refresh-interval "10"
```
