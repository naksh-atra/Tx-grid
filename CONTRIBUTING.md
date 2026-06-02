# Contributing to tmux-taskgrid

Thank you for your interest in contributing!

## Getting Started

1. Fork the repo
2. Create a feature branch: `git checkout -b feat/my-feature`
3. Make your changes
4. Run `cargo fmt`, `cargo clippy`, `cargo test`
5. Open a PR against `dev`

## Branch Naming

- `feat/*` — new features
- `fix/*` — bug fixes
- `docs/*` — documentation only

## Commit Convention

We use conventional commits:

- `feat: add pane discovery parser`
- `fix: handle tmux popup unsupported versions`
- `docs: expand installation guide`
- `test: add integration coverage for pane classification`
- `chore: enable clippy in CI`

## Code Standards

- All code must pass `cargo fmt` and `cargo clippy -- -D warnings`
- Tests must pass with `cargo test`
- Public APIs should be minimal and well-documented
- Prefer small, focused modules

## Code of Conduct

Be respectful and constructive in all interactions.
