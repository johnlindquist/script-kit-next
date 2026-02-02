# Notes App Keyboard Shortcuts & Hotkeys (Research)

_Last updated: 2026-02-01_

## Scope
- Focused on desktop usage (macOS, Windows, web) for mainstream notes apps.
- Sources are official help docs where available; third-party sources are avoided unless necessary.
- Shortcuts may vary by keyboard layout and language; always verify against the app’s in-product shortcut list. [Apple Notes](https://support.apple.com/guide/notes/keyboard-shortcuts-and-gestures-apd46c25187e/mac), [Google Keep](https://support.google.com/keep/answer/12862970?co=GENIE.Platform%3DDesktop&hl=en)

## App snapshot (selected shortcuts)

### Apple Notes (macOS)
- New note: `Cmd+N`
- Quick Note: `Fn/Globe + Q`
- Search all notes: `Option+Cmd+F`
- Move focus between sidebar, notes list, and search field: `Tab`
Source: [Apple Notes keyboard shortcuts](https://support.apple.com/guide/notes/keyboard-shortcuts-and-gestures-apd46c25187e/mac)

### Apple Quick Note (macOS system feature)
- Invoke Quick Note from any app: hold `Fn/Globe`, press `Q`
- Alternative trigger: Hot corner (default bottom-right)
Source: [Create a Quick Note on Mac](https://support.apple.com/en-lamr/guide/notes/apdf028f7034/mac)

### Microsoft OneNote (Windows)
- Quick Note (global): `Win+Alt+N` (works even when OneNote isn’t running)
- Quick Note (app shortcut): `Ctrl+Shift+M` (also shown as `Alt+Windows+N`)
Source: [Create Quick Notes](https://support.microsoft.com/en-us/office/create-quick-notes-0f126c7d-1e62-483a-b027-9c31c78dad99), [OneNote keyboard shortcuts](https://support.microsoft.com/en-gb/office/keyboard-shortcuts-in-onenote-44b8b3f4-c274-4bcc-a089-e80fdcc87950)

### Evernote (Mac/Windows)
- Global new note window: `Ctrl+Alt+N` (Windows) / `Ctrl+Option+Cmd+N` (Mac)
- In-app new note: `Cmd+N` / `Ctrl+N`
- Search: `Cmd+K` / `Ctrl+K`
- Shortcuts drawer/help: `Cmd+/` / `Ctrl+/`
- Close window: `Cmd+W` / `Ctrl+W`
Source: [Evernote keyboard shortcuts](https://help.evernote.com/hc/en-us/articles/34296687388307-Keyboard-shortcuts)

### Google Keep (web/desktop)
- New note: `c`
- New list: `l`
- Search: `/`
- Next/previous note: `j` / `k`
- Open note: `Enter`
- Finish editing: `Esc`
- Shortcut help: `?`
Source: [Google Keep keyboard shortcuts (desktop)](https://support.google.com/keep/answer/12862970?co=GENIE.Platform%3DDesktop&hl=en)

### Notion
- New page: `Cmd/Ctrl+N`
- Search inside page: `Cmd/Ctrl+F`
- Search or jump to recently viewed page: `Cmd/Ctrl+P` or `Cmd/Ctrl+K`
- Back/forward: `Cmd/Ctrl+[` and `Cmd/Ctrl+]`
Source: [Notion keyboard shortcuts](https://www.notion.com/help/keyboard-shortcuts)

### Simplenote (Electron desktop)
- New note: `Ctrl+Shift+I`
- Search notes: `Ctrl+Shift+S`
- Show shortcuts: `Ctrl+/`
Source: [Simplenote help](https://simplenote.com/help/)

### Bear (macOS)
- End editing (return focus to navigation): `Cmd+Return`
- Search box: `Cmd+Shift+F`
- Navigate back/forward through recently viewed notes: `Cmd+Option+Left/Right`
- Open All Notes: `Cmd+1` (All Notes) / `Cmd+3` (ToDo section)
- Quick Open panel (search across notes/tags/sections): `Cmd+O`, then type + arrows + `Return`
Source: [Bear keyboard navigation tips](https://blog.bear.app/2018/08/bear-tips-you-can-navigate-bear-with-keyboard-shortcuts/), [Bear search + quick open](https://bear.app/faq/how-to-search-notes-in-bear/)

### Obsidian
- Command palette: `Cmd/Ctrl+P` (lists commands and any assigned hotkeys)
- Hotkeys are viewable and customizable in Settings → Hotkeys
Source: [Obsidian command palette](https://help.obsidian.md/plugins/command-palette), [Obsidian hotkeys](https://help.obsidian.md/hotkeys)

## Common patterns (invoke, dismiss, navigate)

### Invoke / capture
- **“New note/page” defaults to `Cmd/Ctrl+N`** in multiple apps (Apple Notes, Evernote, Notion). [Apple Notes](https://support.apple.com/guide/notes/keyboard-shortcuts-and-gestures-apd46c25187e/mac), [Evernote](https://help.evernote.com/hc/en-us/articles/34296687388307-Keyboard-shortcuts), [Notion](https://www.notion.com/help/keyboard-shortcuts)
- **Global quick-capture hotkeys use OS-level modifiers** (Apple Quick Note `Fn/Globe+Q`, OneNote `Win+Alt+N`, Evernote `Ctrl+Alt+N`). _Inference: these combos appear chosen to reduce collisions because they include system or less-common modifier sets._ [Apple Quick Note](https://support.apple.com/en-lamr/guide/notes/apdf028f7034/mac), [OneNote Quick Notes](https://support.microsoft.com/en-us/office/create-quick-notes-0f126c7d-1e62-483a-b027-9c31c78dad99), [Evernote](https://help.evernote.com/hc/en-us/articles/34296687388307-Keyboard-shortcuts)
- **Single-letter shortcuts for fast capture** appear in web-first notes (Google Keep uses `c`/`l`). [Google Keep](https://support.google.com/keep/answer/12862970?co=GENIE.Platform%3DDesktop&hl=en)

### Dismiss / exit
- **`Esc` to finish editing** is a common escape hatch (Google Keep). [Google Keep](https://support.google.com/keep/answer/12862970?co=GENIE.Platform%3DDesktop&hl=en)
- **Close-window shortcuts (`Cmd/Ctrl+W`)** are used to dismiss quick-note windows (Evernote). [Evernote](https://help.evernote.com/hc/en-us/articles/34296687388307-Keyboard-shortcuts)
- **Dedicated “end editing”** to return focus to navigation (Bear `Cmd+Return`). [Bear tips](https://blog.bear.app/2018/08/bear-tips-you-can-navigate-bear-with-keyboard-shortcuts/)

### Navigate / find
- **Search-first navigation is universal**:
  - Apple Notes: `Option+Cmd+F` (search all notes)
  - Google Keep: `/`
  - Simplenote: `Ctrl+Shift+S`
  - Notion: `Cmd/Ctrl+P` or `Cmd/Ctrl+K` (search/jump)
  - OneNote: search across notebooks (Quick Notes page references Quick Notes organization; keyboard shortcut list includes `Ctrl+Shift+M` and other navigation). [Apple Notes](https://support.apple.com/guide/notes/keyboard-shortcuts-and-gestures-apd46c25187e/mac), [Google Keep](https://support.google.com/keep/answer/12862970?co=GENIE.Platform%3DDesktop&hl=en), [Simplenote](https://simplenote.com/help/), [Notion](https://www.notion.com/help/keyboard-shortcuts), [OneNote shortcuts](https://support.microsoft.com/en-gb/office/keyboard-shortcuts-in-onenote-44b8b3f4-c274-4bcc-a089-e80fdcc87950)
- **List navigation with `j/k`** appears in Keep for moving between notes. [Google Keep](https://support.google.com/keep/answer/12862970?co=GENIE.Platform%3DDesktop&hl=en)
- **Pane focus switching** is supported in Apple Notes (Tab cycles sidebar/list/search field). [Apple Notes](https://support.apple.com/guide/notes/keyboard-shortcuts-and-gestures-apd46c25187e/mac)
- **Back/forward through recent notes/pages** is supported in Bear (`Cmd+Option+Left/Right`) and Notion (`Cmd/Ctrl+[` and `Cmd/Ctrl+]`). [Bear tips](https://blog.bear.app/2018/08/bear-tips-you-can-navigate-bear-with-keyboard-shortcuts/), [Notion](https://www.notion.com/help/keyboard-shortcuts)

### Discoverability & customization
- **Shortcut help overlays/drawers** are common: Evernote (`Cmd/Ctrl+/`), Simplenote (`Ctrl+/`), Google Keep (`?`). [Evernote](https://help.evernote.com/hc/en-us/articles/34296687388307-Keyboard-shortcuts), [Simplenote](https://simplenote.com/help/), [Google Keep](https://support.google.com/keep/answer/12862970?co=GENIE.Platform%3DDesktop&hl=en)
- **Customizable hotkeys** are explicitly supported in Obsidian (Settings → Hotkeys) and Evernote (shortcut drawer lets you edit/disable). [Obsidian hotkeys](https://help.obsidian.md/hotkeys), [Evernote](https://help.evernote.com/hc/en-us/articles/34296687388307-Keyboard-shortcuts)

## Design takeaways (inferred from the patterns above)
- Provide two entry points: a **standard in-app “New note” (`Cmd/Ctrl+N`)** and a **global quick-capture hotkey** that uses OS-level modifiers to reduce conflicts (Fn/Globe, Win+Alt, Ctrl+Alt). [Apple Notes](https://support.apple.com/guide/notes/keyboard-shortcuts-and-gestures-apd46c25187e/mac), [Apple Quick Note](https://support.apple.com/en-lamr/guide/notes/apdf028f7034/mac), [OneNote Quick Notes](https://support.microsoft.com/en-us/office/create-quick-notes-0f126c7d-1e62-483a-b027-9c31c78dad99), [Evernote](https://help.evernote.com/hc/en-us/articles/34296687388307-Keyboard-shortcuts)
- Make **search/jump the primary navigation affordance**, with a quick switcher shortcut (`Cmd/Ctrl+P` or `Cmd/Ctrl+K`) or a single-key search (`/`). [Notion](https://www.notion.com/help/keyboard-shortcuts), [Google Keep](https://support.google.com/keep/answer/12862970?co=GENIE.Platform%3DDesktop&hl=en)
- Provide an **always-available escape** (e.g., `Esc`) to exit editing and return focus to the list or previous context. [Google Keep](https://support.google.com/keep/answer/12862970?co=GENIE.Platform%3DDesktop&hl=en)
- Include a **built-in shortcut list** for discoverability and onboarding. [Evernote](https://help.evernote.com/hc/en-us/articles/34296687388307-Keyboard-shortcuts), [Simplenote](https://simplenote.com/help/), [Google Keep](https://support.google.com/keep/answer/12862970?co=GENIE.Platform%3DDesktop&hl=en)

