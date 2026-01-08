# Rendering and Elements in GPUI

## Core Rendering Traits

### Render Trait (Stateful Views)

```rust
use gpui::*;

struct CounterView { count: i32 }

impl Render for CounterView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .gap_2()
            .child(format!("Count: {}", self.count))
    }
}
```

### RenderOnce Trait (Stateless Components)

```rust
struct Badge { label: SharedString, color: Hsla }

impl RenderOnce for Badge {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        div().px_2().py_1().rounded_md().bg(self.color).child(self.label)
    }
}
```

## The div() Element (Tailwind-Style API)

### Flexbox Layout

```rust
// Horizontal flex container
div().flex().gap_3().child("Item 1").child("Item 2")

// Vertical flex container
div().flex().flex_col().gap_2().child("Top").child("Bottom")

// Centering content
div()
    .flex()
    .justify_center()  // horizontal centering
    .items_center()    // vertical centering
    .size_full()       // fill available space
    .child("Centered!")
```

### Sizing

```rust
div()
    .size(px(200.0))      // width and height: 200px
    .size_full()          // width: 100%, height: 100%
    .w(px(300.0))         // width: 300px
    .h(px(100.0))         // height: 100px
    .w_full()             // width: 100%
    .min_w(px(100.0))     // min-width: 100px
    .max_w(px(500.0))     // max-width: 500px
```

### Colors and Backgrounds

```rust
div()
    .bg(rgb(0x1e1e2e))              // background color (hex)
    .bg(hsla(0.6, 0.5, 0.5, 1.0))   // background color (hsla)
    .text_color(rgb(0xffffff))      // text color
```

### Spacing (Padding and Margin)

```rust
div()
    // Padding
    .p_4()      // padding: 1rem (16px) all sides
    .px_2()     // padding-left and padding-right
    .py_3()     // padding-top and padding-bottom
    .pt_1()     // padding-top only
    
    // Margin
    .m_4()      // margin: 1rem all sides
    .mx_auto()  // margin-left and margin-right: auto
```

### Borders and Shadows

```rust
div()
    .border_1()                    // border-width: 1px
    .border_color(rgb(0x3b82f6))   // border color
    .rounded_md()                  // border-radius: medium
    .rounded_lg()                  // border-radius: large
    .rounded_full()                // border-radius: 9999px (circle)
    .shadow_md()                   // medium shadow
```

### Text Styling

```rust
div()
    .text_sm()                     // font-size: small
    .text_lg()                     // font-size: large
    .text_xl()                     // font-size: extra large
    .text_color(rgb(0xffffff))     // text color
    .font_weight(FontWeight::BOLD) // font weight
```

## Adding Children

### Single Child

```rust
div()
    .child("Text content")
    .child(another_div())
    .child(my_component())
```

### Multiple Children

```rust
let items = vec!["Apple", "Banana", "Cherry"];

div().flex().flex_col().gap_2().children(
    items.iter().map(|item| {
        div().px_3().py_2().bg(rgb(0x2d2d2d)).rounded_md().child(*item)
    })
)
```

## Conditional Rendering

### when() Method

```rust
let is_selected = true;

div()
    .px_4().py_2().rounded_md()
    .bg(rgb(0x2d2d2d))
    .when(is_selected, |this| this.bg(rgb(0x3b82f6)))
    .child("Button")
```

### when_some() for Options

```rust
let user_avatar: Option<SharedString> = Some("avatar.png".into());

div().when_some(user_avatar, |this, avatar_url| {
    this.child(img(avatar_url))
})
```

## Layout Helpers (gpui-component)

```rust
use gpui_component::*;

// Horizontal flex container
h_flex().gap_2().child("Left").child("Right")

// Vertical flex container  
v_flex().gap_2().child("Top").child("Bottom")
```

## Reusable Component Functions

```rust
fn card(title: &str, content: impl IntoElement) -> Div {
    div()
        .p_4()
        .bg(rgb(0x1e1e2e))
        .rounded_lg()
        .shadow_md()
        .border_1()
        .border_color(rgb(0x3d3d3d))
        .child(div().text_lg().font_weight(FontWeight::BOLD).mb_2().child(title))
        .child(content)
}

// Usage
impl Render for MyView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        card("Welcome", "This is the card content")
    }
}
```

## Complex Layout Composition

```rust
fn header() -> Div {
    div()
        .flex()
        .justify_between()
        .items_center()
        .px_4().py_2()
        .bg(rgb(0x1a1a2e))
        .child("Logo")
        .child(div().flex().gap_4().child("Home").child("About"))
}

fn sidebar() -> Div {
    div()
        .flex().flex_col()
        .w(px(250.0)).h_full()
        .bg(rgb(0x16162a))
        .p_4().gap_2()
        .child("Dashboard")
        .child("Settings")
}

impl Render for AppView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex().flex_col()
            .child(header())
            .child(
                div().flex().flex_1()
                    .child(sidebar())
                    .child(div().flex_1().p_4().child("Main content"))
            )
    }
}
```
