# tmux-taskgrid v0.1.4 — Code Review & Security Audit

## Architecture Comparison: Our App vs tmux

### tmux architecture (for reference):
- **Server process**: Maintains all sessions, windows, panes. Single source of truth.
- **Client process**: Connects to server, renders UI, handles input.
- **Protocol**: Clients communicate with server via Unix socket.
- **Rendering**: Each client renders its own terminal. No shared UI state.
- **Panes**: PTY-backed, run independent shells. pane_pid → process tree.

### Our architecture:
- **Single binary**: No server/client split. Runs as a standalone process.
- **Data collection**: Calls `tmux list-panes -a -F ...` to enumerate panes.
- **UI**: Renders via ratatui/crossterm in a tmux popup or pane.
- **State**: In-memory Vec<Task>, refreshed on timer.
- **Actions**: Calls tmux commands (select-pane, kill-pane, send-keys).

### Key architectural differences:
1. **No server**: We're a stateless client that queries tmux on each refresh. This is simpler but means we can't react to tmux events in real-time.
2. **Single process**: No persistence between runs. Each invocation rebuilds the task list.
3. **Polling-based**: 5-second timer refreshes all pane data. tmux itself is event-driven.
4. **No session management**: We don't create/manage sessions, only observe and act on panes.

---

## CRITICAL Issues

### C1: `restart_task` still exists but `r` key now calls it (should be rename)
- **File**: `src/main.rs:325-351`
- **Issue**: The `restart_task` function was supposed to be replaced with rename, but the code still has both the restart function AND the old `r` key binding calling `restart_task` instead of `enter_rename_mode`.
- **Impact**: Pressing `r` does restart (broken C-c + Up+Enter) instead of rename.
- **Fix**: Replace `restart_task(app)` with `app.enter_rename_mode()` in the key handler, remove `restart_task` function.

### C2: Pane classification doesn't match tmux's concept
- **File**: `src/models/task.rs:47-83`
- **Issue**: We classify `Running | Sleeping` as "running" for non-shell processes. But tmux considers a pane "active" based on `pane_activity` flag, not process state. A pane running `sleep 3600` (Sleeping) should not show as "running work".
- **Impact**: Idle sleep/bash processes appear as "running" even when no real work is happening.
- **Fix**: Check `pane_activity` tmux flag, or use `pane_last_activity` timestamp to determine if a pane has been idle.

### C3: `build_tasks` runs process inspection for every pane on every tick
- **File**: `src/models/task.rs:87-103`
- **Issue**: Every 5 seconds, we spawn a `/proc/<pid>/cmdline` read for EVERY pane. With 50 panes, that's 50 file reads per tick.
- **Impact**: Unnecessary I/O load. `/proc` reads are fast but wasteful at scale.
- **Fix**: Cache process info, only re-inspect panes whose PID has changed or that are new.

### C4: No handling of tmux command failures in UI actions
- **File**: `src/main.rs:250-258` (jump), `src/main.rs:286-298` (kill), `src/main.rs:291-300` (rename)
- **Issue**: When `select_pane`, `kill_pane`, or `select_pane_by_id` fails, the error is silently swallowed with `.ok()`. Users get no feedback.
- **Impact**: User presses Enter to jump, nothing happens, no error shown.
- **Fix**: Show error in status message area. E.g., `app.set_status(format!("jump failed: {}", e))`.

### C5: `select_pane_by_id` doesn't actually select the pane for renaming
- **File**: `src/services/tmux_service.rs` (missing function)
- **Issue**: The rename handler calls `select_pane_by_id` which calls `select-pane -t <id> -T <name>`. But this jumps the CURRENT client to that pane. The user loses their popup context.
- **Impact**: After renaming, the user is focused on the renamed pane instead of the popup.
- **Fix**: Use `select-pane -T <name>` without `-t` to just set the title, or don't call select-pane at all — use `tmux select-pane -t <id>` separately.

---

## HIGH Issues

### H1: Notes file has no locking
- **File**: `src/ui/state.rs` (load_notes/save_notes)
- **Risk**: If multiple tmux-taskgrid instances run simultaneously (e.g., in different panes), they can corrupt the notes file.
- **Fix**: Use file locking (`flock`) or write to per-instance temp files.

### H2: Unbounded `notes_text` growth
- **File**: `src/ui/state.rs`
- **Issue**: Notes are limited to 1024 chars per pane, but the notes file has no size limit. Over time with many panes, it grows unbounded.
- **Impact**: File could become large with abandoned pane notes.
- **Fix**: Add a cleanup function, or limit total file size, or use a proper database (sled/sqlite).

### H3: Pane notes survive pane deletion
- **Issue**: When a pane is killed, its notes remain in `~/.tmux-taskgrid-notes`.
- **Impact**: Stale data accumulates.
- **Fix**: Add a `--cleanup-notes` flag to remove entries for panes that no longer exist, or clean up on each load.

### H4: `load_notes`/`save_notes` are in `state.rs` (wrong module)
- **File**: `src/ui/state.rs`
- **Issue**: File I/O doesn't belong in the UI state module. Should be in a separate `notes.rs` or `storage.rs` module.
- **Impact**: Mixing concerns, harder to test.
- **Fix**: Move to `src/storage/notes.rs`.

### H5: `open_taskgrid.sh` uses `bash` despite being `#!/usr/bin/env sh`
- **File**: `scripts/open_taskgrid.sh`
- **Issue**: The shebang says `sh` but uses bashisms like `&>/dev/null`.
- **Impact**: May fail on systems where `sh` is not bash (e.g., dash on Debian).
- **Fix**: Change shebang to `#!/usr/bin/env bash`.

### H6: Differences from PSMux
- **Issue**: Our app relies on `display-popup -E` which provides a full PTY. PSMux's `display-popup` doesn't support interactive TUI apps.
- **Impact**: App works in tmux but popup doesn't work in PSMux. Only "run in pane" mode works in PSMux.
- **Fix**: Document this limitation, or add a `--no-popup` mode that creates a new window instead.

---

## MEDIUM Issues

### M1: `_modifiers` parameter silently drops Ctrl-C handling
- **File**: `src/main.rs:216`
- **Issue**: Earlier we added `Ctrl-C` handling but the `_modifiers` is prefixed with `_` to suppress warnings. The `Ctrl-C` check was only added in match arms but not consistently.
- **Impact**: `Ctrl-C` may not quit in all modes.
- **Fix**: Rename `_modifiers` to `modifiers` and handle `Ctrl+C` in all mode matches.

### M2: `current_pane_id` exclusion is fragile
- **File**: `src/models/task.rs:94-97`
- **Issue**: We exclude the current pane by comparing `pane_id.0 != current_pane_id`. But `TMUX_PANE` format is `%123` and tmux pane IDs are also `%123`. This works but is a narrow string comparison.
- **Impact**: If formats differ (e.g., different tmux versions), the popup pane shows up in its own list.
- **Fix**: Add a comment documenting the format assumption, or use a more robust comparison.

### M3: No error handling for missing `HOME` env var
- **File**: `src/main.rs:57` (install_keybinding), `src/ui/state.rs` (notes_file_path)
- **Issue**: `install_keybinding` uses `std::env::var("HOME").context(...)` which panics if HOME is not set.
- **Impact**: Crash in minimal environments (e.g., containers, systemd services).
- **Fix**: Fall back to `~/.tmux.conf` or `/tmp/`.

### M4: `tmux_command` doesn't handle tmux socket path
- **File**: `src/services/tmux_service.rs:8-9`
- **Issue**: `Command::new("tmux")` assumes tmux uses the default socket. Users with custom sockets (`-S` flag) will fail.
- **Impact**: App doesn't work for users with non-standard tmux setups.
- **Fix**: Read `TMUX` env var which contains the socket path.

---

## LOW Issues

### L1: Duplicate `tmux_command` in `get_tmux_option`
- **File**: `src/services/tmux_service.rs:52-54`
- **Issue**: `get_tmux_option` calls `Command::new("tmux")` directly instead of using the existing `tmux_command` helper.
- **Impact**: Inconsistent error handling, silent stderr.
- **Fix**: Refactor `get_tmux_option` to use `tmux_command`.

### L2: Tests don't clean up after themselves comprehensively
- **Issue**: Notes file cleanup only happens in `test_notes_mode`, not in other tests that might create notes.
- **Impact**: Test pollution, flaky tests.
- **Fix**: Add a test fixture that cleans up the notes file.

### L3: Binary version in `cli.rs` is hardcoded
- **File**: `src/cli.rs:6`
- **Issue**: Version `"0.1.3"` is hardcoded. Should use `env!("CARGO_PKG_VERSION")`.
- **Impact**: Version string goes out of sync with Cargo.toml.
- **Fix**: Use `clap::crate_version!()` or `env!("CARGO_PKG_VERSION")`.

### L4: No `--version` short flag (`-V`)
- **Issue**: Only `--version` works, not `-V` which is standard.
- **Fix**: Add `#[arg(short = 'V', long)]` or use clap's built-in version handling.

---

## Summary

| Severity | Count | Critical Action |
|----------|-------|-----------------|
| CRITICAL | 5 | Fix `r` → rename, fix swallowed errors, fix pane selection for rename |
| HIGH | 6 | Add file locking, fix PSMux compatibility note, fix bash/sh mismatch |
| MEDIUM | 4 | Fix Ctrl-C handling, add HOME fallback |
| LOW | 4 | Use CARGO_PKG_VERSION, refactor get_tmux_option |

**Biggest architectural gap**: We're a polling stateless client while tmux is event-driven. For v0.2.0, consider using tmux hooks (`set-hook`) to get notified of pane changes instead of polling every 5 seconds.
