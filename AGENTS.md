# AGENTS.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

Only what the source cannot tell you is recorded here; the index points at where to look.

## Checks

The Stop hook (`.claude/hooks/stop.sh`) blocks finishing until these pass, so run the same commands:

```
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
```

`.claude/hooks/edit.sh` formats edited `.rs` files automatically (edition 2024).

## Constraints

- **Rust version lives in two places**: `rust-toolchain.toml` and `ARG RUST_VERSION` in `Dockerfile`. Change both together.
- **Releases are driven by release-please**: only Conventional Commits (`feat:`, `fix:`) produce a Release PR; never bump the version or tag by hand.
- **Binary and image publishing stays inside `release.yml`**: tags created with `GITHUB_TOKEN` do not trigger other workflows, so a separate tag-triggered workflow would never run.
- **GitHub Actions**: pin every action to a full commit SHA with a version comment. `actions/*`, `docker/*` and `googleapis/release-please-action` are approved; any other third-party action needs the user's approval first.
- **Image runtime is `scratch`** with a static musl binary: no CA certificates, timezone data or shell. Adding HTTPS or similar needs a different final stage.

## Index

| Path | What is there |
|------|---------------|
| `src/` | Application source (Rust binary crate `auto-renamer`) |
| `Dockerfile`, `.dockerignore` | Multi-stage image build; `.dockerignore` is an allow-list |
| `.github/workflows/ci.yml` | PR / main checks and an image build without push |
| `.github/workflows/release.yml` | release-please, Linux musl binaries, multi-arch ghcr.io image |
| `release-please-config.json`, `.release-please-manifest.json` | Versioning config (`release-type: rust`) |
| `.claude/settings.json`, `.claude/hooks/` | Hook registration and scripts |
