Changed files:
- `src/components/prompt_footer.rs:364`  
  - Updated footer button label default color from `accent` to `text_muted`, so accent is not the default button label color.

What I verified:
- `PROMPT_FOOTER_PADDING_X_PX` is already `14.0` in `src/components/prompt_footer.rs:54`.
- Shortcut keycap/pill styling is already present in `src/components/prompt_footer.rs:368` with:
  - `.px(px(6.0)).py(px(1.0)).rounded(px(4.0))`
  - background from border token at `0x20`
  - `.border_1().border_color(...)` with border token at `0x40`
  - monospace font via `crate::list_item::FONT_MONO`

How to test:
1. Run compile check:
   - `cargo check -p script-kit-gpui`
2. Optional visual check:
   - `echo '{"type":"show"}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1`
   - Confirm footer action labels are neutral by default and shortcut hints render as pill keycaps.

Risks / known gaps:
- `cargo test -p script-kit-gpui prompt_footer` currently fails due unrelated pre-existing test/module errors in other areas (`actions`, `protocol`, `scripts`), not from this footer change.

Commits:
- None.