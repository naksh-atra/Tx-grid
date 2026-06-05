# GitHub Rules & Conventions

## Security Rules

### NEVER commit secrets to git
- No tokens, passwords, API keys, or credentials in any file
- If a secret is accidentally committed, it must be removed from ALL of git history using `git filter-branch`, not just deleted from the current commit
- After removing secrets, force push all affected branches: `git push origin <branch> --force`
- Use `.gitignore` for all sensitive files from the start

### Sensitive files to always ignore
- `*token*`, `*secret*`, `*.key`, `*.pem`, `*credential*`
- Any file containing API keys, passwords, or tokens
- Environment files: `.env`, `.env.local`, `.env.production`

### Fine-grained PAT limitations
- Fine-grained tokens may NOT support HTTPS git push operations
- Use SSH (`git@github.com:user/repo.git`) for pushing instead of HTTPS with token in URL
- If HTTPS push is needed, use a classic PAT with `repo` scope

## Commit Conventions

### Commit message format
- Use conventional commit types: `feat`, `fix`, `refactor`, `test`, `chore`, `docs`, `style`
- NEVER use "auto" as a type
- Split unrelated changes into separate commits
- Subject line: imperative mood, lowercase, no period (e.g., "fix: resolve compile error")

### Author configuration
Always set explicit author flags for automated/cron commits:
```bash
git commit --author="naksh-atra <nakshastra.rajput@outlook.com>"
```

### Branch strategy
- `dev` — development branch, all feature work happens here
- `main` — production-ready, merge from dev after testing
- Tag releases on main: `git tag v0.1.0 && git push origin v0.1.0`

## Git Operations on WSL

### Caching issues with `/mnt/c/`
- Hermes `write_file` tool does NOT reliably persist files on WSL `/mnt/c/` paths
- Use `os.open`/`os.write` via Python `execute_code` for critical file writes
- For complete file rewrites, use bash heredoc: `cat > file << 'EOF' ... EOF`
- Always verify writes by reading the file back

### Line endings
- WSL may introduce `\r\n` line endings on Windows-hosted files
- Fix with: `sed -i 's/\r$//' <file>`
- Add `* text=auto eol=lf` to `.gitattributes`

### Staging files
- `git add -A` is slow on `/mnt/c/` — stage specific files instead: `git add file1 file2`
- `git commit --no-verify` skips pre-commit hooks when needed

## GitHub Actions

### Workflow triggers
- CI (test/clippy/fmt) runs on push to `main`/`dev` and PRs to `main`
- Quality checks (shellcheck/markdownlint) run on push to `main`
- Release workflow runs on tag push matching `v*`

### CI failures
- Fix formatting: `cargo fmt`
- Fix clippy: address all warnings with `cargo clippy --all-targets --all-features -- -D warnings`
- Fix markdownlint: check for long lines, missing language on code blocks, trailing whitespace
- Fix shellcheck: proper quoting, use `set -euo pipefail` carefully

### Release workflow
Multi-platform builds create binaries for:
- Linux x64 (`tmux-taskgrid-linux-x64`)
- Linux ARM64 (`tmux-taskgrid-linux-arm64`)

Release is created automatically via `softprops/action-gh-release@v2` with artifacts uploaded.

## Crash Course: Key Git Commands

```bash
# Force push (overwrite remote history)
git push origin <branch> --force

# Remove file from entire git history
git filter-branch --force --index-filter \
  'git rm --cached --ignore-unmatch <file>' \
  --prune-empty --tag-name-filter cat -- HEAD

# Merge branch with no fast-forward
git merge <branch> --no-ff -m "merge: <description>"

# Stage specific files (faster on WSL)
git add file1.rs file2.sh

# Commit with explicit author
git commit --author="Name <email>" -m "type: description"

# Tag and push
git tag v1.0.0
git push origin v1.0.0

# View branch history
git log --oneline --all --graph

# Check what files are tracked
git ls-files
```
