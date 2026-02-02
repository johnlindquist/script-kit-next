# Notes Window Improvements - Consolidated Recommendations

**Date:** 2026-01-31
**Research Sources:** 15 parallel research agents analyzing competitors, UX patterns, and codebase

---

## Executive Summary

Based on comprehensive research of Raycast Notes, Apple Quick Note, Notion, Alfred, Obsidian, and 10+ other notes apps, combined with codebase analysis, we've identified **critical gaps** in the current Script Kit Notes implementation and prioritized improvements by impact.

---

## Current State (from Codebase Analysis)

### What's Working
- GPUI PopUp window with Raycast-style single-note view
- Multi-line editor with 300ms autosave debounce
- SQLite/FTS5 storage with WAL mode
- Cmd+K command bar actions
- Cmd+P note switcher via CommandBar
- Export to text/markdown/HTML
- Deep linking via `scriptkit://notes/{id}`
- Window bounds persistence per display
- Auto-sizing based on content

### Critical Gaps
| Gap | Status | Impact |
|-----|--------|--------|
| Search UI not wired to main UI | FTS5 ready, UI stubbed | HIGH |
| Quick capture doesn't create new note | TODO in code | HIGH |
| Formatting shortcuts not functional | Stubbed but not applied | HIGH |
| Markdown preview not implemented | Mentioned in docs | MEDIUM |
| Pin management not in new UI | Only in legacy panel | MEDIUM |

---

## Top 10 Recommended Improvements

### Tier 1: Critical (Implement First)

#### 1. Wire Up Search UI to Notes Window
**Impact:** HIGH | **Effort:** LOW-MEDIUM

Every major notes app prioritizes search:
- Raycast: Cmd+P browse notes, Search Notes command
- Apple Notes: Option+Cmd+F search all
- Notion: Cmd+P / Cmd+K jump to page
- Google Keep: `/` for search

**Current state:** `render_search` and `search_state` exist but aren't in the render tree. FTS5 is ready.

**Recommendation:** Add a search bar that appears on Cmd+F or when typing `/`, filtering notes via the existing FTS5 implementation.

---

#### 2. Fix Quick Capture to Create New Note
**Impact:** HIGH | **Effort:** LOW

This is the #1 use case for quick notes:
- Apple Quick Note: Fn+Q creates new note instantly
- Raycast: Option-click menu bar = new note
- OneNote: Win+Alt+N global capture
- Notion: Cmd+N new page

**Current state:** `quick_capture` opens Notes but doesn't create a new note (TODO in code).

**Recommendation:** When `quick_capture` is triggered:
1. Create a new empty note
2. Focus the editor
3. Position cursor at start
4. Optionally pre-fill from clipboard if flag is set

---

#### 3. Implement Markdown Formatting Shortcuts
**Impact:** HIGH | **Effort:** MEDIUM

Raycast Notes has comprehensive shortcuts that users expect:

| Action | Raycast Shortcut | Script Kit Status |
|--------|-----------------|-------------------|
| Heading 1 | Opt+Cmd+1 | Not implemented |
| Heading 2 | Opt+Cmd+2 | Not implemented |
| Heading 3 | Opt+Cmd+3 | Not implemented |
| Bold | Cmd+B | Not implemented |
| Italic | Cmd+I | Not implemented |
| Code block | Opt+Cmd+C | Not implemented |
| Bullet list | Shift+Cmd+8 | Not implemented |
| Task list | Shift+Cmd+9 | Not implemented |
| Inline code | Cmd+E | Not implemented |
| Link | Cmd+L | Not implemented |

**Current state:** `insert_formatting` builds formatted strings but doesn't apply them to editor content.

**Recommendation:**
1. Get cursor position from editor
2. Insert markdown syntax at cursor (or wrap selection)
3. Reposition cursor appropriately

---

### Tier 2: High Value

#### 4. Add Keyboard Shortcut Help Overlay
**Impact:** MEDIUM | **Effort:** LOW

Common pattern across apps:
- Google Keep: `?`
- Evernote: Cmd+/
- Simplenote: Ctrl+/

**Recommendation:** Add a `?` or `Cmd+/` shortcut that shows a modal with all available shortcuts. This dramatically improves discoverability.

---

#### 5. Pinned Notes Quick Access (Cmd+0-9)
**Impact:** MEDIUM | **Effort:** MEDIUM

Raycast pattern: Cmd+0 through Cmd+9 opens first 10 pinned notes.

**Current state:** Pinning exists in data model and legacy panel, but quick access shortcuts not implemented.

**Recommendation:**
1. Track pinned note order (already in sort_order)
2. Map Cmd+0-9 to pinned notes
3. Show pin numbers in note switcher

---

#### 6. Esc to Dismiss / Navigate Back
**Impact:** MEDIUM | **Effort:** LOW

Universal pattern:
- Google Keep: Esc finishes editing
- Bear: Cmd+Return ends editing
- GNOME HIG: Esc should always cancel/dismiss

**Recommendation:**
- Single Esc: Close any open panel (search, actions)
- Double Esc: Close notes window (or configurable)

---

### Tier 3: Polish

#### 7. Note History Navigation (Cmd+[ and Cmd+])
**Impact:** MEDIUM | **Effort:** MEDIUM

Raycast and Notion pattern for back/forward through recently viewed notes.

**Recommendation:** Maintain a note history stack and navigate with Cmd+[ / Cmd+].

---

#### 8. Inline Emoji Picker
**Impact:** LOW | **Effort:** LOW

Raycast pattern: typing `:` opens emoji picker.

**Recommendation:** Detect `:` at start of word and show emoji completion.

---

#### 9. Markdown Live Preview Toggle
**Impact:** MEDIUM | **Effort:** HIGH

Apps offer different preview modes:
- Obsidian: Reading view, Live Preview, Source
- Typora: Inline live preview
- Bear: Hide Markdown toggle

**Recommendation:** Start with a simple toggle to hide/show markdown syntax.

---

#### 10. Text Capture from Selection
**Impact:** MEDIUM | **Effort:** HIGH

PopClip/Alfred Universal Actions pattern:
- Global hotkey captures selected text
- Opens notes with text pre-filled

**Recommendation:**
1. Use Accessibility API to get selected text
2. Trigger via global hotkey (e.g., Cmd+Shift+N)
3. Open notes with captured text

---

## Implementation Priority Matrix

```
                    LOW EFFORT ─────────────────────► HIGH EFFORT
    │
    │  ┌─────────────────┐   ┌─────────────────┐
H   │  │ 2. Quick Capture│   │ 3. Formatting   │
I   │  │ 4. Shortcut Help│   │    Shortcuts    │
G   │  │ 6. Esc Dismiss  │   │ 1. Wire Search  │
H   │  └─────────────────┘   └─────────────────┘
    │
I   │  ┌─────────────────┐   ┌─────────────────┐
M   │  │ 8. Emoji Picker │   │ 5. Pinned Notes │
P   │  │                 │   │ 7. Note History │
A   │  └─────────────────┘   └─────────────────┘
C   │
T   │  ┌─────────────────┐   ┌─────────────────┐
    │  │                 │   │ 9. MD Preview   │
L   │  │                 │   │ 10. Text Capture│
O   │  └─────────────────┘   └─────────────────┘
W   │
    ▼
```

---

## Top 3 for Immediate Implementation

Based on impact vs effort analysis:

### 1. Fix Quick Capture (HIGH impact, LOW effort)
- Change `quick_capture` to create new note + focus editor
- Estimated: ~30 lines of code

### 2. Wire Up Search UI (HIGH impact, MEDIUM effort)
- Add search bar to render tree
- Connect to existing FTS5 queries
- Filter notes list in real-time
- Estimated: ~100-150 lines of code

### 3. Implement Formatting Shortcuts (HIGH impact, MEDIUM effort)
- Handle keyboard events for Cmd+B/I/U, Opt+Cmd+1-3
- Apply markdown syntax at cursor position
- Estimated: ~150-200 lines of code

---

## Competitor Feature Comparison

| Feature | Raycast | Apple Notes | Notion | Script Kit |
|---------|---------|-------------|--------|------------|
| Quick capture | ✅ | ✅ Fn+Q | ✅ Cmd+N | ❌ Broken |
| Search | ✅ Cmd+P | ✅ Opt+Cmd+F | ✅ Cmd+P | ⚠️ Stubbed |
| Formatting | ✅ Full | ✅ Basic | ✅ Full | ❌ Stubbed |
| Pinned notes | ✅ Cmd+0-9 | ✅ Pin gesture | ✅ Favorites | ⚠️ Legacy only |
| Shortcut help | ✅ | ❌ | ✅ | ❌ |
| Markdown preview | ❌ | ❌ | ✅ | ❌ |
| Cloud sync | ✅ | ✅ iCloud | ✅ | ❌ |
| Templates | ❌ | ❌ | ✅ | ❌ |

---

## Design Principles (from Research)

1. **Single-note focus** - Show one note at a time (Raycast pattern)
2. **Command-first entry** - Expose as commands with hotkeys
3. **Keyboard-first** - Every action should be keyboard accessible
4. **Lightweight organization** - Pinning over heavy folders/tags
5. **Low-friction capture** - Minimize steps from trigger to typing
6. **Search as navigation** - Search is the primary way to find notes

---

## References

All research documents are in `/docs/research/`:
- `notes-window-codebase-analysis.md` - Current implementation
- `raycast-notes-research.md` - Raycast patterns
- `apple-notes-research.md` - Apple Quick Note
- `notion-notes-research.md` - Notion capture
- `alfred-notes-research.md` - Alfred snippets
- `obsidian-notes-research.md` - Obsidian capture
- `floating-notes-ux.md` - Window behavior
- `notes-keyboard-shortcuts.md` - Shortcut patterns
- `markdown-notes-research.md` - Editor patterns
- `notes-sync-research.md` - Sync patterns
- `notes-organization-research.md` - Organization
- `popclip-notes-research.md` - Text capture
- `notes-visual-design.md` - Visual styling
- `notes-templates-research.md` - Templates
- `notes-accessibility-research.md` - Accessibility
