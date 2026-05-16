# 042 Menu Syntax Power Commands and Capture Composer



## Executive Summary


The feature is deliberately target-gated. Unknown capture heads, top-level tags, URLs, localhost strings, legacy handoff triggers, and normal fuzzy queries stay launcher search unless a built-in or metadata-registered capture target owns them.

## What Users Can Do

- Type bare `;` or legacy `+` to open the capture target picker.
- Filter capture targets with `;<partial>` before locking a target.
- Use tags, priority, URLs, durations, dates, and key/value tokens inside a capture body.
- Press Enter to validate and run a matching capture handler.
- Use footer actions to create a local handler scaffold or request an AI scaffold for an unknown target.
- Inspect read-only capture hints and validation details from state receipts instead of screenshots.

## Core Concepts

| Concept | Meaning | Owner |
|---|---|---|
| Capture target | Built-in or metadata-registered slug that may own capture syntax. | `src/menu_syntax/payload.rs`, `src/menu_syntax/metadata.rs` |
| Capture invocation | Parsed target, body, tags, priority, URL, dates, duration, key/value, and raw input. | `src/menu_syntax/capture.rs`, `src/menu_syntax/payload.rs` |
| Trigger picker | Detached popup for capture targets, qualifiers, command rows, and setup footer actions. | `src/menu_syntax/trigger_picker.rs` |
| Capture composer | ScriptList-owned body mode where input becomes payload rather than fuzzy launcher search. | `src/app_impl/menu_syntax_main_hint.rs` |
| Capture schema | Built-in or dynamic required, optional, and forbidden field rules. | `src/menu_syntax/capture_schema.rs` |
| Capture handler | Script or scriptlet with `menuSyntax.family == "capture.v1"` and matching targets. | `src/menu_syntax/filter.rs`, `src/menu_syntax/handler_index.rs` |
| Payload v1 | Tempfile JSON contract plus small `KIT_MENU_SYNTAX_*` environment. | `src/menu_syntax/execute.rs` |

## Entry Points

| Entry point | Source | Behavior |
|---|---|---|
| Bare `;` | ScriptList input | Opens capture target picker with built-ins and registered targets. |
| `;<partial>` | ScriptList input | Filters target picker rows by slug and label. |
| `;target <body>` | ScriptList input | Enters capture composer when target is built in or registered. |
| `+target <body>` | Legacy alias | Runs the same target-gated capture path while preserving legacy compatibility. |
| Create-handler footer | Trigger picker | Writes a local scaffold or emits an AI scaffold outcome. |
| Cmd+K in capture mode | ScriptList actions | Opens capture-specific Power Syntax actions. |
| Cmd+Enter in capture mode | ScriptList input | Builds a structured AI request from the current `CaptureInvocation`. |

## User Workflows

### Pick A Capture Target

Typing `;` opens the detached trigger picker. Built-in targets and metadata-registered targets appear as selectable rows; footer actions are enabled but not default-selectable.

### Compose A Capture Body

After a known target plus body boundary, the picker closes and ScriptList enters capture composer mode. The normal launcher list is suppressed, and the body text is parsed as payload fields rather than fuzzy search text.

### Register A Dynamic Target


### Validate And Execute

Enter runs the validation gate. Missing required fields block with target-aware copy; malformed or forbidden fields block before incomplete errors; allowed captures write the v1 JSON payload, record best-effort history, and spawn the selected handler.

### Create A Handler From The Footer

When an unknown slug is typed, the setup-focused hint and footer explain the scaffold path and registration line. Enter writes a non-overwriting local handler file; Cmd+Enter emits an AI scaffold outcome.

### Use Capture Actions Or AI

Cmd+K opens Power Syntax actions such as cancel, copy raw expression, edit payload JSON, change handler, open captures browser, and target-specific safe fixes. Cmd+Enter sends structured capture context to AI and applies only non-stale proposals.

## Interaction Matrix

| User intent | Entry point | UI state | Key/click | Code path | Result | Proof |
|---|---|---|---|---|---|---|
| Open capture picker | ScriptList | Plain search | `;` | `plan_trigger_popup_transition` | Target picker opens. | `menu-syntax-trigger-popup` elements. |
| Filter targets | Trigger picker | Popup open | Type slug | `trigger_picker` filter | Rows narrow by slug/label. | Popup snapshot rows. |
| Enter target body | ScriptList | Exact target | Space/text | `parse_with_capture_targets` | Capture composer starts. | `stateResult.menuSyntaxMainHint`. |
| Keep unknown text as search | ScriptList | Unknown head | `;unknown body` | Parser returns non-capture | Launcher search remains owner. | Parser test and state. |
| Create handler | Trigger picker | Unknown target footer | Enter | Create footer outcome | Scaffold file written without overwrite. | HUD/log/scaffold path. |
| Request AI scaffold | Trigger picker | Unknown target footer | Cmd+Enter | AI scaffold outcome | No local file write. | Footer outcome receipt. |
| Validate missing fields | Composer | Missing required | Enter | `decide_capture_gate_with_accepts` | HUD blocks execution. | No payload file/spawn. |
| Validate malformed field | Composer | Bad URL/amount | Enter | Capture schema validation | Malformed blocks before incomplete. | HUD and no spawn. |
| Execute capture | Composer | Ready | Enter | `menu_syntax_execution` | Payload written and handler launched. | Payload path/env/log. |
| Open actions | Composer | Capture input | Cmd+K | Capture action section | Power Syntax actions shown. | `actionsDialog`. |
| Ask AI to fix capture | Composer | Capture input | Cmd+Enter | Inline AI proposal | Proposal can add field/tag/date or rewrite. | Proposal state and stale guard. |
| Dismiss popup | Trigger picker | Popup open | Escape | Popup key transition | Popup closes before filter clears. | Popup absent; filter stable. |

## State Machine

| State | Enters from | Exits to | Guards |
|---|---|---|---|
| Plain launcher search | Normal typing or unknown capture-looking text | Capture picker, source filter, refine query, file handoff, normal execution | Unknown heads, URLs, localhost strings, top-level tags, and legacy triggers remain search. |
| Bare capture trigger | `;` or legacy `+` | Filtered target picker, Escape, setup footer | Shows capture targets; footer rows are not default-selected. |
| Filtered capture target picker | `;<partial>` | Exact target focus, create-handler footer, Escape | Filtering applies only before body boundary. |
| Exact target focus | `;todo` or registered `;target` without body | Body composer, Escape, create-handler action | Focuses one target row and may show target-specific footer. |
| Unknown target setup | `;<slug>` with no catalog match | Create local handler, AI scaffold request, Escape | Shows setup-focused guidance without stealing search semantics silently. |
| Capture body composer | `;target <body>` | Validation gate, Cmd+K actions, Cmd+Enter AI, Escape, text edit | Body is payload, not fuzzy target search. |
| Capture validation gate | Enter in body composer | Allow execution, block missing HUD, block malformed HUD | No payload file or spawn before validation allows. |
| Handler execution | Gate allowed and handler resolved | Payload written, history recorded, handler spawned | Only `capture.v1` handlers matching target or wildcard can run. |
| Popup dismissed | Escape or accepted target | Main hint repaint, filter clear on next Escape, hide on third Escape | Escape ordering is popup, then filter, then window. |

## Visual And Focus States


## Keystrokes And Commands

| Key | Context | Behavior |
|---|---|---|
| `;` | ScriptList | Opens capture target picker. |
| `+` | ScriptList | Legacy capture alias, target-gated like `;`. |
| Arrow Up/Down | Trigger picker | Moves popup row selection before main-list navigation. |
| Home/End | Trigger picker | Moves to first or last selectable row. |
| Page Up/Page Down | Trigger picker | Jumps to first or last selectable row. |
| Tab/Shift+Tab | Trigger picker | Routes through popup intent handling before sibling ACP branches. |
| Enter | Trigger picker | Accepts selected target, qualifier, or footer outcome. |
| Cmd+Enter | Create footer | Emits AI scaffold outcome. |
| Enter | Capture composer | Runs validation gate, then execution if allowed. |
| Cmd+K | Capture composer | Opens capture-specific Power Syntax actions. |
| Cmd+Enter | Capture composer | Builds structured AI proposal request. |
| Escape | Popup/filter/window | First closes popup, second clears filter, third hides window. |

## Actions And Menus

| Action family | Behavior |
|---|---|
| Cancel | Closes or cancels capture action flow. |
| Copy raw expression | Copies the raw capture expression without running it. |
| Edit payload JSON | Opens the generated payload JSON for inspection/edit flow where supported. |
| Change handler | Lets the user choose among matching capture handlers. |
| Open captures browser | Opens local capture artifact/history browsing. |
| Create handler | Writes a local non-overwriting capture handler scaffold. |
| AI scaffold | Requests a handler scaffold without writing a file directly. |

## Automation And Protocol Surface

| Receipt | What it proves |
|---|---|
| `stateResult.menuSyntaxMainHint` | Capture composer, validation, handler, setup, command warning, and advanced-query empty states without screenshots. |
| `menu-syntax-trigger-popup` elements | Popup rows, selected row, token, detail, footer action, and selection metadata. |
| Input decoration spans | Prefix and typed fragments render as highlights without mutating stored input. |
| `actionsDialog` | Capture-specific Power Syntax action section and selected action. |
| Payload tempfile path | Execution wrote a versioned JSON payload only after validation allowed. |
| Handler env contract | Handler received payload path plus structured `KIT_MENU_SYNTAX_*` env values. |
| History/artifact logs | Best-effort local artifact and per-target history writes. |

## Data, Storage, And Privacy Boundaries

- Raw input is stored in the JSON payload, not duplicated into environment variables.
- Handler execution passes only payload path, feature marker, version, family, target, handler kind, and command id through env.
- Payload v1 is additive; optional duration, recurrence, and unresolved date fields use default/skip serialization rules.
- Capture payload tempfiles are retention-managed separately from user-authored artifacts.
- Retention deletes only caller-filtered `capture_v1-*.json` payload tempfiles, never user JSONL, markdown, calendar, or draft artifacts.
- Tag/key history is local-first, best-effort, and path-safe through slug/key encoding.
- Schema-bound key/value enum values come from schema metadata rather than free-form history.

## Error, Empty, Loading, And Disabled States

- Unknown target heads stay search or setup-focused picker flow; they do not silently steal launcher semantics.
- Top-level `#tag` remains launcher search; tags are capture labels only inside capture/refine contexts.
- Unknown keyword aliases fall through unless registered as capture targets.
- Empty or footer-only picker states keep footer rows enabled but not default-selectable.
- Missing required schema fields block with incomplete copy and write no payload.
- Malformed or forbidden fields block before incomplete fields.
- Unsupported capture actions return unsupported instead of running unsafe side effects.
- Stale AI proposals dismiss when current input differs from the originating raw input.
- Existing scaffold file collisions choose `capture-<slug>-handler-2.ts`, then `-3.ts`, and never overwrite user code.

## Code Ownership

| Behavior | Owner files/tests |
|---|---|
| Public parser classifier | `src/menu_syntax/parse.rs` |
| Capture token parsing | `src/menu_syntax/capture.rs`, `src/menu_syntax/payload.rs` |
| Built-in and dynamic schemas | `src/menu_syntax/capture_schema.rs`, `src/menu_syntax/metadata.rs` |
| Handler filtering/ranking | `src/menu_syntax/filter.rs`, `src/menu_syntax/handler_index.rs`, `src/scripts/grouping.rs` |
| Trigger picker model/keys | `src/menu_syntax/trigger_picker.rs`, `src/menu_syntax/trigger_picker_keys.rs` |
| Popup adapter/window | `src/app_impl/menu_syntax_trigger_popup.rs`, `src/app_impl/menu_syntax_trigger_popup_window.rs` |
| Main hint adapter | `src/menu_syntax/main_hint.rs`, `src/app_impl/menu_syntax_main_hint.rs` |
| Execution payload/env | `src/app_execute/menu_syntax_execution.rs`, `src/menu_syntax/execute.rs` |
| Source tests | `tests/menu_syntax_source_filters.rs`, `tests/file_search_tilde_entry.rs` |

## Invariants And Regression Risks

- `parse` is the public classifier for Power Syntax ownership.
- Unknown capture heads, unknown keyword heads, URLs, localhost strings, top-level tags, and legacy triggers must not become capture.
- Capture body text is payload, not fuzzy search.
- Handler matching is metadata-based, not body-search-based.
- Only `capture.v1` handlers can handle capture.
- Footer rows are enabled but not default-selectable.
- The popup closes when capture body composition starts.
- The main launcher list is suppressed while capture composer owns input.
- Hint content is read-only and must not become selectable launcher rows.
- Malformed validation beats incomplete validation.
- A blocked validation gate writes no payload and spawns no process.
- Dynamic examples must not promote demo targets into parser-known built-ins accidentally.
- `mcal` remains parser-known and schema-capable but not bare-picker-visible unless registered.
- Escape ordering stays popup, filter, window.

## Verification Recipes


```bash
cargo test menu_syntax
cargo test menu_syntax_source_filters
cargo test file_search_tilde_entry
cargo fmt --check
git diff --check
source checks
```


- `;todo Renew passport #errands p1` parses target, body, tag, and priority.
- `;github body` captures only after `github` is registered by metadata.
- Bare `;` opens the trigger picker and exposes popup elements.
- `;gcal` with no match shows footer-only create-handler setup guidance.
- Footer Enter writes a scaffold without overwriting; Cmd+Enter emits AI scaffold outcome.
- `;cal` with no body/date blocks before payload execution.
- Bad `;link` URL blocks malformed before incomplete.
- Unknown no-schema custom targets allow execution when a handler exists.
- `stateResult.menuSyntaxMainHint` exposes composer, validation, setup, and handler hints.
- First Escape closes popup only; second clears filter; third hides the window.

Screenshots are only needed for popup placement or visual highlight acceptance after state receipts prove ownership.

## Agent Notes

- Treat this as a focused Power Syntax capture slice, not the broad main-menu or special-entry handoff chapter.
- Keep parser boundary tests close to changes; capture ownership bugs can steal normal launcher search.
- When debugging target registration, check metadata indexing before parser code.
- When Enter does not run a handler, inspect schema validation before handler ranking.
- When popup behavior looks wrong, verify footer selectability and popup-before-filter Escape handling.
- When automation sees selectable hint rows, inspect composer ownership; hint content should be read-only.

## Related Features

- [001 Main Menu](./001-main-menu.md)
- [013 ScriptList Special Entry Triggers](./013-scriptlist-special-entry-triggers.md)
- [041 Main Menu Renderer Key Handling](./041-main-menu-renderer-key-handling.md)

## Open Questions And Gaps

- Full source-backed inventories for all current capture fixture/demo targets should be kept separate from the stable built-in taxonomy.
- Pixel-level popup placement and highlight span acceptance still need visual proof when UI styling changes.
- Scriptlet nested metadata remains flatter than full TypeScript metadata for complex capture authoring.
- Unsafe backlog actions remain unsupported until an owner defines exact side effects and receipts.
