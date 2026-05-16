[acp-sdk-runtime-apis]


- The feature map is a human-and-agent atlas. Each feature chapter must fully explain capabilities, entry points, interactions, state machines, APIs, proof paths, and open risk.
- Repo process requires `removed-docs/` updates for changed behavior/docs and `source checks` verification. For this atlas loop, every Oracle session's full output is preserved under `feature-map/raw-oracle/<feature-id>/` and distilled into `feature-map/features/<feature-id>.md`.



- `aiAppendMessage(chatId, content, role)`.
- `aiSendMessage(chatId, content, imagePath?, parts?)`.
- `aiSetSystemPrompt(chatId, prompt)`.
- `aiOn(eventType, handler, chatId?)`, `aiSubscribe`, `aiUnsubscribe`, and pushed event messages such as `aiStreamChunk`, `aiStreamComplete`, `aiNewMessage`, and `aiError`.


- Feature 030 established that `aiIsOpen`, `aiGetActiveChat`, `aiListChats`, `aiGetConversation`, `aiStartChat`, `aiFocus`, `aiGetStreamingStatus`, and `aiDeleteChat` are proven through SDK/protocol/direct-handler or prompt-message paths.
- Feature 030 explicitly left `aiAppendMessage`, `aiSendMessage`, `aiSetSystemPrompt`, and `aiOn` as declared but unproven for app-side runtime handling.
- This pass should not repeat the already-proven API catalog except where needed to explain the contrast between proven and unproven runtime paths.




Return a dense, implementation-ready atlas chapter outline for local agents to distill into `feature-map/features/043-acp-sdk-runtime-apis.md`.


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


Return your answer as text in this response only. Do not create, attach, export,
or offer any downloadable file. Do not create local project artifacts yourself.
The local agent will write any needed files, plans, notes, goals, commits, or
verification logs using local tools.
