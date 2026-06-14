I'll investigate the actual code facts before proposing anything. Let me examine the key files in parallel.
Now let me look at the hosted Agent Chat API surface, the footer AI gating, and the Agent Chat save actions.
Let me read the hosted view API and the current `FooterAction::Ai` handling in `ui_window.rs` for DayPage.
Let me examine the Agent Chat view's host-context API and the existing main-window Agent Chat view switching pattern.
Let me read the Agent Chat view's host-context APIs and the main-window preserving-return Agent Chat opener.
Let me check the return-origin seeding for DayPage and whether the main app already has an embedded Agent Chat field, plus the referenced requirements doc.
Let me confirm the return path restores DayPage and check the DayPageDocumentSession API for appending content.
## Role Findings

The architecture is already ~90% built. The main `ScriptListApp` already owns the full embedded-Agent-Chat stack that Notes reimplements: `embedded_agent_chat` (`src/main_sections/app_state.rs:1186`), `tab_ai_harness_return_view` (`app_state.rs:1177`), `AppView::AgentChatView`, `seed_agent_chat_return_origin_for_view` (`agent_handoff/mod.rs:3749`), and `open_tab_ai_agent_chat_with_entry_intent_preserving_return` (`agent_handoff/mod.rs:786`), which generically seeds **whatever `current_view` is** — including `AppView::DayPage` — as the return origin and restores it on close (`agent_handoff/mod.rs:3424-3443`).

The entire feature is gated by **one `else if` arm** at `src/app_impl/ui_window.rs:305-310` that explicitly ignores `FooterAction::Ai` while `AppView::DayPage`. Flipping that arm to route to the preserving-return opener is the whole "open" side. Notes's `agent_chat_host.rs` is the reference, but Day Page should **not** clone it — Day Page lives in the main window and should reuse the main-window embedded path, not mint a parallel host.

## Evidence And Assumptions

- **Evidence:** `ui_window.rs:293-320` — the `FooterAction::Ai` dispatcher already has a DayPage-specific ignore branch with `event = "main_window_footer_ai_ignored_day_page"`. The `day_page_context_return.is_some()` branch above it is the *other* return mechanism.
- **Evidence:** `agent_handoff/mod.rs:798-843` — `open_tab_ai_agent_chat_with_entry_intent_preserving_return_and_options` calls `seed_agent_chat_return_origin_for_view(&source_view)` with `source_view = self.current_view`, so seeding from `AppView::DayPage` works unchanged. Close path at `3382-3393` restores that view generically (with a self-guard only for `AgentChatView`→`AgentChatView`).
- **Evidence:** `day_page_round_trip.rs:52` — the `@context` round trip already does `self.save(cx)` *before* leaving the surface. The Agent Chat opener must do the same so the day buffer is authoritative on disk.
- **Evidence:** `agent_chat_host.rs:949-998` — `agent_chat_save_as_note` is the "bring back" precedent; it builds transcript markdown via `build_agent_chat_conversation_markdown_from_thread` and writes via `save_note_with_content_and_source`. The Day equivalent writes to today's day page.
- **Evidence:** `brain/substrate/mod.rs:72` `append_to_day` and `brain/store.rs:152` `append_activity` already exist as the append path for day content — reuse, do not invent.
- **Assumption:** `DayPageView`'s `Entity` is held alive across the Agent Chat surface because `tab_ai_harness_return_view` clones the `AppView::DayPage { entity }` (same way it holds any view). Verified by the generic restore path.
- **Assumption:** the main-window embedded Agent Chat already registers its own automation identity (it is the canonical `AppView::AgentChatView` surface). Day Page must **not** mint `day_page:ai` — it is not a separate window.

## Failure Modes

1. **Two return mechanisms collide.** `day_page_context_return` (the `@context` trip) and `tab_ai_harness_return_view` (Agent Chat) are independent slots. If a user mid-`@context`-trip somehow triggers the Agent footer, both restorations fire. Mitigation: the Agent Chat opener arm must `return` early when `self.day_page_context_return.is_some()` (keep the existing guard at `ui_window.rs:299-304` as the first branch, and route-to-Agent-Chat as the second). Symmetrically, `maybe_begin_day_page_context_round_trip_from_edit` should no-op while `current_view == AgentChatView`.
2. **Autosave race on leave.** If the opener switches to `AgentChatView` before flushing, unsaved day edits are in the editor entity only. Replicate `self.save(cx)` (or `schedule_autosave_flush` + await) in the DayPage→Agent handoff, exactly as `day_page_round_trip.rs:52` does.
3. **Automation-target identity collision.** Notes needed `notes:ai` because Notes is a distinct window with its own `NotesSurfaceMode`. Day Page is **in the main window**; minting `day_page:ai` would either collide with or shadow the main embedded-Agent-Chat automation child. Reuse the main-window identity — do not register a new `AutomationWindowInfo`.
4. **"Bring back" writes to wrong day.** If the user opened a past day (`day_page:open_past_day`), the Agent Chat "bring back" must target the *currently bound* day page, not "today." Use `DayPageView::session.path()` / bound date, not `Utc::now()`.
5. **Source-audit over-minting.** Per `AGENTS.md`, do NOT add a new `source_audit` test for the footer wiring. `src/app_impl/tests.rs` and `actions_button_visibility_tests.rs` already assert footer→Agent-Chat dispatch as decision locks; extend the *existing* assertions to cover the DayPage arm rather than adding a new audit file.
6. **Stale-context bleed.** Reused embedded Agent Chat view retains prior context parts. The opener should `clear_hosted_context_parts_from_host` then stage the day content, mirroring `relaunch_embedded_agent_chat` semantics (`agent_chat_host.rs:746-781`) — or use `suppress_focused_part` + explicit day-page context staging.

## Recommendation

**Smallest viable implementation — four edits, one new action, no new host module:**

1. **Footer route (the unlock):** In `src/app_impl/ui_window.rs:305-310`, replace the `else if matches!(self.current_view, AppView::DayPage { .. })` ignore arm with a call to a new `open_day_page_agent_chat(window, cx)` helper on `ScriptListApp`. Keep the `day_page_context_return.is_some()` guard above it as-is.

2. **Opener helper** (add to `src/main_sections/day_page_view.rs` or a new thin `day_page_agent_chat.rs`, ~40 lines):
   - Guard: `if self.day_page_context_return.is_some() { return; }`
   - `save(cx)` / flush autosave (reuse `DayPageView::save`).
   - Read today's content from the bound `DayPageDocumentSession` (`disk_content()`).
   - Call `self.open_tab_ai_agent_chat_with_entry_intent_preserving_return(Some(seed), cx)` where `seed` is the user's implicit question or empty; stage day content as an owned context part via the thread's `replace_pending_context_parts` (same API Notes's `clear_hosted_context_parts_from_host` uses at `view.rs:6908`).
   - Return origin is already seeded as `AppView::DayPage` by the preserving-return opener — nothing else to do for return.

3. **"Bring back into Today" action** — add `day_page:absorb_agent_chat` (mirrors `agent_chat_save_as_note`):
   - Action id: `day_page:absorb_agent_chat`; title "Absorb into Today"; section "Today" via `day_page_host_actions_section` so it appears in the shared ActionsDialog row list — **do not** build a Day-Page-local popup (AGENTS.md UI-consistency rule).
   - Implement in the main app's Agent-Chat-action dispatcher (the main-window analog of `dispatch_notes_agent_chat_action`). Build the transcript markdown with the existing `build_agent_chat_conversation_markdown_from_thread`, then append to the **currently bound** day page via `BrainSubstrate::append_to_day` / a small `DayPageView::append_block` helper that calls `apply_editor_content` + `schedule_autosave_flush` + `sync_footer`.
   - After absorb: close Agent Chat back to DayPage (reuse `close_tab_ai_harness_terminal` / `exit_embedded_agent_chat_surface`).

4. **State model:** zero new fields on `ScriptListApp`. The return slot (`tab_ai_harness_return_view`) and the embedded entity (`embedded_agent_chat`) already exist. The only new state is "which day page to absorb into" — and that's already encoded in the `AppView::DayPage { entity }` held by `tab_ai_harness_return_view`. Read the bound date off that entity at absorb time.

**User flow:** DayPage → footer Agent (or `cmd+enter`) → flush → Agent Chat opens with today's day content staged as context, return origin = DayPage → user asks/discusses → `cmd+k` → "Absorb into Today" → transcript/summary appended to the bound day page → returns to DayPage with editor focused. Escape from Agent Chat returns to DayPage unchanged via the existing close path.

**Prompt seed format:** Do not over-engineer. Seed composer text empty and stage a single owned context part `[Today: YYYY-MM-DD]` whose body is `disk_content()`. This matches how explicit-target handoff works (`open_tab_ai_agent_chat_with_explicit_target`, `agent_handoff/mod.rs:915`) — reuse the same `AiContextPart` variant rather than inventing a "day page" part kind.

**Code owners:** `src/app_impl/ui_window.rs` (footer arm), `src/main_sections/day_page_view.rs` or new `day_page_agent_chat.rs` (opener + absorb helper), `src/main_sections/day_page_actions.rs` (action row), main-window Agent-Chat-action dispatcher (absorb handler, analog of `dispatch_notes_agent_chat_action`). No changes to `src/ai/agent_chat/**` — the hosted API is sufficient.

**Test strategy (cheapest first, per AGENTS.md ladder):**
- **Behavior unit test** on the absorb helper: given a `DayPageDocumentSession` bound to a fixed date and a transcript string, assert `apply_editor_content` result contains the absorbed block and the bound date is unchanged. This is pure logic, no GPUI.
- **Extend existing decision-lock** `src/app_impl/tests.rs` / `actions_button_visibility_tests.rs`: assert the DayPage footer arm now dispatches to `open_day_page_agent_chat` (not the ignore branch). One-line assertion change in an already-justified audit file — not a new audit.
- **No new source-audit test.** The two-return-mechanism guard is enforced by the `if self.day_page_context_return.is_some() { return }` early-return — a behavior test can cover it, or leave it to code review.

**script-kit-devtools verification strategy:** (load the `agy-script-kit-devtools` skill for the exact primitives) Drive the app: focus DayPage → emit footer Agent → assert `semantic_surface` flips to the Agent Chat surface and `return_origin` is `dayPage` → emit the absorb action → assert the day page file on disk (`brain/days/YYYY-MM-DD.md`) grew by the transcript block and the surface returned to `dayPage`. One probe script under `scripts/agentic/` covers the end-to-end round trip that a `#[gpui::test]` cannot reach cheaply.

## Self Score

**8.5/10.** The implementation is genuinely small (one footer arm + one opener helper + one action + one dispatcher arm) because the main window already owns the embedded-Agent-Chat stack and the generic return-origin machinery. The score is held back by two real risks that need careful ordering, not more code: (1) the `day_page_context_return` vs `tab_ai_harness_return_view` collision guard must be a hard precondition on both openers, and (2) the "absorb into the *bound* day, not today" invariant needs a behavior test to lock it. No new host module, no new automation identity, no new source audit, no new UI components — all consistent with the pragmatist mandate.
