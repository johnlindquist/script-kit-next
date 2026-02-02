# Raycast Animation and Motion Design Research

> Research compiled: January 2026
> Purpose: Guide Script Kit GPUI's animation implementation based on Raycast's proven patterns

## Table of Contents

1. [Animation Philosophy](#animation-philosophy)
2. [Feedback Animations](#feedback-animations)
3. [View Transitions](#view-transitions)
4. [Micro-interactions](#micro-interactions)
5. [Performance Considerations](#performance-considerations)
6. [GPUI Animation Implementation](#gpui-animation-implementation)
7. [Implementation Recommendations](#implementation-recommendations)

---

## Animation Philosophy

### Core Design Principles

Raycast operates on three fundamental design principles that directly influence animation decisions:

1. **Fast** - Animations must enhance, never delay. Every animation should feel instant and purposeful.
2. **Simple** - Subtle animations that users barely notice consciously but feel subconsciously.
3. **Delightful** - Small animation details that make the experience enjoyable without being distracting.

### Performance Target

- **120 FPS** - Raycast targets buttery-smooth animations at 120fps on ProMotion displays
- **99.8% crash-free rate** - Native performance with pure macOS rendering
- **Native with pure performance** - Built directly on macOS APIs, not web views

### Animation Curve Guidelines

Based on Raycast's Animation Expert AI preset and best practices:

| Curve Type | When to Use | Rationale |
|------------|-------------|-----------|
| **ease-out** | Most animations | Makes interface feel fast and natural - quick start, smooth finish |
| **ease-in-out** | Already-visible content transitions | Starts and ends slowly, speeds up in middle - feels controlled |
| **linear** | NEVER use | Exception: infinite loops like marquees requiring constant speed |
| **spring physics** | Interactive elements | More natural feel, interruptible, no fixed duration |

> "Use 'ease-out' animation curve for most cases as it makes the interface feel fast and natural."

---

## Feedback Animations

### Toast Notifications

Raycast provides three distinct Toast styles with associated animations:

#### Toast.Style.Animated
- **Use case**: Ongoing/async operations
- **Visual**: Loading spinner animation
- **Behavior**: Persists until manually hidden or updated
- **Animation**: Continuous spinning indicator

```
Example: "Fetching data..." with spinning indicator
         Updates to "Fetched 42 items" with checkmark
```

#### Toast.Style.Success
- **Use case**: Confirmations and positive outcomes
- **Visual**: Checkmark icon with subtle appear animation
- **Behavior**: Auto-dismisses after brief display

#### Toast.Style.Failure
- **Use case**: Errors and negative outcomes
- **Visual**: Error icon with attention-grabbing but not jarring animation
- **Behavior**: May persist longer to ensure user notices

### HUD Notifications

When Raycast closes during an action (e.g., clipboard copy), a HUD confirms success:

- **Appearance**: Compact message at bottom of screen
- **Animation**: Fade-in on appear, **fade-out on dismiss** (added in v0.40)
- **Duration**: Brief, non-blocking
- **Transparency**: Carefully calibrated for readability (previous bug fixed "too transparent" issue)

### Loading Indicators

- **`isLoading` prop** on top-level components (List, Detail, Grid, Form)
- **Visual**: Subtle loading spinner in search bar area
- **Behavior**: Non-blocking - user can still interact with UI
- **Pagination placeholders**: Animated placeholders shown while loading next page

---

## View Transitions

### Navigation Stack Animations

Raycast's `useNavigation` hook manages view transitions:

#### Push Animation
- **Trigger**: `Action.Push` or `navigation.push()`
- **Animation**: New view slides/fades in from right
- **Behavior**: Previous view remains on stack

#### Pop Animation
- **Trigger**: `Escape` key or `navigation.pop()`
- **Animation**: Current view slides/fades out to right
- **Behavior**: Previous view revealed underneath

### Window Appearance

Based on the Animated Window Manager extension patterns:

- **macOS-native feeling transitions** - no jarring jumps
- **Smooth, polished movements** using native macOS window APIs
- **Spring-like physics** for natural deceleration

### Action Panel Transitions

- **Trigger**: `Cmd+K` to open
- **Animation**: Panel slides/fades in from bottom or side
- **Interaction**: Keyboard navigation immediately available
- **Dismiss**: `Escape` with reverse animation

---

## Micro-interactions

### Selection Highlighting

When navigating list items with arrow keys:

- **Instant feedback** - selection highlight moves immediately
- **No animation delay** on selection changes
- **Visual focus** clearly indicates current item

### Hover States

For mouse interactions:

- **Subtle highlight** on hover
- **Quick transition** - nearly instant, ~50ms
- **Consistent behavior** across all interactive elements

### Search Bar

- **Instant filtering** - results update as you type
- **Throttling available** for async operations
- **Visual feedback** during search processing

### Easter Egg Animations

Raycast includes delightful micro-interactions:

#### Confetti
- **Trigger**: `open raycast://confetti` or search "confetti"
- **Animation**: Colorful confetti particles explode from corners
- **Sound**: Optional celebratory sound effect
- **Use case**: Celebrating completed tasks, goals, milestones

#### DVD Bounce
- **Trigger**: Hidden command
- **Animation**: Raycast window bounces around screen like classic DVD logo
- **Purpose**: Pure delight, nostalgia

### Loading Pagination

- **Animated placeholders** appear at list end when loading more items
- **Smooth replacement** when actual items load
- **Memory-aware** - stops pagination if memory pressure detected

---

## Performance Considerations

### GPU Acceleration

For smooth 60+ FPS animations:

1. **Prefer transforms and opacity** - GPU-accelerated properties
2. **Avoid layout changes** - Don't animate width, height, margin, padding
3. **Minimize repaints** - Keep animated layers separate

### Common Performance Killers

| Issue | Impact | Solution |
|-------|--------|----------|
| Semi-transparent layers | GPU fills same pixel multiple times | Minimize overlapping transparency |
| Offscreen drawing | Shadow, masks, rounded corners require offscreen buffers | Use sparingly |
| Layout thrashing | Animating layout properties triggers reflow | Stick to transform/opacity |
| Too many animated elements | Overloads GPU compositor | Limit concurrent animations |

### Accessibility: Reduced Motion

**Critical**: Always respect `prefers-reduced-motion`:

```
if (user prefers reduced motion) {
    - Disable non-essential animations
    - Keep functional indicators (loading spinners)
    - Use instant transitions instead of animated
    - Maintain all accessibility cues
}
```

### Memory Management

- Monitor extension memory usage during animations
- Implement heuristics to stop pagination if memory exhaustion risk
- Clean up animation resources when views unload

---

## GPUI Animation Implementation

### Core Animation API

GPUI provides animations through the `Animation` struct and `AnimationExt` trait:

```rust
use gpui::{Animation, AnimationExt};
use std::time::Duration;

// Basic animation with duration
Animation::new(Duration::from_secs(2))

// Repeating animation
Animation::new(Duration::from_millis(500))
    .repeat()

// Animation with easing
Animation::new(Duration::from_millis(200))
    .with_easing(ease_out_quint)
```

### Applying Animations to Elements

```rust
// Pulsating opacity effect
Label::new("Loading...")
    .with_animation(
        "loading-label",
        Animation::new(Duration::from_secs(2))
            .repeat()
            .with_easing(pulsating_between(0.3, 0.7)),
        |label, delta| label.alpha(delta),
    )
```

### Built-in Easing Functions

GPUI provides these easing functions in `gpui/src/elements/animation.rs`:

| Function | Description | Use Case |
|----------|-------------|----------|
| `linear(delta)` | No easing, direct mapping | Marquees, progress bars |
| `quadratic(delta)` | `delta * delta` | Subtle acceleration |
| `ease_in_out(delta)` | Slow start/end, fast middle | State transitions |
| `ease_out_quint()` | Quick start, slow finish | "Animate in" transitions |
| `bounce(easing)` | Forward then reverse | Attention-grabbing effects |
| `pulsating_between(min, max)` | Natural breathing rhythm | Loading indicators |

### DefaultAnimations Trait

GPUI's UI crate provides common transition patterns:

```rust
use ui::DefaultAnimations;

div()
    .id("panel")
    .animate_in_from_bottom(true)  // Slide up + optional fade

div()
    .id("sidebar")
    .animate_in_from_left(false)   // Slide from left, no fade
```

Available directions:
- `animate_in_from_bottom(fade: bool)`
- `animate_in_from_left(fade: bool)`
- `animate_in_from_right(fade: bool)`
- `animate_in_from_top(fade: bool)`

### Animation Frame Requests

For continuous animations:

```rust
// Request next animation frame
window.request_animation_frame();
```

This ensures smooth updates for elements that need to animate continuously.

### Transform Animations

```rust
use gpui::Transformation;

svg
    .with_animation(
        "rotating-icon",
        Animation::new(Duration::from_secs(2))
            .repeat()
            .with_easing(bounce(ease_in_out)),
        |svg, delta| {
            svg.with_transformation(Transformation::rotate(percentage(delta)))
        },
    )
```

---

## Implementation Recommendations

### Priority 1: Essential Feedback Animations

#### Selection Transitions
```rust
// Smooth selection highlight movement
div()
    .id(format!("item-{}", index))
    .when(is_selected, |d| {
        d.bg(theme.colors.selection)
         .animate_in_from_left(false)
    })
```

#### Loading States
```rust
// Pulsating loading indicator
fn render_loading(&self, cx: &mut ViewContext<Self>) -> impl IntoElement {
    div()
        .with_animation(
            "loading-pulse",
            Animation::new(Duration::from_secs(1))
                .repeat()
                .with_easing(pulsating_between(0.4, 1.0)),
            |div, delta| div.opacity(delta),
        )
        .child(Icon::new(IconName::Loader))
}
```

### Priority 2: View Transitions

#### Push/Pop Navigation
```rust
fn animate_push(&self, view: impl IntoElement) -> impl IntoElement {
    div()
        .id("pushed-view")
        .animate_in_from_right(true)
        .child(view)
}

fn animate_pop(&self, view: impl IntoElement) -> impl IntoElement {
    div()
        .id("popping-view")
        .animate_in_from_left(true)
        .child(view)
}
```

#### Toast Notifications
```rust
fn render_toast(&self, toast: &Toast, cx: &mut ViewContext<Self>) -> impl IntoElement {
    let style = match toast.style {
        ToastStyle::Animated => (Icon::Spinner, true),
        ToastStyle::Success => (Icon::Check, false),
        ToastStyle::Failure => (Icon::XCircle, false),
    };

    div()
        .id("toast")
        .animate_in_from_bottom(true)
        .child(style.0)
        .when(style.1, |d| {
            d.with_animation(
                "spinner",
                Animation::new(Duration::from_secs(1))
                    .repeat()
                    .with_easing(linear),
                |div, delta| {
                    div.with_transformation(
                        Transformation::rotate(percentage(delta))
                    )
                },
            )
        })
        .child(Label::new(&toast.title))
}
```

### Priority 3: Micro-interactions

#### Hover Effects
```rust
div()
    .id("list-item")
    .hover(|style| {
        style
            .bg(theme.colors.hover)
            .transition_background_color(Duration::from_millis(50))
    })
```

#### Action Panel Appearance
```rust
fn render_action_panel(&self, cx: &mut ViewContext<Self>) -> impl IntoElement {
    div()
        .id("action-panel")
        .absolute()
        .bottom_0()
        .w_full()
        .animate_in_from_bottom(true)
        .children(self.actions.iter().map(|a| self.render_action(a, cx)))
}
```

### Priority 4: Accessibility

#### Reduced Motion Support
```rust
fn should_animate(cx: &WindowContext) -> bool {
    !cx.prefers_reduced_motion()
}

fn render_with_animation(&self, cx: &mut ViewContext<Self>) -> impl IntoElement {
    let base = div().id("animated-element");

    if should_animate(cx) {
        base.animate_in_from_bottom(true)
    } else {
        base // No animation, instant appearance
    }
}
```

### Animation Timing Guidelines

| Animation Type | Duration | Easing |
|----------------|----------|--------|
| Selection change | 0ms (instant) | none |
| Hover feedback | 50ms | ease-out |
| Toast appear | 150ms | ease-out |
| View push/pop | 200ms | ease-out-quint |
| Loading pulse | 1000ms (repeat) | pulsating_between |
| Confetti celebration | 2000ms | custom physics |

### Performance Checklist

1. **Use transform/opacity only** - Never animate layout properties
2. **Limit concurrent animations** - Max 3-5 simultaneously
3. **Clean up on unmount** - Stop animations when views close
4. **Request animation frames** - Use `request_animation_frame()` for continuous updates
5. **Test on older hardware** - Ensure smooth performance on base models
6. **Monitor memory** - Watch for leaks in long-running animations
7. **Respect reduced motion** - Always check `prefers_reduced_motion()`

---

## Sources

### Raycast Official
- [Raycast Official Website](https://www.raycast.com/)
- [Raycast API - Toast Documentation](https://developers.raycast.com/api-reference/feedback/toast)
- [Raycast API - HUD Documentation](https://developers.raycast.com/api-reference/feedback/hud)
- [Raycast API - Navigation](https://developers.raycast.com/api-reference/user-interface/navigation)
- [Raycast Blog - A Fresh Look and Feel](https://www.raycast.com/blog/a-fresh-look-and-feel)
- [Raycast Animation Expert AI Preset](https://ray.so/presets/preset/animations-expert)
- [Raycast Changelog](https://developers.raycast.com/misc/changelog)

### Raycast Extensions
- [Animated Window Manager](https://www.raycast.com/matheuschein/animated-window-manager)
- [Easings Extension](https://www.raycast.com/madebyankur/easings)
- [1-Click Confetti](https://www.raycast.com/peduarte/1-click-confetti)
- [Motion Preview](https://www.raycast.com/ayarse/raycast-motion-preview)

### GPUI Framework
- [GPUI Official Site](https://www.gpui.rs/)
- [GPUI README - Zed Repository](https://github.com/zed-industries/zed/blob/main/crates/gpui/README.md)
- [GPUI Animation Example](https://github.com/zed-industries/zed/blob/main/crates/gpui/examples/animation.rs)
- [GPUI Technical Overview (Medium)](https://beckmoulton.medium.com/gpui-a-technical-overview-of-the-high-performance-rust-ui-framework-powering-zed-ac65975cda9f)
- [DeepWiki - GPUI Framework](https://deepwiki.com/zed-industries/zed/2.2-ui-framework-(gpui))

### Animation Theory
- [Design Spells - Raycast DVD Animation](https://www.designspells.com/spells/raycast-has-a-command-that-makes-it-bounce-around-like-the-dvd-logo)
- [Josh W. Comeau - Spring Physics Introduction](https://www.joshwcomeau.com/animation/a-friendly-introduction-to-spring-physics/)
- [Josh W. Comeau - CSS Springs](https://www.joshwcomeau.com/animation/linear-timing-function/)
- [iOS Animation Efficiency (Toptal)](https://www.toptal.com/ios/ios-animation-and-tuning-for-efficiency)

### Community
- [Raycast Dribbble Profile](https://dribbble.com/raycastapp)
- [Raycast for Designers (UX Collective)](https://uxdesign.cc/raycast-for-designers-649fdad43bf1)
- [Raycast Must-Have Productivity App](https://www.stefanimhoff.de/raycast/)
