---
name: isolated-devtools-session
description: >-
  Canonical bootstrap for isolated Script Kit GPUI DevTools sessions: one front
  door script (classify, verify-script, start, prove, cleanup), repo-level helpers
  under scripts/agentic/, bounded timeouts, JSON stdout, progress stderr. Use when
  agents need parallel DevTools workers, isolated runtime proof, SK_VERIFY script
  gates, or dev-watch reuse without starting a second GPUI under ./dev.sh.
disable-model-invocation: true
---

# Isolated DevTools Session

**Primary entry:** `scripts/agentic/devtools-session.sh` — agents must use this front door instead of ad-hoc `session.sh start` + unbounded `agent-cargo.sh build`.

Pair with `$script-kit-devtools` for inspect/act primitives after bootstrap.

## Front Door (required)

```bash
# 1. Classify mode (script-only vs reuse-dev-watch vs isolated)
bash scripts/agentic/devtools-session.sh classify \
  --script kit-init/examples/scripts/todoist-demo.ts \
  --mode auto

# 2. Bun-only proof first (no GPUI, no cargo) when script supports SK_VERIFY
bash scripts/agentic/devtools-session.sh verify-script \
  --script kit-init/examples/scripts/todoist-demo.ts

# 3. Start isolated session (JSON on stdout; progress on stderr)
SESSION="dt-agent-$(date +%s)"
bash scripts/agentic/devtools-session.sh start \
  --session "$SESSION" \
  --mode isolated \
  --build never \
  --ready-timeout-sec 60 \
  --prove \
  --cleanup-on-fail

# 4. Prove / cleanup
bash scripts/agentic/devtools-session.sh prove --session "$SESSION"
bash scripts/agentic/devtools-session.sh cleanup --session "$SESSION"
```

### Modes

| Mode | When | Cargo | GPUI |
| --- | --- | --- | --- |
| `script-only` | `SK_VERIFY` script proof only | Skip | Skip |
| `reuse-dev-watch` | `./dev.sh` running, healthy `dev-watch` | Skip | Attach only |
| `isolated` | Parallel agent / post-Rust proof | Optional (`--build`) through `target-agent/pools/<pool>` | New session from staged binary |

**Rules:**

- Never start `isolated` while `./dev.sh` cargo-watch is running (exit `11`).
- Never accept `session.sh start` with `ready:false` as done — the front door runs `wait-session-ready.sh` (60s visible gate; internal start timeout 5s).
- Never run unbounded `agent-cargo.sh build` — use `build-isolated-binary.sh` (120s default) or `--build auto|always|never` on `start`.
- `agent-cargo.sh` defaults to the bounded `target-agent/pools/agent-debug` pool with a visible lock; use `SCRIPT_KIT_AGENT_TARGET_MODE=exclusive` only when a task truly needs its own cache.
- Isolated builds stage `target-agent/runtime/<session>/script-kit-gpui` and sessions launch that staged binary via `SCRIPT_KIT_GPUI_BINARY`; agents must not promote into `target/debug/script-kit-gpui`.

### Reuse dev-watch

```bash
bash scripts/agentic/devtools-session.sh start \
  --mode reuse-dev-watch \
  --session dev-watch \
  --build never
```

## Thin Skill Wrappers

Scripts under `.agents/skills/isolated-devtools-session/scripts/` are `exec` wrappers to `scripts/agentic/`:

- `preflight-isolated.sh`
- `wait-session-ready.sh`
- `build-isolated-binary.sh`
- `start-isolated.sh`

Do not duplicate logic in the skill folder.

## Verification

```bash
bash scripts/agentic/verify-devtools-session.sh
./scripts/agentic/agent-cargo.sh test devtools_session_contract --
```

## Environment

| Variable | Purpose |
| --- | --- |
| `SCRIPT_KIT_SESSION_DIR` | FIFO/state root (default `/tmp/sk-agentic-sessions`) |
| `SCRIPT_KIT_AGENT_ID` | Agent identity for logs and explicit exclusive targets |
| `SCRIPT_KIT_CARGO_TARGET_POOL` | Named shared agent target pool (default `agent-debug`) |
| `SCRIPT_KIT_AGENT_TARGET_MODE` | `pool` by default; set `exclusive` only for per-agent caches |
| `SCRIPT_KIT_GPUI_BINARY` | Staged binary path for isolated session launch |
| `SCRIPT_KIT_TEST_NOTES_DB_PATH` | Sandbox notes DB (`--notes-sandbox` on start) |

## Anti-Patterns

- `./dev.sh` + isolated `session.sh start` (second GPUI → empty `app.log`, RPC hangs).
- Bare `cargo build` while `./dev.sh` holds `target/.cargo-lock`.
- Full GPUI bootstrap before `verify-script` for Bun-only demos.
- `SCRIPT_KIT_SESSION_READY_TIMEOUT_MS=60000` inside start **and** a 60s wait loop (double silence) — use the front door defaults.

## Pairing

- **`$script-kit-devtools`** — investigation after bootstrap.
- **`$protocol-automation`** — RPC semantics.
- **`$agentic-testing`** — regression recipes after isolation.
