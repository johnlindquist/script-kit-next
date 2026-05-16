# Bundle Map: 031 Legacy `chat()` Prompt

## Session Attempts

- Attempt 1: `legacy-chat-prompt-atlas`, about 47.3K tokens. This completed successfully and is promoted to the root raw files for this feature.
- Attempt 2: `legacy-chat-prompt-retry`, about 33.4K tokens. The browser session initially looked stale but later completed; the complete supplemental session output is preserved under `attempt-2-stale-session/`.
- Attempt 3: `legacy-chat-prompt-api`. Oracle API fallback failed because `OPENAI_API_KEY` is not configured; the failed session output is preserved under `attempt-3-api-missing-key/`.
- Attempt 4: `legacy-chat-prompt-files`, about 33.4K tokens, uploaded as real browser attachments. The browser run failed before submit because attachments were not present in the composer; the complete failed session output is preserved under `attempt-4-attachment-failed/`.
- Attempt 5: `legacy-chat-summary-atlas`, compact source-derived evidence prompt with no attachments. This run completed and is preserved under `attempt-5-summary-atlas/`, but it was superseded by the completed first bundle session for chapter authorship.

## Successful Packx Command

```bash
packx --limit 49k -l 15 \
  -s "chatFn" \
  -s "ChatPrompt" \
  -s "Message::Chat" \
  -s "ChatSubmit" \
  -s "ChatStream" \
  -s "chatSubmit" \
  -s "useBuiltinAi" \
  -s "with_builtin_ai" \
  -s "handle_submit" \
  -s "toggle_chat_actions" \
  -s "execute_chat_action" \
  -s "show_inline_ai_chat" \
  -s "MiniAiUiRequest" \
  -f markdown --no-interactive --stdout \
  AGENTS.md CLAUDE.md .goals/feature_map.md \
  .agents/skills/prompt-runtime/SKILL.md \
  .agents/skills/sdk-script-execution/SKILL.md \
  .agents/skills/acp-chat-core/SKILL.md \
  .agents/skills/protocol-automation/SKILL.md \
  lat.md/protocol.md lat.md/acp-chat.md lat.md/design.md lat.md/verification.md \
  scripts/kit-sdk.ts \
  src/protocol/message/variants/prompts_media.rs \
  src/protocol/types/chat.rs \
  src/main_sections/prompt_messages.rs \
  src/prompt_handler/mod.rs \
  src/prompts/chat/prompt.rs \
  src/prompts/chat/streaming.rs \
  src/prompts/chat/state.rs \
  src/prompts/chat/actions.rs \
  src/prompts/chat/render_core.rs \
  src/prompts/chat/render_turns.rs \
  src/prompts/chat/render_input.rs \
  src/prompts/chat/render_setup.rs \
  src/prompts/chat/types.rs \
  src/render_prompts/other.rs \
  src/app_impl/actions_toggle.rs \
  src/app_impl/actions_dialog.rs \
  src/app_impl/chat_actions.rs \
  src/app_impl/prompt_ai.rs \
  src/main_sections/render_impl.rs \
  src/main_sections/app_view_state.rs \
  tests/sdk/test-chat.ts \
  tests/smoke/run-chat-tests.ts \
  tests/smoke/test-chat-oninit.ts \
  tests/smoke/test-chat-errors.ts \
  tests/smoke/test-chat-ai-sdk-compat.ts \
  tests/smoke/test-chat-visual-layout.ts \
  tests/smoke/test-chat-visual-content.ts \
  tests/smoke/test-chat-callbacks.ts \
  tests/smoke/test-chat-edge-cases.ts \
  tests/smoke/test-chat-prompt.ts \
  tests/mini_ai_snapshot_contract.rs \
  tests/mini_window_sizing_contract.rs \
  tests/mini_ai_actions_contract.rs \
  > ~/.oracle/bundles/legacy-chat-prompt-atlas.txt
```

## Successful Pack Summary

- Total files: 45
- Search mode: ripgrep
- Context lines: 15 around each match
- Total matches: 268
- Context windows: 76
- Total tokens: about 47.3K
- Total chars: 178,764 in packx summary; 181,003 bytes on disk

## Inclusion Rationale

- `AGENTS.md`, `CLAUDE.md`, and `.goals/feature_map.md` preserve repo process rules and the feature-map/oracle-loop contract.
- Prompt, SDK, ACP, and protocol skills provide ownership context and distinguish legacy `chat()` from ACP Chat.
- `lat.md/protocol.md`, `lat.md/acp-chat.md`, `lat.md/design.md`, and `lat.md/verification.md` provide architectural context and verification expectations.
- `scripts/kit-sdk.ts` is the SDK entry point and controller implementation.
- `src/protocol/...` and `src/main_sections/prompt_messages.rs` define message shapes and protocol handling.
- `src/prompt_handler/mod.rs`, `src/prompts/chat/*`, and `src/render_prompts/other.rs` define prompt construction, rendering, streaming, setup, input, actions, and shell integration.
- `src/app_impl/actions_*`, `src/app_impl/chat_actions.rs`, `src/app_impl/prompt_ai.rs`, `src/main_sections/render_impl.rs`, and `src/main_sections/app_view_state.rs` define parent app integration, actions dialogs, inline Mini AI routing, focus, and view state.
- Tests cover SDK behavior, smoke-level chat behavior, and Mini AI snapshot/sizing/actions source contracts.
