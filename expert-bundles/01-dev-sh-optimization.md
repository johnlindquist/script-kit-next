# Dev Script Optimization Expert Bundle

## Original Goal

> Optimizing how quickly ./dev.sh loads/refreshes
>
> This is the original task description that prompted the creation of this bundle.

## Executive Summary

The `dev.sh` script uses `cargo-watch` to auto-rebuild on file changes. Performance can be improved by optimizing watch patterns, leveraging incremental compilation, and reducing unnecessary rebuilds.

### Key Problems:
1. **Broad watch patterns** - Currently watches entire `src/` directory which may trigger rebuilds on non-critical changes
2. **Cold start time** - Initial build is slow due to full compilation
3. **Rebuild cascade** - Changes to shared modules trigger full rebuilds

### Required Fixes:
1. **dev.sh** - Optimize watch patterns and ignore more non-essential files
2. **Cargo.toml** - Enable incremental compilation optimizations
3. **build.rs** - Minimize build script re-runs

### Files Included:
- `dev.sh`: Main development runner script using cargo-watch
- `Cargo.toml`: Rust project configuration with dependencies
- `build.rs`: Build script that runs at compile time
- `DEV.md`: Development documentation
- `src/setup.rs`: Kit environment setup (triggers on changes)
- `src/watcher.rs`: File watching implementation

---

