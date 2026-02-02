# Raycast AI Chat: Features and UX Patterns (Research)

Date: 2026-02-01
Scope: Raycast AI Chat and related AI surfaces (Quick AI, AI Commands, AI Extensions) with emphasis on keyboard shortcuts, context awareness, response streaming, conversation history, and UI/UX patterns.

## Surfaces in scope

- Quick AI: instant answers from Raycast root search (Tab to open), with follow-up questions supported in the same window. [S2][S4]
- AI Chat: separate chat window with its own history navigation and "Ask AI" hotkey. [S1]
- AI Commands: commands that can inject selected text or clipboard content via placeholders. [S13]
- AI Extensions (tools): can be invoked via @-mentions inside Quick AI, AI Commands, and AI Chat. [S14]

## Keyboard shortcuts & entry points

- Default hotkey to open Raycast is Option+Space (user-configurable). [S10]
- Quick AI is launched from the root search using Tab (fast, inline entry point). [S2]
- AI Chat opens in a separate window; Raycast provides an "Ask AI" hotkey. [S1]
- AI Chat history is accessible via Cmd+P; new chat via Cmd+N. [S1]
- AI Chat presets can be launched directly via Cmd+Shift+N. [S5]
- AI Chat attachments can be added via Cmd+Shift+A (and via a plus button). [S6]
- The Action Panel (Cmd+K) is a consistent secondary-action surface. [S11]
- Quick chat switching: Cmd+1, Cmd+2, etc. jump to the first ten chats. [S4]

## Context awareness & inputs

- "Capture Context" lets users send selected text to AI Chat from anywhere. [S1]
- AI Commands support dynamic placeholders (selected text, clipboard history, arguments). [S13]
- AI Chat supports attachments and context-aware inputs, including PDFs, browser tabs, and screenshots. [S9]
- Attachments can be added from the Action Panel and managed within chat. [S12]
- A companion browser extension can add active webpage context to Raycast AI. [S4]
- AI Extensions are discoverable via @-mentions in chat/commands, enabling tool-based context. [S14]
- Quick AI can provide web answers with citations (indicates built-in web search context). [S9]
- The Raycast Companion browser extension exposes browser history and bookmarks to Raycast AI via @browser. [S17]

## Response streaming & latency handling

- Raycast’s AI clients use response streaming with animation polish (fade-in during streaming). [S15]
- Performance improvements and bug fixes specifically target streaming responsiveness. [S15]
- Loading indicators are used while AI responses are generated (reduces perceived latency). [S15]
- Code blocks include copy actions even during AI Chat completions (stream-friendly tooling). [S2]

## Conversation history & organization

- AI Chat history is saved automatically after the first assistant response. [S1]
- A left sidebar supports chat list navigation and content search. [S3]
- Chats can be pinned, and keyboard shortcuts provide quick jumps to recent chats. [S4]
- Quick AI follow-ups can be promoted into a saved AI Chat with an auto-generated title. [S4]
- Chat branching enables alternate conversation paths from a prior message. [S8]
- Composer text is preserved when switching between chats. [S8]

## UI/UX patterns observed

- AI Chat opens in a separate, resizable floating window by default. [S2]
- Users can choose between "Always on Top" (floating) and normal window behavior; full macOS window management is supported. [S3]
- A consistent Action Panel (Cmd+K) centralizes secondary actions (attachments, tools, copy/share). [S11]
- Chat layout is adjustable: full content width is available, and text size/line spacing can be tuned. [S7][S9]
- Rich rendering features (e.g., LaTeX support) appear in AI Chat and Quick AI. [S7]
- Attachments are surfaced as first-class UI elements (plus button + action panel flows). [S6][S12]

## How these patterns apply to a chat prompt window

1) Provide multiple entry points:
   - Inline/ephemeral "Quick AI" for one-off queries (fast) and a full chat window for persistent work.
   - A clear hotkey to open chat and a Tab-style quick entry from the main launcher.

2) Make shortcuts first-class:
   - Dedicated shortcuts for new chat, history switcher, and action panel.
   - Offer quick chat switching and a compact chat switcher for power users.

3) Invest in context injection:
   - Capture selected text, clipboard, and attachments with a lightweight UI.
   - Support tool invocation via @-mentions (extension/tool routing).
   - Consider browser context integration and citation-backed web search for trust.

4) Streaming UX polish:
   - Stream responses with a stable layout and minimal reflow.
   - Use subtle streaming animations + loading indicators to reduce perceived latency.
   - Keep actions (copy/share) accessible during streaming.

5) History & recall:
   - Auto-save chats after first assistant response.
   - Left sidebar with search-by-content + pinning.
   - Preserve composer text when switching and allow branching for exploration.

6) Window & layout flexibility:
   - Allow a floating always-on-top mode and a normal window mode.
   - Provide readable typography controls and full-width layout options for longer answers.

## Sources

- [S1] https://www.raycast.com/changelog/1-53-0
- [S2] https://www.raycast.com/changelog/1-50-0
- [S3] https://www.raycast.com/changelog/macos/4
- [S4] https://www.raycast.com/changelog/1-70-0
- [S5] https://www.raycast.com/changelog/1-72-0
- [S6] https://www.raycast.com/changelog/1-77-0
- [S7] https://www.raycast.com/changelog/1-81-0
- [S8] https://www.raycast.com/changelog/1-101-0
- [S9] https://www.raycast.com/changelog
- [S10] https://manual.raycast.com/hotkey
- [S11] https://manual.raycast.com/action-panel
- [S12] https://manual.raycast.com/ai/attachments-in-ai-chat
- [S13] https://manual.raycast.com/windows/ai/ai-commands
- [S14] https://manual.raycast.com/ai-extensions
- [S15] https://www.raycast.com/changelog/ios
- [S17] https://apps.apple.com/us/app/raycast/id1636329337
