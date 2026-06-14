**Recommendation**

Use the existing main-window embedded `AppView::AgentChatView` return-origin lifecycle as the default architecture. It is better supported than creating `DayPageSurfaceMode::AgentChat`, `AppView::DayPageAgentChat`, or a required `day_page:ai` automation child. That reuse is promising, not proven complete: validate Day Page focus restoration, footer state, Day-specific action routing, and devtools receipts during implementation.

Do not implement this by flipping the stale `FooterAction::Ai` ignore branch. Keep Day Page’s footer contract as Save/Actions. Open from the Day Page Actions section.

**User Flow**

1. User opens Today / Day Page.
2. In Day Page Actions, selects `day_page:ask_agent_about_today`.
3. App blocks or defers if `day_page_context_return.is_some()`.
4. App saves or flushes the live Day Page editor/session.
5. App captures the bound Day Page target: date, path, and `DayPageDocumentSession` identity.
6. App opens embedded Agent Chat through the existing preserving-return opener.
7. App stages Today’s markdown as structured context.
8. User discusses, asks questions, or has the agent implement work.
9. Agent Chat Actions expose `agent_chat_insert_response_in_today`.
10. That action inserts useful content back through the captured Day Page session/editor path, then returns to Day Page with editor focus and Save/Actions footer restored.

**Architecture**

Add a narrow Day Page launch path, not a new visual chat system.

Primary owners:

- `src/main_sections/day_page_actions.rs`: add `day_page:ask_agent_about_today`.
- `src/main_sections/day_page_view.rs` or new `src/main_sections/day_page_agent_chat.rs`: save/flush, capture binding, build context, launch Agent Chat.
- `src/ai/agent_chat/**` action planning layer: add a Day Page host/action mode or equivalent return-origin-derived mode.
- Agent Chat action dispatcher: add `agent_chat_insert_response_in_today`.
- Day Page session/editor code: provide a safe insert/append helper that preserves dirty state, autosave, footer sync, scroll, and focus.

State model:

```rust
struct DayPageAgentChatLaunch {
    bound_date: NaiveDate,
    bound_path: PathBuf,
    day_page: Entity<DayPageView>,
    session_id_or_generation: DayPageDocumentSessionId,
}
```

Store this as launch metadata on the Agent Chat session/host context, or derive it from the captured `AppView::DayPage { entity }` in `tab_ai_harness_return_view`. Do not use wall-clock `today` during bring-back. Past-day viewing and midnight rollover make `append_to_day(now)` unsafe.

**Context Staging**

Use structured context, not a raw markdown paste into the composer.

Prompt seed:

```text
I'm looking at Today's brain for YYYY-MM-DD.

Use the attached Today context as the source of truth. Help me reason about it, identify useful next steps, and format anything worth keeping so it can be inserted back into Today.
```

Context part:

```text
label: Today - YYYY-MM-DD
source: scriptkit://day-page/YYYY-MM-DD
mime: text/markdown
text: <saved Day Page markdown>
```

If existing hosted launch APIs cannot accept initial context because `spawn_hosted_view` starts with empty `initial_context_parts`, use one of two explicit paths:

- Preferred narrow path: open Agent Chat, then call `stage_inline_context_parts_from_host`.
- Alternative: extend the launch requirements with initial context parts.

Do not assume hosted spawn already stages Today context without checking.

**Agent Chat Actions**

Add a Day Page host/action mode such as:

```rust
AgentChatActionsDialogHost::DayPage
```

or an equivalent context marker derived from the return origin.

For Day-launched chat:

- Include `agent_chat_insert_response_in_today`.
- Include generic actions like copy/export if already shared.
- Do not make `agent_chat_save_as_note` the primary persistence affordance.
- Prefer hiding or de-emphasizing Save as Note in this host to avoid the wrong mental model.

Bring-back should use the Day Page entity/session/editor path. A safe helper should:

1. Read selected assistant response or generated summary.
2. Reject empty content.
3. Switch/ensure the captured Day Page entity is live.
4. Insert at cursor or append a block, depending on the chosen product behavior.
5. Apply content through `DayPageDocumentSession` / editor state.
6. Save or schedule autosave flush.
7. Sync footer.
8. Scroll inserted content into view.
9. Focus the Day Page editor.

Suggested inserted block:

```md
## Agent Chat - HH:mm

<content>
```

If provenance wants `DayEntry::Trace` or fragment storage, reconcile it back into the live Day Page session immediately. Do not rely on external append alone while an editor buffer is open.

**Pitfalls**

- `day_page_context_return` and Agent Chat return-origin state are separate. If `@context` is pending, block or defer `day_page:ask_agent_about_today`.
- Autosave must be flushed before context extraction. Reading disk/index without saving the live editor will stage stale context.
- Fragment Day Page bindings need a product decision: insert into parent day, insert into fragment, or disable Day Agent actions there. Do not silently guess.
- Automation identity should follow the actual implementation. If reusing main `AppView::AgentChatView`, do not assert `day_page:ai` is required. Add a Day-specific automation child only if devtools cannot distinguish host/return origin otherwise.
- Keep `FooterAction::Ai` ignored for `AppView::DayPage` unless the footer contract intentionally changes.

**Verification**

Use the smallest checks that can fail, and use `./scripts/agentic/agent-cargo.sh` for Cargo.

Tests:

- Unit test Day context builder: fixed date/path produces the expected seed and context source.
- Unit/behavior test action filtering: Day-launched Agent Chat shows `agent_chat_insert_response_in_today` and does not route through `agent_chat_save_as_note`.
- Behavior test guard: `day_page_context_return.is_some()` prevents or defers opening chat.
- Behavior test dirty buffer: Day Page saves before Agent Chat receives context.
- Behavior test bound date: open June 14, simulate current date June 15, bring back content, assert June 14 is targeted.
- `#[gpui::test]`: open Day Page, launch Agent Chat, close, assert `AppView::DayPage`, editor focus, and Save/Actions footer restored.

Avoid new source-audit tests unless no higher-rung invariant works. If existing source audits lock stale footer behavior, update them narrowly rather than adding another broad text audit.

**script-kit-devtools Probe**

Add one runtime probe under `scripts/agentic/`, using the repo’s devtools skill/primitives:

1. Launch app from an agent-built artifact.
2. Navigate to Day Page.
3. Type unique dirty text.
4. Trigger `day_page:ask_agent_about_today` through Actions.
5. Assert Agent Chat surface is active and records Day Page return origin/host metadata.
6. Assert staged context includes the unique dirty text.
7. Trigger a controlled `agent_chat_insert_response_in_today` path.
8. Assert Day Page is restored.
9. Assert inserted content is visible in the editor.
10. Assert footer is Save/Actions and editor focus is active.
11. Assert the target file is the captured bound day, not wall-clock today.

That gives the narrowest implementation with the fewest new surfaces while still covering the real risks: stale saves, return-state collision, wrong-day writes, action confusion, and runtime focus/footer regressions.
