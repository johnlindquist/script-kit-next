# Prompt Suggestions & Autocomplete UX Research

Date: 2026-02-01

Scope: AI chat interfaces. Focus on suggested prompts, slash commands, template prompts, and recent prompt/history UX. Sources emphasize product docs and UX accessibility guidance.

## Executive summary

- Conversation starters work best as a small, focused set (2-5) of short, action-oriented prompts, with an explicit path to ask a custom question. This pattern is consistently recommended in chatbot UX guidance. [SiteGPT][iAdvize]
- Command-driven autocomplete is a strong discoverability pattern. GitHub Copilot Chat uses `/` for slash commands, `@` for participants, and `#` for variables, while the VS Code team documents a small set of high-value commands like `/explain`, `/fix`, and `/tests`. [GitHub Docs][VS Code Blog]
- Template prompts surface as: (a) slash-command actions (Notion AI), (b) prebuilt "AI commands" and prompt libraries (Raycast), and (c) importable prompt definitions (Raycast AI commands JSON). [Notion AI][Raycast AI][Raycast Import]
- Follow-up suggestions (e.g., "Related questions") provide a low-friction continuation path after an answer. [Kalauz]
- "Recent prompts" are usually implemented via chat history lists and search. ChatGPT keeps a cache of recent chats and supports search to retrieve older conversations. Raycast's Quick AI supports previous/next chat navigation and can auto-start a fresh chat after a configurable time window. [OpenAI Help][Raycast AI]

## Pattern catalog (with examples)

### 1) Conversation starters (empty state suggestions)

**Observed pattern**: Show a small set of suggested prompts or questions when the chat is empty. These are meant to reduce time-to-first-prompt and teach the system's capabilities.

- Recommended count: 3-5 starters, short and action-oriented. [SiteGPT]
- Recommended count: 2-5 questions, avoid overwhelming; include a "custom question" option so users can ask their own. [iAdvize]

**Why it works**
- Reduces blank-page anxiety and teaches the "shape" of good prompts.
- Provides a fast path for first-time users.

**Design notes (synthesis)**
- Use plain-language verbs (e.g., "Summarize", "Explain", "Draft").
- Label suggestions in a way that doubles as a template (so clicking inserts the full prompt).

### 2) Slash-command autocomplete and structured input tokens

**Observed pattern**: Trigger suggestion lists from a prefix character (`/`, `@`, `#`) inside the prompt input. The suggestion list functions as autocomplete and command discovery.

- GitHub Copilot Chat supports:
  - `/` for slash commands, `@` for participants, `#` for variables. [GitHub Docs]
  - Participants can be inferred automatically, but are also available via `@` suggestions. [GitHub Docs]
- VS Code documents common slash commands: `/doc`, `/explain`, `/fix`, `/generate`, `/optimize`, `/tests` (with short explanations of each). [VS Code Blog]

**Accessibility/interaction**
- ARIA combobox pattern recommends a text input that controls a listbox popup, with keyboard interactions like Arrow Down/Up to move, Enter to accept, and Escape to dismiss. [W3C ARIA]

**Design notes (synthesis)**
- Keep the slash command list short and task-oriented; show brief descriptions.
- Prioritize commands based on context (file selection, prompt length, user role).
- Suggest `@` participants or `#` variables when appropriate to reduce prompt complexity.

### 3) Template prompts and "AI commands"

**Observed pattern**: Provide templated actions with a consistent format, often as commands.

- Notion AI supports `/ai` to pick actions such as summarization and action-items, and exposes a "Custom AI block" to prompt freely. [Notion AI]
- Notion also enables "Ask AI" from selection or via slash command to apply AI to specific content. [Notion AI]
- Raycast AI offers built-in AI commands plus custom commands and a Prompt Library. [Raycast AI]
- Raycast supports importing AI commands from JSON with fields like `name`, `prompt`, `creativity`, and `model`, indicating a structured template format that's easy to share and reuse. [Raycast Import]

**Design notes (synthesis)**
- Templates should include a short "label" and a longer "prompt body."
- Use parameters (e.g., {{topic}}, {{tone}}) to keep templates adaptable.
- Treat templates as first-class items in search/command palette results.

### 4) Context-aware suggestions

**Observed pattern**: Suggestions adapt to context such as selected text, files, or domain.

- Notion's "Ask AI" flows are triggered from selected text or from a slash command, making the current document context the implicit input. [Notion AI]
- GitHub Copilot Chat uses participants and variables to inject explicit context (repositories, files, terminals) via `@` and `#` tokens. [GitHub Docs]

**Design notes (synthesis)**
- Surface "context tags" near the input to show what's being referenced.
- Suggest actions that align with the current selection (e.g., summarize selection, explain code region).

### 5) Follow-up suggestions after responses

**Observed pattern**: After delivering a response, show related questions to keep exploration going.

- Perplexity shows "Related questions" to extend the conversation after the initial answer. [Kalauz]

**Design notes (synthesis)**
- Keep follow-ups tightly related to the answer content.
- Mix question types (clarify, compare, go deeper, show examples).

### 6) Recent prompts and history

**Observed pattern**: Provide a recent chat list with search to retrieve older prompts and reduce re-entry.

- ChatGPT includes a history search in the sidebar; only recent chats are cached in the sidebar list, and search retrieves older items. [OpenAI Help]
- Raycast Quick AI allows navigating previous/next chats and can start a new chat automatically after a specified time. [Raycast AI]

**Design notes (synthesis)**
- Use recency to populate "recent prompts" chips when the input is empty.
- Provide "start fresh" vs "continue last" affordances so users can reset context.

## Pattern-to-feature mapping (cheat sheet)

| Pattern | Primary goal | Example(s) | Notes |
| --- | --- | --- | --- |
| Conversation starters | Faster time-to-first-prompt | 3-5 suggestion chips | 2-5 recommended; include "custom question" option. [SiteGPT][iAdvize] |
| Slash-command autocomplete | Discoverability + speed | `/explain`, `/fix`, `/tests` | Use `@` participants and `#` variables. [GitHub Docs][VS Code Blog] |
| Template prompts | Standardize common tasks | `/ai` actions, AI commands | Support template import/export. [Notion AI][Raycast AI][Raycast Import] |
| Context-aware prompts | Better relevance | Ask AI on selection | Suggest actions that match selection. [Notion AI] |
| Follow-up suggestions | Keep conversation flowing | Related questions | Use after-response suggestions. [Kalauz] |
| Recent prompt history | Reduce re-entry | Sidebar history + search | Cache recent, search older. [OpenAI Help] |

## Best-practice recommendations (synthesis)

1. **Limit suggestion count**
   - Keep the initial set small (2-5) to reduce cognitive load; add a "More" affordance for exploration.

2. **Use action-first labels**
   - Starter prompts should read as clear actions (summarize, draft, explain), reflecting SiteGPT/iAdvize guidance.

3. **Dual-layer suggestion UX**
   - Combine: empty-state starters + in-input autocomplete (slash/mentions) + post-answer follow-ups.

4. **Make template prompts composable**
   - Separate label vs prompt body; include parameters and defaults; allow user editing before send.

5. **Expose context explicitly**
   - When using selection or file context, show a visible "context pill" or badge to reduce surprise.

6. **Keyboard-first interaction**
   - Implement ARIA combobox semantics with Arrow/Enter/Escape support to ensure fast keyboard usage. [W3C ARIA]

7. **History-aware suggestions**
   - Surface "recent prompts" and "resume last chat" options; provide search for older prompts.

## Implementation checklist (for Script Kit GPUI)

- [ ] Empty state: 3-5 prompt chips, plus "Ask something else..." entry. [SiteGPT][iAdvize]
- [ ] Autocomplete: `/`, `@`, `#` suggestions with ARIA-combobox keyboard behavior. [GitHub Docs][W3C ARIA]
- [ ] Template library: built-in + user-defined prompts (import/export). [Raycast AI][Raycast Import]
- [ ] Context actions: "Summarize selection", "Explain code", etc. [Notion AI][VS Code Blog]
- [ ] Follow-up suggestions after response (2-4 related questions). [Kalauz]
- [ ] History: recent chat list + search to recover older prompts. [OpenAI Help][Raycast AI]

## Sources

- GitHub Copilot Chat: slash commands, participants, and variables. [GitHub Docs]
- VS Code blog: slash commands list and descriptions (/doc, /explain, /fix, /generate, /optimize, /tests). [VS Code Blog]
- Notion AI: /ai actions, Ask AI for selected text, custom AI block. [Notion AI]
- Raycast AI: AI commands, prompt library, Quick AI and chat navigation. [Raycast AI]
- Raycast AI: import AI commands JSON structure. [Raycast Import]
- OpenAI Help: search chat history, cached recent chats. [OpenAI Help]
- SiteGPT: conversation starter best practices (3-5, short, action-oriented). [SiteGPT]
- iAdvize: conversation starters best practices (2-5, personalized, custom question option). [iAdvize]
- W3C ARIA: combobox pattern and keyboard interactions. [W3C ARIA]
- Kalauz (library guide) on Perplexity: related questions after answers. [Kalauz]

[GitHub Docs]: https://docs.github.com/en/copilot/github-copilot-chat/about-github-copilot-chat
[VS Code Blog]: https://devblogs.microsoft.com/visualstudio/learn-more-about-github-copilot-chat-features/
[Notion AI]: https://www.notion.so/help/guides/notion-ai-for-docs
[Raycast AI]: https://manual.raycast.com/raycast-ai
[Raycast Import]: https://manual.raycast.com/raycast-ai#how-to-import-ai-commands
[OpenAI Help]: https://help.openai.com/en/articles/10769701-chatgpt-search
[SiteGPT]: https://sitegpt.ai/docs/how-to/add-conversation-starters
[iAdvize]: https://help.iadvize.com/hc/en-us/articles/17632186420114-Conversation-starters
[W3C ARIA]: https://www.w3.org/WAI/ARIA/apg/patterns/combobox/
[Kalauz]: https://libguides.vwu.edu/c.php?g=753528&p=11142175
