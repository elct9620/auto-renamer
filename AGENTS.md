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
| playground | the core's own answers, on a virtual tree | timing of watching, how a filesystem moves a file |

## Checks

`.claude/hooks/stop.sh` blocks finishing until the checks CI runs pass, and until React Doctor finds no error in `playground/`; only the hook runs React Doctor. Linux-only code (`runner`, and the tests of watching, signals and moves between filesystems) is compiled out on macOS, where the checks pass without running it. The playground's checks need `wasm-bindgen`, so only CI runs them; its E2E tests (`pnpm e2e`) drive the built page in the Google Chrome the machine already has. Run the Linux code in the test container:

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
| Every crate keeps a written version, and none lists `"."` as a member | the rust strategy raises all members together and resolves each listed one |

## Pinned Versions

Nothing updates these automatically, so each pair changes together.

| What | Where |
|---|---|
| Rust | `rust-toolchain.toml` and `ARG RUST_VERSION` in `Dockerfile` |
| sumi | `SUMI_VERSION` and `SUMI_SHA256` in `ci.yml` |
| wasm-bindgen | the exact `wasm-bindgen` in `crates/wasm/Cargo.toml`, and `WASM_BINDGEN_VERSION` and `WASM_BINDGEN_SHA256` in `ci.yml` |
| pnpm | `packageManager` in `playground/package.json` and `PNPM_VERSION` in `ci.yml` |
| React Doctor | the `react-doctor@` version in `.claude/hooks/stop.sh` |
| GitHub Actions | a full commit SHA with a version comment |

Actions from `actions/*`, `docker/*` and `googleapis/release-please-action` are approved; any other needs the user's approval first.

## Test Coupling

Some tests depend on things outside their own file.

| Test | Depends on |
|---|---|
| `tests/examples.rs` | the `toml` pipelines in `docs/design.md`, which `crates/core/src/config/series.toml` must match |
| kernel overflow in `tests/queue.rs` | a handed-in overflow notice |
| moves between mounts in `tests/move.rs` | the `mv` on PATH |
| MV-028 in `tests/move.rs` | the 1 MiB tmpfs at `/small` of the test container |
| `playground/src/config.test.ts`, `tree.test.ts`, `i18n.test.ts`, `settings.test.ts` | the module `pnpm wasm` builds, whose `stages()` lists the stages `declare.rs` reads |
| `playground/src/steps.test.ts` | the steps a simulation by the module `pnpm wasm` builds reports |
| `playground/src/examples.test.ts` | the outputs `docs/cases.md` gives each example, simulated by the module `pnpm wasm` builds |
| `playground/e2e/page.e2e.ts` | the example the page opens with, `FIRST` in `examples.ts`, and `pnpm build` having run |
| DEC-035 in `tests/declaration.rs` | debug assertions, which a release build drops |

Editing a design example can break a test, and the image build leaves out `docs/`, so `tests/examples.rs` runs only on the host or in the mounted container. OrbStack queues 1,048,576 kernel events, so an overflow cannot be caused by volume there. CI runs the tests with the runner's GNU `mv`; only the test container uses the busybox `mv` the image ships.

## Index

Where to look for each part of the project.

| Path | What is there |
|------|---------------|
| `crates/core/` | What decides about files: pipeline, configuration, effects through a `Tree` |
| `crates/wasm/` | The core for the playground, over a virtual tree |
| `src/` | The CLI: filesystem, watcher, runner; modules are mapped in `docs/architecture.md` |
| `playground/` | The GitHub Pages page: pnpm, Vite, React Flow, Tailwind CSS with shadcn/ui, i18next |
| `Dockerfile`, `.dockerignore` | Image build with a `test` stage; `.dockerignore` is an allow-list, `crates/wasm` by manifest only |
| `docker-compose.test.yml` | `sut` service: the tests on Linux, the repo mounted |
| `.github/workflows/` | `ci.yml` checks, publishes `latest` and the playground; `release.yml` releases |
| `docs/design.md`, `docs/cases.md`, `docs/architecture.md` | Design, test cases derived from it, module structure |
| `tests/` | Integration tests; each claims a behavior with `// @behavior ID` |
| `scripts/measure.sh` | CPU time and memory of the watcher in a container |
| `.spec/`, `.sumi.json` | sumi specification: glossary, behavior, contract |
