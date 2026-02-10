**Findings (Performance Audit)**
- `src/render_builtins/theme_chooser.rs:19`  
  `theme_chooser_filtered_indices` was doing `to_lowercase()` allocations on every preset field each filter pass. Added an ASCII fast path (`contains_ascii_case_insensitive`) to avoid per-item string allocation for common typing.
- `src/render_builtins/theme_chooser.rs:399`  
  The theme list path built `Vec<String>` (`preset_names`, `preset_descs`) each render and then cloned row strings. Replaced with `presets_for_list` and direct `&'static str` usage (`name`, `desc`, badge text) to remove those allocations.
- `src/render_builtins/theme_chooser.rs:151`  
  Terminal preview colors were rebuilt as heap `Vec<u32>` each render. Switched to fixed arrays to avoid heap allocs in render.
- `src/app_render/group_header_item.rs:1`  
  `preview_keyword_tags` now preallocates with `Vec::with_capacity(keywords.len().min(6))`.
- `src/app_render/group_header_item.rs:21`  
  `builtin_feature_annotation` returned allocated `String` for static labels; switched to `Cow<'_, str>` (borrowed for static labels and app name references) to avoid unnecessary allocations.
- `src/app_render/group_header_item.rs:70`  
  Removed `to_string()` allocations for `style.name()` / `style.description()` in `render_group_header_item`.

**Changed Files**
- `src/render_builtins/theme_chooser.rs`
- `src/app_render/group_header_item.rs`
- `src/app_render/tests.rs`

**How To Test**
1. `cargo check --bin script-kit-gpui`
2. `cargo clippy --bin script-kit-gpui -- -D warnings`
3. `cargo test --bin script-kit-gpui preview_panel_metadata_tests:: -- --nocapture`
4. `cargo test --bin script-kit-gpui theme_chooser_filter_tests:: -- --nocapture`

All commands above passed.

**Risks / Known Gaps**
- ASCII optimization is intentionally scoped: non-ASCII filters still use the previous lowercase allocation path.
- Verification was scoped to this target (`--bin script-kit-gpui`) per parallel-agent constraints; full workspace gate was not run in this parallel session.