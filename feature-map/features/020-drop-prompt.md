# 020 Drop Prompt / drop()

This chapter maps the SDK-facing file drop prompt and its empty/submit/footer contracts.


## Executive Summary


```ts
```




## What Users Can Do

| User capability | Entry | Result |
|---|---|---|
| Drop one or more files. | OS drag/drop onto prompt. | Intended to populate dropped file metadata; wiring needs runtime proof. |
| See dropped count. | Files present. | UI shows `N file(s) dropped)`. |
| Submit dropped files. | Enter or footer Submit. | Resolves SDK promise with `FileInfo[]`. |
| Avoid empty submit. | No files present. | Footer Submit is disabled and prompt submit guard no-ops. |
| Cancel prompt. | Escape. | Submit callback receives cancellation; SDK treats `null` as Escape/exit. |
| Inspect with automation. | `getState`, `getElements`, `getLayoutInfo`. | Agents can identify prompt type, footer disabled state, layout, and dropped file elements. |

## Core Concepts

| Concept | Meaning | Contract |
|---|---|---|
| `drop()` SDK API | Script-facing file metadata prompt. | No visible public args; resolves `FileInfo[]`. |
| `DropPrompt` | Rust prompt entity. | Owns id, placeholder, hint, dropped files, drag-over state, focus handle, submit callback, theme, design variant. |
| Empty disabled footer | Empty state guard. | Submit remains visible but disabled with `no_files`; submit method also guards empty state. |
| Drop event wiring | OS drop integration. | `.on_drop` routes GPUI `ExternalPaths` into prompt-owned `DroppedFile` state. |

## Entry Points

| Entry | Context | Result |
|---|---|---|
| `render_drop_prompt`. | Render dispatch. | Draws drop target UI and surface-specific hints. |
| Native footer. | Main window footer. | Submit disabled until files exist; Actions affordance shown. |
| `collect_elements`. | Protocol inspection. | Exposes dropped file list only when files exist. |

## User Workflows

### Open Empty Drop Prompt


```ts
const files = await drop()
```


### Empty Submit Attempt


### Drop Files


### Submit Files


### Cancel

Escape calls `submit_cancel`, which invokes the submit callback with `None`. The SDK treats `msg.value === null` as Escape and exits the script process.

## Interaction Matrix

| User intent | Entry point | UI state | Key/click | Code path | Result | Proof |
|---|---|---|---|---|---|---|
| Inspect empty state. | Protocol. | Empty DropPrompt. | `getState`. | App view state mapping. | Prompt type `drop`, no input, counts zero, selected index -1. | `src/prompt_handler/mod.rs`. |
| Inspect footer empty state. | Protocol. | Empty DropPrompt. | `getState` active footer. | Drop footer branch checks no files. | Submit disabled with `no_files`; Actions present. | `src/app_impl/ui_window.rs`, `removed-docs`. |
| Drop files. | OS drag/drop. | Drop target. | File drop. | `.on_drop` -> `handle_external_paths` -> `handle_drop`. | Dropped files populate state. | `src/prompts/drop.rs`. |
| Render files present. | DropPrompt with files. | Count visible. | Render. | `render` branches on `dropped_files.is_empty()`. | User sees `N file(s) dropped`. | `src/prompts/drop.rs`. |
| Inspect dropped files. | Protocol. | Files present. | `getElements`. | DropPrompt collector. | List plus one choice row per file. | `src/app_layout/collect_elements.rs`. |
| Cancel. | DropPrompt active. | Any state. | Escape. | `submit_cancel`. | SDK receives null/cancel. | `src/prompts/drop.rs`, `scripts/kit-sdk.ts`. |
| Open Actions. | DropPrompt footer. | DropPrompt active. | Cmd+K/footer Actions. | Shared dispatcher. | Actual action host support is unproven. | `src/app_impl/ui_window.rs`, `src/main_sections/app_view_state.rs`. |

## State Machine

| State | Trigger | Transition | Notes |
|---|---|---|---|
| SDK idle. | No drop active. | Script continues. | No pending drop id. |
| Drop request created. | `drop()`. | Create id, send drop message. | Public API has no args in bundle. |
| Empty. | Initial state. | Submit disabled; dropped file list empty. | Submit guard prevents callback. |
| Drag over. | OS drag enters target. | `is_drag_over` should become true. | Setter path not proven. |
| Files dropped. | OS drop event. | `handle_drop` sets `dropped_files`. | Event hook not proven. |
| Files present. | Dropped files non-empty. | Footer Submit enables; elements expose files. | UI shows count only in bundle. |
| Submit. | Enter/footer Submit. | Serialize metadata and resolve SDK. | Path/name/size only. |
| Cancel. | Escape. | Callback with `None`; SDK exits. | Same for empty or files-present state. |

## Visual And Focus States

| State | Visible result | Focus owner | Automation signal |
|---|---|---|---|
| Drag over. | Active drop colors intended. | DropPrompt. | `is_drag_over` exists; transition proof gap. |
| Files present. | Count text, Submit enabled. | DropPrompt. | `getElements` dropped-files list and row choices. |
| Cancelled. | Prompt closes/SDK exits. | None. | Submit callback receives null. |
| Actions requested. | Footer affordance visible. | Dispatcher/action host uncertain. | Needs runtime proof. |

## Keystrokes And Commands

| Input | Scope | Behavior |
|---|---|---|
| Enter. | Empty DropPrompt. | Calls submit guard; no callback. |
| Enter. | Files present. | Submits JSON metadata array. |
| Footer Submit. | Empty DropPrompt. | Disabled with `no_files`. |
| Escape. | DropPrompt. | Cancels with `None`. |
| Cmd+K / Actions. | DropPrompt footer. | Affordance exists; actual actions host support unproven. |

## Actions And Menus


- Actions opens a real DropPrompt-scoped action popup.
- Actions intentionally no-ops with safe state.
- Actions is a stale footer affordance that should be removed or implemented.

## Automation And Protocol Surface

| Automation target | Assertion |
|---|---|
| `getState`. | Prompt type `drop`, active id, no text input, counts zero, selected index -1, and `drop.fileCount` / redacted `drop.files[]`. |
| `getElements` empty. | No dropped file body elements when empty. |
| `getLayoutInfo`. | Prompt type/drop content, not launcher list/preview. |
| simulateKey Enter empty. | Prompt remains active and does not resolve. |
| simulateKey Escape. | Cancels prompt; remember simulateKey is fire-and-forget. |
| File drop. | Needs native/manual drop proof because event wiring is not proven. |
| ForceSubmit. | Not proven for DropPrompt. |

## Data, Storage, And Privacy Boundaries

- File contents, MIME type, directory flag, modified time, and image bytes are not part of SDK drop in the bundle.
- Paths are exposed to scripts only in the final SDK submit payload; automation state/elements expose basename, index, and byte size only.
- Screenshots can expose private path names or filenames.
- Rust `u64` size becomes a JS number and can lose precision for extremely large values.
- Chat/AI image drops read file bytes; SDK DropPrompt does not in the captured source.

## Error, Empty, Loading, And Disabled States

| State | Behavior |
|---|---|
| Empty. | Submit disabled in footer and guarded in `submit`. |
| Invalid submit JSON. | SDK parser falls back to `[]`. |
| Non-array submit value. | SDK parser falls back to `[]`. |
| Cancel/null. | SDK treats as Escape and exits process. |
| Non-file drop. | Not proven. |
| Directory drop. | Not proven; returned metadata has no `is_dir`. |
| Dropped file deleted before submit. | Not checked in bundle; metadata may still submit. |
| Loading. | No explicit loading state proven. |
| Drag-over transitions. | Drop handling works, but visual enter/leave transitions remain unproven. |

## Code Ownership

| Area | Owner |
|---|---|
| SDK API and parsing. | `scripts/kit-sdk.ts` owns `FileInfo`, `drop()`, null handling, JSON parse fallbacks. |
| Drop state/render. | `src/prompts/drop.rs` owns `DropPrompt`, `DroppedFile`, render, submit, cancel, and internal drop handling. |
| Render dispatch. | `src/main_sections/render_impl.rs` and `src/render_prompts/other.rs` route and wrap `render_drop_prompt`. |
| Footer. | `src/app_impl/ui_window.rs` owns Submit/Actions footer, disabled reason, and Run dispatch. |
| App-view contracts. | `src/main_sections/app_view_state.rs`, `src/focus_coordinator/mod.rs`, `theme_focus`, and orchestrator bridge own focus/surface identity. |
| Automation. | `src/app_layout/collect_elements.rs`, `build_layout_info`, and protocol state helpers own receipts. |
| Adjacent drop flows. | `src/ai/window/*`, `src/prompts/chat/*`, File Search drag tests, and `platform/permiso/drag_source.rs` are separate drop/drag features. |

## Invariants And Regression Risks

- Public SDK API is no-argument `drop()` unless deliberately changed across docs/types/runtime.
- Return shape is exactly `path`, `name`, `size` in the captured API.
- Empty submit must not complete.
- DropPrompt footer must omit launcher AI.
- Native footer surface id must remain `drop_prompt`.
- DropPrompt stays prompt-owned DivPrompt layout, not launcher list/preview.
- `getState.choiceCount` is not a dropped file count.
- `getElements` file rows are the reliable automation surface after files exist.
- Real OS drop wiring must be protected by runtime/source proof.
- Actions affordance should be either proven, implemented, or corrected.

## Verification Recipes

| Recipe | Expected proof |
|---|---|
| Initial state. | `getState` reports prompt type `drop`, prompt id, empty input, zero counts, selected index -1. |
| Empty Enter. | simulateKey Enter leaves prompt active and produces no submit result. |
| Escape. | simulateKey Escape cancels; SDK null path exits. |
| Actual file drop. | Native/manual drop populates files, UI count changes, Submit enables, elements list files. |
| Submit with files. | Enter/footer Submit resolves `FileInfo[]`; launcher fallback does not run. |
| Element semantics. | `getState.drop` and `getElements` expose redacted `{index,name,size}` metadata and omit paths. |
| Layout. | `getLayoutInfo` shows DropContent prompt, not launcher list/preview. |
| Actions. | Cmd+K/footer Actions behavior is recorded and either proven supported or documented as a gap. |
| Adjacent-flow boundary. | ACP/chat image drops and File Search drag-out are not used as proof of SDK `drop()`. |

## Agent Notes

Use `getState.activeFooter` to prove disabled empty submit; screenshots are weaker for this state.

Use `getElements` after files are present to prove metadata. `choiceCount` is not file count for DropPrompt in the captured state arm.

Do not claim `drop("Drop files here")` is public API unless the SDK signature changes. Embedded docs appear stale relative to the SDK source.

Do not claim SDK DropPrompt reads file contents. It submits metadata only.

Do not conflate SDK DropPrompt with ACP/chat image drops or File Search drag-out.

Verify the actual drag/drop event wiring first. The handler exists; the hook is not proven in the bundle.

## Related Features

| Feature | Relationship |
|---|---|
| [002 File Search / Browser / Attachment Portals](./002-file-search.md). | Separate drag-out and filesystem browsing feature with richer metadata and portals. |
| [003 Agent Chat Context Composer](./003-agent-chat-context.md). | Separate chat attachment/drop flows that can read file/image contents. |
| [019 Path Prompt](./019-path-prompt.md). | File/folder picker with Select footer; DropPrompt is drag/drop metadata submission. |
| [016 Prompt Runtime Core](./016-prompt-runtime-core.md). | Shares prompt shell, footer, and automation concepts. |
| Permission Assistant Drag Source. | Native drag source exists for permissions; shared behavior with DropPrompt is not proven. |

## Open Questions And Gaps

- Where is `is_drag_over` set to true and false outside `handle_drop`?
- Does DropPrompt actually support Cmd+K Actions?
- Should public SDK `drop()` accept placeholder/hint, or should embedded docs be corrected?
- Is ForceSubmit supported?
- What happens on non-file drops?
- Are directories accepted or rejected?
- What happens if dropped files are deleted before submit?
- Are file paths logged through generic submit plumbing?
- Should visible UI render individual filenames instead of only count?
