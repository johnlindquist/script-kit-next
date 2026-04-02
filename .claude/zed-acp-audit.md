# Zed ACP Agent Panel UI/UX Audit

**Audit Date:** 2026-04-01  
**Codebase:** Zed editor ACP (Agent Client Protocol) implementation  
**Key Files Analyzed:**
- `crates/agent_ui/src/conversation_view.rs` (6814 lines)
- `crates/agent_ui/src/conversation_view/thread_view.rs` (8794 lines)
- `crates/agent_ui/src/message_editor.rs`
- `crates/agent_ui/src/mode_selector.rs`

---

## 1. Message Rendering Architecture

### User Messages
**File:** `thread_view.rs:4332-4500`

User messages are rendered in bordered containers with:
- **Background:** `editor_background` color with rounded corners
- **Border:** 1px border, color changes on focus/hover
- **Padding:** `py_3() px_2()` (12px vertical, 8px horizontal)
- **Focus state:** Gold border (`focus_border` color), shadow effect on hover
- **Non-editable subagent messages:** Dashed border to indicate read-only status
- **Checkpoint button:** When message has checkpoint, displays "Restore Checkpoint" with Undo icon above the message box

```rust
// Line 4397-4426 - User message card styling
div()
    .rounded_md()
    .bg(cx.theme().colors().editor_background)
    .border_1()
    .border_color(cx.theme().colors().border)
    .when(editing && editor_focus) {
        .border_color(focus_border)
    }
    .when(!editing && !editor_focus) {
        .shadow_md()  // on hover effect
    }
```

### Assistant Messages
**File:** `thread_view.rs:4508-4578`

Assistant messages use a flexible layout with:
- **Container:** `v_flex().w_full().gap_3()` for spacing between chunks
- **Padding:** `px_5() py_1p5()` (20px horizontal, 6px vertical)
- **Bottom padding:** Extra `pb_4()` for last message only
- **Blank handling:** Empty messages completely collapse (return `Empty.into_any()`)
- **Markdown rendering:** Uses `MarkdownStyle::themed(MarkdownFont::Agent, window, cx)`
- **Context menu:** Right-click menu for copying agent response

The architecture supports two chunk types:
1. **Message chunks** → rendered via `render_markdown()`
2. **Thought chunks** → rendered via `render_thinking_block()`

### Thinking Blocks (Assistant Internal Reasoning)
**File:** `thread_view.rs:5234-5357`

Sophisticated collapsible/expandable design:

```
┌─────────────────────────────────────┐
│ 🧠 Thinking        [v] (chevron)    │  ← Header (always visible)
├─────────────────────────────────────┤
│ ┃ Content here (max-height-constrained)  ← Left border indicator
│ ┃ with gradient fade-out at bottom       (smooth overflow)
└─────────────────────────────────────┘
```

**Key features:**
- **Icon:** `IconName::ToolThink` in muted color
- **Header:** `text_size(tool_name_font_size())`, muted text color
- **Disclosure control:** Chevron icon `ChevronUp`/`ChevronDown` visible on hover
- **Left border:** `border_l_1()` with `tool_card_border_color`
- **Constrained mode:** `max_h_64()` (256px) with gradient fade overlay
- **Gradient:** Linear fade from opaque to transparent at bottom using `linear_gradient(180°)`
- **Expansion states:**
  - `ThinkingBlockDisplay::Automatic` → Starts collapsed, auto-expand on user toggle or settings
  - `ThinkingBlockDisplay::AlwaysExpanded` → Always open
  - `ThinkingBlockDisplay::AlwaysCollapsed` → Always closed
- **Auto-scroll:** `scroll_to_bottom()` when thinking block auto-expands mid-stream

---

## 2. Input Editor

**File:** `thread_view.rs:3116-3220` (main editor render)
**File:** `message_editor.rs` (MessageEditor struct and logic)

### Architecture
- **Editor mode:** `EditorMode::AutoHeight` with configurable min/max lines
- **Settings-driven:** `AgentSettings::get_global(cx).message_editor_min_lines` and `set_message_editor_max_lines()`
- **Multi-buffer support:** Uses `MultiBuffer` for input content
- **Completion provider:** Integrated slash command completions (e.g., `/context`, `/fetch`)

### Expand Icon (↗)
**File:** `thread_view.rs:3156-3187`

Located in top-right corner of editor:
```rust
h_flex()
    .absolute()
    .top_0()
    .right_0()
    .opacity(0.5)  // Faded at rest
    .hover(|this| this.opacity(1.0))  // Full opacity on hover
    .child(
        IconButton::new("toggle-height", expand_icon)  // Minimize/Maximize
            .icon_size(IconSize::Small)
            .icon_color(Color::Muted)
            .tooltip("Expand Message Editor" / "Minimize Message Editor")
    )
```

**Behavior:**
- Hidden at rest (opacity 0.5)
- Full opacity on hover
- Toggles between:
  - `IconName::Maximize` → expand editor
  - `IconName::Minimize` → collapse editor
- **Expanded state:** Takes 80vh of viewport height (`h(vh(0.8, window))`)
- **Action:** `ExpandMessageEditor` action

### Placeholder & Context
- **Mention support:** Integrated `MentionSet` for @-mentions
- **Image support:** Conditional based on `SessionCapabilities::supports_images()`
- **Slash commands:** Completion-driven with context hints (File, Symbol, Thread, Diagnostics, Fetch, Rules, BranchDiff)

---

## 3. Bottom Toolbar/Footer

**File:** `thread_view.rs:3189-3220`

Two-column layout with `justify_between()`:

### Left Column (Context & Features)
```rust
h_flex()
    .gap_0p5()
    .child(self.render_add_context_button(cx))
    .child(self.render_follow_toggle(cx))
    .children(self.render_fast_mode_control(cx))
    .children(self.render_thinking_control(cx))
```

**Buttons present:**
1. **Add Context** (`+` icon) → Opens context menu for @-mentions
2. **Follow Toggle** → Toggles auto-scroll behavior (eye icon?)
3. **Fast Mode Control** → Toggle fast mode (thunder/lightning icon?)
4. **Thinking Control** → Toggle thinking/reasoning mode

### Right Column (Config & Send)
```rust
h_flex()
    .gap_1()
    .children(self.render_token_usage(cx))
    .children(self.profile_selector.clone())
    .map(|this| {
        // Either config_options_view OR (mode_selector + model_selector)
        match self.config_options_view.clone() {
            Some(config_view) => this.child(config_view),
            None => this
                .children(self.mode_selector.clone())
                .children(self.model_selector.clone()),
        }
    })
    .child(self.render_send_button(cx))
```

**Elements in order:**
1. **Token Usage** → Context tokens display
2. **Profile Selector** (optional) → Configuration profiles
3. **Mode Selector** (or Config Options View) → Session modes (e.g., "write", "edit", "analyze")
4. **Model Selector** → Language model picker
5. **Send Button** → Primary action (likely colored/prominent)

**Layout:** Wraps flexibly (`flex_wrap()`) and adjusts with viewport

---

## 4. Streaming UX

### Generating Indicator
**File:** `thread_view.rs:5072-5152`

Appears in the list when agent is generating/executing:

```
⏳ [elapsed time] ↓ [tokens] tokens  ← Or arrows for token flow direction
```

**Elements:**
- **Spinner:** `SpinnerLabel::new()` (animated spinner)
- **Status labels:** 
  - "Awaiting Confirmation" (when waiting for tool approval)
  - `LoadingLabel::new()` component
- **Elapsed time:** Shows when `STOPWATCH_THRESHOLD` (30 seconds) exceeded
- **Token count:** When tokens exceed `TOKEN_THRESHOLD` (250)
- **Arrow icon:** 
  - `IconName::ArrowDown` → Generating (output flowing out)
  - `IconName::ArrowUp` → Waiting for confirmation

**Display logic:**
- Shows in list after last entry when `generating_indicator_in_list = true`
- Confirmation state adds sand-colored spinner

---

## 5. Tool Calls (Edit, Execute, Terminal, etc.)

### Tool Call Container
**File:** `thread_view.rs:5953-6105`

Flexible rendering based on tool kind:

```rust
fn render_any_tool_call()  // Dispatcher
├─ is_subagent() → render_subagent_tool_call()
├─ has_terminals() → render_terminal_tool_call() (per terminal)
└─ else → render_tool_call()
```

### Standard Tool Call Card
**File:** `thread_view.rs:5953-6200+`

**Card header:**
```rust
h_flex()
    .relative()
    .w_full()
    .pr_1()
    .justify_between()
    .child(
        h_flex()
            .h(window.line_height() - px(2.))
            .gap_1p5()
            .child(Tool icon)  // File edit icon, Terminal icon, etc.
            .child("Tool Label")  // e.g., "Edit `src/main.rs`"
    )
    .child(Disclosure::new().opened_icon(ChevronUp).closed_icon(ChevronDown))
```

**Tool status indicators:**
- **Completed:** Default rendered
- **Failed/Canceled:** Uses `failed_or_canceled` styling (red/muted)
- **WaitingForConfirmation:** Always expanded, shows permission buttons
- **Rejected:** Collapsed, muted coloring

### Tool Output Display
**File:** `thread_view.rs:6014-6200+`

When expanded (`is_open = true`):

**For edits (diffs):**
- Shows side-by-side or unified diff view via `AgentDiff` component
- Reveals specific ranges on demand
- Shows diff stats (lines added/removed)

**For terminal executions:**
- Embedded terminal output/PTY view
- Scrollable if output is long

**For other content:**
- **Images:** Rendered inline
- **Text/markdown:** Rendered with `render_tool_call_content()`
- **Raw input:** Collapsible "Raw Input:" section showing what was sent to the tool

**Collapsibility:**
```rust
let is_collapsible = !tool_call.content.is_empty() && !needs_confirmation;
// If collapsible and not awaiting permission, toggle-able via click
```

### Permission Buttons
**File:** `thread_view.rs:6380-6500+`

Two approval patterns:

**1. Flat buttons:**
```
[Allow Once] [Allow Always] [Reject Once] [Reject All]
```

**2. Dropdown-based (for patterns):**
```
[Allow ▼] [Reject ▼]
  └─ Dropdown menu with:
     • Once
     • Always
     • Pattern-specific ("Allow for similar operations...")
```

**Styling:**
- Buttons use `ButtonStyle::Outlined` or similar
- Accent color for "Allow" variants
- Red/Error color for "Reject" variants
- Tooltips show what "Always" / "Pattern" means

---

## 6. Markdown Rendering

### Style System
**File:** `thread_view.rs:5516-5532`, `conversation_view.rs:2290`

Uses `MarkdownStyle::themed(MarkdownFont::Agent, window, cx)` which provides:
- **Font:** `MarkdownFont::Agent` (likely monospace-friendly for code)
- **Colors:** Theme-derived
- **Insets:** Respects theme padding/margins
- **Code blocks:** Syntax highlighting (language-specific)

### Rendering Pipeline
```rust
self.render_markdown(md_entity, style)
    .into_any_element()
```

The `Markdown` entity (from `markdown` crate) handles:
- Inline code (backticks)
- Code blocks (triple backticks with language)
- Lists (ordered/unordered)
- Links (clickable, can reference files/threads via `MentionUri::parse()`)
- Bold/italic
- Headers
- Tables

---

## 7. Scroll Behavior

### Auto-scroll on New Messages
**File:** `thread_view.rs:4954-4955` (scroll_to_end)

- **When:** After new assistant message appears
- **Method:** `self.list_state.scroll_to_end()` 
- **Trigger:** Automatic on message updates (via ListState)

### Thinking Block Auto-scroll
**File:** `thread_view.rs:5261-5273`

```rust
let should_auto_scroll = self.auto_expanded_thinking_block == Some(key);
if should_auto_scroll {
    if let Some(ref handle) = scroll_handle {
        handle.scroll_to_bottom();  // Scroll within constrained thinking block
    }
}
```

### Manual Scroll Controls
**File:** `thread_view.rs:4768-4932`

Bottom-right scroll buttons:
1. **Scroll to Recent User Prompt** → `IconName::ForwardArrow`
2. **Scroll to Top** → `IconName::ArrowUp`

Both buttons are hint-opacity until hover.

---

## 8. Permission & Approval UI

### Subagent Approval
**File:** `thread_view.rs:2471-2540`

When a subagent is waiting for permission from the parent agent:

```
┌─ Subagent Awaiting Approval ──────────┐
│ [Subagent Name] requesting:           │
│ • Permission for tool X               │
│ [Allow] [Deny]                        │
└──────────────────────────────────────┘
```

Rendered in the activity bar before edits/queue sections.

### Tool Call Permissions
**File:** `thread_view.rs:6380-6500+`

When a tool call is `WaitingForConfirmation`:

1. **Inline in tool card:** Always expanded
2. **Show content preview:** What would be changed/executed
3. **Button layout:** 
   - Flat version: 4 buttons in a row
   - Dropdown version: 2 button groups with dropdowns for modality selection
4. **Pattern support:** Can select "Allow for similar patterns" (with regex/glob patterns)

---

## 9. Mode Selector (Write/Edit/Analyze)

**File:** `mode_selector.rs:1-140`

### Rendering
- **Type:** `PopoverMenu` with `ContextMenu`
- **Trigger:** Button showing current mode with dropdown indicator
- **Menu items:** Checkmark shows active mode, radio-button style selection

```rust
for mode in all_modes {
    let is_selected = &mode.id == &current_mode;
    let is_default = Some(&mode.id) == default_mode.as_ref();
    // Add menu entry with icon and checkmark if selected
}
```

### Behavior
- **Cycle action:** `CycleModeSelector` action rotates through modes
- **Apply mode:** `set_mode(mode_id)` sends to server, shows loading state
- **Feature flag gated:** May show `ConfigOptionsView` instead on newer servers

---

## 10. Model Selector

**File:** `agent_model_selector.rs`, `model_selector.rs`

### Rendering
- **Type:** `PopoverMenu` or `ModelSelectorPopover`
- **Display:** Shows current model with icon (if provider has icon)
- **Menu content:** 
  - Recently used models (favorite cycling available)
  - All available models grouped by provider
  - "Favorite" models section

### Features
- **Cycle favorite models:** `CycleFavoriteModels` action
- **Toggle selector:** `ToggleModelSelector` action
- **Async loading:** Model list fetches asynchronously from server

---

## 11. Message Queue UI

**File:** `thread_view.rs:3222-3365`

### Queue Summary
A collapsible section in the activity bar showing:
- Queue item count
- "Next in queue" indicator (circle icon in accent color)

### Queue Entries Expanded View
**File:** `thread_view.rs:3222-3365`

Each queued message shown as a row with:
```
[●] [Message text...] [Edit] [Send Now]
    (next in queue)                     (focused)

[●] [Message text...] [Trash] [Edit]
    (in queue)           (hover visible)
```

**Per-item layout:**
- **Left:** Circle icon (accent = next, muted = queued) with tooltip
- **Center:** The message text (read-only display)
- **Right (on focus):** "Edit" button + "Send Now" button with keybinding hint
- **Right (on hover):** "Delete" button + "Edit" button

---

## 12. Visual Styling Details

### Colors
- **Background:** `cx.theme().colors().panel_background` (light/dark theme aware)
- **Editor BG:** `cx.theme().colors().editor_background` (slightly darker)
- **Borders:** `cx.theme().colors().border` and `border_variant`
- **Text:** Primary, muted, error colors from theme
- **Accent:** Gold/accent color for active states and CTA buttons

### Spacing Units
- **Padding:** `px_1()` = 4px, `px_2()` = 8px, `px_3()` = 12px, `px_5()` = 20px
- **Gap:** `gap_1()` = 4px, `gap_1p5()` = 6px, `gap_2()` = 8px, `gap_3()` = 12px
- **Border radius:** `.rounded_md()`, `.rounded_sm()`, `.rounded_xs()`

### Opacity
- **Hint:** 0.40-0.55 (secondary labels, icons)
- **Muted:** 0.60-0.75 (metadata, descriptions)
- **Present:** 0.85-1.0 (primary content, active states)

---

## 13. Key Observations & Patterns

### Architectural Patterns
1. **Render delegation:** Main `render()` delegates to specialized `render_*()` functions
2. **Entity-based state:** Uses GPUI entity/context model for all stateful UI
3. **Subscription-driven updates:** Thread/action events trigger re-renders via cx.observe()
4. **Focus management:** Tracks which elements have focus for styling/UX

### UX Patterns
1. **Progressive disclosure:** Tool outputs are collapsed by default, expand on demand
2. **Inline operations:** Edit/delete buttons appear on hover or when focused
3. **Visual hierarchy:** Muted secondary info, prominent primary content
4. **Keyboard-first:** Actions have keybinding hints, all major UX accessible via keyboard
5. **Streaming indicators:** Show elapsed time and token count during generation
6. **Graceful degradation:** Subagents, terminals, edits handled specially based on tool type

### Performance Considerations
1. **Virtualized lists:** Uses `List` with processor callbacks (render on demand per visible item)
2. **Lazy content:** Tool outputs only rendered when expanded
3. **Scroll tracking:** ListState maintains scroll position across updates
4. **Markdown entity caching:** Markdown is pre-parsed into Entity<Markdown> for efficient re-renders

---

## 14. Notable Technical Details

### Window Level Management
- Thread view is in main `PopUp` window
- Child popups (selectors, menus) stay at same level via PopoverMenu/ContextMenu

### Async Patterns
- `cx.spawn()` for background tasks (loading model lists, setting modes)
- Channel-based communication with agent servers
- Futures-based task composition

### Copy & Selection
- Right-click context menu for copying agent responses
- Selection support in markdown for partial copy
- Keybinding hints in menus (e.g., `Cmd+C` for copy)

### Accessibility
- Tooltip text describes buttons and actions
- Muted vs. present color contrast for readability
- Focus tracking for keyboard navigation

---

## 15. Summary of Key Files & Line Ranges

| Feature | File | Lines |
|---------|------|-------|
| Main thread view render | `thread_view.rs` | 8475-8685 |
| Message rendering | `thread_view.rs` | 4315-4593 |
| Thinking blocks | `thread_view.rs` | 5234-5357 |
| Tool call rendering | `thread_view.rs` | 5953-6200+ |
| Permission buttons | `thread_view.rs` | 6380-6500+ |
| Activity bar | `thread_view.rs` | 2157-2252 |
| Message editor | `thread_view.rs` | 3116-3220 |
| Message queue | `thread_view.rs` | 3222-3365 |
| Generating indicator | `thread_view.rs` | 5072-5152 |
| Mode selector | `mode_selector.rs` | 1-140 |
| Message editor logic | `message_editor.rs` | 1-150+ |

---

## 16. Gaps & Opportunities vs. Script Kit

### What Zed Does Well
- ✅ Collapsible thinking blocks with gradient fade
- ✅ Sophisticated permission UI with pattern selection
- ✅ Message queue with inline editing
- ✅ Activity bar aggregating edits, plans, queue
- ✅ Separate expand icon for editor (↗)
- ✅ Mode & model selectors in toolbar
- ✅ Streaming indicator with elapsed time + token count
- ✅ Subagent approval flow

### Potential Script Kit Improvements
1. **Thinking blocks:** Implement with max-height + gradient fade
2. **Message queue:** Add inline editing + "Send Now" capability
3. **Activity bar:** Consolidate edits/approvals/queue in one expandable section
4. **Streaming feedback:** Show elapsed time and token flow arrows
5. **Pattern-based permissions:** Support pattern selection for tool approvals
6. **Expand icon:** Add ↗ button for expanding message editor

