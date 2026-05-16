# 021 Env Prompt / env()

This chapter maps the SDK environment-variable prompt, secret storage path, footer ownership, and privacy boundaries.


## Executive Summary


```ts
```

The SDK first returns `process.env[key]` if already set. If a custom `promptFn` is provided, it calls that function, stores the result in `process.env[key]`, and does not show the GPUI `EnvPrompt`. Only the missing-value/no-prompt-function path sends an env message to Rust.


Secret-store lookup distinguishes a missing key/file from storage failures. `SecretStoreErrorKind` classifies path, read, format, decrypt, parse, and cache failures, and EnvPrompt carries a storage-error state so corrupt or unreadable storage is not shown as a first-run missing value.


## What Users Can Do

| User capability | Entry | Result |
|---|---|---|
| Read existing env value. | `await env("MY_KEY")` with `process.env.MY_KEY` set. | Resolves immediately, no UI. |
| Use custom prompt function. | `await env("MY_KEY", async () => "...")`. | Calls function, stores result in `process.env`, no EnvPrompt. |
| Prompt for secret-like key. | `await env("API_TOKEN")`. | Opens secret prompt, masks value, stores secret persistently. |
| Auto-submit stored secret. | Secret exists and no contextual title/prompt. | Rust returns stored value without showing UI. |
| Update stored secret. | Contextual API-key prompt with title/prompt. | Shows update UI instead of auto-submit. |
| Delete stored value. | Existing stored secret UI. | Deletes local stored secret; completion behavior needs verification. |
| Cancel. | Escape. | SDK treats null as cancellation/exit. |
| Inspect safely. | `getState`, `getElements`, Tab AI context. | Secret values should be redacted/masked. |

## Core Concepts

| Concept | Meaning | Contract |
|---|---|---|
| SDK resolver | `globalThis.env`. | Existing process env and promptFn paths bypass EnvPrompt. |
| `EnvMessage` | SDK-to-Rust message. | Carries id, key, and optional secret in visible SDK. |
| `ShowEnv` | Rust prompt message. | Carries id, key, prompt, title, and secret. |
| `EnvPrompt` | Rust prompt entity. | Owns key, title/prompt copy, input, secret state, stored-secret info, focus, submit/cancel. |
| Secret detection | SDK key-name heuristic. | `secret`, `password`, `token`, `key` mark secret. |
| Secret store | `src/secrets.rs`. | Encrypted local `~/.scriptkit/secrets.age`, cached in memory, mode `0o600`, typed failure kinds. |
| Native footer | `env_prompt`. | Submit only; no launcher AI. |
| Redaction | Automation/privacy boundary. | Secret values must not leak through elements, snapshots, logs, or screenshots. |

## Entry Points

| Entry | Context | Result |
|---|---|---|
| `globalThis.env` in `scripts/kit-sdk.ts`. | Script calls `env(key, promptFn?)`. | Resolves process value, custom prompt value, or sends EnvMessage. |
| `show_api_key_prompt`. | App flow needs provider API key. | Creates EnvPrompt directly from Rust with contextual title/prompt and completion channel. |
| `collect_elements`. | Protocol inspection. | Exposes key, value/status, redacted secret values. |

## User Workflows

### Existing Process Env

A script calls `env("MY_KEY")`. The SDK checks `process.env.MY_KEY`. If it is defined and non-empty, the value resolves immediately. No Rust message is sent and no secret store lookup occurs.

### Custom Prompt Function


```ts
const value = await env("CUSTOM_KEY", async () => "value")
```

If there is no existing process value, the SDK calls the supplied function and stores its result into `process.env[CUSTOM_KEY]`. This path bypasses `EnvPrompt` entirely.

### Missing Non-secret Value


### Missing Secret Value


### Stored Secret Auto-submit

When a stored secret exists and EnvPrompt has no contextual `title` or `prompt`, Rust can auto-submit the stored secret without showing UI. This is the stored-secret fast path for ordinary `env("SCRIPT_KIT_*_API_KEY")` calls.

### Contextual API-key Prompt

Rust app flows such as inline chat setup call `show_api_key_prompt` directly. Those prompts force secret mode and include contextual title/prompt copy. If a stored secret exists, the prompt shows update/delete UI instead of auto-submitting, so the user can replace or remove the value.

## Interaction Matrix

| User intent | Entry point | UI state | Key/click | Code path | Result | Proof |
|---|---|---|---|---|---|---|
| Resolve existing env. | `env(key)`. | No UI. | SDK call. | `globalThis.env` checks `process.env[key]`. | Promise resolves value. | `scripts/kit-sdk.ts`. |
| Use custom prompt. | `env(key, promptFn)`. | PromptFn-owned. | SDK call. | SDK calls `promptFn`, sets `process.env`. | Promise resolves custom value. | `scripts/kit-sdk.ts`. |
| Block empty submit. | EnvPrompt active. | Empty input. | Submit. | Validation helper blocks empty. | Prompt remains active with error. | `src/prompts/env/helpers.rs`. |
| Auto-submit stored secret. | Stored secret, no context. | No UI or transient prompt. | Rust route. | `check_keyring_and_auto_submit`. | Stored value resolves. | `src/prompt_handler/mod.rs`, `src/prompts/env/prompt.rs`. |
| Update stored secret. | Contextual prompt/title. | Existing-value UI. | Submit new value. | EnvPrompt update path. | Secret replaced. | `src/app_execute/execution_helpers.rs`, `src/prompts/env/prompt.rs`. |
| Delete stored secret. | Existing-value UI. | Delete target. | Click Delete. | `delete_secret`. | Stored value removed; completion semantics need proof. | `src/prompts/env/prompt.rs`, `src/secrets.rs`. |
| Inspect safely. | Automation. | Secret EnvPrompt. | `getElements`. | Env collector redacts secret value. | Secret not exposed. | `src/app_layout/collect_elements.rs`, `tests/tab_ai_input_coverage.rs`. |

## State Machine

| State | Trigger | Transition | Notes |
|---|---|---|---|
| SDK preflight. | `env(key)`. | Check `process.env[key]`. | Existing value ends flow. |
| Custom prompt. | `promptFn` provided. | Call function and set process env. | EnvPrompt bypassed. |
| Env message sent. | Missing value, no promptFn. | SDK sends key/secret/id. | Secret inferred by key name. |
| Rust route. | `ShowEnv`. | Check stored secret info. | Prompt/title fields can come from Rust/protocol routes. |
| Auto-submit. | Stored secret and no context. | Callback with stored secret. | No visible UI. |
| Editing. | User types/pastes/dictates. | EnvPrompt input state mutates. | Secret display may be masked. |
| Validation. | Submit. | Empty value blocked. | Current behavior rejects empty env values. |
| Persist secret. | Secret submit. | Write encrypted store. | Non-secret not persisted by EnvPrompt. |
| Complete/cancel. | Submit/Escape/delete. | Callback resolves or cancels. | SDK null exits. |

## Visual And Focus States

| State | Visible result | Focus owner | Automation signal |
|---|---|---|---|
| Secret EnvPrompt. | Masked input, secret/storage copy. | EnvPrompt. | Value redacted as `[secret]` or masked. |
| Existing secret update. | Update/delete stored value UI. | EnvPrompt. | `exists_in_keyring`/status elements. |
| Empty invalid submit. | Validation error/status. | EnvPrompt. | No callback result. |
| Auto-submitted secret. | No visible prompt. | None or transient. | Script receives stored value. |
| Footer. | Submit-only native footer. | Native footer slot. | `env_prompt` surface; no launcher AI. |

## Keystrokes And Commands

| Input | Scope | Behavior |
|---|---|---|
| Text input. | EnvPrompt. | Updates value. |
| Enter. | EnvPrompt. | Attempts submit. |
| Escape. | EnvPrompt. | Cancels; SDK exits on null. |
| Delete stored value. | Existing secret UI. | Deletes stored secret and completes/cancels. |

## Actions And Menus

EnvPrompt is Submit-only in the captured footer contract. It should not expose launcher AI or generic action menus unless a real EnvPrompt-specific route is designed. This is intentional prompt-owned footer behavior, not missing launcher chrome.

## Automation And Protocol Surface

| Automation target | Assertion |
|---|---|
| `getState`. | Prompt type `env`, active id, footer surface `env_prompt`. |
| `getElements`. | Env key and value/status elements present. |
| Secret `getElements`. | Secret value redacted/masked, not plaintext. |
| Active footer. | Submit action, no launcher AI. |
| `try_set_prompt_input`. | Can inject value into EnvPrompt. |
| simulateKey Enter. | Uses EnvPrompt submit path. |
| simulateKey Escape. | Cancels; no response envelope from simulateKey itself. |
| Tab AI/context snapshots. | Secret inputs must not leak. |
| ForceSubmit. | Not proven for EnvPrompt in the tight bundle. |

## Data, Storage, And Privacy Boundaries

- Secret values are script-visible after submit, but should not leak through UI, logs, automation receipts, screenshots, or Tab AI snapshots.
- Secret persistence is `~/.scriptkit/secrets.age`, not macOS Keychain in the captured source.
- The encrypted store uses age scrypt passphrase encryption, memory caching, modified timestamps, and Unix `0o600` permissions.
- Non-secret env values are returned to the running script and set in `process.env`, but EnvPrompt does not persist them.
- Key names are logged and displayed; values should not be logged.
- `SecretInfo` contains the value and modified timestamp; keep its propagation limited.
- Empty values are currently rejected.

## Error, Empty, Loading, And Disabled States

| State | Behavior |
|---|---|
| Existing process env. | Resolves immediately. |
| Empty input submit. | Blocked by validation. |
| Secret store missing. | Treated as no stored secret. |
| Secret store read/decrypt/parse failure. | EnvPrompt shows a storage-error state and `getElements` exposes the stable error kind, not secret values or raw low-level details. |
| Secret set failure. | Prompt logs error and should avoid leaking value. |
| Secret delete failure. | Logs error; user semantics need runtime proof. |
| Stored secret with no context. | Auto-submits. |
| Stored secret with title/prompt. | Shows update UI. |
| SDK null. | Treated as cancel/Escape. |
| Options object second arg. | Stale smoke tests use it; visible SDK treats second arg as function. |

## Code Ownership

| Area | Owner |
|---|---|
| SDK env resolver. | `scripts/kit-sdk.ts` owns process env preflight, promptFn bypass, secret heuristic, message send, parse/cancel handling. |
| Rust protocol conversion. | `src/prompt_handler/mod.rs`, `src/execute_script/mod.rs`, and `src/main_sections/prompt_messages.rs`. |
| EnvPrompt state/behavior. | `src/prompts/env/prompt.rs`, `helpers.rs`, `render.rs`, `tests.rs`. |
| Secret persistence. | `src/secrets.rs`. |
| API-key app flow. | `src/app_execute/execution_helpers.rs`, `src/main_sections/render_impl.rs`. |
| App view/focus/footer. | `src/main_sections/app_view_state.rs`, `src/app_impl/ui_window.rs`, `theme_focus`, `focus_coordinator`. |
| Automation receipts. | `src/app_layout/collect_elements.rs`, `build_layout_info`, `runtime_stdin_match_simulate_key.rs`, Tab AI coverage tests. |
| Contract tests. | `tests/minimal_chrome_audit.rs`, `tests/source_audits/execution_helpers.rs`, `tests/sdk/test-env.ts`, smoke env tests. |

## Invariants And Regression Risks

- SDK must not show EnvPrompt when `process.env[key]` already exists.
- SDK promptFn path must bypass EnvPrompt.
- Secret heuristic must stay documented if it controls persistence.
- Stored secrets auto-submit only when no contextual title/prompt is present.
- EnvPrompt footer Submit must not fall through to launcher execution.
- Launcher AI stays omitted.
- Secret values must remain redacted in automation and Tab AI.
- Do not assume OS Keychain; current source uses `secrets.age`.
- Stale smoke tests can mislead agents into adding unsupported options behavior by accident.
- Empty-value rejection is a product behavior and may conflict with env vars that intentionally use empty strings.
- Storage failures must not resemble missing secrets; typed storage-error kinds are surfaced in UI/automation without exposing secret values.

## Verification Recipes

| Recipe | Expected proof |
|---|---|
| SDK env test. | Existing env, secret key, custom prompt function, and missing prompt paths behave as documented. |
| Non-secret submit. | `getElements` shows value; submit resolves and sets `process.env`; no secret store write. |
| Secret submit. | Value is masked/redacted in UI/automation; script receives actual value; encrypted store created. |
| Stored secret auto-submit. | Second `env(secretKey)` returns without visible UI. |
| Contextual update. | API-key-style prompt with title/prompt shows update UI despite stored secret. |
| Delete stored value. | Delete removes secret and subsequent `env()` prompts again. |
| Privacy proof. | Secret value absent from getElements, Tab AI snapshot, logs, and screenshot. |
| Storage proof. | Missing store, invalid age data, encrypted invalid JSON, and unreadable paths stay distinct from missing keys; `~/.scriptkit/secrets.age` mode remains `0o600`, plaintext absent. |
| Stale test audit. | Reconcile tests that mention Keychain or options object support. |

## Agent Notes

Do not say EnvPrompt uses macOS Keychain unless the storage backend changes. The current proof is `~/.scriptkit/secrets.age`.


Do not add Cmd+K or launcher AI footer affordances to EnvPrompt without a product decision and tests.

Do not route footer Run through generic launcher execution.

Do not log secret values. Avoid logging full env protocol payloads.

If adding SDK options support, update TypeScript overloads, message serialization, protocol parsing, smoke tests, docs, and this chapter together.

If changing storage backend, update `src/secrets.rs`, UI copy, smoke tests, removed-docs, and privacy docs together.

## Related Features

| Feature | Relationship |
|---|---|
| [016 Prompt Runtime Core](./016-prompt-runtime-core.md). | Shared prompt state and submit patterns, but EnvPrompt has its own footer and storage boundary. |
| [017 Form and Fields Prompt](./017-form-fields-prompt.md). | EnvPrompt uses form-like visual helpers but is not `form()`. |
| [019 Path Prompt](./019-path-prompt.md). | Adjacent prompt-owned footer semantics and no launcher fallback. |
| [020 Drop Prompt](./020-drop-prompt.md). | Adjacent footer-owned prompt with privacy-sensitive data. |
| SDK Script Execution. | `env()` is SDK code and sets `process.env` for the running script. |
| Storage Cache Security. | Secret persistence, encryption, deletion, and cache behavior live in `src/secrets.rs`. |
| ACP/inline chat setup. | API key configuration prompts can create EnvPrompt directly from Rust. |

## Open Questions And Gaps

- Should product copy say Keychain or encrypted local `secrets.age`?
- Should public SDK expose prompt/title fields that Rust already supports?
- Exact `None` callback serialization is inferred from SDK null handling; `make_submit_callback` was not in the tight excerpt.
- Is ForceSubmit supported for EnvPrompt?
- How does the user trigger `toggle_secret_reveal`, and what are the privacy receipts when reveal is active?
- Full text editing behavior for Backspace/arrows/paste/IME is outside the tight bundle.
- Secret-store load/decrypt failures are surfaced separately from missing secrets via typed storage-error kinds.
- Should empty env values be allowed?
- Exact activeFooter JSON shape for EnvPrompt should be captured in runtime receipts.
