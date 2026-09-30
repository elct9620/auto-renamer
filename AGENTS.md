# AGENTS.md

Guidance for Claude Code in this repository. Only what the source cannot tell is recorded here; the index points at where to look. The tool renames files, and any work is weighed by where it falls:

```
  source ──► watch ──► pipeline ──► move ──► target
             when       what name    put it there
             support    core         minimum
```

## Scope

Each part has a boundary; what lies past it belongs to the operating system and to whoever deploys the tool.

| Part | Ours | Not ours |
|---|---|---|
| pipeline | turning names into plans | — |
| watch | knowing a file is finished and which files belong together | — |
| move | the plan reached, nothing overwritten, no data lost | copy speed, sharing data, mounts, memory limits, owners and ACLs |

## Checks

`.claude/hooks/stop.sh` blocks finishing until the checks CI runs pass. Linux-only code (`runner`, and the tests of watching, signals and moves between filesystems) is compiled out on macOS, where the checks pass without running it. Run it in the test container:

```
docker compose -f docker-compose.test.yml run --rm sut
```

## Releases

release-please drives versions, so the rules below keep it working.

| Rule | Why |
|---|---|
| Only `feat:` and `fix:` produce a release | release-please reads Conventional Commits |
| Never bump the version or tag by hand | release-please owns both |
| Binaries and the version image are built in `release.yml` | tags made with `GITHUB_TOKEN` trigger no other workflow |

## Pinned Versions

Nothing updates these automatically, so each pair changes together.

| What | Where |
|---|---|
| Rust | `rust-toolchain.toml` and `ARG RUST_VERSION` in `Dockerfile` |
| sumi | `SUMI_VERSION` and `SUMI_SHA256` in `ci.yml` |
| GitHub Actions | a full commit SHA with a version comment |

Actions from `actions/*`, `docker/*` and `googleapis/release-please-action` are approved; any other needs the user's approval first.

## Test Coupling

Some tests depend on things outside their own file.

| Test | Depends on |
|---|---|
| `tests/examples.rs` | the `toml` pipelines in `docs/design.md` |
| kernel overflow in `tests/queue.rs` | a handed-in overflow notice |
| moves between mounts in `tests/move.rs` | the `mv` on PATH |

Editing a design example can break a test, and the image build leaves out `docs/`, so `tests/examples.rs` runs only on the host or in the mounted container. OrbStack queues 1,048,576 kernel events, so an overflow cannot be caused by volume there. CI runs the tests with the runner's GNU `mv`; only the test container uses the busybox `mv` the image ships.

## Index

Where to look for each part of the project.

| Path | What is there |
|------|---------------|
| `src/` | Application source; modules are mapped in `docs/architecture.md` |
| `Dockerfile`, `.dockerignore` | Image build with a `test` stage; `.dockerignore` is an allow-list |
| `docker-compose.test.yml` | `sut` service: the tests on Linux, the repo mounted |
| `.github/workflows/` | `ci.yml` checks and publishes `latest`; `release.yml` releases |
| `docs/design.md`, `docs/cases.md`, `docs/architecture.md` | Design, test cases derived from it, module structure |
| `tests/` | Integration tests; each claims a behavior with `// @behavior ID` |
| `scripts/measure.sh` | CPU time and memory of the watcher in a container |
| `.spec/`, `.sumi.json` | sumi specification: glossary, behavior, contract |
