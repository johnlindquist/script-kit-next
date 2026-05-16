# 003 Agent Chat / Context Composer / Attachment Portals

Agent Chat is the ACP-backed assistant surface. It can live embedded in the main window, detached in its own popup, or hosted by Notes, while the composer owns slash commands, mentions, context chips, pasted attachments, and portal-backed target selection.

## Executive Summary

Agent Chat turns launcher intent, selected files, actions, dictation, plugin skills, notes, pasted content, and portal selections into ACP composer text plus typed `AiContextPart` attachments. Entry paths must preserve the originating surface, choose embedded or detached reuse correctly, and stage context without mutating stale threads. Composer interactions must treat `/` slash commands, `@` mentions, pasted text/images, focused mention previews, and attachment portals as one state machine with exact accept/cancel semantics.

The primary risk is losing user draft/context across reuse, agent switch, portal open, portal cancel, or host close. The preferred proof path is `getAcpState`, `getAcpTestProbe`, exact PromptPopup ids, actionsDialog receipts, and source-contract tests.

## What Users Can Do

- Open Agent Chat from the main launcher with Cmd+Enter or Tab.
- Reuse an existing detached Agent Chat window instead of opening duplicate chat surfaces.
- Stage selected launcher rows, file-search selections, action payloads, notes, plugin skills, dictation transcripts, large pastes, and images.
- Type in the composer, open slash picker with `/`, open mention/context picker with `@`, and accept rows with Enter or Tab.
- Use attachment portals for File Search, Clipboard History, Dictation History, Browser History, Notes Browse, ACP History, Script Search, Scriptlet Search, and Skill Search.
- Reopen focused portal-backed inline mentions with Cmd+. or Cmd+Shift+O.
- Cancel portals and return to the original ACP composer state.
- Switch agents/models while preserving draft text, caret, pending context, typed aliases, pasted tokens, and portal state.
- Cancel streaming with Escape before closing the surface.
- Use setup cards, Agent Catalog, model selector, history popup, ACP actions, copy/export/save/retry/new-conversation flows.
- Target embedded or detached Agent Chat through automation.

## Core Concepts

| Concept | Meaning | Owner |
|---|---|---|
| `AcpChatView` | Live chat/composer surface for embedded, detached, and hosted ACP. | `src/ai/acp/view.rs` |
| Detached ACP | Separate Agent Chat popup sharing semantic surface `acpChat` and kind `acpDetached`. | `src/ai/acp/chat_window.rs` |
| Entry request | Typed handoff carrying origin, target thread, seed policy, staging, and return origin. | `src/app_impl/tab_ai_mode/acp_entry.rs` |
| Composer state | Text, caret, pending context, typed aliases, pasted tokens, inline-owned tokens, picker state. | `src/ai/acp/composer_state.rs` |
| Context part | Typed attachment such as `ResourceUri`, `FilePath`, `SkillFile`, `FocusedTarget`, `AmbientContext`, or `TextBlock`. | `src/ai/message_parts.rs` |
| PromptPopup | Attached picker for slash/mention/model/history rows. | `src/ai/acp/picker_popup.rs`, `src/ai/acp/popup_registry.rs` |
| Portal session | Staged attachment portal contract with kind, query seed, composer snapshot, and replacement target. | `src/ai/acp/portal_contract.rs`, `src/app_impl/attachment_portal.rs` |
| Setup state | Agent availability/auth/config blocker shown as inline setup card. | `src/ai/acp/catalog.rs`, `src/ai/acp/config.rs` |

## Entry Points

| Entry point | Source | Behavior |
|---|---|---|
| Main launcher Cmd+Enter | ScriptList | Opens/reuses ACP with launcher return origin and optional focused context. |
| Main launcher Tab with text | ScriptList | Sends/submits launcher text to existing detached or embedded ACP. |
| Main launcher Tab empty | ScriptList | Focuses existing detached or opens ACP without focused context. |
| Plugin skill selection | ScriptList or slash picker | Inserts `/{skill}` and stages one `SkillFile`; no auto-submit. |
| File Search Cmd+Enter | File Search | Stages selected file/query through shared context staging. |
| Actions Cmd+Enter | Actions dialog | Stages action payload as focused target and restores ACP composer focus. |
| Notes Cmd+Enter | Notes | Opens Notes-owned embedded ACP; not the main ACP cache. |
| Dictation target | Dictation overlay | Delivers transcript to composer or auto-submits harness prompt. |
| Large paste | Main menu or composer | Stages `TextBlock` or image file part with compact inline token. |
| Detached hotkey/reattach | ACP surface | Moves between embedded and detached without losing thread identity. |

## User Workflows

### Open From The Launcher

Cmd+Enter builds an `AcpEntryRequest` with source view, target preference, context staging, seed text policy, and return-origin behavior. If detached ACP is open, the existing detached thread receives focus or input; otherwise embedded ACP opens in the main panel. Close should restore the originating launcher surface and filter focus.

### Use Slash Or Mention Picker

Typing `/` or `@` in the composer opens `acp-mention-popup`. Slash rows include command/skill entries such as `/new-script`; mention rows include context/portal rows. Arrow keys move the selected row, Enter or Tab accepts, and Escape dismisses while suppressing the same trigger/query until text or cursor changes.

### Attach A File Or Context Through A Portal

Accepting a portal row stages an `AcpPendingPortalSession` without mutating composer text, captures the original text/caret/replacement target, then opens the host surface. Accept returns an `AiContextPart` and replaces the original token exactly if unchanged; cancel restores the composer snapshot and clears the portal session.

### Reopen A Focused Mention

When the caret is inside a portal-backed inline token, the focused preview tells the user how to reopen it. Cmd+. or Cmd+Shift+O opens the same portal contract. Preview-only tokens such as large pasted text or pasted images do not reopen a portal.

### Switch Agent Or Model

Agent switch captures a full draft snapshot, relaunches with the selected agent, revalidates staged skill context, suppresses fresh host staging, and restores composer text/caret/context. Model switch changes the model for the current agent/thread without replacing the draft.

### Stream, Cancel, And Close

When the agent streams, tool calls, thinks, captures context, or waits for permission, the footer dot stays active. Escape cancels streaming before the surface can close. Idle Escape closes embedded ACP to its return origin; Cmd+W or native close runs the close lifecycle and cleans embedded/detached automation entries.

### Use Detached Chat

The detached window reuses the live thread, has kind `acpDetached`, semantic surface `acpChat`, and its own runtime window handle. When already open, new ACP entries focus/reuse it. Return to Panel closes detached and reuses the cached embedded `AcpChatView` when possible.

## Interaction Matrix

| User intent | Entry point | UI state | Key/click | Code path | Result | Proof |
|---|---|---|---|---|---|---|
| Submit launcher text | Main launcher | ScriptList | Tab with text | Launcher Tab ACP route | Text submits or seeds ACP | `tab_ai_plain_tab_*` traces |
| Stage plugin skill | ScriptList | Skill selected | Enter | `open_acp_with_selected_skill` | Slash token + `SkillFile` | Skill thread affinity tests |
| Set composer input | Protocol/user | Composer | Type / setAcpInput | `set_input_in_window` | Text/caret/popup refresh | `getAcpState` |
| Open slash picker | Composer | Idle | `/` | ACP picker popup | `acp-mention-popup` rows | PromptPopup getElements |
| Open mention picker | Composer | Idle | `@` | ACP picker popup | Context/portal rows | PromptPopup getElements |
| Accept picker row | Picker | Row selected | Enter/Tab | `accept_mention_selection` | Token/part/portal staged | Picker tests |
| Dismiss picker | Picker | Open | Escape | Picker dismiss owner | Suppressed trigger/query | Popup lifecycle receipt |
| Accept portal | Portal host | Selection | Enter | `attach_portal_part` | Composer token replaced | Portal contract tests |
| Cancel portal | Portal host | Active | Escape | `cancel_pending_portal_session` | Draft restored | Portal cancel receipt |
| Reopen focused portal | Composer | Caret in token | Cmd+. | `open_focused_mention_portal` | Same portal opens | Focused preview receipt |
| Paste large text | Main/composer | Paste | Cmd+V | TextBlock staging | Compact text token | Context submission receipt |
| Paste image | Main/composer | Paste | Cmd+V | Temp PNG FilePath staging | Compact image token | Pasted image contract |
| Cancel stream | ACP | Streaming | Escape | `cancel_streaming_from_escape` | Stream cancels, surface stays | Footer idle/state |
| Close embedded | ACP idle | Embedded | Escape/Cmd+W | ACP close lifecycle | Return origin or hide | Embedded close tests |
| Open ACP actions | ACP | Focused | Cmd+K | ACP action route | Actions dialog | `actionsDialog` ACP host |
| Change agent | ACP actions | Dialog | Select action | Draft-preserving relaunch | Agent changes, draft restored | Agent switch tests |
| Open history | ACP footer/actions | ACP | Action | `acp-history-popup` | Local history picker | History portal tests |

## State Machine

| State | Enters from | Exits to | Guards |
|---|---|---|---|
| Embedded reuse | Existing cached view | Focus/submit/stage | Preserves thread and view identity. |
| Detached open | Detach/open window | Close, reattach, focus | Runtime and automation registry pair must clean on both close paths. |
| Streaming | Submit | Cancel/finish/permission/tool | Escape cancels before close. |
| Permission wait | Agent asks | Approve/deny/cancel | Footer dot remains active; portal open clears permission UI. |
| Setup required | Preflight blocker | Setup/catalog/config | No prompt sent through broken agent state. |
| Composer idle | ACP visible | Picker/submit/portal/close | Text/caret/pending chips persist. |
| Picker open | `/` / `@` / setAcpInput | Accept/dismiss | Same `acp-mention-popup` id for slash and mention. |
| Portal staged | Picker accept/reopen | Host open/refusal | Composer text unchanged while host opens. |
| Portal active | Host surface | Accept/cancel | Launcher ACP entry blocked to avoid nested portal drift. |
| Portal accepted | Host selection | Composer idle | Exact token replacement if original unchanged. |
| Portal cancelled | Escape/cancel/back | Composer idle | Restores composer text/caret snapshot. |
| Agent switch relaunch | Change Agent | ACP ready/setup | Snapshot restores draft, context, aliases, tokens, portal state. |

## Visual And Focus States


## Keystrokes And Commands

| Key | Context | Behavior |
|---|---|---|
| Cmd+Enter | Launcher/File Search/Actions | Open/reuse ACP through typed entry request. |
| Tab | Launcher text | Auto-submit or seed launcher text into ACP when not blocked. |
| Tab | Picker open | Accept focused picker row. |
| Tab | Attachment portal active | Suppressed; portal remains active. |
| `/` | Composer | Open slash picker. |
| `@` | Composer | Open mention/context picker. |
| Enter | Picker open | Accept focused row. |
| Escape | Picker open | Dismiss and suppress trigger/query until edit. |
| Escape | Streaming ACP | Cancel stream; do not close surface. |
| Escape | Embedded ACP idle | Close to return origin. |
| Escape | Attachment portal | Cancel portal and restore composer snapshot. |
| Cmd+K | ACP focused | Toggle ACP actions. |
| Cmd+W | Embedded ACP | Close lifecycle, hide/reset as appropriate. |
| Cmd+W | Detached ACP | Close detached popup and clean registries. |
| Cmd+. | Caret in portal-backed token | Reopen focused mention portal. |
| Cmd+Shift+O | Caret in portal-backed token | Same focused portal reopen. |
| Plain Up | Empty composer idle/error | Recall last user-authored message. |
| Cmd+0 | ACP focused | Reset Agent Chat zoom/font sizes. |

## Actions And Menus

| Action family | Behavior |
|---|---|
| Change Agent | Captures draft snapshot, relaunches selected agent, restores text/context. |
| Change Model | Uses active agent-advertised models when available; preserves thread/draft. |
| ACP History | Opens local history popup or portal-backed history selection. |
| New Conversation | Clears transcript/collapsed state and starts empty thread. |
| Retry Last | Resubmits latest user turn. |
| Copy/export/save | Copies last response, exports markdown, or saves conversation/note. |
| Close | Embedded returns to origin; detached closes popup. |
| Return to Panel | Detached closes and reuses cached embedded view if possible. |
| `/new-script` Run | Footer Run appears only after validated `SCRIPT_READY path=... validated=true`. |

## Portal Matrix

| Portal | Trigger examples | Host | Accept result | Important guard |
|---|---|---|---|---|
| File Search | `@file`, file token, file row | File Search | `FilePath` part | Portal mode Enter attaches, not OS-open. |
| Browser History | `@browser-history` | Browser history browser | Focused visit target | No page content in row receipts. |

## Automation And Protocol Surface

| Receipt | What it proves |
|---|---|
| `getAcpState` target main | Reads embedded main ACP state. |
| `getAcpState` detached | Reads detached ACP entity state. |
| `getAcpTestProbe` | Internal ACP test/probe state such as selected picker row. |
| PromptPopup getElements | Slash/mention/model/history row-aware popup elements. |
| `actionsDialog` | ACP action route, selected action id, visible actions. |
| Portal logs/receipts | Attachment portal opened, accepted, cancelled, refused, and return snapshot restored. |
| Context resolution receipt | Pending context parts resolved, failed, or converted to metadata-only fallback. |

## Data, Storage, And Privacy Boundaries

- Pending context parts are resolved at submit time, not when first typed.
- Resource URIs resolve to `<context>` blocks; files resolve to `<attachment>` blocks.
- Unreadable files fall back to metadata-only attachments and record failures instead of aborting submission.
- Large pasted text uses `TextBlock` and compact preview-only inline tokens.
- Pasted images become temporary PNG `FilePath` parts and preview-only image tokens.
- Prompt popup and actions receipts should expose row/action metadata, not raw local content.
- Detached and embedded automation share semantic surface `acpChat`; `kind` distinguishes targeting.
- Setup blockers prevent prompts from being sent through unavailable or misconfigured agents.

## Error, Empty, Loading, And Disabled States

- Launcher ACP entry is blocked while an attachment portal is active.
- Unsupported detached/Notes portal kinds refuse cleanly and must not leave staged sessions.
- Failed detached/Notes history popup open cancels the staged session.
- Setup required renders an inline setup card rather than silently falling back to another agent.
- Model selector uses agent-advertised models when available and bootstrap defaults only before session models exist.
- Empty dictation transcript aborts quietly for target-aware flows.
- Late/stale pending portal accepts dismiss instead of mutating changed input.
- Config reload during streaming must not interrupt the running ACP subprocess.
- Close/hide paths must remove embedded AI, actions dialog, confirm popup, detached runtime, and automation registry entries as appropriate.

## Code Ownership

| Behavior | Owner files/tests |
|---|---|
| ACP entry requests and launcher handoff | `src/app_impl/tab_ai_mode/acp_entry.rs`, `src/app_impl/tab_ai_mode/mod.rs` |
| ACP launch/reuse/return origin | `src/app_impl/tab_ai_mode/acp_launch.rs`, `src/app_impl/tab_ai_mode/acp_context_staging.rs` |
| AcpChatView and composer | `src/ai/acp/view.rs`, `src/ai/acp/composer_state.rs` |
| ACP thread/session | `src/ai/acp/thread.rs` |
| Detached window | `src/ai/acp/chat_window.rs` |
| Agent catalog/config/model setup | `src/ai/acp/catalog.rs`, `src/ai/acp/config.rs`, `src/ai/acp/model_selector_popup.rs` |
| Picker/popup registry | `src/ai/acp/picker_popup.rs`, `src/ai/acp/popup_registry.rs`, `src/ai/acp/history_popup.rs` |
| Context parts | `src/ai/acp/context.rs`, `src/ai/context_mentions/mod.rs`, `src/ai/message_parts.rs` |
| Attachment portals | `src/app_impl/attachment_portal.rs`, `src/ai/acp/portal_contract.rs` |
| Context snapshots/resources | `src/context_snapshot/`, `removed-docs` |
| Key tests | `tests/acp_cmd_enter_entry_request_contract.rs`, `tests/acp_portal_contract.rs`, `tests/acp_portal_host_refusal_contract.rs`, `tests/acp_agent_switch_draft_contract.rs`, `tests/acp_popup_automation_parity_contract.rs`, `tests/acp_mention_popup_registry_lifecycle_contract.rs`, `tests/context_part_composer_state.rs`, `tests/context_part_resolution.rs`, `tests/context_part_submission_flow.rs`, `tests/context_picker.rs` |

## Invariants And Regression Risks

- Entry paths must go through typed entry requests so origin, target, staging, and return policy stay together.
- Detached reuse must focus the existing thread, not create a second detached window.
- Plugin skill handoff stages context on the thread that will receive focus.
- Large pastes bypass ScriptList filtering and become ACP context.
- Portal staging must happen before host open and only when the host supports the portal.
- Portal accept must replace the exact original token when unchanged; otherwise insert at fallback cursor.
- Portal cancel must restore composer text/caret snapshot and clear terminal state to idle.
- Escape must cancel streaming before embedded/detached close behavior.
- Agent switch must preserve draft/context/aliases/tokens and suppress fresh focused host staging.
- Detached close paths must use take-from-mutex cleanup and drain runtime/automation registries exactly once.
- Main semantic surface must re-key when detaching, hiding, reattaching, or triggering built-ins.

## Verification Recipes


```bash
cargo test --test acp_cmd_enter_entry_request_contract
cargo test --test acp_main_menu_skill_launch_contract
cargo test --test acp_plugin_skill_thread_affinity_contract
cargo test --test acp_portal_contract
cargo test --test acp_portal_host_refusal_contract
cargo test --test acp_agent_switch_draft_contract
cargo test --test acp_popup_automation_parity_contract
cargo test --test acp_mention_popup_registry_lifecycle_contract
cargo test --test context_part_composer_state
cargo test --test context_part_resolution
cargo test --test context_part_submission_flow
cargo test --test context_picker
cargo check --lib
cargo fmt --check
git diff --check
source checks
```


```bash
bun scripts/agentic/index.ts acp-setup-recovery --select-agent codex-acp --json
bun scripts/agentic/notes-acp-draft-agent-switch-replay.ts
bun scripts/agentic/notes-acp-actions-originating-view.ts
```


- Cmd+Enter from ScriptList opens/reuses ACP with correct return origin.
- `setAcpInput "/"` opens `acp-mention-popup` with row-aware getElements.
- `@file` portal staging preserves composer text until host accept/cancel.
- Portal cancel restores composer text/caret and leaves session idle.
- Agent switch preserves draft text and pending context.
- Escape while streaming cancels stream and keeps composer visible.
- Detached close removes runtime and automation registry entries.

Screenshots are only needed for visual acceptance of popup placement, setup card layout, footer activity dot, or chip rendering.

## Agent Notes

- Do not treat ACP as one window. Embedded main, detached popup, and Notes-hosted ACP have different host callbacks and portal capabilities.
- Do not mutate ACP thread context directly from File Search or actions; route through shared staging.
- To verify composer/picker behavior, use `getAcpState`, `getAcpTestProbe`, and exact PromptPopup ids before screenshots.
- If a portal accept replaces the wrong text, inspect the replacement target and original-token equality guard.
- If draft text disappears after agent switch, inspect draft snapshot fields for typed aliases, pasted tokens, pending context, and portal state.
- This belongs to `acp-chat-core` for lifecycle/runtime and `acp-context-composer` for composer/context/portal behavior.
- Screenshots are only needed when visual rendering is the asserted behavior.

## Related Features

- [001 Main Menu](./001-main-menu.md)
- [002 File Search](./002-file-search.md)
- [006 Notes Window](./006-notes-window.md)
- [004 MCP / SDK / Protocol](../raw-oracle/004-mcp-sdk-protocol/answer.md)

## Raw Oracle References

- [Prompt](../raw-oracle/003-agent-chat-context/prompt.md)
- [Bundle map](../raw-oracle/003-agent-chat-context/bundle-map.md)
- [Full answer](../raw-oracle/003-agent-chat-context/answer.md)
- [Full output log](../raw-oracle/003-agent-chat-context/output.log)
- [Session metadata](../raw-oracle/003-agent-chat-context/session.json)

## Open Questions And Gaps

- Exact Cmd+Shift+Enter ACP behavior needs focused source verification.
- Run button rendering/action after `SCRIPT_READY` needs deeper new-script/session UI mapping for pixel-level coverage.
- Agent Catalog row states/actions need a deeper catalog/setup-card render pass.
- Complete `getAcpState` and `getAcpTestProbe` JSON schemas need protocol-source expansion.
- Picker arrow key handlers and model/history popup navigation should be verified against popup source.
- Attachment portal accept keystrokes per host surface should be mapped in each filterable-surface chapter.
- ACP permission option rows and selectors need a dedicated permission/tool-call pass.
