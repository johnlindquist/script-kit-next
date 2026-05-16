# 033 Sharing and Clipboard Trust Install

This chapter maps portable Script Kit share links, the clipboard trust prompt, and the plugin install path for shared scripts, scriptlets, skills, and agents.


## Executive Summary



The security model is user-mediated trust, not cryptographic trust. Share URIs contain file contents. There is no proven signature, publisher verification, remote attestation, automatic trust, sandboxed execution, or malware scanning. Install is gated by a trust prompt, path validation, known top-level directories, plugin-id normalization, and writes under `~/.scriptkit/plugins/<plugin-id>/`.

## What Users Can Do

| Result type | Portable share? | Included content |
| Script | Yes | One text file under `scripts/<filename>`. |
| Scriptlet / snippet | Yes | Source Markdown under `scriptlets/<filename>`. |
| Skill | Yes | Skill directory under `skills/<skill_id>/`, with `SKILL.md` as entry. |
| Agent | Yes | One agent Markdown file under `agents/<filename>`. |
| Other launcher/config-backed result | No | Plain Script Kit deeplink. |

Users can also copy a received share URI to the clipboard. Script Kit detects it and prompts before installation. Ignore, close, cancel, or dialog failure does not install anything.

## Core Concepts


| Field | Meaning |
|---|---|
| `version` | Bundle format version; current accepted version is `1`. |
| `kind` | `script`, `scriptlet`, `skill`, or `agent`. |
| `title` | Human-facing title used in prompt and fallback metadata. |
| `plugin` | Sender-provided plugin manifest metadata. |
| `entry_path` | Relative path to the primary file inside the installed plugin root. |
| `files` | Text file payloads, each with relative `path` and `content`. |


```text
```



## Entry Points

| Entry | Owner | Result |
|---|---|---|
| Script context `copy_deeplink` action | `src/actions/builders/script_context.rs` | Shows **Share** for scripts, scriptlets, agents, and skills; otherwise **Copy Deep Link**. |
| Action dispatch `copy_deeplink` | `src/app_actions/handle_action/files.rs` | Builds a share URI for shareable results; otherwise copies command/run deeplink. |
| Clipboard watcher | `src/script_sharing.rs#spawn_clipboard_share_watcher` | Emits `ClipboardShareImport` for valid, non-duplicate share URIs. |
| Startup import loop | `src/app_impl/startup.rs` | Requests main window, opens trust prompt, installs on confirmation. |
| Parent confirm dialog | `src/confirm/parent_dialog.rs` | Returns true only for explicit confirm. |


## User Workflows

### Share A Script

1. User selects a script result.
2. User opens actions and chooses **Share**.
3. `copy_deeplink` checks `is_shareable_result`.
4. `bundle_from_search_result` reads the script text file.
5. The bundle stores it under `scripts/<filename>`.
7. `mark_recently_exported_share` records the URI hash and timestamp.
8. The URI is copied to the clipboard.
9. If the local watcher sees the same URI within 5 seconds, it suppresses the import prompt.

### Share A Scriptlet

Scriptlet sharing uses the source Markdown file. The file path anchor is stripped before packaging, and the file is stored under `scriptlets/<filename>`. The exact manifest fallback fields should be verified in `bundle_from_search_result` before changing scriptlet metadata behavior.

### Share A Skill

Skill sharing resolves the skill root folder, derives a skill id from the folder name or falls back to a slug from the title, collects files under `skills/<skill_id>/`, and sets `skills/<skill_id>/SKILL.md` as the entry path. If the folder contains no files, sharing fails.


### Share An Agent


### Copy A Non-shareable Deeplink


### Receive And Install A Share

1. A valid share URI appears on the clipboard.
2. The watcher wakes on the next 350 ms poll.
3. If clipboard change count is unchanged, it does not read clipboard text.
4. If changed, it reads text, extracts the URI, and decodes it.
5. Recently exported local URIs are ignored for 5 seconds.
6. Recently prompted URIs are ignored for 10 seconds.
7. Startup receives `ClipboardShareImport`.
8. The app requests the main window and waits 180 ms.
9. Parent confirmation opens with **Install** and **Ignore**.
10. **Ignore**, close, cancel, or dialog failure stops the flow.
11. **Install** calls `install_share_bundle`.

## Interaction Matrix

| User intent | Entry point | UI state | Key/click | Code path | Result | Proof |
|---|---|---|---|---|---|---|
| Share scriptlet | Selected scriptlet result | Actions menu | Share | Scriptlet branch of share builder | Clipboard gets `scriptlets/<filename>` Markdown source. | `is_shareable_result`, `removed-docs` |
| Share skill | Selected skill result | Actions menu | Share | Skill branch + directory collection | Clipboard gets `skills/<skill_id>/...` with `SKILL.md` entry. | `src/script_sharing.rs` |
| Share agent | Selected agent result | Actions menu | Share | Agent branch + single-file bundle | Clipboard gets `agents/<filename>`. | `src/script_sharing.rs`, `removed-docs` |
| Avoid self-import | Local app just copied share | Watcher | None | `mark_recently_exported_share` + recent-export check | No trust prompt for sender's own recent share. | `RECENT_EXPORT_TTL = 5s` |
| Avoid duplicate prompts | Same URI repeats | Watcher | None | `RECENTLY_PROMPTED_SHARE` | Same URI suppressed for 10 seconds. | `RECENT_PROMPT_TTL = 10s` |
| Receive valid share | Clipboard changed | Main window + parent prompt | Clipboard change | watcher -> startup -> confirm | User sees trust prompt. | `src/script_sharing.rs`, `src/app_impl/startup.rs` |
| Ignore share | Trust prompt | Parent confirmation | Ignore / close | `confirm_with_parent_dialog` false path | No files written. | `src/app_impl/startup.rs` |
| Install share | Trust prompt | Parent confirmation | Install | `install_share_bundle` | Plugin manifest/files written; scripts/skills refreshed. | `src/script_sharing.rs`, `src/app_impl/startup.rs` |

## State Machine


```text
Selected result
  -> copy_deeplink
  -> shareable?
       no  -> build plain deeplink -> copy -> done
       yes -> build ScriptShareBundle
            -> mark recently exported
            -> copy -> done
```


```text
Startup
  -> spawn clipboard watcher
  -> clipboard changed?
       no  -> skip text read
       yes -> read text
            -> share URI present?
                 no  -> continue
                 yes -> decode
                      -> recently exported or prompted?
                           yes -> suppress
                           no  -> send ClipboardShareImport
                                -> request main window
                                -> parent confirm
                                     Ignore/close/error -> no install
                                     Install -> validate/write plugin files
                                               -> refresh scripts/skills
                                               -> ScriptList + HUD
```

## Visual And Focus States

The export path is surfaced through the existing actions dialog. For shareable script-like rows, the visible action title is **Share** and the description says it copies a portable Script Kit share link. For non-shareable rows, the visible action title remains **Copy Deep Link**.

The import path uses a parent confirmation dialog. The title is `Install Shared <Kind>?`, where kind display names are `Script`, `Snippet`, `Skill`, and `Agent`. The body names the shared item kind, title, plugin label, file count, and trust warning. Buttons are **Install** and **Ignore**.

On successful install, focus returns to the main app's `ScriptList` view after scripts and skills refresh.

## Keystrokes And Commands

| Command | Shortcut | Behavior |
|---|---|---|
| `copy_deeplink` / Share | Cmd+Shift+D | Copies portable share URI for shareable results; plain deeplink otherwise. |
| Install | Button in trust prompt | Writes shared bundle under a unique plugin root. |
| Ignore | Button in trust prompt | Leaves clipboard content alone and writes nothing. |

## Actions And Menus


- For scripts, scriptlets, agents, and skill paths, `copy_deeplink` is titled **Share**.
- For other result types, `copy_deeplink` is titled **Copy Deep Link**.
- The shareable description is `Copy a portable Script Kit share link to clipboard`.
- The section is **Share**.


## Automation And Protocol Surface


- Unit-test `encode_share_bundle` / `decode_share_text`.
- Unit-test wrapped URI extraction.
- Unit-test path validation with malicious paths.
- Unit-test install with a temp `HOME`, as `install_share_bundle_writes_plugin_manifest_and_entry_file` does.
- Source-audit or action-test the `copy_deeplink` label/shortcut/section.
- Source-audit command namespace fallback for non-shareable rows.

Runtime clipboard watcher behavior is less directly covered by the packed context. Before changing startup import routing, add or run a source audit that proves startup spawns the watcher only outside tests, requests the main window, opens **Install** / **Ignore** confirmation, installs only after confirmation, refreshes scripts and skills, and returns to `ScriptList`.

## Data, Storage, And Privacy Boundaries



Installation writes only under `~/.scriptkit/plugins/<unique-plugin-id>/`. Every shared file path and `entry_path` goes through `validate_share_relative_path`.


- Empty paths.
- Absolute paths.
- Parent traversal such as `../scripts/nope.ts`.
- Root/prefix components.
- Unknown top-level directories such as `assets/`.


- `scripts/`
- `scriptlets/`
- `skills/`
- `agents/`

Sender-provided plugin manifest metadata is not identity proof. Do not treat manifest title, author, description, or repo URL as a trusted publisher signal.

## Error, Empty, Loading, And Disabled States

| Failure | Behavior |
|---|---|
| No selected result | `copy_deeplink` cannot build a share; exact selection-required feedback is shared action behavior. |
| Script/agent path has no filename | Share building fails. |
| File cannot be read as text | Share building fails with file path context. |
| Skill folder has no files | Share building fails with `Skill folder does not contain any files to share`. |
| JSON serialization fails | Share building fails with serialization context. |
| Clipboard unchanged | Watcher skips payload read. |
| Clipboard has no share URI | No import event. |
| Bad base64 / invalid JSON | Decode fails; no install prompt. |
| Same URI recently exported | Suppressed for 5 seconds. |
| Same URI recently prompted | Suppressed for 10 seconds. |
| User ignores/closes prompt | No install. |
| Unsupported bundle version | Install fails. |
| Empty file list | Install fails. |
| Invalid file path | Install fails. |
| Plugin root / manifest / file write fails | Install fails with filesystem context. |

Install currently writes `plugin.json`, writes files, then validates/computes `entry_path` based on the visible Oracle bundle. If atomic install semantics are required, audit this order because a failure after some writes can leave partial contents unless cleanup exists elsewhere.

## Code Ownership

| Area | File/function | Responsibility |
|---|---|---|
| Sharing model and URI format | `src/script_sharing.rs` | Bundle structs, share kind, encode/decode, install, watcher, suppression. |
| Shareability boundary | `src/script_sharing.rs#is_shareable_result` | Defines portable result types. |
| Bundle construction | `src/script_sharing.rs#bundle_from_search_result` | Builds per-kind payloads and plugin metadata. |
| Single-file payloads | `src/script_sharing.rs#build_single_file_bundle` | Reads text files and creates one-file bundles. |
| Install | `src/script_sharing.rs#install_share_bundle` | Writes plugin manifest and shared files under plugin root. |
| Action dispatch | `src/app_actions/handle_action/files.rs` | Chooses share bundle vs plain deeplink. |
| Plain deeplink fallback | `src/app_actions/handle_action/files.rs#deeplink_for_result` | Prefers command namespace, then run namespace. |
| Action labels | `src/actions/builders/script_context.rs` | Shows Share vs Copy Deep Link. |
| Startup watcher | `src/app_impl/startup.rs` | Spawns watcher, opens trust prompt, installs, refreshes. |
| Confirmation dialog | `src/confirm/parent_dialog.rs` | Parent-window async confirmation helper. |
| Product docs | `removed-docs` | User-facing sharing contract. |

## Invariants And Regression Risks

| Invariant | Why it matters |
|---|---|
| Only scripts, scriptlets, skills, and agents become portable bundles. | Prevents unintended result types from becoming filesystem-writing payloads. |
| Non-shareable results still copy plain deeplinks. | Preserves existing launcher deeplink behavior. |
| Copying a local share marks it recently exported first. | Prevents immediate self-import prompts. |
| Watcher does not inspect unchanged clipboard text. | Preserves lightweight/privacy boundary. |
| Trust prompt happens before install. | Core safety contract. |
| Ignore/close/cancel/error never installs. | Fail-closed trust behavior. |
| File paths stay relative and under allowed roots. | Blocks path traversal outside plugin root. |
| Plugin-root collisions create unique siblings. | Avoids overwriting existing plugins. |
| Success refreshes scripts and skills. | Installed content becomes visible. |

High-risk changes include logging full share URIs, adding new share kinds without path-root updates, weakening path restrictions, removing recent-share suppression, reusing an existing plugin id, assuming binary skill assets work, treating plugin metadata as identity, or adding automatic install.

## Verification Recipes


```bash
cargo test --lib share_bundle_round_trips_through_uri_encoding -- --nocapture
cargo test --lib decode_share_text_finds_uri_inside_wrapped_text -- --nocapture
cargo check --lib
```


```bash
cargo test --lib validate_share_relative_path_rejects_parent_dirs -- --nocapture
cargo test --lib validate_share_relative_path_requires_known_top_level_dir -- --nocapture
cargo test --lib install_share_bundle_writes_plugin_manifest_and_entry_file -- --nocapture
cargo check --lib
```


```bash
cargo test --test builders_tests -- --nocapture
```


```bash
cargo test --test source_audits copy_deeplink_prefers_command_namespace_for_config_backed_rows -- --nocapture
```


- `src/app_impl/startup.rs` calls `spawn_clipboard_share_watcher` only under `#[cfg(not(test))]`.
- The import loop requests the main window.
- The prompt uses **Install** and **Ignore**.
- Install happens only after confirmation.


```bash
source checks
git diff --check
```

## Agent Notes

- Do not mutate the real clipboard in automated tests unless the harness explicitly owns clipboard state.
- Use temp `HOME` for install tests; do not write into the user's real `~/.scriptkit/plugins/` during proof.
- To verify share format, test encode/decode and wrapped URI extraction directly.
- To verify install safety, test invalid paths and temp-home install behavior.
- To verify UI copy, inspect action builder tests and the `copy_deeplink` action dispatch.
- If this fails at runtime, inspect `src/script_sharing.rs`, `src/app_actions/handle_action/files.rs`, and `src/app_impl/startup.rs` first.
- This belongs to sharing, actions, clipboard/security, plugin install, and parent confirmation.
- Screenshots are only needed for visual trust-prompt regressions; most behavior is unit/source-audit proof.

## Related Features

- [011 Root Unified Search Result Actions](./011-root-source-actions.md)
- [024 Confirm Prompt and Dialogs](./024-confirm-prompt-and-dialogs.md)
- [026 Clipboard, Selected Text, and Accessibility APIs](./026-clipboard-selected-text-accessibility.md)
- [032 Script Metadata, Scriptlets, and Execution Catalog](./032-script-metadata-scriptlets.md)

## Open Questions And Gaps

| Gap | Note |
|---|---|
| Exact `resolve_plugin_manifest` fallback precedence | Verify full function before changing manifest metadata behavior. |
| Binary skill assets | Data model is text content; binary asset support is not proven. |
| Maximum payload size | No explicit maximum share URI size was visible in the Oracle bundle. |
| Signature/provenance | No publisher verification, signing, or remote attestation is proven. |
| Undo/uninstall | No rollback or uninstall path is mapped here. |
| Atomic install | Direct writes may leave partial contents on late failure unless cleanup exists elsewhere. |
| Agent refresh | Success visibly refreshes scripts and skills; no explicit `refresh_agents` call was proven. |
