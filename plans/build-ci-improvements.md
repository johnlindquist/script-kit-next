# Build + CI Improvements Report

Date: 2026-02-07
Scope: `Cargo.toml`, `build.rs`, `.cargo/config.toml`, `.github/workflows/ci.yml`, `.github/workflows/release.yml`
Agent: `codex-ci-build`

## Executive Summary

The current setup is optimized for very fast PR feedback, but it trades away important safety checks by default and has a few avoidable compile/release inefficiencies. The highest-value improvements are:

1. tighten CI dependency graph and job gating so artifacts are only built after all validations pass,
2. remove stale/duplicate manifest entries and improve profile tuning for size/perf,
3. stop forcing single-threaded tests globally, and
4. reduce release workflow churn (tool install/caching consistency).

## Current State Findings

### CI coverage and gating

1. Default PR path skips lint/tests unless `full-ci` label is manually added.
- `fast-fmt` and `fast-compile` run for most PRs (`.github/workflows/ci.yml:32`, `.github/workflows/ci.yml:45`).
- `clippy`, `test`, and `typescript-tests` are gated behind `full-ci` or `main` (`.github/workflows/ci.yml:69`, `.github/workflows/ci.yml:80`, `.github/workflows/ci.yml:100`, `.github/workflows/ci.yml:120`).

2. `build-artifact` does not depend on TypeScript tests.
- It currently needs only `[fmt, clippy, test]` (`.github/workflows/ci.yml:152`), so it can produce/upload an artifact even if `typescript-tests` fails.

3. CI and release workflows lack cancellation of superseded runs.
- Neither workflow defines a `concurrency` block, so outdated runs continue consuming macOS minutes.

4. Release workflow always reinstalls `cargo-bundle` from git.
- `cargo install cargo-bundle ...` on every tag (`.github/workflows/release.yml:28-29`) with no dedicated tool cache step.

5. Bun is pinned to `latest` in CI.
- `bun-version: latest` (`.github/workflows/ci.yml:135`) hurts reproducibility for flaky SDK tests.

### Cargo/build configuration

1. Global single-threaded tests are forced for all `cargo test` runs.
- `.cargo/config.toml` sets `RUST_TEST_THREADS = "1"` globally (`.cargo/config.toml:6-7`).
- This improves stability for known syntect teardown races, but slows local iteration and any harness respecting this env var.

2. Build dependency appears stale.
- `dirs` is listed under `[build-dependencies]` (`Cargo.toml:171-172`) but `build.rs` does not use it.

3. Duplicate `tempfile` declaration.
- `tempfile` is in both `[dependencies]` (`Cargo.toml:146`) and `[dev-dependencies]` (`Cargo.toml:168`).

4. No explicit release profile tuning.
- `Cargo.toml` defines no `[profile.release]`, leaving default codegen behavior and symbols.

5. macOS-only crates are in global `[dependencies]`.
- Examples: `cocoa`, `core-graphics`, `core-video`, `objc`, `macos-accessibility-client`, `smappservice-rs` (`Cargo.toml:34-37`, `Cargo.toml:67`, `Cargo.toml:141`).
- This increases resolution/build work in non-mac contexts and weakens portability.

### build.rs details

1. Git hash invalidation is narrow.
- `build.rs` uses `cargo:rerun-if-changed=.git/HEAD` (`build.rs:31`) plus a `git rev-parse` command (`build.rs:10-12`).
- This can miss some ref changes in worktree/packed-ref scenarios.

## Prioritized Recommendations

## P0 (high impact, low risk)

1. Make artifact generation depend on all validation jobs.
- Change `needs` to include `typescript-tests` in `.github/workflows/ci.yml`.
- Expected impact: prevents publishing green-looking artifacts from partially failing CI.

2. Add workflow `concurrency` cancellation.
- Add to both `ci.yml` and `release.yml`:
```yaml
concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true
```
- Expected impact: fewer wasted macOS runner minutes and faster signal on force-pushes.

3. Pin Bun to a tested major/minor.
- Replace `bun-version: latest` with an explicit version (for example `1.2.x`) in `.github/workflows/ci.yml`.
- Expected impact: improved determinism and lower test flake rate.

4. Remove stale/duplicate manifest entries.
- Drop `[build-dependencies] dirs` if no longer used.
- Remove duplicate `tempfile` in `[dev-dependencies]`.
- Expected impact: small compile graph reduction and less maintenance noise.

5. Cache/install `cargo-bundle` consistently in release workflow.
- Mirror the cache pattern already used in CI build-artifact job (`.github/workflows/ci.yml:169-178`) inside `release.yml`.
- Expected impact: faster/reliable tag builds.

## P1 (medium effort, measurable wins)

1. Add explicit release profile tuning for binary size.
- Recommended starting point:
```toml
[profile.release]
lto = "thin"
codegen-units = 1
strip = "symbols"
```
- Validate launch behavior and signing/notarization unchanged.

2. Move macOS-only dependencies into target-specific dependency tables.
- Use `[target.'cfg(target_os = "macos")'.dependencies]` for mac-only crates.
- Expected impact: cleaner cross-platform graph, less accidental non-mac breakage.

3. Replace global `RUST_TEST_THREADS=1` with scoped serialization.
- Keep parallel test defaults.
- Mark only problematic tests/modules serialized (or use nextest profile overrides).
- Expected impact: much faster local `cargo test` while preserving stability.

4. Strengthen PR checks without losing speed.
- Keep fast path, but add a lightweight smoke test job on PRs (for example `cargo test --lib --quiet` or a curated `nextest` subset).
- Goal: catch obvious runtime regressions before merge, without always paying full CI cost.

## P2 (larger refactors)

1. Introduce feature slicing for expensive subsystems.
- Potential feature flags: tray/menu bar, OCR backend, notes DB, AI providers.
- Use default feature set for app behavior, plus lean CI/dev profiles (`--no-default-features` variants where possible).

2. Deduplicate workflow setup with reusable actions.
- Rust toolchain/cache/sccache scaffolding is repeated across jobs and files.
- Create a reusable workflow/composite action for consistency and faster maintenance.

3. Add scheduled build telemetry.
- Weekly `cargo build --release --timings` artifact and optional `cargo bloat` snapshots.
- Enables data-backed decisions instead of one-off impressions.

## Suggested Rollout Plan

1. Week 1 quick wins
- Add `concurrency` blocks.
- Gate `build-artifact` on `typescript-tests`.
- Pin Bun version.
- Remove stale `build-dependency` and duplicate `tempfile`.
- Add `cargo-bundle` cache/install optimization to `release.yml`.

2. Week 2 measurable optimizations
- Add release profile tuning.
- Move macOS-only dependencies to target-specific sections.
- Replace global test-thread override with targeted serialization.

3. Week 3 workflow hardening
- Introduce PR smoke test subset.
- Extract shared workflow setup.
- Add scheduled build timing report.

## Verification Commands For Follow-up Changes

Use these commands after each recommendation is implemented:

```bash
cargo check
cargo clippy --all-targets -- -D warnings
cargo test
```

For size and compile-time measurement:

```bash
cargo clean
cargo build --release --timings
ls -lh target/release/script-kit-gpui
```

Optional binary-size attribution:

```bash
cargo install cargo-bloat
cargo bloat --release -n 20
```

## Risks / Notes

1. Enabling stricter PR checks can increase CI time unless smoke coverage is carefully scoped.
2. `strip`/LTO changes can affect debugability and must be validated against code signing/notarization.
3. Changing test-thread policy requires identifying and fixing the underlying syntect-related race rather than masking it globally.
