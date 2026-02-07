# Dependency Improvements Report

Date: 2026-02-07
Scope: `Cargo.toml`, `Cargo.lock`
Agent: `codex-cargo-deps`

## Executive Summary

- Security: `cargo audit` found **2 active vulnerabilities** in the lockfile (`bytes 1.11.0`, `time 0.3.44`) and several warning-level advisories (including `lru 0.12.5` unsoundness and unmaintained GTK3 stack on Linux).
- Cleanup: `cargo machete` flagged **5 likely unused direct dependencies**: `clap`, `inventory`, `ropey`, `rust-i18n`, `time`.
- Upgrades: multiple direct dependencies are behind latest compatible releases.
- Size/compile opportunities: current feature selection enables heavier stacks than needed in a few places (`syntect` with Oniguruma, `rusqlite` default cache/hashlink, `ureq` default `gzip`).

## What I Ran

```bash
cargo audit
cargo update --workspace --dry-run --verbose
cargo machete --with-metadata
cargo tree -d
cargo tree -e features -i syntect
cargo tree -e features -i rusqlite
cargo tree -e features -i ureq
cargo check
cargo clippy --all-targets -- -D warnings
cargo test
```

Notes:
- `cargo outdated --root-deps-only` was not usable in this repo due a `cocoa` solver mismatch between transient versions.
- `cargo check` passed.
- `cargo clippy`/`cargo test` currently fail from existing unrelated workspace issues (see Verification section).

## 1) Security Advisories (Highest Priority)

### Active vulnerabilities from `cargo audit`

1. `RUSTSEC-2026-0007` (`bytes 1.11.0`): integer overflow in `BytesMut::reserve`
- Fix: `>= 1.11.1`
- Path: transitive via `gpui/http_client` and `ureq`

2. `RUSTSEC-2026-0009` (`time 0.3.44`): DoS via stack exhaustion
- Fix: `>= 0.3.47`
- Path: direct + transitive (`tracing-subscriber`, `cookie_store`)

### Warning-level advisories to track

- `RUSTSEC-2026-0002` (`lru 0.12.5`) unsound `IterMut`.
- `RUSTSEC-2025-0052` (`async-std`) unmaintained (transitive through `gpui/http_client`).
- GTK3 crate set (`gtk`, `gdk`, `atk`, etc.) marked unmaintained through `tray-icon` Linux path.
- `bincode 1.3.3` unmaintained, pulled by `syntect`.

### Immediate remediation commands

```bash
cargo update -p bytes --precise 1.11.1
cargo update -p time --precise 0.3.47
```

Then re-run:

```bash
cargo audit
```

## 2) Likely Unused Direct Dependencies

`cargo machete --with-metadata` reports these as unused:
- `clap` (declared `Cargo.toml:115`)
- `inventory` (declared `Cargo.toml:114`)
- `ropey` (declared `Cargo.toml:53`)
- `rust-i18n` (declared `Cargo.toml:127`)
- `time` (declared `Cargo.toml:48`)

Quick corroboration:
- `src/bin/storybook.rs` uses manual `std::env::args` parsing, not `clap`.
- `src/storybook/registry.rs` explicitly references moving away from `inventory`.
- no code hits for `ropey::`, `inventory::`, `rust_i18n`, or direct `time::` imports.

Recommendation:
1. Remove these dependencies from `Cargo.toml`.
2. Run `cargo check` and `cargo test`.
3. If any are macro/reflection false positives, add only those to `[package.metadata.cargo-machete].ignored` with comments.

## 3) Outdated Dependencies (Direct)

From `cargo update --workspace --dry-run --verbose`, direct deps with newer versions available include:

- Patch/minor-risk bucket: `anyhow`, `chrono`, `clap`, `cocoa`, `core-foundation`, `filetime`, `libc`, `macos-accessibility-client`, `regex`, `serde_json`, `thiserror`, `tray-icon`, `uuid`, `xcap`.
- Potentially breaking bucket (major API/behavior checks needed): `dirs` (`5 -> 6`), `lru` (`0.12 -> 0.16`), `pulldown-cmark` (`0.12 -> 0.13`), `shellexpand` (`2 -> 3`), `sysinfo` (`0.37 -> 0.38`), `tree-sitter` (`0.25 -> 0.26`), `ureq` (`3.1 -> 3.2`), `tiny-skia` (`0.11 -> 0.12`), `core-video` (`0.4 -> 0.5`).

Recommendation:
- Do a two-wave upgrade:
  - Wave A (security + patch/minor low-risk)
  - Wave B (majors one-by-one with focused tests)

## 4) Feature-Flag Opportunities (Binary Size / Compile Impact)

### A) `rusqlite` currently enables default `cache/hashlink`

Current (`Cargo.toml:76`):
```toml
rusqlite = { version = "0.38", features = ["bundled"] }
```

Observed feature graph includes `default -> cache -> hashlink`.

If statement cache isn't needed, use:
```toml
rusqlite = { version = "0.38", default-features = false, features = ["bundled"] }
```

### B) `ureq` default features include `gzip`

Current (`Cargo.toml:138`) enables default features implicitly, which include `rustls` + `gzip`.

If gzip decoding isn't required:
```toml
ureq = { version = "3", default-features = false, features = ["json", "rustls"] }
```

### C) `syntect` uses Oniguruma (`regex-onig`)

Current (`Cargo.toml:40`) uses:
- `default-syntaxes`
- `default-themes`
- `regex-onig`

Potential lighter option (pure Rust regex backend):
```toml
syntect = { version = "5.2", default-features = false, features = ["default-syntaxes", "default-themes", "default-fancy"] }
```

Risk: syntax edge-case differences and possible performance regressions; needs rendering parity tests.

## 5) Duplicate Crate Families / Graph Noise

`cargo tree -d` shows notable duplication:
- `cocoa` (`0.24`, `0.26`)
- `core-foundation` (`0.9`, `0.10`)
- `core-graphics` (`0.22`, `0.23`, `0.24`)
- `notify` (`7`, `8`)
- `resvg`/`usvg` (`0.45`, `0.46`)
- `ropey` (`1.6`, `2.0.0-beta.1`)

Most are induced by `gpui` + `gpui-component` transitive stacks, but two actionable points here are:
- remove unused direct `ropey` (gets rid of one explicit root).
- align `notify` major version with `gpui-component` when practical to reduce duplicate builds.

## 6) Lighter Alternatives (Medium-Term)

- `lru` unsound advisory: either upgrade to `0.16.x` or replace with an actively maintained cache (e.g. `moka` for concurrent scenarios, `clru` for simple LRU).
- `syntect` backend: prefer fancy-regex path if behavior is acceptable.
- If future HTTP needs expand beyond simple JSON APIs, consider one consolidated HTTP stack rather than split client ecosystems across transitive graph.

## 7) Compile-Time Optimization Opportunities

1. Add explicit profile tuning in `Cargo.toml` (none currently defined):
```toml
[profile.release]
lto = "thin"
codegen-units = 1
strip = "debuginfo"
```

2. Move macOS-only dependencies to target-specific sections where possible:
- candidates include `cocoa`, `core-graphics`, `core-video`, `objc`, `macos-accessibility-client`, `smappservice-rs`.

3. Keep dependency graph lean by removing unused direct crates first; this provides immediate compile/link wins with minimal risk.

## 8) Recommended Execution Plan

1. Security-first lockfile refresh (`bytes`, `time`) and rerun `cargo audit`.
2. Remove unused direct dependencies identified by `cargo machete`.
3. Apply feature trimming for `rusqlite` and `ureq` (low-to-medium risk).
4. Trial `syntect` fancy-regex backend behind a feature flag; compare rendering output.
5. Upgrade majors one crate at a time with focused test passes.

## Verification Status

- `cargo check`: ✅ passed
- `cargo clippy --all-targets -- -D warnings`: ❌ failed due unrelated pre-existing workspace issues (duplicate `mod tests`, unused imports in generated action test files, missing `AiApp::message_body_content` symbol).
- `cargo test`: ❌ failed due the same pre-existing workspace issues.

No manifest or lockfile modifications were applied in this pass; this report is analysis + prioritized recommendations.
