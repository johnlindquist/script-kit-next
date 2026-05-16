# 029 Widget Media And Find APIs Bundle Map

Slug: `widget-media-find-apis-atlas`

Feature: Widget, Media, and Find APIs / `widget()` / `webcam()` / `mic()` / `eyeDropper()` / `find()`.

Included context:

- `AGENTS.md`, `CLAUDE.md`
- `.agents/skills/sdk-script-execution/SKILL.md`
- `.agents/skills/prompt-runtime/SKILL.md`
- `.agents/skills/file-search-portals/SKILL.md`
- `.agents/skills/dictation-media/SKILL.md`
- `.agents/skills/protocol-automation/SKILL.md`
- `.agents/skills/agentic-testing/SKILL.md`
- `lat.md/scripting.md`
- `lat.md/surfaces.md`
- `lat.md/protocol.md`
- `lat.md/design.md`
- `lat.md/verification.md`
- `lat.md/tests/dictation-setup-nux.md`
- `scripts/kit-sdk.ts`
- `src/protocol/message/variants/prompts_media.rs`
- `src/protocol/message/constructors/general.rs`
- `src/protocol/message/constructors/prompts.rs`
- `src/execute_script/mod.rs`
- `src/prompt_handler/mod.rs`
- `src/main_sections/prompt_messages.rs`
- `src/render_prompts/other.rs`
- `src/mcp_resources/mod.rs`
- `tests/sdk/test-widget.ts`
- `scripts/generate-api-tests.ts`
- `tests/minimal_chrome_audit.rs`
- `tests/mcp_resources_sdk_reference.rs`

Packx command:

```bash
packx AGENTS.md CLAUDE.md .agents/skills/sdk-script-execution/SKILL.md .agents/skills/prompt-runtime/SKILL.md .agents/skills/file-search-portals/SKILL.md .agents/skills/dictation-media/SKILL.md .agents/skills/protocol-automation/SKILL.md .agents/skills/agentic-testing/SKILL.md lat.md/scripting.md lat.md/surfaces.md lat.md/protocol.md lat.md/design.md lat.md/verification.md lat.md/tests/dictation-setup-nux.md scripts/kit-sdk.ts src/protocol/message/variants/prompts_media.rs src/protocol/message/constructors/general.rs src/protocol/message/constructors/prompts.rs src/execute_script/mod.rs src/prompt_handler/mod.rs src/main_sections/prompt_messages.rs src/render_prompts/other.rs src/mcp_resources/mod.rs tests/sdk/test-widget.ts scripts/generate-api-tests.ts tests/minimal_chrome_audit.rs tests/mcp_resources_sdk_reference.rs -s "globalThis.widget" -s "globalThis.webcam" -s "globalThis.mic" -s "globalThis.eyeDropper" -s "globalThis.find" -s "WidgetMessage" -s "WidgetActionMessage" -s "WidgetEventMessage" -s "WebcamMessage" -s "MicMessage" -s "EyeDropperMessage" -s "FindMessage" -s "Message::Widget" -s "Message::Webcam" -s "Message::Mic" -s "Message::Find" -s "WidgetComingSoon" -s "WebcamComingSoon" -s "MicComingSoon" -s "show_prompt_coming_soon_toast" -s "widget() is not yet implemented" -s "webcam() is not implemented" -s "mic() is not implemented" -s "eyeDropper() is not implemented" -s "media streaming" -s "SDK_NOT_YET_IMPLEMENTED_IN_GPUI" -s "find-api" -s "widget-basic" -l 14 --strip-comments --minify -f markdown --no-interactive --stdout > ~/.oracle/bundles/widget-media-find-apis-atlas.txt
```

Final bundle size on disk: 29,199 bytes. Packx reported 8,888 exact tokens and 27,538 total chars.
