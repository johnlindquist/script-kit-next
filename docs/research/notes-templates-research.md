# Notes Templates and Snippets Research
Date: 2026-02-01

## Scope
This note summarizes how popular note apps handle pre-defined templates, user-defined templates, snippet-style insertion, quick notes, and text expansion. Sources are official product docs where possible.

## Cross-app patterns
- Pre-defined templates are typically distributed via a template gallery or built-in template lists, and are applied when creating a new note or page rather than retrofitted onto existing notes. citeturn0search0turn0search4turn0search6
- User-defined templates often come from saving an existing note/page as a template, then reusing it from a templates panel or New menu. citeturn0search3turn6view0
- Snippet-style insertion is commonly implemented as an "insert template" action that drops pre-defined blocks into the current note (vs. creating a new note). citeturn0search1turn5view0
- Dynamic variables (date/time/title) are a common template feature in markdown-centric apps to avoid manual edits after insertion. citeturn0search1

## App-specific findings

### Notion
- Database templates are scoped to a specific database and are created from the New menu in that database; they define the content and properties for new pages created from the template. citeturn6view0
- Repeating database templates can automatically generate new pages on a schedule, which supports recurring note workflows. citeturn6view0
- Buttons can be configured to insert blocks or create/edit database pages, enabling snippet-like insertion or mini automations inside a page. citeturn5view0
- Public templates are often shared as public pages that can be duplicated into a workspace via the Duplicate control (including an option to duplicate as a template). citeturn4search1

### Evernote
- Evernote templates are pre-formatted notes; you can create a new note from a template or apply a template when starting a note. citeturn0search0
- Users can create their own templates by saving an existing note as a template, then reuse it from the Templates section. citeturn0search3
- Evernote provides a template gallery with ready-made templates that can be added to your account. citeturn0search0

### OneNote
- OneNote includes built-in page templates (layouts and content) that you apply when creating a new page. citeturn0search4turn0search6
- You can create and customize templates, but templates apply only to new pages (not to existing pages). citeturn0search4turn0search6

### Obsidian
- The core Templates plugin inserts pre-defined text into the current note; templates are stored in a designated folder and inserted on demand. citeturn0search1
- Template variables (such as date/time/title) are supported for quick personalization on insertion. citeturn0search1
- The community Templater plugin adds a richer templating language and JavaScript support for more advanced snippets. citeturn0search5

### Apple Notes (Quick Notes)
- Quick Notes lets users capture notes from any app context and stores them in the Notes app, prioritizing capture speed over template structure. citeturn1search0

## Text expansion and snippet systems

### OS-level text replacement
- macOS provides system text replacements where users define shortcut-to-phrase pairs; supported apps can automatically replace typed text with the configured replacements. citeturn3search0
- iOS provides text replacement in Keyboard settings with user-defined shortcuts that expand into longer phrases as you type. citeturn2search0

### Dedicated expansion tools
- TextExpander organizes snippets into groups and expands abbreviations into longer text (plain or rich) across many apps, with per-app control for expansion behavior. citeturn2search2

### App-level expansion (productivity suites)
- Microsoft Office AutoCorrect uses a shared replacement list across Office apps and lets users add custom replacements that substitute text while typing. citeturn3search1turn3search3

## Design implications for Script Kit (from patterns above)
- Templates often exist at two scopes: global (app-wide gallery) and local (database/folder). Consider both scopes if Script Kit supports templates or bundles. citeturn0search0turn6view0
- Snippet insertion is a distinct UX from "new note from template"; supporting both could map to different user intents (editing vs. creating). citeturn0search1turn5view0
- Date/time variables are a baseline expectation in template systems; a minimal variable set would align with common patterns. citeturn0search1
- Quick capture flows are optimized for speed and minimal UI, and can coexist with templates rather than replace them. citeturn1search0
