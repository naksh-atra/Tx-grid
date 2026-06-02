# UX Contract

## Keymap

| Key | Action |
|-----|--------|
| `j`/`↓` | Move selection down |
| `k`/`↑` | Move selection up |
| `g`/`Home` | Jump to first task |
| `G`/`End` | Jump to last task |
| `PgUp` | Page up |
| `PgDn` | Page down |
| `/` | Enter filter mode |
| `s` | Cycle sort order |
| `Enter` | Jump to pane |
| `x` | Kill pane (with confirmation) |
| `r` | Restart task |
| `q`/`Esc` | Quit |

## Default Behavior

- Tasks sorted by runtime (longest first)
- No filter active
- Selection at top of list

## Actions

- **Kill**: Requires confirmation by default (toggleable via config)
- **Jump**: Closes popup and focuses selected pane
- **Restart**: Best-effort (C-c, then Up+Enter)

## Empty States

- **No panes**: "No panes found. Press q to quit."
- **No filter matches**: "No tasks match the filter. Press Esc to clear filter."

## Error States

- **tmux not found**: Exit with error message
- **tmux too old**: Exit with "tmux version X is too old; need 3.2+"
- **Action failure**: Show error in footer status area

## Refresh

- Polling mode: refreshes every 5 seconds
- Manual: no manual refresh in v1
