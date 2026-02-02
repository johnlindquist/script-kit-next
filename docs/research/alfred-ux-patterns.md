# Alfred UX Patterns Research

This document captures Alfred's key UX patterns, design decisions, and interaction paradigms for reference when building Script Kit features.

---

## 1. Key UX Differentiators

### Speed-First Philosophy
Alfred's core identity is built around speed. It consistently outperforms competitors in raw performance:
- Blazing fast file search and indexing
- Minimal latency between keystrokes and results
- Optimized for power users who invoke the launcher hundreds of times daily

### Sentence-Based Interaction
Unlike Raycast's menu-driven approach, Alfred uses a **sentence-based interaction model**:
- Users type natural keywords and commands
- Results appear as the user types
- Actions flow from typed queries rather than navigating menus

### Non-Activating Panel Mode
Alfred defaults to appearing as a non-activating panel (like Spotlight):
- Does not steal focus from the current application
- Allows quick lookups without context switching
- Optional "Compatibility mode" makes it behave like a standard app window

### Deep Powerpack Integration
Advanced features require the Powerpack (paid):
- Clipboard history
- Snippets and text expansion
- Custom workflows
- Universal Actions
- Theme customization

---

## 2. Search and Filtering Patterns

### Default Matching Modes

Alfred provides four filtering modes for Script Filters:

| Mode | Description | Example Match |
|------|-------------|---------------|
| **Word Boundary (Default)** | Matches from the start of any word | "Family Photos" matches "Photos" |
| **Strict** | Matches only from the beginning | "My Family" matches, "Family" does not |
| **Loose (Any Order)** | Words can appear in any order | "Photos Family" matches "My Family Photos" |
| **Loose (Written Order)** | Words must appear in written order | "My Photos" matches, "Photos My" does not |

### Fuzzy Capital Letters
The default search uses fuzzy capital letter matching:
- "dd" matches "DragonDrop"
- "rk" matches "ReadKit"
- "of" matches "OmniFocus"

With "Anchored" mode on, Alfred only searches from the first character of every word.

### Performance Considerations
- Alfred's native Objective-C filtering handles 10,000+ items smoothly
- Third-party fuzzy algorithms (like alfred-workflow's Python implementation) become sluggish around 1,500-2,500 items
- For very large datasets (20,000+), SQLite fulltext search is recommended

### Filtering Best Practices
- Fuzzy search works well for single-field searches (name/title)
- Word-based search provides better results when searching multiple fields or tags
- Essential information should be in the title, not just the subtitle (users can hide subtitles)

---

## 3. Keyboard Shortcut Patterns

### Activation
| Shortcut | Action |
|----------|--------|
| `Option + Space` | Show Alfred (default, customizable) |
| `Command + ,` | Open Preferences |
| `Escape` | Hide Alfred / Go back |
| `Command + Escape` | Hide (ignore stacks) |

### Navigation
| Shortcut | Action |
|----------|--------|
| `Up/Down` | Move between items |
| `Return` | Action selected item |
| `Command + 1-9` | Direct action on numbered result |
| `Tab` | Autocomplete |
| `Command + Down` | Browse into folder |
| `Backspace` | Go up one folder level |
| `.` (period) | Show hidden files |

### File Operations
| Shortcut | Action |
|----------|--------|
| `Command + O` | Open in default app |
| `Command + Return` | Reveal in Finder |
| `Shift` or `Command + Y` | Quick Look preview |
| `Option + Command + \` | Show available actions |

### File Buffer (Multi-Select)
| Shortcut | Action |
|----------|--------|
| `Option + Down` | Add to buffer, select next |
| `Option + Up` | Toggle buffer inclusion |
| `Option + Left` | Remove last item from buffer |
| `Option + Right` | Action all buffered items |

### Clipboard & Snippets
| Shortcut | Action |
|----------|--------|
| `Option + Command + C` | Open clipboard history |
| `Command + C` | Copy selected clip |
| `Command + S` | Save as snippet |

### Modifier Key Actions
Holding modifier keys while pressing Return changes the action:
- Each modifier (Cmd, Option, Ctrl, Shift) can trigger a different action
- The subtitle updates to show what the modified action will do
- Workflows can define custom modifier actions
- Combinations of modifiers are supported (e.g., Option + Shift)

### Hotkey Trigger Behavior
- Alfred waits a few milliseconds before releasing modifier keys
- "Pass through modifier keys" setting removes ~500ms latency
- Hotkeys can be connected to almost anything in a workflow

---

## 4. Visual Hierarchy and Information Density

### Result Display Structure
Each result item contains:
1. **Icon** - Large, on the left
2. **Title** - Primary text, large font
3. **Subtitle** - Secondary text, smaller, below title
4. **Shortcut Number** - Optional, on the right (1-9)

### Configurable Display Elements
Users can show/hide:
- Bowler hat and cog icons
- Result shortcut numbers (1-9 on the right)
- Menu bar icon
- Scroll bar in results
- Number of results displayed

### Theme Customization
- **Text sizing**: Drag up/down to resize any text element
- **Corner radius**: Drag corners to adjust roundness
- **Colors**: Click any element to change its color
- **Opacity**: Option + drag to adjust transparency
- **Fonts**: Right-click to change fonts, Ctrl-click to apply globally
- **Window blur**: Global setting for non-opaque themes

### Information Density Principles
- Power users prefer higher information density
- Novice users benefit from simpler, less dense UIs
- Alfred lets users customize density via result count and element visibility
- Essential info should be in the title (subtitle can be hidden)

### Screen Positioning
- Default: Center top of screen
- Configurable grid for custom positioning
- Multi-monitor support: default screen, mouse screen, or active screen

---

## 5. Universal Actions

### Core Concept
Universal Actions (introduced in Alfred 4.5) allow users to:
- Select any content (text, URLs, files) anywhere on macOS
- Invoke Alfred with a hotkey
- See contextually relevant actions for that content

### Keyboard Access
| Shortcut | Context |
|----------|---------|
| `Command + /` | Default Universal Actions hotkey |
| `Right Arrow` | Show actions from results (configurable) |
| `Option + Right` | Action all buffered files |

### Features
- 60+ built-in actions
- Actions filter based on content type
- Type to filter action list
- Chain multiple actions without leaving Alfred
- Custom actions via Workflows

### Deep Integration Points
- Alfred results
- File Navigation
- Clipboard History
- System-wide selection

---

## 6. Suggestions for Script Kit

### Speed Optimizations
1. **Minimize render latency** - Every millisecond matters at 200+ invocations/day
2. **Use native filtering** - Rust filtering will outperform JavaScript
3. **Lazy load results** - Show first 10 results immediately, load more on scroll

### Search Behavior
1. **Implement word boundary matching** as the default
2. **Support fuzzy capital letters** - "sk" should match "Script Kit"
3. **Allow configurable matching modes** per script
4. **Consider loose matching** for user-facing search, strict for commands

### Keyboard Patterns to Adopt
1. **Consistent modifier actions** - Option, Command, Ctrl, Shift should have predictable behaviors
2. **File buffer pattern** - Option + arrows for multi-select operations
3. **Quick Look with Shift** - Preview without opening
4. **Direct number shortcuts** - Command + 1-9 for instant selection

### Visual Hierarchy
1. **Two-line result items** - Title + subtitle pattern
2. **Configurable density** - Let users choose result count
3. **Optional shortcut numbers** - Some users find them distracting
4. **Clear modifier feedback** - Update subtitle when modifier is held

### Actions Panel
1. **Context-aware actions** - Filter actions by item type
2. **Searchable action list** - Type to filter available actions
3. **Chainable actions** - Allow multiple actions in sequence
4. **Custom action support** - Scripts can define their own actions

### Theming Considerations
1. **Non-opaque themes** with window blur
2. **Customizable element sizing** via drag
3. **Color picker for all elements**
4. **Import/export themes**

---

## Sources

- [Alfred - Productivity App for macOS](https://www.alfredapp.com/)
- [Alfred Cheatsheet](https://www.alfredapp.com/help/getting-started/cheatsheet/)
- [Alfred Appearance & Theming](https://www.alfredapp.com/help/appearance/)
- [Script Filter Input](https://www.alfredapp.com/help/workflows/inputs/script-filter/)
- [Universal Actions](https://www.alfredapp.com/help/features/universal-actions/)
- [Hotkey Trigger](https://www.alfredapp.com/help/workflows/triggers/hotkey/)
- [Alfred-Workflow Filtering Documentation](https://www.deanishe.net/alfred-workflow/guide/filtering.html)
- [Alfred vs Raycast Comparison - Josh Collinsworth](https://joshcollinsworth.com/blog/alfred-raycast)
- [Raycast vs Alfred - Raycast](https://www.raycast.com/raycast-vs-alfred)
- [Spotlight vs Alfred vs Raycast - Medium](https://medium.com/@andriizolkin/spotlight-vs-alfred-vs-raycast-31bd942ac3b6)
- [Type Less with Fuzzy Match Search in Alfred](https://sayzlim.net/alfred-full-fuzzy-match/)
- [Creating a Custom Alfred Theme - The Sweet Setup](https://thesweetsetup.com/creating-a-custom-alfred-theme/)
