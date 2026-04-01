# Chooser Consolidation Audit

Task: Map broader consolidation candidates for chooser-based system actions.

Date: 2026-03-31

## Status of Prior Work

### Notes Family (DONE)

The three duplicate notes entries were already removed from root search:

| ID | Status | Behavior |
|----|--------|----------|
| `builtin-notes` | **Kept** | Opens notes window |
| `builtin-quick-capture` | **Kept** | Opens quick capture (distinct) |
| `builtin-open-notes` | **Removed** | Was identical to `builtin-notes` |
| `builtin-new-note` | **Removed** | Was identical to `builtin-notes` |
| `builtin-search-notes` | **Removed** | Was identical to `builtin-notes` |

Regression tests exist at `src/builtins/mod.rs:2192-2204`.

Stale references in `NO_MAIN_WINDOW_BUILTINS` (`src/app_impl/execution_scripts.rs`) cleaned up in this cycle.

`NotesCommandType` enum still carries `OpenNotes`, `NewNote`, `SearchNotes` with `#[allow(dead_code)]`. The executor arm at `src/app_execute/builtin_execution.rs:1944-1946` still handles them (routed to `notes::open_notes_window`). These are safe to remove in a follow-up but are not user-visible today.

### Legacy AI & AppLauncher (documented, not user-visible)

See `docs/audits/legacy-command-audit.md`. These are compatibility-only enum variants and executor arms that are not registered in root search. Separate cleanup cycle.

---

## Consolidation Candidates

### 1. Volume Presets → Chooser

**Current state:** 6 separate root-search entries.

| ID | Label | `SystemActionType` |
|----|-------|--------------------|
| `builtin-volume-0` | Volume 0% | `Volume0` |
| `builtin-volume-25` | Volume 25% | `Volume25` |
| `builtin-volume-50` | Volume 50% | `Volume50` |
| `builtin-volume-75` | Volume 75% | `Volume75` |
| `builtin-volume-100` | Volume 100% | `Volume100` |
| `builtin-volume-mute` | Mute/Unmute Volume | `VolumeMute` |

**Consolidation plan:** Replace the 6 entries with a single `builtin-volume` entry ("Set Volume") that opens a chooser with the preset list. `VolumeMute` is the only behaviorally distinct entry (toggle vs set) and could remain as a separate root entry or appear as the first chooser option.

**Keyword retention:** The single entry must carry all current search keywords from the 6 entries: `volume`, `mute`, `unmute`, `sound`, `audio`, `0`, `25`, `50`, `75`, `100`, `percent`, `zero`, `off`, `low`, `quiet`, `half`, `medium`, `high`, `loud`, `max`, `full`, `toggle`. This ensures no search coverage loss.

**Risk:** Users who have memorized "volume 25" as a direct command will now need an extra selection step. Mitigate by matching the typed number in the chooser so `volume 25` pre-selects the 25% option.

**Unique behaviors today:** Each preset calls the same `set_volume` codepath with a different value. `VolumeMute` is a toggle. No behavior is lost.

### 2. System Settings Pages → Chooser

**Current state:** 8 separate root-search entries for individual settings panes, plus one top-level entry.

| ID | Label | `SystemActionType` |
|----|-------|--------------------|
| `builtin-system-preferences` | Open System Settings | `OpenSystemPreferences` |
| `builtin-privacy-settings` | Privacy & Security Settings | `OpenPrivacySettings` |
| `builtin-display-settings` | Display Settings | `OpenDisplaySettings` |
| `builtin-sound-settings` | Sound Settings | `OpenSoundSettings` |
| `builtin-network-settings` | Network Settings | `OpenNetworkSettings` |
| `builtin-keyboard-settings` | Keyboard Settings | `OpenKeyboardSettings` |
| `builtin-bluetooth-settings` | Bluetooth Settings | `OpenBluetoothSettings` |
| `builtin-notifications-settings` | Notification Settings | `OpenNotificationsSettings` |

**Consolidation plan:** Keep `builtin-system-preferences` as the single root entry ("System Settings"). When selected, open a chooser listing all 7 sub-pages plus a "General" option that opens the top-level pane. The top-level entry's existing keywords cover the generic case; each sub-page keyword set moves to the chooser filter.

**Keyword retention:** The root entry must carry merged keywords: `system`, `settings`, `preferences`, `prefs`, `privacy`, `security`, `display`, `monitor`, `screen`, `resolution`, `sound`, `audio`, `volume`, `network`, `wifi`, `ethernet`, `internet`, `keyboard`, `shortcuts`, `input`, `bluetooth`, `wireless`, `notifications`, `alerts`, `banners`. This preserves root-search reachability for all current keywords.

**Alternate discoverability:** Each sub-page also appears in the Actions dialog (⌘K) when "System Settings" is focused, matching the design principle that discovery lives in Actions.

**Risk:** Users who type "bluetooth" directly today get a single result and hit Enter. After consolidation they get "System Settings" and must select "Bluetooth" from the chooser. Mitigate by pre-filtering the chooser to show the matching sub-page when the original search term is specific enough.

**Unique behaviors today:** Each entry opens a distinct System Settings pane via `open -b com.apple.systempreferences <url>`. All are unique but follow the same pattern — they are parameterized variants of one action, not distinct features. No behavior is removed; all move into the chooser.

---

## Behavioral Grouping of Current Built-in Surface

### Exact Duplicates (already resolved)
- Notes family: 3 entries removed (Open Notes, New Note, Search Notes = identical to Notes)

### Distinct-but-Related (consolidation candidates)
- **Volume presets** (6 entries → 1 chooser): parameterized `set_volume` calls
- **System Settings pages** (8 entries → 1 chooser): parameterized `open settings pane` calls

### Distinct Behaviors (no consolidation recommended)
- All remaining entries map to unique execution paths
- `VolumeMute` is a toggle (not a preset) but fits naturally in the volume chooser
- `Open System Settings` (top-level) subsumes into the chooser as the default/first option

### Dead/Compatibility-Only (separate cleanup)
- `NotesCommandType::{OpenNotes, NewNote, SearchNotes}` — enum + executor arms, not registered
- `AiCommandType::{OpenAi, MiniAi, NewConversation, ClearConversation}` — not registered
- `BuiltInFeature::AppLauncher` — not registered, but deeply wired (see legacy-command-audit.md)

---

## Incomplete Source Context

The following files were referenced in the research but not fully inspected in this audit. Follow-up work should verify before implementing:

- `src/actions/builders/notes.rs` — may expose Notes sub-commands in Actions dialog already
- `src/actions/builders/chat.rs` — still exposes `Clear Conversation` action per legacy audit
- `src/menu_bar/current_app_commands.rs` — menu bar integration for system actions
- `src/notes/actions_panel.rs` — Notes-internal Actions panel (may already host New Note / Search Notes)
- `src/app_execute/builtin_execution.rs` (full file) — executor arms for all system actions, needed to verify volume/settings codepaths before chooser refactor
