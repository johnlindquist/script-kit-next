# ChatGPT Desktop App UX Research

> Research compiled: Feb 1, 2026
> Scope: Official OpenAI docs for macOS and Windows desktop apps, plus the desktop features page.

## Sources (official)
- OpenAI desktop features page (ChatGPT on your desktop)
- OpenAI Help Center: macOS app release notes
- OpenAI Help Center: Windows app release notes
- OpenAI Help Center: How to launch the Chat Bar (macOS)

---

## Key features (desktop)

### Global access + quick launch
- Global shortcut to open ChatGPT from anywhere (Option+Space on macOS, Alt+Space on Windows).
- macOS has a lightweight Chat Bar (launcher) for quick prompts and attachments; can open from menu bar, drag anywhere, and submit with Return.
- macOS companion window provides always-on-top, side-by-side access and can be re-focused or reopened with the shortcut.

### Input + attachment workflows
- Attachments include files, screenshots, and photos (macOS Chat Bar; Windows screenshot via Snipping Tool; Windows webcam capture).
- Data analysis entry points are supported from the launcher for CSV and images.
- Desktop app highlights "screenshots" and "files" as first-class inputs on the marketing page.

### Voice and multimodal
- Advanced Voice Mode offers real-time conversation with interruption support and emotional cues.
- On macOS, Advanced Voice Mode can work alongside other apps (for live debugging, documents, and speaker notes).

### App and IDE integration
- ChatGPT can read content from supported coding apps (macOS) and can edit code directly in IDEs (macOS).
- The macOS app explicitly lists expanded support for popular coding tools (VS Code forks, JetBrains IDEs, etc.).

### Chat UX additions
- Slash commands on macOS for quick actions (search, reasoning, image generation).
- Redesigned chat bar on macOS with model picker and Temporary Chats.
- Improvements for data analysis on desktop (tables/charts expand, select cells, export charts).

### Windows-specific UX
- Search chat history with a dedicated shortcut.
- Floating sidebar mode added to the Windows app.
- Text scaling is adjustable via keyboard shortcuts.

---

## Conversation management patterns

- Global search across conversation history (macOS search bar; Windows Control+K).
- In-conversation find (macOS Command+F).
- Companion window that stays above other apps; can pop out a previous conversation into the companion window.
- Draft message restoration (macOS).
- Sidebar UX iterated: floating sidebar mode on Windows and restored sidebar toggle shortcut on macOS.
- Temporary chats surfaced directly in the chat bar (macOS).

---

## Keyboard shortcuts (officially documented)

### macOS
- Option+Space: open Chat Bar; also used to focus/reopen the companion window.
- Command+F: find text within a conversation.
- Command+.: stop streaming responses.
- Command+Shift+1: share main desktop.
- Command+Shift+2: share active window.
- Return: submit a Chat Bar prompt.

### Windows
- Alt+Space: open companion window.
- Control+K: search chat history.
- Ctrl+Plus / Ctrl+Minus: adjust text scaling.

---

## Suggestions for Script Kit AI chat window

1) Provide a global launcher and a compact "Chat Bar" mode
- Default shortcut should open a compact, draggable launcher for fast prompts and quick attachments.
- Offer a preference to customize the shortcut.

2) Add a companion window for always-on-top workflows
- A small, always-on-top chat window that can be re-opened or refocused from the shortcut.
- Allow popping an existing conversation into the companion window for side-by-side work.

3) Build first-class conversation search
- Global search across history (Control+K / Command+K pattern).
- In-thread search (Command+F pattern) for long chats.

4) Optimize attachment + screenshot capture flows
- One-click insert for screenshot (full screen, window, or region).
- Support quick file and image attachments from the launcher.

5) Include voice-first flows
- Real-time voice mode with interruption support.
- Clear visual indicators for listening, speaking, and background state.

6) Model + mode controls near the prompt
- Model picker in the chat bar.
- Temporary chats and other modes surfaced next to the input.

7) Offer app-aware context and IDE editing
- Optional integration with active apps for context-aware answers.
- Support code edits in IDEs where possible.

---

## Notes
- All features and shortcuts above are taken from OpenAI official docs and release notes as of Feb 1, 2026.
- Some items are platform-specific; mirror this pattern in our settings UI to avoid confusing cross-platform shortcuts.

## References
- https://openai.com/chatgpt/desktop
- https://help.openai.com/en/articles/9703738-desktop-app-release-notes
- https://help.openai.com/en/articles/9982051-chatgpt-windows-app-release-notes
- https://help.openai.com/en/articles/10119834-how-to-launch-the-chat-bar
