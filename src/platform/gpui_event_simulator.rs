//! GPUI Event Simulator
//!
//! Dispatches [`SimulatedGpuiEvent`] through GPUI's real input pipeline,
//! targeting a specific window resolved via the automation registry.
//!
//! This is explicitly separate from the legacy `simulateKey` path in
//! `runtime_stdin_match_simulate_key.rs`, which routes through `AppView`
//! match arms and bypasses GPUI intercepts.
//!
//! Note: these functions are called from the binary crate via `include!()`,
//! not from the library crate directly, so they appear unused to `--lib`.
#![allow(dead_code)]

/// Returns `true` when GPUI dispatch still collapses all windows of this kind
/// to a single `WindowRole`, meaning it cannot distinguish between multiple
/// visible windows of the same kind.
fn kind_collapses_to_single_window_role(
    kind: crate::protocol::AutomationWindowKind,
) -> bool {
    automation_kind_to_window_role(kind).is_some()
}

/// Count how many visible windows share the given [`AutomationWindowKind`].
fn visible_window_count_for_kind(kind: crate::protocol::AutomationWindowKind) -> usize {
    crate::windows::list_automation_windows()
        .into_iter()
        .filter(|w| w.kind == kind && w.visible)
        .count()
}

/// Map an [`AutomationWindowKind`] to the corresponding [`WindowRole`]
/// used by the unified window registry.
///
/// Returns `None` for kinds that don't map to a single `WindowRole`
/// (e.g. `ActionsDialog` and `PromptPopup` are attached to their parent
/// window, not registered as independent window handles).
fn automation_kind_to_window_role(
    kind: crate::protocol::AutomationWindowKind,
) -> Option<crate::windows::WindowRole> {
    use crate::protocol::AutomationWindowKind;
    use crate::windows::WindowRole;

    match kind {
        AutomationWindowKind::Main => Some(WindowRole::Main),
        AutomationWindowKind::Notes => Some(WindowRole::Notes),
        AutomationWindowKind::Ai => Some(WindowRole::Ai),
        AutomationWindowKind::MiniAi => Some(WindowRole::AiMini),
        AutomationWindowKind::AcpDetached => Some(WindowRole::AcpChat),
        // Attached surfaces — no independent window handle
        AutomationWindowKind::ActionsDialog | AutomationWindowKind::PromptPopup => None,
    }
}

/// Build a GPUI [`Keystroke`] from a `SimulatedGpuiEvent::KeyDown`.
fn build_keystroke(
    key: &str,
    modifiers: &[crate::stdin_commands::KeyModifier],
    text: Option<&str>,
) -> gpui::Keystroke {
    use crate::stdin_commands::KeyModifier;

    let mut mods = gpui::Modifiers::default();
    for m in modifiers {
        match m {
            KeyModifier::Cmd => mods.platform = true,
            KeyModifier::Shift => mods.shift = true,
            KeyModifier::Alt => mods.alt = true,
            KeyModifier::Ctrl => mods.control = true,
        }
    }

    gpui::Keystroke {
        modifiers: mods,
        key: key.to_string(),
        key_char: text.map(String::from),
    }
}

/// Result of a GPUI event simulation dispatch.
pub(crate) struct GpuiEventDispatchResult {
    /// Whether the event was dispatched (even if not consumed by any handler).
    pub success: bool,
    /// Error message if dispatch could not be attempted.
    pub error: Option<String>,
}

/// Dispatch a [`SimulatedGpuiEvent`] to the resolved target window
/// through GPUI's real input pipeline.
///
/// This function must be called on the main GPUI thread with valid
/// `window` and `cx` references.  It resolves the target via the
/// automation registry, maps to a `WindowRole`, fetches the window
/// handle, and dispatches through `Window::dispatch_keystroke` (for
/// key events) or `Window::dispatch_event` (for mouse events).
///
/// # Tracing
///
/// Emits structured logs at `script_kit::automation` with:
/// - `request_id`, `window_id`, `kind`, `event_type` on entry
/// - `success` / `error` on completion
pub(crate) fn dispatch_gpui_event(
    request_id: &str,
    target: Option<&crate::protocol::AutomationWindowTarget>,
    event: &crate::protocol::SimulatedGpuiEvent,
    cx: &mut gpui::App,
) -> GpuiEventDispatchResult {
    use crate::protocol::SimulatedGpuiEvent;

    // 1. Resolve the target window
    let resolved = match crate::windows::resolve_automation_window(target) {
        Ok(info) => info,
        Err(err) => {
            tracing::warn!(
                target: "script_kit::automation",
                request_id = %request_id,
                error = %err,
                "gpui_event_simulation.target_failed"
            );
            return GpuiEventDispatchResult {
                success: false,
                error: Some(err.to_string()),
            };
        }
    };

    let event_type = match event {
        SimulatedGpuiEvent::KeyDown { .. } => "keyDown",
        SimulatedGpuiEvent::MouseMove { .. } => "mouseMove",
        SimulatedGpuiEvent::MouseDown { .. } => "mouseDown",
        SimulatedGpuiEvent::MouseUp { .. } => "mouseUp",
    };

    tracing::info!(
        target: "script_kit::automation",
        request_id = %request_id,
        window_id = %resolved.id,
        kind = ?resolved.kind,
        event_type = %event_type,
        "gpui_event_simulation.dispatch"
    );

    // 2a. Ambiguity guard — fail closed whenever the resolved kind still
    //     routes through a single WindowRole and more than one visible
    //     window shares that kind.  This is unconditional: even an
    //     unqualified `{"type":"kind","kind":"acpDetached"}` target is
    //     rejected when two detached ACP windows are visible, because
    //     GPUI dispatch cannot distinguish between them.
    let visible_count = visible_window_count_for_kind(resolved.kind);
    if kind_collapses_to_single_window_role(resolved.kind) && visible_count > 1 {
        let msg = format!(
            "Resolved target {} ({:?}) is ambiguous: {} visible windows share this kind \
             and GPUI dispatch still routes through one WindowRole",
            resolved.id, resolved.kind, visible_count
        );
        tracing::warn!(
            target: "script_kit::automation",
            request_id = %request_id,
            window_id = %resolved.id,
            kind = ?resolved.kind,
            visible_count = visible_count,
            "gpui_event_simulation.ambiguous_role"
        );
        return GpuiEventDispatchResult {
            success: false,
            error: Some(msg),
        };
    }

    // 2b. Map to WindowRole and get the handle
    let role = match automation_kind_to_window_role(resolved.kind) {
        Some(r) => r,
        None => {
            // For attached surfaces (ActionsDialog, PromptPopup), dispatch
            // to the parent (Main) window since they share its GPUI context.
            crate::windows::WindowRole::Main
        }
    };

    let handle = match crate::windows::get_valid_window(role, cx) {
        Some(h) => h,
        None => {
            let msg = format!(
                "Window handle not available for role {:?} (kind {:?})",
                role, resolved.kind
            );
            tracing::warn!(
                target: "script_kit::automation",
                request_id = %request_id,
                error = %msg,
                "gpui_event_simulation.no_handle"
            );
            return GpuiEventDispatchResult {
                success: false,
                error: Some(msg),
            };
        }
    };

    // 3. Dispatch through the real GPUI pipeline
    let dispatch_result = handle.update(cx, |_root, window, cx| match event {
        SimulatedGpuiEvent::KeyDown {
            key,
            modifiers,
            text,
        } => {
            let keystroke = build_keystroke(key, modifiers, text.as_deref());
            window.dispatch_keystroke(keystroke, cx);
        }
        SimulatedGpuiEvent::MouseMove { x, y } => {
            let position = gpui::point(gpui::px(*x as f32), gpui::px(*y as f32));
            let platform_event = gpui::PlatformInput::MouseMove(gpui::MouseMoveEvent {
                position,
                pressed_button: None,
                modifiers: gpui::Modifiers::default(),
            });
            window.dispatch_event(platform_event, cx);
        }
        SimulatedGpuiEvent::MouseDown { x, y, button } => {
            let position = gpui::point(gpui::px(*x as f32), gpui::px(*y as f32));
            let mouse_button = parse_mouse_button(button.as_deref());
            let platform_event = gpui::PlatformInput::MouseDown(gpui::MouseDownEvent {
                button: mouse_button,
                position,
                modifiers: gpui::Modifiers::default(),
                click_count: 1,
                first_mouse: false,
            });
            window.dispatch_event(platform_event, cx);
        }
        SimulatedGpuiEvent::MouseUp { x, y, button } => {
            let position = gpui::point(gpui::px(*x as f32), gpui::px(*y as f32));
            let mouse_button = parse_mouse_button(button.as_deref());
            let platform_event = gpui::PlatformInput::MouseUp(gpui::MouseUpEvent {
                button: mouse_button,
                position,
                modifiers: gpui::Modifiers::default(),
                click_count: 1,
            });
            window.dispatch_event(platform_event, cx);
        }
    });

    match dispatch_result {
        Ok(()) => {
            tracing::info!(
                target: "script_kit::automation",
                request_id = %request_id,
                window_id = %resolved.id,
                event_type = %event_type,
                "gpui_event_simulation.complete"
            );
            GpuiEventDispatchResult {
                success: true,
                error: None,
            }
        }
        Err(err) => {
            let msg = format!("GPUI dispatch failed: {err}");
            tracing::warn!(
                target: "script_kit::automation",
                request_id = %request_id,
                error = %msg,
                "gpui_event_simulation.failed"
            );
            GpuiEventDispatchResult {
                success: false,
                error: Some(msg),
            }
        }
    }
}

fn parse_mouse_button(button: Option<&str>) -> gpui::MouseButton {
    match button {
        Some("right") => gpui::MouseButton::Right,
        Some("middle") => gpui::MouseButton::Middle,
        _ => gpui::MouseButton::Left,
    }
}
