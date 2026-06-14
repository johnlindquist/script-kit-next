# Script Kit Project Imps

Project imps are feature-bound Codex specialists for this repository. They use
the copied `codex-imps` runtime so warm daemons, hot reload, local lesson
overlays, failure classification, pruning, and promotion workflows stay real.

Every feature imp is configured for `gpt-5.5` with `medium` reasoning. If that
model is unavailable, the imp should fail visibly instead of silently
downgrading.

## Commands

```bash
cd .agents/imps
npm install
bun imps/project-imps list
bun imps/project-imp --which "fix @file attachment in Agent Chat"
bun imps/project-imp "fix @file attachment in Agent Chat"
bun imps/imp-sk-agent-chat "fix @file attachment in Agent Chat"
```

`project-imp` routes by registry triggers and owner paths. Direct `imp-sk-*`
commands skip routing and run the named specialist.

## Self-Improvement

Local lessons live under `lessons/local/` and receipts under `receipts/`; both
are git-ignored. Lessons are folded into the imp developer instructions and are
included in the warm-daemon fingerprint, so the next run restarts with new
lessons active.

Promotion is manual and reviewed:

- repeated command or workflow failure -> permanent imp prompt
- cross-cutting repo rule -> `AGENTS.md`
- user-visible regression -> focused test or runtime probe
- durable product/domain assumption -> owning docs or `.notes`
- one-off local failure -> stays local until pruned

Tracked examples live in `evals/cases/`.
