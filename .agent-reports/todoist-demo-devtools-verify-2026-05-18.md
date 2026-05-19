# Todoist Demo — DevTools Verification (2026-05-18)

## Session

- **App session:** `todoist-verify` (`bash scripts/agentic/session.sh start todoist-verify`)
- **Script:** `~/.scriptkit/plugins/main/scripts/todoist-demo.ts`
- **Launch:** `{"type":"run","path":"…/todoist-demo.ts"}` — `parseOutcome: parsed`

## Green proof (runtime)

### Main menu (`ArgPrompt` id `1`)

Placeholder **Todoist Demo** with **11 choices**:

| Choice | Description (from app log) |
|--------|---------------------------|
| Today | 1 tasks due today or overdue |
| Upcoming | Next 7 days · 2 tasks |
| Inbox | 1 tasks |
| All tasks | 3 open |
| Add task | Title, due date, priority, project, labels |
| Search | Filter open tasks |
| Projects | 3 projects |
| Labels | 2 labels |
| Sync ;todo captures | Import ~/.scriptkit/menu-syntax/todos.jsonl |
| Dashboard | Stats overview |
| Quit | Exit |

Surface: `arg_prompt`, `filtered_choices=11`.

### Today view (`ArgPrompt` id `2`)

After `simulateKey` Enter on main menu (Today selected):

- Placeholder: **Today**
- Task row: **Welcome to Todoist Demo** — `P4 · Today · Inbox`

### Non-interactive

`SK_VERIFY=1 bun ~/.scriptkit/plugins/main/scripts/todoist-demo.ts` → `{"ok":true,"parseDue":true,"tasks":1}`

## DevTools primitive stack

| Primitive | Result | Notes |
|-----------|--------|-------|
| `session.sh send` run / show / simulateKey | **ok** | Protocol commands parsed |
| `devtools.inspect --main` | **blocked-by-timeout** | `getState`/`getElements` RPC did not land in `responses.ndjson` during active script prompt |
| `devtools.elements.snapshot` | **blocked-by-timeout** | Same RPC capture gap |
| `devtools.act.ts key enter` | **blocked** (hung) | Depends on RPC responses |
| `devtools.coverage.ts --surface main` | **ok** | Static coverage map |
| `devtools.investigate.ts --surface main` | **ok** | Proof plan emitted |

**Workaround used:** Correlated `Sending state result` and `ArgPrompt` choice payloads in `/private/tmp/sk-agentic-sessions/todoist-verify/app.log` (app emits `stateResult`; session RPC reader did not record them while Bun script owned the prompt).

## Classification

**reproduced** — Todoist Demo runs in the real app with full main-menu and Today task-list prompts. DevTools CLI RPC layer is **blocked-by-missing-primitive** for live `getState`/`getElements` receipts during active SDK script prompts (not a Todoist Demo defect).

## Cleanup

```bash
bash scripts/agentic/session.sh stop todoist-verify
```
