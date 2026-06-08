# Profile Builder Plan

This plan is based on the Oracle browser sessions `profile-builder-daemons` and
`profile-builder-pi-review` from June 1, 2026, plus source and DevTools receipts
from the Script Kit GPUI worktree.

## Core Model

- A profile is a Pi Agent Chat runtime boundary: prompt, provider/model, tools, cwd/session policy, and ambient-resource policy.
- A skill is a reusable task recipe invoked inside Agent Chat.
- A legacy agent is a compatibility/import source, not a competing runtime profile concept.
- Profile selection is a main Menu Search operation. The only remaining popup
  selector path for this family of interactions is the Actions menu popup; Agent
  Chat profiles must use the Spine/Menu Search `|` rows, like other launcher
  sigils.

The first shippable path is plugin-owned profile artifacts plus the `/build-profile` skill. Keep `~/.scriptkit/config.ts` for selected profile and compatibility overrides, but do not copy generated profile bodies into config.

Current Pi backend reality as of `pi 0.75.5`: Script Kit can harden a profile with `--tools`/`--no-tools`, `--no-extensions`, `--no-skills`, `--no-prompt-templates`, `--no-context-files`, `--session-dir`/`--no-session`, cwd, provider/model, thinking, and prompt flags. Pi does not currently accept `--path-policy-json`, `--blocked-action-message`, `--profile-id`, `--profile-name`, `--hide-cwd-in-prompt`, `--extension-policy`, or `--session-durability`. Therefore `pathPolicy`, blocked messages, and hide-cwd settings are profile metadata and prompt/validation inputs until Pi grows native enforcement or Script Kit adds a wrapper/tool layer.

## Codex Daemons Reference

`~/dev/codex-daemons` is a useful isolation reference, but it is not the runtime
model Script Kit should copy. That repo isolates Codex SDK/CLI profiles by
creating a disposable `CODEX_HOME`, symlinking only `auth.json`, disabling apps,
skills, plugins, hooks, memories, tool discovery, web search, and environment
context, and then running either a cold SDK turn or a warm `codex app-server`
daemon. It also uses a consistent single-purpose prompt shape: operating rule,
command map, workflow, command rules, and output.

Translate those ideas to Pi-backed Script Kit profiles this way:

- Borrow the single-purpose prompt shape and fail-closed examples.
- Borrow the ambient-resource minimization principle: `--no-extensions`,
  `--no-skills`, `--no-prompt-templates`, and `--no-context-files`.
- Borrow profile-specific cwd/session/warm-key separation.
- Do not copy the daemon process architecture. Script Kit already has Pi RPC
  launch and warm-session management.
- Do not claim `CODEX_HOME`-style config isolation for Pi profiles. Pi profiles
  use supported Pi flags plus prompt policy until Pi or Script Kit adds native
  filesystem/config isolation.
- Treat `pathPolicy` as an artifact review contract and prompt appendix, not as
  a Codex-daemons-style runtime sandbox.

## Artifact Location

```text
~/.scriptkit/plugins/<plugin-id>/profiles/<profile-id>/
  profile.json
  PROMPT.md
  README.md
  examples/smoke.json
```

For the initial builder skill, generate into:

```text
~/.scriptkit/plugins/main/profiles/<profile-id>/
```

## First Runtime Slice

1. Add plugin profile discovery:
   - `src/plugins/discovery.rs`: `plugin_profiles_dir(plugin_id)`
   - `src/plugins/types.rs`: `PluginProfile`
   - `src/plugins/profiles.rs`: parse, validate, and discover `profile.json`
   - `src/plugins/mod.rs`: export profile discovery

2. Extend Agent Chat profile resolution:
   - `src/ai/agent_chat/profiles.rs`: add `AgentChatProfileSource::Plugin`
   - Include plugin profiles in picker entries.
   - Resolve `plugin:<plugin-id>/<profile-id>` ids as plugin-only ids so legacy
     custom profiles cannot shadow plugin artifacts.
   - Do not allow plugin profiles to shadow `general`, `script-kit`, or `text`.
   - Route profile selection through main Menu Search / Spine profile rows, not
     the deprecated Agent Chat profile popup.

3. Keep `PiLaunchSpec` aligned with supported Pi flags:
   - Plugin artifacts should resolve into the existing launch spec fields.
   - Launch argv must not include profile metadata flags rejected by Pi.
   - Warm keys should change when prompt, tools, metadata, ambient disable flags, cwd, or session policy changes.
   - Isolated generated profiles should pass `--no-context-files` by default.
   - Because Pi does not accept `pathPolicy` as a native flag, inject a compact
     policy appendix into plugin profile prompts so advisory path/tool policy is
     visible to the model without claiming runtime filesystem enforcement.

4. Add tests:
   - `tests/pi_profile_artifact_contract.rs`
   - `tests/pi_profile_examples_contract.rs`
   - `tests/agent_chat_profile_selector_contract.rs`
   - update `tests/pi_profile_warm_key_contract.rs`
   - plugin profile parser tests under `src/plugins/profiles.rs`
   - profile picker resolution tests under `src/ai/agent_chat/profiles.rs`

## Built-In Example Set

- `profile-builder`: write only under `~/.scriptkit/plugins/main/profiles`.
- `codebase-scout`: read-only repo search with `read`, `grep`, `find`, and `ls`.
- `plugin-sandbox-builder`: write-scoped to one plugin root.
- `docs-researcher`: `web_search` plus local docs read access and a notes write path.
- `text-polisher`: no filesystem access, optionally `web_search`, and `noSession: true`.
- `safe-git-reviewer`: read-only repo inspection. Do not include `bash` in schema
  v1 profile artifacts until Pi or Script Kit supports command-level shell
  allowlists.

The shipped examples live in the managed examples plugin:

```text
~/.scriptkit/plugins/examples/profiles/profile-builder/
~/.scriptkit/plugins/examples/profiles/codebase-scout/
~/.scriptkit/plugins/examples/profiles/plugin-sandbox-builder/
~/.scriptkit/plugins/examples/profiles/text-polisher/
~/.scriptkit/plugins/examples/profiles/docs-researcher/
~/.scriptkit/plugins/examples/profiles/project-docs-maintainer/
~/.scriptkit/plugins/examples/profiles/package-manager-plan-only/
~/.scriptkit/plugins/examples/profiles/legacy-agent-import/
~/.scriptkit/plugins/examples/profiles/invalid-schema-collision/
~/.scriptkit/plugins/examples/profiles/ambient-leakage-stress/
```

`docs-researcher` and the built-in Text profile use Pi's explicit `web_search`
tool. The remaining examples avoid network tools so ambient extensions can stay
disabled without hiding undeclared capabilities.

## Plugin And Subagent Integration

Profiles and plugin subagents overlap but should not be merged:

- Plugin profiles are user-selectable Agent Chat runtimes. They own prompt, provider/model, Pi tools, cwd, session behavior, and ambient-resource isolation.
- Plugin skills are task recipes loaded into a chat. Schema v1 profile artifacts cannot enable ambient skills; a future schema can allow an explicit `skillPaths` list once the picker can show that the profile is intentionally skillful rather than isolated.
- Plugin legacy agents remain import/compatibility inputs. A `legacy-agent-import` profile iteration should read an agent markdown file and write a profile artifact that preserves purpose, model hints, and refusal examples without preserving hidden ambient assumptions.
- Subagents should be represented as either separate profiles or explicit profile handoff examples, not as implicit background workers. That keeps selection, warm keys, logs, and blocked capabilities attributable to one runtime boundary.
- Plugin manifests do not need a new top-level subagent concept for v1. Profile discovery can remain filesystem-based under `profiles/<id>/profile.json`, matching skills and scripts.
- The launcher should present plugin profile source labels as profile metadata,
  not as separate subagent rows. A plugin can ship several profiles, and each
  profile should be selectable, persisted, warmed, logged, and tested as its own
  Agent Chat runtime boundary.
- If a plugin wants a "subagent-like" workflow in v1, ship multiple profiles plus
  README/examples that describe when to switch. Avoid hidden background delegation
  until there is UI and log attribution for handoffs.

Future schema candidates:

```json
{
  "schemaVersion": 2,
  "handoffs": [
    { "profileId": "plugin:examples/codebase-scout", "label": "Inspect source read-only" }
  ],
  "skillPaths": ["skills/review-profile/SKILL.md"],
  "pathPolicyMode": "advisory|enforced"
}
```

Do not add these fields to schema v1. They need UI affordances and runtime receipts first.

## 10-Iteration Evaluation Harness

Each iteration should:

1. Build a focused `packx` bundle.
2. Ask Oracle to generate or critique one candidate profile.
3. Install it under `~/.scriptkit/plugins/main/profiles/<iteration-id>`.
4. Run profile parser and launch-spec tests.
5. Launch Script Kit.
6. Use `script-kit-devtools` to open the main Menu Search through the real user path.
7. Type `|` so the Spine/Menu Search profile rows are shown in the main list.
8. Verify the profile appears in the main list, select it through the Spine row,
   and confirm persistence. Do not open or test the deleted Agent Chat profile popup.
9. Submit one allowed prompt and one blocked prompt.
10. Collect source, UI, log, config, warm-key, and filesystem receipts.

Iterations:

1. `profile-builder`: self-profile that creates profile artifacts.
2. `codebase-scout`: read-only codebase search.
3. `plugin-sandbox-builder`: bounded plugin authoring.
4. `text-polisher`: no filesystem access.
5. `docs-researcher`: web plus local docs.
6. `project-docs-maintainer`: write only docs.
7. `package-manager-plan-only`: read package files, no installs.
8. `legacy-agent-import`: convert one legacy agent into profile artifacts.
9. `invalid-schema-collision`: prove reserved ids and invalid JSON fail closed.
10. `ambient-leakage-stress`: try skills, memories, extensions, Gmail/Slack, `~/.codex/auth.json`, writes outside scope, and disallowed bash.

Receipts per iteration:

- Oracle prompt and answer
- Packx bundle manifest or hash
- `profile.json`, `PROMPT.md`, `README.md`, `examples/smoke.json`
- artifact validation output
- `PiLaunchSpec` argv snapshot, including proof that unsupported metadata flags are omitted
- warm-key normalized material snapshot
- cargo test output
- devtools target and elements receipts
- profile switch action receipt
- Agent Chat state receipt after switch
- allowed and blocked prompt transcripts
- filesystem diff under `~/.scriptkit/plugins/main/profiles`
- app logs around profile switch and warm-session acquisition

Current partial receipt:

- `profile-builder-pi-review`: Oracle confirmed the Pi flag surface and called
  out that `pathPolicy` is not Pi-enforced. The implementation responds by
  omitting unsupported Pi metadata flags and appending profile policy into
  plugin prompts.
- `profile-main-menu-pipe-proof`: DevTools showed `|` profile rows on the main
  `ScriptList` surface and selected `choice:5:profile-builder` without opening
  the deprecated Agent Chat profile popup. Plain Enter persistence still needs a direct
  runtime receipt because the DevTools submit-like activation primitive refused
  that action for safety.
- `profile-builder-ledger-proof`: DevTools `main.inspect` resolved the real
  `ScriptList` target, `set-input --main --value '|'` produced seven Spine-owned
  profile rows, `choice:5:profile-builder` inserted the plugin profile token, and
  `key Enter --submit-intent profile-switch` left `Profile Builder` marked as the
  current Plugin/Pi profile. This also required a scoped DevTools allowlist for
  non-destructive main-menu profile switch submissions.
- `profile-builder-final-review`: Oracle completed final review and accepted
  the architecture as coherent for a first implementation path, while requiring
  fail-closed validator hardening before calling it shippable. The implemented
  response requires explicit `toolPolicy.allow`, preserves explicit empty
  allowlists as Pi `--no-tools`, rejects `bash` in schema v1, and normalizes
  trailing slashes before broad/secret path checks.
- `profile-builder-ten-profiles-proof`: DevTools proved a fresh app setup seeded
  ten plugin example profiles under `plugins/examples/profiles`, and main Menu
  Search `|` showed 13 Spine-owned profile rows: 3 built-ins plus the ten plugin
  examples. No ActionsDialog or Agent Chat profile popup was involved.
- `profile-builder-docs-switch-proof`: DevTools selected the new
  `docs-researcher` example through main Menu Search, inserted the profile
  token, submitted the profile switch with the scoped `profile-switch` intent,
  and ended with `Docs Researcher ✓` as the current Plugin/Pi profile.
- `profile-builder-prompt-transcript-profile-builder`: DevTools proved the
  selected `profile-builder` profile from main Menu Search, cleared the profile
  token, opened the compatibility `AgentChat` surface with Cmd+Enter from the
  main Menu, submitted one allowed prompt and one blocked prompt through real
  Agent Chat input, and exported the conversation markdown with
  `agent_chat_export_markdown`. The allowed prompt was
  answered in-scope, and the blocked prompt refused editing `~/.scriptkit` or
  reading `~/.codex/auth.json`.

Remaining hardening gap before claiming native isolation:

- The real allowed/blocked prompt transcripts are captured, but path refusals
  remain prompt/tool-policy behavior until there is native Pi or wrapper-level
  filesystem enforcement.

## Verification Stack

Use the cargo wrapper:

```bash
./scripts/agentic/agent-cargo.sh test --test pi_profile_artifact_contract
./scripts/agentic/agent-cargo.sh test --test pi_profile_examples_contract
./scripts/agentic/agent-cargo.sh test --test pi_profile_launch_contract
./scripts/agentic/agent-cargo.sh test --test agent_chat_profile_selector_contract
./scripts/agentic/agent-cargo.sh test --test pi_profile_warm_key_contract
./scripts/agentic/agent-cargo.sh test --test agent_chat_warm_lifecycle_contract
./scripts/agentic/agent-cargo.sh check --lib
```

Use direct DevTools primitives for runtime proof once plugin profile discovery exists:

```bash
bash scripts/agentic/session.sh start profile-builder-main-menu
bun scripts/devtools/main.ts inspect --session profile-builder-main-menu --start --show --prove-open-close-freshness --prove-early-frame-freshness
bun scripts/devtools/act.ts set-input --session profile-builder-main-menu --main --value '|' --strict
bun scripts/devtools/elements.ts snapshot --session profile-builder-main-menu --main --strict
bun scripts/devtools/act.ts select --session profile-builder-main-menu --main --semantic-id 'choice:<index>:profile-builder' --strict
bun scripts/devtools/act.ts key --session profile-builder-main-menu --main --key Enter --allow-submit --submit-intent profile-switch --allow-submit-reason 'Profile switch receipt'
```

Pass only when the profile appears in the main Menu Search profile rows, switches,
persists, launches with supported Pi args, and refuses blocked prompts. Treat
path refusals as prompt/tool-policy behavior unless a Pi-native or wrapper-level
filesystem policy receipt exists.
