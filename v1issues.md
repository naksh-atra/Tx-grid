# tmux-taskgrid v0.1.1 — Production Issues Tracker

## Round 1 Fixes (Committed)

| # | Issue | Severity | Status |
|---|-------|----------|--------|
| 1 | Runtime calculation wrong (clock ticks vs seconds) | CRITICAL | FIXED |
| 2 | PaneId inner field pub | CRITICAL | FIXED |
| 3 | No Ctrl-C handling | CRITICAL | FIXED |
| 4 | Terminal cleanup on panic | CRITICAL | FIXED |
| 5 | get_tmux_option stderr leak | HIGH | FIXED |
| 7 | scroll_offset desync after filter | HIGH | FIXED |
| 10 | Missing process states (Stopped, DiskSleep, Idle) | MEDIUM | FIXED |
| 11 | Unbounded /proc cmdline read | MEDIUM | FIXED |
| 15 | bash-specific syntax in taskgrid.tmux | MEDIUM | FIXED |
| 17 | thiserror unused dependency | LOW | FIXED |

## Round 2 Fixes (Committed)

| # | Issue | Severity | Status |
|---|-------|----------|--------|
| D-1 | No cargo audit in CI | MEDIUM | FIXED |
| S-1 | Unsanitized shell variables in open_taskgrid.sh | HIGH | FIXED |
| C-2 | No PATH validation for tmux binary | HIGH | FIXED |
| Q-2 | No filter text length limit | LOW | FIXED |

## Remaining (Deferred to v0.2.0)

| # | Issue | Severity | Status |
|---|-------|----------|--------|
| D-2 | Pinned GitHub Actions to SHAs | LOW | DEFERRED |
| D-3 | Unmaintained release action | MEDIUM | DEFERRED |
| D-4 | Release lacks SLSA attestation | LOW | DEFERRED |
| S-2 | bash/sh inconsistency in open_taskgrid.sh | LOW | DEFERRED |
| C-1 | unsafe libc::sysconf call | MEDIUM | ACCEPTABLE |
| C-3 | JSON output data sanitization | LOW | DEFERRED |
| C-4 | No rate limiting on tmux commands | LOW | DEFERRED |
| Q-1 | Serialize exposes internal fields | LOW | DEFERRED |
| Q-3 | ConfirmAction extensibility | LOW | ACCEPTABLE |
| Q-4 | Test cleanup of tmux state | MEDIUM | DEFERRED |
| #8 | Restart sends literal "Up" not arrow key | HIGH | NEEDS TESTING |
