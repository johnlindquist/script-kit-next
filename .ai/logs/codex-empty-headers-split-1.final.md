Implemented the requested header-style adjustments in `src/app_render/group_header_item.rs:57`.

Changed file:
- `src/app_render/group_header_item.rs:68`
  - Imported `ALPHA_SEPARATOR`, `SECTION_HEADER_HEIGHT`, and `SECTION_PADDING_TOP` from `crate::list_item`.
  - `render_group_header_item` now uses `SECTION_HEADER_HEIGHT` (32.0) and `SECTION_PADDING_TOP` (12.0).
  - Header label remains uppercased via `group_header_section_name(style.name())`.
  - Header text color and divider tint now use secondary/muted tone (`colors.text_secondary`).
  - Separator uses subtle alpha via `ALPHA_SEPARATOR` (`0x14`).

How to test:
1. `cargo check -p script-kit-gpui`
2. `cargo clippy -p script-kit-gpui --all-targets -- -D warnings`
3. `cargo test -p script-kit-gpui group_header_item`

Verification status:
- Attempted all three commands above.
- They currently fail due unrelated existing workspace issues outside this file (for example: `src/prompts/select/render.rs` method error and multiple pre-existing test/module/import errors in `src/actions/*`, `src/protocol/*`, `src/scripts/*`).

Risks / known gaps:
- Full verification gate is blocked by unrelated compile/test failures in other modules, so this change could not be validated end-to-end in a green workspace.

Commits made:
- None.