# Notes Window Improvements - Research Consolidation

## Executive Summary

This document consolidates research from 15 parallel agents analyzing notes window improvements based on Raycast, Alfred, Apple Notes, Obsidian, Notion, and other productivity tools.

---

## Current Implementation Status

### Already Implemented
- SQLite + FTS5 full-text search with WAL mode
- Auto-resizing floating window with vibrancy
- Cmd+K actions panel (Raycast-style)
- Cmd+P note switcher
- Soft delete/trash with 60-day recovery
- Export (txt, md, html)
- Deeplinks (`scriptkit://notes/{id}`)
- 300ms debounced auto-save
- `is_pinned` field on notes
- Window bounds persistence per-display

### Known Gaps
- Quick Capture (incomplete)
- Cmd+0-9 quick access to pinned notes
- Configurable Escape key behavior
- Inline hashtag tagging
- Note history navigation (Cmd+[ / Cmd+])
- Always-on-top pin toggle
- Note color coding

---

## Prioritized Recommendations

### Tier 1: High Impact, High Feasibility (Implement First)

#### 1. Pinned Notes with Cmd+0-9 Quick Access
**Source**: Raycast Notes
**Impact**: High - Power users can instantly jump to frequently used notes
**Effort**: Low - `is_pinned` already exists, just need keyboard handling

**Implementation**:
- Cmd+1 through Cmd+9 opens first 9 pinned notes
- Cmd+0 opens the 10th pinned note
- Show pin order in note switcher with numbered badges
- Add "Pin to Slot 1-9" option in actions panel

#### 2. Configurable Escape Key Behavior
**Source**: Raycast v1.84.0
**Impact**: High - Accommodates different user preferences
**Effort**: Low - Single preference toggle

**Options**:
- **Close window** (default): Escape dismisses the notes window entirely
- **Unfocus window**: Escape removes focus but keeps window visible for reference

**Implementation**:
- Add `escape_behavior` setting: `"close"` | `"unfocus"`
- Store in notes preferences
- Add toggle in Cmd+K actions panel

#### 3. Note History Navigation (Cmd+[ / Cmd+])
**Source**: Raycast, Notion, Browser conventions
**Impact**: High - Natural navigation through recently viewed notes
**Effort**: Low - Track note view history

**Implementation**:
- Maintain stack of recently viewed note IDs
- Cmd+[ navigates backward in history
- Cmd+] navigates forward in history
- Limit history to 50 entries

---

### Tier 2: High Impact, Medium Feasibility

#### 4. Inline Hashtag Tagging
**Source**: Apple Notes, Bear, Obsidian
**Impact**: Very High - Enables powerful organization
**Effort**: Medium - Requires parser and UI changes

**Implementation**:
- Parse `#tagname` in note content
- Activate tag on space/enter after hashtag
- Visual styling (colored pill)
- Autocomplete dropdown with existing tags
- Tag sidebar in note switcher
- Filter notes by tag

#### 5. Complete Quick Capture
**Source**: Apple Quick Note, Drafts, Things 3
**Impact**: High - Core feature for quick note-taking
**Effort**: Medium - Scaffolding exists

**Implementation**:
- Global hotkey (Option+N recommended)
- Capture clipboard contents option
- Append to daily note or specific note
- Create new note with content
- Auto-dismiss after capture

#### 6. Always-on-Top Pin Toggle
**Source**: macOS Stickies, Windows Sticky Notes, Notezilla
**Impact**: Medium - Keeps notes visible during work
**Effort**: Low - Window level change

**Implementation**:
- Toggle in titlebar (pin icon)
- Keyboard shortcut: Cmd+Option+F (matches Stickies)
- Visual indicator when pinned
- Persist state across sessions

---

### Tier 3: Medium Impact, High Feasibility

#### 7. Note Color Coding
**Source**: macOS Stickies (6 colors), Notezilla
**Impact**: Medium - Visual organization
**Effort**: Low

**Colors** (Stickies palette):
- Yellow (default), Blue, Green, Purple, Pink, Gray
- Cmd+1-6 to set color (when not in text field)
- Color picker in actions panel

#### 8. Collapse to Title Bar
**Source**: macOS Stickies
**Impact**: Medium - Reduces clutter while keeping notes accessible
**Effort**: Low

**Implementation**:
- Double-click title bar to collapse/expand
- Show only title when collapsed
- Keyboard shortcut: Cmd+M or custom

#### 9. Vim-Style Navigation (j/k)
**Source**: Gmail, Twitter, GitHub
**Impact**: Medium - Power user efficiency
**Effort**: Low

**Implementation**:
- j/k for up/down in note list (when not in editor)
- gg/G for first/last note
- Enter to open selected note

---

### Tier 4: Nice to Have (Future)

#### 10. Template Variables
**Source**: Obsidian QuickAdd
Variables: `{{date}}`, `{{time}}`, `{{title}}`, `{{clipboard}}`

#### 11. Export to Apple Notes
**Source**: Raycast Notes
Direct integration with macOS Notes.app

#### 12. Clipboard Merging
**Source**: Alfred
Cmd+C+C to append to clipboard instead of replace

#### 13. Smart Folders by Tag
**Source**: Apple Notes
Save tag filter combinations as virtual folders

#### 14. Sync via iCloud/Dropbox
**Source**: File-based sync research
Export notes as Markdown to synced folder

---

## Keyboard Shortcuts Summary

### Current
| Shortcut | Action |
|----------|--------|
| Cmd+K | Open actions panel |
| Cmd+P | Open note switcher |
| Cmd+N | Create new note |
| Cmd+D | Duplicate note |
| Cmd+W | Close notes window |
| Cmd+B | Bold formatting |
| Cmd+I | Italic formatting |

### Recommended Additions
| Shortcut | Action |
|----------|--------|
| Cmd+0-9 | Open pinned notes 1-10 |
| Cmd+[ | Navigate back in history |
| Cmd+] | Navigate forward in history |
| Cmd+Option+F | Toggle always-on-top |
| Escape | Close or unfocus (configurable) |
| Shift+Cmd+P | Pin/unpin current note |

---

## Implementation Priority

### Phase 1 (Implement Now - High Value, Low Effort) - COMPLETED

1. **Cmd+0-9 pinned note access** - IMPLEMENTED
   - Cmd+1-9 opens first 9 pinned notes
   - Cmd+0 opens the 10th pinned note
   - Notes are sorted by sort_order, then updated_at

2. **Configurable Escape behavior** - IMPLEMENTED
   - Toggle via Cmd+K actions panel > Settings > "Toggle Escape Behavior"
   - "close" mode (default): Escape closes the notes window
   - "unfocus" mode: Escape removes focus but keeps window visible
   - Setting persists per-session

3. **Cmd+[ / Cmd+] history navigation** - IMPLEMENTED
   - Cmd+[ navigates backward through recently viewed notes
   - Cmd+] navigates forward
   - History limited to 50 entries
   - Deleted notes are automatically skipped

### Phase 2 (Next Sprint)
4. Inline hashtag tagging
5. Complete quick capture
6. Always-on-top toggle

### Phase 3 (Future)
7. Note color coding
8. Collapse to title bar
9. Vim-style navigation
10. Template variables

---

## Research Sources

All research documents are available in:
`/private/tmp/claude/-Users-johnlindquist-dev-script-kit-gpui/9d9abb45-ffd5-434d-97b1-375833ed1274/scratchpad/`

- `raycast-notes-research.md`
- `alfred-notes-research.md`
- `apple-notes-research.md`
- `floating-notes-ux-research.md`
- `keyboard-shortcuts-research.md`
- `notion-notes-research.md`
- `obsidian-notes-research.md`
- `persistence-research.md`
- `code-snippet-research.md`
- `tagging-notes-research.md`
- `clipboard-notes-research.md`
- `selection-capture-research.md`
- `markdown-notes-research.md`
- `window-management-research.md`
