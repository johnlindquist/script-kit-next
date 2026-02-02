# Conversation History UX Patterns for AI Chat

Scope: history sidebar, search, branching conversations, and session management.

## Pattern catalog

### History sidebar
- Persistent left sidebar list of conversations with a compact "recent" set for quick load, plus a way to reach older chats via search or on-demand fetch. This keeps the sidebar fast while still making all conversations discoverable. (ChatGPT search + fast-loading sidebar cache) [Sources 1]
- Sidebar as the primary entry point across platforms (web, desktop, mobile), with a toggle or icon to open it when collapsed. (Copilot, Claude) [Sources 2, 5]
- Conversation items show lightweight actions (rename, delete, share) from an overflow menu or hover affordance. (Microsoft 365 Copilot rename/delete; Copilot delete/share) [Sources 2, 3]
- Bulk operations for cleanup (multi-select + delete) in the history list. (Claude) [Sources 4]
- Pinned or starred section above recents to keep priority threads visible. (Gemini Enterprise: pinned/starred + recent list) [Sources 7]
- Auto-generated titles with user rename to improve scanning. (Microsoft 365 Copilot auto titles + rename) [Sources 3]
- Retention policy surfaced as part of management expectations (example: auto-delete after a set window). (Gemini Enterprise auto-delete after 60 days) [Sources 7]

### Search
- Dedicated search entry point in the sidebar or nav menu; optionally with keyboard shortcut. (ChatGPT search in left sidebar; Gemini search via nav menu) [Sources 1, 6]
- Search spans conversation titles and message content; be explicit about limitations such as exact-match only or content types excluded from indexing. (ChatGPT: title + content, exact match, canvas not searchable) [Sources 1]
- Archived chats remain searchable even if not shown in the sidebar; deleted chats are removed from search index. (ChatGPT) [Sources 1]
- Search-as-a-tool: users can prompt the assistant to retrieve relevant past chats, with explicit scope limits (all chats vs per-project). (Claude chat search + scope) [Sources 5]
- Search and reference is a toggleable capability; provide opt-out in settings and exclusion rules. (Claude search toggle; exclusion requires delete) [Sources 5]
- Temporary or incognito chats are excluded from history and search. (Claude incognito; Gemini Temporary Chat) [Sources 5, 6]

### Branching conversations
- Explicit branch/fork action creates an independent copy from a point in time; fork does not sync with the original. (TeamAI branch/fork) [Sources 8]
- Branching can be implemented as a share-based snapshot flow: generate a share link, then fork from that snapshot to avoid mutating the original. (TeamAI fork via share; Copilot share link is a snapshot) [Sources 2, 8]

### Session management
- "New chat" as a clear reset, plus optional "Temporary/Incognito chat" for non-persistent sessions. (Gemini Temporary Chat; Claude incognito) [Sources 5, 6]
- Rename, delete, and bulk delete in history to support long-term maintenance. (Claude; Copilot; Microsoft 365 Copilot) [Sources 2, 3, 4]
- Share full conversations or single responses with a preview step so users can verify the snapshot before sending. (Copilot sharing flow + preview) [Sources 2]
- Clear-all or account-level history controls for privacy. (Copilot history deletion options) [Sources 2]
- Memory/search features that span sessions can be separated from chat history storage via explicit toggles and scopes. (Claude memory + search toggles, project-scoped search) [Sources 5]

## Implications for Script Kit GPUI
- Use a fast "recents" cache in the sidebar; route older discovery through search or on-demand fetch to keep UI responsive. [Sources 1]
- Make search first-class in the sidebar, and document indexing limits (exact match, excluded content types). [Sources 1]
- Provide per-conversation actions (rename, delete, share) inline, plus multi-select bulk delete. [Sources 2, 3, 4]
- Add pinned/starred section above recents. [Sources 7]
- Implement branching as "Duplicate from message" or "Fork conversation" with an explicit note that it is a snapshot copy. [Sources 2, 8]
- Offer Temporary/Incognito sessions that do not persist to history or search. [Sources 5, 6]
- Separate "memory/search across chats" from history storage with clear toggles and scope (global vs project). [Sources 5]

## Sources
1. OpenAI Help Center - "How do I search my chat history in ChatGPT?" https://help.openai.com/en/articles/10056348-how-do-i-search-my-chat-history-in-chatgpt
2. Microsoft Support - "Conversation history in Microsoft Copilot" https://support.microsoft.com/en-gb/topic/conversation-history-in-microsoft-copilot-9a07325a-0366-4c2d-82cb-dab61be8287c
3. Microsoft Support - "Revisit your Microsoft 365 Copilot Chat history" https://support.microsoft.com/en-us/topic/revisit-your-copilot-chat-history-6ea899e3-3bb1-450a-a2ae-220341ac193a
4. Anthropic Help Center - "How can I delete or rename a conversation?" https://support.anthropic.com/en/articles/8230524-how-can-i-delete-or-rename-a-conversation
5. Claude Help Center - "Using Claude's chat search and memory to build on previous context" https://support.claude.com/en/articles/11817273-using-claude-s-chat-search-and-memory-to-build-on-previous-context
6. Gemini Apps Release Notes - "Revisit your conversations" + "Temporary Chat" (Aug 2025) https://gemini.google/ml/release-notes/
7. Google Cloud Docs - "Find and organize chats and files (Gemini Enterprise)" https://cloud.google.com/gemini/enterprise/docs/assistant-organize
8. TeamAI Help Center - "How to Branch (Fork) My Conversation" https://help.teamai.com/en/articles/8186379-how-to-branch-fork-my-conversation
