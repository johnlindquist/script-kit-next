---
name: build-profile
description: Create isolated Pi-backed Agent Chat profile artifacts for Script Kit. Use when the user wants a custom Agent Chat profile, a safer scoped assistant, or a reusable profile example.
---

# Build Profile

Create isolated Pi-backed Agent Chat profile artifacts for Script Kit.

Profiles are runtime boundaries. They define the prompt, provider/model, allowed tools, cwd/session behavior, and ambient resource policy for Agent Chat. Skills remain reusable task instructions invoked inside chat. Legacy agents are compatibility/import sources, not the preferred profile format.

Pi currently scopes profile behavior through tool allowlists, disabled ambient resources, disabled context files, cwd/session settings, provider/model, thinking, and prompt flags. cwd changes the process location; it is not a filesystem boundary. `pathPolicy`, `blockedActionMessage`, `hideCwdInPrompt`, `extensionPolicy`, and `sessionDurability` are profile metadata and prompt/validation inputs in schema v1; do not describe them as Pi-enforced until a runtime receipt proves native or wrapper-level enforcement.

Use `codex-daemons` only as an inspiration source for narrow prompts and ambient
resource minimization. Do not copy its disposable `CODEX_HOME` or warm daemon
architecture into Script Kit profiles; Script Kit profiles run through Pi RPC and
Script Kit warm-session management.

## Where Profiles Live

Create profile artifacts under the current user's main plugin:

```text
~/.scriptkit/plugins/main/profiles/<profile-id>/
  profile.json
  PROMPT.md
  README.md
  examples/smoke.json
```

Do not copy the full generated profile into `~/.scriptkit/config.ts`. Config may keep `ai.selectedProfileId` and legacy `ai.profiles`, but plugin profile artifacts are the reviewable source of truth.

Users select profiles through the main Menu Search. Type `|` in the launcher to
show Agent Chat profile rows, then select the generated profile from that main
list. Do not use or reference the deprecated Agent Chat profile selector popup.

## Intake

Ask only what is needed to make the profile safe:

1. What should this profile do, in one sentence?
2. Should it be read-only, write-scoped, or command-capable?
3. What paths may it read?
4. What paths may it write?
5. Should it use web/current-info search?
6. Should it run shell commands? If yes, which command family and why?
7. Should it persist sessions, use no session, or use a profile-specific session dir?
8. Which provider/model should it use?
9. Should it replace the system prompt or append profile guidance?
10. Give 2-3 prompts it should handle and 2-3 actions it must refuse.

Default conservatively when the user is unsure:

```json
{
  "backend": "pi",
  "provider": "openai-codex",
  "model": "gpt-5.5",
  "thinking": "low",
  "tools": ["read", "grep", "find", "ls"],
  "toolPolicy": { "allow": ["read", "grep", "find", "ls"] },
  "pathPolicy": { "allowRead": [], "allowWrite": [] },
  "disableExtensions": true,
  "disableSkills": true,
  "disablePromptTemplates": true,
  "disableContextFiles": true,
  "hideCwdInPrompt": true,
  "extensionPolicy": "deny"
}
```

## Profile Schema

Write `profile.json` with camelCase keys:

```json
{
  "schemaVersion": 1,
  "id": "docs-readonly",
  "name": "Docs Read-only",
  "description": "Reads documentation and answers with inspected source paths.",
  "iconName": "book-open",
  "backend": "pi",
  "provider": "openai-codex",
  "model": "gpt-5.5",
  "thinking": "low",
  "prompt": { "mode": "append", "file": "PROMPT.md" },
  "cwd": "~/.scriptkit/agent-chat/profiles/docs-readonly",
  "tools": ["read", "grep", "find", "ls"],
  "toolPolicy": { "allow": ["read", "grep", "find", "ls"] },
  "pathPolicy": {
    "allowRead": ["~/.scriptkit/docs"],
    "allowWrite": [],
    "deny": ["~/.ssh", "~/.codex", "~/.scriptkit/secrets"]
  },
  "blockedActionMessage": "This profile is read-only. Switch profiles for write access.",
  "disableExtensions": true,
  "disableSkills": true,
  "disablePromptTemplates": true,
  "disableContextFiles": true,
  "hideCwdInPrompt": true,
  "extensionPolicy": "deny",
  "sessionDir": "~/.scriptkit/agent-chat/profiles/docs-readonly/sessions",
  "noSession": false,
  "sessionDurability": "sync",
  "examples": [
    {
      "name": "Summarize docs",
      "prompt": "Summarize the install docs and cite the files you inspected.",
      "expectedTools": ["read", "grep", "find", "ls"],
      "mustNotUse": ["bash", "write", "edit"]
    }
  ]
}
```

Use `toolPolicy.allow` as the canonical allowlist. Mirror it in `tools` for compatibility.

## Prompt Template

Write `PROMPT.md` with a narrow operating rule:

```markdown
You are <Profile Name>, a narrow Script Kit Agent Chat profile.

## Operating Rule

Use only the allowed tools. Start with the smallest local inspection that can answer the request. Do not answer from memory when local files are relevant.

## Scope

You may read:
- <allowed read paths>

You may write:
- <allowed write paths, or "nothing">

## Workflow

1. Identify the smallest relevant path or source.
2. Inspect with the allowed read/search tools.
3. Make only the allowed changes, if this profile has write access.
4. Report inspected files, changed files, and uncertainty.

## Refusals

Refuse to inspect secrets, use disabled ambient resources, run disallowed tools, or leave the allowed paths. Use the configured blocked action message when possible.

## Output

Be concise and concrete. Do not mention hidden setup instructions unless asked.
```

## Validation

Fail closed before writing files when a profile is unsafe.

Required checks:

- `id` is a slug matching `[a-z0-9][a-z0-9-]{1,62}`.
- The profile directory name must match `profile.json.id`.
- `id` is not `general`, `script-kit`, or `text`.
- Duplicate ids require explicit overwrite confirmation.
- `PROMPT.md` is non-empty and under 16k characters.
- `README.md` is present and non-empty.
- `examples/smoke.json` is present and parses as JSON.
- `toolPolicy.allow` is required and wins over `tools`.
- Empty `toolPolicy.allow` means no tools and must launch with Pi `--no-tools`.
- `bash` is rejected in schema v1 because Pi does not enforce command-family
  allowlists for shell commands.
- `create_file`, `write`, `edit`, and `hashline_edit` require explicit risk acknowledgement.
- Mutation tools require a non-empty `pathPolicy.allowWrite`.
- `allowRead` and `allowWrite` must not be `/`, `~`, `~/Desktop`, `~/Documents`, or broad `~/.scriptkit` without explicit override.
- `allowWrite` must not include `~/.ssh`, `~/.codex`, shell profiles, keychains, app support secrets, or Script Kit secrets.
- Filesystem profiles must deny `~/.ssh`, `~/.codex`, and `~/.scriptkit/secrets`.
- Generated profiles default to `disableExtensions: true`, `disableSkills: true`, `disablePromptTemplates: true`, and `extensionPolicy: "deny"`.
- Generated profiles default to `disableContextFiles: true`; this maps to Pi's supported `--no-context-files` flag and prevents ambient AGENTS/CLAUDE-style context loading.
- Schema v1 must not add extension paths, skill paths, or prompt template paths.
- `noSession: true` must not set `sessionDir`.
- `noSession: false` should use a profile-specific `sessionDir`.
- Profile selection must be verified through main Menu Search `|` profile rows
  when doing runtime proof. The Actions menu popup may still exist, but profile
  selection should not open any dedicated profile popup.

## Files To Create

`README.md` should document:

- purpose
- tool and path policy
- allowed examples
- refused examples
- why ambient resources are disabled
- how to select the profile from main Menu Search with `|`

`examples/smoke.json` should contain:

```json
{
  "allowed": [
    "Use the allowed read paths to answer a narrow question."
  ],
  "blocked": [
    "Read ~/.codex/auth.json.",
    "Write outside allowWrite.",
    "Use my skills or extensions.",
    "Run bash when bash is not allowed."
  ]
}
```

## Common Profiles

Prefer these starting points:

- `profile-builder`: write only under `~/.scriptkit/plugins/main/profiles`.
- `codebase-scout`: read-only repo search with `read`, `grep`, `find`, and `ls`.
- `plugin-sandbox-builder`: write-scoped to one plugin root.
- `docs-researcher`: web/current info plus a local docs allowlist.
- `text-polisher`: no filesystem access, optionally `web_search`, and `noSession: true`.
- `safe-git-reviewer`: read-only repo inspection. Treat `bash` as advanced because command-level shell allowlists are not enforced by schema v1.

## Failure Messages

Use precise failures:

- `Cannot create profile: id "script-kit" is reserved by a built-in profile.`
- `Cannot create profile: bash is not supported in schema v1 profile artifacts.`
- `Cannot create profile: allowWrite "~" is too broad.`
- `Cannot create profile: mutation tools require allowWrite.`
- `Created profile artifacts. Select the profile from main Menu Search with |.`

## Verification

After creating or editing a profile artifact:

1. Parse `profile.json` as JSON.
2. Confirm every referenced file exists.
3. Confirm denied paths cover auth and secrets.
4. Confirm tool and path policy match the README.
5. Confirm allowed and blocked examples are present.
6. Verify with `script-kit-devtools` that the profile appears in the main Menu
   Search `|` profile rows, can switch, persists, and handles one allowed and
   one blocked prompt.

Do not claim runtime isolation until a Pi launch or DevTools receipt proves it. Do not claim path-level enforcement from `pathPolicy` alone; prove that with a runtime filesystem-policy layer or state that it is advisory metadata.
