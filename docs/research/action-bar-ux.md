# Action Bar and Shortcut Display UX Patterns

> Research compiled: January 2026
> Purpose: Guide Script Kit's action bar implementation based on proven patterns from Raycast, Alfred, and modern productivity apps

## Table of Contents

1. [Raycast Action Bar Design](#raycast-action-bar-design)
2. [Contextual Actions Based on Selection](#contextual-actions-based-on-selection)
3. [Shortcut Key Display Conventions](#shortcut-key-display-conventions)
4. [Action Discoverability Patterns](#action-discoverability-patterns)
5. [Command Palette Best Practices](#command-palette-best-practices)
6. [Implementation Recommendations for Script Kit](#implementation-recommendations-for-script-kit)

---

## Raycast Action Bar Design

### Action Bar Layout

Raycast's Action Bar is positioned at the bottom of the window and consists of two parts:

| Section | Content |
|---------|---------|
| **Left** | Navigation title of active command + any toast notifications |
| **Right** | Available actions with keyboard shortcuts |

The action bar was introduced in Raycast v1.38.0 as a key discoverability feature. It shows actions available in the current context along with their keyboard shortcuts, helping users learn shortcuts to become more efficient over time.

### Primary and Secondary Actions

Actions automatically receive default keyboard shortcuts:

| View Type | Primary Action | Secondary Action |
|-----------|----------------|------------------|
| List, Grid, Detail | `Enter` | `Cmd+Enter` |
| Form | `Cmd+Enter` | `Cmd+Shift+Enter` |

The first two actions in an Action Panel become the primary and secondary actions. Custom shortcuts can be assigned but won't display in the UI.

### Accessing Actions

- **Command+K**: Opens the full Action Panel for searching/browsing all actions
- **Enter**: Executes the primary action (context-aware)
- **Action Bar Button**: Click to reveal the full actions popup

### Compact Mode

Raycast offers a Compact Mode that shows just the search bar initially, expanding to full view when typing. However, the Action Bar remains visible even in compact mode, ensuring action discoverability is never compromised.

---

## Contextual Actions Based on Selection

### Context-Aware Action Panel

The Action Panel adapts based on:
- **Selected item type** (file, script, URL, text, etc.)
- **Current view/command** (list, detail, form)
- **User permissions and state** (pinned items, favorites)

Example context variations:
- File selected: Open, Show in Finder, Quick Look, Copy Path
- Script selected: Run, Edit, Copy, Share, Configure
- URL selected: Open, Copy, Share, Preview

### Section Organization

Actions are grouped into semantic sections for easier navigation:

```jsx
<ActionPanel title="Activity Monitor">
  <ActionPanel.Section title="Actions">
    <Action title="Force Quit" shortcut="Cmd+K" />
    <Action title="Inspect" shortcut="Cmd+I" />
  </ActionPanel.Section>
  <ActionPanel.Section title="Copy">
    <Action.CopyToClipboard title="Copy PID" />
    <Action.CopyToClipboard title="Copy Name" />
  </ActionPanel.Section>
  <ActionPanel.Section title="Danger Zone">
    <Action title="Kill Process" style="destructive" />
  </ActionPanel.Section>
</ActionPanel>
```

### Submenu Support

For actions requiring additional selection (e.g., "Assign Label"), Raycast uses ActionPanel.Submenu with:
- Lazy loading via `onOpen` callback
- Auto-focus when parent panel opens
- Support for filtering within submenus

---

## Shortcut Key Display Conventions

### macOS Modifier Symbols

Standard macOS keyboard symbols should be used for consistency:

| Symbol | Key | Description |
|--------|-----|-------------|
| `⌘` | Command | Primary modifier |
| `⌃` | Control | Secondary modifier |
| `⌥` | Option/Alt | Alternative modifier |
| `⇧` | Shift | Shift modifier |
| `↵` | Return/Enter | Execute/confirm |
| `⎋` | Escape | Cancel/close |
| `⇥` | Tab | Move focus |
| `⌫` | Delete/Backspace | Delete |
| `␣` | Space | Space key |
| `↑` `↓` `←` `→` | Arrows | Navigation |
| `⇞` `⇟` | Page Up/Down | Scroll |
| `↖` `↘` | Home/End | Jump to start/end |

### Symbol Order Convention

When displaying compound shortcuts, modifiers appear in this order:
```
Control + Option + Shift + Command + Key
⌃⌥⇧⌘K
```

### Visual Design for Keycaps

Modern apps display shortcuts using styled "keycap" elements:

```css
/* Keycap styling pattern */
.keycap {
  min-width: 20px;
  height: 22px;
  padding: 0 6px;
  background: rgba(128, 128, 128, 0.3);
  border: 1px solid rgba(128, 128, 128, 0.5);
  border-radius: 5px;
  font-size: 12px;
  font-weight: 500;
}
```

Design considerations:
- **Light mode**: Light gray background (e.g., `#E8E8E8`) with dark text (`#6B6B6B`)
- **Dark mode**: Semi-transparent dark background with light text
- **Spacing**: 3-4px gap between individual keycaps
- **Alignment**: Right-aligned in action rows

### Displaying in Tooltips

Shortcuts can be revealed progressively:
1. **In action rows**: Always visible for discoverable actions
2. **In tooltips**: Show on hover for toolbar buttons
3. **In menus**: Standard menu item suffix (e.g., "Save...⌘S")

---

## Action Discoverability Patterns

### Layered Discoverability

Modern apps use multiple layers to help users discover actions:

| Layer | Visibility | Purpose |
|-------|------------|---------|
| **Action Bar** | Always visible | Show primary actions + entry point |
| **Tooltips** | On hover | Bind shortcuts to visual affordances |
| **Action Panel** | On demand (Cmd+K) | Full searchable action list |
| **Shortcut Reference** | On demand (?) | Complete keyboard shortcut list |

### Raycast Approach

1. **Action Bar Footer**: Shows primary action + "Actions" button
2. **Cmd+K Panel**: Searchable list of all actions with shortcuts
3. **Learning Loop**: Users see shortcuts, eventually memorize them

### Alfred Approach

1. **Universal Actions** (Option+Cmd+\): Pop up actions for any selection
2. **60+ Built-in Actions**: Copy, paste, search, open, extract URLs
3. **Workflow Actions**: Custom actions appear in the panel
4. **Arrow Key Access**: Right arrow opens action panel on selected item

### Superhuman/Linear Approach

1. **Cmd+K Command Palette**: Central hub for all actions
2. **Inline Hints**: Shortcuts displayed next to commands
3. **Muscle Memory Training**: App "admonishes" mouse use, teaching keyboard
4. **Context-Aware Suggestions**: Recent and relevant actions first

### Best Practices for Discoverability

1. **Show shortcuts alongside actions**: Users learn by seeing
2. **Use fuzzy search**: "picture" should find "image" command
3. **Display recent actions**: Reduce friction for repeated tasks
4. **Progressive disclosure**: Start simple, reveal complexity on demand
5. **Visual feedback**: Confirm action execution with subtle animation/toast

---

## Command Palette Best Practices

### Core Design Principles

From Superhuman's command palette research:

1. **Universal Access**: Same shortcut (Cmd+K) works everywhere in the app
2. **Quick Dismiss**: Same shortcut closes the palette for fast recovery
3. **Unlimited Features**: No UI real estate constraints
4. **Keyboard-First**: Users stay in flow without reaching for mouse

### Search vs. Command Palette

Key distinction:
- **Search**: Finding content (files, items, data)
- **Command Palette**: Taking actions (create, delete, configure)

Some apps merge these (Notion, VS Code), while others keep them separate (Superhuman, Linear).

### Effective Command Palette Features

| Feature | Description |
|---------|-------------|
| **Fuzzy Search** | Match against title, keywords, aliases |
| **Recent Actions** | Show last-used commands at top |
| **Keyboard Shortcuts Display** | Train users on faster paths |
| **Context Awareness** | Show relevant actions for current state |
| **Sections/Categories** | Organize many actions logically |
| **Inline Parameters** | Handle simple inputs in the palette |

### When to Use Command Palettes

**Good fit:**
- Apps with many features/commands
- Power users who prefer keyboard
- Complex workflows with many steps
- Products where speed matters

**Poor fit:**
- Simple apps with few actions
- Infrequent users who won't learn shortcuts
- Mobile-first applications

---

## Implementation Recommendations for Script Kit

### 1. Action Bar Footer Design

Based on Raycast's proven pattern, Script Kit should implement:

```
+--------------------------------------------------+
| [List Items...]                                   |
|                                                   |
+--------------------------------------------------+
| Search actions...  [↵ Run] [⌘↵ Edit] [⌘K Actions] |
+--------------------------------------------------+
```

**Footer Components:**
- **Left**: Search input (Raycast-style, minimal styling)
- **Right**: Primary action with shortcut + Secondary action + Actions button

### 2. Contextual Action Sets

Define context-specific action sets:

| Context | Primary | Secondary | Additional Actions |
|---------|---------|-----------|-------------------|
| Script | Run (↵) | Edit (⌘↵) | Copy, Share, Favorite, Configure |
| File | Open (↵) | Show in Finder (⌘↵) | Quick Look, Copy Path, Open With |
| Clipboard | Paste (↵) | Copy (⌘↵) | Pin, Delete, Share |
| Chat/AI | Send (↵) | Continue (⌘↵) | Model Select, Clear, Copy |

### 3. Keyboard Shortcut Implementation

```rust
// Action definition with shortcut
struct Action {
    id: String,
    title: String,
    shortcut: Option<String>,  // e.g., "cmd+shift+c"
    category: ActionCategory,
    section: Option<String>,
    icon: Option<Icon>,
}

// Shortcut parsing and display
fn format_shortcut(shortcut: &str) -> String {
    // "cmd+shift+c" -> "⌘⇧C"
    shortcut.split('+')
        .map(|part| match part.trim().to_lowercase().as_str() {
            "cmd" | "command" => "⌘",
            "ctrl" | "control" => "⌃",
            "alt" | "opt" | "option" => "⌥",
            "shift" => "⇧",
            "enter" | "return" => "↵",
            "escape" | "esc" => "⎋",
            key => key.to_uppercase(),
        })
        .collect()
}
```

### 4. Keycap Visual Design

Implement consistent keycap styling:

```rust
fn render_keycap(key: &str, theme: &Theme) -> impl IntoElement {
    let (bg, border, text) = if theme.is_dark() {
        (rgba(0x80808080), rgba(0x808080A0), theme.text.dimmed)
    } else {
        (rgba(0xE8E8E8FF), rgba(0xE8E8E8FF), rgba(0x6B6B6BFF))
    };

    div()
        .min_w(px(20.0))
        .h(px(22.0))
        .px(px(6.0))
        .bg(bg)
        .border_1()
        .border_color(border)
        .rounded(px(5.0))
        .text_xs()
        .text_color(text)
        .flex()
        .items_center()
        .justify_center()
        .child(key)
}
```

### 5. Action Panel Trigger

Implement Cmd+K action panel with:

1. **Searchable action list**: Fuzzy match against title/keywords
2. **Section headers**: Group by category (Actions, Copy, Navigation, etc.)
3. **Shortcut display**: Keycaps right-aligned on each row
4. **Context awareness**: Show relevant actions for current selection
5. **Recent actions**: Remember frequently used actions

### 6. Progressive Learning System

Help users discover and learn shortcuts:

1. **Action bar visibility**: Always show shortcuts on primary actions
2. **Tooltip hints**: Show shortcuts on hover for buttons/icons
3. **Cmd+K education**: Users see shortcuts while searching
4. **Keyboard-first prompts**: Encourage keyboard over mouse

### 7. Accessibility Considerations

- **Focus trapping**: Keep focus within action panel when open
- **ARIA roles**: Announce panel state changes
- **Visible focus indicators**: Clear focus ring on selected action
- **Color contrast**: Ensure shortcuts readable in both themes
- **Keyboard-only operation**: All actions accessible without mouse

### 8. Recommended Shortcut Assignments

| Action | Shortcut | Notes |
|--------|----------|-------|
| Primary action | `↵` | Context-dependent (Run, Open, Paste) |
| Secondary action | `⌘↵` | Context-dependent (Edit, Show, Copy) |
| Open action panel | `⌘K` | Raycast/Slack convention |
| Quick select 1-9 | `1-9` | Direct item selection |
| Navigate up/down | `↑/↓` or `j/k` | Support Vim bindings optionally |
| Cancel/close | `⎋` | Universal escape |
| Toggle favorite | `⌘D` | Like Chrome bookmarks |
| Copy to clipboard | `⌘C` | Standard, when applicable |
| Edit selected | `⌘E` | Quick edit access |
| Show in Finder | `⌘⇧F` | Reveal in filesystem |

---

## Sources

### Raycast
- [Raycast Action Panel API](https://developers.raycast.com/api-reference/user-interface/action-panel)
- [Raycast Actions API](https://developers.raycast.com/api-reference/user-interface/actions)
- [Raycast Action Panel Manual](https://manual.raycast.com/action-panel)
- [Raycast Keyboard Shortcuts](https://manual.raycast.com/keyboard-shortcuts)
- [Raycast v1.38.0 - A Fresh Look and Feel](https://www.raycast.com/blog/a-fresh-look-and-feel)
- [Raycast for macOS gets new UI, Action Bar](https://www.ghacks.net/2022/07/20/raycast-for-macos-gets-a-new-ui-action-bar-and-compact-mode/)

### Alfred
- [Alfred Universal Actions](https://www.alfredapp.com/help/features/universal-actions/)
- [Alfred Universal Actions Feature Page](https://www.alfredapp.com/universal-actions/)
- [Alfred Hotkey Workflows](https://www.alfredapp.com/help/workflows/triggers/hotkey/creating-a-hotkey-workflow/)

### Command Palette Design
- [How to Build a Remarkable Command Palette - Superhuman](https://blog.superhuman.com/how-to-build-a-remarkable-command-palette/)
- [Command Palette UI Design - Mobbin](https://mobbin.com/glossary/command-palette)
- [How to Design Great Keyboard Shortcuts - Knock](https://knock.app/blog/how-to-design-great-keyboard-shortcuts)
- [The Art of Keyboard Shortcuts - Medium](https://medium.com/design-bootcamp/the-art-of-keyboard-shortcuts-designing-for-speed-and-efficiency-9afd717fc7ed)
- [Designing Command Palettes - Retool](https://retool.com/blog/designing-the-command-palette)
- [From Counter-Strike to Keyboard Shortcuts - Pitch](https://pitch.com/blog/from-counter-strike-to-keyboard-shortcuts)

### Mac Conventions
- [Mac Keyboard Shortcuts - Apple Support](https://support.apple.com/en-us/102650)
- [Mac Menu Symbols - Apple Support](https://support.apple.com/guide/mac-help/what-are-those-symbols-shown-in-menus-cpmh0011/mac)
- [Making Sense of Mac Keyboard Symbols](https://osxdaily.com/2012/03/27/making-sense-of-mac-keyboard-symbols/)

### Launcher Comparisons
- [Spotlight vs Alfred vs Raycast - Medium](https://medium.com/@andriizolkin/spotlight-vs-alfred-vs-raycast-31bd942ac3b6)
- [Alfred vs Raycast: Ultimate Face-Off - Medium](https://medium.com/the-mac-alchemist/alfred-vs-raycast-the-ultimate-launcher-face-off-855dc0afec89)
- [Raycast for Mac: The Next-Generation Alfred - The Sweet Setup](https://thesweetsetup.com/raycast-for-mac-the-next-generation-alfred/)
