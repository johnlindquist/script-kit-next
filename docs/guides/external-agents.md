# Use Script Kit from your own agent

Stay in your agent's current conversation. Kit can save a quiet Markdown note, run a particular version of a trusted personal script, or share one selected passage with a named local Herdr agent. No mouse, clipboard fallback, or second chat is involved.

## Give an agent just the operations it needs

From your operator shell, using the `scriptkit` shim shipped by this build (`~/.scriptkit/bin/scriptkit` if not on PATH):

```sh
mkdir -p -m 700 ~/.kit-agent-credentials
scriptkit grant-agent writer ~/.kit-agent-credentials/writer notes:quiet
```

Keep the credential in a private directory outside your scripts, themes, notes, and backups. A `0700` script directory is still a source directory: another tool can read a script when you ask it to inspect the command. Use a new filename for each grant. Kit refuses to put a bearer in its content, registration, or operator-token paths, including a directory reached through an alias. A plugin’s script folder is still Kit content when it points into a separate project. Use a different folder, even when supplying the project’s own path. Kit also refuses a credential that an existing `.ts` or `.js` script link would expose. If the name or output file is taken, the earlier grant and file are left alone.

Granting checks folder identities and links, not script contents. If Kit cannot inspect the catalog or finds more than 65,536 plugin and script entries, it stops without issuing a credential. Keep credentials out of later script links and shared repositories, too.

Give **that agent**, not every shell, `SCRIPT_KIT_AGENT_CREDENTIAL=$HOME/.kit-agent-credentials/writer`. A narrowly credentialed agent reads only its own token and Kit's loopback address in `server-location.json`, not the operator's bearer in `server.json`. The CLI needs Bun on PATH; live calls need Kit running. Discovery lists the numeric loopback address Kit listens on. Use it only on a trusted local machine: ordinary HTTP MCP does not authenticate the server to you. To use a different loopback server, set `SCRIPT_KIT_MCP_ENDPOINT` and supply the scoped credential issued for that server. With no explicit credential, `scriptkit mcp` uses the token in `server.json` only for the endpoint recorded alongside it. A different port, hostname, or scheme is a different destination. If Kit reports invalid JSON in its connection file, restart Kit to recreate it. Do not paste the file or its token into a chat, issue, or repository.

Install the small opt-in skill in a skills directory your agent already discovers, for example `scriptkit install-skill "$HOME/.agents/skills"` if your harness uses that directory. The parent must exist. Kit will not replace an existing skill. Alternatively tell the agent to run `scriptkit --skill`. Reading the manual does not start Kit. `scriptkit agent discover` reports the caller's tools, resources and reviewed command revisions; it does not read the desktop. The same operations are available to authenticated HTTP MCP clients on `/rpc`.

Ask: **“Save your findings in a Kit note. Don't interrupt me.”** The agent chooses a UUID, pipes Markdown to `scriptkit agent note save <uuid>`, and verifies the body with `scriptkit agent note status <uuid>`. The same bearer and UUID address the same note across restarts. An edit or conflict isn't overwritten. If Kit cannot load the exact same words after a restart (for example, leading blank lines removed by the Notes file format), it refuses the save before writing rather than claiming a literal copy. This grant can read its addressed notes, not search your notebook or install standing `#instructions` notes.

To revoke a bearer, remove its exact registration `~/.scriptkit/agents/<name>.json` (under `SK_PATH` when configured), then remove its credential file. Revocation denies subsequent calls; it can't recall data already shared or stop work already dispatched. The operator credential intentionally retains broad compatibility access. A scoped credential is **not an OS sandbox** against a process with unrestricted shell access as your user.

If an older agent credential was exposed as a script, revoke that grant and issue a fresh credential in a private, unshared directory. If the same file also served as Kit’s operator token, removing the named grant alone is not enough: quit every Kit instance for that workspace, rotate the operator token, then restart Kit and reconnect trusted clients.

## Share a selection with Herdr

1. `scriptkit grant-agent workshop ~/.kit-agent-credentials/workshop handoff:receive`. Give that credential to the **workshop agent's own local Herdr terminal**. There, run `scriptkit agent available workshop`. The name must match the operator's grant. The terminal must report idle to accept a notification. `scriptkit agent away` closes pending shares. An unchanged registration remains available up to eight hours.
2. In Kit select the **Selected text** subject (Space stages it). **Read the full selection** shows exactly what's going to be shared. Choose **Share selection with a named agent…**, then the recipient and terminal ID. The line counts down for eight seconds; press ⌘Z on an empty input if you change your mind. Only the frozen passage is shared.
3. The addressed agent gets a notification containing a request ID, **not the passage or a credential**. Only the registered recipient can take it and reply. No terminal screen is scraped. A busy, replaced, missing, or unrecognized terminal is not substituted.
4. The reply doesn't take focus. Open **Agent Handoffs → Edit reply** when ready. The Markdown is editable, not executed or pasted. Esc or ⌘S keeps your edit in this session and returns to the handoffs list. A pending share can be closed there too; text already read cannot be recalled. Handoffs and draft edits are in Kit's memory, addressable for one hour or until Kit exits. Save important replies before then. Editing a reply in Kit does not send those edits back.

A sender with a separate explicit `handoff:send:workshop` grant may send **its own supplied text** without taking over Kit. `scriptkit agent targets` returns current registrations; pipe the text to `scriptkit agent send <target-uuid> <fresh-request-uuid>`. Use `scriptkit agent status <request-uuid>` or `cancel <request-uuid>`. Check status after any uncertain outcome; don't send a second notification on a timeout. That grant cannot read the user's selection. Each sender can hold up to 12 one-hour receipts, including refusals, and external senders cannot use the launcher's reserved slots.

The adapter speaks generation 1 on an existing **local Unix socket**. It rechecks core ID, terminal address, creation time, PID and idle status; Herdr checks `if: idle` at send. It does not start an agent or core. For a custom Herdr socket set `SCRIPT_KIT_HERDR_SOCKET` in Kit and `HC_SOCKET` for the registered agent to the same local endpoint. Remote terminals are not supported.

## Approve a personal command, not a script runner

Start with the shipped `~/.scriptkit/share/external-agent/prepare-handout.ts`. Copy it (without replacing an existing file) into the main plugin's `scripts/` directory. It is an ordinary launcher script: run **Prepare Handout** interactively, or provide typed input to it as an agent.

```sh
scriptkit inspect-command prepare-handout
# After reviewing the entire program and the effect, use the exact reported SHA-256:
scriptkit grant-agent handouts ~/.kit-agent-credentials/handouts command:prepare-handout:<sha256>
```

With the handouts credential, `scriptkit agent discover` shows the digest, declared effect and input/output schemas. Pipe `{"title":"Session notes","body":"Bring a notebook."}` to `scriptkit agent run prepare-handout <sha256>`. Success includes the output file path; verify the file for this command. A changed script is reported as `revision_changed` and isn't dispatched. Review and issue a new grant for the new version. No wildcard grants for this route.

Reviewed programs must be self-contained main-plugin `.ts` files with a lowercase slug, `// Agent effect: ...`, and `schema` or `defineSchema` with typed output. In headless mode the SDK adapter provides `input`, `output`, and `defineSchema`; interactive prompt calls are not supported. Bun executes the exact inspected bytes in a private temporary directory, without automatic package installation, caller-supplied environment/arguments, or inherited MCP credentials. Input and output are validated and bounded (64 KiB each; 10 seconds). Trusted script code can still access OS APIs, load external code and produce external effects. A source digest pins the entry file, **not the contents of files it reads or imports**. Review those dependencies or keep the command self-contained. A timeout, nonzero exit, or invalid output does not reverse side effects.

## Scope and observation

The public MCP catalog filters by the bearer. Metadata is not permission to execute. Private resources have domain grants (such as `notes:read`, `clipboard:read`, `context:read`); requesting context pixels additionally requires `computer:capture`. Neither the full nor minimal MCP context profile captures pixels unless explicitly requested. `computer/*` is still observation-only. `kit/state` and `kit://state` use the live app bridge, or report unavailable; unavailable is not “window hidden.” The separate filename operation still requires an explicit `desktop:fill` grant.
