[acp-sdk-runtime-apis]

Project briefing:

- Repository: Script Kit GPUI, a Rust/GPUI desktop app with TypeScript/Bun script SDK compatibility.
- The feature map is a human-and-agent atlas. Each feature chapter must fully explain capabilities, entry points, interactions, state machines, APIs, proof paths, and open risk.
- Repo process requires `lat.md/` updates for changed behavior/docs and `lat check` verification. For this atlas loop, every Oracle session's full output is preserved under `feature-map/raw-oracle/<feature-id>/` and distilled into `feature-map/features/<feature-id>.md`.
- Relevant ownership skills for this pass: `acp-chat-core`, `sdk-script-execution`, `protocol-automation`, `agentic-testing`, and `testing-quality-gates`.

Goal:

Create a comprehensive feature-map reference for the ACP/Agent Chat script-facing SDK runtime APIs, focused on the gap left by feature 030:

- `aiAppendMessage(chatId, content, role)`.
- `aiSendMessage(chatId, content, imagePath?, parts?)`.
- `aiSetSystemPrompt(chatId, prompt)`.
- `aiOn(eventType, handler, chatId?)`, `aiSubscribe`, `aiUnsubscribe`, and pushed event messages such as `aiStreamChunk`, `aiStreamComplete`, `aiNewMessage`, and `aiError`.

Current evidence:

- Feature 030 established that `aiIsOpen`, `aiGetActiveChat`, `aiListChats`, `aiGetConversation`, `aiStartChat`, `aiFocus`, `aiGetStreamingStatus`, and `aiDeleteChat` are proven through SDK/protocol/direct-handler or prompt-message paths.
- Feature 030 explicitly left `aiAppendMessage`, `aiSendMessage`, `aiSetSystemPrompt`, and `aiOn` as declared but unproven for app-side runtime handling.
- This pass should not repeat the already-proven API catalog except where needed to explain the contrast between proven and unproven runtime paths.

Bundle map:

- Repo process docs: `AGENTS.md`, `CLAUDE.md`.
- Owning skills: ACP core, SDK script execution, protocol automation, agentic testing, testing gates.
- Lat docs: ACP chat, protocol, scripting, verification.
- SDK/protocol/runtime source excerpts: `scripts/kit-sdk.ts`, `src/protocol/message/variants/ai.rs`, `src/protocol/types/ai.rs`, `src/ai/sdk_handlers.rs`, `src/execute_script/mod.rs`, `src/main_sections/prompt_messages.rs`, `src/prompt_handler/mod.rs`, ACP state/source excerpts.
- Tests: SDK and protocol tests around ACP/AI APIs and targeted ACP reads.
- Prior feature-map evidence: feature 030 chapter and raw Oracle answer, plus feature 003 context where relevant.

Deliverable:

Return a dense, implementation-ready atlas chapter outline for local agents to distill into `feature-map/features/043-acp-sdk-runtime-apis.md`.

Please include:

1. Exact current behavior and runtime status for each unproven SDK API.
2. Where each API is declared in TypeScript and Rust protocol.
3. Whether each API has a direct handler, prompt-message bridge, ACP runtime handler, subscription manager, or event producer.
4. The user-visible capabilities a script author can safely rely on today.
5. The unsafe claims the atlas must avoid until implementation/proof exists.
6. State machine / lifecycle details for existing-chat mutation, streaming, event subscription, event delivery, and unsubscribe if supported or intentionally absent.
7. Concrete source file/function targets that prove current behavior.
8. The highest-leverage implementation plan if the team wants to make the unproven APIs real.
9. Tests and agentic receipts that would prove the current behavior and any future implementation.
10. Any docs wording or SDK compatibility caveats that should be explicit for humans and AI agents.

Output boundary:

Return your answer as text in this response only. Do not create, attach, export,
or offer any downloadable file. Do not create local project artifacts yourself.
The local agent will write any needed files, plans, notes, goals, commits, or
verification logs using local tools.
