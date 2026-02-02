# Script Kit GPUI: UX Improvement Recommendations

> Synthesized from 15 parallel research agents analyzing Raycast, Alfred, Spotlight, VS Code, Warp, Fig, and UX best practices.

---

## Executive Summary

Script Kit GPUI already has strong foundations: native Rust rendering, vibrancy support, keyboard navigation, and a flexible action system. The research identified **high-impact improvements** that would bring Script Kit to parity with—and beyond—competitors like Raycast and Alfred.

**Top 5 Most Impactful Changes:**
1. Number keys (1-9) for instant item selection
2. Action panel with Cmd+K
3. Fuzzy search with character highlighting
4. Skeleton loading states
5. Animation polish (ease-out, 150-200ms transitions)

---

## Priority Matrix

### P0: Critical (High Impact, Low-Medium Effort)

| Feature | Description | Source |
|---------|-------------|--------|
| **1-9 Quick Select** | Press 1-9 to instantly select items 1-9 in the list | Raycast, Alfred |
| **Cmd+K Action Panel** | Searchable panel showing all available actions with shortcuts | Raycast, VS Code |
| **Fuzzy Match Highlighting** | Highlight matched characters in results (character-by-character) | VS Code, Alfred |
| **MRU Ranking** | Most recently used items appear at top | All launchers |
| **Instant Selection Feedback** | 0ms delay on arrow key navigation | Raycast |

### P1: High Priority (High Impact, Medium Effort)

| Feature | Description | Source |
|---------|-------------|--------|
| **Skeleton Loading** | Show placeholder shapes while loading (perceived 30% faster than spinners) | UX research |
| **Empty State with CTA** | When no results, offer to create a script | Raycast, UX research |
| **View Transitions** | 200ms ease-out-quint push/pop animations | Raycast |
| **Toast Notifications** | Non-blocking feedback for actions (success/error/loading) | Raycast, Warp |
| **Search Debounce Tuning** | 150-200ms for API calls, instant for in-memory | UX research |

### P2: Medium Priority (Medium Impact, Medium Effort)

| Feature | Description | Source |
|---------|-------------|--------|
| **Input Mode Prefixes** | `>` commands, `@` symbols, `/` categories | VS Code |
| **Ghost Text Suggestions** | Inline dimmed completion based on history | Fig/Amazon Q |
| **Contextual Actions** | Actions adapt based on selected item type | Raycast, Alfred |
| **Keycap Visual Polish** | Consistent keycap styling with proper gaps and contrast | Action bar research |
| **Category Grouping** | Visual separators between result groups | Alfred, VS Code |

### P3: Nice to Have (Lower Impact or Higher Effort)

| Feature | Description | Source |
|---------|-------------|--------|
| **Vim Navigation** | Optional j/k/gg/G bindings | Keyboard research |
| **Density Modes** | Comfortable/Compact/Dense list views | List item research |
| **AI Integration** | `#` trigger for natural language to script | Warp |
| **Clipboard History** | Built-in clipboard manager | Spotlight (macOS 26) |
| **Completion Specs** | Declarative schema for dynamic suggestions | Fig |

---

## Detailed Recommendations

### 1. Keyboard Navigation Enhancements

**Current State:** Basic arrow key navigation with section skipping

**Recommended Additions:**

```
1-9         → Instant select item 1-9
Cmd+K       → Open action panel
Cmd+Enter   → Secondary action
Escape      → Close/go back
Tab         → Accept suggestion / next field
Shift+Tab   → Previous field
Cmd+1-9     → Pin/favorite shortcuts
```

**Implementation Notes:**
- Match both lowercase and CamelCase key variants (already documented in CLAUDE.md)
- Number keys should be opt-in per prompt (might conflict with text input)
- Show numbers as accessories on list items when enabled

### 2. Search & Filtering

**Recommended Algorithm:** Sublime Text-style sequential fuzzy matching

**Scoring Priorities:**
1. Exact match → highest score
2. Word boundary match (e.g., "op" matches "**O**pen **P**roject")
3. CamelCase match (e.g., "gf" matches "**G**et**F**ile")
4. Consecutive character bonus
5. Recent usage boost
6. Frequency boost

**Rust Crate:** [`code-fuzzy-match`](https://github.com/D0ntPanic/code-fuzzy-match) provides VS Code-inspired matching

**Highlighting:**
- Use `theme.colors.text_accent` for matched characters
- Apply bold weight or background highlight
- Character-by-character, not substring

### 3. Action Panel (Cmd+K)

**Layout:**
```
┌─────────────────────────────────────┐
│ 🔍 Search actions...                │
├─────────────────────────────────────┤
│ ACTIONS                             │
│   Open                        ↵     │
│   Open With...          ⌘ ↵        │
│   Quick Look            ⌘ Y        │
├─────────────────────────────────────┤
│ COPY                                │
│   Copy Name             ⌘ C        │
│   Copy Path        ⌘ ⇧ C          │
├─────────────────────────────────────┤
│ DANGER ZONE                         │
│   Delete                ⌘ ⌫        │
└─────────────────────────────────────┘
```

**Features:**
- Fuzzy searchable
- Semantic sections (Actions, Copy, Danger Zone)
- Keyboard navigable
- Shows shortcuts right-aligned
- Contextual based on selected item

### 4. Animation Timing Guide

| Animation | Duration | Easing | Notes |
|-----------|----------|--------|-------|
| Selection change | 0ms | none | Must feel instant |
| Hover feedback | 50ms | ease-out | Subtle |
| List item appear | 100ms | ease-out | Stagger 20ms per item |
| Toast notification | 150ms | ease-out | Slide in from top |
| View push/pop | 200ms | ease-out-quint | Content fades + slides |
| Action panel | 200ms | ease-out | Slide up from bottom |
| Loading pulse | 1000ms | pulsating | Repeat |

**GPUI Implementation:**
```rust
// View transition
.animate_in_from_bottom(Duration::from_millis(200))

// Loading spinner
Animation::new(Duration::from_millis(1000))
    .with_easing(pulsating_between(0.4, 1.0))
    .repeat()
```

### 5. Loading & Empty States

**Loading:**
- < 200ms: No indicator needed
- 200ms-1s: Subtle inline spinner
- > 1s: Skeleton placeholders
- Never block the UI

**Skeleton Example:**
```
┌─────────────────────────────────────┐
│ ████████████████                    │
├─────────────────────────────────────┤
│ ▓▓▓  ██████████████  ████          │
│ ▓▓▓  ████████████    ██████        │
│ ▓▓▓  ██████████████████  ████      │
└─────────────────────────────────────┘
```

**Empty State:**
```
┌─────────────────────────────────────┐
│ 🔍 "foobar"                         │
├─────────────────────────────────────┤
│                                     │
│         No scripts found            │
│                                     │
│    [Create "foobar" script]         │
│                                     │
└─────────────────────────────────────┘
```

### 6. Visual Design Polish

**Typography:**
- Search input: 17px
- List item title: 14px
- List item subtitle: 12px, 60% opacity
- Section header: 11px, uppercase, 80% opacity

**Spacing (8pt grid):**
- Window padding: 8px
- List item height: 40px (comfortable), 32px (compact)
- List item padding: 12px horizontal
- Gap between keycaps: 4px

**Colors:**
- Avoid pure black (#000) and pure white (#FFF)
- Use soft blacks (#1A1A1A - #2D2D2D)
- Use soft whites (#E8E8E8 - #F0F0F0)
- Desaturate accent colors in dark mode

### 7. Accessibility Checklist

- [ ] All functionality accessible via keyboard
- [ ] Visible focus indicators (3:1 contrast minimum)
- [ ] Focus trapping in modals/panels
- [ ] Respect `prefers-reduced-motion`
- [ ] ARIA roles for combobox, listbox, option
- [ ] Live region announcements for result counts
- [ ] Screen reader-friendly shortcut display

---

## Implementation Roadmap

### Phase 1: Quick Wins (1-2 weeks)
- [ ] Add 1-9 quick select keys
- [ ] Tune animation timing (instant selection, 200ms transitions)
- [ ] Add MRU ranking to search results
- [ ] Improve empty state with create action

### Phase 2: Core UX (2-4 weeks)
- [ ] Implement Cmd+K action panel
- [ ] Add fuzzy match character highlighting
- [ ] Implement skeleton loading states
- [ ] Add toast notification system

### Phase 3: Polish (2-4 weeks)
- [ ] Implement view push/pop transitions
- [ ] Add input mode prefixes
- [ ] Contextual actions per item type
- [ ] Density mode toggle

### Phase 4: Advanced (4+ weeks)
- [ ] Ghost text suggestions from history
- [ ] AI integration with # trigger
- [ ] Clipboard history panel
- [ ] Completion spec system

---

## Key Metrics to Track

| Metric | Target | Current |
|--------|--------|---------|
| Window appear latency | < 100ms | ? |
| Search response | < 50ms | ? |
| Animation frame rate | 120 FPS | ? |
| Time to first result | < 16ms | ? |

---

## Competitive Comparison

| Feature | Script Kit | Raycast | Alfred | Spotlight |
|---------|------------|---------|--------|-----------|
| Native rendering | ✅ Rust/GPUI | ✅ AppKit | ✅ AppKit | ✅ AppKit |
| Vibrancy | ✅ | ✅ | ✅ | ✅ |
| 1-9 Quick select | ❌ | ✅ | ✅ | ❌ |
| Action panel | 🟡 Partial | ✅ | ✅ | ❌ |
| Fuzzy highlighting | ❌ | ✅ | ✅ | ✅ |
| Toast notifications | ❌ | ✅ | ❌ | ❌ |
| View transitions | ❌ | ✅ | ❌ | ✅ |
| Skeleton loading | ❌ | ✅ | ❌ | ❌ |
| Extensions/Scripts | ✅ | ✅ | ✅ | ❌ |

---

## Conclusion

Script Kit GPUI has excellent technical foundations. The highest-ROI improvements are:

1. **1-9 quick select** — Dramatically speeds up power user workflows
2. **Cmd+K action panel** — Matches user expectations from Raycast/VS Code
3. **Fuzzy highlighting** — Makes search feel more intelligent
4. **Animation polish** — Small timing changes create perception of speed
5. **Skeleton loading** — Perceived 30% faster than spinners

These five changes would significantly improve the perceived quality and usability of Script Kit while leveraging the existing codebase strengths.

---

## Research Sources

All research documents are available in `/docs/research/`:

1. `raycast-ux-patterns.md`
2. `raycast-animations.md`
3. `alfred-ux-patterns.md`
4. `spotlight-ux-patterns.md`
5. `vscode-palette-ux.md`
6. `keyboard-ux-patterns.md`
7. `current-scriptkit-ux-analysis.md`
8. `search-filter-ux.md`
9. `state-ux-patterns.md`
10. `window-appearance-ux.md`
11. `action-bar-ux.md`
12. `list-item-ux.md`
13. `performance-ux.md`
14. `warp-terminal-ux.md`
15. `fig-autocomplete-ux.md`
