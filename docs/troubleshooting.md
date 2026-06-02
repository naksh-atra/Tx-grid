# Troubleshooting

## Popup does not open

- Ensure tmux 3.2+: `tmux -V`
- Ensure you're inside tmux: echo `$TMUX`
- Try running manually: `tmux display-popup -E "tmux-taskgrid --check"`

## Binary not found

- Ensure `tmux-taskgrid` is on `$PATH`
- Try full path: `set -g @taskgrid-path "/path/to/tmux-taskgrid"`

## No tasks shown

- Run `tmux-taskgrid --check` to verify pane discovery
- Check that tmux server is running

## Wrong command detected

- This can happen when a pane runs nested processes
- We show the process at the top of the pane's process tree

## Restart not working

- Restart is best-effort (sends C-c, then Up+Enter)
- Works best in shells with history enabled
- Won't work if the shell doesn't have the previous command in history

## tmux version incompatible

- tmux-taskgrid requires tmux 3.2+ for `display-popup`
- Upgrade your tmux or use an older binding method

## macOS limitations

- Process inspection may differ on macOS
- Linux-first; macOS is not yet tested
