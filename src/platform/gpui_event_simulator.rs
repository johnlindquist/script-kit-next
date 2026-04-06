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

/// Returns `true` when the given kind represents a surface that is visually
/// attached to a parent window and dispatches mouse events through that parent.
///
/// Attached surfaces need coordinate rebasing: popup-local (x, y) must be
/// translated into the parent window's GPUI dispatch space.
fn is_attached_surface(kind: crate::protocol::AutomationWindowKind) -> bool {
    matches!(
        kind,
        crate::protocol::AutomationWindowKind::ActionsDialog
            | crate::protocol::AutomationWindowKind::PromptPopup
    )
}

/// Translate mouse coordinates from target-local space into the parent
/// window's GPUI dispatch space for attached surfaces.
///
/// The parent window is determined from the popup's recorded `parent_window_id`
/// metadata (set at popup registration time via `register_attached_popup`).
/// If an attached popup has no parent metadata, dispatch **fails closed** with
/// an explicit error instead of silently falling back to Main.
///
/// Detached windows and key events pass through unchanged.
/// Returns `Err` with a deterministic message when bounds are unavailable.
fn rebase_mouse_event_to_dispatch_space(
    resolved: &crate::protocol::AutomationWindowInfo,
    event: &crate::protocol::SimulatedGpuiEvent,
) -> Result<crate::protocol::SimulatedGpuiEvent, String> {
    use crate::protocol::SimulatedGpuiEvent;

    if !is_attached_surface(resolved.kind) {
        return Ok(event.clone());
    }

    // Key events don't have coordinates — pass through.
    if matches!(event, SimulatedGpuiEvent::KeyDown { .. }) {
        return Ok(event.clone());
    }

    let target_bounds = resolved.bounds.as_ref().ok_or_else(|| {
        format!(
            "Resolved target {} ({:?}) has no bounds; cannot translate attached-surface coordinates",
            resolved.id, resolved.kind
        )
    })?;

    // Resolve the parent window from the popup's recorded metadata.
    // Fail closed if no parent metadata exists — never silently fall back to Main.
    let parent_id = resolved.parent_window_id.as_ref().ok_or_else(|| {
        format!(
            "Attached surface {} ({:?}) has no parent_window_id metadata; \
             cannot rebase coordinates (fail-closed: will not silently dispatch against Main)",
            resolved.id, resolved.kind
        )
    })?;

    let parent = crate::windows::resolve_automation_window(Some(
        &crate::protocol::AutomationWindowTarget::Id {
            id: parent_id.clone(),
        },
    ))
    .map_err(|err| {
        format!(
            "Failed to resolve parent window '{}' for attached-surface {} dispatch: {err}",
            parent_id, resolved.id
        )
    })?;

    let parent_bounds = parent.bounds.as_ref().ok_or_else(|| {
        format!(
            "Parent window {} ({:?}) has no bounds; cannot translate attached-surface coordinates for {}",
            parent.id, parent.kind, resolved.id
        )
    })?;

    let offset_x = target_bounds.x - parent_bounds.x;
    let offset_y = target_bounds.y - parent_bounds.y;

    // Log the rebased coordinates for observability, including parent identity.
    match event {
        SimulatedGpuiEvent::MouseMove { x, y }
        | SimulatedGpuiEvent::MouseDown { x, y, .. }
        | SimulatedGpuiEvent::MouseUp { x, y, .. } => {
            tracing::info!(
                target: "script_kit::automation",
                window_id = %resolved.id,
                kind = ?resolved.kind,
                parent_window_id = %parent.id,
                parent_kind = ?parent.kind,
                local_x = x,
                local_y = y,
                offset_x = offset_x,
                offset_y = offset_y,
                rebased_x = x + offset_x,
                rebased_y = y + offset_y,
                "gpui_event_simulation.rebased_coordinates"
            );
        }
        SimulatedGpuiEvent::KeyDown { .. } => {}
    }

    let translated = match event {
        SimulatedGpuiEvent::MouseMove { x, y } => SimulatedGpuiEvent::MouseMove {
            x: x + offset_x,
            y: y + offset_y,
        },
        SimulatedGpuiEvent::MouseDown { x, y, button } => SimulatedGpuiEvent::MouseDown {
            x: x + offset_x,
            y: y + offset_y,
            button: button.clone(),
        },
        SimulatedGpuiEvent::MouseUp { x, y, button } => SimulatedGpuiEvent::MouseUp {
            x: x + offset_x,
            y: y + offset_y,
            button: button.clone(),
        },
        SimulatedGpuiEvent::KeyDown { .. } => event.clone(),
    };

    Ok(translated)
}

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
    /// The dispatch path that was used: `"exact_handle"` when the resolved
    /// automation target had a registered runtime handle, `"window_role_fallback"`
    /// when we fell back to `WindowRole`-based dispatch, or `None` on error.
    pub dispatch_path: Option<String>,
    /// The resolved automation window ID, when available.
    pub resolved_window_id: Option<String>,
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
    dispatch_path: &str,
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
                dispatch_path: Some(dispatch_path.to_string()),
                resolved_window_id: Some(resolved_id.to_string()),
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
                dispatch_path: Some(dispatch_path.to_string()),
                resolved_window_id: Some(resolved_id.to_string()),
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
                dispatch_path: None,
                resolved_window_id: None,
            };
        }
    };

    // Rebase mouse coordinates for attached surfaces (ActionsDialog, PromptPopup)
    // before GPUI dispatch so clicks land inside the popup, not at the same
    // (x, y) in the parent window.
    let event = match rebase_mouse_event_to_dispatch_space(&resolved, event) {
        Ok(rebased) => rebased,
        Err(msg) => {
            tracing::warn!(
                target: "script_kit::automation",
                request_id = %request_id,
                window_id = %resolved.id,
                error = %msg,
                "gpui_event_simulation.coordinate_translation_failed"
            );
            return GpuiEventDispatchResult {
                success: false,
                error_code: Some("coordinate_translation_failed".to_string()),
                error: Some(msg),
                dispatch_path: None,
                resolved_window_id: Some(resolved.id.clone()),
            };
        }
    };

    let event_type = match &event {
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
        return dispatch_with_any_handle(
            handle,
            request_id,
            &resolved.id,
            event_type,
            &event,
            cx,
            "exact_handle",
        );
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
            dispatch_path: None,
            resolved_window_id: Some(resolved.id.clone()),
        };
    }

    // 3b. Map to WindowRole and get the handle.
    let role = match automation_kind_to_window_role(resolved.kind) {
        Some(r) => r,
        None => {
            // For attached surfaces (ActionsDialog, PromptPopup), dispatch
            // to the parent window since they share its GPUI context.
            // Resolve the parent from the popup's recorded metadata.
            let parent_kind = resolved
                .parent_kind
                .unwrap_or(crate::protocol::AutomationWindowKind::Main);
            match automation_kind_to_window_role(parent_kind) {
                Some(r) => r,
                None => {
                    let msg = format!(
                        "Attached surface {} ({:?}) parent kind {:?} has no WindowRole mapping",
                        resolved.id, resolved.kind, parent_kind
                    );
                    tracing::warn!(
                        target: "script_kit::automation",
                        request_id = %request_id,
                        error = %msg,
                        "gpui_event_simulation.parent_role_unmapped"
                    );
                    return GpuiEventDispatchResult {
                        success: false,
                        error_code: Some("handle_unavailable".to_string()),
                        error: Some(msg),
                        dispatch_path: None,
                        resolved_window_id: Some(resolved.id.clone()),
                    };
                }
            }
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
                dispatch_path: None,
                resolved_window_id: Some(resolved.id.clone()),
            };
        }
    };

    dispatch_with_any_handle(
        handle,
        request_id,
        &resolved.id,
        event_type,
        &event,
        cx,
        "window_role_fallback",
    )
}

fn parse_mouse_button(button: Option<&str>) -> gpui::MouseButton {
    match button {
        Some("right") => gpui::MouseButton::Right,
        Some("middle") => gpui::MouseButton::Middle,
        _ => gpui::MouseButton::Left,
    }
}
