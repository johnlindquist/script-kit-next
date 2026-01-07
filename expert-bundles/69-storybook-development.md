# Storybook Development - Expert Bundle

## Overview

Script Kit includes a built-in storybook system for developing and testing UI components in isolation.

## Storybook Architecture

### Story Registry (src/storybook/registry.rs)

```rust
use std::collections::HashMap;

pub type StoryFactory = Box<dyn Fn(&mut Context<StoryBrowser>) -> AnyView>;

pub struct StoryRegistry {
    stories: HashMap<String, Vec<StoryEntry>>,
}

pub struct StoryEntry {
    pub name: String,
    pub description: Option<String>,
    pub factory: StoryFactory,
}

impl StoryRegistry {
    pub fn new() -> Self {
        Self {
            stories: HashMap::new(),
        }
    }

    pub fn register(&mut self, category: &str, name: &str, factory: StoryFactory) {
        let entry = StoryEntry {
            name: name.to_string(),
            description: None,
            factory,
        };
        
        self.stories
            .entry(category.to_string())
            .or_default()
            .push(entry);
    }

    pub fn categories(&self) -> Vec<&str> {
        let mut cats: Vec<_> = self.stories.keys().map(|s| s.as_str()).collect();
        cats.sort();
        cats
    }

    pub fn stories_in_category(&self, category: &str) -> Option<&Vec<StoryEntry>> {
        self.stories.get(category)
    }
}
```

### Story Browser (src/storybook/browser.rs)

```rust
pub struct StoryBrowser {
    registry: Arc<StoryRegistry>,
    selected_category: Option<String>,
    selected_story: Option<usize>,
    current_view: Option<AnyView>,
    focus_handle: FocusHandle,
}

impl StoryBrowser {
    pub fn new(registry: Arc<StoryRegistry>, cx: &mut Context<Self>) -> Self {
        Self {
            registry,
            selected_category: None,
            selected_story: None,
            current_view: None,
            focus_handle: cx.focus_handle(),
        }
    }

    fn select_story(&mut self, category: &str, index: usize, cx: &mut Context<Self>) {
        self.selected_category = Some(category.to_string());
        self.selected_story = Some(index);
        
        if let Some(stories) = self.registry.stories_in_category(category) {
            if let Some(entry) = stories.get(index) {
                self.current_view = Some((entry.factory)(cx));
            }
        }
        
        cx.notify();
    }
}

impl Render for StoryBrowser {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .w_full()
            .h_full()
            // Sidebar
            .child(
                div()
                    .w(px(250.0))
                    .h_full()
                    .border_r_1()
                    .bg(rgb(0x1E1E1E))
                    .child(self.render_sidebar(cx))
            )
            // Content area
            .child(
                div()
                    .flex_1()
                    .h_full()
                    .p_4()
                    .child(self.render_content(cx))
            )
    }
}
```

## Creating Stories

### Story Module Pattern

```rust
// src/stories/button_stories.rs
use super::*;

pub fn register_button_stories(registry: &mut StoryRegistry) {
    registry.register("Buttons", "Primary Button", Box::new(|cx| {
        cx.new(|_| PrimaryButtonStory).into_any()
    }));
    
    registry.register("Buttons", "Secondary Button", Box::new(|cx| {
        cx.new(|_| SecondaryButtonStory).into_any()
    }));
    
    registry.register("Buttons", "Icon Button", Box::new(|cx| {
        cx.new(|_| IconButtonStory).into_any()
    }));
    
    registry.register("Buttons", "Button States", Box::new(|cx| {
        cx.new(|_| ButtonStatesStory).into_any()
    }));
}

struct PrimaryButtonStory;

impl Render for PrimaryButtonStory {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_4()
            .p_8()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::BOLD)
                    .child("Primary Button")
            )
            .child(
                Button::new("click-me")
                    .label("Click Me")
                    .primary()
            )
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(0x888888))
                    .child("Use for primary actions")
            )
    }
}
```

### Variation Stories

```rust
// src/stories/list_item_state_variations.rs
pub fn register_list_item_stories(registry: &mut StoryRegistry) {
    registry.register("List Items", "State Variations", Box::new(|cx| {
        cx.new(|_| ListItemStateVariations).into_any()
    }));
}

struct ListItemStateVariations;

impl Render for ListItemStateVariations {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let states = [
            ("Normal", false, false),
            ("Hovered", true, false),
            ("Selected", false, true),
            ("Selected + Hovered", true, true),
        ];
        
        div()
            .flex()
            .flex_col()
            .gap_2()
            .p_4()
            .children(states.map(|(label, hovered, selected)| {
                div()
                    .flex()
                    .items_center()
                    .gap_4()
                    .child(
                        div()
                            .w(px(150.0))
                            .text_sm()
                            .child(label)
                    )
                    .child(
                        render_list_item("Example Item", hovered, selected)
                    )
            }))
    }
}
```

## Registration

### Mod File (src/stories/mod.rs)

```rust
mod button_stories;
mod list_item_stories;
mod header_stories;
mod footer_stories;
mod form_field_stories;
mod toast_stories;
mod design_token_stories;

pub use button_stories::*;
pub use list_item_stories::*;
pub use header_stories::*;
pub use footer_stories::*;
pub use form_field_stories::*;
pub use toast_stories::*;
pub use design_token_stories::*;

pub fn register_all_stories(registry: &mut StoryRegistry) {
    button_stories::register_button_stories(registry);
    list_item_stories::register_list_item_stories(registry);
    header_stories::register_header_stories(registry);
    footer_stories::register_footer_stories(registry);
    form_field_stories::register_form_field_stories(registry);
    toast_stories::register_toast_stories(registry);
    design_token_stories::register_design_token_stories(registry);
}
```

## Running Storybook

### Launch Command

```bash
# Via stdin protocol
echo '{"type":"openStorybook"}' | ./target/debug/script-kit-gpui
```

### Main Integration

```rust
// In main.rs or window setup
fn open_storybook(cx: &mut AppContext) {
    let mut registry = StoryRegistry::new();
    stories::register_all_stories(&mut registry);
    
    let options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(
            Bounds::centered(None, size(px(1200.0), px(800.0)), cx)
        )),
        titlebar: Some(TitlebarOptions {
            title: Some("Script Kit Storybook".into()),
            ..Default::default()
        }),
        ..Default::default()
    };
    
    let _ = cx.open_window(options, |window, cx| {
        let browser = cx.new(|cx| {
            StoryBrowser::new(Arc::new(registry), cx)
        });
        cx.new(|cx| Root::new(browser, window, cx))
    });
}
```

## Design Token Stories

```rust
// src/stories/design_token_stories.rs
pub fn register_design_token_stories(registry: &mut StoryRegistry) {
    registry.register("Design", "Color Palette", Box::new(|cx| {
        cx.new(|_| ColorPaletteStory).into_any()
    }));
    
    registry.register("Design", "Typography", Box::new(|cx| {
        cx.new(|_| TypographyStory).into_any()
    }));
    
    registry.register("Design", "Spacing", Box::new(|cx| {
        cx.new(|_| SpacingStory).into_any()
    }));
}

struct ColorPaletteStory;

impl Render for ColorPaletteStory {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let colors = [
            ("Background", 0x1E1E1E),
            ("Surface", 0x252526),
            ("Primary", 0x3B82F6),
            ("Secondary", 0x6366F1),
            ("Success", 0x22C55E),
            ("Warning", 0xF59E0B),
            ("Error", 0xEF4444),
            ("Text Primary", 0xE4E4E7),
            ("Text Secondary", 0xA1A1AA),
        ];
        
        div()
            .flex()
            .flex_wrap()
            .gap_4()
            .p_4()
            .children(colors.map(|(name, color)| {
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .w(px(80.0))
                            .h(px(80.0))
                            .rounded_lg()
                            .bg(rgb(color))
                    )
                    .child(
                        div().text_sm().child(name)
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0x888888))
                            .child(format!("#{:06X}", color))
                    )
            }))
    }
}
```

## Interactive Stories

```rust
struct InteractiveButtonStory {
    click_count: usize,
}

impl InteractiveButtonStory {
    fn new() -> Self {
        Self { click_count: 0 }
    }
}

impl Render for InteractiveButtonStory {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let count = self.click_count;
        
        div()
            .flex()
            .flex_col()
            .gap_4()
            .p_4()
            .child(
                Button::new("counter")
                    .label(format!("Clicked {} times", count))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.click_count += 1;
                        cx.notify();
                    }))
            )
            .child(
                Button::new("reset")
                    .label("Reset")
                    .secondary()
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.click_count = 0;
                        cx.notify();
                    }))
            )
    }
}
```

## Story Layout

```rust
// src/storybook/layout.rs

pub fn story_container(title: &str, content: impl IntoElement) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_6()
        .p_8()
        .child(
            div()
                .text_xl()
                .font_weight(FontWeight::BOLD)
                .pb_4()
                .border_b_1()
                .border_color(rgb(0x333333))
                .child(title)
        )
        .child(content)
}

pub fn story_section(title: &str, content: impl IntoElement) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_3()
        .child(
            div()
                .text_sm()
                .font_weight(FontWeight::MEDIUM)
                .text_color(rgb(0x888888))
                .child(title)
        )
        .child(content)
}

pub fn story_row(items: Vec<impl IntoElement>) -> impl IntoElement {
    div()
        .flex()
        .gap_4()
        .flex_wrap()
        .children(items)
}
```

## Best Practices

1. **One story per state** - Makes variations clear
2. **Use descriptive names** - "Primary Button Disabled" not "Button 3"
3. **Group by component** - Categories help navigation
4. **Include edge cases** - Empty states, long text, etc.
5. **Add descriptions** - Document usage patterns
6. **Make interactive** - Clickable/editable when possible

## Summary

| Component | Location | Purpose |
|-----------|----------|---------|
| Registry | `storybook/registry.rs` | Store stories |
| Browser | `storybook/browser.rs` | Navigation UI |
| Stories | `stories/*.rs` | Component demos |
| Layout | `storybook/layout.rs` | Shared helpers |
