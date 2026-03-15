# Overnight Quality & AI Chat UX Sweep — Script Kit GPUI

You are running an autonomous quality improvement loop on the Script Kit GPUI
Rust codebase using GPT 5.4 Pro. Read CLAUDE.md and AGENTS.md first — they
are the law for all coding conventions.

## Verification Gate (run after EVERY logical change)
```bash
cargo check && cargo clippy --all-targets -- -D warnings && cargo test
```
If the gate fails, fix the issue before moving on. Never leave the repo broken.
Commit after each file or small logical group passes the gate.

## Priority Order

### P0: Fix the 3 Failing Tests
Green CI is prerequisite to everything. Fix these:
1. `notes::storage::tests::test_notes_au_trigger_has_when_guard_for_real_content_changes`
2. `notes::window::notes::..::test_on_search_change_saves_before_filtering_and_restores_search_focus`
3. `ocr::tests::test_async_extraction_calls_callback`
Read each test, understand the assertion, fix code or test as appropriate.

### P1: AI Chat Window — UX Quality-of-Life Improvements
This is the highest-value creative work. The AI chat window lives in
`src/ai/window/` (30+ files, ~10K lines). The inline ChatPrompt lives in
`src/prompts/chat/`. Study both before making changes.

Read ALL render_*.rs files in src/ai/window/ to understand the current layout,
then implement these improvements IN THIS ORDER:

#### 1a. Multi-line input (textarea behavior)
The composer input is currently single-line (COMPOSER_H = 40px). GPUI's
shape_line panics on newlines so the code sanitizes them to spaces.
- Make the input area grow vertically as the user types multiple lines
  (up to a max height, e.g. 200px), similar to ChatGPT/Claude's input
- Use Shift+Enter for newlines, Enter to submit
- The composer should auto-shrink when text is cleared after submission
- If GPUI's Input truly can't handle newlines, use the TextInput component
  from gpui-component or build a simple multiline wrapper
- Search the vendor/ directory for TextArea, Editor, or multiline input
  components that already exist

#### 1b. Better message differentiation
Currently both user and assistant messages use subtle muted backgrounds
(OPACITY_MESSAGE_USER_BACKGROUND vs OPACITY_MESSAGE_ASSISTANT_BACKGROUND).
- Give user messages a clearly different visual treatment — e.g. right-aligned
  with accent-tinted background (like iMessage/WhatsApp), while assistant
  messages stay left-aligned with the current subtle background
- Or: add a small colored left-border accent bar to differentiate roles
  (accent color for user, muted for assistant)
- Keep it tasteful — look at how Claude.ai, ChatGPT, and Raycast AI
  differentiate messages

#### 1c. Token/word count in input area
Show a live character or word count in the composer footer as the user types.
This helps users gauge message length before submitting.
- Show it subtly next to the model picker (left side of the bottom bar)
- Only show when input has content (>0 chars)
- Format: "~42 words" or "256 chars"

#### 1d. Keyboard shortcut discoverability
The AI window has great shortcuts (Cmd+N, Cmd+B, Cmd+K, Cmd+Shift+C, etc.)
but they're not discoverable unless you press Cmd+/.
- Add subtle shortcut hints to the sidebar toggle, search, and new chat buttons
  (already partially done via tooltips, but make them visible in the UI like
  the action strip buttons do)
- Make the Cmd+/ overlay more polished — organize shortcuts by category
  (Navigation, Chat, Input, Actions)

#### 1e. Improve the welcome screen
The welcome screen (render_welcome.rs) shows 4 generic suggestions. Make it
more useful:
- Add a "Recent chats" section showing the 3 most recent chats (if any exist
  in the database) with click-to-resume
- Make suggestion cards more specific to Script Kit: "Write a script to
  monitor clipboard", "Create a menu bar shortcut", etc.
- Add subtle keyboard hints (already shows ⌘1-4, which is good)

#### 1f. Streaming UX polish
- The "Thinking with [model]" state shows animated dots — add the model's
  provider icon or a small provider label to help users know which service
  is responding
- After streaming completes, the "Generated in Xs · ~N words · N words/s"
  label only shows for 8 seconds. Keep it visible until the next message
  is sent (it's useful context)
- Add a subtle progress indicator showing tokens streamed so far during
  active streaming (e.g. "~150 words so far" below the streaming content)

#### 1g. Chat sidebar improvements
- Add a right-click context menu on chat items with: Rename, Delete, Export
  (currently delete requires Cmd+Backspace which is hidden)
- Show a 1-line preview of the last message under each chat title in the
  sidebar (like Apple Messages)
- Make the sidebar width resizable by dragging the border (store preference)

### P2: Replace unwrap()/expect() in AI-related files
Work through these files first (they're the ones the AI chat touches):
1. src/ai/providers.rs (22 unwraps)
2. src/ai/storage.rs (36 unwraps)
3. src/ai/session.rs
4. src/ai/window/*.rs (all window files)
5. src/mcp_protocol/mod.rs (57 unwraps)
6. src/mcp_server/mod.rs (45 unwraps)
7. src/stdin_commands/mod.rs (39 unwraps)

Replace with `?` + `.context()` or graceful fallback. Never just suppress.

### P3: Add // SAFETY: comments to unsafe blocks (~126 missing)
Focus on the highest-traffic modules:
- src/window_control/
- src/platform/
- src/clipboard_history/
- src/keyboard_monitor/
- src/executor/

### P4: Migrate remaining `log::` to `tracing::` (23 uses in 12 files)
Use structured fields: `tracing::info!(path = %p.display(), "loading script")`

### P5: Replace `let _ =` with proper error handling (focus on AI files first)
Use `.log_err()` from `gpui::ResultExt` or explicit handling.

## Design Principles for AI Chat UX Changes
- Follow the existing design language: use theme colors from get_cached_theme(),
  opacity constants from theme/opacity.rs, spacing tokens (S1-S9), radius tokens
  (R_SM, R_MD, R_LG, R_XL)
- Never hardcode colors — always use cx.theme() or the theme system
- Vibrancy: never set opaque backgrounds on containers that should show through
- Test with both light and dark themes mentally (the theme system handles this
  if you use theme colors correctly)
- Study how the existing code handles hover states, keyboard/mouse mode
  switching, and focus — maintain those patterns
- When adding new UI, write at least one unit test for any pure logic
  (not render code, but things like formatting, state decisions, etc.)

## Rules
1. Read CLAUDE.md and AGENTS.md before starting
2. include!() files (src/app_impl/, src/render_prompts/, etc.) have NO
   top-level use statements, NO mod declarations
3. src/ai/window/*.rs are proper module files — normal use/mod rules apply
4. Run verification gate after EVERY change
5. Commit working changes frequently with descriptive messages
6. If a UX change is too risky or complex, implement a simpler version
   and leave a TODO comment explaining the fuller vision
7. For P1 items, if you can't implement the full vision, implement whatever
   subset you can that still improves the experience
8. git push after every 5-10 commits to save progress
9. Do NOT touch files outside the scope of these tasks

## Session Completion
When done or 10 hours elapsed:
1. Run full verification gate
2. Commit remaining work
3. `git pull --rebase && git push`
4. Write a summary: what was completed, what was skipped, what remains
