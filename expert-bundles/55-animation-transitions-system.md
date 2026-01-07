# Animation & Transitions System - Expert Bundle

## Overview

Script Kit uses a lerp-based animation system with easing functions for smooth UI transitions.

## Core Traits and Types (src/transitions.rs)

### Lerp Trait

```rust
/// Linear interpolation trait for animatable values
pub trait Lerp {
    fn lerp(&self, to: &Self, delta: f32) -> Self;
}

// Implementations for primitives
impl Lerp for f32 {
    #[inline]
    fn lerp(&self, to: &Self, delta: f32) -> Self {
        self + (to - self) * delta
    }
}

impl Lerp for f64 {
    #[inline]
    fn lerp(&self, to: &Self, delta: f32) -> Self {
        self + (to - self) * (delta as f64)
    }
}
```

### Standard Durations

```rust
/// Fast transition (100ms) - hover effects, micro-interactions
pub const DURATION_FAST: Duration = Duration::from_millis(100);

/// Standard transition (150ms) - selection changes, hover feedback
pub const DURATION_STANDARD: Duration = Duration::from_millis(150);

/// Medium transition (200ms) - panel reveals, focus changes
pub const DURATION_MEDIUM: Duration = Duration::from_millis(200);

/// Slow transition (300ms) - large UI changes, modal appearances
pub const DURATION_SLOW: Duration = Duration::from_millis(300);
```

## Easing Functions

```rust
/// Linear easing - constant velocity
#[inline]
pub fn linear(t: f32) -> f32 {
    t
}

/// Quadratic ease out - fast start, slow end
/// Good for elements entering the screen
#[inline]
pub fn ease_out_quad(t: f32) -> f32 {
    1.0 - (1.0 - t) * (1.0 - t)
}

/// Quadratic ease in - slow start, fast end
/// Good for elements leaving the screen
#[inline]
pub fn ease_in_quad(t: f32) -> f32 {
    t * t
}

/// Quadratic ease in-out - slow start and end
/// Good for continuous looping animations
#[inline]
pub fn ease_in_out_quad(t: f32) -> f32 {
    if t < 0.5 {
        2.0 * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
    }
}

/// Cubic ease out - faster deceleration
#[inline]
pub fn ease_out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

/// Cubic ease in - slower acceleration
#[inline]
pub fn ease_in_cubic(t: f32) -> f32 {
    t * t * t
}
```

## Transition Color

```rust
/// Color value supporting linear interpolation
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TransitionColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl TransitionColor {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn from_hex_alpha(hex: u32, alpha: f32) -> Self {
        Self {
            r: ((hex >> 16) & 0xFF) as f32 / 255.0,
            g: ((hex >> 8) & 0xFF) as f32 / 255.0,
            b: (hex & 0xFF) as f32 / 255.0,
            a: alpha,
        }
    }

    pub fn transparent() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0)
    }

    pub fn to_rgba(self) -> Rgba {
        Rgba { r: self.r, g: self.g, b: self.b, a: self.a }
    }
}

impl Lerp for TransitionColor {
    fn lerp(&self, to: &Self, delta: f32) -> Self {
        Self {
            r: self.r + (to.r - self.r) * delta,
            g: self.g + (to.g - self.g) * delta,
            b: self.b + (to.b - self.b) * delta,
            a: self.a + (to.a - self.a) * delta,
        }
    }
}
```

## Opacity Transition

```rust
/// Opacity value for fade transitions (0.0-1.0)
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Opacity(pub f32);

impl Opacity {
    pub const INVISIBLE: Self = Self(0.0);
    pub const VISIBLE: Self = Self(1.0);
    pub const HALF: Self = Self(0.5);

    pub fn new(value: f32) -> Self {
        Self(value.clamp(0.0, 1.0))
    }
}

impl Lerp for Opacity {
    fn lerp(&self, to: &Self, delta: f32) -> Self {
        Self(self.0 + (to.0 - self.0) * delta)
    }
}
```

## Slide Offset

```rust
/// Offset for slide animations
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct SlideOffset {
    pub x: f32,
    pub y: f32,
}

impl SlideOffset {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    pub fn from_bottom(amount: f32) -> Self {
        Self { x: 0.0, y: amount }
    }

    pub fn from_top(amount: f32) -> Self {
        Self { x: 0.0, y: -amount }
    }

    pub fn from_left(amount: f32) -> Self {
        Self { x: -amount, y: 0.0 }
    }

    pub fn from_right(amount: f32) -> Self {
        Self { x: amount, y: 0.0 }
    }
}

impl Lerp for SlideOffset {
    fn lerp(&self, to: &Self, delta: f32) -> Self {
        Self {
            x: self.x + (to.x - self.x) * delta,
            y: self.y + (to.y - self.y) * delta,
        }
    }
}
```

## Combined Transitions

### Appear Transition (Toast/Notification)

```rust
/// Combined opacity and slide for appear animations
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AppearTransition {
    pub opacity: Opacity,
    pub offset: SlideOffset,
}

impl AppearTransition {
    /// Initial hidden state (invisible, offset down)
    pub fn hidden() -> Self {
        Self {
            opacity: Opacity::INVISIBLE,
            offset: SlideOffset::from_bottom(20.0),
        }
    }

    /// Visible state (fully visible, no offset)
    pub fn visible() -> Self {
        Self {
            opacity: Opacity::VISIBLE,
            offset: SlideOffset::ZERO,
        }
    }

    /// Dismiss state (invisible, offset up)
    pub fn dismissed() -> Self {
        Self {
            opacity: Opacity::INVISIBLE,
            offset: SlideOffset::from_top(10.0),
        }
    }
}

impl Lerp for AppearTransition {
    fn lerp(&self, to: &Self, delta: f32) -> Self {
        Self {
            opacity: self.opacity.lerp(&to.opacity, delta),
            offset: self.offset.lerp(&to.offset, delta),
        }
    }
}
```

### Hover State (List Items)

```rust
/// Hover state for list item backgrounds
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HoverState {
    pub background: TransitionColor,
}

impl HoverState {
    pub fn normal() -> Self {
        Self {
            background: TransitionColor::transparent(),
        }
    }

    pub fn with_background(color: TransitionColor) -> Self {
        Self { background: color }
    }
}

impl Lerp for HoverState {
    fn lerp(&self, to: &Self, delta: f32) -> Self {
        Self {
            background: self.background.lerp(&to.background, delta),
        }
    }
}
```

## Animation Driver Pattern

```rust
pub struct AnimationDriver {
    start_time: Instant,
    duration: Duration,
    easing: fn(f32) -> f32,
}

impl AnimationDriver {
    pub fn new(duration: Duration, easing: fn(f32) -> f32) -> Self {
        Self {
            start_time: Instant::now(),
            duration,
            easing,
        }
    }

    /// Get current progress (0.0 to 1.0) with easing applied
    pub fn progress(&self) -> f32 {
        let elapsed = self.start_time.elapsed();
        let t = (elapsed.as_secs_f32() / self.duration.as_secs_f32()).min(1.0);
        (self.easing)(t)
    }

    /// Check if animation is complete
    pub fn is_complete(&self) -> bool {
        self.start_time.elapsed() >= self.duration
    }
}
```

## Usage in Components

### Toast Animation

```rust
struct Toast {
    message: String,
    state: AppearTransition,
    animation: Option<AnimationDriver>,
}

impl Toast {
    fn show(&mut self, cx: &mut Context<Self>) {
        self.state = AppearTransition::hidden();
        self.animation = Some(AnimationDriver::new(
            DURATION_MEDIUM,
            ease_out_quad,
        ));
        
        // Schedule animation updates
        cx.spawn(|this, mut cx| async move {
            while let Ok(()) = this.update(&mut cx, |toast, cx| {
                if let Some(ref anim) = toast.animation {
                    if anim.is_complete() {
                        toast.state = AppearTransition::visible();
                        toast.animation = None;
                    } else {
                        let progress = anim.progress();
                        toast.state = AppearTransition::hidden()
                            .lerp(&AppearTransition::visible(), progress);
                        cx.notify();
                    }
                }
                !anim.is_none() // Continue if animation running
            }) {
                Timer::after(Duration::from_millis(16)).await; // ~60fps
            }
        }).detach();
    }

    fn render(&self) -> impl IntoElement {
        div()
            .opacity(self.state.opacity.0)
            .top(px(self.state.offset.y))
            .child(&self.message)
    }
}
```

### List Item Hover

```rust
struct ListItem {
    text: String,
    hover_state: HoverState,
    is_hovered: bool,
}

impl ListItem {
    fn render(&mut self, theme: &Theme, cx: &mut Context<Self>) -> impl IntoElement {
        let target = if self.is_hovered {
            HoverState::with_background(
                TransitionColor::from_hex_alpha(theme.colors.ui.hover, 0.1)
            )
        } else {
            HoverState::normal()
        };

        // Smooth transition
        self.hover_state = self.hover_state.lerp(&target, 0.2);

        div()
            .bg(self.hover_state.background.to_rgba())
            .on_mouse_enter(cx.listener(|this, _, _, cx| {
                this.is_hovered = true;
                cx.notify();
            }))
            .on_mouse_leave(cx.listener(|this, _, _, cx| {
                this.is_hovered = false;
                cx.notify();
            }))
            .child(&self.text)
    }
}
```

## Selection Transition

```rust
struct SelectableList {
    items: Vec<String>,
    selected_index: usize,
    selection_y: f32,  // Animated vertical position
}

impl SelectableList {
    fn update_selection(&mut self, new_index: usize, cx: &mut Context<Self>) {
        let old_y = self.selection_y;
        let new_y = (new_index as f32) * ITEM_HEIGHT;
        
        self.selected_index = new_index;
        
        // Animate selection indicator
        cx.spawn(|this, mut cx| async move {
            let driver = AnimationDriver::new(DURATION_FAST, ease_out_quad);
            
            while !driver.is_complete() {
                let _ = this.update(&mut cx, |list, cx| {
                    list.selection_y = old_y.lerp(&new_y, driver.progress());
                    cx.notify();
                });
                Timer::after(Duration::from_millis(16)).await;
            }
            
            let _ = this.update(&mut cx, |list, cx| {
                list.selection_y = new_y;
                cx.notify();
            });
        }).detach();
    }

    fn render(&self) -> impl IntoElement {
        div()
            .relative()
            // Selection indicator
            .child(
                div()
                    .absolute()
                    .top(px(self.selection_y))
                    .w_full()
                    .h(px(ITEM_HEIGHT))
                    .bg(selection_color)
            )
            // Items
            .children(self.items.iter().map(|item| {
                div().child(item.clone())
            }))
    }
}
```

## Performance Tips

### 1. Use 60fps Target

```rust
const FRAME_DURATION: Duration = Duration::from_millis(16); // ~60fps
```

### 2. Skip Animation for Reduced Motion

```rust
fn should_animate() -> bool {
    // Check system preference (macOS)
    #[cfg(target_os = "macos")]
    {
        // Check NSWorkspace.accessibilityDisplayShouldReduceMotion
        false // Placeholder
    }
    #[cfg(not(target_os = "macos"))]
    true
}

fn transition_or_instant<T: Lerp + Copy>(from: T, to: T, progress: f32) -> T {
    if should_animate() {
        from.lerp(&to, progress)
    } else {
        to // Instant
    }
}
```

### 3. Batch Updates

```rust
// Bad - triggers multiple renders
for item in &mut self.items {
    item.update_hover(cx);
    cx.notify(); // Many notifies!
}

// Good - single render
for item in &mut self.items {
    item.update_hover_internal();
}
cx.notify(); // Once
```

## Summary

1. **Lerp trait** for linear interpolation of any animatable value
2. **Easing functions** for natural motion curves
3. **TransitionColor** for smooth color changes
4. **Opacity** and **SlideOffset** for fade/slide effects
5. **AppearTransition** combines opacity + slide for toasts
6. **AnimationDriver** manages timing and progress
7. Target **60fps** (16ms frame budget)
8. Consider **reduced motion** preferences
