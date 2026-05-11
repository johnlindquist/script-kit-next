# Distribution

Script Kit GPUI's current shipping path is a macOS app bundle built with `cargo-bundle`, verified locally with repo scripts, and published from GitHub Actions. The older cross-platform bundling roadmap is not the current durable contract.

## Local bundle path

The repo's explicit macOS bundle path is:

- `cargo build --release --bin script-kit-gpui`
- `cargo bundle --release --bin script-kit-gpui`
- `bash scripts/verify-macos-bundle.sh`

The resulting app lives at `target/release/bundle/osx/Script Kit.app`.

## Bundle metadata

The canonical bundle metadata lives in `Cargo.toml` under `[package.metadata.bundle.bin.script-kit-gpui]`.

That metadata currently defines the app name, bundle identifier, icons, minimum macOS version, URL scheme, bundled resources, and the `LSUIElement`-style agent-app plist extension.

## CI build artifact

The `CI` workflow on pushes to `main` builds the release binary, creates the macOS bundle, verifies bundle contents, ad-hoc signs the app, zips it, and uploads the archive as a short-lived artifact.

That is the current dev-build path. It is useful for download and testing, but it is not the notarized release path.

## Tagged release path

The `Release` workflow runs on `v*` tags and currently does this:

- runs `bash scripts/verify-release-version.sh` so a tag like `v1.5.0` fails fast if Cargo.toml still says `1.4.0`
- validates the repo gates with `bash scripts/verify.sh --skip-bundle` — gated cargo invocations now use `--locked` so a Cargo.lock drift fails the release before signing
- builds the release binary and macOS bundle against the toolchain pinned in `rust-toolchain.toml`; bun is pinned to an exact 1.2.x patch and `cargo-bundle` installs with `--locked`
- verifies the bundled app contents
- signs the app with the Developer ID certificate and `entitlements.plist`. The identity comes from the `APPLE_CODESIGN_IDENTITY` secret (with the legacy `Developer ID Application: John Lindquist (<TEAM>)` as a fallback). Frameworks are signed with `find -type d -name "*.framework"` and a separate file pass covers `*.dylib` / `*.so`.
- notarizes the zip with Apple's notary service
- staples the notarization ticket
- runs `spctl --assess --type execute` AFTER stapling — this step has no `|| echo` fallback so a Gatekeeper rejection fails the build
- generates `release-manifest.json` with the SHA256 + size of every shipped artifact. The manifest filename does not match `.zip` / `.dmg` / `.tar.gz`, so the in-app updater (asset-aware after [[src/updates.rs#pick_release]]) keeps choosing the zip as the download target while [[src/updates.rs#manifest_sha256_for]] is reserved for the future installer that will verify integrity.
- uploads the final `Script-Kit-macos.zip` AND `release-manifest.json` to the GitHub release

This is the current production distribution contract.

## Human-only gate

`make ship-check` is the full local ship gate for humans. It runs the full validation path plus bundle sanity checks.

AI agents should not run `make ship-check`; they should use `make verify` or narrower checks unless a human explicitly asks for packaging validation.

## Local dev target hygiene

`./dev.sh` protects local dev loops from silently growing `target/` past a disk-budget threshold.

On startup, `dev.sh` measures `target/` and runs `cargo clean` by default when the directory is larger than `SCRIPT_KIT_TARGET_CLEAN_THRESHOLD_GB`, which defaults to 50 GiB. Set `SCRIPT_KIT_TARGET_AUTO_CLEAN=0` to warn without cleaning when preserving a hot build cache matters more than disk use.

## Source files

These files define the current local, CI, and release packaging paths.

- [Cargo.toml](../Cargo.toml)
- [dev.sh](../dev.sh)
- [Makefile](../Makefile)
- [rust-toolchain.toml](../rust-toolchain.toml)
- [.github/workflows/ci.yml](../.github/workflows/ci.yml)
- [.github/workflows/release.yml](../.github/workflows/release.yml)
- [scripts/verify-macos-bundle.sh](../scripts/verify-macos-bundle.sh)
- [scripts/verify-release-version.sh](../scripts/verify-release-version.sh)
- [scripts/verify.sh](../scripts/verify.sh)
