# AI Chat Window Improvements - Master Document

> Updated: 2026-02-01 | Research from 15 parallel agents

---

## Implemented Features ✅

### Core Infrastructure (Already Complete)

#### 1. Vercel AI Gateway Provider ✅
**Location:** `src/ai/providers.rs` (VercelGatewayProvider)

Full integration with Vercel AI Gateway:
- SSE streaming with proper error handling
- Model IDs in `creator/model` format (e.g., `anthropic/claude-haiku-4.5`)
- Automatic model normalization (unprefixed → `openai/`)
- Multimodal support (images)
- Curated model list: Claude Haiku/Sonnet/Opus, GPT-4o
- Env var: `SCRIPT_KIT_VERCEL_API_KEY`

#### 2. Claude Code CLI Provider ✅
**Location:** `src/ai/providers.rs` (ClaudeCodeProvider)

Headless Claude Code integration using existing subscriptions:
- Uses `claude -p --output-format stream-json`
- Session persistence via `--session-id`
- Permission mode configuration (`--permission-mode`)
- Allowed tools gating (`--allowedTools`)
- Directory context (`--add-dir`)
- Env vars: `SCRIPT_KIT_CLAUDE_CODE_ENABLED`, `SCRIPT_KIT_CLAUDE_CODE_PATH`

#### 3. Persistent Claude Sessions ✅
**Location:** `src/ai/session.rs` (ClaudeSessionManager)

Multi-turn conversations without respawning:
- `--input-format stream-json` for JSONL protocol
- Background thread parsing stdout events
- Session auto-cleanup after idle timeout
- Resume support with `--resume`

#### 4. Copy Button on Code Blocks ✅
**Location:** `src/prompts/markdown.rs`

- Copy button in code block header (next to language badge)
- Clipboard integration via GPUI
- Only shown for completed responses

#### 5. @ Context Mention System ✅
**Location:** `src/prompts/context.rs`

Raycast/Cursor-style context injection:
- `@clipboard` - Insert clipboard contents
- `@selection` - Insert selected text
- `@file:path/to/file` - Insert file contents
- `@terminal` - Insert terminal output

#### 6. Enhanced Streaming Visual Feedback ✅
**Location:** `src/prompts/chat.rs`

- Animated "Thinking..." indicator
- Blinking cursor (▌) during streaming
- Smooth chunk rendering

#### 7. Model Selector UI ✅
**Location:** `src/prompts/chat.rs`, `src/ai/window.rs`

- Dropdown showing current model
- Provider registry integration
- Quick switch via actions menu

---

## Research Summary (15 Agents)

This document synthesizes findings from parallel research on:
- Raycast AI, ChatGPT Desktop, Claude Desktop, Cursor, Copilot
- Vercel AI SDK patterns
- Claude API and CLI integration
- Accessibility and keyboard navigation

---

## Executive Summary

This document synthesizes research from 15 parallel agents analyzing AI chat improvements for Script Kit GPUI. The research covered:
- Current codebase architecture analysis
- Competitive analysis (Raycast, Cursor, Warp, GitHub Copilot)
- UX patterns (streaming, markdown, keyboard shortcuts)
- Integration patterns (Claude API, Vercel AI SDK)

---

## Current State Analysis

### Architecture Overview
- **ChatPrompt** (`src/prompts/chat.rs`): 2,191 lines - SDK chat interface
- **AI Window** (`src/ai/window.rs`): 5,717 lines - Main chat UI
- **Storage** (`src/ai/storage.rs`): SQLite + FTS5 full-text search
- **Providers** (`src/ai/providers.rs`): Multi-provider support (OpenAI, Anthropic, Google, Groq, etc.)

### Remaining Opportunities
1. **Slash commands** - `/explain`, `/fix`, `/tests` quick actions
2. **Global quick entry** - Double-tap Option overlay from anywhere
3. **Syntax highlighting** - Shiki or highlight.js for code blocks
4. **Virtual scrolling** - Optimize for long conversations
5. **Incremental markdown** - Reduce O(n²) parsing overhead
6. **Token counting** - Context window indicator
7. **Chat presets** - Save model + instructions combinations

---

## New Recommendations from Research (Feb 2026)

Based on comprehensive analysis of Raycast, ChatGPT, Claude Desktop, Cursor, and Copilot patterns.

### 1. Slash Commands and Prompt Templates (HIGH IMPACT)
**Source:** Research docs 5, 10

**What:** Quick actions via `/` commands with autocomplete.

**Built-in Commands:**
```
/explain  - Explain selected code
/fix      - Propose fixes for errors
/tests    - Generate unit tests
/summarize - Summarize with action items
/clear    - New chat session
/doc      - Generate documentation
```

**Template System:**
- Variables: `{{tone|Friendly,Professional}}` → select dropdown
- Version history for templates
- `/` opens command palette with search

**Implementation:**
```rust
// In chat input handler
if input.starts_with('/') {
    let command = parse_slash_command(&input);
    show_command_palette(command, cx);
}
```

---

### 2. Global Quick Entry Overlay (HIGH IMPACT)
**Source:** Research docs 1, 2, 3

**What:** Summon AI chat from anywhere without switching windows.

**Pattern (from Claude/Raycast/ChatGPT):**
- Double-tap Option (Claude) or Option+Space (Raycast)
- Compact Chat Bar for quick prompts
- Context capture: selected text, screenshot, clipboard
- Always-on-top companion window option

**Key Shortcuts:**
- `Cmd+N`: New chat
- `Cmd+P`: History switcher
- `Cmd+K`: Action panel

---

### 3. Enhanced Context Pills UI (MEDIUM IMPACT)
**Source:** Research docs 4, 5, 7

**What:** Visual context indicators like Cursor/Copilot.

**Features:**
- Show context chips above input (removable)
- `@file`, `@folder`, `@selection` badges
- "Context used" disclosure after response
- Token budget indicator with truncation warning

---

### 4. Code Block Action Toolbar (MEDIUM IMPACT)
**Source:** Research doc 12

**What:** Rich actions on AI-generated code blocks.

**Actions:**
| Action | Description |
|--------|-------------|
| Copy | Copy code to clipboard |
| Insert | Insert at cursor in editor |
| Apply Diff | Apply as patch with preview |
| Run | Execute in sandboxed environment |
| Save Snippet | Save to snippet library |

**Diff View:**
- Render patches as side-by-side diff
- Accept/Reject per hunk
- Checkpoint system for reverting

---

### 5. Keyboard Navigation (MEDIUM IMPACT)
**Source:** Research doc 6

**What:** Power-user keyboard support matching industry standards.

**Recommended Shortcuts:**
| Action | Shortcut |
|--------|----------|
| Open chat | Cmd+L |
| New chat | Cmd+N |
| History | Cmd+P |
| Search history | Cmd+K |
| In-chat search | Cmd+F |
| Stop generation | Cmd+. or Esc |
| Send message | Enter |
| New line | Shift+Enter |
| Prompt history | Up/Down (empty input) |
| Shortcuts overlay | Cmd+/ |

---

### 6. Conversation History & Search (MEDIUM IMPACT)
**Source:** Research doc 9

**What:** Persistent, searchable chat history.

**Features:**
- Global search with filters (date, model, keywords)
- In-conversation search (Cmd+F)
- Chat pinning and quick switch (Cmd+1-9)
- History toggle for privacy (like ChatGPT)
- Export to ZIP (HTML + JSON)

---

### 7. Accessibility Compliance (MEDIUM IMPACT)
**Source:** Research doc 13

**What:** WCAG-compliant accessible chat interface.

**ARIA Requirements:**
- Message list: `role="log"` with `aria-live="polite"`
- Status updates: `role="status"` for typing indicators
- Focus trap for modal dialogs

**Contrast:**
- Text: 4.5:1 minimum
- UI components: 3:1 for focus rings

---

---

## Implementation Status Summary

| Feature | Status | Location |
|---------|--------|----------|
| Vercel AI Gateway | ✅ Complete | `src/ai/providers.rs` (VercelGatewayProvider) |
| Claude Code CLI | ✅ Complete | `src/ai/providers.rs`, `src/ai/session.rs` |
| Model Selector | ✅ Complete | `src/prompts/chat.rs`, `src/ai/window.rs` |
| @ Context Mentions | ✅ Complete | `src/prompts/context.rs` (integrated into chat.rs) |
| Copy Button | ✅ Complete | `src/prompts/markdown.rs` |
| Streaming Cursor | ✅ Complete | `src/prompts/chat.rs` |
| Slash Commands | ✅ Complete | `src/prompts/commands.rs` (integrated into chat.rs) |
| Global Quick Entry | 🔲 Pending | - |
| Syntax Highlighting | 🔲 Pending | - |
| Virtual Scrolling | 🔲 Pending | - |

---

## Provider Configuration

### Vercel AI Gateway
```bash
export SCRIPT_KIT_VERCEL_API_KEY="your-key"
```

Models available via `creator/model` format:
- `anthropic/claude-haiku-4.5` (default)
- `anthropic/claude-sonnet-4.5`
- `anthropic/claude-opus-4.5`
- `openai/gpt-4o`
- `openai/gpt-4o-mini`

### Claude Code CLI (Use Existing Subscription)
```bash
export SCRIPT_KIT_CLAUDE_CODE_ENABLED=1
# Optional: custom path
export SCRIPT_KIT_CLAUDE_CODE_PATH=/path/to/claude
```

Uses your existing Claude Code subscription - no API key needed.

### Direct API Keys
```bash
export SCRIPT_KIT_ANTHROPIC_API_KEY="sk-..."
export SCRIPT_KIT_OPENAI_API_KEY="sk-..."
```

---

## Next Steps (Recommended Priority)

### Phase 1: Quick Wins
1. **Slash Commands** - `/explain`, `/fix`, `/tests` quick actions
2. **Syntax Highlighting** - Add Shiki/highlight.js to code blocks

### Phase 2: Power Features
3. **Global Quick Entry** - Double-tap Option overlay
4. **Chat Presets** - Save model + system prompt combinations

### Phase 3: Performance
5. **Virtual Scrolling** - Optimize long conversations
6. **Incremental Markdown** - Reduce re-parsing overhead

---

## Research Documents

All findings are synthesized from 15 parallel research agents. Full documents at `docs/research/`:

| Doc | Topic |
|-----|-------|
| 01 | Raycast AI Chat |
| 02 | ChatGPT Desktop |
| 03 | Claude Desktop |
| 04 | Cursor AI Chat |
| 05 | GitHub Copilot Chat |
| 06 | Keyboard Shortcuts |
| 07 | Context Awareness |
| 08 | Streaming Rendering |
| 09 | History Management |
| 10 | Prompt Templates |
| 11 | Model Switching |
| 12 | Code Blocks |
| 13 | Accessibility & Theming |
| 14 | Vercel AI SDK |
| 15 | Claude API |

---

## Conclusion

The AI chat infrastructure is **production-ready** with:
- ✅ Multi-provider support (Vercel AI Gateway, Claude Code CLI, OpenAI, Anthropic)
- ✅ Streaming with visual feedback
- ✅ Context injection (@-mentions)
- ✅ Model selector

**Next focus areas** should be:
1. **Slash commands** for quick actions
2. **Syntax highlighting** for code blocks
3. **Global quick entry** for system-wide access
