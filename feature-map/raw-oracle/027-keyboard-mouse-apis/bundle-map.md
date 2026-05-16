# 027 Keyboard And Mouse APIs Bundle Map

Slug: `keyboard-mouse-apis-atlas`

Feature: Keyboard and Mouse APIs / `keyboard.type()` / `keyboard.tap()` / `mouse.move()` / `mouse.leftClick()` / `mouse.rightClick()` / `mouse.setPosition()`.

Included context:

- `AGENTS.md`, `CLAUDE.md`
- `.agents/skills/platform-windowing-macos/SKILL.md`
- `.agents/skills/keyboard-focus-routing/SKILL.md`
- `.agents/skills/protocol-automation/SKILL.md`
- `.agents/skills/agentic-testing/SKILL.md`
- `.agents/skills/sdk-script-execution/SKILL.md`
- `lat.md/protocol.md`
- `lat.md/design.md`
- `lat.md/verification.md`
- `scripts/kit-sdk.ts`
- `src/protocol/message/variants/system_control.rs`
- `src/protocol/message/constructors/general.rs`
- `src/protocol/types/primitives.rs`
- `scripts/generate-api-tests.ts`
- `scripts/term-perf-bench.ts`
- `scripts/scroll-bench.ts`
- `tests/sdk/test-system.ts`
- `tests/smoke/test-sdk-warnings.ts`
- `tests/smoke/test-protocol-keyboard.ts`
- `tests/smoke/test-protocol-filter.ts`
- `tests/smoke/test-protocol-submit.ts`
- `tests/smoke/test-protocol-escape.ts`
- `tests/smoke/test-actions-visual.ts`
- `tests/smoke/test-actions-autonomous.ts`
- `tests/smoke/design-gallery.ts`
- `tests/smoke/test-actions-click-outside.ts`
- `tests/smoke/test-term-perf-regression.ts`
- `src/mcp_resources/mod.rs`

Packx command:

```bash
packx AGENTS.md CLAUDE.md .agents/skills/platform-windowing-macos/SKILL.md .agents/skills/keyboard-focus-routing/SKILL.md .agents/skills/protocol-automation/SKILL.md .agents/skills/agentic-testing/SKILL.md .agents/skills/sdk-script-execution/SKILL.md lat.md/protocol.md lat.md/design.md lat.md/verification.md scripts/kit-sdk.ts src/protocol/message/variants/system_control.rs src/protocol/message/constructors/general.rs src/protocol/types/primitives.rs scripts/generate-api-tests.ts scripts/term-perf-bench.ts scripts/scroll-bench.ts tests/sdk/test-system.ts tests/smoke/test-sdk-warnings.ts tests/smoke/test-protocol-keyboard.ts tests/smoke/test-protocol-filter.ts tests/smoke/test-protocol-submit.ts tests/smoke/test-protocol-escape.ts tests/smoke/test-actions-visual.ts tests/smoke/test-actions-autonomous.ts tests/smoke/design-gallery.ts tests/smoke/test-actions-click-outside.ts tests/smoke/test-term-perf-regression.ts src/mcp_resources/mod.rs -s "globalThis.keyboard" -s "globalThis.mouse" -s "KeyboardMessage" -s "MouseMessage" -s "keyboard.type" -s "keyboard.tap" -s "mouse.move" -s "mouse.leftClick" -s "mouse.rightClick" -s "mouse.setPosition" -s "Message::Keyboard" -s "Message::Mouse" -s "KeyboardAction" -s "MouseAction" -s "MouseData" -s "not yet implemented" -s "ignored" -s "simulateKey" -s "batch" -l 14 --strip-comments --minify -f markdown --no-interactive --stdout > ~/.oracle/bundles/keyboard-mouse-apis-atlas.txt
```

Final bundle size on disk: 136,587 bytes. Packx reported 37,694 exact tokens and 134,741 total chars.
