# Script Kit GPUI

Rust (GPUI app shell) + TypeScript (bun script runner) + SDK. Backwards-compatible rewrite of Script Kit.

## Scope Rules

- Do ONLY what is explicitly requested. No unrequested changes, refactors, or "improvements."
- If you notice something worth improving, mention it at the end — do not implement it.
- Stay within the boundaries of the task. A docs request is not a code change.

## Verification Gate (Mandatory)

Every code change must pass before reporting success:

```bash
cargo check && cargo clippy --all-targets -- -D warnings && cargo test
```

After the gate passes, verify the change actually works:
- **Logic changes**: check logs with `SCRIPT_KIT_AI_LOG=1`
- **UI changes**: capture screenshot AND read the PNG to confirm visually
- **Never** report success without running verification

## Build & Test

| Action | Command |
|--------|---------|
| Check | `cargo check` |
| Format check | `cargo fmt --check` |
| Lint | `cargo clippy --all-targets -- -D warnings` |
| Test | `cargo test` |
| Test (CI) | `cargo nextest run` |
| Test (system) | `cargo test --features system-tests` |
| Test (slow) | `cargo test --features slow-tests` |
| Run | `echo '{"type":"show"}' \| SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1` |
| Bundle | `cargo bundle --release` |

## Coding Conventions

### Rust
- Use `?` or graceful error handling — never `unwrap()` in unsafe/ObjC code
- After any render-affecting mutation: `cx.notify()`
- Use `theme.colors.*` — never hardcode `rgb(0x...)`
- Every `unsafe` block must include a `// SAFETY:` comment.
- Use `SharedString` for UI-facing text props; `String` for internal state.
- Font: use `FONT_MONO` constant, never hardcode font family strings.
- Keyboard keys — prefer `is_key_*` helpers from `crate::ui_foundation`:
  ```rust
  use crate::ui_foundation::{is_key_up, is_key_down, is_key_enter, is_key_escape, ...};
  let key = event.keystroke.key.as_str();
  if is_key_up(key) { ... }
  ```
  If raw matching is needed, always match both variants: `"up" | "arrowup"`, `"enter" | "Enter"`, etc.

### UI Testing
- **Never** pass scripts as CLI args — use stdin JSON protocol
- Always use `SCRIPT_KIT_AI_LOG=1` for compact log output
- After screenshots, **read the PNG file** to verify

## User Feedback Rules

| Feedback type | When to use | Duration |
|---------------|------------|----------|
| **HUD** (show_hud()) | Lightweight confirmations: 'Copied', 'Saved', 'Pinned', status toggles | HUD_SHORT_MS to HUD_MEDIUM_MS |
| **Toast** (toast_manager.push()) | Errors, warnings, multi-line info, messages needing user attention | TOAST_SUCCESS_MS to TOAST_CRITICAL_MS |
| **Silent** (no feedback) | View transitions where the new view IS the feedback (opening ClipboardHistory, EmojiPicker) | N/A |

Rules:
- Never use last_output for new code — it is deprecated.
- Never use inline duration numbers — always use named constants from helpers.rs.
- Every error path must show Toast with .error() variant.
- Success feedback is optional for view transitions but required for side-effect operations (copy, delete, save).
- Never use both HUD and Toast for the same action.

## Compilation Context

- `include!()` into `main.rs` (shared `main.rs` scope): `main_sections/`, `app_impl/`, `render_prompts/`, `app_execute/`, `app_navigation/`, `prompt_handler/`, `execute_script/`, `render_script_list/`, `render_builtins/`, `app_actions/`, `app_render/`, `app_layout/`.
- In `include!()` files: NO top-level `use` statements.
- In `include!()` files: NO `mod` declarations.
- In `include!()` files: use fully qualified paths or existing `main.rs` scope imports.
- Proper module trees (normal `mod` + `use crate::...`): `theme/`, `protocol/`, `prompts/`, `components/`, `scripts/`, `builtins/`, `ai/`, `notes/`, `platform/`, `hotkeys/`, `watcher/`.

## GPUI Lifecycle Rules

1. `render()` is read-only: no state mutation and no `cx.notify()`.
2. After any state mutation that affects UI, call `cx.notify()`.
3. For async work, use `cx.spawn(...)`; do not use `std::thread::spawn`.
4. Store subscriptions in struct fields, or explicitly call `.detach()`.
5. Store spawned tasks in struct fields, or explicitly call `.detach()`.
6. Closures outliving entities must capture `WeakEntity` via `.downgrade()`.
7. Never create entities (`cx.new()`) inside `render()` — causes per-frame state loss and leaked subscriptions.
8. Render trait: `fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement`. RenderOnce: `fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement`. Use `Render` for stateful views, `RenderOnce` for stateless consumed elements.
9. Use `cx.listener(|this, event, window, cx| { ... })` to create entity-bound callbacks in render context.
10. Flex children containing lists need `.min_h(px(0.))` to prevent overflow beyond parent bounds.

## Keyboard Event Propagation

- Call `cx.stop_propagation()` after handling a key to prevent parent handlers from also processing it.
- In the `_ =>` fallthrough arm of key handlers, call `cx.propagate()` so unhandled keys bubble up.
- Use `window.dispatch_action(action)` (not `cx.dispatch_action`) to dispatch actions from key handlers.
- **Deep dive**: See [`GPUI.md`](GPUI.md) for the full event dispatch architecture — dual dispatch (actions vs raw key events), propagation asymmetry, two-phase capture/bubble model, and common pitfalls.

## ObjC Interop Rules

- Runtime crate is `objc = 0.2` (NOT `objc2`).
- Correct imports: `objc::{class, msg_send, sel, sel_impl}`.
- Use `msg_send!` with explicit return types at call sites.
- Nil-check Objective-C pointers before sending follow-up messages.
- `src/platform/*.rs` often use `include!()` flat namespace rules.
- Never call `orderOut:` directly; use `defer_hide_main_window`.
- Use `c""` string literals for ObjC string interop.
- Call `require_main_thread()` at the start of any function that touches AppKit APIs.
- Every `unsafe` block must include a `// SAFETY:` comment explaining the invariants being upheld.
- Every `#[cfg(target_os = "macos")]` function needs a `#[cfg(not(target_os = "macos"))]` no-op stub.

## Async & Channel Discipline

- Use bounded channels only: `async_channel::bounded(...)`.
- Guard async result application with generation counters (drop stale updates).
- Use `parking_lot::Mutex` (non-poisoning) for shared mutable state.
- Use `cx.background_executor().timer(...)` for delays/timeouts.
- Prefer cancellation-safe flows that can early-return on stale generation checks.
- No tokio — GPUI has its own async executor. Use `cx.spawn()` and `cx.background_executor()`.
- Release `Mutex` locks before calling `entity.update()` or `cx.update()` to prevent deadlocks.
- Never hold `Mutex` locks across `.await` boundaries.
- Use `tx.send_blocking()` for sync-to-async channel bridges from `std::thread::spawn`.
- Channel capacity: `1` for one-shot confirmations, `100-256` for streaming data.

## Serde Protocol Contracts

- Protocol structs/enums use `#[serde(rename_all = "camelCase")]`.
- Message enums use `#[serde(tag = "type")]` tagging.
- Optional/deprecated input fields use `#[serde(default)]`.
- Optional output fields use `#[serde(skip_serializing_if = "Option::is_none")]`.
- Keep wire names stable; add defaults before adding new required fields.
- Simple string enums (roles, modifiers): use `#[serde(rename_all = "lowercase")]`.
- Use `#[serde(untagged)]` only when enum variants are structurally distinct (different field sets).

## Theme Details

- Opacity: use constants from `src/theme/opacity.rs` (`OPACITY_HOVER`, `OPACITY_SELECTED`, etc.) — never magic floats.
- Color methods via `HexColorExt` trait: `.to_rgb()`, `.rgba8(alpha_byte)`, `.with_opacity(f32)`.
- Two theme systems coexist: `get_cached_theme()` (Script Kit's cached theme) and `cx.theme()` (gpui-component's theme). Prefer `get_cached_theme()` for Script Kit UI; `cx.theme()` only in gpui-component wrappers.

## Error Handling Patterns

- Use `.context("message")?` or `.with_context(|| format!(...))?` on all fallible operations (`anyhow::Context`).
- For recoverable errors in event handlers: use `.log_err()` or `.warn_on_err()` (`ResultExt` trait).
- For domain errors callers pattern-match on: define with `thiserror`, not `anyhow`.
- `bail!("message")` for precondition failures.
- Never log full protocol messages — they may contain base64 screenshots or clipboard data.

## Component Structure

- 4-file split: `component.rs` (struct + impl), `types.rs` (Colors/Config), `render.rs` (Render impl), `tests.rs`.
- Colors struct: `#[derive(Clone, Copy)]` with `from_theme(&Theme)` constructor — extract BEFORE closures.
- Stateless elements: `#[derive(IntoElement)]` + `impl RenderOnce` (consumed on render).
- Stateful views: `impl Render` (borrowed, survives across frames).

## High-Risk Files

| Path | Why High Risk |
|---|---|
| `src/platform/*.rs` | ObjC interop + window lifecycle side effects |
| `src/main_sections/render_impl.rs` | Central render dispatch and view routing |
| `src/main_sections/app_state.rs` | Shared app state and mutation pathways |
| `src/protocol/message/mod.rs` | Wire protocol compatibility and serde contracts |
| `src/prompts/term_prompt/mod.rs` | Terminal prompt IO flow and interaction edge cases |

**Rule:** Read the full file before editing any of the above.

## Architecture Quick Ref

- Built-in commands: `BuiltInFeature` (`src/builtins/mod.rs`) → `get_builtin_entries()` (startup/search) → `execute_builtin()` (`src/app_execute/builtin_execution.rs`) → `AppView` (`src/main_sections/app_view_state.rs`) → render dispatch (`src/main_sections/render_impl.rs`)
- Built-in caveat: some built-ins open external windows or perform side effects without setting `AppView` (AI/Notes/system/menu/quicklinks paths in `src/app_execute/builtin_execution.rs`)
- Non-dismissable views: add to `is_dismissable_view()` in `src/app_impl/shortcuts_hud_grid.rs`
- Vibrancy: prompts should NOT set opaque bg — let vibrancy show through from Root
- Prompt rendering split: `src/render_prompts/*.rs` are outer wrappers; `src/prompts/**` are inner prompt entities (Arg prompt remains inline in `src/render_prompts/arg.rs`)
- Protocol: bidirectional JSONL over stdin/stdout between bun scripts and Rust app — see `docs/PROTOCOL.md`, runtime code in `src/protocol/**` and `src/stdin_commands/mod.rs`
- Organization: there is no monolithic `app_impl.rs`; app logic is split across `src/main_sections/`, `src/app_impl/`, `src/app_execute/`, and `src/render_*` modules

## AI Context & Introspection

### Element Introspection (`getElements`)

Protocol command allowing scripts to query the live UI surface. Returns semantic IDs, element types, and observation metadata.

**Key files:**
- `src/protocol/message/variants/query_ops.rs` — `GetElements` / `ElementsResult` message variants
- `src/protocol/types/elements_actions_scriptlets.rs` — `ElementType` enum (Choice, Input, Button, Panel, List, Unknown), `ElementInfo` struct
- `src/app_layout/collect_elements.rs` — `ElementCollectionOutcome`, per-view collectors (`collect_visible_elements()`)
- `src/prompt_handler/mod.rs` — request handler (clamps limit, builds receipt)
- `src/protocol/message/constructors/query_ops.rs` — `get_elements()`, `elements_result()` constructors

**Element types:** `Choice`, `Input`, `Button`, `Panel`, `List`, `Unknown` (forward-compatible)

**Semantic ID format:** `input:filter`, `list:choices`, `choice:<index>:<value>`, `button:<index>:<label>`, `panel:<type>`

**Observation receipts** (on every `ElementsResult`):
- `focused_semantic_id` / `selected_semantic_id` — extracted from `ElementCollectionOutcome`
- `truncated` — `true` when `total_count > returned elements`
- `warnings` — machine-readable codes for views with limited introspection:
  - `panel_only_theme_chooser`, `panel_only_actions_dialog`, `panel_only_div_prompt`, `panel_only_form_prompt`, `panel_only_editor_prompt`, `panel_only_chat_prompt`, `panel_only_env_prompt`, `panel_only_drop_prompt`, `panel_only_template_prompt`, `panel_only_naming_prompt`, `panel_only_webcam`, `panel_only_scratch_pad`, `panel_only_quick_terminal`, `collector_used_current_view_fallback`

**Tests:** `src/protocol/types/tests/get_elements.rs` (request parsing, response roundtrip, semantic IDs, truncation, receipts)

### MCP Desktop Context (`kit://context`)

Exposes a deterministic, schema-versioned snapshot of ambient desktop state as an MCP resource.

**Key files:**
- `src/mcp_resources/mod.rs` — resource definition (URI `kit://context`), read handler (`read_context_resource()`), query parameter parsing, schema generation
- `src/context_snapshot/types.rs` — `CaptureContextOptions` (profiles), `AiContextSnapshot`, `FrontmostAppContext`, `BrowserContext`, `FocusedWindowContext`, `MenuBarItemSummary`
- `src/context_snapshot/capture.rs` — `capture_context_snapshot()` (live), `capture_context_snapshot_from_seed()` (deterministic for tests)

**Profiles:**
- `CaptureContextOptions::all()` — all fields (default, `?profile=full`)
- `CaptureContextOptions::minimal()` — excludes `selected_text` and `menu_bar` (`?profile=minimal`)

**Per-field flags:** `?selectedText=0|1`, `?frontmostApp=0|1`, `?menuBar=0|1`, `?browserUrl=0|1`, `?focusedWindow=0|1` — accepts `0`, `1`, `true`, `false`

**Special URIs:**
- `kit://context/schema` — self-describing JSON with profiles, parameters, and diagnostics schema
- `kit://context?diagnostics=1` — adds `ContextFieldStatus` per field (disabled/captured/empty/failed) and overall status

**URI routing:** `kit://context` → `read_resource()` → `read_context_resource()` → `parse_context_resource_request()` → `capture_context_snapshot()` → `serialize_context_resource()`

**Tests:** `tests/context_snapshot.rs` (resource listing, JSON validity, profile stability, minimal resolution, content validation)

### Typed Context Parts & Resolution

Composable context attachments for AI chat flows with deterministic resolution and partial-failure tolerance.

**Key files:**
- `src/ai/message_parts.rs` — `AiContextPart` enum, `ContextResolutionReceipt`, `resolve_context_parts_with_receipt()`, `resolve_context_part_to_prompt_block()`
- `src/ai/window/context_commands.rs` — slash commands (`/context`, `/context-full`, `/selection`, `/browser`, `/window`)
- `src/ai/window/state.rs` — `pending_context_parts: Vec<AiContextPart>` (composer state)
- `src/ai/window/streaming_submit.rs` — resolution at submit time

**`AiContextPart` enum** (serde-tagged by `kind`):
- `ResourceUri { uri, label }` — MCP resource (e.g., `kit://context?profile=minimal`)
- `FilePath { path, label }` — local file attachment

**`ContextResolutionReceipt`:**
- `attempted` / `resolved` — counts for success/failure tracking
- `failures: Vec<ContextResolutionFailure>` — each with label, source, error message
- `prompt_prefix` — concatenated `<context>` / `<attachment>` blocks from successful parts
- `has_failures()` — convenience check

**Resolution algorithm:**
1. `ResourceUri` → `mcp_resources::read_resource()` → wrap in `<context source="..." mimeType="...">...</context>`
2. `FilePath` → read file → wrap in `<attachment path="...">...</attachment>`; unreadable files get `<attachment path="..." unreadable="true" bytes="N" />`
3. Failures recorded but don't block other parts; `prompt_prefix` contains only successful blocks

**Slash command mappings:**
| Command | URI | Label |
|---------|-----|-------|
| `/context` | `kit://context?profile=minimal` | Current Context |
| `/context-full` | `kit://context` | Full Context |
| `/selection` | `kit://context?selectedText=1&frontmostApp=0&menuBar=0&browserUrl=0&focusedWindow=0` | Selected Text |
| `/browser` | `kit://context?selectedText=0&frontmostApp=0&menuBar=0&browserUrl=1&focusedWindow=0` | Browser URL |
| `/window` | `kit://context?selectedText=0&frontmostApp=0&menuBar=0&browserUrl=0&focusedWindow=1` | Focused Window |

**Tests:**
- `tests/context_part_resolution.rs` — serde roundtrip, single/multi-part resolution, receipt tracking, partial failure
- `tests/context_part_composer_state.rs` — file_path_parts extraction, order preservation, removal
- `tests/context_part_start_chat_flow.rs` — empty message + parts, message + parts, invalid parts, mixed success, order
- `tests/context_part_submission_flow.rs` — mixed success tracking, full success prefix persistence

## Consistency Rules (Non-Negotiable)

These rules exist because mixed patterns break both human navigation and AI agent effectiveness.

### 1) No `part_*.rs` files
- Do NOT create or extend `part_000.rs`, `part_001.rs`, etc.
- Do NOT use `include!("part_*.rs")` for hand-written code.
- If a module is too large, split into a directory module with named files:
  - `mod.rs` is a facade that does `mod foo; mod bar;` and `pub use ...;`
  - Filenames must be semantic (`model.rs`, `render.rs`, `storage.rs`), never numeric.

### 2) Tests have only two homes (pick the right one)
- Unit tests live next to code: `src/<module>/tests.rs` (referenced via `#[cfg(test)] mod tests;`)
- Integration tests live in `tests/<feature>/mod.rs` (may have submodules + fixtures)
- Never create numbered test directories (`*_tests_2`, `*_tests_3`, ...). Use semantic names.

### 3) No unwrap/expect in production code
- In `src/` (non-test code), `.unwrap()` and `.expect()` are forbidden.
- Use `?` + `anyhow::Context`, or explicit handling + logs.
- Tests may use `.expect("useful message")`.

### 4) Logging: one canonical API
- Use `tracing::{info,warn,error,debug,trace}` for all new/modified code.
- Do not introduce new `log::info!` / `log::warn!` usage in `src/`.
- Prefer structured fields over string formatting.

### 5) Module visibility: default private + facade exports
- Default: private items.
- Use `pub(crate)` for cross-module internals.
- Use `pub` only when intentionally part of the crate's public surface.
- Export intentional API via `pub use` from the module's facade file.

---

## Vendored Dependencies

`gpui` is vendored locally from Zed tag `v0.226.0-pre`.
Starting at `v0.226.0-pre`, GPUI is split into vendored crates under
`vendor/`: `gpui` (core), `gpui_platform` (platform abstraction),
`gpui_macos` (macOS backend), and `gpui_macros` (proc macros).
`gpui-component` (UI component library) is vendored locally at
`vendor/gpui-component/` from its upstream repository.
These are intentional local copies so we can patch behavior without
waiting for upstream releases.
One common reason is adding repository-specific hooks such as layout
debugging instrumentation.

## Session Completion

Work is not done until `git push` succeeds.

1. Run verification gate (check/clippy/test)
2. Commit with descriptive message
3. `git pull --rebase && git push && git status`
4. Never say "ready to push when you are" — just push

## Skills (Loaded On-Demand)

Detailed guidance lives in `.claude/skills/` — load only when relevant:

| Skill | When to Use |
|-------|-------------|
| `script-kit-agent-workflow` | Fix-verify loop, session completion |
| `script-kit-ui-testing` | Screenshots, stdin JSON protocol, layout debugging |
| `gpui-patterns` | UI code, keyboard events, layouts, themes |
| `visual-test` | Visual iteration, named-pipe testing, captureWindow |
| `dev-loop` | Background dev server, log monitoring, runtime verification |
| `script-kit-architecture` | Navigating codebase, understanding modules |
| `script-kit-logging` | Adding logs, observability, correlation IDs |
| `script-kit-testing` | Writing tests, test organization |
| `script-kit-scripting` | Script metadata, scriptlet bundles |
| `script-kit-hive` | Task management, beads, issue tracking |

**When to load skills:** If editing `src/platform/` load `gpui-patterns`. If editing `src/prompts/` or `src/render_prompts/` load `gpui-patterns`. If writing tests load `script-kit-testing`. If adding protocol messages load `script-kit-architecture`. If debugging UI load `script-kit-ui-testing` + `visual-test`.

## References

- GPUI docs: https://docs.rs/gpui/latest/gpui/
- Zed source: https://github.com/zed-industries/zed/tree/main/crates/gpui
- Protocol reference: `docs/PROTOCOL.md`

## Design Context

### Users
Power developers and automation enthusiasts who demand speed and precision. They invoke Script Kit as a launcher/command palette — it must appear instantly, respond to keystrokes without lag, and disappear the moment the task is done. The interface should evoke **confidence**: every interaction feels deliberate, fast, and under their control.

### Brand Personality
**Fast. Focused. Minimal.**

Script Kit is a sharp tool, not a playground. It respects the user's time and attention. No unnecessary animation, no visual noise, no chrome that doesn't earn its place. The gold accent (#fbbf24) is the one warm touch — a signature that says "this is Script Kit" without shouting.

### Aesthetic Direction
- **Reference:** Raycast — clean launcher with macOS vibrancy, polished transitions, keyboard-first interaction, information-dense but not cluttered
- **Anti-references:** Electron apps with visible latency, over-decorated dashboards, anything that feels like a web page pretending to be native
- **Theme:** Dark mode primary with native macOS vibrancy (popover blur). Semi-transparent backgrounds let the desktop bleed through. Light mode supported but secondary
- **Visual tone:** Native macOS feel — if Apple made a scriptable launcher, it would look like this. Precision over personality

### Design Principles

1. **Speed is the design** — Every pixel serves instant comprehension. If an element slows the user down (visually or mechanically), remove it. Sub-frame response to input is non-negotiable.

2. **Keyboard-first, always** — The mouse is a fallback. Every interaction must be reachable and obvious via keyboard. Visual affordances should reinforce keyboard shortcuts, not compete with them.

3. **Quiet confidence** — Use restraint. One accent color (gold). One font stack (system). Subtle opacity shifts over hard borders. Let vibrancy and spacing do the work instead of decoration.

4. **Native or nothing** — Respect macOS conventions. Vibrancy, system fonts, PopUp panel behavior, proper focus/unfocus dimming. Users should forget they're in a third-party app.

5. **Information density without clutter** — Show everything the user needs in a single glance. Use typography hierarchy (weight, size, opacity) to create visual layers, not boxes and dividers.
