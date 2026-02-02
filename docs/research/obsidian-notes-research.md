# Obsidian quick capture + daily notes research

Date: 2026-02-01

## Scope
This document summarizes Obsidian's quick-capture options, Daily Notes core plugin behavior, and hotkey-driven note workflows, with emphasis on how each integrates with the vault (file system) model. Sources are limited to official Obsidian Help docs plus community plugin docs for quick capture.

## Vault model (foundation for all capture paths)
- Obsidian stores notes as Markdown plain-text files inside a vault, which is a local folder (including subfolders). Changes made outside Obsidian are picked up because the vault refreshes automatically. A vault has an `.obsidian` configuration folder that stores vault-specific settings such as hotkeys, themes, and community plugins. citeturn1search0

## Daily Notes (core plugin)
- Daily Notes is a core plugin that opens today's note or creates it if it does not exist. You can open it from the ribbon, command palette, or by assigning a hotkey to the command. citeturn2search1
- By default, the daily note file name is the current date in `YYYY-MM-DD`. The plugin supports a configurable date format (including nested subfolders via format strings) and a configurable new-file location so daily notes live in a dedicated vault folder if desired. citeturn2search1
- Daily Notes can automatically insert a template when a daily note is created. Templates are a core plugin; you set a template folder and can use variables like `{{date}}`/`{{time}}` with Moment.js format strings. The Templates docs explicitly call out use inside Daily Notes. citeturn2search3turn2search1
- When the Daily Notes plugin is enabled and a note contains a `date` property, Obsidian will turn that date into a clickable link to the daily note for that day, further reinforcing daily notes as a vault-wide hub. citeturn2search1

## Quick capture options
### 1) Obsidian URI (official automation)
- Obsidian supports a custom `obsidian://` URI protocol to automate actions like creating notes (`new`), opening/creating the daily note (`daily`), and opening a specific note (`open`). This enables quick capture from external tools that can trigger a URL. citeturn3search1
- The `new` action can target a vault by name or ID, create a note by name or path, and optionally provide content (or clipboard), append to an existing file, overwrite, and run silently without opening the note. This is a direct, vault-aware capture API. citeturn3search1
- The `daily` action can create/open the daily note and accepts the same parameters as `new`, which makes it easy to wire a global hotkey or external capture tool straight into the vault's daily note. citeturn3search1

### 2) QuickAdd Capture (community plugin)
- QuickAdd's Capture choice is explicitly designed for quick capture without disrupting your current Obsidian window. It can capture to the active file or to a specified file name. citeturn2search0
- Capture targets can use a format syntax for dynamic file names (e.g., date-based). The docs show examples that map captures into daily-note-like files, which is a common quick-capture pattern. citeturn2search0
- Capturing to a folder is supported; QuickAdd will prompt for which file in that folder to capture to, keeping everything inside the vault file hierarchy. citeturn2search0

## Hotkey-driven note workflows
- Obsidian hotkeys are customizable keyboard shortcuts for commands and are managed in Settings > Hotkeys. The Command Palette shows existing hotkeys for commands, and multiple hotkeys can be assigned to the same command. citeturn1search1
- Practical hotkey patterns for quick capture: bind a hotkey to the Daily Notes command, or bind a hotkey to a QuickAdd Capture choice for append-style capture, or use an OS-level hotkey to call an `obsidian://new` or `obsidian://daily` URI for global capture into a vault. These options rely on the hotkey system, Daily Notes, QuickAdd, and the URI protocol described above. citeturn1search1turn2search1turn2search0turn3search1

## How this integrates with the vault system
- Daily Notes writes a normal Markdown file into the vault; folder location and file naming (date format) are configurable, so teams can enforce a predictable vault structure for journals/logs. citeturn2search1
- Templates are also stored inside the vault (in a configured templates folder) and are inserted into the active note, so Daily Notes + Templates is a purely file-based, vault-native workflow. citeturn2search3
- Obsidian URI actions are explicitly vault-addressable (by name or ID) and accept file paths that resolve within the vault, so capture can be routed to a specific file or folder in a specific vault from outside Obsidian. citeturn3search1
- QuickAdd Capture targets files/folders within the vault and can dynamically resolve file names via format syntax, which supports structured capture into vault subfolders or date-based file naming. citeturn2search0

## Key takeaways (for design/implementation)
- There is no single built-in "quick capture" button, but Obsidian provides a complete quick-capture toolkit via Daily Notes (core), Templates (core), Hotkeys (core), and Obsidian URI (official automation); community plugins like QuickAdd build on top of the same vault model for faster capture flows. citeturn2search1turn2search3turn1search1turn3search1turn2search0
- Because notes are plain Markdown files in a local folder, any quick-capture flow ultimately resolves to creating/appending a file in the vault, with `.obsidian` holding vault-specific configuration that influences those flows. citeturn1search0

## Sources consulted
- Obsidian Help: Daily Notes (core plugin). citeturn2search1
- Obsidian Help: Templates (core plugin). citeturn2search3
- Obsidian Help: Obsidian URI (automation and cross-app actions). citeturn3search1
- Obsidian Help: Hotkeys. citeturn1search1
- Obsidian Help: How Obsidian stores data (vault model). citeturn1search0
- QuickAdd documentation: Capture Choice (community plugin). citeturn2search0
