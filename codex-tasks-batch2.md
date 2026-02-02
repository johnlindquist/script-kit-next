# Batch 2: Next 7 Pressing Tasks from Today's Research

## Task 11: AI Chat - Chat Branching (Cmd+Shift+B)
**Research Finding:** Raycast pattern - explore alternate conversation paths.
**File:** `src/ai/window.rs`, `src/ai/storage.rs`, `src/ai/model.rs`

Implement:
1. Add `parent_message_id` field to Message model
2. `Cmd+Shift+B` creates a branch from current message
3. Visual indicator for branched conversations
4. Navigate between branches with `Cmd+Option+Up/Down`
5. Store branch relationships in SQLite

## Task 12: AI Chat - Presets System
**Research Finding:** Model + system instructions + creativity saved together.
**Files:** `src/ai/window.rs`, `src/ai/storage.rs`, new `src/ai/presets.rs`

Implement:
1. Create Preset struct: { name, model_id, system_prompt, temperature, tools[] }
2. Store presets in SQLite table
3. `Cmd+Shift+N` to start new chat with preset selection
4. Preset picker in chat header dropdown
5. Import/export presets as JSON

## Task 13: Notes Window - Clipboard Integration
**Research Finding:** Alfred Cmd+C+C merging pattern, Raycast encrypted storage.
**Files:** `src/notes/window.rs`, `src/clipboard_history.rs`

Implement:
1. "Append clipboard to note" action in Cmd+K menu
2. Cmd+Shift+V to append clipboard with timestamp
3. "Create note from clipboard history" - select multiple items
4. Two-way: "Save selection to clipboard history" from note

## Task 14: AI Chat - Vision/Image Support
**Research Finding:** Copilot supports attaching screenshots/diagrams.
**Files:** `src/ai/window.rs`, `src/ai/providers.rs`

Implement:
1. Drag-and-drop image onto chat input
2. Paste image from clipboard (Cmd+V with image)
3. Encode image as base64 for API
4. Display image thumbnail in message
5. Support for Claude vision and GPT-4V

## Task 15: Notes Window - Smart Folders
**Research Finding:** Auto-collect notes by tag criteria with AND/OR.
**Files:** `src/notes/window.rs`, `src/notes/storage.rs`, new smart_folders table

Implement:
1. Create SmartFolder struct: { name, filter_query, icon }
2. Filter syntax: `#tag1 AND #tag2`, `#tag1 OR #tag2`, `-#excluded`
3. Store smart folders in SQLite
4. Show smart folders in sidebar above regular notes
5. Real-time update when notes match/unmatch criteria

## Task 16: AI Chat - Prompt Caching
**Research Finding:** 90% cost reduction, 85% latency reduction from Claude.
**Files:** `src/ai/providers.rs`, `src/ai/window.rs`

Implement:
1. Add cache_control: { type: "ephemeral" } to system prompts
2. Track cached tokens in response metadata
3. Show cache hit/miss indicator in UI
4. Reuse system prompt across messages in same chat
5. Log cost savings: cached_tokens * price_per_token

## Task 17: Notes Window - Template System
**Research Finding:** Obsidian's {{date}}, {{time}}, {{title}} pattern.
**Files:** `src/notes/window.rs`, `src/notes/storage.rs`, new templates table

Implement:
1. Create NoteTemplate struct: { name, content, variables[] }
2. Variable syntax: `{{date}}`, `{{time}}`, `{{clipboard}}`, `{{selection}}`
3. "New from template" in Cmd+K menu
4. Template editor UI (simple text area)
5. Built-in templates: Daily Note, Meeting Notes, Code Snippet
