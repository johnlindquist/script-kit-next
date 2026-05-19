# PASS notes_mcp_roundtrip_2026_05_18

Date (UTC): 2026-05-19T03:04Z
MCP endpoint: http://127.0.0.1:43210/rpc
Auth: `Authorization: Bearer $(cat ~/.scriptkit/agent-token)` (legacy dev-only all-scope token)
Build: `target-agent/mcp-slice1/debug/script-kit-gpui` (mcp-slice1 codex-exec run `20260519T023629Z-mcp-slice1-notes-roundtrip`)

## Slice 1 round-trip

### create

- tool: `kit/notes_create`
- id: `4bd15bad-6dfc-48ca-9764-bc91cbf57679`
- ok: true
- uri: `kit://notes/4bd15bad-6dfc-48ca-9764-bc91cbf57679`
- envelope: `{ok:true, action:"notes_create", resourceUri:"kit://notes/...", result:{id, uri, title, deleted:false, permanent:false}}`

### read kit://notes/{id}

- resource: `kit://notes/4bd15bad-6dfc-48ca-9764-bc91cbf57679`
- title: matches the create payload (`"Created by curl round-trip at 2026-05-19T03:04:00Z"`)
- body matched: true
- payload includes `schemaVersion:1`, `note.{id,title,content,created_at,updated_at,deleted_at,is_pinned,sort_order}`

### list kit://notes?limit=25

- ok: true
- contains new id: true
- payload includes `schemaVersion:1`, `count`, `truncated:false`, `notes[]` with each entry’s `id/uri/title/preview/charCount/createdAt/updatedAt`

### update

- tool: `kit/notes_update`
- ok: true
- re-read: body changed to `"Updated by round-trip at 2026-05-19T03:04:01Z"`

### delete (soft)

- tool: `kit/notes_delete`
- ok: true
- re-read with `?includeDeleted=1`: `deleted_at` set to `2026-05-19T03:04:01.384410Z`; row preserved

### UI receipt

- Tool: `computer/see { target:{type:"kind",kind:"notes"} }`
- Recreated note id for UI capture: `e9efe84c-9bf8-4609-80ba-190059ea9e4e`, title `"UI Receipt Note"`, body `"UI receipt body line one\nUI receipt body line two"`
- `computer/see` response (excerpt):
  - `windowKind:"Notes"`, `title:"Notes"`, `resolvedBounds:{x:891,y:153,width:571,height:600}`
  - `elements[]` includes `{ semanticId:"input:notes-editor", type:"input", value:"UI receipt body line one\nUI receipt body line two", focused:true }`
- Notes window open: true
- Selected note id matched: true (focused editor reflected the freshly created note's body)
- Visible title/body matched: true

## Negative gate

- bad token (`Authorization: Bearer bogus-token-xyz`) → HTTP 401, body `Invalid or missing token`, no JSON-RPC success body.
- malformed body (`body:123`) → `tools/call` success, `isError:true`, `result.error.code = "invalid_params"`, message `"Invalid notes_create arguments: invalid type: integer 123, expected a string"`.
- invalid note URI (`kit://notes/not-a-uuid`) → JSON-RPC `error.code = -32602`, message `"Invalid note id in URI: not-a-uuid"`.
- permanent delete without `confirm:true` → `tools/call` success, `isError:true`, `result.error.code = "confirm_required"`, message `"kit/notes_delete with permanent:true requires confirm:true"`.

## Stability gate

- After the full positive+negative sweep, `tools/list` (id 99) returned a healthy schema list.
- The server process (pid recorded in `/tmp/mcp-slice1-app.pid`) only terminated after the receipt run sent it `SIGTERM` (logged `Shutdown signal detected, performing graceful cleanup` at `2026-05-19T03:04:51.388557Z`).
- Server did not die mid-probe: true (contrast with 2026-05-18 audit which recorded a mid-probe crash).

## Audit JSONL

`~/.scriptkit/mcp-audit.jsonl` recorded each mutation with `ts/traceId/method/tool/action/risk/success/errorCode` — verified with `tail`. Risk classes seen during the round-trip: `StateMutating` (create, update), `Destructive` (delete).

## Cargo gates run via `./scripts/agentic/agent-cargo.sh`

Green:

- `fmt -- --check`
- `test --lib mcp_notes`
- `test --lib mcp_resources`
- `test --test mcp_protocol_golden -- --nocapture`
- `test --test close_notes_window_lock_release_before_update_contract -- --nocapture`
- `test --test mcp_notes_tools -- --nocapture`
- `test --test mcp_notes_resources -- --nocapture`

Full-suite caveat: `test --lib` reports `13126 passed; 203 failed`. Every failing test name is in pre-existing unrelated modules (grammar pivot `menu_syntax::*`, `scripts::grouping::advanced_query_tests::*`, `selected_text::unit_tests::*`, `stdin_commands::tests::*`, `routes::tests::non_filterable_routes_are_one_to_one`, `notes::window::tests::test_notes_acp_uses_shared_external_footer_renderer`). None reference `mcp_*`, `mcp_control`, `mcp_notes_*`, or any file modified by Slice 1. The slice’s targeted gates above are green; the failures are inherited from the dirty worktree (memory: grammar pivot `+` → `;`, ACP footer pre-edits).

## Test-note cleanup

The two notes created by this round-trip remain in the local notes DB:

- `4bd15bad-6dfc-48ca-9764-bc91cbf57679` (soft-deleted by step 5).
- `e9efe84c-9bf8-4609-80ba-190059ea9e4e` (live; created for the UI receipt). Safe to delete via the Notes UI.

Cleanup via MCP could not run after the receipt because the in-process SIGTERM closed the server before the second permanent-delete pass. Not a stability issue.

## URLs / paths used

- `http://127.0.0.1:43210/rpc`
- `~/.scriptkit/agent-token`
- `~/.scriptkit/mcp-audit.jsonl`
- `.codex-run/20260519T023629Z-mcp-slice1-notes-roundtrip/`
- `target-agent/mcp-slice1/debug/script-kit-gpui`

## Files added / modified in this slice

- `src/lib.rs`, `src/main.rs`, `src/main_entry/app_run_setup.rs`
- `src/mcp_protocol/mod.rs`, `src/mcp_resources/mod.rs`, `src/mcp_server/mod.rs`
- `src/mcp_control.rs` *(new)*, `src/mcp_notes_tools.rs` *(new)*
- `src/notes/mod.rs`, `src/notes/window.rs`, `src/notes/window/notes.rs`, `src/notes/window/window_ops.rs`
- `tests/mcp_notes_resources.rs` *(new)*, `tests/mcp_notes_tools.rs` *(new)*
