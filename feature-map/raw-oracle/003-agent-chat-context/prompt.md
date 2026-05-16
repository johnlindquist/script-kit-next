[acp-context-feature-map]

Project briefing:
- Repo: /Users/johnlindquist/dev/script-kit-gpui, Rust GPUI desktop app for Script Kit.
- The user wants a 100% exhaustive FEATURE_MAP.md built iteratively. Each iteration asks Oracle to map one feature's user stories/states/keystrokes/scenarios/visual states tersely, the local agent writes Oracle's response into FEATURE_MAP.md, then continues to the next feature.
- Feature 001 Main Menu / Script List / Menu Syntax / Actions / Shortcut Assignment is captured from Oracle session `main-menu-feature-map`.
- Feature 002 Dedicated File Search / Root Files / Directory Browse / File Actions is captured from Oracle session `file-search-feature-map`.
- Repo rules: run lat search/lat expand before work; update lat.md only when functionality/architecture/tests/behavior change; run lat check after every task before final response. This current task is docs-artifact generation only, not behavior change.
- Required repo context included in bundle: CLAUDE.md/AGENTS.md process context, owning ACP chat/context/MCP skills, and relevant lat.md pages including acp-chat, ai-context, acp portal tests.

Goal for this iteration:
Map FEATURE 003: ACP Chat / Agent Catalog / Context Composer / Attachment Portals.

Scope to cover exhaustively in terse language:
- Agent Chat entry paths: main launcher Tab/Cmd+Enter, plugin skill, File Search AI handoff, actions handoff, notes/dictation/large paste where visible, embedded vs detached reuse.
- Agent catalog and setup: starter agents, default Codex behavior, install/setup states, model selector, change agent/model actions, agent switching preserving drafts and pending context.
- ACP lifecycle: embedded surface, detached popup, focus/close/reuse, Cmd+W, Escape cancellation vs close, streaming/tool/permission states, footer activity dot, Run button/SCRIPT_READY if visible.
- Context composer: input text, caret, slash commands, @mentions, picker popup, model/history popups, pending context parts, inline tokens/chips, typed mention aliases, pasted text/image, large paste, skill file staging.
- Attachment portals: file search, clipboard, dictation, notes, ACP history, scripts/scriptlets/skills if visible; portal open/session/accept/cancel; exact replacement and return-origin behavior; detached portal limits.
- Every keystroke/state transition visible in the bundle: character input, `/`, `@`, Tab, Cmd+Enter, Cmd+Shift+Enter, Cmd+K, Escape, Cmd+W, setAcpInput, submit/cancel, picker arrows/Enter if visible.
- Visual/protocol states: acpChat semantic surface, AcpDetached, embedded Ai registry, PromptPopup ids, getAcpState/getAcpTestProbe, actionsDialog, context summary, pending parts, automation receipts.

Current evidence:
- `lat expand` on this feature name returned no bracket refs to expand.
- `lat search` returned relevant sections: `lat.md/acp-chat#ACP Chat`, `lat.md/ai-context#AI Context and MCP#Portal flow`, `lat.md/acp-chat#ACP Chat#Agent switching`, and portal contract sections.
- `lat check` passed after feature 002 was appended.

Bundle map:
- Bundle contains targeted snippets from process docs, ACP chat/context/MCP skills, lat.md acp-chat/ai-context/acp portal contract, ACP catalog/chat_window/composer/config/context/history/model/picker/popup/portal/surface/thread/view files, context mentions, context snapshot, attachment portal, tab_ai_mode entry/staging/launch files, and focused ACP/context tests.
- Bundle may omit unrelated app features and some full source bodies due token limits; stay within the scoped feature and call out high-confidence gaps as `NEEDS_NEXT_PASS` items rather than inventing.

Deliverable format:
Return only markdown suitable for appending to FEATURE_MAP.md after feature 002.
Use this exact structure:

## 003 ACP Chat / Agent Catalog / Context Composer / Attachment Portals

### Boundaries
- Owns: ...
- Adjacent: ...
- Source anchors: ...
- Verification anchors: ...

### State Map
Use terse bullet rows. Each row should be: `State | Entry | Input/Keystroke | Visible/Protocol State | Exit/Next | Notes`.

### Keystroke Matrix
Use terse bullet rows. Each row should be: `Key | Preconditions | Handler/Owner | Result | Receipt/Test`.

### Visual/Protocol Matrix
Use terse bullet rows. Each row should be: `Surface | Visible State | Protocol/Automation Receipt | Failure Mode | Proof`.

### Context / Portal Matrix
Use terse bullet rows. Each row should be: `Intent | Entry | Context/Portal State | Accept/Cancel | Return/Replacement | Proof`.

### Agent / Model Matrix
Use terse bullet rows. Each row should be: `Intent | Entry | Agent/Model State | Runtime Effect | Visual State | Proof`.

### Exhaustiveness Notes
- Include every explicit scenario/state you can infer from the bundle.
- Include `NEEDS_NEXT_PASS:` bullets for known omissions requiring another Oracle pass or more context.
- End with `Suggested next feature: 004 MCP Resources / SDK / Protocol Automation` unless the bundle strongly suggests a better next slice.

Style constraints:
- Be terse, but do not drop edge cases.
- Prefer exact product terms and source/function names from the bundle.
- Do not write explanatory prose outside the markdown artifact.
- Do not mention that you are Oracle.

Output boundary:
Return your answer as text in this response only. Do not create, attach, export, or offer any downloadable file. Do not create local project artifacts yourself. The local agent will write any needed files, plans, notes, goals, commits, or verification logs using local tools.
