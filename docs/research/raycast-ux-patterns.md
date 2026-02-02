# Raycast UX Patterns Research

> Research compiled: January 2026
> Purpose: Guide Script Kit's UX implementation based on Raycast's proven patterns

## Table of Contents

1. [Design Philosophy](#design-philosophy)
2. [Keyboard Navigation](#keyboard-navigation)
3. [Search Behavior](#search-behavior)
4. [Visual Feedback & Responsiveness](#visual-feedback--responsiveness)
5. [State Management](#state-management)
6. [Actions & Commands](#actions--commands)
7. [List & Item Display](#list--item-display)
8. [Implementation Recommendations for Script Kit](#implementation-recommendations-for-script-kit)

---

## Design Philosophy

Raycast operates on three core design principles:

1. **Fast** - Every interaction should feel instant
2. **Simple** - Minimal UI that gets out of the way
3. **Delightful** - Small details that make the experience enjoyable

### Keyboard-First Paradigm

Raycast is fundamentally keyboard-first. While mouse interaction is supported, the real productivity comes from keyboard shortcuts. The less you reach for your mouse, the better.

> "The keyboard is the ultimate productivity tool, and it's much faster to hit a keyboard shortcut to launch an app, resize a window, search files or run a script than to reach for the mouse and click on an icon or menu."

### Consistent UX Language

All dialogs in Raycast follow a standard layout. Whether you're using the emoji picker, clipboard manager, or a third-party extension, the UX patterns remain identical:
- Same filtering behavior
- Same keyboard shortcuts
- Same action panel patterns
- Same visual hierarchy

**Script Kit Implementation:** Ensure all prompts (arg, select, form, etc.) share identical interaction patterns and keyboard shortcuts.

---

## Keyboard Navigation

### Core Navigation Keys

| Key | Action |
|-----|--------|
| `Up/Down` | Navigate items |
| `Enter` | Execute primary action |
| `Cmd+Enter` | Execute secondary action |
| `Cmd+K` | Open action panel |
| `Escape` | Go back / Close |
| `Tab` | Move between sections |
| `Cmd+P` | Open filter dropdown |

### Number-Based Quick Selection

One of Raycast's "killer features" is accessing items by their position number. Users can press `1-9` to instantly select items in the list without arrow-key navigation.

> "The winning feature - that is so good that I keep trying to use it on other non-Raycast apps - is that an option can be accessed by its number in the list."

### Assignable Shortcuts

Every command and action can have a custom keyboard shortcut assigned. This creates muscle memory for frequently used actions.

**Script Kit Implementation:**
- Support `1-9` quick selection for list items
- Display position numbers alongside items
- Allow users to assign global hotkeys to scripts
- Ensure consistent shortcuts across all prompt types

---

## Search Behavior

### Instant Fuzzy Search

Search results update **instantly** as you type. Raycast uses fuzzy matching against:
- Item `title`
- Item `keywords` (additional searchable terms)

### Built-in vs Custom Filtering

**Built-in filtering** (default):
- Fuzzy matching handled automatically
- Optimal performance out of the box
- Items filtered based on title and keywords

**Custom filtering**:
- Developers can disable built-in filtering
- Implement custom logic via `onSearchTextChange`
- Throttling supported for async operations

### Search Bar Accessories

A dropdown accessory can be added to the search bar for secondary filtering dimensions:
- Activated via `Cmd+P`
- Useful for filtering by category, status, type, etc.

**Script Kit Implementation:**
- Implement fuzzy search as the default
- Support keyword-based searching (hidden search terms)
- Consider search bar accessories for complex filtering
- Throttle search handlers for network/async operations

---

## Visual Feedback & Responsiveness

### What Makes Raycast Feel Fast

1. **Render something immediately** - Show UI skeleton before data loads
2. **Progressive loading** - Start with empty list, fill as data arrives
3. **Visual loading indicators** - `isLoading` prop shows subtle activity
4. **Instant search results** - No debounce delay on local filtering
5. **Native rendering** - Pure macOS performance, no web views

### Animation Philosophy

Raycast uses subtle animations that enhance rather than delay:
- **Ease-out curves** for most animations (feels fast and natural)
- **Ease-in-out** for already-visible content transitions
- **Never use linear** except for infinite loops (marquees)
- Respect `prefers-reduced-motion` accessibility setting

### Loading Indicators

When loading data:
```
isLoading={true} on top-level components (List, Detail, Grid, Form)
```

This shows a subtle loading spinner without blocking UI interaction.

**Script Kit Implementation:**
- Always render UI immediately, even if empty
- Use loading spinners for async operations
- Implement smooth, subtle animations (prefer ease-out)
- Avoid blocking the UI during data fetches

---

## State Management

### Loading States

**Best Practices:**
- Render a React component as quickly as possible
- Start with empty list or static form
- Load data in background to fill the view
- Use `isLoading` prop to signal loading visually

```typescript
// Good: Show empty list immediately, load data async
<List isLoading={isLoading}>
  {items.map(item => <List.Item ... />)}
</List>
```

### Empty States

Customize empty views to:
- Welcome new users
- Guide users when no results match
- Suggest actions when lists are empty

**Components:**
- `List.EmptyView` with icon, title, and description
- Can include action buttons to help users

**Script Kit Implementation:**
- Always provide meaningful empty states
- Include helpful text explaining why it's empty
- Suggest next steps or actions
- Consider placeholder items during initial load

### Error States

**Philosophy:** Don't disrupt user flow for expected errors.

**Best Practices:**
1. Show cached data when network fails
2. Use Toast notifications for errors
3. Handle expected failures gracefully
4. Only show error screens for truly unrecoverable issues

```typescript
// Good: Fallback to cache, show toast
if (networkError && cachedData) {
  showToast({ style: Toast.Style.Failure, title: "Offline - showing cached data" });
  return cachedData;
}
```

**Script Kit Implementation:**
- Implement graceful error handling
- Show toast notifications for non-critical errors
- Cache results where appropriate
- Provide clear, actionable error messages

---

## Actions & Commands

### Action Panel Architecture

The Action Panel (`Cmd+K`) displays context-aware actions based on:
- Current selected item
- Current view/command
- User permissions

### Action Organization

**Primary/Secondary Actions:**
- First action = Primary (activated by `Enter`)
- Second action = Secondary (activated by `Cmd+Enter`)
- Remaining actions in panel

**Sections:**
- Group related actions semantically
- Examples: "Copy Actions", "Open Actions", "Danger Zone"
- Helps users navigate many actions

**Submenus:**
- For actions requiring additional selection
- Example: "Add Label" -> submenu of labels
- Support lazy loading for dynamic options

### Action Styles

| Style | Use Case |
|-------|----------|
| Regular | Standard actions |
| Destructive | Actions requiring caution (delete, remove) |

Destructive actions should show confirmation alerts for irreversible operations.

### Built-in Action Types

Raycast provides standardized actions:
- `Action.OpenInBrowser` - Opens URLs
- `Action.CopyToClipboard` - With optional concealment for sensitive data
- `Action.Paste` - Pastes to frontmost app
- `Action.ShowInFinder` - Reveals in Finder
- `Action.Push` - Navigate to new view
- `Action.PickDate` - Date/time picker
- `Action.Trash` - Move to trash

**Script Kit Implementation:**
- Implement action panel with `Cmd+K`
- Support primary (`Enter`) and secondary (`Cmd+Enter`) actions
- Group actions into sections
- Mark destructive actions visually
- Require confirmation for irreversible actions

---

## List & Item Display

### List Item Anatomy

```
[Icon] Title                    [Accessories...] [Shortcut]
       Subtitle
```

**Components:**
- **Icon** - Visual identifier (left)
- **Title** - Main text (required)
- **Subtitle** - Secondary text
- **Accessories** - Right-aligned metadata
- **Keywords** - Hidden searchable terms

### Accessories

Accessories display supplementary information:

| Type | Description |
|------|-------------|
| Text | Simple string |
| Date | Relative formatting ("now", "1d", "2w") |
| Tag | Colored label with background |
| Icon | Small icon indicator |

All accessories support tooltips for additional context on hover.

### Detail Views

Split-view mode shows rich content alongside the list:
- **Markdown rendering** for formatted content
- **Metadata** section for structured data
- Disable list accessories when detail is shown

**Metadata Components:**
- `Label` - Key-value pairs with optional icons
- `Link` - Clickable URLs
- `TagList` - Grouped colored tags
- `Separator` - Visual grouping

### Sections

Group related items with:
- Section title
- Optional subtitle
- Visual separator

**Script Kit Implementation:**
- Support rich list items with icons, titles, subtitles
- Implement accessories (text, date, tag, icon)
- Support detail/preview panel on the right
- Allow grouping items into sections
- Format dates relatively ("2 hours ago", "yesterday")

---

## Implementation Recommendations for Script Kit

### Priority 1: Keyboard Navigation

1. **Number quick-select** (`1-9` to select items)
2. **Consistent shortcuts** across all prompts:
   - `Enter` = primary action
   - `Cmd+Enter` = secondary action
   - `Cmd+K` = action panel
   - `Escape` = back/close
   - `Up/Down` = navigate
   - `Tab` = cycle sections

3. **Vim-style optional** (`j/k` for up/down)

### Priority 2: Search & Filtering

1. **Instant fuzzy search** with no perceptible delay
2. **Keyword support** for hidden search terms
3. **Throttled async search** for network operations
4. **Filter dropdown** for categorical filtering

### Priority 3: Visual Feedback

1. **Immediate UI render** - never show blank screen
2. **Loading indicators** - subtle spinners during async
3. **Selection highlighting** - clear visual focus
4. **Smooth animations** - ease-out curves, 150-200ms
5. **Reduced motion respect** - honor system preference

### Priority 4: State Handling

1. **Empty states** with helpful messaging and suggested actions
2. **Error toasts** for non-critical failures
3. **Graceful degradation** with cached data
4. **Loading skeletons** for anticipated content

### Priority 5: Actions System

1. **Action panel** triggered by `Cmd+K`
2. **Action sections** for organization
3. **Destructive styling** with confirmation
4. **Submenus** for multi-step actions
5. **Custom shortcuts** per action

### Priority 6: List Excellence

1. **Rich item display** (icon, title, subtitle, accessories)
2. **Relative date formatting**
3. **Colored tags** for status/category
4. **Preview/detail panel** for more information
5. **Section grouping** with headers

### Specific Code Patterns

#### Keyboard Event Handling
```rust
// Match both lowercase and CamelCase key names
match key.as_str() {
    "up" | "arrowup" | "ArrowUp" => handle_up(),
    "down" | "arrowdown" | "ArrowDown" => handle_down(),
    "enter" | "Enter" => handle_primary_action(),
    "escape" | "Escape" => handle_back(),
    "k" if cmd => handle_open_actions(),
    c if c.len() == 1 && c.chars().next().unwrap().is_ascii_digit() => {
        let idx = c.parse::<usize>().unwrap() - 1;
        handle_quick_select(idx);
    }
    _ => {}
}
```

#### Loading State Pattern
```rust
struct ListState {
    items: Vec<Item>,
    is_loading: bool,
    error: Option<String>,
}

// Always render immediately
fn render(&self, cx: &mut ViewContext<Self>) -> impl IntoElement {
    if self.items.is_empty() && !self.is_loading {
        return self.render_empty_state(cx);
    }

    div()
        .when(self.is_loading, |d| d.child(LoadingSpinner))
        .children(self.items.iter().map(|item| self.render_item(item, cx)))
}
```

#### Action Panel Structure
```rust
struct ActionPanel {
    sections: Vec<ActionSection>,
}

struct ActionSection {
    title: Option<String>,
    actions: Vec<Action>,
}

struct Action {
    title: String,
    icon: Option<Icon>,
    shortcut: Option<Shortcut>,
    style: ActionStyle, // Regular | Destructive
    handler: Box<dyn Fn()>,
}
```

---

## Sources

- [Raycast Official Website](https://www.raycast.com/)
- [Raycast API Documentation](https://developers.raycast.com/)
- [Raycast Best Practices](https://developers.raycast.com/information/best-practices)
- [Raycast Actions API](https://developers.raycast.com/api-reference/user-interface/actions)
- [Raycast Action Panel API](https://developers.raycast.com/api-reference/user-interface/action-panel)
- [Raycast List API](https://developers.raycast.com/api-reference/user-interface/list)
- [Raycast User Interface Overview](https://developers.raycast.com/api-reference/user-interface)
- [Raycast Keyboard Shortcuts Manual](https://manual.raycast.com/keyboard-shortcuts)
- [How the Raycast API and Extensions Work](https://www.raycast.com/blog/how-raycast-api-extensions-work)
- [Making Raycast API More Powerful](https://www.raycast.com/blog/making-our-api-more-powerful)
- [A Love Letter to Raycast](https://rmoff.net/2025/12/18/a-love-letter-to-raycast/)
- [Raycast: The Must-Have Productivity App](https://www.stefanimhoff.de/raycast/)
- [Raycast App Overview](https://albertosadde.com/blog/raycast)
