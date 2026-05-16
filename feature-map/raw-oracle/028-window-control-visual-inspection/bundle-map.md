# 028 Window Control And Visual Inspection APIs Bundle Map

Slug: `window-control-visual-inspection-atlas`

Feature: Window Control and Visual Inspection APIs / `show()` / `hide()` / `blur()` / `showGrid()` / `hideGrid()` / `getWindowBounds()` / `captureScreenshot()` / `getLayoutInfo()`.

Included context:

- `AGENTS.md`, `CLAUDE.md`
- `.agents/skills/platform-windowing-macos/SKILL.md`
- `.agents/skills/window-resizing/SKILL.md`
- `.agents/skills/protocol-automation/SKILL.md`
- `.agents/skills/agentic-testing/SKILL.md`
- `.agents/skills/sdk-script-execution/SKILL.md`
- `lat.md/windowing.md`
- `lat.md/automation.md`
- `lat.md/protocol.md`
- `lat.md/verification.md`
- `scripts/kit-sdk.ts`
- `src/protocol/message/variants/query_ops.rs`
- `src/protocol/message/variants/system_control.rs`
- `src/protocol/message/constructors/query_ops.rs`
- `src/protocol/message/constructors/general.rs`
- `src/protocol/message/constructors/final_sections.rs`
- `src/protocol/types/grid_layout.rs`
- `src/protocol/types/system.rs`
- `src/execute_script/mod.rs`
- `src/prompt_handler/mod.rs`
- `src/debug_grid/mod.rs`
- `src/app_layout/build_layout_info.rs`
- `src/app_layout/build_component_bounds.rs`
- `src/platform/screenshots_window_open.rs`
- `src/main_entry/runtime_stdin_match_core.rs`
- `src/main_entry/runtime_stdin_match_tail.rs`
- `src/main_entry/app_run_setup.rs`
- `src/mcp_resources/mod.rs`
- `tests/stdin_show_hide_simulatekey_no_response_envelope_contract.rs`
- `tests/source_audits/stdin_get_window_bounds_wired.rs`
- `tests/get_state_target_contract.rs`
- `tests/verify_shot_strict_window_contract.rs`
- `tests/protocol-coverage-matrix.ts`
- `tests/sdk/test-capture-screenshot.ts`
- `tests/smoke/test-debug-grid-basic.ts`
- `tests/smoke/test-grid-dimensions.ts`
- `tests/smoke/test-layout-info-simple.ts`
- `tests/smoke/test-resize-behavior.ts`

Packx command:

```bash
packx AGENTS.md CLAUDE.md .agents/skills/platform-windowing-macos/SKILL.md .agents/skills/window-resizing/SKILL.md .agents/skills/protocol-automation/SKILL.md .agents/skills/agentic-testing/SKILL.md .agents/skills/sdk-script-execution/SKILL.md lat.md/windowing.md lat.md/automation.md lat.md/protocol.md lat.md/verification.md scripts/kit-sdk.ts src/protocol/message/variants/query_ops.rs src/protocol/message/variants/system_control.rs src/protocol/message/constructors/query_ops.rs src/protocol/message/constructors/general.rs src/protocol/message/constructors/final_sections.rs src/protocol/types/grid_layout.rs src/protocol/types/system.rs src/execute_script/mod.rs src/prompt_handler/mod.rs src/debug_grid/mod.rs src/app_layout/build_layout_info.rs src/app_layout/build_component_bounds.rs src/platform/screenshots_window_open.rs src/main_entry/runtime_stdin_match_core.rs src/main_entry/runtime_stdin_match_tail.rs src/main_entry/app_run_setup.rs src/mcp_resources/mod.rs tests/stdin_show_hide_simulatekey_no_response_envelope_contract.rs tests/source_audits/stdin_get_window_bounds_wired.rs tests/get_state_target_contract.rs tests/verify_shot_strict_window_contract.rs tests/protocol-coverage-matrix.ts tests/sdk/test-capture-screenshot.ts tests/smoke/test-debug-grid-basic.ts tests/smoke/test-grid-dimensions.ts tests/smoke/test-layout-info-simple.ts tests/smoke/test-resize-behavior.ts -s "globalThis.show" -s "globalThis.hide" -s "globalThis.blur" -s "globalThis.showGrid" -s "globalThis.hideGrid" -s "globalThis.getWindowBounds" -s "globalThis.captureScreenshot" -s "globalThis.getLayoutInfo" -s "ShowMessage" -s "HideMessage" -s "BlurMessage" -s "ShowGridMessage" -s "HideGridMessage" -s "GetWindowBoundsMessage" -s "CaptureScreenshotMessage" -s "GetLayoutInfoMessage" -s "Message::Show" -s "Message::Hide" -s "Message::Blur" -s "Message::ShowGrid" -s "Message::HideGrid" -s "Message::GetWindowBounds" -s "Message::CaptureScreenshot" -s "Message::GetLayoutInfo" -s "PromptMessage::HideWindow" -s "PromptMessage::ShowGrid" -s "PromptMessage::HideGrid" -s "PromptMessage::GetLayoutInfo" -s "captureScreenshot" -s "screenshotResult" -s "layoutInfoResult" -s "windowBounds" -s "GridOptions" -s "LayoutInfo" -s "no response envelope" -s "pixel audit" -s "Screen Recording" -l 14 --strip-comments --minify -f markdown --no-interactive --stdout > ~/.oracle/bundles/window-control-visual-inspection-atlas.txt
```

Final bundle size on disk: 200,012 bytes. Packx reported 55,222 exact tokens and 197,609 total chars.
