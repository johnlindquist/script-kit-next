# Alfred Quick Notes and Snippets Research

Research date: 2026-02-01

Scope: Alfred's built-in Snippets and Text Expansion (Powerpack) plus Alfred Gallery workflows commonly used for quick notes.

## 1) Snippets and Text Expansion (built-in, Powerpack)

### Core features
- Snippets store frequently used text clips and can be expanded by keyword or browsed in the Snippets Viewer; Snippets and Text Expansion are Powerpack features. [S1]
- Snippets can be created from Preferences or from an existing Clipboard History item. [S3]
- Rich text snippets are supported; plain text is recommended when you want the pasted text to match destination formatting. [S1]
- Snippets Viewer access patterns:
  - Set a hotkey for the Snippets Viewer, or open Clipboard History and choose "All Snippets" to browse and filter by typing. [S1]
  - Use the `snip` keyword followed by a name or keyword to jump to a snippet. [S1]
- Snippet Collections:
  - Collections group snippets by type and can have an icon. [S2]
  - Collections can define a prefix/suffix (collection-wide affix) applied to snippet keywords. [S2]
- Sharing: Collections can be exported and shared, and snippets can be synced via Alfred preferences. [S1][S2]

### Auto-expansion (text expansion)
- Auto-expansion is disabled by default and enabled in Features > Snippets via "Automatically expand snippets by keyword". [S4]
- Auto-expansion requires Accessibility permissions and does not expand in secure text fields (e.g., password fields). [S4]
- A snippet needs a keyword (and the auto-expand checkbox enabled) for expansion. [S3][S4]

### Advanced expansion controls
- Advanced options include app exclusions, expanding mid-string, sound on expansion, and tweaking key event timing for reliability. [S1][S5]
- Troubleshooting guidance highlights that Alfred relies on Cmd+V for pasting and includes guidance to slow key events if expansion is unreliable. [S6]

### Dynamic placeholders and cursor placement
- Snippets can include `{date}`, `{time}`, `{datetime}`, `{clipboard}`, and `{cursor}` placeholders. [S1]
- Dynamic placeholders support formatting and date/time offsets. [S1][S7]
- `{snippet:keyword}` can embed another snippet; nested snippet placeholders are not recursively expanded. [S7]

### Snippet Triggers (workflow trigger object)
- Snippet Triggers run workflows without showing Alfred, triggered by a keyword you type in any text field. [S8]
- Triggers use a shared prefix (default `\\`) and optional suffix; you type prefix + keyword to run the trigger. [S8]
- Triggers can be limited to a specific app via bundle ID. [S8]
- If a Snippet Trigger keyword conflicts with a Snippet auto-expansion keyword, the workflow trigger takes precedence. [S8]

### Clipboard History and Universal Actions related to snippets
- Clipboard History is a Powerpack feature; it must be enabled in Preferences and requires Accessibility permissions. [S9]
- Clipboard History viewer lets you filter items by typing; use Cmd+S to save a clipboard item as a snippet. [S9]
- Universal Actions let you act on selected text, URLs, or files and include built-in actions like "save as a snippet". [S10]
- Universal Actions require Accessibility permissions; default hotkey is Cmd+/ and you can also invoke Actions from Alfred results (including Clipboard History). [S10]
- The Actions panel filters down to relevant actions based on the selected item type. [S10]

## 2) Quick notes in Alfred (workflow-driven)

Alfred's note-taking options are primarily represented as Alfred Gallery workflows that provide note-taking UIs, hotkeys, and Universal Actions. The workflows below are prominent examples. [S11][S12][S13][S14]

### Notes.app integration (official workflow)
Workflow: "Notes" (Alfred Team). [S11]
- `ns` searches Notes.app. [S11]
- `nn` creates a new note in Notes.app. [S11]
- `nl` opens the last modified note. [S11]
- Enter opens a note; Ctrl+Enter deletes; Cmd+Option+Ctrl+Enter forces a cache flush. [S11]
- Hotkeys can be configured for faster access. [S11]

### Save 'ur note (quick notes file)
Workflow: "Save 'ur note". [S12]
- `cqn` creates a one-line quick note. [S12]
- `cqp` opens a multi-line Text View; Enter opens the editor; Cmd+Enter saves. [S12]
- `vqn` views notes; `eqn` edits; `dqn` deletes. [S12]
- Supports Universal Actions to create a note from selection or Clipboard History. [S12]

### Note Taker (rich note management with Markdown)
Workflow: "Note Taker". [S13]
- Create: `nadd` prompts for a name, then opens the note for editing; supports Universal Action from selection or Clipboard History. [S13]
- Search: `nview` lists notes; actions include:
  - Enter to edit
  - Cmd+Enter to preview (Markdown parsed)
  - Option+Enter to copy contents
  - Ctrl+Enter to copy and delete
  - Cmd+Y to Quick Look. [S13]
- Edit: Alfred's edit view is stacked (Esc to go back; Cmd+Esc force hides). [S13]
- Save patterns: Cmd+Enter saves; Option+Enter saves and opens in external editor. [S13]

### Scratchpad (ephemeral scratch notes)
Workflow: "Scratchpad". [S14]
- Setup defines primary and secondary hotkeys, number of pads, file extension, and save location. [S14]
- Primary hotkey opens last-used pad; pressing it again or Esc dismisses without saving. [S14]
- Editing mode:
  - Cmd+Enter save
  - Shift+Enter Markdown preview
  - Option+Enter view/search all pads
  - Cmd+Shift+Enter cycle pads. [S14]
- Markdown mode:
  - Enter (or Shift+Enter) to edit
  - Option+Enter view/search all pads
  - Cmd+Shift+Enter cycle
  - Esc cancel or back. [S14]
- List mode:
  - Secondary hotkey or `pad` keyword to list pads
  - Enter opens pad
  - Cmd+L shows matched line in Large Type
  - Cmd+Y Quick Look. [S14]

## 3) UX patterns to reuse (cross-cutting)

### Entry points
- Keyword-first workflows: short keywords in Alfred's main bar to start note flows (`ns`, `nn`, `nl`, `nadd`, `nview`, `cqn`, `cqp`, `pad`). [S11][S12][S13][S14]
- Hotkey-first workflows: dedicated hotkeys to open a note UI directly (Scratchpad primary/secondary hotkeys). [S14]
- Contextual actions: Universal Actions on selections or clipboard items to create notes or snippets. [S10][S12][S13]

### Multi-step flows
- Stacked views: edit/preview flows that keep you inside Alfred and let Esc return to the list; Cmd+Esc to hide. [S13]
- Quick Look: Cmd+Y used for preview from list results. [S13][S14]
- Save/alternate actions:
  - Cmd+Enter for save
  - Option+Enter for alternate actions like "save and open" or "copy". [S12][S13][S14]

### Data storage and search
- Notes stored in Notes.app (official workflow) or local text/Markdown files (workflow-defined storage). [S11][S12][S13][S14]
- Search and filtering are list-based within Alfred, with item-level actions (open/edit/delete/copy). [S11][S13]

### Snippet/text expansion UX
- Optional keyword per snippet; auto-expansion is enabled globally and controlled per snippet. [S3][S4]
- Collection-wide affixes provide consistent prefix/suffix for large sets of snippets. [S2]
- Dynamic placeholders enable structured templates (dates, clipboard, cursor placement). [S1][S7]

---

Sources
[S1] Alfred Help: Snippets and Text Expansion - https://www.alfredapp.com/help/features/snippets/
[S2] Alfred Help: Snippet Collections - https://www.alfredapp.com/help/features/snippets/collections/
[S3] Alfred Help: Creating and Editing Snippets - https://www.alfredapp.com/help/features/snippets/editing-snippets/
[S4] Alfred Help: Setting Up Text Auto-Expansion for Snippets - https://www.alfredapp.com/help/features/snippets/auto-expansion/
[S5] Alfred Help: Advanced Snippet Expansion Preferences - https://www.alfredapp.com/help/features/snippets/advanced/
[S6] Alfred Help: Snippets and Text Expansion Troubleshooting - https://www.alfredapp.com/help/troubleshooting/snippets/
[S7] Alfred Help: Dynamic Placeholders - https://www.alfredapp.com/help/workflows/advanced/placeholders/
[S8] Alfred Help: Snippet Trigger - https://www.alfredapp.com/help/workflows/triggers/snippet/
[S9] Alfred Help: Clipboard History - https://www.alfredapp.com/help/features/clipboard/
[S10] Alfred Help: Universal Actions - https://www.alfredapp.com/help/features/universal-actions/
[S11] Alfred Gallery: Notes (Notes.app) - https://alfred.app/workflows/alfredapp/notes/
[S12] Alfred Gallery: Save 'ur note - https://alfred.app/workflows/stephenc/save-ur-note/
[S13] Alfred Gallery: Note Taker - https://alfred.app/workflows/vitor/note-taker/
[S14] Alfred Gallery: Scratchpad - https://alfred.app/workflows/zeitlings/scratchpad/
