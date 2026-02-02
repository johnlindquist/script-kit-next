# Claude Desktop (macOS) - Features and UX Patterns

Date: 2026-02-01
Scope: Claude Desktop on macOS. Focus on conversation management, keyboard shortcuts, artifacts, context handling, and streaming-like UX cues.

## Sources (official)
- Claude Help Center: Quick entry (macOS) - https://support.claude.com/en/articles/12626668-use-quick-entry-with-claude-desktop-on-mac
- Claude Help Center: Chat search + memory - https://support.claude.com/en/articles/11817273-using-claude-s-chat-search-and-memory-to-build-on-previous-context
- Claude Help Center: Projects overview - https://support.claude.com/en/articles/9517075-what-are-projects
- Claude Help Center: Manage projects - https://support.claude.com/en/articles/9519177-how-can-i-create-and-manage-projects
- Claude Help Center: Artifacts - https://support.claude.com/en/articles/9487310-what-are-artifacts-and-how-do-i-use-them
- Claude Help Center: Installing Claude Desktop (extensions) - https://support.claude.com/en/articles/10065433-installing-claude-desktop

## Conversation management
Observed features
- Projects are self-contained workspaces with their own chat histories and knowledge bases; free users can create up to five projects. Projects allow focused chats with uploaded context and project instructions. Context is not shared across chats unless added to the project knowledge base. Paid plans can expand project knowledge via RAG when approaching context limits.
- Chat search lets users prompt Claude to search past conversations; searches are scoped to non-project chats or within a single project. Search appears as tool calls in the conversation and can be toggled in Settings > Capabilities.
- Incognito chats (ghost icon) create temporary conversations not saved to history and excluded from search. (Enterprise/Team exports still include them under retention policies.)
- Memory builds a summary across chat history (excluding project chats), updates on a 24-hour cadence, and provides context for new standalone conversations. Each project has a separate memory summary.
- Memory controls are explicit: toggle in Settings, pause vs reset memory, view/edit memory summary, and past-chat citations when Claude references prior conversations.

Applicable UX patterns
- Make conversation scopes explicit (project vs non-project) and show what context is active.
- Provide user-facing controls for memory/search with clear pause/reset semantics and visibility into what is stored.
- Show citations or links when prior chats are referenced.
- Provide incognito/scratch conversations that avoid persistence and search indexing.

## Keyboard shortcuts and quick entry (macOS)
Observed features
- Quick entry provides global access to Claude via double-tap Option (default), Option+Space, or a custom shortcut.
- Quick entry supports screenshot capture, application-window capture, and voice dictation. Dictation uses Caps Lock and transcribes in real time.
- Requires macOS permissions: screen recording, accessibility, and speech recognition (for voice).
- Quick entry opens a text box for a new chat and surfaces recent conversations ("New chat" shows five most recent).

Applicable UX patterns
- Provide a single, consistent global entry point with simple default shortcuts and a customization UI.
- Offer visual context capture in the entry flow (screenshot/window) with clear permission prompts.
- Keep the quick entry surface minimal: single input, optional attachments, and easy return to full app.

## Artifacts
Observed features
- Artifacts are substantial, self-contained outputs (code, docs, diagrams, single-page HTML, etc.) shown in a dedicated window to the right of chat.
- Users access a dedicated artifacts space in the sidebar to view, organize, and create artifacts.
- Artifacts support in-place updates: changes appear directly in the artifact window, and versions can be switched via a version selector.
- Multiple artifacts can be opened in one conversation; a control in the chat UI lets users switch which artifact is active for updates.
- The artifact window includes view-code, copy, and download actions.
- Artifacts can use MCP integrations; first-time tool access requires user approval and preferences persist.
- Persistent storage is supported for published artifacts (personal vs shared), with a confirmation dialog for shared data.

Applicable UX patterns
- Separate large outputs into a dedicated, persistent panel with clear affordances (view code, copy, download).
- Support versioning and multi-artifact navigation within a single conversation.
- Gate external tool access with explicit, per-artifact approvals and persistent preferences.
- For stateful artifacts, require explicit confirmation when data is shared across users.

## Context handling
Observed features
- Projects provide scoped context via knowledge base uploads and project-level instructions.
- Chat search uses RAG to retrieve relevant prior conversation context and appears as a tool call in the chat UI.
- Memory provides persistent cross-chat context, with per-project separation and user controls to pause/reset.
- Desktop extensions include a curated directory (e.g., iMessage and filesystem access), use code signing and encrypted storage, and are managed in Settings > Extensions.
- Quick entry can attach screenshots or entire app windows to add visual context.

Applicable UX patterns
- Explicit scoping: show when context is project-scoped vs global memory vs ad-hoc attachments.
- Surface retrieval as a first-class event (tool call) so users can see when context was fetched.
- Provide guardrails for connectors: clear permissions, visible source, and manageable settings.

## Streaming and responsiveness cues
Observed features
- Voice dictation in quick entry shows real-time transcription (live input feedback).
- Artifact updates appear directly in the artifact window while iterating, with versioning to compare outcomes.

Applicable UX patterns
- Provide live, incremental feedback where possible (dictation, preview panes, in-place updates).
- Pair live updates with version controls or history to avoid losing earlier results.

## Notes for Script Kit GPUI alignment
- Consider a "quick entry" flow for scripts: a global hotkey, small capture UI, optional screenshot/window attachment, and recent items list.
- Adopt project-like scoping for scripts: per-project context folders, instructions, and separate memory summaries.
- Add an artifacts-style panel for large outputs (docs, code, diagrams) with versioning and explicit export actions.
- If connectors/MCP are used, require per-tool approvals and surface tool calls in logs or UI.
- Provide explicit memory/search toggles and clear explanations of what gets stored and when it updates.
