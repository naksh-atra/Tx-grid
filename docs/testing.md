# Testing

## Unit Tests

Run with: `cargo test`

Tests cover:
- Pane parsing
- Process state parsing
- Task classification
- UI state management (navigation, filtering, sorting)
- Command formatting

## Integration Tests

Run with: `cargo test --test integration_tmux`

Requires tmux installed. Tests:
- Pane discovery from real tmux sessions
- Task building with mock process provider
- Multi-session scenarios

## Manual Test Plan

See `docs/manual-test-plan.md`.

## CI

GitHub Actions runs on every push/PR:
- `cargo fmt --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test --all-targets`
- `cargo build --release`
