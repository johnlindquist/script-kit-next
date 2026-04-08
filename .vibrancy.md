# macOS Vibrancy & Blur for GPUI Overlay Components

## Purpose

This guide explains the current macOS blur/vibrancy stack used by Script Kit GPUI overlays.
It covers:

- GPUI `WindowBackgroundAppearance::Blurred`
- `NSVisualEffectView` configuration
- `CAChameleonLayer` preservation via `swizzle_gpui_blurred_view()`
- `BehindWindow` vs `WithinWindow` blending
- event passthrough, hover, and scroll on overlays
- Raycast-parity rules for launcher chrome

## Overlay Taxonomy

| Surface | Primary files | Blur source | Blending mode | Input/focus rule | Current use |
| --- | --- | --- | --- | --- | --- |
| Main launcher window | `src/main_sections/window_visibility.rs`, `src/platform/vibrancy_config.rs`, `vendor/gpui_macos/src/window.rs` | GPUI blurred background + swizzled `BlurredView` + theme tint | `BehindWindow` (`0`) on recursive `NSVisualEffectView` config | Standard launcher key panel | Primary launcher surface |
| Actions popup | `src/actions/window.rs`, `src/platform/secondary_window_config.rs` | Separate popup window with shared vibrancy config | `BehindWindow` (`0`) | Parent keeps focus; popup uses `setBecomesKeyOnlyIfNeeded:true` | Detached actions menu |
| Confirm popup | `src/confirm/window.rs`, `src/platform/secondary_window_config.rs` | Same popup vibrancy path as actions | `BehindWindow` (`0`) | Flush child dialog; no shadow | Native confirm overlay |
| Native footer host | `src/footer_popup.rs` | In-window `NSVisualEffectView` host inside main content view | `WithinWindow` (`1`) | Swallow footer background clicks, forward scroll, route real button hits | Current launcher footer path |
| Secondary popups sharing actions config | `src/platform/secondary_window_config.rs` | Same popup vibrancy common path | Usually `BehindWindow` (`0`) | Child-popup rules | Reusable overlay family |

## Specialized Overlay Recipes

These surfaces reuse the blur stack but intentionally change the presentation contract.

| Surface | Primary files | Base helper | Required divergence | Why |
| --- | --- | --- | --- | --- |
| ACP inline dropdown | `src/ai/acp/popup_window.rs`, `src/platform/secondary_window_config.rs` | `configure_inline_dropdown_popup_window()` | Keep `WindowKind::PopUp`, attach as a native child window, disable shadow | Inline pickers should read as panel chrome, not as detached popovers |
| Dictation overlay | `src/dictation/window.rs`, `src/platform/secondary_window_config.rs` | `configure_secondary_window_vibrancy()` | Create hidden with `show: false`, surface via `orderFrontRegardless`, optionally call `makeKeyWindow`, and avoid surfacing the main launcher first | The mic pill must appear alone and feel instant |

### Specialized Overlay Rules

- ACP inline dropdowns are still popup-family windows, but they intentionally drop the detached shadow so they feel attached to the chat surface.
- Dictation overlays are popup-family windows that prioritize zero-flash presentation over normal launcher visibility semantics.
- If you document one of these surfaces as "just another popup," the implementation will likely keep the blur but lose the feel.

## GPUI Blur Stack

GPUI blur is necessary but not sufficient.

1. `WindowBackgroundAppearance::Blurred` enables the backend blur path.
2. On macOS, GPUI installs a blur view for blurred windows.
3. GPUI's `BlurredView` hides `CAChameleonLayer` by default.
4. Script Kit swizzles `BlurredView.updateLayer` to preserve native tint.
5. Theme tint alpha still matters for readable contrast.

### GPUI background appearance enum

```rs
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub enum WindowBackgroundAppearance {
    #[default]
    Opaque,
    Transparent,
    Blurred,
    MicaBackdrop,
    MicaAltBackdrop,
}
```

### GPUI backend blur application

```rs
fn set_background_appearance(&self, background_appearance: WindowBackgroundAppearance) {
    let mut this = self.0.as_ref().lock();
    this.background_appearance = background_appearance;
    let opaque = background_appearance == WindowBackgroundAppearance::Opaque;
    this.renderer.update_transparency(!opaque);
    unsafe {
        this.native_window.setOpaque_(opaque as BOOL);
        let background_color = if opaque {
            NSColor::colorWithSRGBRed_green_blue_alpha_(nil, 0f64, 0f64, 0f64, 1f64)
        } else {
            NSColor::colorWithSRGBRed_green_blue_alpha_(nil, 0f64, 0f64, 0f64, 0.0001)
        };
        this.native_window.setBackgroundColor_(background_color);
    }
}
```

### Preserve native vibrancy tint

```rs
#[cfg(target_os = "macos")]
pub fn swizzle_gpui_blurred_view() {
    use std::sync::atomic::Ordering;
    logging::log("VIBRANCY", "swizzle_gpui_blurred_view() called");

    if SWIZZLE_DONE.swap(true, Ordering::SeqCst) {
        logging::log("VIBRANCY", "Swizzle already done, skipping");
        return;
    }

    let Ok(class_name) = std::ffi::CString::new("BlurredView") else {
        tracing::error!("CString creation failed");
        SWIZZLE_DONE.store(false, Ordering::SeqCst);
        return;
    };

    unsafe {
        let blurred_class = objc::runtime::objc_getClass(class_name.as_ptr());
        if blurred_class.is_null() {
            logging::log("VIBRANCY", "BlurredView class not found (GPUI may not have created it yet)");
            return;
        }
        let update_layer_sel = sel!(updateLayer);
        let original_method =
            objc::runtime::class_getInstanceMethod(blurred_class as *const _, update_layer_sel);
        if original_method.is_null() {
            logging::log("VIBRANCY", "updateLayer method not found on BlurredView");
            return;
        }
        let new_imp: extern "C" fn(&objc::runtime::Object, objc::runtime::Sel) = patched_update_layer;
        let _ = objc::runtime::method_setImplementation(
            original_method as *mut _,
            std::mem::transmute::<_, objc::runtime::Imp>(new_imp),
        );
        logging::log(
            "VIBRANCY",
            "Successfully swizzled BlurredView.updateLayer to preserve CAChameleonLayer tint!",
        );
    }
}
```

## Shared Popup Configuration

Generic popup overlays should document the shared recursive `NSVisualEffectView` path.

```rs
#[cfg(target_os = "macos")]
unsafe fn configure_visual_effect_views_recursive(
    view: id,
    count: &mut usize,
    is_dark: bool,
    material: crate::theme::VibrancyMaterial,
) {
    let is_vev: bool = msg_send![view, isKindOfClass: class!(NSVisualEffectView)];
    if is_vev {
        let view_appearance: id = if is_dark {
            msg_send![class!(NSAppearance), appearanceNamed: NSAppearanceNameVibrantDark]
        } else {
            msg_send![class!(NSAppearance), appearanceNamed: NSAppearanceNameVibrantLight]
        };
        if !view_appearance.is_null() {
            let _: () = msg_send![view, setAppearance: view_appearance];
        }
        let material_value = vibrancy_material_value(material);
        let _: () = msg_send![view, setMaterial: material_value];
        let state = if is_dark { 1isize } else { 0isize };
        let _: () = msg_send![view, setState: state];
        let _: () = msg_send![view, setBlendingMode: 0isize];
        let _: () = msg_send![view, setEmphasized: is_dark];
        *count += 1;
    }
}
```

## Footer Special Case

The launcher footer is not a normal popup blur surface. It is a special in-window glass strip.

```rs
#[cfg(target_os = "macos")]
unsafe fn refresh_main_footer_host(ns_window: id) {
    let content_view: id = msg_send![ns_window, contentView];
    let footer_view = find_subview_by_identifier(content_view, FOOTER_EFFECT_ID);
    let theme = crate::theme::get_cached_theme();
    let is_dark = theme.should_use_dark_vibrancy();
    let material = match theme.get_vibrancy().material {
        crate::theme::VibrancyMaterial::Hud => crate::platform::ns_visual_effect_material::HUD_WINDOW,
        crate::theme::VibrancyMaterial::Popover => crate::platform::ns_visual_effect_material::POPOVER,
        crate::theme::VibrancyMaterial::Menu => crate::platform::ns_visual_effect_material::MENU,
        crate::theme::VibrancyMaterial::Sidebar => crate::platform::ns_visual_effect_material::SIDEBAR,
        crate::theme::VibrancyMaterial::Content => crate::platform::ns_visual_effect_material::CONTENT_BACKGROUND,
    };
    let _: () = msg_send![footer_view, setMaterial: material];
    let _: () = msg_send![footer_view, setState: 1isize];
    let _: () = msg_send![footer_view, setBlendingMode: 1isize];
    let _: () = msg_send![footer_view, setEmphasized: is_dark];
}
```

## Raycast-Parity Rules

- Keep the footer to exactly three affordances: Run, Actions, AI.
- Prefer native blur with minimal visible chrome.
- Use hover only as reinforcement, not as the sole discovery mechanism.
- Keep child popups visually above the parent without demoting the parent panel.
- Separate "blur working" from "theme tint/opacity misconfigured".

## Polish Contract: Alpha, Hover, and Tint

`NSVisualEffectView` blur is only half the result. Native feel comes from keeping overlays translucent enough for blur, then using hover and selection as low-amplitude reinforcement.

### Opacity precedence

1. If `theme.opacity.vibrancy_background` is set, that value wins.
2. Otherwise, `ui_foundation::resolve_window_vibrancy_opacity()` falls back to the mode-specific root opacity.
3. Inner surfaces should usually be lower-alpha than the window root so the system blur remains visible.

### Current numeric contract

| Token | Dark | Light | Why |
| --- | --- | --- | --- |
| `hover` | `0.12` | `0.14` | Reinforcement only |
| `selected` | `0.18` | `0.20` | Clearer than hover, still translucent |
| `dialog` | `0.15` | `0.85` | Dark popups stay airy; light popups need stronger readability |
| `vibrancy_background` theme default | `0.85` | `0.85` | Stable default theme tint |
| Root fallback when unset | `VIBRANCY_DARK_OPACITY` | `VIBRANCY_LIGHT_OPACITY` | `ui_foundation` fallback path |

### Footer hover contract

- Pointer affordance comes from native AppKit cursor rects.
- `mouseEntered:` should tint the button container layer only.
- `mouseExited:` should clear that tint, except the Actions button restores its selected tint while the actions popup remains open.
- Hover must brighten subtly, not paint an opaque band.

## Known Pitfalls

- Material changes alone do not fix tint if `CAChameleonLayer` is still hidden.
- `WindowBackgroundAppearance::Blurred` without semi-transparent tint still looks wrong.
- Footer behavior is a special case; do not treat it as just another detached popup.
- Theme overrides can make a correct blur setup look opaque or washed out.

## Event Passthrough and Interaction Patterns

### The Three-Piece Architecture (NON-NEGOTIABLE)

The native footer requires three pieces working together. **Do not change any one independently.**

1. **Native NSVisualEffectView** in front (`NSSortFront`) with `WithinWindow` blending — provides real blur
2. **`hitTest:` returns `nil`** for non-button areas — critical for scroll passthrough
3. **`deferred()` transparent GPUI hitbox** — blocks GPUI hover without rendering into Metal

### Why each piece exists

| Piece | What happens if removed | What happens if changed wrong |
| --- | --- | --- |
| Native footer | No blur — GPUI has no per-element backdrop blur | N/A |
| hitTest: nil | hitTest: self breaks scroll EVERYWHERE (not just footer zone) | Returning self makes the footer first responder for all events in its 30px region |
| deferred hitbox | GPUI delivers hover to list items behind the footer | A visible div hides the blur; `.relative()` on parent breaks scroll |

### hitTest: returns nil for non-button areas, buttons for button areas

**CRITICAL**: Returning `self` from hitTest breaks scroll everywhere. Returning `nil` lets all events pass through to the GPUI Metal view. Buttons still receive clicks because hitTest returns the button when the point is over one.

```rs
#[cfg(target_os = "macos")]
unsafe fn nearest_footer_button(mut view: id) -> id {
    while view != nil {
        let is_button: cocoa::base::BOOL = msg_send![view, isKindOfClass: class!(NSButton)];
        if is_button == YES {
            return view;
        }
        let superview: id = msg_send![view, superview];
        if superview == nil || superview == view {
            break;
        }
        view = superview;
    }
    nil
}

#[cfg(target_os = "macos")]
extern "C" fn footer_hit_test(
    this: &objc::runtime::Object,
    _: objc::runtime::Sel,
    point: cocoa::foundation::NSPoint,
) -> id {
    unsafe {
        let this_id = this as *const _ as id;
        let hit: id = msg_send![super(this_id, class!(NSVisualEffectView)), hitTest: point];
        let button = nearest_footer_button(hit);
        if button != nil {
            return button;  // Buttons receive clicks
        }
        nil  // Everything else passes through — DO NOT return self
    }
}
```

### GPUI hover blocking via deferred transparent hitbox

GPUI's hover is computed from a pre-rendered hit test (`window.rs:860-882`), NOT from event propagation. `stop_propagation()` does NOT prevent hover. The only way to block hover is `HitboxBehavior::BlockMouseExceptScroll`.

A GPUI div with `block_mouse_except_scroll()` and NO background/border/shadow **paints nothing into the Metal layer** (confirmed by reading `Style::paint()` in `vendor/gpui/src/style.rs:645-700`). It only inserts a hitbox during prepaint.

`deferred()` ensures the hitbox is appended LAST to GPUI's hitbox list. Since `hit_test()` iterates in reverse order, this hitbox is checked FIRST, and `BlockMouseExceptScroll` sets `hover_hitbox_count` to exclude all elements behind it from hover.

```rs
// In render_script_list mini mode, as a child of main_div:
main_div = main_div.child(gpui::deferred(
    div()
        .absolute()
        .bottom_0()
        .left_0()
        .w_full()
        .h(px(crate::window_resize::mini_layout::HINT_STRIP_HEIGHT))
        .block_mouse_except_scroll(),
));
```

**DO NOT** add `.relative()` to `main_div` — this breaks scroll.
**DO NOT** use a flex-child div for the blocker — it takes layout space and renders into Metal, hiding the blur.

### Scroll passthrough

With `hitTest:` returning `nil`, scroll events go directly to GPUI's Metal view — no forwarding needed. The `footer_scroll_wheel` override exists as a safety net but is rarely called:

```rs
#[cfg(target_os = "macos")]
extern "C" fn footer_scroll_wheel(this: &objc::runtime::Object, _: objc::runtime::Sel, event: id) {
    unsafe {
        let next: id = msg_send![this, nextResponder];
        if next != nil {
            let _: () = msg_send![next, scrollWheel: event];
        }
    }
}
```

### Footer buttons need per-button NSTrackingArea (not effect-view-level)

GPUI intercepts `mouseMoved:` events at the GPUIView level (`vendor/gpui_macos/src/window.rs:2002`). Tracking areas on the footer effect view never fire. Each button must have its own tracking area via `updateTrackingAreas` override.

**Buttons are recreated every ~0.5s** by `layout_footer_hints`. To avoid use-after-free crashes:
1. Remove all tracking areas from buttons BEFORE removing them from the view hierarchy
2. Do NOT store CGColor pointers in ivars — recompute from theme in each `mouseEntered:`/`mouseExited:` callback

```rs
#[cfg(target_os = "macos")]
extern "C" fn footer_button_update_tracking_areas(
    this: &objc::runtime::Object,
    _: objc::runtime::Sel,
) {
    unsafe {
        let this_id = this as *const _ as id;
        let _: () = msg_send![super(this_id, class!(NSButton)), updateTrackingAreas];
        // Remove old tracking areas
        let existing: id = msg_send![this, trackingAreas];
        if existing != nil {
            let count: usize = msg_send![existing, count];
            for i in (0..count).rev() {
                let area: id = msg_send![existing, objectAtIndex: i];
                let _: () = msg_send![this, removeTrackingArea: area];
            }
        }
        // Add fresh tracking area
        let opts: usize = 0x01 /* MouseEnteredAndExited */ | 0x80 /* ActiveAlways */ | 0x20 /* InVisibleRect */;
        let bounds: cocoa::foundation::NSRect = msg_send![this, bounds];
        let area: id = msg_send![class!(NSTrackingArea), alloc];
        let area: id = msg_send![
            area,
            initWithRect: bounds
            options: opts
            owner: this_id
            userInfo: nil
        ];
        if area != nil {
            let _: () = msg_send![this, addTrackingArea: area];
        }
    }
}

// mouseEntered: recomputes color from theme (no ivar pointers)
extern "C" fn footer_button_mouse_entered(this: &objc::runtime::Object, _: Sel, _event: id) {
    unsafe {
        let superview: id = msg_send![this, superview];
        if superview == nil { return; }
        let layer: id = msg_send![superview, layer];
        if layer == nil { return; }
        let theme = crate::theme::get_cached_theme();
        let chrome = crate::theme::AppChromeColors::from_theme(&theme);
        let hover_ns: id = ns_color_from_rgba(chrome.hover_rgba);
        if hover_ns != nil {
            let cg: id = msg_send![hover_ns, CGColor];
            if cg != nil {
                let _: () = msg_send![layer, setBackgroundColor: cg];
            }
        }
    }
}
```

## Lessons Learned (Failure Modes)

These are real failures encountered during development. Each cost significant debugging time.

### hitTest: self breaks scroll (not just in the footer)

Returning `self` from the footer's `hitTest:` for non-button areas makes the footer the first responder for ALL events in its 30px strip. Even though `scrollWheel:` forwards to `nextResponder`, this breaks GPUI's scroll handling entirely — scroll stops working everywhere, not just in the footer zone.

### stop_propagation does NOT prevent GPUI hover

GPUI hover is computed during the Capture phase from a pre-rendered hit test (`window.rs:860-882`), BEFORE event listeners run. `cx.stop_propagation()` only stops event bubbling, which happens AFTER hover state is already updated. The only mechanism that prevents hover is `HitboxBehavior::BlockMouseExceptScroll` (or `BlockMouse`).

### Any GPUI div with layout space hides the native blur

The Metal layer renders ALL GPUI elements that participate in layout. Even a div with no `.bg()` can paint over the native NSVisualEffectView if it occupies layout space (flex child, fixed height). Only absolutely positioned divs with no visual properties avoid Metal rendering.

### .relative() on the list parent breaks scroll

Adding `.relative()` (CSS `position: relative`) to the main flex container changes Taffy's layout algorithm in a way that breaks GPUI list scrolling. The deferred absolute hitbox works without `.relative()` because it positions relative to the window root.

### CGColor ivar pointers dangle after button recreation

Buttons are recreated every ~0.5s by `layout_footer_hints`. CGColor pointers stored as ivars become dangling when the NSColor that created them is released. Always recompute colors from the live theme in `mouseEntered:`/`mouseExited:`.

### GPUI intercepts mouseMoved at the GPUIView level

GPUI's GPUIView has its own NSTrackingArea with `NSTrackingMouseMoved | NSTrackingActiveAlways`. The `handle_view_event` function at `vendor/gpui_macos/src/window.rs:2002` intercepts all mouseMoved events before they reach native subviews. Per-button tracking areas with `NSTrackingMouseEnteredAndExited` (NOT mouseMoved) bypass this because they're a different event type.

### Child popups should not visually demote the parent panel

```rs
#[cfg(target_os = "macos")]
pub unsafe fn configure_actions_popup_window(window: id, is_dark: bool) {
    if window.is_null() {
        tracing::warn!(
            event = "actions_popup_configure.null_window",
            "Cannot configure null window as actions popup"
        );
        return;
    }

    // Disable window dragging
    let _: () = msg_send![window, setMovable: false];
    let _: () = msg_send![window, setMovableByWindowBackground: false];

    // Regression guard:
    // Detached child popups can still take mouse focus even when GPUI opens them
    // with `focus: false`. If AppKit promotes the child to the key panel on click,
    // the parent panel visually drops its active shadow even though our close/focus
    // policy keeps it open. `setBecomesKeyOnlyIfNeeded:true` keeps these popup
    // windows in the "clickable child" role instead of eagerly stealing key status.
    //
    // Keep this for Actions-style child popups unless we intentionally rework the
    // parent/child focus model and verify the shadow behavior again.
    let _: () = msg_send![window, setBecomesKeyOnlyIfNeeded: true];

    // Keep the level GPUI assigned (WindowKind::PopUp -> NSPopUpMenuWindowLevel = 101).
    // Do NOT call setLevel here — any override downgrades the popup below the
    // main window which is also at 101. See CLAUDE.md "Window Level Rules".

    // NOTE: We intentionally do NOT set setHidesOnDeactivate:true here.
    // The main window is a non-activating panel (WindowKind::PopUp), so the app
    // is never "active" in the macOS sense. If we set hidesOnDeactivate, the
    // actions popup would immediately hide since the app isn't active.
    // Instead, we manage visibility ourselves via close_actions_window().

    // Disable close animation (NSWindowAnimationBehaviorNone = 2)
    // This prevents the white flash on dismiss
    let _: () = msg_send![window, setAnimationBehavior: NS_WINDOW_ANIMATION_BEHAVIOR_NONE];

    // Disable restoration
    let _: () = msg_send![window, setRestorable: false];

    // Disable frame autosave
    let empty_string: id = msg_send![class!(NSString), string];
    let _: () = msg_send![window, setFrameAutosaveName: empty_string];

    configure_window_vibrancy_common(window, "ACTIONS", "Actions popup", is_dark);

    // SAFETY: `window` is a valid, non-null NSWindow pointer (checked at function entry).
    // orderFrontRegardless brings the popup visually above the main panel without
    // activating the app — same pattern as show_main_window_without_activation.
    let _: () = msg_send![window, orderFrontRegardless];
}
```

### Flush child confirm overlays inherit the popup blur path and then flatten chrome

```rs
#[cfg(target_os = "macos")]
pub unsafe fn configure_confirm_popup_window(window: id, is_dark: bool) {
    configure_actions_popup_window(window, is_dark);
    let content_view: id = msg_send![window, contentView];
    if content_view != nil {
        let layer: id = msg_send![content_view, layer];
        if layer != nil {
            let _: () = msg_send![layer, setCornerRadius: 0.0_f64];
        }
        let _: () = msg_send![content_view, setWantsLayer: true];
        let layer: id = msg_send![content_view, layer];
        if layer != nil {
            let _: () = msg_send![layer, setCornerRadius: 0.0_f64];
        }
    }
    let _: () = msg_send![window, setHasShadow: false];
}
```

## Caller-Side Ordering Contract

`configure_actions_popup_window()` and `configure_confirm_popup_window()` do not finish the job alone.
The caller still owns native parent/child ordering.

### Actions popup

```rs
// src/actions/window.rs — attach_actions_popup_to_parent_window()
let _: () = msg_send![parent_ns_window, addChildWindow:child_ns_window ordered:NS_WINDOW_ABOVE];
let _: () = msg_send![child_ns_window, orderFrontRegardless];
```

### Confirm popup

```rs
// src/confirm/window.rs — confirm window setup
let _: () = msg_send![main_ns_window, addChildWindow:confirm_ns_window ordered:NS_WINDOW_ABOVE];
let _: () = msg_send![confirm_ns_window, orderFrontRegardless];
let _: () = msg_send![confirm_ns_window, makeKeyWindow];
```

### Why this matters

The shared popup helper gives you blur, material, and `setBecomesKeyOnlyIfNeeded:true`.
The caller-side attachment gives you stable visual stacking above the parent.

If you skip caller-side attachment, you can keep the blur and still lose the
"parent stays active, child stays above" feel. Confirm additionally calls
`makeKeyWindow` so key events (Escape/Enter) route to the confirm dialog.

### Decision Examples

| Input | Expected decision |
| --- | --- |
| "A detached popup that should stay above the launcher but not steal key focus." | Use the actions-popup family: `WindowBackgroundAppearance::Blurred` + shared popup vibrancy config + `setBecomesKeyOnlyIfNeeded:true` + `orderFrontRegardless`. |
| "A footer strip inside the main launcher that needs hoverable buttons but should not block list scrolling." | Use the three-piece architecture: native in-window footer with `WithinWindow` blending + `hitTest: nil` for non-button areas + `deferred()` transparent GPUI hitbox with `block_mouse_except_scroll()`. See "The Three-Piece Architecture" section. |
| "A flush confirm dialog attached to the bottom edge of the launcher." | Reuse the actions-popup blur path, then remove rounded corners and disable the shadow. |
| "A dense slash or mention picker inside ACP chat that should feel attached, not floating." | Use `configure_inline_dropdown_popup_window()`, keep the shared popup blur path, attach it as a native child window, and disable shadow. |
| "A compact dictation pill that must appear without flashing the hidden launcher." | Create a hidden `WindowKind::PopUp`, configure vibrancy after creation, then surface only that window with `orderFrontRegardless` and optionally `makeKeyWindow`. |

## Verification Checklist

Use this checklist before declaring a new overlay pattern "native enough".

### Visual verification

- Blur is actually enabled with `WindowBackgroundAppearance::Blurred`.
- The blur surface still has readable tint; it is not washed out or fully opaque.
- The footer remains a three-affordance strip: `Run`, `Actions`, `AI`.
- Hover highlights brighten subtly instead of painting opaque bands.

### Behavior verification

- Background clicks on footer glass do not trigger unintended actions.
- Real footer buttons still receive hover, pointer cursor, and click events.
- Scroll over the footer still moves the GPUI list behind it.
- **Scroll works everywhere on the list** (not just above the footer).
- GPUI list items behind the footer do NOT show hover highlight.
- Clicking child popups does not visually demote the parent launcher panel.
- Flush confirm overlays have no shadow and no rounded corners.

### Footer-specific regression tests

These are the exact failure modes encountered during development. Test each one:

1. **Scroll**: Mouse wheel scrolls the list when hovering over any part of the window, including the footer zone.
2. **Blur visible**: The native footer shows frosted glass blur — list items are visible but blurred behind it.
3. **Hover blocked**: Moving the mouse over the footer zone does NOT highlight list items behind it.
4. **Button hover**: Moving the mouse over a footer button shows a hover highlight on the button container.
5. **Pointer cursor**: The mouse cursor changes to a pointing hand over footer buttons.
6. **Actions toggle**: Pressing ⌘K shows the Actions button with a persistent selected background.
7. **No crash**: Moving the mouse rapidly over footer buttons does not crash the app.

### Source-of-truth commands

```bash
rg "WindowBackgroundAppearance::Blurred" src vendor
rg "swizzle_gpui_blurred_view|patched_update_layer" src
rg "setBlendingMode: 0isize|setBlendingMode: 1isize" src
rg "setBecomesKeyOnlyIfNeeded: true|orderFrontRegardless" src
rg "FooterAction::Run|FooterAction::Actions|FooterAction::Ai" src tests
rg "block_mouse_except_scroll|deferred" src/render_script_list
rg "hitTest.*nil|hitTest.*this_id" src/footer_popup.rs
```

### Historical Note

Treat the native in-window footer host as the current launcher-footer pattern. `configure_footer_popup_window(window: id, is_dark: bool)` still exists and should be documented as a reusable popup-family configurator, but it should not be described as the primary current launcher footer implementation if `sync_main_footer_popup()` is the live render path.

### Theme Pitfalls

Correct AppKit setup can still look wrong when theme values are stale. Use these defaults when diagnosing "blur works but selection/hover looks opaque":

```json
{
  "colors": {
    "accent": {
      "selected_subtle": "#FFFFFF"
    }
  },
  "opacity": {
    "selected": 0.33,
    "hover": 0.15
  }
}
```
