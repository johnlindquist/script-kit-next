# ACP Chat View Rendering Audit

**Date:** 2026-04-01  
**Scope:** Script Kit GPUI ACP chat implementation  
**Files Audited:**
- `src/ai/acp/view.rs` (903 lines) — Main ACP chat view rendering
- `src/ai/acp/thread.rs` (1240 lines) — Conversation state machine
- `src/ai/acp/permission_broker.rs` (568 lines) — Permission UI
- `src/ai/acp/types.rs` (82 lines) — Bridging types

---

## 1. Message Card Rendering

### Card Structure (view.rs:188–240)

**Role-specific styling** uses hardcoded alpha values overlaid on theme colors:

```
User:        bg=accent.selected<<8|0x16,  border=accent.selected<<8|0x44, title_opacity=0.82
Assistant:   bg=background.search_box<<8|0xF2, border=ui.border<<8|0x70, title_opacity=0.72
Thought:     bg=text.primary<<8|0x08,     border=text.primary<<8|0x22, title_opacity=0.62
Tool:        bg=accent.selected<<8|0x10,  border=accent.selected<<8|0x38, title_opacity=0.72
System:      bg=text.primary<<8|0x08,     border=ui.border<<8|0x60,  title_opacity=0.62
Error:       bg=0xEF4444(red)<<8|0x20,    border=0xEF4444<<8|0x80,   title_opacity=0.86
```

**Card geometry:**
- Padding: `px(12.0)` horizontal, `px(10.0)` vertical
- Border: 1px solid (theme color dependent)
- Corner radius: `px(10.0)` (rounded corners)
- Width: full container width (`w_full()`)

**Card children structure:**
```
div (card container)
  └─ div (role title row)
     └─ text: "You" / "Thinking" / "Claude Code" / "Tool" / "System" / "Error"
  └─ markdown content (full width)
```

**Issues:**
1. **Role labels are bold and prominent** (FontWeight::SEMIBOLD, opacity 0.6–0.82) — very visible, not whisper chrome
2. **Every card has a visible border** — gold-tinted (accent color) for User/Tool cards, which is visually heavy
3. **Background colors are slightly visible** — alpha values like 0x16, 0xF2 create subtle but noticeable backgrounds
4. **Rounded corners on every card** — adds visual weight; softer rounded corners than Zed's sharper cards

---

## 2. Empty State Rendering (view.rs:242–306)

**Display conditions:**
- Shows when message list is empty (`is_empty`)
- No commands strip when not at Idle status or when messages exist
- Text-centered vertical layout with icon-like design

**Structure:**
```
Centered flex column:
  └─ Title (text_base, FontWeight::SEMIBOLD, opacity=0.9)
  └─ Detail (text_sm, opacity=0.65)
  └─ Optional context note (if present)
     └─ Rounded pill bg with accent color, slight opacity
  └─ Footer text: "Claude Code over ACP" (gold/accent colored)
```

**Issues:**
1. **Centered text layout** — wastes vertical space; Zed would use top-aligned layout
2. **Large title with full opacity** — demands attention; should be more recessive
3. **Hardcoded footer text** → should be configurable or removed

---

## 3. Role Title Labels

**Rendering (view.rs:177–186):**

```rust
fn role_title(role: AcpThreadMessageRole) -> SharedString {
    match role {
        User => "You",
        Assistant => "Claude Code",
        Thought => "Thinking",
        Tool => "Tool",
        System => "System",
        Error => "Error",
    }
}
```

**Styling (view.rs:230–237):**
- `text_xs()` — small text
- `FontWeight::SEMIBOLD` — **bold**
- `opacity(title_opacity)` — 0.62–0.86 (varies by role)
- `pb(px(6.0))` — space below label

**Issues:**
1. **Always present and bold** — occupies first line of every card, visually dominant
2. **Semantic labels are good**, but style is not minimal — should be smaller or use color-coding instead
3. **No icon differentiation** — just text labels

---

## 4. Input Area (view.rs:875–893)

**Structure (footer):**

```
div (footer)
  ├─ div.flex_grow() (input text area)
  │  └─ text_sm: input content
  ├─ div (footer hint)
  │  └─ text_xs, opacity=0.45
  ├─ Status badge
  └─ Mode badge (when active)
```

**Styling:**
- Padding: `p_2()` (standardized spacing)
- Flex: `items_center`, `gap_2()` — row layout
- Input text is displayed as plain text (not interactive input widget)
- Background: none (inherits from parent)

**Issues:**
1. **Not a real text input widget** — just displays `thread.input` as text
2. **Key handling is manual** (view.rs:708–780) — character accumulation via key_down events
3. **No visual input affordance** — looks like static text, not an interactive field
4. **Footer is cramped** — 3 elements (text, hint, status, mode) fight for space

---

## 5. Status Badge (view.rs:351–383)

**Rendering:**

```
div (badge container)
  ├─ px(8.0), py(4.0) padding
  ├─ rounded(999.0) — pill shape
  ├─ Conditional bg based on status:
  │  ├─ Idle + no permission: "Ready", muted bg
  │  ├─ Idle + permission pending: "Permission required", accent bg
  │  ├─ Streaming: "Streaming", gold accent bg
  │  ├─ WaitingForPermission: "Permission required", accent bg
  │  └─ Error: "Error", red bg
  ├─ text_xs(), opacity=0.8
  └─ Label text
```

**Issues:**
1. **Always visible** — takes up footer space
2. **Status text is redundant** with footer hint text (view.rs:308–327) — both show streaming state
3. **Not minimal** — should only appear when needed (error state) or be smaller

---

## 6. Footer Hint Text (view.rs:308–327)

**Logic:**
```
if has_pending_permission:
  "Choose an option to continue"
else if Streaming:
  "Claude Code is working…"
else if Preparing + queued:
  "Queued · sending when context is attached…"
else if Preparing:
  "Attaching context…"
else if Failed:
  "Enter to send · context partial"
else:
  "Enter to send"
```

**Issues:**
1. **Static text, not interactive** — no affordance to press Enter
2. **Multi-part messages** use `·` separator instead of structure
3. **Competes with status badge** for attention

---

## 7. Streaming Indicator (view.rs:329–349)

**Rendering:**

```
div (streaming hint row)
  ├─ div.size(6px), rounded_full, gold bg (dot)
  └─ div.text_xs(), opacity=0.7: "Streaming response…"
```

**Placement:** Above footer, only when status is Streaming (view.rs:865–872)

**Issues:**
1. **Redundant with status badge** — both indicate streaming
2. **Weak signal** — small dot is easy to miss
3. **Text is italicized metaphor** ("…") instead of technical clarity

---

## 8. Commands Strip (view.rs:649–675)

**Rendering:**

```
div (commands container)
  ├─ w_full, px(12), py(6)
  ├─ rounded(8), bg=text.primary<<8|0x06 (very faint)
  ├─ border_1, border_color=ui.border<<8|0x20
  ├─ div: "Commands" label (text_xs, SEMIBOLD, opacity=0.62)
  └─ div: comma-space-separated command names (text_xs, opacity=0.58)
```

**Placement:** Only when:
- Thread is empty (`is_empty`)
- Status is Idle
- `available_commands` is not empty

**Issues:**
1. **Gated too restrictively** — only shows on empty idle state, not during streaming
2. **Sparse content** — list of bare command strings (e.g., "/context", "/browser")
3. **No interactivity** — read-only display, not clickable actions
4. **Low opacity** — hard to read the commands

---

## 9. Plan Strip (view.rs:677–704)

**Rendering:**

```
div (plan container)
  ├─ w_full, px(12), py(8)
  ├─ rounded(8), bg=accent.selected<<8|0x0C (very faint gold)
  ├─ border_1, border_color=accent.selected<<8|0x28
  ├─ div: "Plan" label (text_xs, SEMIBOLD, opacity=0.7)
  └─ children (numbered list)
     └─ For each entry: "{i+1}. {entry}" (text_xs, opacity=0.65)
```

**Placement:** Only when `plan_entries` is not empty (view.rs:845–853)

**Issues:**
1. **Bare numbered list** — no visual differentiation between planned vs. completed steps
2. **No collapse/expand** — all steps always visible
3. **Text-only** — should show progress or checkmarks

---

## 10. Permission Overlay (view.rs:544–631)

### Overlay Backdrop (view.rs:553–562)

```
div (absolute full coverage)
  ├─ top_0, left_0, right_0, bottom_0
  ├─ bg: modal_overlay_bg (semi-transparent dark)
  ├─ flex, items_center, justify_center
  └─ (content div below)
```

### Modal Card (view.rs:564–629)

**Dimensions:**
- Width: `w(px(640.0))`, max_w_full, mx_4
- Padding: `p_4()`
- Rounded: `px(14.0)`
- Background: `rgb(theme.colors.background.search_box)`
- Border: 1px, `border_color=ui.border<<8|0x99`

**Content structure:**

```
Modal card:
  ├─ Title (text_base, SEMIBOLD)
  ├─ Structured preview sections (when available):
  │  ├─ Header row: badge + tool title + subject
  │  ├─ Summary section (if present)
  │  ├─ Input section (if present)
  │  ├─ Output section (if present)
  │  └─ Option summary line (if options present)
  ├─ Fallback body (if no preview)
  ├─ Option rows (enumerated, with keyboard hints)
  └─ Keyboard hint strip
```

### Permission Header (view.rs:410–466)

```
div (header section):
  ├─ pt(8)
  ├─ Flex row (items_center, gap_2):
  │  ├─ Badge (kind-specific: Read/Write/Execute/Generic)
  │  │  └─ pill: bg varies, border varies, text_xs, opacity=0.8
  │  └─ Tool title (text_sm, SEMIBOLD)
  ├─ Subject (if present): pt(6), text_sm, opacity=0.82
  └─ Tool call ID: pt(2), text_xs, opacity=0.52
```

**Badge colors:**
- Read: subtle gray bg + border
- Write: accent gold bg + border
- Execute: orange (#F59E0B) bg + border
- Generic: neutral gray bg + border

### Permission Option Row (view.rs:468–542)

**Rendering:**

```
div (option row):
  ├─ id: "perm-opt-{index}"
  ├─ mt(8), px(12), py(10)
  ├─ rounded(10)
  ├─ cursor_pointer
  ├─ Conditional styling (is_selected affects bg + border):
  │  ├─ Reject: red bg (0xEF4444) with selection highlight
  │  ├─ Persistent Allow: accent gold with selection highlight
  │  └─ Allow Once: muted bg with selection highlight
  ├─ hover: lighter bg
  ├─ on_click handler
  └─ Children:
     ├─ Option name: "{index+1} · {name}" (text_sm, SEMIBOLD)
     ├─ Caption: "Allow once" / "Cancel" / "Remember this choice" (text_xs, opacity=0.58)
     └─ Option kind: "{kind}" (text_xs, opacity=0.44)
```

**Issues:**
1. **Hardcoded captions** ("Allow once", "Remember this choice") — not dynamic from option.kind
2. **Small visual differentiation** — reject vs. allow only differ by color
3. **Three text layers per row** — crowded, hard to scan

### Keyboard Navigation (view.rs:93–164)

**Supported keys:**
- Tab / Shift+Tab — cycle options forward/backward
- Arrow Up / Arrow Down — cycle options
- J / K — vim-style cycle
- 1–9 — instant pick by digit
- Enter — confirm selection
- Escape — cancel

---

## 11. Mode Badge (view.rs:633–647)

**Rendering:**

```
div (mode badge):
  ├─ px(8), py(3)
  ├─ rounded(999) — pill
  ├─ bg=accent.selected<<8|0x14
  ├─ border_1, border_color=accent.selected<<8|0x30
  ├─ text_xs, opacity=0.78
  └─ "Mode: {mode_id}"
```

**Placement:** Right side of footer, only when `active_mode.is_some()`

**Issues:**
1. **Adds footer clutter** — three badges now (status, mode, and border)
2. **Not discoverable** — small pill, easy to miss
3. **Read-only** — no interaction possible

---

## 12. Markdown Rendering

**Method (view.rs:238):**

```rust
render_markdown_with_scope(
  &msg.body,
  colors,
  Some(&scope_id)  // unique ID per message
)
```

**Properties:**
- Full-width (`w_full()`)
- Scope ID prevents CSS class collisions (format!("acp-msg-{}", msg.id))
- Uses `PromptColors` from theme
- No custom language highlighting visible in this file

---

## 13. Scroll Behavior

**Message list container (view.rs:818–843):**

```
div (message-list):
  ├─ id="acp-message-list"
  ├─ flex_grow() — takes remaining space
  ├─ overflow_y_scroll() — scroll when needed
  ├─ min_h(px(0.)) — prevent flex overflow
  ├─ When empty: render_empty_state()
  └─ When not empty:
     ├─ p_2() — padding
     ├─ gap_2() — space between cards
     ├─ flex_col — vertical stack
     └─ children: message cards (each pb(4) for bottom space)
```

**Issues:**
1. **No auto-scroll** — view doesn't scroll to newest message on receive
2. **No scroll-to-bottom affordance** — user must scroll manually
3. **Cards are well-spaced** but footer is fixed at bottom

---

## 14. Overall Layout Structure

**Main view render (view.rs:790–902):**

```
div (full size):
  ├─ size_full(), flex, flex_col, relative
  ├─ track_focus, on_key_down handler
  ├─ Message list (flex_grow)
  ├─ Plan strip (when present)
  ├─ Commands strip (only idle + empty)
  ├─ Streaming hint (only streaming)
  ├─ Footer: input + status
  └─ Permission overlay (absolute, when present)
```

**Issues:**
1. **Multiple "strips"** (plan, commands, streaming) compete for space and attention
2. **Footer is always visible** — no hiding on full-screen editor mode
3. **No header bar** — no title, no close button (handled elsewhere)

---

## 15. Key Handling & Input

**Key handler (view.rs:708–780):**

**Tab key:** (view.rs:40–58)
- Cycles permission options if overlay is open
- Otherwise consumed to prevent re-opening ACP

**Key down handler:** (view.rs:708–780)
- Permission overlay intercept (view.rs:710–725)
  - Calls `handle_permission_key_down()` if overlay present
  - Blocks non-modifier keystrokes while modal is open
- Regular input handling:
  - Shift+Enter: insert newline
  - Enter (no shift): submit input
  - Backspace: delete last character
  - Delete: no-op
  - Regular characters: append to input

**Issues:**
1. **Manual character handling** — re-implements text input widget
2. **Shift+Enter has higher precedence than modifiers** — unusual flow
3. **No undo/redo** — not typical text input behavior
4. **No selection/copy** — read-only text in footer doesn't support editing affordances

---

## 16. Design Principles Compliance (vs. CLAUDE.md)

### Whisper Chrome (CLAUDE.md Design Principles)
- ❌ **Message cards have visible borders** (opacity > 0.2) — not whisper
- ❌ **Role labels are bold and prominent** — should be smaller or hidden
- ✅ **Background colors are subtle** (0x06–0x22 alpha) — mostly compliant

### Opacity Tiers (CLAUDE.md Opacity Tiers)
- **Ghost (0.03–0.06):** Not used explicitly; some backgrounds are close (0x06)
- **Hint (0.40–0.55):** Used for secondary labels (0.44–0.52 opacity)
- **Muted (0.60–0.75):** Overused; many UI elements at 0.58–0.82
- **Present (0.85–1.0):** Hardcoded role labels and primary text

### Three Keys (CLAUDE.md Three Keys)
- ❌ **Footer has 3+ elements** — status badge, mode badge, input, hint text
- ❌ **Not following "Run, Actions, AI" pattern** — permission overlay modal has its own footer

### Minimalism (CLAUDE.md Minimalism)
- ❌ **Role labels on every card** — unnecessary visual clutter
- ❌ **Visible borders on every card** — adds visual weight
- ❌ **Plan + Commands strips** — good idea, but styling is heavy
- ⚠️ **Status badge duplicates footer hint** — redundant

---

## 17. Hardcoded Values vs. Theme

**Hardcoded colors (bad practice):**
- Error red: `0xEF4444` (hardcoded in multiple places)
- Orange for Execute: `0xF59E0B` (hardcoded)
- All alpha overlays: hardcoded nibbles like `0x16`, `0xF2`, `0x22`, etc.

**Theme usage (good practice):**
- Role colors: derived from `theme.colors.accent.selected`, `theme.colors.text.primary`, etc.
- Modal overlay: uses `theme::modal_overlay_bg()`
- Text colors: use `rgb(theme.colors.accent.selected)`

---

## 18. Rough Spots & Unpolished Details

1. **Gold borders on every card** — heavily visible, not minimal
2. **"You", "Thinking", "Claude Code" labels** — large and bold, dominate cards
3. **Commands list appears inside tool messages** — should it? Not clearly indicated
4. **Input field is plain text, not interactive** — no cursor, no selection affordance
5. **Permission modal has three text layers per option** — cramped, hard to scan
6. **Streaming indicator duplicates status badge** — redundant signals
7. **Plan entries are bare numbered list** — no progress indication
8. **Empty state is centered, wastes space** — should be top-aligned
9. **Footer has 4+ visual elements** — status, mode, input, hint all compete
10. **No keyboard visual feedback** — which option is selected? Just color, no highlight glyph

---

## 19. File-Specific Line References

| Element | File:Line | Details |
|---------|-----------|---------|
| Role titles | view.rs:177–186 | Hardcoded "You", "Claude Code", etc. |
| Message cards | view.rs:188–240 | 10px rounded, 1px borders, role-specific colors |
| Empty state | view.rs:242–306 | Centered layout, 3 text blocks |
| Streaming hint | view.rs:329–349 | Small gold dot + text |
| Status badge | view.rs:351–383 | Pill-shaped, footer-right position |
| Footer hint | view.rs:308–327 | Conditional state text, opacity=0.45 |
| Commands strip | view.rs:649–675 | Only when idle+empty, light bg |
| Plan strip | view.rs:677–704 | Numbered list, gold accent border |
| Permission overlay | view.rs:544–631 | Modal card, 640px wide, centered |
| Permission header | view.rs:410–466 | Badge + tool title + subject |
| Option rows | view.rs:468–542 | Indexed, color-coded by type |
| Mode badge | view.rs:633–647 | Pill in footer, "Mode: {id}" |
| Key handling | view.rs:708–780 | Manual character accumulation |
| Tab cycling | view.rs:40–58 | Permission option navigation |

---

## 20. Comparison to Zed's Agent Panel

**Script Kit gaps vs. Zed:**

| Feature | Script Kit | Zed | Script Kit Status |
|---------|-----------|-----|-------------------|
| Card borders | Visible (gold) | Minimal/none | Too heavy |
| Role labels | Bold, large | Smaller/icon | Too prominent |
| Input widget | Plain text display | Full text editor | Not interactive |
| Toolbar | None | Multiple affordances | Missing |
| Scroll behavior | Manual | Auto-scroll to bottom | Missing |
| Thinking blocks | Visible cards | Collapsible sections | No collapse |
| Streaming feedback | Small dot + text | Animated indicator | Too subtle |
| Plan rendering | Bare list | Progress tracker | Not structured |
| Permission modal | 640px centered | In-context option cards | Good, but cramped |

---

## Conclusion

The Script Kit ACP chat view is **functionally complete** but **visually unpolished**:

✅ **Strengths:**
- Structured message history with proper roles
- Permission modal with keyboard navigation
- Staged context injection before first submit
- Plan and command strips for advanced users

❌ **Weaknesses:**
- Overly visible borders and role labels (not whisper chrome)
- Redundant status indicators (badge + hint + streaming dot)
- Plain text input, not interactive widget
- Missing affordances (auto-scroll, visual feedback, collapse/expand)
- Hardcoded colors and opacity values
- Cramped footer with too many elements
- Empty state wastes space with centered layout

**Key recommendation:** Strip visual decoration to match Zed's polished look. Remove or hide role labels, minimize card borders, make input interactive, and consolidate redundant status signals.
