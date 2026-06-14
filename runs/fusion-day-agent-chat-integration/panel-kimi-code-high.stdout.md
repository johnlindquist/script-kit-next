• I’ll inspect the relevant surfaces and existing hosted Agent Chat precedent before giving the edge-case-tester assessment.## Role Findings

  As the edge-case-tester panel, my job is not to write the happy-path spec but to stress the seams where this integration can leak, corrupt data, or regress existing surfaces. The proposed feature—open Agent Chat from Today, discuss the day page, and bring useful results back into the day page—touches four already-complex subsystems:

  1. **Day Page session/autosave** (`src/main_sections/day_page_view.rs`, `src/day_page/document.rs`)
  2. **The existing `@context` round-trip state machine** (`src/main_sections/day_page_round_trip.rs`)
  3. **Hosted Agent Chat lifecycle** (Notes precedent in `src/notes/window/agent_chat_host.rs`, host-neutral bootstrap in `src/ai/agent_chat/ui/hosted.rs`)
  4. **Main-window footer/automation identity** (`src/app_impl/ui_window.rs`, `src/app_impl/automation_surface.rs`)

  The Notes precedent is the right narrow starting point, but Day Page is **not** Notes: it lives inside the *same* main window as ScriptList, it already has a pending-return state (`day_page_context_return`), and its footer currently suppresses `FooterAction::Ai` for `AppView::DayPage`. Any reuse of `spawn_hosted_view` must be wrapped in a new main-window view state that preserves the Day Page entity and can unwind cleanly.

  ---

  ## Evidence And Assumptions

  ### Evidence from current code

  - **Day Page owns a single `NotesEditor` + `DayPageDocumentSession`.** `DayPageView::new` creates the editor pair and `bind_today` loads disk content (`src/main_sections/day_page_view.rs:20-81`). `on_editor_change` schedules a 300 ms debounced autosave plus a trailing flush timer (`src/main_sections/day_page_view.rs:142-239`).
  - **Day Page already swaps away from itself.** The `@context` round trip stores `DayPageContextReturn { entity, line_range, segment_byte_range }` in `ScriptListApp::day_page_context_return`, resets to `AppView::ScriptList`, and later restores the same entity (`src/main_sections/day_page_round_trip.rs:21-256`).
  - **Footer `Agent` is deliberately ignored while on Day Page.** `dispatch_main_window_footer_action` ignores `FooterAction::Ai` and `FooterAction::AgentModel` when `AppView::DayPage` is active, explicitly to avoid stale Agent actions (`src/app_impl/ui_window.rs:293-320` and `:476-496`).
  - **Notes-hosted Agent Chat is a child automation window.** It registers `notes:ai` as a child of `notes`, sets `semantic_surface: "notesAgentChat"`, and restricts portals to `AgentChatHistory` only (`src/notes/window/agent_chat_host.rs:9-42`, `:331-336`).
  - **Day page append API is append-only.** `BrainSubstrate::append_to_day` writes a timestamped `DayEntry` line; `DayEntry::Trace` already exists for Agent Chat provenance (`src/brain/substrate/mod.rs:70-77`, `src/brain/substrate/day.rs:18-38`).
  - **Context parts round-trip through markdown links.** `day_page_markdown_reference_for_context_part` and `context_parts_from_day_page_markdown_links` convert between `@file:…`/`@notes:…` and persisted markdown (`src/day_page/sediment.rs:307-401`).

  ### Working assumptions

  - The feature stays **inside the main launcher window** (no floating Day Chat window).
  - “Bring back into Today” primarily means **appending** the useful result to today’s page, not replacing the open editor buffer.
  - The user may still be typing when Agent Chat is opened, so the Day Page buffer can be dirty.
  - Agent Chat’s normal tool use (file edits, shell commands) can still run while the chat is open.

  ---

  ## Failure Modes

  ### 1. Autosave / external-append race while Agent Chat mutates the day page
  Day Page refreshes from disk only when the buffer is **clean** (`DayPageDocumentSession::maybe_refresh_from_disk` returns `None` if dirty, `src/day_page/document.rs:249-271`). If Agent Chat appends a trace line to today’s file while the user is typing, the open editor will not see it. On next `bind_today` or save, the editor’s older content may overwrite the appended line, or the appended line may appear stale relative to the editor cursor.

  **Worst case:** The “bring back” action writes to disk, then the user’s pending autosave flushes and clobbers it.

  ### 2. Collision with the `@context` round-trip state
  `day_page_context_return` holds the live `Entity<DayPageView>`. If the user triggers Agent Chat while a context round trip is pending, switching `current_view` to a chat surface will drop/overwrite the pending return. Restoring the Day Page entity later is possible, but the `line_range`/`segment_byte_range` stored in `DayPageContextReturn` may now point into stale content.

  **Worst case:** The `@context` search never completes; the accepted token is spliced into wrong byte ranges or lost.

  ### 3. Fragment-view ambiguity
  Day Page can be viewing a fragment (`DayPageBinding::Fragment`). The “Today” actions and the return-to-Day behavior assume a day binding. If Agent Chat is opened from a fragment view, “bring back into today” must target the **parent day page**, not the fragment file.

  **Worst case:** A trace line is written into a fragment markdown file or the wrong date’s day page.

  ### 4. Automation identity collision
  The main window already has automation id `"main"` with semantic surface keyed by `AppView` (`src/app_impl/automation_surface.rs:43-48`). A hosted Agent Chat inside Day Page needs its own child identity. Reusing `"main"` or `notes:ai` would confuse `script-kit-devtools` / MCP queries.

  **Worst case:** External automation sends keystrokes to the wrong surface, or DevTools cannot tell whether Agent Chat is embedded in Notes vs. Day Page.

  ### 5. Day rollover during a long chat
  `DayPageDocumentSession` binds to a `NaiveDate` (`src/day_page/document.rs:36-41`). If the conversation spans midnight, `append_to_day` uses `Utc::now()` and the configured timezone. The chat was seeded from “today” but the result may belong to a different local date than the original binding.

  **Worst case:** The user thinks they are updating Today (June 14) but the append lands on June 15.

  ### 6. Empty / huge / malformed AI output
  - Empty assistant response: should not append a blank trace line.
  - Response above fragment threshold (~200 words): should it become a `DayEntry::FragmentRef` or a very long `Trace` line?
  - Response containing markdown links: `context_parts_from_day_page_markdown_links` may re-parse them on next Agent Chat open, potentially staging stale context.

  ### 7. Footer/action-dialog host filtering
  `AgentChatActionsDialogHost` currently has `Shared`, `Notes`, `Detached` (`src/actions/builders/script_context.rs:1502-1511`). `agent_chat_save_as_note` is wired to Notes. A Day Page host needs a **new** action (`agent_chat_save_to_today` / `day_page:save_chat`) and must **exclude** `agent_chat_save_as_note` to avoid creating a note when the user asked for Today.

  **Worst case:** The actions dialog shows both “Save as Note” and “Add to Today”; the wrong one creates a ghost note.

  ### 8. Escape / dismiss ladder inconsistency
  Day Page Escape currently: returns from fragment → returns from past day → closes window (`src/main_sections/day_page_view.rs:610-633`). Agent Chat Escape normally closes the chat window or, in Notes, returns to the editor. In Day Page, Escape from the embedded chat must return to the editor, but a second Escape must then follow the normal Day Page ladder.

  **Worst case:** One Escape closes the whole launcher while the user only wanted to leave chat.

  ### 9. Source-audit ratchet overreach
  Per `AGENTS.md`, source-audit tests are a last resort. It is tempting to add a test that asserts “Day Page never routes `agent_chat_save_as_note`” by searching source text. That would be brittle and violate the pruning rule.

  ### 10. Focus and accessibility surface mismatch
  `DayPageView` tracks its own `focus_handle` and focuses the editor on entry (`src/main_sections/day_page_view.rs:284-288`). When Agent Chat is shown, focus must move to the chat composer. When returning, focus must move back to the editor. If `current_view` is changed without calling `focus_editor`, the native footer and accessibility tree will disagree about the active surface.

  ---

  ## Recommendation

  ### Architecture

  Add a **new main-window view state** rather than trying to embed Agent Chat inside `AppView::DayPage`’s render:

  ```rust
  // in src/main_sections/app_view_state.rs
  AppView::DayPageAgentChat {
      day_page: Entity<DayPageView>,
      chat: Entity<crate::ai::agent_chat::ui::AgentChatView>,
  }
  ```

  Why a new variant instead of a surface-mode flag inside `DayPageView`?
  - It reuses the existing main-window routing machinery (`uses_shared_main_view_header`, footer dispatch, automation rekeying).
  - It avoids mutating `DayPageView` while it may hold a dirty editor buffer.
  - It lets `DismissPolicy` declare Escape as `LetViewHandle` for the chat surface and then `CloseMainWindow` once returned to Day Page.

  Add a matching `SurfaceKind::DayPageAgentChat` and `automation_semantic_surface: "dayPageAgentChat"` in `SurfaceKind::surface_contract` (`src/main_sections/app_view_state.rs:992+`).

  ### Code owners

  | Layer | Owner file | Responsibility |
  |-------|------------|----------------|
  | View state + routing | `src/main_sections/app_view_state.rs`, `src/app_impl/automation_surface.rs` | New `AppView`/`SurfaceKind`, dismiss policy, automation rekey |
  | Day Page host wrapper | New `src/main_sections/day_page_agent_chat_host.rs` | Spawn/reuse chat, wire callbacks, automation child window |
  | Today actions | `src/main_sections/day_page_actions.rs` | Add `day_page:ask_agent` row |
  | Agent Chat action filtering | `src/actions/builders/script_context.rs` | Add `AgentChatActionsDialogHost::DayPage` |
  | Bring-back persistence | `src/day_page/` or `src/brain/substrate/day.rs` | `DayEntry::Trace` / fragment append |
  | Footer dispatch | `src/app_impl/ui_window.rs` | Route `FooterAction::Ai` for the new view |

  ### State model

  1. **Opening**
     - From `AppView::DayPage { entity }`, save the day page buffer (`entity.update(|v| v.save(cx))`) and **flush** the autosave timer.
     - Capture the bound date from `DayPageView::session.bound_date()` so the chat result targets the correct day even if midnight passes.
     - Spawn chat via `crate::ai::agent_chat::ui::hosted::spawn_hosted_view(Some(seed), …, cx)`.
     - Switch to `AppView::DayPageAgentChat { day_page: entity, chat }`.
     - Register automation child `"daypage:ai"` with `parent_window_id: Some("main")`, `parent_kind: Main`, `semantic_surface: "dayPageAgentChat"`.

  2. **While open**
     - Restrict portals to `AgentChatHistory` only, mirroring Notes (`src/notes/window/agent_chat_host.rs:333-336`).
     - The Day Page entity remains alive but is not rendered; its dirty flag is false because we saved before switching.

  3. **Closing / return-to-Day**
     - `on_close_requested` for the chat calls `prepare_for_host_hide`, removes the automation child, and switches `current_view` back to `AppView::DayPage { entity }`, then calls `focus_editor`.
     - If a bring-back action is triggered, append the result first, **then** close.

  4. **Bring-back**
     - Primary action: `agent_chat_save_to_today` (or `day_page:save_chat`).
     - Implementation:
       - Build markdown from `build_agent_chat_conversation_markdown_from_thread` or last assistant message.
       - If length ≤ fragment threshold and user has no text selection in the day editor: append a `DayEntry::Trace { summary, provenance_link: scriptkit://agent-chat/{thread_id} }` to the **captured bound date** via `BrainSubstrate::append_to_day`.
       - If length > threshold: write a fragment via `BrainSubstrate::write_fragment` with source `scriptkit://agent-chat/{thread_id}`, then append `DayEntry::FragmentRef`.
       - Do **not** splice into the live editor buffer unless the user explicitly chooses “Insert at Cursor” and the buffer is clean.
     - After appending, refresh the day page entity from disk or bump its `last_mtime` so `poll_external_disk_changes` picks it up.

  ### Action ids

  - `day_page:ask_agent` — in the Today actions section (`src/main_sections/day_page_actions.rs`). Opens the chat with the current day page content seeded as context.
  - `agent_chat_save_to_today` — in the Agent Chat actions dialog when hosted in Day Page. Replaces `agent_chat_save_as_note` for this host.
  - `agent_chat_close` — reused; returns to Day Page instead of closing a window.
  - Keep existing `agent_chat_copy_last_response`, `agent_chat_export_markdown`, etc.

  Add `AgentChatActionsDialogHost::DayPage` to `agent_chat_host_action_plan` and **exclude** `agent_chat_save_as_note`, **include** `agent_chat_save_to_today`.

  ### Prompt seed format

  When opening from Day Page:

  ```rust
  let mut parts = context_parts_from_day_page_markdown_links(&day_content);
  parts.push(AiContextPart::TextBlock {
      label: "Today".into(),
      source: format!("scriptkit://day-page/{}", bound_date),
      text: day_content.clone(),
      mime_type: Some("text/markdown".into()),
  });
  // Pass parts into the hosted thread via initial_context_parts.
  ```

  Currently `spawn_hosted_view` hard-codes `initial_context_parts: Vec::new()` (`src/ai/agent_chat/ui/hosted.rs:45`). You will need to extend `AgentChatLaunchRequirements` or add a new `spawn_hosted_view_with_context` helper rather than mutating the existing Notes path.

  If text is selected in the day editor, seed `initial_input` with a quote of the selection plus a focused question; otherwise seed `initial_input: None` and let the user type.

  ### Return-to-Day behavior

  - Close chat → restore `AppView::DayPage { entity }` → `focus_editor` → `sync_main_footer_popup`.
  - If the user pressed Escape and the chat composer has an empty input, close immediately. If the composer has a draft, mirror Notes: close anyway but do not auto-submit (the thread is preserved in history).
  - After bring-back, land back on Day Page with the newly appended content visible (scroll to end).

  ### Test strategy

  1. **Unit tests**
     - `day_page_markdown_reference_for_context_part` round-trip for day-page links → Agent Chat parts.
     - `DayPageDocumentSession` external-append-while-dirty behavior is unchanged.

  2. **Behavior tests (`#[gpui::test]`)**
     - Open Day Page → open Agent Chat → close → assert `current_view` is `AppView::DayPage` and editor has focus.
     - Open Day Chat with a dirty day buffer → assert buffer was saved before view switch.
     - Trigger `agent_chat_save_to_today` → assert `BrainSubstrate::append_to_day` wrote a `Trace` line with `scriptkit://agent-chat/` provenance.
     - Day rollover: bind June 14, open chat, mock time to June 15, bring back → assert the trace lands on June 14 (the captured bound date).

  3. **No source-audit tests**
     - Per `AGENTS.md` ladder, use the type system (`AgentChatActionsDialogHost::DayPage`) and behavior tests, not `read_to_string` assertions.

  ### script-kit-devtools verification strategy

  Use the project skill `script-kit-devtools` (`/Users/johnlindquist/dev/script-kit-gpui/.claude/skills/script-kit-devtools/SKILL.md`):

  1. Drive the app to `AppView::DayPage`, then trigger `day_page:ask_agent`.
  2. Assert automation tree contains a child window with id `"daypage:ai"`, `parent_window_id: "main"`, `semantic_surface: "dayPageAgentChat"`.
  3. Send a simulated user message, wait for assistant response, then trigger `agent_chat_save_to_today`.
  4. Assert the underlying `brain/days/YYYY-MM-DD.md` gained a timestamped trace line linking to `scriptkit://agent-chat/{thread_id}`.
  5. Assert closing the chat returns `semantic_surface` to `"dayPage"` and focus is inside the day editor.

  ### Pitfalls to explicitly avoid

  - **Do not reuse `FooterAction::Ai` on `AppView::DayPage`.** The current ignore is intentional. Only the new `AppView::DayPageAgentChat` should advertise the Agent footer button.
  - **Do not let Agent Chat append directly to the live editor buffer.** Always go through `BrainSubstrate::append_to_day` or `write_fragment` to avoid fighting the autosave state machine.
  - **Do not lose the `@context` round trip.** Before switching to chat, cancel any pending `DayPageContextReturn` with a clear log event, or queue the chat open until the round trip completes.
  - **Do not add `agent_chat_save_as_note` to the Day Page host.** That action is hard-wired to Notes (`src/notes/window/agent_chat_host.rs:949-997`); add a new id instead.
  - **Do not mint a source-audit test** for action filtering. The `AgentChatActionsDialogHost` enum is the compile-time guard.

  ---

  ## Self Score

  **8/10** for this panel role.

  I covered the critical boundary cases (autosave races, `@context` collision, fragment-view ambiguity, automation identity, day rollover, action-dialog host filtering, source-audit policy). The recommendation is concrete enough to implement and is bounded by existing shared APIs.

  What keeps it from a 9 or 10:
  - I did not run the existing Day Page / Notes Agent Chat tests to confirm exact current pass/fail baseline.
  - I did not produce a working `#[gpui::test]` harness snippet, so the behavior-test assertions are described but not executable.
  - I assumed a new `AppView` variant is the least-risky path; a surface-mode flag inside `DayPageView` would have different failure modes that I only partially contrasted.

