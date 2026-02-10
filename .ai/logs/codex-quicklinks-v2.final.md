Implemented the scoped quicklinks storage module and wired it into the crate.

**Changed Files**
- `src/quicklinks/mod.rs:1`
  - Added `Quicklink { id, name, url_template, icon }`.
  - Added:
    - `load_quicklinks() -> Vec<Quicklink>`
    - `save_quicklinks(links: &[Quicklink])`
    - `create_quicklink(name: &str, url_template: &str) -> Quicklink`
    - `delete_quicklink(id: &str)`
    - `expand_url(url_template: &str, query: &str) -> String`
  - Added 7 unit tests for create/load/save/delete/expand behavior.
- `src/lib.rs:116`
  - Added `pub mod quicklinks;`

**How To Test**
1. Reproducible isolated module test (used due unrelated shared-tree compile failures):
```bash
tmpdir=$(mktemp -d)
cat > "$tmpdir/Cargo.toml" <<'EOF'
[package]
name = "quicklinks_harness"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
uuid = { version = "1.0", features = ["v4"] }
dirs = "5.0"
tempfile = "3"
EOF
mkdir -p "$tmpdir/src"
cat > "$tmpdir/src/lib.rs" <<'EOF'
#[path = "/Users/johnlindquist/dev/script-kit-gpui/src/quicklinks/mod.rs"]
pub mod quicklinks;
EOF
cargo test --manifest-path "$tmpdir/Cargo.toml" quicklinks::tests
```
2. Note: `cargo check --lib` and `cargo test --lib quicklinks::tests` currently fail in unrelated in-flight files (`clipboard_history/*`, duplicate `protocol/message/tests` module paths), not in this scope.

**Risks / Known Gaps**
- `save_quicklinks` and `delete_quicklink` intentionally swallow IO/serialization errors because the requested API uses `()` return types.
- Could not complete repo-wide verification gate because the shared tree is currently broken outside this task scope.

**Commit**
- `e880ec7` `feat(quicklinks): add quicklinks model and JSON storage`