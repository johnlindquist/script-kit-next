# Gap Analysis: Zed ACP vs Script Kit Chat UI/UX

**Analysis Date:** 2026-04-01  
**Comparison Scope:** Zed's ACP agent panel vs Script Kit's ChatPrompt  
**Assessment:** Zed is optimized for agent workflows; Script Kit is optimized for simple chat + script generation

---

## Executive Summary

| Dimension | Zed ACP | Script Kit Chat | Gap |
|-----------|---------|-----------------|-----|
| **Message Bundling** | Separate messages | Conversation turns | Zed more granular |
| **Thinking Blocks** | Collapsible with gradient fade | Plain streaming text | 🔴 MISSING in Script Kit |
| **Message Queue** | Inline editing + Send Now | No queue | 🔴 MISSING in Script Kit |
| **Tool Calls** | Full card UI (edit/execute) | Not in scope | 🔴 OUT OF SCOPE |
| **Permissions** | Pattern-based patterns | Not in scope | 🔴 OUT OF SCOPE |
| **Input Placement** | Bottom (standard) | Top (inverted) | Different UX philosophy |
| **Streaming Feedback** | Elapsed time + tokens | Thinking... placeholder | 🟡 LIMITED in Script Kit |
| **Mode/Model Selection** | In-toolbar selectors | Actions menu callback | Different architecture |
| **Activity Bar** | Consolidated edits/queue/plans | Not present | 🔴 MISSING in Script Kit |
| **Scroll Controls** | Scroll-to-recent-prompt buttons | Auto-scroll logic only | 🟡 LIMITED in Script Kit |

---

## Detailed Gaps

### 1. Thinking Blocks (MISSING in Script Kit)

**Zed Implementation:**
- Collapsible disclosure with `ThinkingBlockDisplay` enum (Automatic/AlwaysExpanded/AlwaysCollapsed)
- Left border indicator + icon
- Max-height constraint (256px) with gradient fade-out
- Auto-expand on user toggle or during stream
- Separate scroll handle for constrained content
- Location: `thread_view.rs:5234-5357`

**Script Kit Implementation:**
- All streaming content flows through single `render_markdown()` call
- No structural differentiation between reasoning and response
- Shows "Thinking..." placeholder while waiting
- No expandable/collapsible UI

**Impact:**
- 🔴 Users can't hide verbose reasoning blocks
- 🔴 No gradient fade-out for overflow content
- 🔴 Less visual clarity on what's reasoning vs. final answer

**Recommendation:**
Implement `AssistantMessageChunk` enum with `Message` and `Thought` variants, render `Thought` in collapsible container with max-height + gradient fade.

---

### 2. Message Queue with Inline Editing (MISSING in Script Kit)

**Zed Implementation:**
- Queue UI shows next-in-queue indicator (circle icon, accent color)
- Each queued message is a row with:
  - Icon (accent if next, muted if queued)
  - Message text (read-only)
  - On focus: "Edit" + "Send Now" buttons
  - On hover: "Delete" + "Edit" buttons
- Supports "Send Immediately" action
- Location: `thread_view.rs:3222-3365`

**Script Kit Implementation:**
- No queue of pending messages
- Single active input field
- Submit immediately or not at all

**Impact:**
- 🔴 Can't batch multiple requests
- 🔴 No "Send Now" for queued messages
- 🔴 No inline editing of pending requests
- 🔴 No visibility into what's queued

**Recommendation:**
Not required for basic chat. Consider for future when automating multi-turn sequences.

---

### 3. Tool Calls & Permissions (OUT OF SCOPE)

**Zed Implementation:**
- Sophisticated tool call card system
- Edit diffs (side-by-side, unified, ranges)
- Terminal execution output
- Permission UI (flat buttons or dropdowns)
- Pattern-based permission selection
- Location: `thread_view.rs:5898-6500+`

**Script Kit Implementation:**
- ChatPrompt is message I/O only
- Tool execution delegated to parent (via callbacks)
- No in-chat tool UI

**Impact:**
- 🔴 Can't show diffs/execution inline
- 🔴 Can't approve tool calls within chat
- 🔴 Less context about what's being changed

**Recommendation:**
Out of scope for ChatPrompt. Script Kit's architecture defers tool execution to parent prompts (Editor, Terminal, Div, etc.). Consider if adding agent-like approval flow to ChatPrompt is desired.

---

### 4. Streaming Feedback: Elapsed Time + Token Count (LIMITED in Script Kit)

**Zed Implementation:**
- Shows:
  - Spinner icon
  - "Awaiting Confirmation" label when blocked
  - Elapsed time (only after 30 seconds)
  - Token count (↓ generating, ↑ awaiting approval)
  - Directions arrows indicate flow
- Location: `thread_view.rs:5072-5152`

**Script Kit Implementation:**
- Shows "Thinking..." placeholder
- No elapsed time indicator
- No token count
- No directional arrows

**Impact:**
- 🟡 User doesn't see if generation is hanging (no elapsed time)
- 🟡 No token flow visibility
- 🟡 Less feedback during long operations

**Recommendation:**
Add optional `elapsed_time` and `token_count` to streaming state, render with spinner in separate indicator row.

---

### 5. Activity Bar (MISSING in Script Kit)

**Zed Implementation:**
- Consolidated section showing:
  - **Edits:** Files changed by agent (with diff stats)
  - **Plans:** Agent's task breakdown
  - **Message Queue:** Pending requests
  - **Subagent Approvals:** Awaiting permission from subagents
- Expandable sections with `expand/collapse` toggles
- Dividers between sections
- Summary headers with item counts
- Location: `thread_view.rs:2157-2252`

**Script Kit Implementation:**
- No activity bar
- Chat only shows message history

**Impact:**
- 🔴 Can't see pending edits/plans in chat context
- 🔴 No consolidated view of agent's side effects
- 🔴 User must switch to separate editor/file list to see changes

**Recommendation:**
Consider for advanced agent flows, but out of scope for basic chat. Could be rendered above chat area if needed.

---

### 6. Input Placement: Top vs. Bottom (DESIGN CHOICE)

**Zed Implementation:**
- Input at bottom (standard chat UX)
- Messages scroll upward (newest at bottom)
- User eyes naturally go to bottom for action

**Script Kit Implementation:**
- Input at top (inverted)
- Messages scroll downward (newest below input)
- Emphasizes clarity of response below action
- Unusual but intentional

**Impact:**
- 🟡 Different mental model (input→output vertically)
- 🟡 Muscle memory mismatch for users trained on Slack/ChatGPT
- ✅ Can be advantage (clearer cause-effect relationship)

**Recommendation:**
Keep as-is. This is a deliberate design choice. Could be made configurable if feedback warrants.

---

### 7. Mode & Model Selectors: Architecture Difference

**Zed Implementation:**
- **Mode Selector:** In-toolbar PopoverMenu
  - Current mode always visible
  - Dropdown shows all modes with current selected
  - CycleModeSelector action cycles through modes
- **Model Selector:** In-toolbar PopoverMenu
  - Shows current model + icon
  - Dropdown shows recent + all models
  - CycleFavoriteModels action cycles favorites
- Location: `mode_selector.rs`, `model_selector.rs`

**Script Kit Implementation:**
- **Mode & Model:** Handled by parent via actions menu (⌘K)
- Footer shows only current model as helper text
- ChatPrompt not responsible for selection
- Callback pattern: `on_show_actions: Option<ChatShowActionsCallback>`

**Impact:**
- 🟡 Model changes not instant (require dialog open)
- 🟡 Less discoverable (hidden behind ⌘K)
- ✅ Cleaner separation of concerns (parent handles config)

**Recommendation:**
If in-chat model selection is desired, add PopoverMenu-based selectors to footer (right side). Keep as optional feature.

---

### 8. Scroll Controls: Sticky Buttons vs. Auto-scroll (PARTIAL)

**Zed Implementation:**
- **Scroll-to-Recent-Prompt** button (↗ icon, bottom-right)
  - Jumps to most recent user message
- **Scroll-to-Top** button (↑ icon, bottom-right)
  - Jumps to beginning
- Semi-transparent, hover to full opacity
- Location: `thread_view.rs:4768-4932`

**Script Kit Implementation:**
- No sticky scroll buttons
- Auto-scroll when generation completes
- Manual scrolling handled by user
- ListState remembers scroll position

**Impact:**
- 🟡 Can't jump to recent prompt quickly
- ✅ Less UI clutter
- 🟡 Must scroll manually if user went up

**Recommendation:**
Optional feature. Add if user testing shows scroll jumping is frequent use case.

---

### 9. Copy & Selection (SIMILAR)

**Zed Implementation:**
- Right-click context menu
- "Copy This Agent Response" action
- Selections tracked in markdown entities
- Location: `thread_view.rs:5359-5500+`

**Script Kit Implementation:**
- Copy button on each turn (hover or always visible)
- Copies assistant response to clipboard
- No selection support (copy whole response only)

**Impact:**
- ✅ Both support copying
- 🟡 Zed supports partial copy via selection
- 🟡 Script Kit shows button (discoverable but less elegant)

**Recommendation:**
Keep as-is. Button is more discoverable for users unfamiliar with right-click.

---

### 10. Multi-Turn Architecture (SIMILAR)

**Zed Implementation:**
- Separate `UserMessage` and `AssistantMessage` entries in thread
- Can interleave multiple user→assistant sequences
- Each message is independent in history

**Script Kit Implementation:**
- `ConversationTurn` bundles user prompt + assistant response
- Cleaner visual grouping (user question above answer)
- Computed from flat message list

**Impact:**
- ✅ Both support multi-turn
- 🟡 Zed more flexible (can have assistant→assistant)
- ✅ Script Kit cleaner for typical user→assistant flow

**Recommendation:**
Keep Script Kit's turn-based model. Simpler and matches expected chat flow.

---

## Implementation Priorities

### High Impact / Feasible in Script Kit
1. **Thinking blocks** (collapsible with gradient)
   - Effort: Medium
   - Impact: High (users hide verbose reasoning)
   - Blocks: None

2. **Streaming feedback** (elapsed time + tokens)
   - Effort: Low
   - Impact: Medium (user confidence during long waits)
   - Blocks: None

3. **Scroll-to-recent buttons**
   - Effort: Low
   - Impact: Low-Medium (convenience)
   - Blocks: None

### Medium Impact / Higher Effort
4. **Message queue**
   - Effort: High (new state tracking, UI)
   - Impact: Medium (batch requests)
   - Blocks: Currently not needed

5. **In-toolbar model selector**
   - Effort: Medium
   - Impact: Medium (faster model switching)
   - Blocks: None (optional)

### Out of Scope for ChatPrompt
- Tool calls (belong in parent)
- Permissions UI (belong in parent)
- Activity bar (belongs in layout)
- Plan display (belongs in layout)

---

## Design Philosophy Differences

### Zed's Approach
- **Focus:** Agent as powerful collaborator
- **UX:** Comprehensiveness (show all context)
- **Integration:** Deep (tool approvals, edits inline)
- **Audience:** Advanced developers in IDE

### Script Kit's Approach
- **Focus:** Fast, scriptable automation
- **UX:** Simplicity (focused chat)
- **Integration:** Modular (parent orchestrates)
- **Audience:** Power users, automation enthusiasts

**Neither is wrong.** They optimize for different goals.

---

## Recommendations for Script Kit

### Quick Wins (1-2 days)
1. Add thinking block collapsible (copy Zed's pattern)
2. Show elapsed time during generation

### Medium Term (1-2 weeks)
3. Add scroll-to-recent-prompt button
4. Optional in-toolbar model selector

### Future Enhancements (2+ weeks)
5. Message queue (if batch automation needed)
6. Activity bar integration (if showing side effects needed)

### Non-Recommendations
- Don't add tool card UI (out of scope for ChatPrompt)
- Don't add permission dialogs (parent handles)
- Don't move input to bottom (intentional design choice)

---

## Conclusion

Script Kit's ChatPrompt is **intentionally simpler** than Zed's ACP agent panel. It's optimized for message I/O and script generation, not comprehensive agent coordination.

The key gaps are:
1. **Thinking blocks** — Should add (high value, medium effort)
2. **Streaming feedback** — Should add (high value, low effort)
3. **Message queue** — Nice-to-have (medium value, high effort)
4. **Scroll buttons** — Nice-to-have (low value, low effort)

Everything else (tool calls, permissions, activity bar) belongs in the parent application, not ChatPrompt itself.

