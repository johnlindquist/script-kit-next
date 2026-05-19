# Parallel script-kit-devtools proof (2026-05-18)

## Verdict

**Succeeded** — three DevTools workers ran concurrently against the shared `target/debug/script-kit-gpui` binary (current tree), with isolated session names, session dirs, agent IDs, and app PIDs. Sessions were stopped and verified `not_found` / `alive:false`.

Per-commit **runtime** builds were **not** executed (Option B skipped: three full agent-cargo builds would contend and add minutes). Commits are tied to workers via bug receipt text plus static `git diff-tree` metadata under the artifact dir.

## Strategy

| Choice | Rationale |
| --- | --- |
| **Option A** (parallel sessions, one binary) | `session.sh` hardcodes `target/debug/script-kit-gpui`; binary already present; proves runtime parallelism without rebuild |
| **Option B** (worktree + per-commit build) | Deferred — would need 3× `agent-cargo.sh build` and binary promotion; not required to prove session isolation |
| **Cargo** | Not invoked (existing debug binary used) |

## Commits referenced (non-merge, recent)

| Worker | SHA | Subject |
| --- | --- | --- |
| 1 | `f4be75aec` | MCP: notes round-trip + git-diff bound (slice 1 of full-control plane) |
| 2 | `1e487f2d3` | Prevent the ACP @ inline portal popup from getting stuck open |
| 3 | `4338cac00` | Pin Notes delete modal to the Notes window |

Static commit snapshots: `.agent-reports/parallel-dt-proof-20260518-211400/commit-checks/*.json` (none touch `scripts/devtools/`).

## Overlap evidence

| Event | UTC timestamp |
| --- | --- |
| Orchestrator start | `2026-05-19T03:14:00Z` |
| 1s overlap checkpoint (all workers still running) | `2026-05-19T03:14:01Z` |
| All workers finished | `2026-05-19T03:14:03Z` |

All three workers logged `=== worker N start 2026-05-19T03:14:00Z` before any worker log shows `stop exit 0`, so inspect + targets ran in parallel for ~3s.

## Workers

| Worker | Session | `SCRIPT_KIT_AGENT_ID` | `SCRIPT_KIT_SESSION_DIR` | Shell PID | App PID (on stop) |
| --- | --- | --- | --- | --- | --- |
| 1 | `dt-proof-a` | `dt-proof-1` | `/tmp/sk-agentic-sessions-dt-proof-a` | 71083 | 71215 |
| 2 | `dt-proof-b` | `dt-proof-2` | `/tmp/sk-agentic-sessions-dt-proof-b` | 71084 | 71214 |
| 3 | `dt-proof-c` | `dt-proof-3` | `/tmp/sk-agentic-sessions-dt-proof-c` | 71086 | 71216 |

Commands (each worker):

1. `bun scripts/devtools/inspect.ts --session <name> --start --show --main --bug "parallel proof worker N commit <sha>"`
2. `bun scripts/devtools/targets.ts list --session <name> --show`
3. `bash scripts/agentic/session.sh stop <name>`

Exit codes: **0, 0, 0** (see `orchestrator-final.json`).

## Receipts (key fields)

| Worker | JSON receipt | Session in receipt | Target | Surface | Classification | Errors |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | `worker-1.json` | `dt-proof-a` | `Main` / `main` | `ScriptList` | `blocked-by-missing-primitive` | `[]` |
| 2 | `worker-2.json` | `dt-proof-b` | `Main` / `main` | `ScriptList` | `blocked-by-missing-primitive` | `[]` |
| 3 | `worker-3.json` | `dt-proof-c` | `Main` / `main` | `ScriptList` | `blocked-by-missing-primitive` | `[]` |

Bug text in receipts matches worker/commit pairing (no cross-session bleed).

**Note:** `blocked-by-missing-primitive` is inspect orchestration classification for the generic `--main` proof surface, not a parallel-run failure.

## Artifact directory

`.agent-reports/parallel-dt-proof-20260518-211400/`

- `worker-{1,2,3}.json` — parsed inspect orchestrate receipts
- `worker-{1,2,3}.log` — full stdout/stderr including targets + stop
- `worker-{1,2,3}-meta.json` — timing, PIDs, commit mapping
- `pids.json` — worker shell PIDs at launch
- `cleanup-status.txt` — post-run `session.sh status` (all `not_found`)
- `orchestrator-final.json` — wall-clock span and exit codes

## Cleanup

- `session.sh stop` succeeded for all three sessions (see worker logs).
- Post-run status: `dt-proof-a|b|c` → `"status":"not_found","alive":false`.
- No worktrees created; nothing to remove under `.worktrees/`.

## Blockers / gaps

- **Per-commit binary proof** not run (time/build contention); parallel **session** proof is complete.
- Initial orchestrator `jq` envelope failed harmlessly; metadata repaired in this report pass.
- Worker JSON extract script initially grabbed wrong object; receipts re-parsed from logs into `worker-N.json`.
