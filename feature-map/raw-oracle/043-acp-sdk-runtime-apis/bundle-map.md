# 043 ACP SDK Runtime APIs Bundle Map

Oracle session for the ACP/Agent Chat SDK runtime gap atlas.

## Session

- Feature id: `043-acp-sdk-runtime-apis`
- Oracle slug: `acp-sdk-runtime-apis`
- Status: completed
- Model: `gpt-5.5-pro`
- Browser label: `Latest`
- Thinking time: `extended`
- Completed at: `2026-05-15T15:00:15.016Z`
- Conversation URL: `https://chatgpt.com/c/6a07338d-5ec4-83e8-aa49-82ea636b0c67`

## Token And Size Receipt

- Bundle path: `/Users/johnlindquist/.oracle/bundles/acp-sdk-runtime-apis.txt`
- Bundle size: `144260` bytes
- Oracle reported input tokens: `42735`
- Oracle reported output tokens: `7320`
- Oracle reported total tokens: `50055`
- Raw output log size: `65045` bytes
- Extracted answer size: `58957` bytes

## Bundle Contents

The bundle was narrowed from a 279k-token full-source pass to a 41k-token keyword-context pass around ACP SDK runtime gaps.

Included context:

- `AGENTS.md`
- `CLAUDE.md`
- `.agents/skills/acp-chat-core/SKILL.md`
- `.agents/skills/sdk-script-execution/SKILL.md`
- `.agents/skills/protocol-automation/SKILL.md`
- `.agents/skills/agentic-testing/SKILL.md`
- `.agents/skills/testing-quality-gates/SKILL.md`
- `lat.md/acp-chat.md`
- `lat.md/protocol.md`
- `lat.md/scripting.md`
- `lat.md/verification.md`
- `scripts/kit-sdk.ts`
- `src/protocol/message/variants/ai.rs`
- `src/protocol/types/ai.rs`
- `src/ai/sdk_handlers.rs`
- `src/execute_script/mod.rs`
- `src/main_sections/prompt_messages.rs`
- `src/prompt_handler/mod.rs`
- `src/ai/acp/view.rs`
- `src/ai/acp/thread.rs`
- `src/protocol/types/acp_state.rs`
- `tests/sdk/test-acp-sdk.ts`
- `tests/protocol_ai_parts.rs`
- `tests/acp_targeted_reads.rs`
- `feature-map/features/003-agent-chat-context.md`
- `feature-map/features/030-acp-chat-sdk-apis.md`
- `feature-map/raw-oracle/030-acp-chat-sdk-apis/answer.md`

## Prompt Intent

Oracle was asked to map the current runtime status of declared-but-unproven ACP SDK APIs:

- `aiAppendMessage`
- `aiSendMessage`
- `aiSetSystemPrompt`
- `aiOn`
- `aiSubscribe`
- `aiUnsubscribe`
- pushed events such as `aiStreamChunk`, `aiStreamComplete`, `aiNewMessage`, and `aiError`

The requested output was a dense atlas outline, not implementation code, with exact runtime boundaries, unsafe claims to avoid, source proof targets, implementation plan, and verification receipts.
