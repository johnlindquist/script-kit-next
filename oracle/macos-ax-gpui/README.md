# macos-ax-gpui

A GPUI-friendly Rust wrapper around macOS Accessibility (`AXUIElement` and `AXObserver`) plus CoreGraphics window metadata for reading and controlling desktop apps.

The crate is split deliberately:

- **Core API**: no GPUI dependency. It returns handles (`AxElement`) for direct interaction, snapshots (`ElementSnapshot`) for UI rendering, observer events (`AxEvent`) for live updates, and `WindowInfo` and `DisplayInfo` records for screenshot/window-manager workflows.
- **Optional `gpui` feature**: provides a thin `AxEventSource` that implements GPUI's `EventEmitter<AxEvent>` marker and drains observer events without blocking the UI thread.

This is meant for apps like launchers, command palettes, inspectors, window managers, screen-region/window pickers, screenshot tools, assistive tools, and automation panels. It is not a full replacement for AppleScript, ScreenCaptureKit, CGWindow image capture, or app-specific automation APIs.

## Coverage by common product scenario

| Scenario | Covered APIs |
|---|---|
| Raycast-style launchers / command palettes | focused app/element/window, selected text, text ranges, element search, menu item lookup/press, button/action helpers |
| Window managers | visible window owner PIDs, active display bounds, AX window handles, move/resize/frame helpers, focus/raise/frontmost/minimize/zoom/full-screen button helpers, window observers |
| Screenshot and window pickers | element hit-testing, mouse location, `WindowInfo` with `CGWindowID`, bounds, active display matching, owner PID/name/title/layer/alpha, best-effort AX-window-to-CG-window matching |
| Inspectors / accessibility debuggers | attributes, parameterized attributes, action names, bounded snapshots, role/subrole constants, tree search |
| UI automation | settable values, typed getters, actions, menu traversal, text range helpers, common control/window convenience accessors |
| Live UI overlays | per-app observers, merged multi-app observer group, notification constants for app/window/menu/value/selection/layout events |

## Features

- Check and prompt for Accessibility permission.
- Read focused application, focused window, focused element, element under a screen point, and element under the mouse.
- Read attributes: role, subrole, title, value, description, enabled/focused/selected state, position, size, children, supported attributes, supported parameterized attributes, and supported actions.
- Build bounded recursive snapshots for rendering in GPUI or another UI toolkit.
- Search AX trees by custom predicate, role, title, or label text.
- Mutate settable attributes such as `AXValue`, `AXPosition`, `AXSize`, `AXFocused`, `AXFrontmost`, and `AXMinimized`.
- Perform actions such as `AXPress`, `AXRaise`, `AXIncrement`, `AXDecrement`, `AXShowMenu`, and `AXScrollToVisible`.
- Traverse and press menu items by path, for example `Window → Minimize`.
- Read selected text and text ranges; query parameterized text helpers such as `AXBoundsForRange`, `AXStringForRange`, and `AXRangeForPosition` when the target app supports them.
- List desktop windows through CoreGraphics with `CGWindowID`, PID, app name, title, bounds, layer, alpha, sharing state, and memory usage.
- List active displays and locate the display containing a point or window rectangle.
- Match an AX window to CoreGraphics window metadata so a screenshot tool can pass `WindowInfo::id` to a capture pipeline.
- Observe application notifications with `AXObserver` on a dedicated run-loop thread.
- Merge observers across many currently visible window-owning apps.

## Install

```toml
[dependencies]
macos-ax-gpui = { path = "../macos-ax-gpui" }

# Optional GPUI adapter:
# macos-ax-gpui = { path = "../macos-ax-gpui", features = ["gpui"] }
```

## macOS permission

The calling binary must be approved in:

`System Settings → Privacy & Security → Accessibility`

The default `AxClientOptions` will ask macOS to show the permission prompt. After granting permission, restart the binary; macOS usually does not apply the approval to a running process.

For development, approve the actual executable you run. Depending on your workflow that may be `target/debug/your-app`, your terminal, or your bundled `.app`.

Screenshot capture itself may also require Screen Recording permission if your app later uses ScreenCaptureKit or CoreGraphics image capture. This crate gives you window IDs, bounds, and AX context; it does not bypass screen-capture permissions.

## Basic usage

```rust
use macos_ax_gpui::{action, attr, AxClient, AxClientOptions, TreeOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if !AxClient::trusted(true)? {
        eprintln!("Approve Accessibility permission, then run again.");
        return Ok(());
    }

    let client = AxClient::new(AxClientOptions::default())?;
    let focused = client.focused_element()?;

    println!("role  = {:?}", focused.string_attribute(attr::ROLE)?);
    println!("title = {:?}", focused.string_attribute(attr::TITLE)?);
    println!("frame = {:?}", focused.frame()?);

    if focused.action_names()?.iter().any(|name| name == action::PRESS) {
        focused.press()?;
    }

    let tree = focused.snapshot(TreeOptions::default())?;
    println!("snapshot root: {tree:#?}");

    Ok(())
}
```

## Move, resize, and focus a window

```rust
use macos_ax_gpui::{AxClient, AxClientOptions, Point, Rect, Size};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = AxClient::new(AxClientOptions::default())?;
    let window = client.focused_window()?;

    window.set_position(Point::new(100.0, 100.0))?;
    window.set_size(Size::new(900.0, 700.0))?;
    window.set_frame(Rect::new(100.0, 100.0, 900.0, 700.0))?;
    window.bring_to_front()?;

    Ok(())
}
```

## Display-aware window management

```rust
use macos_ax_gpui::{AxClient, AxClientOptions, Rect};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = AxClient::new(AxClientOptions::default())?;
    let window = client.focused_window()?;

    if let Some(display) = client.display_containing_rect(window.frame()?.unwrap_or(Rect::new(0.0, 0.0, 1.0, 1.0)))? {
        let left_half = Rect::new(
            display.bounds.x(),
            display.bounds.y(),
            display.bounds.width() / 2.0,
            display.bounds.height(),
        );
        window.set_frame(left_half)?;
    }

    Ok(())
}
```

## Screenshot/window-picker metadata

```rust
use macos_ax_gpui::{AxClient, AxClientOptions, WindowQuery};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = AxClient::new(AxClientOptions::default())?;

    for window in client.window_list(WindowQuery::default().regular_windows())? {
        println!(
            "id={} pid={} app={:?} title={:?} bounds={:?}",
            window.id,
            window.owner_pid,
            window.owner_name,
            window.title,
            window.bounds,
        );
    }

    let ax_window = client.focused_window()?;
    if let Some(cg_window) = client.window_info_for_element(&ax_window)? {
        println!("Use CGWindowID {} in your capture pipeline", cg_window.id);
    }

    Ok(())
}
```

## Menu automation

```rust
use macos_ax_gpui::{AxClient, AxClientOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = AxClient::new(AxClientOptions::default())?;
    let app = client.focused_application()?;

    if !app.press_menu_item_by_path(&["Window", "Minimize"])? {
        eprintln!("Focused app did not expose that menu path");
    }

    Ok(())
}
```

## Text-selection helper

```rust
use macos_ax_gpui::{AxClient, AxClientOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = AxClient::new(AxClientOptions::default())?;
    let focused = client.focused_element()?;

    println!("selected text: {:?}", focused.selected_text()?);
    if let Some(range) = focused.selected_text_range()? {
        println!("selected bounds: {:?}", focused.bounds_for_range(range)?);
    }

    Ok(())
}
```

## Observe app/window changes

```rust
use macos_ax_gpui::{notification, AxClient, AxClientOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = AxClient::new(AxClientOptions::default())?;

    let (_observers, rx) = client.observe_visible_window_apps([
        notification::FOCUSED_WINDOW_CHANGED,
        notification::WINDOW_CREATED,
        notification::WINDOW_MOVED,
        notification::WINDOW_RESIZED,
        notification::WINDOW_MINIATURIZED,
        notification::WINDOW_DEMINIATURIZED,
        notification::TITLE_CHANGED,
    ])?;

    while let Ok(event) = rx.recv() {
        println!("{} pid={} {:?}", event.notification, event.pid, event.element);
    }

    Ok(())
}
```

Keep the returned observer handle alive. Dropping it stops the observer thread.

## GPUI integration pattern

Enable the `gpui` feature and keep an `AxEventSource` inside one of your GPUI entities.

```rust
use gpui::{Context, EventEmitter};
use macos_ax_gpui::{gpui_bridge::AxEventSource, AxEvent};

struct Inspector {
    ax_events: AxEventSource,
    latest: Option<AxEvent>,
}

impl EventEmitter<AxEvent> for Inspector {}

impl Inspector {
    fn poll_accessibility(&mut self, cx: &mut Context<Self>) {
        for event in self.ax_events.drain_pending() {
            self.latest = Some(event.clone());
            cx.emit(event);
            cx.notify();
        }
    }
}
```

This crate intentionally does not force a timer or background executor abstraction on your GPUI app. The clean contract is: AX observer events arrive over `std::sync::mpsc`, and the UI layer decides when to drain them.

## Examples

```bash
cargo run --example focused_tree
cargo run --example observe_focused_app
cargo run --example window_list
cargo run --example snap_focused_window
cargo run --example press_menu_item
cargo run --example text_selection
```

## Practical caveats

- Not every app exposes a complete or stable accessibility tree.
- Some apps expose values only after elements become visible or menus are opened.
- `AXUIElement` handles can become invalid when windows close or views are rebuilt.
- Recursive snapshots and searches are bounded by default. Large tables, outlines, browsers, and web views can be expensive.
- `CannotComplete` usually means the target app is busy, blocked, sandboxed in a hostile way, or the messaging timeout is too low.
- AX window geometry and CoreGraphics window bounds usually line up, but not every app exposes enough title/frame metadata to match an AX window to a `CGWindowID` perfectly. Treat `window_info_for_element` as best effort.
- AX window move/resize notifications are end-of-operation events, not high-frequency drag updates.
- Screen coordinates come from macOS global screen coordinates. Use `active_displays`, `display_containing_point`, and `display_containing_rect` as the starting point for multi-display normalization.

## Public API sketch

- `AxClient::trusted(prompt)`
- `AxClient::new(options)`
- `AxClient::focused_application()`
- `AxClient::focused_window()`
- `AxClient::focused_element()`
- `AxClient::element_at_position(point)`
- `AxClient::element_at_mouse()`
- `AxClient::window_list(query)`
- `AxClient::window_at_mouse(query)`
- `AxClient::window_info_for_element(element)`
- `AxClient::observe_application(pid, notifications)`
- `AxClient::observe_applications(pids, notifications)`
- `AxClient::observe_visible_window_apps(notifications)`
- `AxElement::attribute(name)`
- `AxElement::parameterized_attribute(name, parameter)`
- `AxElement::children()`
- `AxElement::snapshot(options)`
- `AxElement::find_first(...)` / `AxElement::find_all(...)`
- `AxElement::selected_text()` / `AxElement::bounds_for_range(range)`
- `AxElement::set_value(value)`
- `AxElement::set_position(point)`
- `AxElement::set_size(size)`
- `AxElement::set_frame(rect)`
- `AxElement::bring_to_front()`
- `AxElement::minimize()` / `AxElement::unminimize()`
- `AxElement::close_window()` / `AxElement::zoom_window()` / `AxElement::toggle_full_screen()`
- `AxElement::perform_action(action)`
- `AxElement::menu_item_by_path(path)` / `AxElement::press_menu_item_by_path(path)`
