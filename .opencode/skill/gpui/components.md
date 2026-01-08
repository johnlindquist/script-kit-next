# Component Patterns in gpui-component

## Stateless vs Stateful Components

### Stateless Components (RenderOnce)

Simple, predictable building blocks that don't maintain internal state.

```rust
use gpui_component::prelude::*;

// Button - configurable via builder pattern
Button::new("save-btn").primary().label("Save")

// Tag - displays a label with styling
Tag::secondary().child("Draft")

// Icon - renders an icon at specified size
Icon::new(IconName::Check).small()

// Badge - shows a count or status
Badge::new().child("3")
```

### Stateful Components (Entity)

Require `Entity<T>` to manage internal state.

```rust
struct FormView {
    name_input: Entity<InputState>,
    role_select: Entity<SelectState>,
    data_table: Entity<Table<UserRow>>,
}

impl FormView {
    fn new(window: &Window, cx: &mut Context<Self>) -> Self {
        let name_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Enter name")
                .default_value("")
        });
        
        let role_select = cx.new(|cx| {
            SelectState::new(window, cx).placeholder("Select role")
        });
        
        Self { name_input, role_select, data_table: cx.new(|cx| Table::new(cx)) }
    }
}
```

## Component Reference

| Component | Type | Description |
|-----------|------|-------------|
| `Button` | Stateless | Clickable action trigger |
| `Icon` | Stateless | SVG icon display |
| `Tag` | Stateless | Colored label/chip |
| `Badge` | Stateless | Count/status indicator |
| `Input` | Stateful | Text input field |
| `Select` | Stateful | Dropdown selection |
| `Table` | Stateful | Data table with sorting |
| `Dialog` | Stateful | Modal dialog |

## The Sizable Trait

```rust
// Available sizes (smallest to largest)
Button::new("btn").xsmall()  // Extra small - toolbars
Button::new("btn").small()   // Small - dense layouts
Button::new("btn").medium()  // Medium - default
Button::new("btn").large()   // Large - prominent actions

// Works on multiple component types
Icon::new(IconName::Star).small()
Input::new(&input_state).large()
Avatar::new("user").xsmall()
```

| Size | Use Case |
|------|----------|
| `xsmall()` | Toolbars, dense data tables |
| `small()` | Secondary actions, compact forms |
| `medium()` | Primary actions (default) |
| `large()` | Hero sections, prominent CTAs |

## Button Variants

```rust
Button::new("save").primary().label("Save Changes")    // Main actions
Button::new("delete").danger().label("Delete")         // Destructive
Button::new("reset").warning().label("Reset")          // Caution
Button::new("confirm").success().label("Confirm")      // Positive
Button::new("cancel").ghost().label("Cancel")          // Minimal
Button::new("edit").outline().label("Edit")            // Bordered
Button::new("more").link().label("Learn more")         // Text link
Button::new("dismiss").text().label("Dismiss")         // Plain text
```

| Variant | Visual Style | Use For |
|---------|--------------|---------|
| `primary()` | Filled, accent | Main action per section |
| `danger()` | Red | Delete, remove, destroy |
| `warning()` | Yellow/amber | Actions needing caution |
| `success()` | Green | Confirm, complete, approve |
| `ghost()` | No background | Secondary actions |
| `outline()` | Border only | Alternative to ghost |

## Event Handlers

```rust
Button::new("action-btn")
    .label("Perform Action")
    .on_click(|event, window, cx| {
        println!("Button clicked!");
    })
```

### Updating State on Click

```rust
impl MyView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity().clone();
        
        Button::new("increment")
            .label("Add")
            .on_click(move |_, _, cx| {
                cx.update_entity(&entity, |view, cx| {
                    view.counter += 1;
                    cx.notify();
                });
            })
    }
}
```

## child() vs label()

### label() - Simple Text

```rust
Button::new("btn").label("Click Me")
Button::new("btn").label(format!("Items: {}", count))
```

### child() - Complex Content

```rust
Button::new("btn").child(
    h_flex()
        .gap_2()
        .child(Icon::new(IconName::Download))
        .child("Download")
)
```

## Input Component Pattern

```rust
struct FormView {
    input: Entity<InputState>,
}

impl FormView {
    fn new(window: &Window, cx: &mut Context<Self>) -> Self {
        let input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Enter value...")
                .default_value("Initial")
        });
        Self { input }
    }
    
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .gap_2()
            .child(Label::new("Username"))
            .child(
                Input::new(&self.input)
                    .small()
                    .on_change(|value, cx| {
                        println!("Input changed: {}", value);
                    })
            )
    }
}

// Read value: self.input.read(cx).value()
```

## Select Component Pattern

```rust
struct SettingsView {
    theme_select: Entity<SelectState>,
}

impl SettingsView {
    fn new(window: &Window, cx: &mut Context<Self>) -> Self {
        let theme_select = cx.new(|cx| {
            SelectState::new(window, cx).placeholder("Choose theme")
        });
        Self { theme_select }
    }
    
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        Select::new(&self.theme_select)
            .items(vec![
                SelectItem::new("light", "Light"),
                SelectItem::new("dark", "Dark"),
                SelectItem::new("system", "System"),
            ])
            .on_change(|selected, cx| {
                println!("Selected: {:?}", selected);
            })
    }
}
```

## Best Practices

1. **Unique IDs**: Use descriptive, unique button IDs (`"user-profile-save"` not `"btn1"`)
2. **Semantic Variants**: Match variant to action meaning (danger for delete)
3. **Consistent Sizing**: Use same size throughout a UI section
4. **Store Entities**: Create stateful components in `new()`, store in struct fields
5. **Prefer label()**: Use `child()` only when you need custom content
