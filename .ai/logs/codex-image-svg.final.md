Added the audit report at `crates-audits/image-svg.md`.

Changed files:
- `crates-audits/image-svg.md`

Verification run:
- `cargo tree -e features -i image`
- `cargo tree -d | rg -n "resvg|usvg|tiny-skia|image|xcap|png|jpeg"`
- `cargo tree -i resvg@0.45.1`
- `cargo tree -i resvg@0.46.0`
- `cargo tree -i usvg@0.45.1`
- `cargo tree -i usvg@0.46.0`
- `cargo tree -i tiny-skia@0.11.4`
- `cargo tree -i xcap@0.8.0`
- `cargo check` (failed due pre-existing unrelated compile errors)

`cargo check` blocker details (unrelated to this doc change):
- `src/watcher.rs` references private `config::defaults::*` (E0603)
- `src/hotkeys.rs` has `&Option<HotkeyConfig>` vs `&HotkeyConfig` mismatches (E0308)

How to test:
1. Read the report: `crates-audits/image-svg.md`
2. Re-run the audit verification commands above.
3. After workspace compile issues are fixed, re-run `cargo check` (or full gate).

Risks / known gaps:
- Audit recommendations are documented but not implemented in code.
- Full compile/test gate could not be completed because of unrelated existing build errors.

Commits:
- None.