# Manual Test Plan

## Setup

1. Start tmux: `tmux new-session -s test`
2. Create multiple panes with different workloads
3. Open taskgrid: `tmux-taskgrid`

## Test Cases

### T01: Popup opens
- **Steps**: Run `display-popup -E "tmux-taskgrid"` from tmux
- **Expected**: Popup opens with task list showing current panes

### T02: Navigation
- **Steps**: Press j/k, g/G, PgUp/PgDn
- **Expected**: Selection moves correctly, scrolls as needed

### T03: Filter
- **Steps**: Press `/`, type "vim", press Enter
- **Expected**: Only panes running vim are shown

### T04: Sort cycle
- **Steps**: Press `s` multiple times
- **Expected**: Sort order cycles: runtime → session → state → runtime

### T05: Empty state
- **Steps**: Run with filter that matches nothing
- **Expected**: "No tasks match the filter" message

### T06: Jump to pane
- **Steps**: Select a pane, press Enter
- **Expected**: Popup closes, selected pane is focused

### T07: Kill pane
- **Steps**: Select a pane, press `x`, press `y`
- **Expected**: Pane is closed, task list updates

### T08: Cancel kill
- **Steps**: Select a pane, press `x`, press `n`
- **Expected**: No action, confirmation dismissed

### T09: Restart task
- **Steps**: Select a pane with a long-running command, press `r`
- **Expected**: C-c sent, then Up+Enter; footer shows "restarted"

### T10: Quit
- **Steps**: Press `q`
- **Expected**: Popup closes, returns to previous pane

### T11: Narrow terminal
- **Steps**: Resize terminal to 60 columns
- **Expected**: Table truncates gracefully, no crashes

### T12: Wide terminal
- **Steps**: Resize terminal to 200 columns
- **Expected**: Table uses available width
