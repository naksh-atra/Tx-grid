# Release Checklist

Before tagging a release:

- [ ] Update version in `Cargo.toml`
- [ ] Update `CHANGELOG.md`
- [ ] Run `cargo fmt --check`
- [ ] Run `cargo clippy --all-targets -- -D warnings`
- [ ] Run `cargo test --all-targets`
- [ ] Verify README install steps work
- [ ] Verify release notes draft
- [ ] Confirm CI passes on latest commit

## Tagging

```bash
git tag v0.1.0
git push origin v0.1.0
```

The release workflow will build binaries and publish automatically.
