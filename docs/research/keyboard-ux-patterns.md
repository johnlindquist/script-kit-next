# Keyboard-First UX Patterns for Launcher Applications

Research compiled for Script Kit GPUI - January 2026

---

## Table of Contents

1. [Overview](#overview)
2. [Keyboard Navigation Best Practices](#keyboard-navigation-best-practices)
3. [Command Palette Design Patterns](#command-palette-design-patterns)
4. [Shortcut Discovery and Teaching](#shortcut-discovery-and-teaching)
5. [Vim-Style Navigation](#vim-style-navigation)
6. [Accessibility for Keyboard-Only Users](#accessibility-for-keyboard-only-users)
7. [Fuzzy Search and Filtering](#fuzzy-search-and-filtering)
8. [Recommendations for Script Kit](#recommendations-for-script-kit)
9. [Sources](#sources)

---

## Overview

Keyboard-first design is a fundamental principle for launcher applications like Raycast, Alfred, and Script Kit. Users of these tools expect to accomplish tasks without touching the mouse, relying on muscle memory and efficient navigation patterns. This document synthesizes research from industry best practices, WCAG accessibility guidelines, and popular launcher implementations.

---

## Keyboard Navigation Best Practices

### Core Principles

1. **Single Hotkey Activation**: Use a memorable, system-wide hotkey (e.g., Cmd+Space, Cmd+K) to invoke the launcher instantly from anywhere.

2. **Keyboard-First Architecture**: Every feature must be accessible via keyboard. As Raycast demonstrates: "built with a keyboard-first approach that allows you to navigate the entire interface and use extensions without touching your mouse."

3. **Logical Focus Order**: Navigation should follow visual flow (top to bottom, left to right) with predictable movement patterns.

4. **Muscle Memory Support**: Consistent key bindings become second nature. "It soon becomes part of your muscle-memory" when shortcuts remain stable.

### Essential Key Bindings

| Action | Primary | Alternative |
|--------|---------|-------------|
| Navigate up | `Arrow Up` | `k` (vim) / `Ctrl+P` |
| Navigate down | `Arrow Down` | `j` (vim) / `Ctrl+N` |
| Select/confirm | `Enter` | `Return` |
| Cancel/close | `Escape` | `Ctrl+[` |
| Action menu | `Cmd+K` | `Tab` |
| Go back | `Cmd+[` | `Backspace` |
| First item | `Home` | `gg` (vim) |
| Last item | `End` | `G` (vim) |

### Navigation Patterns for Lists

1. **Arrow Key Navigation**: Standard up/down arrows for list traversal
2. **Type-Ahead Selection**: Typing characters jumps to matching items
3. **Wrap-Around Navigation**: Optionally wrap from last to first item
4. **Home/End Keys**: Quick jump to list boundaries
5. **Page Up/Down**: Skip multiple items at once for long lists

### Focus Management

- **Roving Tabindex**: Only one element in a composite widget should be in the tab order. Arrow keys navigate within the widget, Tab moves to the next widget.
- **Focus Persistence**: Remember last-focused item when returning to a view
- **Visual Focus Indicators**: Clear, high-contrast focus rings (minimum 3:1 contrast ratio)

---

## Command Palette Design Patterns

### Definition

A command palette provides a searchable list of commands in a popup window, accessible via keyboard shortcut. Also known as "command bars," "command launchers," or "omniboxes."

### Key Components

1. **Search Input**: Always focused on open, accepts immediate typing
2. **Command List**: Filterable list with fuzzy matching
3. **Action Preview**: Show keyboard shortcuts inline with results
4. **Recent Items**: Display recently used commands for quick access
5. **Contextual Results**: Adapt results based on current application state

### Common Trigger Shortcuts

| Shortcut | Used By |
|----------|---------|
| `Cmd+K` | Superhuman, Linear, Slack |
| `Cmd+Shift+P` | VS Code, Sublime Text |
| `Cmd+Space` | Raycast, Alfred, Spotlight |
| `Cmd+E` | Notion, some editors |
| `?` | Gmail, Dropbox (for help) |

### Design Best Practices

1. **Universal Availability**: The palette should be accessible from anywhere in the app with the same shortcut.

2. **Fuzzy Search**: "Users don't even have to remember the exact name of a command. Fuzzy search can help them find it by simply typing in similar names or related keywords."

3. **Dual Functionality**: Combine search (finding things) with commands (doing things). "You're not only searching through the available actions in an app, you can also search through content."

4. **Context Awareness**: "Knowing what the user will want to do in a given situation is where the super powers come from."

5. **Immediate Response**: Response time under 200ms for smooth experience.

6. **Visual Feedback**: After executing a command, provide clear indication of success.

### Implementation Notes

- Show keyboard shortcuts next to command names
- Group commands by category (Actions, Navigation, Settings)
- Support command chaining (e.g., "new file in folder")
- Provide "no results" guidance with suggestions

---

## Shortcut Discovery and Teaching

### Discovery Mechanisms

1. **Menu Labels**: Display shortcuts next to menu items (e.g., "Save ... Cmd+S"). This treats menus as learning guides.

2. **Tooltips**: Show shortcuts on hover: "Print (Cmd+P)". "These gentle reminders help transition users from GUI reliance to shortcut usage."

3. **In-App Reference (Cheat Sheets)**: Dedicated shortcut reference accessible via a meta-shortcut (e.g., Ctrl+/ in Google Docs, ? in Gmail/Dropbox).

4. **Command Palette Integration**: "The best command bar experiences give users indicators of the keyboard shortcuts available for those actions so they won't have to come back next time."

5. **Contextual Tips**: Show tips when users perform actions the "long way" (e.g., "You used Paste Special often, try Cmd+Shift+V next time!").

### Teaching Strategies

1. **Mnemonic Design**: "Gmail shortcuts are designed to be intuitive: 'C' for Compose, 'R' for Reply, 'A' for Reply All."

2. **Progressive Disclosure**: Introduce shortcuts gradually during onboarding rather than overwhelming new users.

3. **Usage-Based Suggestions**: Track repeated actions and suggest relevant shortcuts.

4. **Consistent System**: "Whatever you choose, make it a clear and consistent system."

### Gamification Approaches

Tools like ShortcutFoo and KeyCombiner demonstrate effective learning patterns:

- **Spaced Repetition**: Practice shortcuts at intervals to build retention
- **Confidence Scores**: Track performance metrics per shortcut
- **Progressive Difficulty**: Gradually increase complexity
- **Immediate Feedback**: Show success/failure instantly

### Best Practices

1. Don't override native browser/OS shortcuts
2. Prefer one or two letter shortcuts when possible
3. Document shortcuts in menus AND tooltips
4. Provide a searchable shortcut reference
5. Support customization for power users

---

## Vim-Style Navigation

### Why Vim Navigation?

"Once your fingers have learned to speak Vim, they don't want to speak anything else! It's simply a very effective way of navigating, creating, and editing text."

### Core Vim Bindings for Launchers

| Key | Action |
|-----|--------|
| `j` | Move down |
| `k` | Move up |
| `h` | Move left / Go back |
| `l` | Move right / Enter |
| `gg` | Go to first item |
| `G` | Go to last item |
| `Ctrl+d` | Page down |
| `Ctrl+u` | Page up |
| `/` | Enter search mode |
| `n` | Next search result |
| `N` | Previous search result |

### Modal Navigation

Vim-style navigation often implies modal interfaces:

1. **Normal Mode**: Navigate with hjkl, execute commands
2. **Insert Mode**: Text input in search field
3. **Visual Mode**: Multi-select items

### Popular Implementations

- **Raycast**: Native vim-style navigation option
- **Lazygit**: Full vim keybindings for git operations
- **Yazi**: Vim-based terminal file manager
- **VSCodeVim/IdeaVim**: Editor plugins
- **Vimium/VimNav**: Browser extensions

### Macros System-Wide Tools

- **Homerow** (macOS): Add vim-like navigation to any app
- **win-vind** (Windows): Operate Windows GUI like Vim
- **warpd** (Linux): Modal keyboard-driven mouse manipulation

### Implementation Considerations

1. **Optional Activation**: Vim bindings should be opt-in, not default
2. **Visual Mode Indicator**: Clearly show current mode
3. **Escape Hatch**: Always allow Escape to return to normal mode
4. **Conflict Resolution**: Handle conflicts with native shortcuts gracefully

---

## Accessibility for Keyboard-Only Users

### Why It Matters

- "Many users with motor disabilities rely on a keyboard"
- "Blind users also typically use a keyboard for navigation"
- "68% of screen reader users encounter keyboard traps or inaccessible interfaces monthly"
- "7 million Americans cannot use a mouse due to motor disabilities"

### WCAG Requirements (Level A)

| Criterion | Requirement |
|-----------|-------------|
| 2.1.1 Keyboard | All functionality available via keyboard |
| 2.1.2 No Keyboard Trap | Focus can always move away from components |
| 2.1.4 Character Key Shortcuts | Single-key shortcuts must be remappable |
| 2.4.1 Bypass Blocks | Mechanism to skip repeated content |
| 2.4.3 Focus Order | Logical, meaningful navigation sequence |

### Focus Management

1. **Visible Focus Indicators**: "Sighted keyboard users must be able to see where focus is at all times."
   - Minimum 3:1 contrast ratio for custom focus indicators
   - Default browser focus rings are exempt from contrast requirements

2. **Focus Trapping (Intentional)**: Modal dialogs should trap focus within until dismissed.

3. **Focus Restoration**: When closing modals/dialogs, return focus to the trigger element.

4. **Roving Tabindex**: For composite widgets (toolbars, listboxes), use arrow keys for internal navigation, Tab to move out.

### Screen Reader Considerations

1. **ARIA Roles**: Use appropriate roles (listbox, option, menu, menuitem)
2. **Live Regions**: Announce dynamic content changes
3. **Label Association**: All interactive elements need accessible names
4. **State Communication**: Announce selected, expanded, disabled states

### Avoiding Keyboard Traps

- Modal windows must close with Escape
- Custom widgets need clear exit shortcuts
- Focus must never get "stuck"
- Test navigation flow comprehensively

### Testing Protocol

1. Disconnect the mouse
2. Navigate entire interface with Tab, Shift+Tab, arrows
3. Verify all actions are possible
4. Check focus visibility at every step
5. Test with screen reader (VoiceOver, NVDA)

---

## Fuzzy Search and Filtering

### Fuzzy Matching Principles

- Match partial strings with character omissions
- Rank results by relevance (match position, frequency)
- Maximum 2 character edits for accuracy balance
- Support synonym matching ("make" finds "create")

### Performance Requirements

- **Response Time**: Under 200ms for smooth experience
- **Debounce**: 300-500ms delay before triggering search
- **Incremental Results**: Show partial results as user types

### List Filtering UX

1. **Live Filtering**: Update results in real-time as user types
2. **Clear Feedback**: Show match count, highlight matching characters
3. **Empty State**: Provide helpful messaging when no results found
4. **Type-Ahead Selection**: Jump to items starting with typed characters

### Implementation Libraries

- **Fuse.js**: Client-side fuzzy search
- **match-sorter**: Ranked search matching
- **fzf**: Command-line fuzzy finder (reference implementation)

### Best Practices

1. Highlight matched characters in results
2. Preserve original order for equally-ranked results
3. Support multiple search terms (AND logic)
4. Remember search history for quick recall
5. Provide keyboard shortcuts to clear search

---

## Recommendations for Script Kit

Based on this research, here are specific recommendations for Script Kit GPUI:

### 1. Core Navigation

```
Priority: Critical

- Arrow keys for list navigation (required)
- Enter to select, Escape to close (required)
- Home/End for list boundaries
- Optional vim-style navigation (j/k/gg/G) behind a setting
- Cmd+K or Tab to open action menu
```

### 2. Focus Management

```
Priority: Critical

- Clear, high-contrast focus indicators
- Roving tabindex for list navigation
- Focus trap in modal dialogs
- Return focus to trigger on close
- Never lose focus during navigation
```

### 3. Shortcut Discovery

```
Priority: High

- Show shortcuts inline with actions (e.g., "Open  Cmd+O")
- Implement ? or Cmd+/ for shortcut cheat sheet
- Add tooltips showing shortcuts on hover (if applicable)
- Track usage and suggest shortcuts for repeated actions
```

### 4. Command Palette Enhancements

```
Priority: High

- Fuzzy search with character highlighting
- Recent items section at top of results
- Context-aware results based on current state
- Sub-200ms response time
- Group results by category (Actions, Files, Scripts)
```

### 5. Accessibility

```
Priority: Critical

- All features accessible via keyboard
- ARIA roles for screen readers (listbox, option, etc.)
- Announce selection changes via live regions
- Support remapping of single-key shortcuts
- Test with VoiceOver regularly
```

### 6. Progressive Learning

```
Priority: Medium

- Show "Pro tip" when user performs action inefficiently
- Track shortcut usage and celebrate mastery
- Optional onboarding tour of key shortcuts
- Keyboard shortcut trainer mode (gamification)
```

### 7. Vim Integration

```
Priority: Medium (opt-in feature)

- j/k navigation in lists
- gg/G for first/last
- / to focus search
- Modal indicators if implemented
- Clear documentation for vim users
```

### 8. Implementation Checklist

- [ ] Arrow key navigation works in all lists
- [ ] Enter/Escape behave consistently
- [ ] Focus indicators visible and high-contrast
- [ ] Shortcuts shown in UI
- [ ] Cheat sheet accessible via shortcut
- [ ] Fuzzy search with highlighting
- [ ] Screen reader testing passed
- [ ] No keyboard traps
- [ ] vim mode (optional)
- [ ] Usage analytics for shortcut suggestions

---

## Sources

### Launcher Applications
- [Raycast vs Alfred](https://www.raycast.com/raycast-vs-alfred)
- [Raycast for Designers](https://www.hackdesign.org/toolkit/raycast/)
- [Alfred vs Raycast: The Ultimate Launcher Face-Off](https://medium.com/the-mac-alchemist/alfred-vs-raycast-the-ultimate-launcher-face-off-855dc0afec89)
- [Spotlight vs Alfred vs Raycast](https://medium.com/@andriizolkin/spotlight-vs-alfred-vs-raycast-31bd942ac3b6)

### Keyboard Shortcuts UX
- [The UX of Keyboard Shortcuts](https://medium.com/design-bootcamp/the-art-of-keyboard-shortcuts-designing-for-speed-and-efficiency-9afd717fc7ed)
- [How to Design Great Keyboard Shortcuts](https://knock.app/blog/how-to-design-great-keyboard-shortcuts)
- [UI Copy: Command Names and Keyboard Shortcuts](https://www.nngroup.com/articles/ui-copy/)
- [Keyboard Shortcuts Design Pattern](https://ui-patterns.com/patterns/keyboard-shortcuts)
- [Selecting Keyboard Shortcuts for Your App](https://www.command.ai/blog/selecting-keyboard-shortcuts-for-your-app/)

### Command Palette Design
- [Command Palette UX Patterns](https://medium.com/design-bootcamp/command-palette-ux-patterns-1-d6b6e68f30c1)
- [Command Palette UI Design Best Practices](https://mobbin.com/glossary/command-palette)
- [Command K Bars](https://maggieappleton.com/command-bar)
- [How to Build a Remarkable Command Palette](https://blog.superhuman.com/how-to-build-a-remarkable-command-palette/)
- [Designing Command Palettes](https://solomon.io/designing-command-palettes/)
- [Command Palette Interfaces](https://philipcdavis.com/writing/command-palette-interfaces)

### Vim-Style Navigation
- [Vim Keybindings Everywhere - The Ultimate List](https://github.com/erikw/vim-keybindings-everywhere-the-ultimate-list)
- [win-vind: Vim-like Windows Control](https://github.com/pit-ray/win-vind)
- [VimNav Browser Extension](https://vimnav.dev/)

### Accessibility
- [WebAIM: Keyboard Accessibility](https://webaim.org/techniques/keyboard/)
- [Focus & Keyboard Operability](https://usability.yale.edu/web-accessibility/articles/focus-keyboard-operability)
- [WCAG 2.4.3 Focus Order](https://www.w3.org/WAI/WCAG21/Understanding/focus-order.html)
- [Managing Focus and Visible Focus Indicators](https://vispero.com/resources/managing-focus-and-visible-focus-indicators-practical-accessibility-guidance-for-the-web/)
- [Designing Usable Focus Indicators](https://www.deque.com/blog/give-site-focus-tips-designing-usable-focus-indicators/)
- [Keyboard Navigation Patterns for Complex Widgets](https://www.uxpin.com/studio/blog/keyboard-navigation-patterns-complex-widgets/)
- [W3C ARIA Keyboard Interface Practices](https://www.w3.org/WAI/ARIA/apg/practices/keyboard-interface/)

### Fuzzy Search & Filtering
- [Search UX Best Practices](https://www.pencilandpaper.io/articles/search-ux)
- [15 Filter UI Patterns That Work](https://bricxlabs.com/blogs/universal-search-and-filters-ui)
- [fzf: Command-Line Fuzzy Finder](https://github.com/junegunn/fzf)

### Learning & Gamification
- [KeyCombiner: Master Keyboard Shortcuts](https://keycombiner.com/)
- [ShortcutFoo: Learn Shortcuts](https://www.shortcutfoo.com/)
- [Learn Keyboard Shortcuts (Zapier)](https://zapier.com/blog/learn-keyboard-shortcuts/)

---

*Last updated: January 2026*
