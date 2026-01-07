# Script Kit GPUI - Expert Bundle 43: Shared UI Components

## Project Context

Script Kit GPUI is a **Rust desktop app** built with GPUI (Zed's UI framework) that serves as a command launcher and script runner. Think: Raycast/Alfred but scriptable with TypeScript.

**Key UI Surfaces:**
- Main menu with script list, clipboard history, app launcher, window switcher
- Prompt dialogs (arg, div, editor, path, form, env, drop, template, select)
- Secondary windows (Notes, AI Chat)
- Overlays (Actions dialog, HUD notifications)

---

## Goal

Create a **unified component library** for elements shared across multiple list views:
1. List items (script items, clipboard entries, app icons, window items)
2. Section headers (RECENT, SUGGESTED, MAIN, etc.)
3. Empty states and loading indicators
4. Search/filter inputs with consistent styling

---

## Current State

### List Item Implementations

| View | File | Lines | Implementation |
|------|------|-------|----------------|
| Main Menu | `list_item.rs` | ~400 | `ListItem` component + `GroupedListItem` enum |
| Clipboard | `render_builtins.rs` | ~200 | Inline div construction |
| App Launcher | `render_builtins.rs` | ~150 | Inline div construction |
| Window Switcher | `render_builtins.rs` | ~100 | Inline div construction |
| Actions Dialog | `actions.rs` | ~150 | Custom `ActionItem` struct |

### Common Patterns Duplicated

```rust
// Pattern 1: List item container (appears 15+ times)
div()
    .id(ElementId::NamedInteger("item".into(), ix as u64))
    .w_full()
    .h(px(48.))  // or 52, or LIST_ITEM_HEIGHT
    .px(px(16.))
    .flex()
    .flex_row()
    .items_center()
    .gap(px(12.))
    .when(is_selected, |d| d.bg(rgb(selected_bg)))
    .when(is_hovered && !is_selected, |d| d.bg(rgb(hover_bg)))
    .rounded(px(6.))

// Pattern 2: Icon container (appears 10+ times)
div()
    .size(px(32.))
    .flex()
    .items_center()
    .justify_center()
    .rounded(px(6.))
    .bg(rgb(icon_bg))
    .child(/* icon content */)

// Pattern 3: Text content (appears 20+ times)
div()
    .flex_1()
    .overflow_hidden()
    .flex()
    .flex_col()
    .justify_center()
    .child(
        div()
            .text_sm()
            .font_weight(FontWeight::MEDIUM)
            .text_color(rgb(text_primary))
            .text_ellipsis()
            .child(title)
    )
    .when_some(description, |d, desc| {
        d.child(
            div()
                .text_xs()
                .text_color(rgb(text_muted))
                .text_ellipsis()
                .child(desc)
        )
    })

// Pattern 4: Keyboard shortcut badge (appears 8+ times)
div()
    .px(px(6.))
    .py(px(2.))
    .rounded(px(4.))
    .bg(rgba((badge_bg << 8) | 0x40))
    .text_xs()
    .text_color(rgb(badge_text))
    .child(shortcut)
```

---

## Proposed Component Library

### 1. UnifiedListItem

```rust
/// A consistent list item component for all list views
#[derive(IntoElement)]
pub struct UnifiedListItem {
    /// Unique identifier for the item
    id: ElementId,
    /// Primary text (title)
    title: SharedString,
    /// Secondary text (description, path, etc.)
    subtitle: Option<SharedString>,
    /// Leading element (icon, image, avatar)
    leading: Option<LeadingContent>,
    /// Trailing element (shortcut badge, count, chevron)
    trailing: Option<TrailingContent>,
    /// Visual state
    state: ItemState,
    /// Colors from theme
    colors: ListItemColors,
}

#[derive(Clone)]
pub enum LeadingContent {
    /// SF Symbol or custom icon
    Icon { name: IconName, color: Option<u32> },
    /// App icon from bundle
    AppIcon { path: PathBuf },
    /// Clipboard content preview
    Preview { content_type: PreviewType },
    /// Custom element
    Custom(Box<dyn Fn() -> AnyElement>),
}

#[derive(Clone)]
pub enum TrailingContent {
    /// Keyboard shortcut badge
    Shortcut(String),
    /// Count badge (e.g., "12 items")
    Count(usize),
    /// Chevron for navigation
    Chevron,
    /// Checkmark for selected items
    Checkmark,
    /// Custom element
    Custom(Box<dyn Fn() -> AnyElement>),
}

#[derive(Clone, Copy)]
pub struct ItemState {
    pub is_selected: bool,
    pub is_hovered: bool,
    pub is_disabled: bool,
    pub is_loading: bool,
}

impl UnifiedListItem {
    pub fn new(id: impl Into<ElementId>, title: impl Into<SharedString>) -> Self { ... }
    
    pub fn subtitle(mut self, s: impl Into<SharedString>) -> Self { ... }
    pub fn leading(mut self, l: LeadingContent) -> Self { ... }
    pub fn trailing(mut self, t: TrailingContent) -> Self { ... }
    pub fn state(mut self, s: ItemState) -> Self { ... }
    pub fn colors(mut self, c: ListItemColors) -> Self { ... }
    
    pub fn on_click(mut self, handler: impl Fn(&ClickEvent) + 'static) -> Self { ... }
    pub fn on_hover(mut self, handler: impl Fn(bool) + 'static) -> Self { ... }
}
```

### 2. SectionHeader

```rust
/// Consistent section header for grouped lists
#[derive(IntoElement)]
pub struct SectionHeader {
    label: SharedString,
    /// Optional count (e.g., "SCRIPTS (42)")
    count: Option<usize>,
    /// Collapsible state
    collapsible: Option<CollapsibleState>,
    /// Visual style
    style: SectionStyle,
    colors: ListItemColors,
}

pub enum SectionStyle {
    /// Uppercase text, left-aligned (default)
    UppercaseLeft,
    /// With count badge
    WithCount,
    /// With icon prefix
    WithIcon(IconName),
    /// Collapsible with chevron
    Collapsible,
}

pub struct CollapsibleState {
    pub is_collapsed: bool,
    pub on_toggle: Box<dyn Fn(bool)>,
}

impl SectionHeader {
    pub fn new(label: impl Into<SharedString>) -> Self { ... }
    pub fn count(mut self, c: usize) -> Self { ... }
    pub fn collapsible(mut self, collapsed: bool, on_toggle: impl Fn(bool)) -> Self { ... }
    pub fn style(mut self, s: SectionStyle) -> Self { ... }
}
```

### 3. EmptyState

```rust
/// Consistent empty state for lists with no items
#[derive(IntoElement)]
pub struct EmptyState {
    /// Primary message
    message: SharedString,
    /// Optional icon
    icon: Option<IconName>,
    /// Optional action button
    action: Option<EmptyStateAction>,
    colors: EmptyStateColors,
}

pub struct EmptyStateAction {
    label: String,
    on_click: Box<dyn Fn()>,
}

impl EmptyState {
    pub fn new(message: impl Into<SharedString>) -> Self { ... }
    
    /// Create for "no results" state
    pub fn no_results(query: &str) -> Self {
        Self::new(format!("No results match '{}'", query))
            .icon(IconName::Search)
    }
    
    /// Create for "empty list" state  
    pub fn empty_list(item_type: &str) -> Self {
        Self::new(format!("No {} found", item_type))
    }
}
```

### 4. VirtualizedList

```rust
/// Wrapper around GPUI's list() with consistent behavior
pub struct VirtualizedList<T> {
    items: Vec<T>,
    /// Height per item (supports variable heights via callback)
    item_height: ItemHeight,
    /// Render callback
    render_item: Box<dyn Fn(&T, usize, ItemState) -> AnyElement>,
    /// Scroll handle for programmatic scrolling
    scroll_handle: UniformListScrollHandle,
    /// Scrollbar configuration
    scrollbar: ScrollbarConfig,
}

pub enum ItemHeight {
    /// Fixed height for all items
    Fixed(f32),
    /// Variable heights with measurement callback
    Variable(Box<dyn Fn(&T) -> f32>),
    /// Auto-detect from content
    Auto,
}

impl<T> VirtualizedList<T> {
    pub fn new(items: Vec<T>) -> Self { ... }
    
    pub fn item_height(mut self, h: ItemHeight) -> Self { ... }
    pub fn render(mut self, f: impl Fn(&T, usize, ItemState) -> AnyElement) -> Self { ... }
    
    /// Scroll to bring item into view
    pub fn scroll_to(&self, index: usize) { ... }
    
    /// Get current scroll position
    pub fn scroll_offset(&self) -> usize { ... }
}
```

---

## Usage Examples

### Main Menu Script List
```rust
VirtualizedList::new(self.scripts.clone())
    .item_height(ItemHeight::Fixed(LIST_ITEM_HEIGHT))
    .render(|script, ix, state| {
        UnifiedListItem::new(("script", ix), &script.name)
            .subtitle(script.description.as_deref())
            .leading(LeadingContent::Icon { 
                name: IconName::Terminal, 
                color: None 
            })
            .trailing_if(script.shortcut.is_some(), || {
                TrailingContent::Shortcut(script.shortcut.clone().unwrap())
            })
            .state(state)
            .colors(theme_colors)
            .into_any()
    })
```

### Clipboard History
```rust
VirtualizedList::new(self.clipboard_entries.clone())
    .item_height(ItemHeight::Fixed(52.))
    .render(|entry, ix, state| {
        UnifiedListItem::new(("clipboard", ix), entry.preview_text())
            .subtitle(entry.timestamp_display())
            .leading(LeadingContent::Preview { 
                content_type: entry.content_type() 
            })
            .trailing(TrailingContent::Shortcut(format!("Cmd+{}", ix + 1)))
            .state(state)
            .colors(theme_colors)
            .into_any()
    })
```

### App Launcher
```rust
VirtualizedList::new(self.apps.clone())
    .item_height(ItemHeight::Fixed(48.))
    .render(|app, ix, state| {
        UnifiedListItem::new(("app", ix), &app.name)
            .leading(LeadingContent::AppIcon { path: app.icon_path.clone() })
            .state(state)
            .colors(theme_colors)
            .into_any()
    })
```

---

## Implementation Checklist

- [ ] Create `src/components/list_item.rs` with `UnifiedListItem`
- [ ] Create `src/components/section_header.rs` with `SectionHeader`
- [ ] Create `src/components/empty_state.rs` with `EmptyState`
- [ ] Create `src/components/virtualized_list.rs` wrapper
- [ ] Extract `LeadingContent` variants for all content types
- [ ] Extract `TrailingContent` variants (shortcut, count, chevron)
- [ ] Migrate main menu to use `UnifiedListItem`
- [ ] Migrate clipboard history to use `UnifiedListItem`
- [ ] Migrate app launcher to use `UnifiedListItem`
- [ ] Migrate window switcher to use `UnifiedListItem`
- [ ] Migrate actions dialog to use `UnifiedListItem`
- [ ] Add unit tests for component rendering
- [ ] Document component API in module docs

---

## Key Questions

1. Should `UnifiedListItem` handle its own keyboard navigation hints?
2. How to handle match highlighting (fuzzy search) in the title?
3. Should icons be lazy-loaded for performance (app icons)?
4. How to handle accessibility labels for screen readers?
5. Should there be a "compact" variant for denser lists?

---

## Related Bundles

- Bundle 40: UI Layout Helpers - provides stack primitives
- Bundle 42: App Shell Architecture - wraps these components
- Bundle 50: Search/Filtering - integrates with match highlighting
