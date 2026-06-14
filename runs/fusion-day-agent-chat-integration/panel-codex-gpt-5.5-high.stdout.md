## Role Findings

The best architecture is **Day Page-hosted Agent Chat**, modeled after Notes-hosted Agent Chat, not a detached chat and not the generic main `AppView::AgentChatView` path.

Keep `AppView::DayPage { entity }` as the owning surface and add an internal Day Page mode:

```rust
enum DayPageSurfaceMode {
    Editor,
    AgentChat,
}
```

Add a focused owner file such as `src/main_sections/day_page_agent_chat.rs` with:

- `DAY_PAGE_EMBEDDED_AI_AUTOMATION_ID = "day_page:ai"`
- `embedded_agent_chat: Option<Entity<AgentChatView>>`
- `day_page_agent_chat_generation`
- `open_or_focus_day_page_agent_chat(...)`
- `switch_to_day_page_editor(...)`
- `insert_agent_chat_response_into_today(...)`

This keeps Today’s document session, autosave, focus restoration, footer syncing, and automation identity owned by Day Page.

Do **not** re-enable `FooterAction::Ai` for Day Page. Entry should be through the existing contextual Actions system:

- `day_page:ask_agent_about_today`
- optionally later: `day_page:ask_agent_about_selection`
- optionally later: `day_page:implement_today_selection`

Agent Chat actions need a new host mode, not Notes reuse:

```rust
AgentChatActionsDialogHost::DayPage
```

That host should include normal chat actions, but replace note export semantics with Today semantics:

- include `agent_chat_copy_last_response`
- include `agent_chat_export_markdown`
- include `agent_chat_insert_response_in_today`
- likely exclude `agent_chat_save_as_note`

## Evidence And Assumptions

Evidence from current code:

- Day Page is `AppView::DayPage` and owns the editor/session in [day_page_view.rs](/Users/johnlindquist/dev/script-kit-gpui/src/main_sections/day_page_view.rs:1).
- Day Page already saves through `DayPageDocumentSession`, wakes the indexer, and syncs footer from the editor path, so returned AI content should go through `notes_editor` plus `session.apply_editor_content`, not direct file writes.
- The existing `@context` handoff saves before leaving and restores the same `Entity<DayPageView>` in [day_page_round_trip.rs](/Users/johnlindquist/dev/script-kit-gpui/src/main_sections/day_page_round_trip.rs:1).
- Day Page Actions already use `day_page:*` ids and route through `execute_day_page_action` in [day_page_actions.rs](/Users/johnlindquist/dev/script-kit-gpui/src/main_sections/day_page_actions.rs:1).
- Generic Day Page AI footer events are currently ignored in [ui_window.rs](/Users/johnlindquist/dev/script-kit-gpui/src/app_impl/ui_window.rs:296), and source audits explicitly lock that down.
- Notes proves the embedded-host pattern with `spawn_hosted_view`, host callbacks, child automation id `notes:ai`, `prepare_for_host_hide`, and host-specific action dispatch in [agent_chat_host.rs](/Users/johnlindquist/dev/script-kit-gpui/src/notes/window/agent_chat_host.rs:1).
- Agent Chat already exposes host hooks and context staging APIs in [hosted.rs](/Users/johnlindquist/dev/script-kit-gpui/src/ai/agent_chat/ui/hosted.rs:1) and [view.rs](/Users/johnlindquist/dev/script-kit-gpui/src/ai/agent_chat/ui/view.rs:1120).

Prompt seed shape:

```text
@today

I’m looking at Today’s brain for YYYY-MM-DD.
Use Today as the primary context. Help me reason about it, identify useful next steps, and format anything worth keeping so it can be inserted back into Today.
```

Implementation-wise, prefer staging Today as an `AiContextPart` for `DocSource::DayPage` / today’s `sourceId`, then call `stage_inline_context_parts_from_host` or `submit_reused_entry_intent_with_host_context`. Avoid dumping raw markdown into the composer unless no structured context part exists.

## Failure Modes

Autosave race: if Day Page switches to Agent Chat while dirty content is only in the editor, Agent Chat can reason over stale disk/index state. Fix: force `view.save(cx)` before spawning or staging chat context.

`@context` collision: `day_page_context_return` already owns a temporary main-menu round trip. Opening Agent Chat while that is pending can strand the restore token. Fix: block `day_page:ask_agent_about_today` while pending, or cancel/restore first.

Wrong return path: using the generic main `AgentChatView` would make close/escape return to launcher semantics instead of Today semantics. Day Page must own `on_close_requested`.

Wrong persistence path: `agent_chat_save_as_note` is the wrong affordance for this feature. Day needs “insert/append into Today,” through the editor/session path, with autosave scheduled.

Automation identity drift: do not reuse `ai` or `notes:ai`. Register `day_page:ai` as a child of `main`, expose a state receipt like `embeddedAgentChat.host = "day_page"`, and update DevTools target matching intentionally.

Footer regression: do not add `FooterAction::Ai` to Day Page footer. Use Actions. Existing source audits are likely to fail if this path resurrects the deleted inline assistant behavior.

Source-audit policy: avoid minting a broad new source audit. Prefer behavior tests and pure unit tests. If an invariant must be locked, extend the existing Day Page no-inline-agent audit narrowly.

## Recommendation

Build this in three narrow slices:

1. **Host shell**
   Add Day Page embedded Agent Chat state and rendering. Reuse `spawn_hosted_view`, wire callbacks, add `DayPageSurfaceMode::AgentChat`, register `day_page:ai`, and close back to editor with `prepare_for_host_hide`.

2. **Actions and context**
   Add `day_page:ask_agent_about_today` to `day_page_host_actions_section`. Execution saves Today, builds/stages the Today context part, switches mode, focuses chat, and syncs footer. Add `AgentChatActionsDialogHost::DayPage`.

3. **Bring-back action**
   Add `agent_chat_insert_response_in_today`. It reads `pastable_response_text(cx)`, switches back to editor, inserts or appends a markdown block, calls `session.apply_editor_content`, `schedule_autosave_flush`, `sync_footer`, and focuses the editor.

Suggested inserted block:

```md
## Agent Chat - HH:mm

<assistant response>
```

Verification strategy:

- Rust checks through `./scripts/agentic/agent-cargo.sh`, not bare `cargo`.
- Unit tests for prompt/context seed builder, host action filtering, and Day Page insertion behavior.
- Existing source audit adjusted only if required for the no-generic-footer/no-inline-assistant invariant.
- DevTools probe under `scripts/agentic/day-page-agent-chat-probe.ts`:
  - `bash scripts/agentic/ensure-pi-sidecar.sh`
  - build artifact with `SCRIPT_KIT_AGENT_ARTIFACT_NAME=day-page-agent-chat ./scripts/agentic/agent-cargo.sh build --bin script-kit-gpui`
  - launch `Driver` with `sandboxHome: true` and that artifact
  - open Day Page through the real user path
  - enter Today text and prove save/dirty convergence
  - open Actions, select `day_page:ask_agent_about_today`
  - assert `day_page:ai` automation child, parent `main`, host receipt `day_page`
  - assert Agent Chat has Today context staged
  - trigger/apply a controlled response path if a fixture primitive exists; otherwise classify that as the missing primitive and cover insertion with unit behavior tests

## Self Score

8/10. The host shape is clear and fits the existing code. The main uncertainty is the exact existing `AiContextPart` constructor for a Day Page doc and whether DevTools already has a fixture path for setting an assistant response without waiting on a real model turn.


