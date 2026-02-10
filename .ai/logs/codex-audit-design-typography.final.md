**Findings (Typography Audit)**
1. Medium, fixed: Active preview panel used hardcoded typography values instead of design tokens (`11.0` label size, `20.0` line height).  
   References: `src/app_render/preview_panel.rs:1`, `src/app_render/preview_panel.rs:138`, `src/app_render/preview_panel.rs:404`, `src/app_render/preview_panel.rs:432`.

2. Medium, open: Group-header item text has no truncation controls, so long names/descriptions can overflow in constrained widths.  
   References: `src/app_render/group_header_item.rs:96`, `src/app_render/group_header_item.rs:105`, `src/app_render/group_header_item.rs:111`.

3. Low, open (currently dormant code path): `src/app_render/preview_panel/result_*.rs` still has raw typography literals (`11.0`, `20.0`).  
   References: `src/app_render/preview_panel/result_app.rs:23`, `src/app_render/preview_panel/result_window.rs:22`, `src/app_render/preview_panel/result_script.rs:194`.  
   Active include path currently points to `src/app_render/preview_panel.rs` via `src/app_render.rs:2`.

**Changed Files**
- `src/app_render/preview_panel.rs`
- `src/app_render/tests.rs`

**How To Test**
1. `cargo test --bin script-kit-gpui preview_panel_metadata_tests::test_preview_panel_typography`
2. `cargo check --bin script-kit-gpui`
3. `cargo clippy --bin script-kit-gpui -- -D warnings -A dead_code`

**Risks / Known Gaps**
- Strict clippy with `-D warnings` currently fails on unrelated existing dead-code in `src/execute_script/part_000.rs`; I scoped around that with `-A dead_code` for this verification run.
- Open findings above were not modified in this pass.

**Commit**
- `c8fc833`