# Raycast Notes / Scratchpad Research

Research date: 2026-02-01
Scope: Built-in Raycast Notes (formerly Floating Notes). Excludes third-party extensions.

## Sources
[1] Raycast Manual - Notes: https://manual.raycast.com/notes
[2] Raycast Core Features - Notes: https://www.raycast.com/core-features/notes
[3] Raycast Blog - Meet the new Raycast Notes: https://www.raycast.com/blog/raycast-notes
[4] Raycast Changelog v1.85.0: https://www.raycast.com/changelog/macos/1-85-0
[5] Raycast Changelog v1.5.1 (Floating Notes): https://www.raycast.com/changelog/1-5-1
[6] Raycast Manual - Action Panel: https://manual.raycast.com/action-panel

## Product positioning and evolution
- Floating Notes launched in 2020 as a simple notes extension that stays on top of other windows, enabled in Preferences > Extensions, with shortcuts recommended for fast toggling. [5]
- Floating Notes was renamed to Raycast Notes in 2024, with an alias so searching "floating notes" still finds it, and the two old toggle commands were merged into a single Raycast Notes command. [3][4]
- Raycast markets Notes as a frictionless, lightweight, Markdown-capable, hotkey-first note tool, positioned for quick capture and "idea scratchpad" use cases. [1][2]

## Feature set (what the Notes window can do)
- Floating window above other apps; designed as a lightweight overlay that auto-sizes to content; cannot be used as a regular non-floating window. [2]
- Multiple notes with a lightweight UI that shows one note at a time; notes are organized as a stack. [1][4]
- Create note quickly: plus button in toolbar or Cmd+N inside Notes; Create Note command in root search; first line becomes the note title. [1]
- Search notes by title and content via Search Notes command; browse notes from the Notes window (Cmd+P). [1][4]
- Pin notes to keep them at the top of Search Notes/Browse Notes; pinned notes can be opened with Cmd+0..9. [1]
- Navigate note history with Cmd+[ and Cmd+]. [1]
- Markdown formatting with keyboard shortcuts, Action Panel actions, or a Format Bar at the bottom of the window. [1][4][6]
- Inline emoji picker by typing ':' and horizontal rules via --- or ___. [1][4]
- Export notes to plain text, Markdown, or HTML; share to Apple Notes and other apps. [2][3]
- Quicklinks can target a specific note and can be assigned hotkeys for instant access. [2][3]
- Integrations: AI Commands (e.g., Fix Spelling/Grammar), Snippets, and Quicklinks. [2][4]
- Cloud Sync across Macs; notes stored in an encrypted database; recover deleted notes within 60 days (free plan can restore but cannot preview deleted notes). [2]
- Menu bar entry point: open notes from the menu bar; Option-click to create a new note. [2]

## UX patterns Raycast uses for Notes
- Single-note focus with stack navigation: only one note visible at a time, with quick browse/search and back/forward history, keeping attention on a single context. [1]
- Floating window behavior: intentionally stays above other apps and is designed to feel like a scratchpad rather than a full document editor. [2][5]
- Command-first entry: Notes is exposed as three root-search commands (Raycast Notes / Create Note / Search Notes). Users are encouraged to set hotkeys for instant access. [1][3][4]
- Keyboard-first discoverability: Action Panel (Cmd+K) surfaces available actions and their shortcuts; formatting is available via shortcuts and a format bar. [1][6]
- Lightweight organization: pinning with number shortcuts (Cmd+0..9) mirrors pinned tab behavior; avoids heavy folder/tag UI. [1]
- Low-friction entry points: menu bar open and option-click to create a new note reduce mode switching. [2]

## Keyboard shortcuts

### Commands and entry points
- Raycast Notes command: toggles the Notes window; recommended hotkey is Opt+N (user-configurable). [1][3]
- Create Note command: creates a new note and opens the window. [1][3]
- Search Notes command: searches title and content. [1][4]
- Action Panel: Cmd+K to show more actions. [1][6]

### In-notes navigation and organization
- New note: Cmd+N. [1]
- Browse notes: Cmd+P. [1]
- Pin note: Shift+Cmd+P. [1]
- Open pinned notes: Cmd+0..9. [1]
- Back/forward between previously opened notes: Cmd+[ and Cmd+]. [1]

### Formatting shortcuts (Markdown helpers)
Paragraph formatting:
- Heading 1: Opt+Cmd+1. [1]
- Heading 2: Opt+Cmd+2. [1]
- Heading 3: Opt+Cmd+3. [1]
- Code block: Opt+Cmd+C. [1]
- Blockquote: Shift+Cmd+B. [1]
- Ordered list: Shift+Cmd+7. [1]
- Bullet list: Shift+Cmd+8. [1]
- Task list: Shift+Cmd+9. [1]

Text formatting:
- Bold: Cmd+B. [1]
- Italic: Cmd+I. [1]
- Strikethrough: Shift+Cmd+S. [1]
- Underline: Cmd+U. [1]
- Inline code: Cmd+E. [1]
- Link: Cmd+L. [1]

Other:
- Emoji picker: type ':' to open inline emoji picker. [1]
- Horizontal rule: type --- or ___ at the beginning of a new line. [1]

## Notes for Script Kit GPUI parity (optional takeaways)
- Notes is positioned as a fast, floating scratchpad with single-note focus and quick navigation rather than a full editor. [1][2]
- Command-driven access plus hotkey-first flow is core to the UX. [1][3]
- Pinning with number shortcuts and action-panel-driven actions are key discovery patterns. [1][6]
