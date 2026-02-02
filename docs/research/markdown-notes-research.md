# Markdown Support in Quick Notes Apps

Date: 2026-02-01
Scope: Quick notes apps with Markdown support. Focus on rendering, syntax highlighting, and preview UX.

## Cross-app UX patterns (from documented behavior)
- Mode switching: Many apps separate "editing" from "reading/preview" via a view switcher or a toggle.
- Live preview: Some apps render Markdown inline and hide most syntax until the cursor enters that region.
- Split view: A minority expose a two-pane editor with raw Markdown on the left and rendered HTML on the right.
- Minimal UX: A common minimal UX pattern is "hide Markdown characters" so the text reads cleanly during editing.
- Syntax highlighting is often a separate subsystem from Markdown rendering (editor highlight vs. preview renderer).

## App snapshots (documented behavior)

### Obsidian
- Rendering/preview: Reading view shows a note without Markdown syntax. Editing view offers two modes: Live Preview (formatted inline, hides most syntax, reveals syntax when cursor enters) and Source mode (shows all Markdown). You can open editing and reading views side-by-side. (Source: https://help.obsidian.md/edit-and-read)
- Syntax highlighting: Not explicitly documented in the help page above.

### Typora
- Rendering/preview: Live Preview renders inline; inline tags are hidden or displayed smartly, and block-level tags are hidden after the block is rendered. Uses GitHub Flavored Markdown. (Source: https://support.typora.io/Quick-Start/)
- Syntax highlighting: Not explicitly documented in Quick Start.

### Bear
- Rendering/preview: Markdown characters can be shown or hidden via a "Hide Markdown" toggle in Settings > General, which gives a cleaner reading/editing experience while still using Markdown. (Source: https://bear.app/faq/how-to-use-markdown-in-bear/)
- Syntax highlighting: Not explicitly documented in the FAQ above.

### Simplenote
- Rendering/preview: Markdown is supported across web, Android, Electron, and iOS. Markdown can be enabled per note. There is a "Toggle Markdown preview" shortcut on desktop, and iOS supports swipe-left for preview and swipe-right to edit. (Source: https://simplenote.com/help/)
- Syntax highlighting: Not documented in the help page above.

### Drafts
- Rendering/preview: Drafts ships with Markdown parsers (MultiMarkdown, GitHub Flavored Markdown) to convert Markdown to HTML for previews/output. Markdown settings affect preview output but not editor highlighting. (Source: https://docs.getdrafts.com/docs/settings/markdown.html)
- Syntax highlighting: Drafts ships with Markdown syntax highlighting in the editor; it is separate from preview parsing. (Source: https://docs.getdrafts.com/docs/settings/markdown.html)

### Joplin
- Rendering/preview: Joplin has two editor types: a two-pane Markdown editor (left editor pane with Markdown text, right viewer pane with rendered HTML) and a WYSIWYG rich text editor. The two-pane Markdown editor keeps editor and viewer panes in sync while scrolling. (Sources: https://joplinapp.org/help/dev/spec/sync_scroll/)
- Rendering/preview (rich text): Notes are stored as Markdown under the hood; the rich text editor provides WYSIWYG editing but is constrained by Markdown limitations. (Source: https://joplinapp.org/help/apps/rich_text_editor)
- Syntax highlighting: Not explicitly documented in the pages above.

### Standard Notes
- Rendering/preview: Markdown is available via editor extensions rather than the default editor. (Source: https://standardnotes.com/help/18/how-do-i-use-markdown-in-my-notes)
- Syntax highlighting/preview (extension examples):
  - "Markdown Basic" uses Markdown-It rendering, syntax highlighting via Highlight.js, and optional split-pane view. (Source: https://www.npmjs.com/package/sn-simple-markdown-editor)
  - "Rich Markdown Editor" persists content as Markdown and includes code sections with syntax highlighting. (Source: https://github.com/arturolinares/sn-rme)
  - "Code Pro" (Monaco-based) provides syntax highlighting for Markdown and other languages. (Source: https://github.com/standardnotes/code-pro)

### Notesnook
- Rendering/preview: Notesnook supports Markdown shortcuts in the rich-text editor but explicitly does not support raw Markdown editing. The shortcuts apply formatting rather than showing Markdown source. (Source: https://help.notesnook.com/rich-text-editor/markdown-notes-editing)
- Import compatibility: Notesnook’s importer supports CommonMark, GitHub-flavored Markdown, and Obsidian-flavored Markdown when importing .md files. (Source: https://help.notesnook.com/importing-notes/import-notes-from-markdown-files)
- Syntax highlighting: Not documented in the pages above.

## Takeaways for a minimal Markdown editor UX
- Offer three tiers of editing experience:
  1) Source mode (raw Markdown),
  2) Live preview (inline rendering, hide syntax), and
  3) Optional split view (source + rendered preview).
- Keep preview toggles discoverable (toolbar button, status-bar toggle, or keyboard shortcut). Mobile apps often use swipe gestures for preview.
- Separate concerns: editor syntax highlighting can be independent of the Markdown parser used for preview/export.
- Provide a "hide Markdown characters" option for minimal, low-distraction editing.
- Consider import/export compatibility (CommonMark/GFM) even if the editor itself is rich-text-first.

## Sources
- Obsidian views and editing modes: https://help.obsidian.md/edit-and-read
- Typora live preview: https://support.typora.io/Quick-Start/
- Bear Markdown FAQ: https://bear.app/faq/how-to-use-markdown-in-bear/
- Simplenote Markdown help: https://simplenote.com/help/
- Drafts Markdown settings: https://docs.getdrafts.com/docs/settings/markdown.html
- Joplin rich text editor: https://joplinapp.org/help/apps/rich_text_editor
- Joplin sync scroll spec (two-pane editor): https://joplinapp.org/help/dev/spec/sync_scroll/
- Standard Notes Markdown help: https://standardnotes.com/help/18/how-do-i-use-markdown-in-my-notes
- Standard Notes Markdown Basic extension: https://www.npmjs.com/package/sn-simple-markdown-editor
- Standard Notes Rich Markdown Editor: https://github.com/arturolinares/sn-rme
- Standard Notes Code Pro editor: https://github.com/standardnotes/code-pro
- Notesnook Markdown shortcuts: https://help.notesnook.com/rich-text-editor/markdown-notes-editing
- Notesnook Markdown import formats: https://help.notesnook.com/importing-notes/import-notes-from-markdown-files
