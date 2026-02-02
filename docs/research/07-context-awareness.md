# Context Awareness Patterns for AI Chat

> Research compiled: 2026-02-01
> Purpose: Identify practical context-awareness patterns for Script Kit's AI chat window.

## Table of Contents

1. File/Folder Context
2. Clipboard Integration
3. Screen Context
4. Suggestions for Script Kit AI Chat
5. Sources

---

## 1. File/Folder Context

### Common patterns

- Default context tied to the active file and current repo, surfaced as removable context chips. Users can add or remove context explicitly. (Sourcegraph Cody chat)
- Explicit context selection via @-mentions for files, symbols, line ranges, repositories, and other artifacts. (Sourcegraph Cody chat; Sourcegraph Cody context)
- Folder-level context selection for large repos/monorepos when you do not know exact files. (Sourcegraph Cody directory mentions)
- Multi-repo context and repo-based context support for cross-repo questions. (Sourcegraph Cody context)
- Curated "spaces" or knowledge bases that collect repo content and documents for a task. (GitHub Copilot Spaces; GitHub Copilot knowledge bases)
- Path scoping inside repos so search only considers specified directories. (GitHub Copilot knowledge bases)
- Extensible context via connectors or protocols (MCP) to bring in data from other systems. (GitHub Copilot provide-context)

### Code-awareness behaviors

- Retrieval often uses multiple strategies: keyword search, native code search, and code-graph relationships to locate relevant context. (Sourcegraph Cody context)
- Tools can disclose which files were read to build trust and make context visible. (Sourcegraph Cody chat)

### Implications

- Provide a clear, editable "context set" that is visible before the user sends.
- Make context scoping easy (file, folder, repo, or curated collection) and avoid requiring exact filenames.
- Offer a fast way to narrow to paths or "areas" of a repo for targeted answers.

## 2. Clipboard Integration

### Common patterns

- Clipboard-to-structured-text actions (plain text, Markdown, JSON) with quick shortcuts and preview. (PowerToys Advanced Paste)
- AI-powered transforms on clipboard content, with optional local or cloud model providers and custom actions. (PowerToys Advanced Paste)
- Local OCR for image-to-text extraction from clipboard images. (PowerToys Advanced Paste)
- Image paste from clipboard into chat inputs for vision-capable models. (Sourcegraph Cody chat; ChatGPT Image Inputs FAQ)

### Implications

- Provide a "Paste clipboard" action that cleanly injects content into chat (and labels it).
- Offer small, reusable transforms (summarize, translate, extract tasks, convert to JSON).
- Support image paste for screenshots and diagrams; keep a lightweight preview to confirm what was pasted.

## 3. Screen Context

### Common patterns

- Explicit, user-initiated "vision" sessions where the model can see the screen or selected app windows. (Microsoft Copilot Vision)
- Scope-limited sharing (select specific apps/windows) with a clear live indicator and easy stop. (Microsoft Copilot Vision)
- Privacy assurances: no background capture, only active during a user-initiated session; content not retained beyond the session. (Microsoft Copilot Vision)

### Implications

- Screen context should be opt-in, time-bounded, and visibly active.
- Provide a "share window" picker rather than full-screen by default.
- Make privacy expectations explicit at activation and show a persistent indicator while sharing.

## 4. Suggestions for Script Kit AI Chat

### File/Folder context

- Default context chips: active file + repo/workspace.
- Add @-mention for files, folders, symbols, and line ranges.
- Folder context for monorepos (scoped to a directory).
- "Saved context sets" (like Spaces) for recurring tasks (e.g., build pipeline, onboarding, release notes).
- Path scoping UI: include/exclude paths to narrow retrieval.
- Show "context used" after each response (file list + count).

### Clipboard integration

- One-click "Paste clipboard as context" with source label.
- Quick transforms (summarize, extract tasks, convert to JSON) stored as reusable actions.
- Support image paste + OCR to turn screenshots into text summaries when needed.
- Allow optional local-only processing for sensitive clipboard content.

### Screen context

- Add "Share window" mode for live screen context (explicit start/stop).
- Limit to selected app windows and show a persistent indicator while active.
- Keep a "no-action" guarantee: guidance only, no clicks or typing.
- Provide a per-session privacy notice and a compact session log.

### UX safeguards

- "No context" mode for privacy-sensitive prompts.
- Visible context budget (token usage) and truncation warnings.
- Clear controls to remove or replace context items.

---

## Sources

- Sourcegraph Cody chat docs: https://sourcegraph.com/docs/cody/capabilities/chat
- Sourcegraph Cody context docs: https://sourcegraph.com/docs/cody/core-concepts/context
- Sourcegraph Cody directory mentions: https://sourcegraph.com/changelog/at-mention-directories
- GitHub Copilot provide-context overview: https://docs.github.com/en/copilot/how-tos/provide-context
- GitHub Copilot knowledge bases (path scoping, repo selection): https://docs.github.com/en/copilot/how-tos/provide-context/create-knowledge-bases
- GitHub Copilot Spaces (context types): https://github.com/skills/scale-institutional-knowledge-using-copilot-spaces
- Microsoft PowerToys Advanced Paste: https://learn.microsoft.com/en-us/windows/powertoys/advanced-paste
- ChatGPT Image Inputs FAQ (paste image from clipboard): https://help.openai.com/en/articles/8400551-image-inputs-for-chatgpt-faq
- Microsoft Copilot Vision support: https://support.microsoft.com/en-gb/topic/using-copilot-vision-with-microsoft-copilot-3c67686f-fa97-40f6-8a3e-0e45265d425f
