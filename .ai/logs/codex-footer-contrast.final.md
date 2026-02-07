Implemented the footer contrast + height restoration with regression tests.

**Changed files**
- `src/components/prompt_footer.rs`
- `src/window_resize.rs`
- `tests/prompt_footer.rs`

**What changed**
- Footer surface now uses `theme.colors.background.main` (theme-driven, no hardcoded rgb) and darker opacity in dark mode to improve helper-text contrast.
- Footer height restored to previous value: `FOOTER_HEIGHT = 30.0` in `src/window_resize.rs`.
- Added tests to lock both behaviors:
  - `prompt_footer_dark_surface_uses_theme_main_background`
  - `prompt_footer_height_restored_to_30px`

**Exact command to verify my changes**
```bash
cargo test --test prompt_footer
```

**UI check command (stdin protocol)**
```bash
echo '{"type":"run","path":"'"$(pwd)"'/tests/smoke/test-chat-footer-style.ts"}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1
```

**Risks / known gaps**
- Full workspace gate (`cargo check && cargo clippy --all-targets -- -D warnings && cargo test`) still has pre-existing unrelated failures outside this change set (unused imports / duplicate test module / missing AI method / clippy lint in other files).  
