# AI Chat History and Conversation Management (Research)

Goal: summarize evidence from existing products/research and turn it into concrete design suggestions for the Script Kit AI chat window.

---

## Evidence and patterns

### History persistence + privacy controls
- OpenAI's April 25, 2023 ChatGPT update added a "turn off chat history" control: conversations started with history disabled do not appear in the sidebar and are not used for training; OpenAI retains those conversations for 30 days for abuse monitoring, then deletes them. [S1]
- The ChatGPT iOS FAQ notes that history has a searchable list and allows deleting a conversation from the history view. [S2]

### Search and organization
- Microsoft Teams search supports filters after a query (e.g., messages/people, location, sender, date, and more), and provides in-conversation search via Cmd/Ctrl+F. [S3]
- Teams search results can be filtered by channels vs chats and sorted by Top vs Latest, with additional filters like "From" and "Has mentions." [S4]

### Threading and conversation grouping
- Slack threads are positioned as a way to keep discussions organized around specific messages and reduce channel clutter; replies can stay in the thread or be sent to the channel, and the Threads view aggregates replies (unread first). [S5]
- Outlook's conversation view groups related messages into a single conversation (based on the same subject) and allows switching between conversation-grouped and individual message views. [S6]

### Export and portability
- ChatGPT data export (via Data Controls) delivers a downloadable .zip that includes chat history in `chat.html`; the email download link expires after 24 hours. [S7]

### Integrity considerations
- Recent research shows chat-history tampering attacks can alter LLM behavior, motivating detection and mitigation for untrusted history. [S8]

---

## Implications for our AI chat window

1. **Persistence must be optional and explicit.** Users expect an easy on/off history toggle and clear retention behavior (like OpenAI's 30-day retention for "history off" conversations). [S1]
2. **Search needs both global and in-conversation modes.** Teams' model suggests global filters plus per-thread search shortcuts. [S3]
3. **Threading should reduce clutter, not add it.** Slack's "reply in thread + optionally send to main" is a strong default. [S5]
4. **Provide a "flat vs grouped" view toggle.** Outlook's conversation view shows that some users prefer unthreaded lists. [S6]
5. **Export should be a first-class, time-limited download.** The ChatGPT export flow sets a clear expectation for ZIP + HTML with expiring links. [S7]
6. **Treat history as untrusted input.** The tampering research implies we should preserve provenance and guardrails. [S8]

---

## Recommendations for Script Kit (AI window)

### 1) History persistence
- **Add a History toggle** ("Save chats" on/off) that mirrors ChatGPT's model: when off, conversations don't appear in history and follow a clear retention policy. [S1]
- **Expose per-chat delete** from the history list, since users expect that control in a chat UI. [S2]
- **Default to local-first storage** (SQLite) with optional sync later; show storage location + retention in Settings.

### 2) Search
- **Global search across all chats** with filters (role/sender, date, location/context, message type) modeled after Teams filters. [S3]
- **In-conversation search** (Cmd/Ctrl+F) with jump-to-result, matching Teams' per-thread search. [S3]
- **Result ordering toggle** (Top vs Latest / Relevance vs Recency) similar to Teams' search sorting. [S4]
- **History search field in sidebar** for quick retrieval, consistent with ChatGPT's history search pattern. [S2]

### 3) Threading + branching
- **Per-message threads** with a "Reply in thread" action; add a checkbox or shortcut to "also post to main" (Slack). [S5]
- **Threads view** that aggregates active/unread thread replies so users can catch up quickly. [S5]
- **Flat vs threaded toggle** at the conversation level, following Outlook's conversation-view pattern. [S6]

### 4) Export
- **One-click Export** from Settings that generates a ZIP containing HTML + machine-readable JSON/MD, mirroring ChatGPT's export expectations. [S7]
- **Time-limited download links** (or local file expiry prompts) to reduce exposure for exported data. [S7]

### 5) Integrity + provenance
- **Record message provenance** (system/tool/user/imported) and surface it in the UI to reduce confusion and mitigate tampering risks noted in recent research. [S8]
- **Immutable audit metadata** (timestamps, model, tool calls) to detect or explain altered history.

---

## Open questions for product/design

1. Should "History off" be a global setting, per-chat flag, or both?
2. Do we want full-text search across tool outputs and file attachments?
3. Should threads be treated as separate chats in storage/export, or as branches of a single conversation?
4. What export formats should be default (HTML + JSON) vs optional (Markdown, PDF)?

---

## Sources

- [S1] OpenAI - "New ways to manage your data in ChatGPT" (Apr 25, 2023)
  https://openai.com/index/new-ways-to-manage-your-data-in-chatgpt/
- [S2] OpenAI Help - "ChatGPT iOS app" (FAQ)
  https://help.openai.com/en/articles/7900486-chatgpt-ios-app-faq
- [S3] Microsoft Support - "Search for messages and more in Microsoft Teams"
  https://support.microsoft.com/en-us/office/search-for-messages-and-more-in-microsoft-teams-2e350591-78ea-4e0d-9827-5a5b263f34f0
- [S4] Microsoft Support - "Search and filter messages in Microsoft Teams"
  https://support.microsoft.com/en-us/office/search-and-filter-messages-2e350591-78ea-4e0d-9827-5a5b263f34f0
- [S5] Slack Help - "Use threads to organize discussions"
  https://slack.com/help/articles/115000769927-Use-threads-to-organize-discussions
- [S6] Microsoft Support - "View messages by conversation"
  https://support.microsoft.com/en-us/office/view-messages-by-conversation-0eeec76c-49c8-4ad9-a42c-94c4630568f7
- [S7] OpenAI Help - "How do I export my ChatGPT history and data?"
  https://help.openai.com/en/articles/7260999-how-do-i-export-my-chatgpt-history-and-data%3F
- [S8] arXiv - "Chat History Tampering Attack to LLMs via Poisoned Data"
  https://arxiv.org/abs/2408.11365
