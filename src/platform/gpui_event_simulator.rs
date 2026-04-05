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
fn kind_collapses_to_single_window_role(kind: crate::protocol::AutomationWindowKind) -> bool {
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
    /// Machine-readable error category: `target_not_found`, `target_ambiguous`,
    /// `handle_unavailable`, or `dispatch_failed`.
    pub error_code: Option<String>,
    /// Human-readable error message if dispatch could not be attempted.
    pub error: Option<String>,
}

/// Dispatch a simulated event through an [`AnyWindowHandle`] via the real GPUI
/// input pipeline and return the result.
fn dispatch_with_any_handle(
    handle: gpui::AnyWindowHandle,
    request_id: &str,
    resolved_id: &str,
    event_type: &str,
    event: &crate::protocol::SimulatedGpuiEvent,
    cx: &mut gpui::App,
) -> GpuiEventDispatchResult {
    use crate::protocol::SimulatedGpuiEvent;

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
            window.dispatch_event(
                gpui::PlatformInput::MouseMove(gpui::MouseMoveEvent {
                    position,
                    pressed_button: None,
                    modifiers: gpui::Modifiers::default(),
                }),
                cx,
            );
        }
        SimulatedGpuiEvent::MouseDown { x, y, button } => {
            let position = gpui::point(gpui::px(*x as f32), gpui::px(*y as f32));
            window.dispatch_event(
                gpui::PlatformInput::MouseDown(gpui::MouseDownEvent {
                    button: parse_mouse_button(button.as_deref()),
                    position,
                    modifiers: gpui::Modifiers::default(),
                    click_count: 1,
                    first_mouse: false,
                }),
                cx,
            );
        }
        SimulatedGpuiEvent::MouseUp { x, y, button } => {
            let position = gpui::point(gpui::px(*x as f32), gpui::px(*y as f32));
            window.dispatch_event(
                gpui::PlatformInput::MouseUp(gpui::MouseUpEvent {
                    button: parse_mouse_button(button.as_deref()),
                    position,
                    modifiers: gpui::Modifiers::default(),
                    click_count: 1,
                }),
                cx,
            );
        }
    });

    match dispatch_result {
        Ok(()) => {
            tracing::info!(
                target: "script_kit::automation",
                request_id = %request_id,
                window_id = %resolved_id,
                event_type = %event_type,
                "gpui_event_simulation.dispatch_exact_complete"
            );
            GpuiEventDispatchResult {
                success: true,
                error_code: None,
                error: None,
            }
        }
        Err(err) => {
            let msg = format!("GPUI dispatch failed: {err}");
            tracing::warn!(
                target: "script_kit::automation",
                request_id = %request_id,
                window_id = %resolved_id,
                error = %msg,
                "gpui_event_simulation.failed"
            );
            GpuiEventDispatchResult {
                success: false,
                error_code: Some("dispatch_failed".to_string()),
                error: Some(msg),
            }
        }
    }
}

/// Dispatch a [`SimulatedGpuiEvent`] to the resolved target window
/// through GPUI's real input pipeline.
///
/// **Exact-handle dispatch**: If the resolved automation target has a
/// registered runtime handle (via [`upsert_runtime_window_handle`]),
/// events are dispatched directly to that exact window.  This allows
/// two visible detached ACP windows to be targeted independently by
/// exact ID.
///
/// **Fallback dispatch**: When no exact runtime handle exists, the
/// function maps the resolved kind to a [`WindowRole`] and dispatches
/// through the unified window registry.  If more than one visible
/// window shares the same kind and no exact handle is available,
/// dispatch fails closed with `target_ambiguous`.
///
/// # Tracing
///
/// Emits structured logs at `script_kit::automation` with:
/// - `request_id`, `window_id`, `kind`, `event_type` on entry
/// - `dispatch_exact_handle` when using the runtime handle registry
/// - `runtime_handle_missing` when falling back to WindowRole
/// - `dispatch_exact_complete` / `failed` on completion
pub(crate) fn dispatch_gpui_event(
    request_id: &str,
    target: Option<&crate::protocol::AutomationWindowTarget>,
    event: &crate::protocol::SimulatedGpuiEvent,
    cx: &mut gpui::App,
) -> GpuiEventDispatchResult {
    use crate::protocol::SimulatedGpuiEvent;

    // 1. Resolve the target window via the automation metadata registry.
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
                error_code: Some("target_not_found".to_string()),
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

    // 2. Try exact runtime handle first — this preserves the resolved
    //    automation window identity all the way through GPUI dispatch,
    //    so two detached ACP windows can be targeted independently.
    if let Some(handle) = crate::windows::get_valid_runtime_window_handle(&resolved.id, cx) {
        tracing::info!(
            target: "script_kit::automation",
            request_id = %request_id,
            window_id = %resolved.id,
            "gpui_event_simulation.dispatch_exact_handle"
        );
        return dispatch_with_any_handle(handle, request_id, &resolved.id, event_type, event, cx);
    }

    // 3. No exact handle — fall back to WindowRole-based dispatch.
    tracing::warn!(
        target: "script_kit::automation",
        request_id = %request_id,
        window_id = %resolved.id,
        kind = ?resolved.kind,
        "gpui_event_simulation.runtime_handle_missing"
    );

    // 3a. Ambiguity guard — fail closed when multiple visible windows
    //     share the same kind and no exact runtime handle is registered.
    let visible_count = visible_window_count_for_kind(resolved.kind);
    if kind_collapses_to_single_window_role(resolved.kind) && visible_count > 1 {
        let msg = format!(
            "Resolved target {} ({:?}) is ambiguous: {} visible windows share this kind \
             and no exact runtime handle is registered",
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
            error_code: Some("target_ambiguous".to_string()),
            error: Some(msg),
        };
    }

    // 3b. Map to WindowRole and get the handle.
    let role = match automation_kind_to_window_role(resolved.kind) {
        Some(r) => r,
        None => {
            // For attached surfaces (ActionsDialog, PromptPopup), dispatch
            // to the parent (Main) window since they share its GPUI context.
            crate::windows::WindowRole::Main
        }
    };

    let handle = match crate::windows::get_valid_window(role, cx) {
        Some(h) => {
            let any: gpui::AnyWindowHandle = h.into();
            any
        }
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
                error_code: Some("handle_unavailable".to_string()),
                error: Some(msg),
            };
        }
    };

    dispatch_with_any_handle(handle, request_id, &resolved.id, event_type, event, cx)
}

fn parse_mouse_button(button: Option<&str>) -> gpui::MouseButton {
    match button {
        Some("right") => gpui::MouseButton::Right,
        Some("middle") => gpui::MouseButton::Middle,
        _ => gpui::MouseButton::Left,
    }
}
