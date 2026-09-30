# AGENTS.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

Only what the source cannot tell you is recorded here; the index points at where to look.

## Checks

The Stop hook (`.claude/hooks/stop.sh`) blocks finishing until these pass, so run the same commands:

```
sumi fmt --check
sumi verify
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
```

The Linux-only tests (watch loop, signals, cross-filesystem moves) skip on other systems; run them on macOS with `docker compose run --rm test`. The Stop hook skips the two `sumi` commands when `sumi` is not installed; CI does not.

`.claude/hooks/edit.sh` formats edited `.rs` files (edition 2024) and, through `sumi fmt`, the files under `.spec/`.

## Constraints

- **Rust version lives in two places**: `rust-toolchain.toml` and `ARG RUST_VERSION` in `Dockerfile`. Change both together.
- **Releases are driven by release-please**: only Conventional Commits (`feat:`, `fix:`) produce a Release PR; never bump the version or tag by hand.
- **The binaries and the version-tagged image are published inside `release.yml`**: tags created with `GITHUB_TOKEN` do not trigger other workflows, so a separate tag-triggered workflow would never run.
- **GitHub Actions**: pin every action to a full commit SHA with a version comment. `actions/*`, `docker/*` and `googleapis/release-please-action` are approved; any other third-party action needs the user's approval first.
- **sumi is pinned by hand in `ci.yml`**: `SUMI_VERSION` and `SUMI_SHA256` change together, because nothing watches a release asset fetched over curl. The specification lives in `.spec/`; `sumi verify` and `sumi fmt --check` must pass.
- **`tests/examples.rs` reads its pipelines from `docs/design.md`**: keep each example as a `toml` block with a `[pipeline.<name>]` table, or the test stops finding it. The Docker build context excludes `docs/`, so this test runs on the host or in a mounted container, not inside `docker build`.
- **notify reports only `Close(Write)` on Linux for finished writes**: `translate` treats a create or data change as still writing, so a file settles only on close-write or move-in; `runner` is `cfg(target_os = "linux")` and its tests run in a Linux container.
- **A same-filesystem move on Linux is `renameat2(RENAME_NOREPLACE)`** (through `rustix`), not a hard link: a hard link reaches a watcher as a create with no close, so in-place renaming held its folder until the maximum wait. `tests/move.rs` MV-021 watches this.
- **A lost notification cannot be provoked on OrbStack by volume**: it sets `fs.inotify.max_queued_events` to 1,048,576 where a stock kernel has 16,384, so the kernel's own overflow is only witnessed by handing the queue the notification for it (`tests/queue.rs`).
- **A move that shares data is tested only where the temporary folder is mounted twice on a filesystem that can share**: `docker-compose.yml` mounts it again at `/scratch-again`, and OrbStack's volumes are btrfs. CI runs `cargo test` on the runner itself, where there is no second mount, so `tests/move.rs` MV-025 skips there and only the container run witnesses it.
- **Image runtime is `scratch`** with a static musl binary: no CA certificates, timezone data or shell. Adding HTTPS or similar needs a different final stage.

## Index

| Path | What is there |
|------|---------------|
| `src/` | Application source (Rust binary crate `auto-renamer`); modules are mapped in `docs/architecture.md` |
| `Dockerfile`, `.dockerignore` | Multi-stage image build with a `test` stage; `.dockerignore` is an allow-list |
| `docker-compose.yml` | `test` service: mounts the repo, two filesystems for cross-device tests, one of them mounted twice |
| `.github/workflows/ci.yml` | PR / main checks and an image build without push |
| `.github/workflows/release.yml` | release-please, Linux musl binaries, multi-arch ghcr.io image |
| `release-please-config.json`, `.release-please-manifest.json` | Versioning config (`release-type: rust`) |
| `.claude/settings.json`, `.claude/hooks/` | Hook registration and scripts |
| `docs/design.md`, `docs/cases.md`, `docs/architecture.md` | Design, test cases derived from it, module structure |
| `tests/` | Integration tests through the public API; each claims a behavior with `// @behavior ID` |
| `scripts/measure.sh` | Measures CPU time and memory of the watcher in a container; needs only docker |
| `.spec/`, `.sumi.json` | sumi specification: glossary, behavior, contract |
