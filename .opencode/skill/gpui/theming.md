# Theming and Styling in GPUI

## The Root View Requirement

**Critical**: Your app must wrap content in `Root` for theming to work.

```rust
use gpui::{App, Application, WindowOptions};
use gpui_component::Root;

fn main() {
    Application::new().run(|cx: &mut App| {
        gpui_component::init(cx);  // Required
        
        cx.open_window(WindowOptions::default(), |window, cx| {
            let view = cx.new(|_| MyView::new());
            cx.new(|cx| Root::new(view, window, cx))  // MUST wrap in Root
        });
    });
}
```

Without `Root`: No theming, components may panic.

## Accessing Theme Colors

```rust
use gpui::{div, prelude::*, Context, IntoElement, Render, Window};
use gpui_component::ActiveTheme;

struct ThemedView;

impl Render for ThemedView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .size_full()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .border_1()
            .border_color(cx.theme().border)
            .child("Themed content")
    }
}
```

## Theme Color Tokens

### Core Colors

| Token | Purpose |
|-------|---------|
| `cx.theme().primary` | Primary brand/action color |
| `cx.theme().background` | Main background |
| `cx.theme().foreground` | Primary text |
| `cx.theme().border` | Borders and dividers |
| `cx.theme().muted_foreground` | Secondary/muted text |

### Status Colors

| Token | Purpose |
|-------|---------|
| `cx.theme().success` | Success states |
| `cx.theme().warning` | Warning states |
| `cx.theme().destructive` | Errors, destructive actions |

### Named Colors

| Token | Purpose |
|-------|---------|
| `cx.theme().red` | Red accent |
| `cx.theme().green` | Green accent |
| `cx.theme().blue` | Blue accent |

### Chart Colors

`cx.theme().chart_1` through `cx.theme().chart_5`

## Color Primitives

### RGB Colors

```rust
use gpui::rgb;

div().bg(rgb(0x505050))       // Gray
div().bg(rgb(0x3b82f6))       // Blue
div().bg(rgb(0x1e1e1e))       // Dark gray
```

### HSLA Colors

```rust
use gpui::Hsla;

let custom_color = Hsla {
    h: 210.0 / 360.0,  // Blue hue (normalized to 0-1)
    s: 0.8,            // 80% saturation
    l: 0.5,            // 50% lightness
    a: 1.0,            // Fully opaque
};
```

### Built-in Colors

```rust
div().bg(gpui::red())
div().bg(gpui::green())
div().bg(gpui::blue())
div().bg(gpui::black())
div().bg(gpui::white())
```

### Opacity Modifier

```rust
div()
    .bg(cx.theme().primary.opacity(0.5))        // 50% transparent
    .border_color(cx.theme().border.opacity(0.3)) // 30% transparent
```

### Pixel Values

```rust
use gpui::px;

div()
    .w(px(200.0))      // 200 pixels wide
    .h(px(100.0))      // 100 pixels tall
    .p(px(16.0))       // 16 pixels padding
    .rounded(px(4.0))  // 4 pixel border radius
```

## Gradients

```rust
use gpui::{linear_gradient, linear_color_stop};

div()
    .size_full()
    .bg(linear_gradient(
        0.,  // Angle (0 = top to bottom)
        linear_color_stop(cx.theme().chart_1.opacity(0.4), 1.),  // Top
        linear_color_stop(cx.theme().background.opacity(0.3), 0.),  // Bottom
    ))
```

## Dark Mode

Dark mode works automatically. Just use theme tokens:

```rust
// Works in both light and dark mode
div()
    .bg(cx.theme().background)      // Light in light mode, dark in dark mode
    .text_color(cx.theme().foreground)  // Dark in light mode, light in dark mode
```

## Best Practices

### Do: Use Semantic Tokens

```rust
div()
    .bg(cx.theme().background)
    .text_color(cx.theme().foreground)
    .border_color(cx.theme().border)
```

### Don't: Hardcode Colors

```rust
// BAD - Won't adapt to dark mode
div()
    .bg(rgb(0xffffff))
    .text_color(rgb(0x000000))
```

### Do: Use Opacity for Variations

```rust
div()
    .bg(cx.theme().primary.opacity(0.1))  // Subtle primary tint
    .hover(|style| style.bg(cx.theme().primary.opacity(0.2)))
```

## Complete Example

```rust
impl Render for Dashboard {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex().flex_col()
            .size_full()
            .bg(cx.theme().background)
            .p_6().gap_4()
            .child(
                div()
                    .text_2xl()
                    .font_weight(gpui::FontWeight::BOLD)
                    .text_color(cx.theme().foreground)
                    .child("Dashboard")
            )
            .child(
                div()
                    .flex().gap_4()
                    .children(self.metrics.iter().map(|metric| {
                        let change_color = if metric.change >= 0.0 {
                            cx.theme().success
                        } else {
                            cx.theme().destructive
                        };

                        div()
                            .flex().flex_col()
                            .p_4()
                            .bg(cx.theme().background)
                            .border_1()
                            .border_color(cx.theme().border)
                            .rounded_lg()
                            .child(div().text_sm().text_color(cx.theme().muted_foreground).child(&metric.name))
                            .child(div().text_xl().text_color(cx.theme().foreground).child(format!("{:.0}", metric.value)))
                            .child(div().text_sm().text_color(change_color).child(format!("{:+.1}%", metric.change)))
                    }))
            )
    }
}
```
