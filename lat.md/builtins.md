# Built-ins

Built-ins are first-class command entries that live beside scripts in search. They are registered in one place, mapped to stable command IDs, and executed through the built-in execution pipeline rather than through ad hoc UI handlers.

## Key Facts

These facts describe how built-ins are identified, surfaced, and executed.

- `BuiltInFeature` is the authoritative enum for built-in command kinds.
- `get_builtin_entries(...)` materializes the searchable built-in catalog from config-gated feature flags.
- Built-ins now cover more than the old clipboard or app-launcher set: Agent Chat and history, notes commands, system actions, script creation, settings, utility flows, dictation targets and history, kit store commands, and current-app automation helpers.
- New-user Suggested defaults use stable built-in identities, not transient display aliases: Agent Chat, Do in Current App, New Script, Clipboard History, Open Notes, Search Files, Search Browser Tabs, Quick Terminal, and SDK Reference. The current-app row can display as `<App> Commands` while still seeding from the stable `builtin/do-in-current-app` identity.
- Direct-provider API key setup commands are no longer exposed as built-ins or Settings actions; agent setup is driven by the Agent Catalog and `config.ts` preferences instead.
- The launcher now includes a browser-tabs builtin that enumerates tabs from running Safari and Chromium-family browsers, filters them with the shared fuzzy-ranking model, and activates the chosen tab on `Enter`.
- ACP attachment portals also include a browser-history picker that snapshots recent history from supported browsers, caches that snapshot briefly for reopen speed, collapses duplicate rows by normalized page identity before ranking matches, and drives wheel scrolling through the shared selected-row path so large history sets stay responsive.
- Root launcher search can also opt into passive Dictation History, Browser Tabs, and Browser History rows backed by local metadata. These sources are disabled by default; browser sources never read page content, favicons, cookies, downloads, or network data.
- Hidden internal built-ins can still resolve by canonical command ID when hotkeys or other programmatic callers need them without exposing them in launcher search.
- The app launcher installed-app catalog scans configured macOS app roots recursively through [[src/app_launcher/scanning.rs#collect_app_paths]], so vendor subfolders under `/Applications` such as `Universal Audio/` are indexed. The scanner treats each `.app` bundle as a leaf and does not descend into `Contents/`, keeping app internals out of the launcher dataset.
- `config.ts` supports a top-level `hiddenCommands: string[]` array of canonical command IDs (e.g. `"builtin/clipboard-history"`, `"script/foo"`). Hidden commands are filtered out of launcher materialization at startup but stay resolvable via `triggerBuiltin` and hotkeys. The older per-command `commands.*.hidden` override continues to work and is OR'd with `hiddenCommands`.
- User-facing command IDs are canonicalized through the built-in config path instead of being treated as free-form labels.
- Execution routes through the built-in execution pipeline and can branch into view changes, popups, ACP handoffs, note flows, system actions, or current-app automation.
- Reset Windows immediately clears persisted bounds, resets the launcher to the default main-menu search, and moves the visible main window back to its default eye-line position without HUD feedback.
- File search treats plain `Enter` as the default OS open action for the selected item, including directories, while `Tab` browses into a selected directory inline and `Shift+Tab` moves up.
- Root launcher search inserts a capped `Files` section before fallback actions for eligible plain-text queries using a cancellable Spotlight-backed background source. Empty root search can also show a frecency-backed `Recent Files` section from previously opened file rows without starting Spotlight or directory scans. Root file frecency is recorded in the shared successful-open path, so Enter and the root-file Open action both seed Recent Files only after the OS open succeeds. Root unified-search frames are stable for a given filter text: global root file provider work may warm a bounded per-query cache, but it does not stream partial or final provider rows into the active visible frame. Cached global file rows may appear only when that query's frame is first built; uncached queries keep the stable handoff row until the next frame. Explicit directory browsing still applies one final direct-child replacement because that path is an intentional file-browser interaction, not passive global enrichment. Safe multi-word global root queries keep the literal phrase provider branch and add an all-terms filename branch, so `design notes` can recall separator-token files such as `client-design-notes.md` while one-character terms and advanced queries stay literal. Two-character ASCII alphanumeric queries that include a digit, such as `q2`, `v2`, or `3d`, are also safe global root file queries; plain two-letter queries such as `ai` or `ab` remain suppressed to avoid noisy Spotlight searches. While the provider is loading, the root Files header shows `Files · Searching...` for global filename queries or `Files · Loading folder...` for explicit directory queries; real rows still render before the continuation handoff row. Non-empty global root file queries also seed the `Files` section from frecency-backed recent files when the query hits the file basename, stem, ordered filename tokens, or the same ordered parent-directory-context plus filename-token pattern used by root file ranking; path-only recent matches are not shown as instant seeds while the provider is loading. Global root Files suppresses `.app` bundle rows and files nested under `.app` bundle path components so app launching and app internals stay out of global file results; explicit directory browse still renders `.app` direct children and contents when the user intentionally browses into those paths. Provider-returned path matches, explicit directory browse, and dedicated File Search behavior stay unchanged. Empty-root Recent Files hydrates and renders only rows that pass the same global root file eligibility gate as non-empty root Files, so direct `.app` bundles and files nested under `.app` components stay out of launcher recents; explicit directory browse can still show those paths when the user intentionally browses into them. This gives instant file rows without starting extra filesystem providers; directory browse mode does not mix recent files and still lists only direct children. Strong global filename matches in the root `Files` section are promoted above ordinary command, app, script, and skill groups when the typed query exactly matches, prefixes, or starts at a separator-, camel-case-, acronym-, or digit-delimited token in the top file row's filename or stem, unless the current top launcher result is itself a strong token match for the query. Multi-word strong matches require every meaningful query term to hit filename or stem tokens in order, so `design notes` matches `client-design-notes.md` while `notes design`, path-only, mid-token, and fuzzy-only matches stay unpromoted. The promotion check shares the same filename-token gate used for non-empty recent-file seeds, so token matches like `design` in `client-design-notes.md` or `ClientDesignNotes.md` are strong while command collisions such as a top `Fix Spelling and Grammar` row still keep the launcher command ahead of a matching file. Promotion is grouping-only and disabled while root file search is loading: weak/path-only matches, explicit directory browse, direct-child folder listing, fallback handoff rows, and the dedicated File Search provider behavior stay unchanged. Explicit directory-path queries such as `~/dev/` switch that same root `Files` section into a bounded direct-child directory listing; continuing with a child fragment such as `~/dev/al` filters those direct children inline without recursive search. Child-fragment edits reuse the active directory listing when the containing folder and hidden-file mode are unchanged, so narrowing `~/dev/` to `~/dev/al` invalidates grouped rows without clearing results or restarting the provider. Plain `Tab` on a selected root directory row rewrites the root query to that folder, and `Shift+Tab` first clears an active child fragment back to the containing folder before moving to its parent. Non-empty file sections always end with a continuation row: `Search Files for "<query>"` for global filename queries, or `Open File Search in "<folder>"` for directory-browse queries. Selection snapshots and main-window preflight receipts use `[[src/scripts/types.rs#SearchResult#stable_selection_key]]`, which is separate from `[[src/scripts/types.rs#SearchResult#history_result_key]]` so fallback and other non-history rows remain selection-safe without becoming input-history memory targets. Enter on fallback-mode rows executes the visible grouped fallback selection before consulting the legacy fallback cursor, so this continuation row cannot visually select one fallback while running another. Selected root file rows expose a minimal MainList `Cmd+K` palette for open, reveal in Finder, copy path, copy name, and Quick Look; directory rows also add `Search Inside Folder`, which hands off to the dedicated File Search view scoped to that folder. Regular file rows also expose `Browse Parent Folder`, which opens dedicated File Search at the containing directory without changing root `Enter` behavior or enabling root filesystem fallback. Root file rows honor the action palette's row-level shortcuts directly from the selected row: `Cmd+Shift+F` reveals in Finder, `Cmd+Shift+C` copies the full path, and `Cmd+Y` opens Quick Look through the same root-file action executor. The palette captures the selected file context when it opens so detached action execution survives filter focus resync. Root file rows use `FileResult.file_type` only for static SVG icon selection across folders, images, documents, media, application bundles surfaced by directory browse, and generic files. Dedicated File Search remains richer: it owns filtered directory search, fuller browser affordances, thumbnails, and bounded filesystem fallback; root directory browsing only lists direct children for explicit directory queries.
- Root file ranking is filename-first: exact filename or stem matches, filename or stem prefixes, separator-boundary filename matches, camel-case token matches, acronym-boundary matches, digit-to-word token matches, and ordered multi-word filename-token matches outrank path-only matches. Safe multi-word queries can also use ordered directory context: leading query terms may match parent-directory tokens while the remaining suffix terms match filename or stem tokens. These directory-context matches rank above path-only and fuzzy rows but below filename-only exact, prefix, and token matches. The Spotlight provider adds bounded `kMDItemPath` plus `kMDItemFSName` branches for those safe queries so ranking can see the intended rows without widening short or noisy searches. Directory-context recent seeds render quickly but do not trigger Files section promotion unless the filename itself is a strong token match. Frecency still breaks close ties inside the same textual relevance tier, but it cannot override strong filename relevance.
- Root recent-file hydration keeps a deeper frecency-backed seed pool than the empty-root Recent Files render cap, so non-empty global root file queries can instantly seed matching recent filename-token rows beyond the six visible empty-root recents. Empty-root Recent Files remains capped, eligibility-filtered, and provider-free.
- File-search directory browsing keeps the current directory rows visible until the next directory stream completes and applies one stable replacement batch, avoiding blank flashes and visible row churn during `Tab` navigation.
- File search renders a six-row skeleton while choices are still loading and no cached results are visible, preserving the real row columns instead of collapsing to a text-only spinner.
- File-search modified-time sorts compare folders and files together, so newest/oldest ordering is not overridden by directory-first grouping.
- File search still uses Spotlight first, but simple filename queries now fall back to a bounded filesystem scan when `mdfind` returns no rows, so unindexed dev folders are still discoverable.
- The `SyncToGithub` builtin wraps a `gh`-CLI worker in `src/sync/` that writes a sensitive-exclusion `.gitignore` before committing (`agent-token`, `server.json`, `**/.env*`, `**/secrets/**`, `**/*.pem`/`.key`/`.p12`/`.pfx`, `acp/auth/**`, `logs/`, `*.log`, `.DS_Store`, `node_modules/`, `.cache/`, `tmp/`). `SCRIPT_KIT_SYNC_DRY_RUN=1` gates the `gh repo create` / `git push` step so verification never pushes.
- Current-app commands now open as a session-owned capture that keeps the source app PID, bundle identity, placeholder copy, and entries together so tray-opened filtering and execution can refresh with an explicit HUD on app switch or fail closed when the frontmost app changes, relaunches under the same bundle, or can no longer be verified.
- The frontmost-app tracker now owns cached menu snapshots by `pid + bundle_id` and only publishes the latest fetch for that identity, so same-bundle relaunches and overlapping same-app refreshes cannot republish stale menu trees while the tray capture path is rebuilding state.
- The current-app snapshot loader prefers the pre-fetched menu cache that the frontmost-app tracker publishes on activation — identity-validated by `pid + bundle_id` — so "Do This in Current App" and the Current App Commands tray do not block the main UI thread on per-call Accessibility menu traversal, and falls back to a live PID-bound capture with a bounded retry + identity re-check that fails closed when the frontmost app changes during capture.
- The script-list fallback surface keeps `Do This in Current App` at the top, and current-app AI generation now emits plain shareable code instead of embedding recipe headers in the script body.
- The DoInCurrentApp → GenerateScript path now captures selected text and the focused browser URL on `cx.background_executor()` before running the automation-memory lookup and recipe build, so the launcher UI stays responsive while macOS answers the Accessibility and scripting queries.
- The focused-browser-URL lookup reads frontmost app identity from the in-process frontmost-app tracker (populated by the NSWorkspace observer) instead of spawning `osascript` against `System Events`, and only invokes a single `osascript` call when the tracked bundle identifier matches a supported browser (Safari, Chrome, Arc, Brave, Edge, Chromium, Vivaldi, Opera). Non-browser frontmost apps skip the subprocess entirely.

## Key Files

These files define the built-in catalog and its execution paths.

- [src/builtins/mod.rs](/Users/johnlindquist/dev/script-kit-gpui/src/builtins/mod.rs) - Built-in enums, grouping, entry construction, action text, and the current built-in catalog.
- [src/app_launcher/scanning.rs](/Users/johnlindquist/dev/script-kit-gpui/src/app_launcher/scanning.rs) - Installed macOS app catalog scanning, including recursive vendor-folder discovery and `.app` leaf handling.
- [src/app_execute/builtin_execution.rs](/Users/johnlindquist/dev/script-kit-gpui/src/app_execute/builtin_execution.rs) - Built-in execution paths, including ACP, notes, dictation, utility routing, and the PID-aware current-app session refresh guards.
- [src/app_impl/lifecycle_reset.rs](/Users/johnlindquist/dev/script-kit-gpui/src/app_impl/lifecycle_reset.rs) - Shared lifecycle reset helpers, including the immediate Reset Windows return-to-menu behavior.
- [src/frontmost_app_tracker/mod.rs](/Users/johnlindquist/dev/script-kit-gpui/src/frontmost_app_tracker/mod.rs) - Frontmost-app identity tracking and PID-aware menu cache ownership so refreshes cannot republish stale menu trees for the wrong app.
- [src/fallbacks/builtins.rs](/Users/johnlindquist/dev/script-kit-gpui/src/fallbacks/builtins.rs) - Built-in fallback ordering, including the current-app fallback that anchors empty-result flows.
- [src/scripts/grouping.rs](/Users/johnlindquist/dev/script-kit-gpui/src/scripts/grouping.rs) - Main-menu grouping and the exact-name default Suggested seed list used for empty frecency stores.
- [src/app_impl/root_file_search.rs](/Users/johnlindquist/dev/script-kit-gpui/src/app_impl/root_file_search.rs) - Root launcher file-result source, cancellation, debounce, and cache invalidation.
- [src/file_search/mod.rs](/Users/johnlindquist/dev/script-kit-gpui/src/file_search/mod.rs) - File search result types, root-file eligibility, and root-file ranking caps.
- [src/scripts/search/scripts.rs](/Users/johnlindquist/dev/script-kit-gpui/src/scripts/search/scripts.rs) - Script body-content search scoring and the exclusion of legacy machine-only current-app recipe headers from launcher matching.
- [src/render_builtins/dictation_history.rs](/Users/johnlindquist/dev/script-kit-gpui/src/render_builtins/dictation_history.rs) - Dictation history browser rendering, keyboard actions, and preview layout.
- [src/browser_history.rs](/Users/johnlindquist/dev/script-kit-gpui/src/browser_history.rs) - Browser history snapshot loading, duplicate collapse, caching, and fuzzy ranking for the ACP attachment portal.
- [src/render_builtins/browser_history.rs](/Users/johnlindquist/dev/script-kit-gpui/src/render_builtins/browser_history.rs) - Browser history browser rendering, keyboard navigation, and preview metadata.
- [src/browser_tabs.rs](/Users/johnlindquist/dev/script-kit-gpui/src/browser_tabs.rs) - Browser tab enumeration, fuzzy ranking, and activation routing for the browser-tabs builtin.
- [src/render_builtins/browser_tabs.rs](/Users/johnlindquist/dev/script-kit-gpui/src/render_builtins/browser_tabs.rs) - Browser tab list rendering, keyboard navigation, and footer hints.
- [src/dictation/history.rs](/Users/johnlindquist/dev/script-kit-gpui/src/dictation/history.rs) - Persistent dictation history storage, search, deletion, and MCP resource hydration.
- [src/app_execute/builtin_confirmation.rs](/Users/johnlindquist/dev/script-kit-gpui/src/app_execute/builtin_confirmation.rs) - Confirmation flow around built-in execution.
- [src/menu_bar/current_app_commands.rs](/Users/johnlindquist/dev/script-kit-gpui/src/menu_bar/current_app_commands.rs) - Current-app snapshot capture, PID-aware session metadata, recipe generation, plain-code AI prompt shaping, and replay contracts that stay outside the generated script body.
- [scripts/config-schema.ts](/Users/johnlindquist/dev/script-kit-gpui/scripts/config-schema.ts) - Command ID conventions used by config and shortcuts.

## Source Documents

These source files back the built-in behavior described here.

- [src/builtins/mod.rs](/Users/johnlindquist/dev/script-kit-gpui/src/builtins/mod.rs)
- [src/app_launcher/scanning.rs](/Users/johnlindquist/dev/script-kit-gpui/src/app_launcher/scanning.rs)
- [src/app_execute/builtin_execution.rs](/Users/johnlindquist/dev/script-kit-gpui/src/app_execute/builtin_execution.rs)
- [src/app_impl/lifecycle_reset.rs](/Users/johnlindquist/dev/script-kit-gpui/src/app_impl/lifecycle_reset.rs)
- [src/frontmost_app_tracker/mod.rs](/Users/johnlindquist/dev/script-kit-gpui/src/frontmost_app_tracker/mod.rs)
- [src/fallbacks/builtins.rs](/Users/johnlindquist/dev/script-kit-gpui/src/fallbacks/builtins.rs)
- [src/scripts/grouping.rs](/Users/johnlindquist/dev/script-kit-gpui/src/scripts/grouping.rs)
- [src/app_impl/root_file_search.rs](/Users/johnlindquist/dev/script-kit-gpui/src/app_impl/root_file_search.rs)
- [src/file_search/mod.rs](/Users/johnlindquist/dev/script-kit-gpui/src/file_search/mod.rs)
- [src/scripts/search/scripts.rs](/Users/johnlindquist/dev/script-kit-gpui/src/scripts/search/scripts.rs)
- [src/render_builtins/dictation_history.rs](/Users/johnlindquist/dev/script-kit-gpui/src/render_builtins/dictation_history.rs)
- [src/browser_history.rs](/Users/johnlindquist/dev/script-kit-gpui/src/browser_history.rs)
- [src/render_builtins/browser_history.rs](/Users/johnlindquist/dev/script-kit-gpui/src/render_builtins/browser_history.rs)
- [src/browser_tabs.rs](/Users/johnlindquist/dev/script-kit-gpui/src/browser_tabs.rs)
- [src/render_builtins/browser_tabs.rs](/Users/johnlindquist/dev/script-kit-gpui/src/render_builtins/browser_tabs.rs)
- [src/dictation/history.rs](/Users/johnlindquist/dev/script-kit-gpui/src/dictation/history.rs)
- [src/app_execute/builtin_confirmation.rs](/Users/johnlindquist/dev/script-kit-gpui/src/app_execute/builtin_confirmation.rs)
- [src/menu_bar/current_app_commands.rs](/Users/johnlindquist/dev/script-kit-gpui/src/menu_bar/current_app_commands.rs)
- [scripts/config-schema.ts](/Users/johnlindquist/dev/script-kit-gpui/scripts/config-schema.ts)

## Related Pages

These pages cover the launcher surfaces and workspace data built-ins interact with.

- [surfaces](./surfaces.md)
- [workspace](./workspace.md)
- [architecture](./architecture.md)

## Built-in Families

The current registry includes:

- core launcher tools such as clipboard history, favorites, file search, emoji picker, and ACP history
- browser navigation tools such as browser-tabs, which reads open tabs from running supported browsers and switches focus back into the selected tab
- ACP browser-history lookup, which snapshots recent visits from supported browsers, removes duplicate pages by normalized URL or title+host, reuses that snapshot briefly while the session stays hot, and keeps wheel scrolling on the shared selection path instead of GPUI native drift
- system actions such as power controls, dark mode, volume presets, and system settings
- notes and ACP commands
- script-creation and permission-management commands
- utility flows such as scratch pad, quick terminal, current-app intent routing, context inspection, and recipe replay
- kit-store and settings commands
- dictation targets for launcher, ACP, frontmost app, and notes, plus a searchable dictation history browser

The notes dictation route stays out of the top-level launcher list, but it still resolves as an internal built-in command ID. The legacy hidden `builtin/dictation-to-app` id now resolves to ACP dictation so global dictation opens chat and submits the transcript as the first prompt.

Saved dictations now share that same built-in registry. The launcher-visible Dictation History command opens a filtered browser over the persisted transcript log, uses the shared prompt search box and ranking model, wraps long preview text inside the detail pane, pastes the selected transcript to the frontmost app on `Enter`, and exposes an actions palette for ACP handoff, note creation, clipboard copy, and deletion.

Each saved entry also carries a human-readable metadata line in the browser. New external-app dictations record the tracked frontmost app name instead of the generic "Frontmost App" label, and the browser shows readable durations and local timestamps instead of raw milliseconds and RFC3339 strings.

When ACP opens dictation history as an attachment portal, that same browser switches its default `Enter` action from frontmost-app paste to attach. The selected transcript returns to ACP as a stable `kit://dictation-history?id=...` context part instead of collapsing back to the generic provider token.

That is a materially broader contract than the older “few built-ins plus apps” description.

## Root Unified Search Files

Root file rows are a passive launcher source, so async file search cannot displace primary command intent.

Root Files and empty-root Recent Files render after primary launcher rows such as commands, scripts, apps, skills, and windows, but before fallback handoff rows. Promotion defaults to `never`; the opt-in promotion policy is exact filename or stem only and still refuses to promote when primary launcher rows exist.

`config.ts` exposes `unifiedSearch.files` controls for the implemented file source: `enabled`, `globalSearch`, `recentFiles`, `directoryBrowse`, and `promotion`. Disabled sources are gated before provider startup and again during grouping so stale provider state cannot leak rows.

Global root file rows are frozen in a per-query frame. Provider completion may update provider loading receipts and warm the bounded cache, but it must not mutate active file rows, recent rows, visible loading, grouped caches, preflight, selection, or notifications for the same filter text. Warmed rows become eligible only when a new frame is built after the query changes or the frame is intentionally reset, so cold searches can show only a loading header while a later same query can show file rows immediately from cache. Explicit directory browse remains the only root file mode that may publish a final direct-child replacement into the active frame.

Source-filter heads can make Files the only participating root source. When `files:` or `f:` appears anywhere as a standalone token, grouping suppresses primary and fallback rows, strips the token before provider search, and keys the file frame by both stripped query and source-filter set so delayed provider results cannot replace a different source-only frame. These filters are transparent refinements, not power-user UI mode switches, so `png files:` renders File rows without the menu-syntax hint; a bare leading `:` is reserved for filter discovery and insertion.

When the stripped query is empty, `files:` and `f:` browse the frecency-backed Recent Files set even if ordinary root file search is disabled. Explicit source-only Files browse uses the deeper recent-file seed target instead of the ordinary empty-root render cap, while plain empty root still renders the capped Recent Files set. Disabled recent-file refreshes leave the hydration revision invalid so an explicit source-only Files filter can load current recents immediately instead of reusing an empty disabled frame.

Explicit Files source-filter queries also own a source-specific length floor. A stripped one-character ASCII alphanumeric query such as `f:s` or `files:s` may search Files, while the same plain root query `s` remains below the ordinary global file-search threshold and `f: ` continues to browse Recent Files.

Explicit positive source-filter queries expose source-chip status metadata for Files, passive sources, base launcher sources, and Windows. Status is informational metadata, not a ScriptList row: it stays out of executable results, selection, list item counts, mini-window sizing, and scroll height, while `getElements` still exposes it as `role:"status"` / `kind:"sourceStatus"` so automation can prove capped, loading, empty, or exhausted source states without treating status as an action target. Source-filter mode also disables launcher input-history recall so Up and Down remain list navigation, even when selection is at the first selectable row. `f:<text>` and `files:<text>` start on a 12-row source-chip page and automatically reveal the next page when keyboard selection reaches the final few Files rows, while plain root searches keep the ordinary six-row passive cap. Page expansion must sync the ScriptList row count and use a deferred footer-safe reveal so lazy provider rows cannot leave the selected file below the footer or snap selection back to the first visible row.

The root-file `Browse Parent Folder` action is file-only. It clears the stale MainList highlight before handing off to dedicated File Search and displays home-scoped parent folders with `~`, while the File Search provider still resolves the query to an absolute filesystem path.

The root-file `Quick Look` action is also MainList-scoped. Both the actions dialog and the direct `Cmd+Y` shortcut call the shared file OS helper with the captured file path, report missing-path or launch failures through HUD feedback, and do not route through dedicated File Search or clipboard preview state.

The `~` / `~/...` mini File Search handoff clears menu-syntax input decorations before the first File Search paint. Source heads such as `f:` enable Files source filtering; home-path syntax is separate and must never inherit source-chip chrome from the launcher input.

## Root Unified Search ACP History

ACP conversation rows are a passive launcher source backed by saved conversation history.

Eligible non-empty root queries append an AI Conversations section using the existing ACP history search index. Rows render after primary launcher intent and root file rows, before fallback handoff rows. They never promote above commands, scripts, apps, skills, windows, or actions.

`config.ts` exposes `unifiedSearch.acpHistory` only for the implemented ACP history source. Selecting a row resumes the saved conversation through the shared `[[src/render_builtins/acp_history.rs#ScriptListApp#resume_acp_conversation_from_history]]` path; root search does not expose attach-summary actions in this first pass.

When `conversations:` or `ai:` has no stripped search text, AI Conversations returns recent saved conversation metadata. This default browse stays source-only and does not make saved conversations appear on the ordinary empty root launcher.

## Root Unified Search Notes

Notes rows are a passive launcher source backed by local Notes metadata search.

Eligible non-empty root queries append a Notes section through the configured passive source order. By default they appear after root Files and Browser Tabs, before Clipboard History, Dictation History, AI Conversations, Browser History, and fallback rows. The rows are not part of primary fuzzy sorting and never promote above commands, scripts, apps, skills, windows, or actions.

`config.ts` exposes `unifiedSearch.notes` for the implemented source: `enabled`, `maxResults`, `minQueryChars`, and `searchContent`. The storage search may use Notes FTS over title and content, but root rows only carry note id, title, updated time, pinned state, character count, and score.

Selecting a root Note row opens or focuses the floating Notes window through `[[src/notes/window/window_ops.rs#open_note_in_notes_window]]`, then selects the note in the editor. Root search must not call the toggle-style `[[src/notes/window/window_ops.rs#open_notes_window]]` helper, because that helper closes an already-open Notes window.

Standalone source-filter heads `notes:` and `n:` make Notes the only passive source allowed to append rows for the stripped query. The passive frame key includes the source-filter set, preventing cached Notes rows from bleeding into Files, Clipboard History, or unfiltered frames with the same search words.

Explicit positive source heads opt into their source for the active query. Disabled passive defaults still stay hidden during ordinary search, but typing a matching head such as `n:` or `clipboard:` means the user intentionally selected that source.

When `notes:` or `n:` has no stripped search text, Notes returns the pinned and most recently updated active notes. Non-empty note filters keep FTS as the first path, then fall back to bounded title/content substring matching so `n:not` can find a note titled `Welcome to Notes`. This keeps source-only notes filters usable as a browse mode without exposing note bodies in root rows.

## Root Unified Search Browser Tabs

Browser tab rows are an opt-in passive launcher source backed by currently open tab metadata.

Eligible root queries append a Browser Tabs section through the configured passive source order. By default they appear after root Files and Recent Files, before Notes, Clipboard History, Dictation History, AI Conversations, Browser History, and fallback rows. Rows are capped through the shared passive-score helper so they never outrank commands, scripts, apps, skills, windows, or actions.

`config.ts` exposes `unifiedSearch.browserTabs`, disabled by default, with controls for `maxResults`, `minQueryChars`, `scanLimit`, `providers`, `searchUrls`, and `cacheTtlMs`. The root source reads only from the current open-tab metadata snapshot on the foreground grouping path, while stale or missing snapshots refresh in the background. Refresh completion warms future frames without notifying the main list, invalidating grouped results, fetching favicons, or reading page content.

Root Browser Tabs rows carry title, URL, domain, provider label, tab location, and a stable `browser-tab/...` key. Selecting a row switches the existing tab through `[[src/browser_tabs.rs#activate_tab]]` via the root focus helper rather than opening a duplicate URL.

When `tabs:` or `t:` has no stripped search text, Browser Tabs returns the current open-tab snapshot in source-only mode. The foreground path still reads only cached tab metadata and still suppresses every unselected source.

## Root Unified Passive Snapshot Caches

Passive root sources use cached snapshots on the foreground grouping path so late local-provider work cannot shift the selected target.

Browser Tabs and Browser History refresh stale snapshots in the background. Notes, Clipboard History, Dictation History, and ACP History use cache-only foreground lookups and start cold SQLite or JSONL work on background warmers. The app layer freezes every passive hit vector in a per-query passive frame before grouping, so a background refresh can change cache status without changing the active query's visible rows, focused target, or Enter action. ACP history clamps legacy multi-megabyte `search_text` fields before cached ranking so root typing does not repeatedly lowercase or clone large transcripts. Refresh completion must not invalidate grouped results, notify the main list, or change the visible result fingerprint for the same filter text; it only warms a future frame after the query changes or the cache is rebuilt for another reason.

## Root Unified Search Passive Ranking Receipt

Root passive ranking receipts make the launcher row-order contract visible to automation without exposing row content.

The `mainWindowPreflight` receipt now includes `selectedResultRole` and `visibleResults`. Each visible row receipt carries grouped index, visible rank, stable selection key, role, action kind, type label, and source name. The role is derived directly from `SearchResult`: scripts, scriptlets, skills, built-ins, apps, and windows are primary; root files are root-file rows; Notes, ACP History, Clipboard History, Dictation History, Browser Tabs, and Browser History are root-passive rows; fallbacks, script issues, and agents stay distinct. This lets state-first proofs verify that real passive rows are present but remain below primary launcher intent for the same query, including when `config.ts` reorders passive sections.

## Root Unified Search Result Actions

Root result actions make Cmd+K operate on the same focused ScriptList row identity that Enter would execute.

When `AppView::ScriptList` focuses a committed root unified source row, Cmd+K resolves the visible `SearchResult` through the grouped selection projection, converts it into a captured `RootUnifiedActionSubject`, and opens a MainList actions dialog from `[[src/app_impl/root_unified_result_actions.rs#root_unified_actions_for_subject]]`. The pending subject stays attached to the dialog so action execution does not re-read selection after keyboard movement, cache warming, or source-filter frame changes.

The shared actions dialog remains only the host: it owns presentation, popup lifecycle, search inside actions, and focus restoration. Source-specific IDs, labels, sections, and execution are owned by the root result catalog in `[[src/app_impl/root_unified_result_actions.rs#RootUnifiedResultAction]]`; unknown root IDs log and no-op instead of falling through to generic script `handle_action`.

Plain Enter and action-row shortcuts route through the same activation handler used by click callbacks. The route carries whether the popup should close so MainList root actions can execute against the captured subject before close/reset clears pending context.

Scripts and scriptlets remain delegated to the existing MainList script action owner so SDK-defined and script-specific actions are preserved. Dedicated built-in views such as Clipboard History, Dictation History, File Search, ACP History, Browser History, Browser Tabs, App Launcher, and Window Switcher keep their existing action hosts; the root result action path is scoped to `ActionsDialogHost::MainList`.

Windows source filters use the same primary `SearchResult::Window` rows as the window switcher search model. In agentic sessions, `SCRIPT_KIT_WINDOW_SEARCH_TEST_PROVIDER` can seed metadata-only `WindowInfo::for_test` rows so `w:` action receipts do not depend on macOS Accessibility state.

The `actionsDialog` state receipt exposes only content-light action metadata: host, context title, context stable key, context source, selected action id, and visible action IDs/labels/sections/shortcuts. It must not expose note bodies, raw clipboard content, dictation transcripts, browser page contents, or other local payloads.

## Root Unified Search Passive Source Order

Passive source order is user-configurable while primary rows, root file rows, and fallback rows keep their safety positions.

`config.ts` exposes `unifiedSearch.passiveSourceOrder` with values for `browserTabs`, `notes`, `clipboardHistory`, `dictationHistory`, `acpHistory`, and `browserHistory`. The runtime deduplicates configured entries, appends missing defaults, and only reorders passive local sections. It does not enable disabled sources, skip enabled sources, or let passive rows move ahead of primary launcher rows or root Files.

Source filters are the exception to passive defaults: an explicit positive source head enables that source for the current stripped query, then suppresses unselected primary, fallback, root-file, and passive rows. This keeps `c: text`, `clipboard: text`, `ai: text`, and similar filters predictable even when the source is disabled for unfiltered passive search.

Source-only filters also define each source's default browse behavior. Empty `c: ` shows recent clipboard metadata, `ai: ` shows saved conversations, `d: ` shows recent dictations, `n: ` shows pinned/recent notes, `t: ` shows open tabs, `h: ` shows recent browser history, and `f: ` shows Recent Files.

## Root Unified Search Passive Result Limits

Passive result limits keep enabled local sources useful without letting them dominate command intent.

`config.ts` exposes `unifiedSearch.passiveResultLimits` with a global passive cap, a lower cap when primary launcher rows are visible, and a per-source cap for primary collisions. The budget is applied after root Files and Recent Files and before passive sections are appended, so it never reorders primary rows, root file rows, or fallback rows. A zero collision budget is valid and hides passive rows when commands, scripts, apps, skills, or windows already match.

## Root Unified Search Clipboard History

Clipboard history rows are an opt-in passive launcher source for non-empty root queries.

Root Clipboard History scans bounded recent clipboard metadata only, never raw clipboard content during grouping. Rows render through the configured passive source order; by default they appear after Files, Browser Tabs, and Notes, before Dictation History, AI Conversations, Browser History, and fallback rows. Enter reuses the existing clipboard copy plus simulated paste contract.

`config.ts` exposes `unifiedSearch.clipboardHistory`, disabled by default and additionally gated by `builtIns.clipboardHistory`. This source excludes empty-root recents, images, OCR, pin/delete actions, and attach-to-AI actions in its first pass.

Source-filter heads `clipboard:` and `c:` make Clipboard History the only passive source allowed for the stripped query. The query text may be spaced or attached, so `c: skip`, `c:skip`, and `clipboard:skip` all search Clipboard History for `skip`. Primary, fallback, Files, Notes, and other passive rows are suppressed unless their own source head is also present.

Because `clipboard:` and `c:` are explicit source selection, they opt Clipboard History into the current query even though unfiltered passive Clipboard History remains disabled by default. The query still uses bounded metadata and minimum-query rules, but explicit source selection performs the direct local lookup instead of waiting for a cold passive cache to warm.

When `clipboard:` or `c:` has no stripped search text, Clipboard History returns bounded recent entries from its metadata table. This is the source's default browse mode; it does not turn on unfiltered empty-root clipboard rows.

## Root Unified Search Dictation History

Dictation history rows are opt-in passive launcher rows backed by saved local transcripts.

Root Dictation History scans the compacted local `dictation-history.jsonl` transcript log with a bounded `scanLimit`, excludes empty, short, newline, disabled, and advanced queries, and appends capped passive rows through the configured passive source order. By default it appears after Clipboard History and before AI Conversations. Rows carry metadata only: id, preview, target, timestamp, duration, matched field, subtitle, and score. Enter loads the full transcript by selected id and reuses the existing paste flow. The source is disabled by default through `unifiedSearch.dictationHistory` so users explicitly choose whether voice transcripts appear in the main launcher.

When `dictation:` or `d:` has no stripped search text, Dictation History returns recent saved transcript metadata. The full transcript still loads only after explicit selection.

## Root Unified Search Browser History

Browser history rows are an opt-in passive launcher source backed by local browser URL metadata.

Eligible root queries append a Browser History section through the configured passive source order. By default it appears after Files, Browser Tabs, Notes, Clipboard History, Dictation History, and AI Conversations, before fallback handoff rows. The rows are not part of primary fuzzy sorting and never promote above commands, scripts, apps, skills, windows, or actions.

`config.ts` exposes `unifiedSearch.browserHistory`, disabled by default, with controls for `maxResults`, `minQueryChars`, `maxAgeDays`, `providers`, and `searchUrls`. The root source is intentionally narrower than the dedicated browser-history picker: it only scans copied SQLite snapshots from Arc, Chrome, Brave, and Edge Chromium history databases.

Root Browser History rows carry title, URL, domain, provider label, profile label, last visit time, visit count, and a stable `browser-history/...` key. Selecting a row opens the URL through `[[src/browser_history.rs#open_browser_history_url]]`, which rejects non-HTTP(S) schemes before handing off to the OS default browser.

When `history:` or `h:` has no stripped search text, Browser History returns the current bounded recent-history snapshot. It still filters to safe HTTP(S) URL metadata and never reads favicons, cookies, downloads, page content, or the network.

## Dictation model download prompt

The Parakeet model prompt is a stateful MiniPrompt that must not reinterpret a repeated submit as a destructive action after its choices change.

[[src/app_execute/builtin_execution.rs#ScriptListApp#render_dictation_model_prompt]] persists the latest model-download state and reselects the default row only when the prompt opens or the model state phase changes. When the phase becomes `Downloading`, [[src/app_execute/builtin_execution.rs#ScriptListApp#preferred_dictation_model_prompt_index]] selects `Hide` instead of `Cancel download`, so a duplicate Enter from the preceding retry/download submit cannot immediately cancel the background download. Progress-only updates keep the user's current row selection intact.

## Process Manager

Process Manager is a mini built-in list for active Script Kit child processes, with filtering, selection, and destructive stop actions kept on one visible-row contract.

The renderer filters through one helper family, so keyboard navigation, `getState`, `getElements`, Tab AI targets, and visible rows agree on the same selected process. Wheel movement uses the shared selection-owned uniform-list helpers and vendor scrollbar, while periodic refresh and post-stop clamping re-evaluate the filtered visible rows instead of the raw process cache.

Process Manager routes secondary text through `AppChromeColors`, centers empty states in the available list pane, and stops propagation for row and Stop All clicks so parent surfaces cannot double-handle destructive actions.

## Kit Store Footers

Kit Store browse and installed views keep their domain actions in the native footer slot, preventing an in-content PromptFooter from stacking with the AppKit footer.

Browse maps native `Run` to Install and `Close` to Back, or Clear Search while a query is active; Installed maps `Run` to Update and `Apply` to Remove. The renderer still supplies GPUI hint strips as fallback elements, but `main_window_footer_slot` replaces them with the shared spacer whenever the native footer is active.

Stdin-driven view transitions explicitly resync the native footer before notifying, but the active native surface is only published after the AppKit host is installed. If host installation fails, the GPUI fallback footer remains visible.

Automation checks `activeFooter.owner:"native"`, `nativeFooterHostInstalled:true`, and one visible footer row for Kit Store surfaces. Any missing footer or stacked footer row is a contract failure.

## Design Gallery Footer

Design Gallery keeps its single Select affordance in the native footer slot, preventing an in-content PromptFooter from stacking with the AppKit footer.

The gallery registers the `design_gallery` native footer surface and renders only a `↵ Select` fallback hint through `main_window_footer_slot`. Native footer Run is handled by a gallery-specific guard before launcher fallback, preserving the current no-op Select behavior until a real selection action exists.

Like Kit Store, Design Gallery publishes native footer ownership only after the host is installed. Host failure leaves the GPUI Select fallback visible instead of suppressing the footer with a blank spacer.

## Theme Chooser Footer

Theme Chooser owns only the native main-window footer so its Apply and navigation chrome cannot stack with a GPUI footer fragment.

`ThemeChooserView` maps to the `theme_chooser` native footer surface through `AppView::native_footer_surface()`. The legacy GPUI footer-return fragment is not wired, and automation expects one native footer row with no prompt fallback row.

## Settings Hub

Settings is a mini built-in list for operational configuration actions, with renderer, automation, and state receipts sharing the same filtered-row projection.

The Settings renderer owns typed filtering, keyboard selection, row activation, and actions-popup routing while keeping footer ownership behind `main_window_footer_slot`. `getElements` and `getState` read the same helper family as render, so automation sees the same setting rows and selected value that the user sees on screen. Its surface contract now reports `SurfaceKind::Settings` and `automationSemanticSurface:"settings"`, so matrix proofs can distinguish it from generic `scriptList`.

## Process Manager List Ownership

Process Manager is the reference visible-row owner for mini built-in lists.

Renderer keyboard movement, wheel selection, `getState`, `getElements`, and destructive stop actions all resolve through the Process Manager filtered-entry helpers instead of indexing the backing process vector directly. Runtime duration display routes through `crate::formatting::format_running_duration`, keeping elapsed-time text out of renderer-local chrono math.

## Kit Store List Ownership

Kit Store browse and installed views share the Process Manager visible-row pattern.

Browse and Installed expose `kit_store_*_visible_rows`, selected-visible helpers, dataset/visible count helpers, and row-label helpers. Footer actions, keyboard Enter/Delete, click handlers, `getState`, and `getElements` resolve selected kits through those helpers so selection stays tied to the visible projection. Raw `installed_at` remains RFC3339 storage only and display code must format timestamps through the shared formatting module.

## Adjacent List Ownership

Adjacent built-in list surfaces expose the same visible-row helper shape for automation and state receipts.

Clipboard History, Browser Tabs, Design Gallery, Dictation History, Notes Browse, and ACP History each own helper families for visible rows, selected visible row, dataset/visible counts, and row labels. `getState` and `getElements` route through those helpers so state receipts, element receipts, and render-time selection use the same filter projection. About is explicitly exempt via `ABOUT_SURFACE_EXEMPTION` because it is static content with no list selection owner.

## Trigger-builtin registry

The stdin `triggerBuiltin` verb resolves through a single canonical registry instead of three hand-kept match arms. Duplicate aliases or typoed canonical ids fail loudly at startup, not silently at runtime.

- [[src/builtins/trigger_registry.rs#TriggerBuiltin]] is the exhaustive enum of statically-registered trigger-builtins. Adding a variant forces a matching dispatch arm in [[src/app_impl/trigger_builtin_dispatch.rs]], so the stdin ingress and the internal dispatch path can never drift apart.
- [[src/builtins/trigger_registry.rs#TriggerBuiltinRegistry]] is built once at startup via [[src/builtins/trigger_registry.rs#validate_trigger_registry]] and panics with a descriptive error if aliases or command ids collide. The OnceLock instance is reused for every subsequent lookup.
- Three stdin dispatcher surfaces (`app_run_setup.rs`, `runtime_stdin.rs`, `runtime_stdin_match_core.rs`) all delegate to the single helper `view.dispatch_trigger_builtin_name(name, window, ctx)`. The helper owns the exhaustive match, the rate-limited unknown-name warn, and the payload-capped log preview, so the Run 7 Pass #8/#9 log-spam class cannot re-emerge in only one site.
- The umbrella dispatch (`dispatch_trigger_builtin_name` + `apply_trigger_builtin` in [[src/app_impl/trigger_builtin_dispatch.rs]]) is view-agnostic. Firing `triggerBuiltin X` while an already-active non-main prompt (e.g. `DesignGalleryView`) is on screen flips the prompt in place — no `escape` / `hide` is required to cross from one filterable view to another. The umbrella deliberately does NOT gate on `self.current_view`; it unconditionally clears `opened_from_main_menu`, resolves via the registry, and routes through the pure planner. Pinned by `tests/trigger_builtin_dispatch_view_agnostic_contract.rs` (3 tests: neither function body references `current_view`; the `opened_from_main_menu` clear precedes the `trigger_registry().resolve(` call). Run 9 Pass #16 A30 live-verified this receipt against fresh binary pid 30544 (from `DesignGalleryView` 68-item, `triggerBuiltin browser-tabs` → `{promptType:"browserTabs", choiceCount:39, visibleChoiceCount:39}`).
- Unknown-name dispatches increment [[src/protocol_stats.rs#PROTOCOL_STATS]]`trigger_builtin_unknown_total` and log only on the 1st + every 100th occurrence via [[src/protocol_stats.rs#should_log_occurrence]]. A hostile or buggy peer cannot spam `app.log`.
- Two source-audit tests pin the contract at `cargo test` time (Oracle-Session `protocol-builtin-boundary-refactor-plan` PR1). [[tests/source_audits/trigger_builtin_sdk_literals.rs]] walks every `.ts` / `.tsx` / `.mts` / `.js` / `.mjs` file under `scripts/` and `tests/`, extracts every `triggerBuiltin("X")` function call and every `{"type":"triggerBuiltin","name":"X"}` / `"builtinId":"X"` JSON literal, and fails if any literal does not resolve via the registry — a Bun-land typo or dropped registration is now a publish-time failure instead of a silent runtime no-op. [[tests/source_audits/trigger_builtin_registry_consistency.rs]] walks `TriggerBuiltin::ALL` and asserts every variant whose [[src/builtins/trigger_registry.rs#TriggerBuiltin#requires_builtin_feature_entry]] returns `true` resolves via [[src/builtins/mod.rs#resolve_builtin_entry]]; variants that return `false` (`AppLauncher`, `CurrentAppCommands`) are internal-only routes with no launcher entry and are also asserted to stay un-registered so the flag can never silently desync.
- A pure resolver layer ([[src/builtins/trigger_resolve.rs#resolve_trigger_builtin]], Oracle-Session `protocol-builtin-boundary-refactor-plan` PR3) reads `name` / `builtinId` from a JSON body and returns a structured [[src/builtins/trigger_resolve.rs#TriggerBuiltinResolution]]: `MissingKey`, `Unknown { supplied }`, `Conflict { from_name, from_builtin_id }`, or `Resolved { id, via }`. The `via` field distinguishes [[src/builtins/trigger_resolve.rs#ResolvedVia]]`::BuiltinIdField`, `NameAsCommandId`, `NameAlias`, and `BothAgree`, which is the observability hook PR4 will count. The routing table is audited without a real window by `tests/trigger_builtin_resolve_golden.rs` against `tests/golden/trigger_builtin/basic.jsonl` — every line is one `{input, expected}` record and the rendering is pinned by [[src/builtins/trigger_resolve.rs#render_resolution]]. Two guard tests also assert the fixture still covers every `ResolvedVia` arm and every unresolved arm, so the golden file cannot decay into a narrow subset.

## Narrow route planner

The `triggerBuiltin` dispatch splits "which route should we enter?" (pure data) from "mutate `self` and resize" (imperative). A missing route is a compile break, not a runtime no-op.

- [[src/app_impl/routes.rs#AppRoute]] is the narrow enum of intended UI transitions: `ShowFilterableView(FilterableView)`, `OpenFileSearch`, `OpenTabAi`, `OpenCurrentAppCommands`. Variants deliberately omit `Window`/`Context` handles and cache-seed data — that imperative half lives in [[src/app_impl/trigger_builtin_dispatch.rs#ScriptListApp#apply_trigger_builtin]] (Oracle-Session `protocol-builtin-boundary-refactor-plan` PR5b + PR5c).
- [[src/app_impl/routes.rs#plan_trigger_builtin_route]] is a `const fn` total mapping from [[src/builtins/trigger_registry.rs#TriggerBuiltin]] to [[src/app_impl/routes.rs#AppRoute]]. Because it is `const` and exhaustive, a new `TriggerBuiltin` variant either grows a matching arm here or fails to compile. [[src/app_impl/trigger_builtin_dispatch.rs#ScriptListApp#apply_trigger_builtin]] now calls the planner and matches on the returned [[src/app_impl/routes.rs#AppRoute]] (PR5c); the per-view cache-seed / filter-reset / deferred-resize work is isolated in [[src/app_impl/trigger_builtin_dispatch.rs#ScriptListApp#show_filterable_view]], which exhaustively matches [[src/app_impl/routes.rs#FilterableView]]. There is no wildcard catch-all on either level.
- Six inline tests pin the contract under `cargo test --lib`: `every_trigger_builtin_has_a_route` (exhaustiveness), `every_filterable_view_is_reachable` (reverse coverage: [[src/app_impl/routes.rs#FilterableView]]`::ALL` agrees with the set of views the planner produces), `non_filterable_routes_are_one_to_one` (each of `OpenFileSearch` / `OpenTabAi` / `OpenCurrentAppCommands` is produced by exactly one `TriggerBuiltin`), `specific_known_routes_are_stable` (belt-and-braces literals), `apply_trigger_builtin_is_wired_through_planner` (audit-style source grep on `src/app_impl/trigger_builtin_dispatch.rs` asserting the live dispatcher still routes through `plan_trigger_builtin_route`, so a future refactor cannot silently re-inline the match), and `dispatch_trigger_builtin_name_delegates_to_typed_entry` (Oracle-Session `protocol-builtin-boundary-engineering-plan` Pass 4 / rank #3 sub-pass 1: the string-entry dispatcher must forward resolved ids into the typed `dispatch_trigger_builtin_enum` bridge so the eventual ingress-side resolver migration stays safe). Since PR5c wired the planner into the live dispatcher, these tests now also pin the dispatcher's outer match shape — a dropped `AppRoute` variant fails the test before it reaches `cargo build --bin`.
- [[src/app_impl/trigger_builtin_dispatch.rs]] exposes a typed entry point `dispatch_trigger_builtin_enum(id: TriggerBuiltin, window, cx) -> TriggerBuiltin` (Oracle-Session `protocol-builtin-boundary-engineering-plan` Pass 4, rank #3 sub-pass 1). The string entry point `dispatch_trigger_builtin_name(name, window, cx)` now resolves once via [[src/builtins/trigger_registry.rs#TriggerBuiltinRegistry]] and forwards the resolved variant into `dispatch_trigger_builtin_enum`. Callers that already hold a [[src/builtins/trigger_registry.rs#TriggerBuiltin]] (e.g. after running [[src/builtins/trigger_resolve.rs#resolve_trigger_builtin]] in an ingress layer) can call the typed bridge directly and skip a second registry lookup. No behavior change: `opened_from_main_menu` still resets before `apply_trigger_builtin` fires.
- The planner is re-exported from `src/lib.rs` with the same `#[path = "app_impl/routes.rs"]` pattern used for [[src/app_impl/path_action.rs#PathAction]], so its tests run under `cargo test --lib` without needing a binary build.
- A golden-JSONL dispatcher trace at `tests/golden/trigger_builtin/routes.jsonl` pins the planner's full routing table (Oracle-Session `protocol-builtin-boundary-refactor-plan` PR6). Each line is one `{input, expected}` record where `input` is the canonical `builtin/...` command id and `expected` is the string produced by [[src/app_impl/routes.rs#render_route]]. [[tests/trigger_builtin_route_golden.rs]] carries four guard tests that together guarantee the fixture is: byte-for-byte reproducible (`every_route_case_matches_its_golden_line`), exhaustively variant-covered (`fixture_covers_every_trigger_builtin_variant`), exhaustively route-covered (`fixture_covers_every_app_route_kind`), and 1:1 (`fixture_has_exactly_one_case_per_variant`). Adding a new `TriggerBuiltin` variant without a line in the fixture fails at `cargo test`, not at runtime — this is the same golden-transcript pattern the resolver side uses in PR3 (`tests/golden/trigger_builtin/basic.jsonl`).
- The wire format is bidirectional (Oracle-Session `protocol-builtin-boundary-refactor-plan` PR7): [[src/app_impl/routes.rs#parse_route]] is the inverse of [[src/app_impl/routes.rs#render_route]], so Bun or MCP consumers can encode an `AppRoute` as a string and Rust will ingest it without a second lookup table. The inline test `parse_route_round_trips_render_route` walks `TriggerBuiltin::ALL` and pins `parse_route(render_route(&route)) == Some(route)` for every route the planner can emit; `parse_route_rejects_unknown_strings` pins that unknown / empty / wrong-case strings return `None`. The external guard `every_fixture_expected_parses_back` in [[tests/trigger_builtin_route_golden.rs]] asserts the same round-trip against every `expected` string in `routes.jsonl`, so the golden fixture is now a two-way contract — changing the rendering without updating the parser (or vice-versa) fails at `cargo test`.

## Path-prompt action dispatcher

The path-prompt action-menu ids (`select_file`, `copy_path`, `move_to_trash`, …) parse through a single typed handle instead of a stringly-typed match. A typo or missing registration fails at parse time with one log line, never as a silent no-op arm.

- [[src/app_impl/path_action.rs#PathAction]] is the exhaustive enum of ids the path dispatcher will execute (Oracle-Session `protocol-builtin-boundary-refactor-plan` PR5a). It is re-exported under the library crate via `src/lib.rs` so both binary `include!` and `cargo test --lib` compile the same file, and its round-trip tests run with zero window.
- [[src/app_impl/path_action.rs#PathAction#from_action_id]] strips the optional `file:` prefix once at the boundary and returns `Option<PathAction>`. [[src/app_impl/execution_paths.rs#ScriptListApp#execute_path_action]] binds that option with a `let … else` early-return, so the only unknown-id log line lives at the ingress — the inner `match` is an exhaustive enum switch with no catch-all arm.
- The round-trip table is pinned by inline tests `every_variant_round_trips`, `file_prefix_is_stripped`, `unknown_is_none`, `action_ids_are_unique`, `action_ids_are_snake_case`, and `legacy_select_and_open_dir_still_parse`. Renaming a variant without also updating [[src/app_impl/path_action.rs#PathAction#action_id]] is a compile break; dropping a Bun-side id is a test failure.
- Adding a new path action means adding a variant, which forces a matching arm in `execute_path_action`. Binary and library both see the same enum, so future MCP/JSON-RPC consumers can parse ids through [[src/app_impl/path_action.rs#PathAction#from_action_id]] without duplicating the string table.

## Main Window Sizing Modes

Built-in filterable views open through [[src/app_execute/builtin_execution.rs#ScriptListApp#open_builtin_filterable_view]] and must declare a presentation mode: Mini (narrow 480px) or Full (wide 750px). Width is determined by layout, not by command importance.

- The helper takes an `expanded: bool` flag. `true` sets `MainWindowMode::Full` and resizes to `ViewType::ScriptList`; `false` sets `MainWindowMode::Mini` and resizes to `ViewType::MiniMainWindow`. Both branches live in the single helper body; no callers touch `resize_to_view_sync` directly.
- Deferred resize paths must preserve that same classification through [[src/app_impl/ui_window.rs#ScriptListApp#calculate_window_size_params]]. Triggered built-ins and filter changes cannot re-widen a Mini surface after the open helper has already selected Mini.
- Mini is the default for single-column filterable lists that do not render a right-pane preview or detail column: emoji picker, app launcher, window switcher, browser tabs, design gallery, favorites, process manager, settings, current-app commands, kit browse, installed kits, search AI presets, mini dictation.
- Full is reserved for views whose render layout splits a list pane and a preview/detail pane that justifies the extra width: clipboard history, file search, theme chooser, ACP history, browser history, dictation history, notes browse.
- MiniPrompt uses `ViewType::MiniPrompt`, not full ArgPrompt sizing, so Mini prompt render chrome and bounds stay aligned. ChatPrompt and embedded ACP use `compact_ai_view_type_for_mode`: Mini resolves to `MiniAiChat`, Full resolves to `DivPrompt`.
- A view that does not render a right-pane content column stays Mini even if its dataset is large — the preview pane is what earns the extra width, not row count or visual weight.
- The sizing contract is pinned by [[tests/source_audits/builtin_dispatch_consistency.rs#open_builtin_filterable_view_sets_shared_focus_contract]], [[tests/source_audits/builtin_dispatch_consistency.rs#deferred_sizing_keeps_mini_filterable_builtins_narrow]], [[tests/source_audits/builtin_dispatch_consistency.rs#deferred_sizing_keeps_preview_builtins_wide]], [[tests/current_app_commands.rs#current_app_commands_presentation_opens_mini_filterable_view]], and [[tests/trigger_builtin_current_app_commands_contract.rs#current_app_commands_trigger_builtin_deferred_resize_stays_mini]].

## Emoji picker activation pastes like clipboard history

The emoji picker's Enter keystroke and row click both write the selected emoji to the clipboard, hide the main window, and simulate a Cmd+V paste into the frontmost app — the same pattern clipboard history's paste action uses.

Both activation paths in [[src/render_builtins/emoji_picker.rs]] follow the clipboard-history contract: (a) `cx.write_to_clipboard(gpui::ClipboardItem::new_string(emoji))` writes synchronously via `NSPasteboard setData:forType:`, (b) `this.hide_main_and_reset(cx)` dismisses the launcher so the previously frontmost app regains focus, and (c) a spawned thread sleeps 100 ms and calls [[src/selected_text.rs#simulate_paste_with_cg]] to fire a Core Graphics Cmd+V. The 100 ms delay exists because the pasteboard write happens on the UI thread but the paste keystroke races the OS focus handoff — the shared helper lives at [[src/app_actions/handle_action/clipboard.rs#ScriptListApp#spawn_clipboard_paste_simulation]], which the emoji picker inlines rather than centralizing so the render-builtin stays self-contained. A prior implementation only called `cx.write_to_clipboard` and `close_and_reset_window`, which copied but never pasted — the user-visible regression was "Enter on an emoji leaves nothing inserted in the target app."

## Favorites Cmd+K opens a six-row actions menu

Cmd+K on the favorites built-in opens a six-row actions popup; existing inline U/J/D/Enter shortcuts continue to work alongside it.

The six rows are Run, Edit Script, Copy Script URL, Move Up, Move Down, and Remove from Favorites (bottom-separated). The actions menu duplicates the inline shortcuts, doesn't replace them.

Routing follows the shared actions-dialog host pattern: a dedicated `ActionsDialogHost::Favorites` variant lives in [[src/main_sections/app_view_state.rs]] alongside `ClipboardHistory`, `BuiltinList`, and the other host kinds, and exhaustive matches across the actions-dialog wiring (open, close, focus restoration) include the favorites arm. The Cmd+K key handler in [[src/render_builtins/favorites.rs]] mirrors the structure used by [[src/render_builtins/clipboard.rs]]'s Cmd+K branch. Each action row dispatches through `handle_action` with a stable string id (`favorites_run`, `favorites_edit_script`, `favorites_copy_script_url`, `favorites_move_up`, `favorites_move_down`, `favorites_remove`); ids do not change across renders so test fixtures bind to them safely. Closing the popup restores focus to the favorites filter input, matching the focus contract of the other built-in surfaces.

## Clipboard history Cmd+Enter attaches to AI

Plain Enter on a clipboard-history row copies + simulates Cmd+V paste; **Cmd+Enter** instead routes the row to AI/context attach via the existing `clipboard_attach_to_ai` action — the same handler Ctrl+Cmd+A invokes.

The Cmd+Enter branch lives in [[src/render_builtins/clipboard.rs]]'s key handler, ordered above the plain `is_key_enter(key)` arm so the modifier-guarded match wins. Reusing `clipboard_attach_to_ai` keeps the AI-attach contract single-sourced; plain Enter's copy+paste path is unchanged. This is the clipboard manifestation of the cross-surface "Cmd+Enter means AI everywhere" rule that also drives main launcher and file_search behavior. Dictation history's Copy Transcript row was rebound from `⌘↵` to `⌘C` in [[src/render_builtins/actions.rs]] so Cmd+Enter no longer collides with the AI verb on that surface.

## Emoji picker Frequently Used section is frecency-ranked

On empty search with no category pin, the picker renders a "Frequently Used" section above the category grid, ranked by an exponential-decay half-life score frozen at view-open time.

The ranking math and persistence live in [[src/emoji_usage.rs]]. Every commit (Enter or click) passes the chosen glyph through [[src/emoji_usage.rs#record_emoji_use]], which decays the existing `score` to "now" via `score * 0.5_f64.powf(age_secs / EMOJI_USAGE_HALF_LIFE_SECS)`, adds 1.0, and updates `last_used_at_ms` / `total_uses`. The half-life is 14 days (`EMOJI_USAGE_HALF_LIFE_SECS`) — long enough that habitual emoji stay put, short enough that experiments fade. Writes are atomic: temp-file then rename, so a crash mid-write can never leave a truncated JSON file.

The read path is the opposite direction: [[src/emoji_usage.rs#load_frequent_snapshot]] loads the store, ranks entries by `(decayed_score DESC, last_used_at_ms DESC, dataset_order ASC)`, and returns the top `EMOJI_FREQUENT_LIMIT` (two grid rows at current `GRID_COLS = 8`). The dataset-order tie-breaker is what keeps ordering deterministic between renders when two scores are equal — without it the head block could flicker between runs. [[src/app_execute/builtin_execution.rs]] primes `ScriptListApp.emoji_frequent_snapshot` with this result immediately before opening the view; render, navigate, and Enter all read from that frozen snapshot so mid-session usage writes never shift indices under the user.

Rendering, Up/Down navigation (in [[src/app_impl/startup.rs]]), and Enter (in [[src/render_builtins/emoji_picker.rs]]) all route through the shared [[src/emoji/mod.rs#display_ordered_emojis]] + [[src/emoji/mod.rs#build_display_grid_layout]] pair — this is the fix for the Oracle-flagged index-drift footgun. If only the renderer were reordered, arrow-key navigation and the Enter handler would each compute their own (legacy) order and commit the wrong emoji. The layout builder treats the leading `frequent_count` emojis as one synthetic cell block under a "Frequently Used" header row, then groups the remainder by category exactly like [[src/emoji/mod.rs#build_emoji_grid_layout]]. [[src/emoji/mod.rs#compute_display_scroll_row]] mirrors the single-step `compute_scroll_row` helper for the Left/Right arm so scroll-reveal stays correct even when the head block pushes category rows down.

Pins and frequency are intentionally separate: `~/.kenv/emoji-pins.json` is explicit user curation; `~/.kenv/emoji-usage.json` is passive behavioral state. They never merge — first-launch "Frequently Used" is empty until the first commit, and pinned emojis never seed the frequency store.
