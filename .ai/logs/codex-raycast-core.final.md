Changed files:
- `.ai/logs/codex-raycast-core.final.md` (new)

What I delivered:
- A structured markdown research report covering all requested categories:
1. Launcher/search (fuzzy search, calculator, file search, emoji picker)
2. Clipboard history
3. Snippets (including dynamic placeholders)
4. Window management
5. Quicklinks (including parameterized templates)
6. Script commands (including output modes + template languages)
7. Floating notes (Raycast Notes evolution)
8. System commands

How to test:
1. Run `sed -n '1,260p' .ai/logs/codex-raycast-core.final.md`
2. Confirm each section has `What it does`, `How it works`, and `UI it presents`
3. Confirm the source list is present at the bottom

Verification run:
- `markdownlint .ai/logs/codex-raycast-core.final.md` was attempted, but `markdownlint` is not installed in this environment.

Risks / known gaps:
- Some detailed command/action UX references come from Raycast’s `manual.raycast.com/windows/*` pages (used because they are more explicit); platform-specific shortcut labels may differ on macOS.
- Script-command language coverage is inferred from official template files in `raycast/script-commands` rather than a single explicit “supported languages” sentence in the manual.

Commits made:
- None.