# VS Code Command Palette UX Research

Research on VS Code's Command Palette and related command palette patterns for Script Kit GPUI implementation.

---

## Table of Contents

1. [Fuzzy Search](#1-fuzzy-search)
2. [Keyboard Navigation](#2-keyboard-navigation)
3. [Actions and Shortcuts Display](#3-actions-and-shortcuts-display)
4. [Categorization and Grouping](#4-categorization-and-grouping)
5. [Visual Design](#5-visual-design)
6. [Input Modes and Prefixes](#6-input-modes-and-prefixes)
7. [MRU and History](#7-mru-and-history)
8. [Accessibility](#8-accessibility)
9. [Comparison with Raycast](#9-comparison-with-raycast)
10. [Suggestions for Script Kit](#10-suggestions-for-script-kit)

---

## 1. Fuzzy Search

### How VS Code Handles Fuzzy Search

VS Code's Command Palette uses fuzzy matching that allows partial, non-contiguous matches:

- **Partial matching**: Typing "pref key" surfaces commands like "Preferences: Open Keyboard Shortcuts"
- **Acronym matching**: Typing "ttt" matches "**T**ransform **t**o **T**itle case"
- **Character order matters**: Query characters must appear in target in the same sequential order
- **Word boundary preference**: Higher scores for matches at word starts (camelCase, separators)
- **Sequential bonuses**: Consecutive character matches receive preferential scoring

### Fuzzy Matching Algorithm (VS Code-inspired)

The [code-fuzzy-match](https://github.com/D0ntPanic/code-fuzzy-match) Rust crate implements VS Code-style matching:

```rust
let mut matcher = code_fuzzy_match::FuzzyMatcher::new();
let matches = matcher.fuzzy_match("the quick brown fox", "bro fox");
assert!(matches.is_some());
```

Key characteristics:
- **Not Levenshtein distance**: Optimized for command palettes, not spell-checking
- **Designed for**: Command palettes, quick file navigation, code searching
- **Returns scores**: Match quality ranking, not just boolean match

### Configuration Options

VS Code has a feature request for user-selectable matching modes:
- **Strict**: Starting character + consecutive characters only
- **Sequential**: Characters in order but not necessarily consecutive
- **Fuzzy**: Current default behavior

**Source**: [VS Code Issue #141224](https://github.com/microsoft/vscode/issues/141224)

---

## 2. Keyboard Navigation

### Core Navigation Keys

| Key | Action |
|-----|--------|
| `Up/Down Arrow` | Navigate through results |
| `Enter` | Execute selected command |
| `Escape` | Close palette |
| `Tab` | May cycle through sections or accept autocomplete |

### Command History Navigation

- **Arrow keys**: Cycle through command history when input is empty
- **Recent commands**: Show at top of list before other results
- **History persistence**: Commands persist across restarts (global storage)

### Best Practices from Research

1. **Toggle behavior**: Same shortcut opens and closes the palette
2. **Hands on keyboard**: Entire interaction possible without mouse
3. **Focus management**: Return focus to previous location on dismiss
4. **Tab trapping**: Focus stays within palette while open

**Source**: [Superhuman Blog](https://blog.superhuman.com/how-to-build-a-remarkable-command-palette/)

---

## 3. Actions and Shortcuts Display

### How VS Code Displays Shortcuts

- **Inline display**: Keyboard shortcuts shown alongside command names
- **Right-aligned**: Shortcuts appear on the right side of each row
- **Dimmed styling**: Shortcuts use secondary/muted color to not compete with command name

### Visual Layout

```
[Icon] Category: Command Name                    [Shortcut]
```

Example:
```
[>] File: Save                                   Cmd+S
[>] Edit: Copy                                   Cmd+C
[>] View: Toggle Terminal                        Cmd+`
```

### Raycast Action Panel Pattern

Raycast uses a two-tier action system:

| Action Type | Shortcut | Description |
|-------------|----------|-------------|
| Primary | `Enter` | First/default action |
| Secondary | `Cmd+Enter` | Second action |
| Action Panel | `Cmd+K` | Opens full action menu |

**Source**: [Raycast Action Panel API](https://developers.raycast.com/api-reference/user-interface/action-panel)

---

## 4. Categorization and Grouping

### VS Code Category Prefixes

Commands are prefixed with their category for discoverability:

| Prefix | Example Commands |
|--------|------------------|
| `File:` | Save, Save All, Close |
| `Edit:` | Copy, Paste, Undo |
| `View:` | Toggle Terminal, Zoom In |
| `Git:` | Commit, Push, Pull |
| `Terminal:` | New Terminal, Split Terminal |
| `Debug:` | Start Debugging, Add Breakpoint |

### Best Practices

1. **Consistent prefixes**: All commands in a domain share the same prefix
2. **No emojis**: VS Code guidelines explicitly discourage emojis in command names
3. **Clear naming**: Names should be self-documenting
4. **Grouped shortcuts**: Related commands should have related shortcuts

### Visual Separators

Use separators to group related commands:

```
Recent
---------
File: Save
Edit: Undo
---------
All Commands
---------
Debug: Start Debugging
Debug: Add Breakpoint
```

**Source**: [VS Code UX Guidelines](https://code.visualstudio.com/api/ux-guidelines/command-palette)

---

## 5. Visual Design

### Match Highlighting

VS Code highlights matching characters in results:

| Theme Token | Purpose |
|-------------|---------|
| `list.highlightForeground` | Highlight color for matching characters |
| `list.focusHighlightForeground` | Highlight on focused/selected items |

### Selected Item Styling

| Theme Token | Purpose |
|-------------|---------|
| `list.focusBackground` | Background of selected item |
| `list.focusForeground` | Text color of selected item |
| `list.focusOutline` | Border/outline of selected item |

### Design Principles

1. **Visual weight**: Command palette should be visually prominent (center screen, large)
2. **Partial visibility**: Show that more results exist below (cut off last item)
3. **Icons per command**: Visual differentiation and quick scanning
4. **Icon opacity**: Active icons at 54% black, inactive at 38% (on light backgrounds)

**Source**: [Mobbin Command Palette](https://mobbin.com/glossary/command-palette)

---

## 6. Input Modes and Prefixes

### VS Code Special Prefixes

VS Code's palette supports multiple modes via prefix characters:

| Prefix | Mode | Shortcut | Description |
|--------|------|----------|-------------|
| `>` | Commands | `Cmd+Shift+P` | Execute commands |
| (none) | Files | `Cmd+P` | Quick open files |
| `@` | Symbols (file) | `Cmd+Shift+O` | Navigate to symbol in current file |
| `@:` | Symbols (grouped) | `Cmd+Shift+O, :` | Symbols grouped by kind |
| `#` | Symbols (workspace) | `Cmd+T` | Search symbols across workspace |
| `:` | Go to line | `Ctrl+G` | Jump to line number |
| `?` | Help | - | Show available prefix commands |

### Mode Switching UX

- **Seamless transition**: Removing `>` switches from commands to file search
- **Preserved input**: Text after prefix is preserved when switching modes
- **Visual indicator**: Prefix character shows current mode

**Source**: [VS Code Tips & Tricks](https://dev.to/this-is-learning/visual-studio-code-tips-tricks-command-palette-and-its-friends-2bhi)

---

## 7. MRU and History

### VS Code MRU Behavior

- **Recent at top**: Most recently used commands appear first
- **Persistent history**: Stored globally across windows/restarts
- **Configurable count**: `workbench.commandPalette.history` setting (0 disables)
- **Preserved input**: `workbench.commandPalette.preserveInput` restores last query

### MRU Algorithm

1. Recent commands shown at top of unfiltered list
2. When typing, MRU commands remain prioritized in filtered results
3. Commands executed via palette update MRU (not programmatic execution)

### Dynamic Recommendations

Consider showing on palette open:
- Recent queries
- Frequently used commands
- Context-aware suggestions (current file type, git state, etc.)

**Source**: [VS Code Issue #13080](https://github.com/Microsoft/vscode/issues/13080)

---

## 8. Accessibility

### Focus Management

1. **Initial focus**: Move to first focusable element (usually input) on open
2. **Focus trap**: Tab cycles within palette, not to underlying content
3. **Return focus**: Restore focus to previous element on close

### Screen Reader Support

| Requirement | Implementation |
|-------------|----------------|
| Role announcement | `role="dialog"` or `role="combobox"` |
| Name/description | `aria-label` or `aria-labelledby` |
| Live regions | Announce result count changes |
| Keyboard shortcuts | Document via `aria-keyshortcuts` |

### Keyboard Shortcut Considerations

- **Avoid single-letter shortcuts**: Conflicts with screen reader commands
- **Avoid common shortcuts**: Don't override standard OS/AT shortcuts
- **Document shortcuts**: Provide discoverable shortcut reference

**Source**: [W3C ARIA APG](https://www.w3.org/WAI/ARIA/apg/practices/keyboard-interface/)

---

## 9. Comparison with Raycast

### Raycast's Enhanced Patterns

| Feature | VS Code | Raycast |
|---------|---------|---------|
| Actions per item | Single | Multiple (Action Panel) |
| Nested navigation | Limited | Full hierarchy support |
| Extensions | Yes | Yes (richer API) |
| Favorites/pinning | No | Yes (`Cmd+1-9`) |
| Context actions | No | Yes (selection-aware) |

### Raycast Action Panel

```
Primary Action     Enter
Secondary Action   Cmd+Enter
Open Action Panel  Cmd+K
```

Action Panel features:
- **Sections**: Group related actions visually
- **Submenus**: Nested action hierarchies
- **Shortcuts**: Auto-assigned or custom per action

### Script Kit Opportunity

Raycast's action panel pattern could enhance Script Kit:
- Show multiple actions for selected item
- Support `Cmd+Enter` for secondary action
- Allow scripts to define action hierarchies

**Source**: [Raycast Manual](https://manual.raycast.com/action-panel)

---

## 10. Suggestions for Script Kit

### High Priority

#### 1. Implement VS Code-style Fuzzy Matching
- Use [code-fuzzy-match](https://github.com/D0ntPanic/code-fuzzy-match) crate (Rust, VS Code-inspired)
- Support partial matches, acronyms, word boundary bonuses
- Highlight matching characters in results

#### 2. MRU/History Support
- Track recently used scripts and commands
- Show recent items at top of unfiltered list
- Persist across sessions
- Add setting to configure history length

#### 3. Category Prefixes
- Prefix scripts by category: `Git:`, `File:`, `Dev:`, etc.
- Support filtering by category
- Allow scripts to define their category in metadata

### Medium Priority

#### 4. Input Mode Prefixes
Consider supporting VS Code-style mode switching:
- `>` for commands/actions
- `/` for scripts (Script Kit convention)
- `@` for symbols/snippets
- `:` for line numbers or settings

#### 5. Action Panel (Raycast-style)
- Primary action on `Enter`
- Secondary action on `Cmd+Enter`
- Full action panel on `Cmd+K`
- Allow scripts to define multiple actions

#### 6. Match Highlighting
- Highlight matching characters with accent color
- Use theme-aware colors (`list.highlightForeground`)
- Ensure contrast on both light/dark themes

### Lower Priority

#### 7. Visual Separators
- Group results by category with section headers
- Use subtle dividers between groups
- Show "Recent" section at top

#### 8. Shortcut Display
- Show keyboard shortcuts right-aligned
- Use secondary/muted color
- Support custom shortcuts per script

#### 9. Accessibility Enhancements
- Proper ARIA roles (`combobox`, `listbox`)
- Focus management on open/close
- Screen reader announcements for result count

### Implementation Notes

#### Fuzzy Matching Integration

```rust
// Pseudocode for Script Kit integration
use code_fuzzy_match::FuzzyMatcher;

fn filter_scripts(query: &str, scripts: &[Script]) -> Vec<ScoredScript> {
    let mut matcher = FuzzyMatcher::new();
    let mut results: Vec<ScoredScript> = scripts
        .iter()
        .filter_map(|script| {
            matcher.fuzzy_match(&script.name, query)
                .map(|score| ScoredScript { script, score, matches: matcher.matches() })
        })
        .collect();

    // Sort by score descending, MRU items get bonus
    results.sort_by(|a, b| {
        let a_score = a.score + if a.script.is_recent { 1000 } else { 0 };
        let b_score = b.score + if b.script.is_recent { 1000 } else { 0 };
        b_score.cmp(&a_score)
    });

    results
}
```

#### Theme Color Mapping

| VS Code Token | Script Kit Usage |
|---------------|------------------|
| `list.focusBackground` | Selected item background |
| `list.focusForeground` | Selected item text |
| `list.highlightForeground` | Matching character highlight |
| `quickInput.background` | Palette background |
| `quickInputList.focusBackground` | Focused result background |

---

## Sources

- [VS Code Command Palette UX Guidelines](https://code.visualstudio.com/api/ux-guidelines/command-palette)
- [VS Code User Interface Documentation](https://code.visualstudio.com/docs/getstarted/userinterface)
- [VS Code Tips and Tricks](https://code.visualstudio.com/docs/getstarted/tips-and-tricks)
- [code-fuzzy-match Rust Crate](https://github.com/D0ntPanic/code-fuzzy-match)
- [Raycast Action Panel API](https://developers.raycast.com/api-reference/user-interface/action-panel)
- [Raycast Manual - Action Panel](https://manual.raycast.com/action-panel)
- [Mobbin Command Palette Design](https://mobbin.com/glossary/command-palette)
- [Superhuman: How to Build a Remarkable Command Palette](https://blog.superhuman.com/how-to-build-a-remarkable-command-palette/)
- [Command Palette Interfaces by Philip Davis](https://philipcdavis.com/writing/command-palette-interfaces)
- [Designing a Command Palette by Destiner](https://destiner.io/blog/post/designing-a-command-palette/)
- [VS Code Tips & Tricks - Command Palette](https://dev.to/this-is-learning/visual-studio-code-tips-tricks-command-palette-and-its-friends-2bhi)
- [W3C ARIA Keyboard Interface Practices](https://www.w3.org/WAI/ARIA/apg/practices/keyboard-interface/)
- [VS Code Theme Color Reference](https://code.visualstudio.com/api/references/theme-color)
- [VS Code Issue #13080 - MRU Ordering](https://github.com/Microsoft/vscode/issues/13080)
- [VS Code Issue #141224 - Matching Modes](https://github.com/microsoft/vscode/issues/141224)
