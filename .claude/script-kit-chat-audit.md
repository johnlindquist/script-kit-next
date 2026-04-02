# Script Kit Chat UI Audit

**Audit Date:** 2026-04-01  
**Codebase:** Script Kit GPUI - ChatPrompt component  
**Key Files Analyzed:**
- `src/prompts/chat/mod.rs` (82 lines)
- `src/prompts/chat/prompt.rs` (553 lines) - ChatPrompt struct
- `src/prompts/chat/render_core.rs` (700 lines) - Footer & core rendering
- `src/prompts/chat/render_turns.rs` (263 lines) - Message rendering
- `src/prompts/chat/render_input.rs` (119 lines) - Input field
- `src/prompts/chat/state.rs` (680 lines) - State management
- `src/prompts/chat/streaming.rs` (504 lines) - Streaming implementation
- `src/protocol/types/chat.rs` - Chat message types

---

## 1. Architecture Overview

### Core Design Philosophy
**Input at TOP (not bottom)** — unique inversion from traditional chat interfaces

```
┌────────────────────────────────────┐
│ ← Chat                    [Header] │
├────────────────────────────────────┤
│ [Input field (at top)]             │  ← Unconventional placement
├────────────────────────────────────┤
│                                    │
│  [User Prompt - small, bold]       │  ← Conversation turns
│  [Assistant Response (markdown)]   │
│                                    │
│  [User Prompt]                     │
│  [Assistant Response]              │
│                                    │
├────────────────────────────────────┤
│ Model · Shift+Enter newline   [Run]│  ← Footer toolbar
└────────────────────────────────────┘
```

### Core Concepts
- **Conversation Turns:** User prompt + AI response bundled as single container
- **Full-width containers:** Not chat bubbles
- **Streaming reveal:** Word-buffered progressive disclosure
- **Mini mode:** Borderless input matching main window aesthetics

---

## 2. Message Rendering

### Conversation Turn Structure
**File:** `render_turns.rs:10-200`

Each turn is a full-width container with:

```rust
div().flex().flex_col().gap(px(6.0)).w_full()
    .child(user_prompt)  // Small, bold, secondary text
    .children(user_image)  // Optional thumbnail
    .children(error_state)  // If generation failed
    .children(assistant_response)  // Markdown-rendered
```

### User Prompt
- **Typography:** `text_sm()`, `font_weight(SEMIBOLD)`
- **Color:** `theme_colors.text.secondary` (muted)
- **Display:** Only shown if non-empty
- **Image attachment:** Small 64x64px thumbnail below text

### Assistant Response
- **Markdown rendering:** Uses `render_markdown()` helper with prompt colors
- **Streaming state:** Shows "Thinking..." placeholder while waiting
- **Streaming rendering:** Separates markdown cache from cursor to avoid invalidation on every frame
- **Complete response:** Full markdown with proper wrapping

**Key line (127-143):**
```rust
if turn.streaming && response.is_empty() {
    // Empty streaming state
    content = content.child(div().text_xs().opacity(0.6).child("Thinking..."));
} else if turn.streaming {
    // Streaming with content - separate cursor from markdown
    content = content.child(
        div()
            .w_full()
            .min_w_0()
            .overflow_x_hidden()
            .child(render_markdown(markdown_response.as_ref(), colors)),
    );
} else {
    // Complete response
    content = content
        .child(render_markdown(markdown_response.as_ref(), colors).overflow_x_hidden());
}
```

### Error Handling
**File:** `render_turns.rs:59-118`

When generation fails:
- Shows error message with user-friendly wrapper (`ChatErrorType::from_error_string()`)
- Truncated raw error detail (200 chars for known errors, 400 for unknown)
- Optional "Retry" button (colored with error background, hover effect)

---

## 3. Input Area

**File:** `render_input.rs:1-120`

### Layout
- **Header:** Back arrow + title
- **Input field:** Bordered card (in full mode) or bare text (in mini mode)
- **Min height:** 28px
- **Full width:** `w_full()`

### Focus State
- **Idle:** Translucent search box background + dim border
- **Focused:** Gold border with higher opacity

**Alpha constants:**
```rust
CHAT_LAYOUT_INPUT_BG_FOCUSED_ALPHA = 0xC0    // 75% opaque
CHAT_LAYOUT_INPUT_BG_IDLE_ALPHA = 0x90       // 56% opaque
CHAT_LAYOUT_INPUT_BORDER_FOCUSED_ALPHA = 0x90  // 56%
CHAT_LAYOUT_INPUT_BORDER_IDLE_ALPHA = 0x55    // 33%
```

### Placeholder
- Default: "Ask follow-up..."
- Customizable via `placeholder` prop
- Shown as muted text when input empty

### Cursor
- **Text input component:** Custom cursor/selection rendering
- **Blink animation:** 530ms timer cycle
- **Visibility:** Only shown when focused

### Mini Mode
- **No inner padding:** Outer `input_area` handles spacing
- **Font size:** 16px (matches mini main window's visual size)
- **Card styling:** Removed — bare text input
- **Use case:** Tab AI harness terminal or compact chat

---

## 4. Footer Toolbar

**File:** `render_core.rs:34-142`

### Mini Mode Footer
- Simple hint strip: "↵ Run · ⌘K Actions · Tab AI"
- Minimal styling, matches universal prompt footer

### Rich Footer
- **Left slot (optional):** Script generation actions (Save and Run status)
- **Primary button:** Context-aware
  - During generation: "Stop" (Esc)
  - With assistant response: "Continue in AI Chat" (⌘↵)
  - In script mode: "Save and Run" (⌘↵)
  - Default: "Actions" (⌘K)
- **Secondary button:** Always "Actions" (⌘K)
- **Helper text:** "Model · Shift+Enter newline"

### Colors & Styling
- Uses `PromptFooterColors::from_theme()`
- Buttons with hover/active opacity states
- Accent color for primary action
- Muted color for helper text

---

## 5. Streaming & Reveal

**File:** `streaming.rs` (504 lines)

### Word-Buffered Reveal
Accumulates full streamed content, reveals word-by-word:
- **Watermark tracking:** `builtin_reveal_offset` tracks where we've revealed to
- **Update loop:** Each stream chunk updates offset, triggers re-render
- **Performance:** Prevents invalidating markdown on every character

### Auto-scroll Behavior
- **User hasn't scrolled:** Auto-scroll to bottom on new content
- **User scrolled up:** Stop auto-scrolling (preserve scroll position for review)
- **Reset:** On explicit "jump to latest" or new submission

**State:**
```rust
pub(super) user_has_scrolled_up: bool,  // Disables auto-scroll
```

### Streaming Message ID
- Tracks which message is currently streaming
- Stops highlighting when generation completes

---

## 6. Model Selection

**File:** `prompt.rs:8`

- **Property:** `model: Option<String>`
- **List:** `models: Vec<ChatModel>`
- **Display:** Shown in footer helper text "Model · Shift+Enter..."
- **Selection:** Via footer footer integration (PopoverMenu handled by parent)

---

## 7. Actions Menu (⌘K)

**File:** `actions.rs` (534 lines)

Callback-based system:
```rust
pub(super) on_show_actions: Option<ChatShowActionsCallback>,
```

When user presses ⌘K or clicks "Actions" button:
- Calls parent handler (likely opens full actions dialog)
- Not rendered within ChatPrompt itself
- Parent responsible for showing models, modes, etc.

---

## 8. Colors & Theme

### Prompt Colors
```rust
pub(super) prompt_colors: theme::PromptColors
```

Components from theme:
- `text_primary`, `text_secondary`, `text_muted`
- `quote_border`, `code_background`
- `accent.selected` (gold)

### Vibrancy
**File:** `render_turns.rs:19-26`

Container background uses theme-aware overlay:
- **Dark mode:** ~8% white overlay (brightness)
- **Light mode:** ~3% black overlay (subtle shadow)
- **Hover:** ~16% overlay for copy button feedback

```rust
let container_bg = if self.theme.is_dark_mode() {
    theme::hover_overlay_bg(&self.theme, 0x15)  // 8% white
} else {
    theme::hover_overlay_bg(&self.theme, 0x08)  // 3% black
};
```

---

## 9. Markdown Rendering

**File:** `render_turns.rs:121-144`, `src/prompts/markdown.rs`

- Uses custom `render_markdown()` helper
- Receives `PromptColors` for consistent styling
- Supports inline code, code blocks, lists, links
- Handles image attachments

---

## 10. Image Support

**File:** `prompt.rs:71-72`

- **Pending image:** `Option<String>` (data URI or file path)
- **Render cache:** `HashMap<String, Arc<RenderImage>>`
- **Image in turn:** `turn.user_image: Option<RenderImage>`
- **Display:** 64x64px thumbnail with rounded corners

---

## 11. Built-in AI Provider Support

**File:** `prompt.rs:26-36`

Enables inline chat without external SDK:
- **Provider registry:** `ProviderRegistry` for managing LLM connections
- **Available models:** `Vec<ModelInfo>`
- **System prompt:** `builtin_system_prompt`
- **Streaming:** `builtin_streaming_content`, `builtin_is_streaming`
- **Reveal:** `builtin_accumulated_content`, `builtin_reveal_offset`

Used by Tab AI harness terminal mode.

---

## 12. Message Queue

**Status:** NOT IMPLEMENTED in ChatPrompt

ChatPrompt is single-message (input at top → submit).
No queue of pending messages like Zed.

---

## 13. Permission/Approval UI

**Status:** NOT IMPLEMENTED in ChatPrompt

All tool calls, file edits, etc. are out-of-scope for this component.
ChatPrompt is pure message input/output.

---

## 14. Thinking Blocks

**Status:** NOT IMPLEMENTED in ChatPrompt

Thinking/internal reasoning not exposed as collapsible blocks.
All streaming content flows through single markdown renderer.

---

## 15. Script Generation Mode

**File:** `prompt.rs:52-55`, `render_core.rs:93-94`

Special mode for generating runnable scripts:
```rust
pub(super) script_generation_mode: bool,
pub(super) script_generation_status: Option<String>,
pub(super) script_generation_status_is_error: bool,
```

When enabled:
- Primary button changes to "Save and Run" (⌘↵)
- Footer shows generation status (success/error)
- Callbacks for script save/run passed to parent

---

## 16. Setup Mode

**File:** `prompt.rs:50-51`, `render_setup.rs`

Shows API key configuration card instead of chat:
```rust
pub(super) needs_setup: bool,
pub(super) setup_focus_index: usize,
pub(super) on_configure: Option<ChatConfigureCallback>,
pub(super) on_claude_code: Option<ChatClaudeCodeCallback>,
```

Renders provider setup UI with keyboard navigation.

---

## 17. Key State Management

**File:** `state.rs` (680 lines)

Core state mutations:
- **Conversation turns cache:** Lazy-computed from messages via `build_conversation_turns()`
- **Cache invalidation:** `conversation_turns_dirty` flag
- **Focus management:** `focus_handle` for keyboard shortcuts
- **Cursor blinking:** Async timer task
- **Scroll state:** `turns_list_state` (GPUI ListState)

---

## 18. Keyboard Shortcuts

**File:** `types.rs`, `actions.rs`

- **Enter:** Submit (or newline if Shift+Enter)
- **Cmd+K:** Show actions menu
- **Cmd+↵:** Continue in chat or save/run script
- **Esc:** Escape/stop streaming

---

## 19. Copy Functionality

**File:** `render_turns.rs:147-180+`

Copy button on right side of each turn:
- **Trigger:** Hover on turn or always visible
- **Action:** Copy assistant response to clipboard
- **Feedback:** Shows "Copied!" confirmation

---

## 20. Type System

**File:** `src/protocol/types/chat.rs:1-100`

### ChatMessageRole
```rust
pub enum ChatMessageRole {
    System,
    User,
    Assistant,  // default
    Tool,
}
```

### ChatPromptMessage
- **AI SDK compatible:** `role`, `content` fields
- **Script Kit compatible:** `position` (Left/Right), `text`
- **Metadata:** `name`, `model`, `streaming`, `error`, `created_at`, `image`

---

## 21. Visual Styling Constants

**File:** `mod.rs:49-64`

```rust
CHAT_LAYOUT_PADDING_X = 12.0
CHAT_LAYOUT_SECTION_PADDING_Y = 8.0
CHAT_LAYOUT_CARD_PADDING_X = 12.0
CHAT_LAYOUT_CARD_PADDING_Y = 10.0
CHAT_LAYOUT_BORDER_ALPHA = 0x40       // ~25%
CHAT_LAYOUT_INPUT_BG_FOCUSED_ALPHA = 0xC0    // ~75%
CHAT_LAYOUT_INPUT_BG_IDLE_ALPHA = 0x90       // ~56%
CHAT_LAYOUT_INPUT_BORDER_FOCUSED_ALPHA = 0x90  // ~56%
CHAT_LAYOUT_INPUT_BORDER_IDLE_ALPHA = 0x55    // ~33%
CHAT_LAYOUT_FOOTER_BG_DARK_ALPHA = 0x24       // ~14%
CHAT_LAYOUT_FOOTER_BG_LIGHT_ALPHA = 0x14      // ~8%
```

---

## 22. Summary of Key Files

| Component | File | Lines |
|-----------|------|-------|
| Core struct & lifecycle | `prompt.rs` | 553 |
| Footer & main render | `render_core.rs` | 700 |
| Message turn rendering | `render_turns.rs` | 263 |
| Input field styling | `render_input.rs` | 119 |
| State management | `state.rs` | 680 |
| Streaming reveal logic | `streaming.rs` | 504 |
| Actions & handlers | `actions.rs` | 534 |
| Type definitions | `types.rs` | 672 |
| Protocol messages | `../protocol/types/chat.rs` | 100+ |

---

## 23. Current Limitations

1. **No message queue** — Single active message only
2. **No thinking blocks** — Streaming content is undifferentiated
3. **No permission UI** — All tool/approval flows out-of-scope
4. **No tool call cards** — No edit/execute/terminal rendering
5. **Top-placed input** — Unusual UX compared to standard chat
6. **No auto-scroll elapsed time** — No duration indicator while generating
7. **No activity bar** — No consolidated edits/plans/queue section
8. **No pattern-based approvals** — No regex/glob permission UI
9. **No subagent flow** — No nested agent support

---

## 24. What Script Kit Does Well

✅ **Simple, clean design** — Input at top inverts expectation, focuses on clarity
✅ **Word-buffered streaming** — Smooth reveal without markdown cache thrashing
✅ **AI SDK compatibility** — Dual `role/content` and `position/text` fields
✅ **Image support** — Embedded thumbnails + base64 data URIs
✅ **Script generation mode** — Context-aware footer for automation
✅ **Theme-aware vibrancy** — Subtle overlay backgrounds
✅ **Markdown rendering** — Full support with prompt colors
✅ **Mini mode** — Compact input matching main window aesthetics
✅ **Keyboard-first** — All actions accessible via shortcuts
✅ **Status feedback** — Shows "Thinking...", generation errors, retry

