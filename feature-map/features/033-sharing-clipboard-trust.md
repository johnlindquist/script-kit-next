# 033 Sharing and Clipboard Trust Install

This chapter maps portable Script Kit share links, the clipboard trust prompt, and the plugin install path for shared scripts, scriptlets, skills, and agents.

Raw Oracle reference: [answer](../raw-oracle/033-sharing-clipboard-trust/answer.md), [prompt](../raw-oracle/033-sharing-clipboard-trust/prompt.md), [bundle map](../raw-oracle/033-sharing-clipboard-trust/bundle-map.md), [full log](../raw-oracle/033-sharing-clipboard-trust/output.log), [session metadata](../raw-oracle/033-sharing-clipboard-trust/session.json).

## Executive Summary

Sharing has two halves:

- Export path: `copy_deeplink` becomes **Share** for portable shareable launcher results. It builds a `ScriptShareBundle`, serializes it to JSON, base64-url encodes it, prefixes it with `scriptkit-share://v1/`, copies the URI to the clipboard, and marks it as recently exported so the same local app does not immediately prompt to import its own share.
- Import path: startup spawns a clipboard watcher outside tests. The watcher polls clipboard change count every 350 ms, reads clipboard text only after a change or change-detector error, extracts a `scriptkit-share://v1/...` URI even when wrapped in surrounding text, suppresses duplicate prompts, then opens a parent confirmation dialog. Only **Install** writes files.

The security model is user-mediated trust, not cryptographic trust. Share URIs contain file contents. There is no proven signature, publisher verification, remote attestation, automatic trust, sandboxed execution, or malware scanning. Install is gated by a trust prompt, path validation, known top-level directories, plugin-id normalization, and writes under `~/.scriptkit/plugins/<plugin-id>/`.

## What Users Can Do

| Result type | Portable share? | Included content |
|---|---:|---|
| Script | Yes | One text file under `scripts/<filename>`. |
| Scriptlet / snippet | Yes | Source Markdown under `scriptlets/<filename>`. |
| Skill | Yes | Skill directory under `skills/<skill_id>/`, with `SKILL.md` as entry. |
| Agent | Yes | One agent Markdown file under `agents/<filename>`. |
| Other launcher/config-backed result | No | Plain Script Kit deeplink. |
| Config-backed row with launcher command id | No | `scriptkit://commands/{commandId}`. |
| Row without launcher command id | No | `scriptkit://run/<deeplink-name>`. |

Users can also copy a received share URI to the clipboard. Script Kit detects it and prompts before installation. Ignore, close, cancel, or dialog failure does not install anything.

## Core Concepts

`src/script_sharing.rs#ScriptShareBundle` is the portable payload:

| Field | Meaning |
|---|---|
| `version` | Bundle format version; current accepted version is `1`. |
| `kind` | `script`, `scriptlet`, `skill`, or `agent`. |
| `title` | Human-facing title used in prompt and fallback metadata. |
| `plugin` | Sender-provided plugin manifest metadata. |
| `entry_path` | Relative path to the primary file inside the installed plugin root. |
| `files` | Text file payloads, each with relative `path` and `content`. |

The URI format is:

```text
scriptkit-share://v1/<payload>
```

Encoding is `ScriptShareBundle -> serde_json::to_vec -> base64 URL_SAFE_NO_PAD -> scriptkit-share://v1/<payload>`. Decoding is clipboard text -> URI extraction -> prefix stripping -> base64 decode -> JSON deserialize. `decode_share_text` can find a URI inside wrapped text.

Portable sharing is narrower than generic deeplinks. `src/script_sharing.rs#is_shareable_result` accepts only `SearchResult::Script`, `SearchResult::Scriptlet`, `SearchResult::Skill`, and `SearchResult::Agent`.

## Entry Points

| Entry | Owner | Result |
|---|---|---|
| Script context `copy_deeplink` action | `src/actions/builders/script_context.rs` | Shows **Share** for scripts, scriptlets, agents, and skills; otherwise **Copy Deep Link**. |
| Action dispatch `copy_deeplink` | `src/app_actions/handle_action/files.rs` | Builds a share URI for shareable results; otherwise copies command/run deeplink. |
| Clipboard watcher | `src/script_sharing.rs#spawn_clipboard_share_watcher` | Emits `ClipboardShareImport` for valid, non-duplicate share URIs. |
| Startup import loop | `src/app_impl/startup.rs` | Requests main window, opens trust prompt, installs on confirmation. |
| Parent confirm dialog | `src/confirm/parent_dialog.rs` | Returns true only for explicit confirm. |

`copy_deeplink` uses Cmd+Shift+D and the **Share** section for script-context actions. For non-shareable results, `deeplink_for_result` prefers `scriptkit://commands/{commandId}` through `launcher_command_id()`, then falls back to `scriptkit://run/<deeplink-name>`.

## User Workflows

### Share A Script

1. User selects a script result.
2. User opens actions and chooses **Share**.
3. `copy_deeplink` checks `is_shareable_result`.
4. `bundle_from_search_result` reads the script text file.
5. The bundle stores it under `scripts/<filename>`.
6. `encode_share_bundle` builds `scriptkit-share://v1/<payload>`.
7. `mark_recently_exported_share` records the URI hash and timestamp.
8. The URI is copied to the clipboard.
9. If the local watcher sees the same URI within 5 seconds, it suppresses the import prompt.

### Share A Scriptlet

Scriptlet sharing uses the source Markdown file. The file path anchor is stripped before packaging, and the file is stored under `scriptlets/<filename>`. The exact manifest fallback fields should be verified in `bundle_from_search_result` before changing scriptlet metadata behavior.

### Share A Skill

Skill sharing resolves the skill root folder, derives a skill id from the folder name or falls back to a slug from the title, collects files under `skills/<skill_id>/`, and sets `skills/<skill_id>/SKILL.md` as the entry path. If the folder contains no files, sharing fails.

The bundle model is text-based. `ShareFile.content` is a string, and single-file helpers use `fs::read_to_string`. Do not promise arbitrary binary asset support without verifying `collect_directory_files`.

### Share An Agent

Agent sharing packages one Markdown file under `agents/<filename>` using `ShareKind::Agent`.

### Copy A Non-shareable Deeplink

If the selected result is not a script, scriptlet, skill, or agent, the action does not build a share bundle. It copies `scriptkit://commands/{commandId}` when the row has a launcher command id, otherwise `scriptkit://run/<deeplink-name>`.

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
12. On success, the app refreshes scripts and skills, switches to `AppView::ScriptList`, and shows a HUD: `Installed shared <kind> into <plugin_id>`.

## Interaction Matrix

| User intent | Entry point | UI state | Key/click | Code path | Result | Proof |
|---|---|---|---|---|---|---|
| Share script | Selected script result | Actions menu | Share | `copy_deeplink -> bundle_from_search_result -> encode_share_bundle` | Clipboard gets `scriptkit-share://v1/...` with `scripts/<filename>`. | `src/script_sharing.rs`, `src/app_actions/handle_action/files.rs`, `lat.md/sharing.md` |
| Share scriptlet | Selected scriptlet result | Actions menu | Share | Scriptlet branch of share builder | Clipboard gets `scriptlets/<filename>` Markdown source. | `is_shareable_result`, `lat.md/sharing.md` |
| Share skill | Selected skill result | Actions menu | Share | Skill branch + directory collection | Clipboard gets `skills/<skill_id>/...` with `SKILL.md` entry. | `src/script_sharing.rs` |
| Share agent | Selected agent result | Actions menu | Share | Agent branch + single-file bundle | Clipboard gets `agents/<filename>`. | `src/script_sharing.rs`, `lat.md/sharing.md` |
| Copy command deeplink | Config-backed row | Actions menu | Copy Deep Link | `deeplink_for_result -> command_id_to_deeplink` | Clipboard gets `scriptkit://commands/{commandId}`. | `src/app_actions/handle_action/files.rs` |
| Copy fallback run deeplink | Non-command row | Actions menu | Copy Deep Link | `deeplink_for_result` fallback | Clipboard gets `scriptkit://run/<name>`. | `src/app_actions/handle_action/files.rs` |
| Avoid self-import | Local app just copied share | Watcher | None | `mark_recently_exported_share` + recent-export check | No trust prompt for sender's own recent share. | `RECENT_EXPORT_TTL = 5s` |
| Avoid duplicate prompts | Same URI repeats | Watcher | None | `RECENTLY_PROMPTED_SHARE` | Same URI suppressed for 10 seconds. | `RECENT_PROMPT_TTL = 10s` |
| Receive valid share | Clipboard changed | Main window + parent prompt | Clipboard change | watcher -> startup -> confirm | User sees trust prompt. | `src/script_sharing.rs`, `src/app_impl/startup.rs` |
| Ignore share | Trust prompt | Parent confirmation | Ignore / close | `confirm_with_parent_dialog` false path | No files written. | `src/app_impl/startup.rs` |
| Install share | Trust prompt | Parent confirmation | Install | `install_share_bundle` | Plugin manifest/files written; scripts/skills refreshed. | `src/script_sharing.rs`, `src/app_impl/startup.rs` |

## State Machine

Sender path:

```text
Selected result
  -> copy_deeplink
  -> shareable?
       no  -> build plain deeplink -> copy -> done
       yes -> build ScriptShareBundle
            -> encode scriptkit-share://v1/<payload>
            -> mark recently exported
            -> copy -> done
```

Receiver path:

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

`src/actions/builders/script_context.rs` decides action copy:

- For scripts, scriptlets, agents, and skill paths, `copy_deeplink` is titled **Share**.
- For other result types, `copy_deeplink` is titled **Copy Deep Link**.
- The shareable description is `Copy a portable Script Kit share link to clipboard`.
- The section is **Share**.

The action id stays `copy_deeplink`, so implementation agents must preserve both meanings: portable share for shareable rows and classic deeplink copy for all other rows.

## Automation And Protocol Surface

Most behavior can be verified without touching the real clipboard:

- Unit-test `encode_share_bundle` / `decode_share_text`.
- Unit-test wrapped URI extraction.
- Unit-test path validation with malicious paths.
- Unit-test install with a temp `HOME`, as `install_share_bundle_writes_plugin_manifest_and_entry_file` does.
- Source-audit or action-test the `copy_deeplink` label/shortcut/section.
- Source-audit command namespace fallback for non-shareable rows.

Runtime clipboard watcher behavior is less directly covered by the packed context. Before changing startup import routing, add or run a source audit that proves startup spawns the watcher only outside tests, requests the main window, opens **Install** / **Ignore** confirmation, installs only after confirmation, refreshes scripts and skills, and returns to `ScriptList`.

## Data, Storage, And Privacy Boundaries

Share URIs contain file contents. Base64 URL-safe encoding is transport encoding, not encryption. Treat `scriptkit-share://v1/...` as sensitive because logs, clipboard managers, or other apps with clipboard access can recover the shared file contents.

The watcher avoids reading clipboard text on unchanged polls. It reads text only when `ClipboardChangeDetector::has_changed()` reports changed, or when that check errors because the implementation uses `unwrap_or(true)`.

Installation writes only under `~/.scriptkit/plugins/<unique-plugin-id>/`. Every shared file path and `entry_path` goes through `validate_share_relative_path`.

Rejected path classes:

- Empty paths.
- Absolute paths.
- Parent traversal such as `../scripts/nope.ts`.
- Root/prefix components.
- Unknown top-level directories such as `assets/`.

Allowed top-level directories:

- `scripts/`
- `scriptlets/`
- `skills/`
- `agents/`

Sender-provided plugin manifest metadata is not identity proof. Do not treat manifest title, author, description, or repo URL as a trusted publisher signal.

## Error, Empty, Loading, And Disabled States

| Failure | Behavior |
|---|---|
| No selected result | `copy_deeplink` cannot build a share; exact selection-required feedback is shared action behavior. |
| Unsupported result sent to bundle builder | Error: `This item only supports launcher deeplinks, not clipboard sharing`. |
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
| Product docs | `lat.md/sharing.md` | User-facing sharing contract. |

## Invariants And Regression Risks

| Invariant | Why it matters |
|---|---|
| Only scripts, scriptlets, skills, and agents become portable bundles. | Prevents unintended result types from becoming filesystem-writing payloads. |
| Non-shareable results still copy plain deeplinks. | Preserves existing launcher deeplink behavior. |
| Config-backed rows prefer `scriptkit://commands/{commandId}`. | Avoids lossy `scriptkit://run/...` links for command-backed rows. |
| Share URI prefix remains `scriptkit-share://v1/` unless versioned deliberately. | Keeps decoder compatibility clear. |
| Copying a local share marks it recently exported first. | Prevents immediate self-import prompts. |
| Watcher does not inspect unchanged clipboard text. | Preserves lightweight/privacy boundary. |
| Trust prompt happens before install. | Core safety contract. |
| Ignore/close/cancel/error never installs. | Fail-closed trust behavior. |
| File paths stay relative and under allowed roots. | Blocks path traversal outside plugin root. |
| Plugin-root collisions create unique siblings. | Avoids overwriting existing plugins. |
| Success refreshes scripts and skills. | Installed content becomes visible. |

High-risk changes include logging full share URIs, adding new share kinds without path-root updates, weakening path restrictions, removing recent-share suppression, reusing an existing plugin id, assuming binary skill assets work, treating plugin metadata as identity, or adding automatic install.

## Verification Recipes

Before changing bundle format, prefix, encoding, or decode:

```bash
cargo test --lib share_bundle_round_trips_through_uri_encoding -- --nocapture
cargo test --lib decode_share_text_finds_uri_inside_wrapped_text -- --nocapture
cargo check --lib
```

Before changing install path rules or filesystem writes:

```bash
cargo test --lib validate_share_relative_path_rejects_parent_dirs -- --nocapture
cargo test --lib validate_share_relative_path_requires_known_top_level_dir -- --nocapture
cargo test --lib install_share_bundle_writes_plugin_manifest_and_entry_file -- --nocapture
cargo check --lib
```

Before changing action labels or availability:

```bash
cargo test --test builders_tests -- --nocapture
```

Before changing non-shareable deeplink fallback:

```bash
cargo test --test source_audits copy_deeplink_prefers_command_namespace_for_config_backed_rows -- --nocapture
```

Before changing startup watcher or trust prompt routing, add or run a source audit that verifies:

- `src/app_impl/startup.rs` calls `spawn_clipboard_share_watcher` only under `#[cfg(not(test))]`.
- The import loop requests the main window.
- The prompt uses **Install** and **Ignore**.
- Install happens only after confirmation.
- Success calls `refresh_scripts`, `refresh_skills`, sets `AppView::ScriptList`, and shows the installed HUD.

Always run:

```bash
lat check
git diff --check
```

## Agent Notes

- Do not assume `scriptkit-share://v1/...` is encrypted; it contains encoded file contents.
- Do not mutate the real clipboard in automated tests unless the harness explicitly owns clipboard state.
- Use temp `HOME` for install tests; do not write into the user's real `~/.scriptkit/plugins/` during proof.
- To verify share format, test encode/decode and wrapped URI extraction directly.
- To verify install safety, test invalid paths and temp-home install behavior.
- To verify UI copy, inspect action builder tests and the `copy_deeplink` action dispatch.
- If this fails at runtime, inspect `src/script_sharing.rs`, `src/app_actions/handle_action/files.rs`, and `src/app_impl/startup.rs` first.
- This belongs to sharing, actions, clipboard/security, plugin install, and parent confirmation.
- This does not prove OS-level external URL handler integration for `scriptkit-share://`; the mapped path is clipboard detection.
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
| External URL handling | Clipboard detection is proven; OS-level `scriptkit-share://` URL open handling is not. |
