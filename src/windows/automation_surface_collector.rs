//! Secondary-surface semantic element collectors.
//!
//! Provides [`collect_surface_snapshot`] which returns semantic elements for
//! non-main automation windows (the snap overlay).
//!
//! Used by both `getElements` and `inspectAutomationWindow` so agents see one
//! consistent semantic model regardless of which protocol command they use.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use crate::protocol;
use crate::protocol::{
    AutomationWindowInfo, AutomationWindowKind, ElementInfo, ElementStyleInfo, ElementType,
};

fn paint_measurement_component_type(stable_id: &str) -> protocol::LayoutComponentType {
    use protocol::LayoutComponentType;

    if stable_id.starts_with("list-row:")
        || stable_id.contains("transcript-row-")
        || stable_id.starts_with("chat-transcript-user-")
        || stable_id.starts_with("chat-transcript-response-")
        || stable_id.starts_with("actions-row:")
        || stable_id.starts_with("agent_chat-history-popup-row-")
    {
        return LayoutComponentType::ListItem;
    }

    if stable_id == crate::components::builtin_leading_separator::BUILTIN_LEADING_SEPARATOR_ID {
        return LayoutComponentType::Header;
    }

    match stable_id {
        "main-view-context-cwd-button"
        | "main-view-context-model-button"
        | "agent-chat-send-button" => LayoutComponentType::Button,
        "main-view-input-shell"
        | "main-view-input-body"
        | "focused-text-mini-input-row"
        | "focused-text-mini-scope-row"
        | "actions-search"
        | "notes-editor-input-viewport" => LayoutComponentType::Input,
        "agent-chat-transcript-viewport"
        | "chat-transcript-viewport"
        | "actions-list-viewport"
        | "agent_chat-history-popup-list" => LayoutComponentType::List,
        "native-main-window-footer-spacer" | "snap-overlay-root" => LayoutComponentType::Panel,
        "main-view-header"
        | "notes-titlebar"
        | "actions-context-header"
        | "agent_chat-history-popup-header" => LayoutComponentType::Header,
        "main-view-shell"
        | "main-view-context-zone"
        | "main-view-main"
        | "notes-window-root"
        | "actions-window-root"
        | "actions-dialog-root" => LayoutComponentType::Container,
        _ => LayoutComponentType::Other,
    }
}

#[cfg(test)]
mod paint_measurement_component_type_tests {
    #[gpui::test]
    fn standalone_snap_layout_uses_actual_paint_and_rejects_replaced_generation(
        cx: &mut gpui::TestAppContext,
    ) {
        use crate::window_control::snap_session::{SnapOverlayModel, SnapOverlayTarget};
        use gpui::{point, px, size, AppContext as _};
        let handle = cx.update(|cx| {
            cx.open_window(
                gpui::WindowOptions {
                    window_bounds: Some(gpui::WindowBounds::Windowed(gpui::Bounds::new(
                        point(px(0.), px(0.)),
                        size(px(400.), px(240.)),
                    ))),
                    focus: false,
                    show: false,
                    ..Default::default()
                },
                |_, cx| cx.new(|_| crate::window_control::SnapOverlayView::new()),
            )
            .unwrap()
        });
        cx.update(|cx| {
            handle
                .update(cx, |view, _window, cx| {
                    let display = crate::window_control::Bounds {
                        x: 0,
                        y: 0,
                        width: 400,
                        height: 240,
                    };
                    view.set_model(
                        Some(SnapOverlayModel {
                            display_bounds: display,
                            mode: crate::window_control::SnapMode::Simple,
                            is_dominant: true,
                            targets: vec![SnapOverlayTarget {
                                tile: crate::window_control::TilePosition::LeftHalf,
                                bounds: crate::window_control::Bounds {
                                    width: 200,
                                    ..display
                                },
                                active: true,
                            }],
                        }),
                        cx,
                    );
                })
                .unwrap()
        });
        cx.update(|cx| {
            gpui::AnyWindowHandle::from(handle)
                .update(cx, |_, window, cx| window.draw(cx).clear(cx))
                .unwrap()
        });
        let info = cx.update(|cx| {
            crate::windows::register_runtime_window_instance(
                super::AutomationWindowInfo {
                    id: "test:standalone-snap-layout".into(),
                    kind: super::AutomationWindowKind::SnapOverlay,
                    title: Some("Snap".into()),
                    focused: false,
                    visible: false,
                    semantic_surface: Some("snapOverlay".into()),
                    bounds: None,
                    parent_window_id: None,
                    parent_window_generation: None,
                    parent_kind: None,
                    pid: Some(std::process::id()),
                    generation: None,
                },
                handle.into(),
                cx,
            )
            .unwrap()
        });
        let layout = cx
            .update(|cx| super::collect_registered_surface_layout(&info, cx))
            .unwrap();
        let target = layout
            .components
            .iter()
            .find(|component| component.name == "snap:target:LeftHalf")
            .expect("real target paint");
        assert_eq!(target.bounds.width, 200.0);
        assert_eq!(target.bounds.height, 240.0);
        assert!(target
            .measurement_frame_generation
            .is_some_and(|frame| frame > 0));
        let mut stale = info.clone();
        stale.generation = info.generation.map(|generation| generation + 1);
        assert!(cx
            .update(|cx| super::collect_registered_surface_layout(&stale, cx))
            .is_err());
        cx.update(|cx| {
            handle
                .update(cx, |_, window, _| {
                    window.refresh();
                    assert!(!window.rendered_frame_is_current());
                })
                .unwrap();
            let error = super::collect_registered_surface_layout(&info, cx).unwrap_err();
            assert!(error
                .to_string()
                .contains("surface_layout_unavailable:unpainted_or_stale"));
        });
        crate::windows::remove_runtime_window_instance(&info.id, info.generation.unwrap());
        cx.update(|cx| {
            handle
                .update(cx, |_, window, _| window.remove_window())
                .unwrap()
        });
    }
}
/// Machine-readable indicator of the semantic element quality level.
///
/// Mirrors [`crate::protocol::SemanticQuality`] at the collector layer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SnapshotQuality {
    /// Full semantic elements collected.
    #[default]
    Full,
    /// Only a panel-level element was collected (entity unavailable).
    PanelOnly,
}

impl SnapshotQuality {
    pub const fn projection_quality(self) -> crate::protocol::ProjectionQuality {
        match self {
            Self::Full => crate::protocol::ProjectionQuality::Complete,
            Self::PanelOnly => crate::protocol::ProjectionQuality::Partial,
        }
    }

    pub fn reason_codes(self) -> Vec<crate::protocol::ProjectionReason> {
        match self {
            Self::Full => Vec::new(),
            Self::PanelOnly => vec![crate::protocol::ProjectionReason::PanelOnly],
        }
    }
}

/// Lightweight snapshot of semantic elements from a non-main surface.
#[derive(Clone, Debug, Default)]
pub struct SurfaceElementSnapshot {
    pub elements: Vec<ElementInfo>,
    pub total_count: usize,
    pub focused_semantic_id: Option<String>,
    pub selected_semantic_id: Option<String>,
    pub warnings: Vec<String>,
    /// Semantic quality level of this snapshot.
    pub quality: SnapshotQuality,
}

impl SurfaceElementSnapshot {
    /// Returns semantic fallback warnings relevant to popup capture receipts.
    ///
    /// These are the `panel_only_*` warnings that indicate the surface could
    /// not be fully introspected and only a panel-level element was collected.
    /// Agents use these to know when semantic receipts are degraded for a
    /// popup surface.
    pub fn popup_semantic_warnings(&self) -> Vec<String> {
        self.warnings
            .iter()
            .filter(|w| w.starts_with("panel_only_"))
            .cloned()
            .collect()
    }
}

fn element(
    semantic_id: &str,
    element_type: ElementType,
    text: Option<String>,
    value: Option<String>,
    selected: Option<bool>,
    focused: Option<bool>,
    index: Option<usize>,
) -> ElementInfo {
    ElementInfo {
        semantic_id: semantic_id.to_string(),
        element_type,
        text,
        value,
        content: None,
        selected,
        focused,
        index,
        role: None,
        kind: None,
        source: None,
        source_name: None,
        selectable: None,
        status_kind: None,
        action_disabled: None,
        style: None,
    }
}

fn conversation_semantic_role(
    role: crate::components::main_view_chrome::SemanticChipRole,
) -> crate::protocol::ConversationSemanticRole {
    use crate::components::main_view_chrome::SemanticChipRole;
    use crate::protocol::ConversationSemanticRole;

    match role {
        SemanticChipRole::ContextAttachment => ConversationSemanticRole::ContextChip,
        SemanticChipRole::Identity => ConversationSemanticRole::IdentityBadge,
        SemanticChipRole::DestinationSelector => ConversationSemanticRole::DestinationSelector,
    }
}

fn conversation_semantic_action(
    role: crate::components::main_view_chrome::SemanticChipRole,
    action: crate::components::main_view_chrome::SemanticChipAction,
) -> crate::protocol::ConversationSemanticAction {
    use crate::components::main_view_chrome::{SemanticChipAction, SemanticChipRole};
    use crate::protocol::ConversationSemanticAction;

    match (role, action) {
        (SemanticChipRole::ContextAttachment, SemanticChipAction::OpenDetails) => {
            ConversationSemanticAction::OpenContextDetails
        }
        (SemanticChipRole::ContextAttachment, SemanticChipAction::RemoveContext) => {
            ConversationSemanticAction::RemoveContext
        }
        (SemanticChipRole::Identity, SemanticChipAction::OpenSelector) => {
            ConversationSemanticAction::OpenIdentitySelector
        }
        (
            SemanticChipRole::Identity,
            SemanticChipAction::OpenDetails | SemanticChipAction::OpenSurface,
        ) => ConversationSemanticAction::OpenIdentityDetails,
        (SemanticChipRole::DestinationSelector, SemanticChipAction::SelectDestination) => {
            ConversationSemanticAction::SelectDestination
        }
        _ => unreachable!("SemanticChipSpec rejects role/action mismatches"),
    }
}

fn semantic_chip_action_element(
    chip: &crate::components::main_view_chrome::SemanticChipSpec,
    action: Option<crate::components::main_view_chrome::SemanticChipAction>,
    trailing: bool,
) -> ElementInfo {
    let action = action.map(|action| conversation_semantic_action(chip.role, action));
    let semantic_id = if trailing {
        action.map_or_else(
            || format!("{}:unavailable", chip.semantic_id),
            |action| format!("{}:{}", chip.semantic_id, action.as_str()),
        )
    } else {
        chip.semantic_id.to_string()
    };

    ElementInfo {
        semantic_id,
        element_type: if chip.enabled && action.is_some() {
            ElementType::Button
        } else {
            ElementType::Panel
        },
        text: Some(if trailing {
            format!("Remove {}", chip.label)
        } else {
            chip.label.to_string()
        }),
        value: (!trailing && !chip.shortcut_tokens.is_empty())
            .then(|| chip.shortcut_tokens.join("")),
        content: None,
        selected: None,
        focused: None,
        index: None,
        role: Some(conversation_semantic_role(chip.role).as_str().to_string()),
        kind: action.map(|action| action.as_str().to_string()),
        source: Some("ConversationSemanticChip".to_string()),
        source_name: None,
        selectable: Some(chip.enabled && action.is_some()),
        status_kind: None,
        action_disabled: chip.disabled_reason.as_ref().map(ToString::to_string),
        style: None,
    }
}

/// Project the same typed semantic chip specification used by renderers into
/// the element protocol. Every main and secondary conversational collector
/// goes through this function, so role/action strings cannot drift by host.
#[allow(
    dead_code,
    reason = "the separately compiled application binary's flow and day page element collectors call this"
)]
pub(crate) fn collect_semantic_chip_element(
    chip: &crate::components::main_view_chrome::SemanticChipSpec,
) -> ElementInfo {
    semantic_chip_action_element(chip, chip.body_action, false)
}

#[allow(dead_code)] // The binary's element collectors project conversation commands with this.
pub(crate) fn collect_conversation_command_elements<Handler>(
    commands: &[crate::components::conversation_actions::BoundConversationCommand<Handler>],
) -> Vec<ElementInfo> {
    commands
        .iter()
        .map(|command| {
            let action = command.descriptor.command_action();
            ElementInfo {
                semantic_id: action.id.clone(),
                element_type: ElementType::Button,
                text: Some(action.title),
                value: action.shortcut,
                content: None,
                selected: Some(false),
                focused: Some(false),
                index: None,
                role: Some("conversationCommand".to_string()),
                kind: Some(action.id),
                source: Some("ConversationCommandDescriptor".to_string()),
                source_name: None,
                selectable: Some(action.availability.is_executable()),
                status_kind: action
                    .requires_confirmation
                    .then(|| "confirmationRequired".to_string()),
                // Preserve the host's exact reviewed copy; the shared action
                // owns availability while the typed host reason owns wording.
                action_disabled: command
                    .descriptor
                    .availability
                    .disabled_reason()
                    .map(str::to_string),
                style: None,
            }
        })
        .collect()
}

/// Resolve only the requested runtime lifetime, never a singleton replacement.
pub(crate) fn current_surface_metadata(
    resolved: &AutomationWindowInfo,
) -> Option<AutomationWindowInfo> {
    let generation = resolved.generation?;
    let current = crate::windows::automation_window_by_id(&resolved.id)?;
    if generation == 0 || current.generation != Some(generation) || current.kind != resolved.kind {
        return None;
    }
    crate::windows::get_runtime_window_handle_for_generation(&resolved.id, generation)?;
    if let Some(parent_id) = &current.parent_window_id {
        crate::windows::get_runtime_window_handle_for_generation(
            parent_id,
            current.parent_window_generation?,
        )?;
    }
    Some(current)
}

/// Read an exact root without re-entering a window already borrowed by the caller.
pub(crate) fn read_window_root<T: gpui::Render + 'static, R>(
    handle: gpui::WindowHandle<T>,
    window: Option<&gpui::Window>,
    cx: &gpui::App,
    read: impl FnOnce(&T, &gpui::App) -> R,
) -> anyhow::Result<R> {
    if let Some(window) = window {
        anyhow::ensure!(
            window.window_handle() == handle.into(),
            "window_root_owner_mismatch"
        );
        let root = window
            .root::<T>()
            .flatten()
            .ok_or_else(|| anyhow::anyhow!("window_root_type_mismatch"))?;
        Ok(read(root.read(cx), cx))
    } else {
        handle.read_with(cx, read)
    }
}

fn exact_root<T: 'static>(
    resolved: &AutomationWindowInfo,
    cx: &gpui::App,
) -> Option<gpui::Entity<T>> {
    let current = current_surface_metadata(resolved)?;
    crate::windows::get_runtime_window_handle_for_generation(&current.id, current.generation?)?
        .read(cx, |entity: gpui::Entity<T>, _| entity)
        .ok()
}

/// Append the same frame-coherent measured nodes used by Main, without Main ownership.
pub fn append_window_paint_measurements(layout: &mut protocol::LayoutInfo, window: &gpui::Window) {
    let mut measurements: Vec<_> = window.debug_bounds_entries().iter().collect();
    measurements.sort_by(|left, right| left.selector.cmp(&right.selector));
    let frame = window.rendered_frame_generation();
    for component in &mut layout.components {
        if component.measurement_frame_generation.is_none() {
            component.measurement_frame_generation = Some(frame);
        }
    }
    for measurement in measurements {
        let bounds = measurement.bounds;
        let visible = measurement.visible_bounds;
        let clip = measurement.clip_bounds;
        layout.components.push(
            protocol::LayoutComponentInfo::new(
                measurement.selector.clone(),
                paint_measurement_component_type(&measurement.selector),
            )
            .with_bounds(
                bounds.origin.x.as_f32(),
                bounds.origin.y.as_f32(),
                bounds.size.width.as_f32(),
                bounds.size.height.as_f32(),
            )
            .with_measurement("paint-time", "window")
            .with_paint_visibility(
                visible.origin.x.as_f32(),
                visible.origin.y.as_f32(),
                visible.size.width.as_f32(),
                visible.size.height.as_f32(),
                clip.origin.x.as_f32(),
                clip.origin.y.as_f32(),
                clip.size.width.as_f32(),
                clip.size.height.as_f32(),
            )
            .with_measurement_frame(frame),
        );
    }
}

/// Project an exact production root and its completed paint without constructing a launcher.
pub fn collect_registered_surface_layout(
    resolved: &AutomationWindowInfo,
    cx: &mut gpui::App,
) -> anyhow::Result<protocol::LayoutInfo> {
    collect_registered_surface_layout_inner(resolved, None, cx)
}

/// Atomic evaluator-only layout read, qualified by the published render store
/// rather than by a caller's assertion that a dirty window is safe to inspect.
#[cfg(all(target_os = "macos", any(test, feature = "owned-ui-evaluation")))]
pub fn collect_registered_surface_layout_for_completed_frame(
    resolved: &AutomationWindowInfo,
    completed: &protocol::CompletedFrameIdentity,
    cx: &mut gpui::App,
) -> anyhow::Result<protocol::LayoutInfo> {
    anyhow::ensure!(
        resolved.id == completed.target.window_id
            && resolved.generation == completed.target.window_generation,
        "surface_layout_completed_target_mismatch"
    );
    crate::computer_use::owned_render_capture::with_owned_completed_frame(
        completed,
        cx,
        |published, cx| collect_registered_surface_layout_inner(resolved, Some(published), cx),
    )
}

fn layout_frame_matches(
    window: &gpui::Window,
    completed: Option<&protocol::CompletedFrameIdentity>,
) -> bool {
    match completed {
        Some(published) => {
            Some(window.rendered_frame_generation()) == published.target.frame_generation
        }
        None => window.rendered_frame_is_current(),
    }
}

fn collect_registered_surface_layout_inner(
    resolved: &AutomationWindowInfo,
    completed: Option<&protocol::CompletedFrameIdentity>,
    cx: &mut gpui::App,
) -> anyhow::Result<protocol::LayoutInfo> {
    let mut target = current_surface_metadata(resolved)
        .ok_or_else(|| anyhow::anyhow!("surface_layout_target_stale"))?;
    anyhow::ensure!(
        target.kind != AutomationWindowKind::Main,
        "surface_layout_main_requires_main_owner"
    );
    let generation = target
        .generation
        .ok_or_else(|| anyhow::anyhow!("surface_layout_generation_missing"))?;
    let host_policy = crate::windows::runtime_window_host_policy(&target.id, generation)?;
    host_policy.validate()?;
    let handle = crate::windows::get_runtime_window_handle_for_generation(&target.id, generation)
        .ok_or_else(|| anyhow::anyhow!("surface_layout_target_stale"))?;
    let (bounds, viewport, frame) = handle.update(cx, |_, window, _| {
        anyhow::ensure!(
            window.is_owned_hidden() == host_policy.is_hidden(),
            "surface_layout_host_mismatch"
        );
        anyhow::ensure!(
            layout_frame_matches(window, completed),
            "surface_layout_unavailable:unpainted_or_stale"
        );
        if host_policy.is_hidden() {
            anyhow::ensure!(
                !target.visible && !target.focused,
                "surface_layout_hidden_metadata_mismatch"
            );
        }
        Ok((
            window.bounds(),
            window.viewport_size(),
            window.rendered_frame_generation(),
        ))
    })??;
    target.bounds = Some(protocol::AutomationWindowBounds {
        x: bounds.origin.x.as_f32() as f64,
        y: bounds.origin.y.as_f32() as f64,
        width: viewport.width.as_f32() as f64,
        height: viewport.height.as_f32() as f64,
    });
    let mut required = Vec::<String>::new();
    let mut layout = match target.kind {
        AutomationWindowKind::Notes => anyhow::bail!("no_notes_window"),
        AutomationWindowKind::SnapOverlay => {
            let entity = exact_root::<crate::window_control::SnapOverlayView>(&target, cx)
                .ok_or_else(|| anyhow::anyhow!("snap_layout_owner_missing"))?;
            required.push("snap-overlay-root".into());
            if let Some(model) = entity.read(cx).model() {
                required.extend(
                    model
                        .targets
                        .iter()
                        .filter(|target| target.active)
                        .map(|target| format!("snap:target:{:?}", target.tile)),
                );
            }
            protocol::LayoutInfo {
                prompt_type: "snapOverlay".into(),
                ..Default::default()
            }
        }
        _ => anyhow::bail!("surface_layout_owner_unknown"),
    };
    layout.window_width = viewport.width.as_f32();
    layout.window_height = viewport.height.as_f32();
    handle.update(cx, |_, window, _| {
        anyhow::ensure!(
            layout_frame_matches(window, completed) && window.rendered_frame_generation() == frame,
            "surface_layout_frame_changed"
        );
        let entries = window.debug_bounds_entries();
        anyhow::ensure!(
            !entries.is_empty(),
            "surface_layout_unavailable:no_painted_selectors"
        );
        for selector in &required {
            let mut matching = entries.iter().filter(|entry| &entry.selector == selector);
            let entry = matching
                .next()
                .ok_or_else(|| anyhow::anyhow!("surface_layout_unavailable:{selector}"))?;
            anyhow::ensure!(
                matching.next().is_none(),
                "surface_layout_ambiguous:{selector}"
            );
            let bounds = entry.bounds;
            anyhow::ensure!(
                [
                    bounds.origin.x.as_f32(),
                    bounds.origin.y.as_f32(),
                    bounds.size.width.as_f32(),
                    bounds.size.height.as_f32()
                ]
                .into_iter()
                .all(f32::is_finite)
                    && bounds.size.width.as_f32() > 0.0
                    && bounds.size.height.as_f32() > 0.0,
                "surface_layout_invalid_bounds:{selector}"
            );
        }
        append_window_paint_measurements(&mut layout, window);
        Ok::<_, anyhow::Error>(())
    })??;
    anyhow::ensure!(
        current_surface_metadata(&target).is_some(),
        "surface_layout_target_changed"
    );
    layout.timestamp = chrono::Utc::now().to_rfc3339();
    Ok(layout)
}

/// Revisions come from the exact production owner's mutations and applied publication.
pub fn surface_revision_facts(
    resolved: &AutomationWindowInfo,
    window: Option<&gpui::Window>,
    cx: &gpui::App,
) -> Option<(u64, u64, u64, u64)> {
    current_surface_metadata(resolved)?;
    let generation = resolved.generation?;
    let handle =
        crate::windows::get_runtime_window_handle_for_generation(&resolved.id, generation)?;
    if window.is_some_and(|window| window.window_handle() != handle) {
        return None;
    }
    match resolved.kind {
        AutomationWindowKind::Notes => None,
        AutomationWindowKind::SnapOverlay => read_window_root(
            handle.downcast::<crate::window_control::SnapOverlayView>()?,
            window,
            cx,
            |view, _| view.revision_facts(),
        )
        .ok(),
        _ => None,
    }
}

/// Request the real owner's teardown. Callers wait for exact runtime invalidation.
pub fn close_owned_registered_surface(
    resolved: &AutomationWindowInfo,
    cx: &mut gpui::App,
) -> anyhow::Result<()> {
    let resolved = current_surface_metadata(resolved)
        .ok_or_else(|| anyhow::anyhow!("surface_target_stale"))?;
    let generation = resolved
        .generation
        .ok_or_else(|| anyhow::anyhow!("surface_generation_missing"))?;
    anyhow::ensure!(
        crate::windows::runtime_window_host_policy(&resolved.id, generation)?.is_hidden(),
        "owned_surface_required"
    );
    let handle = crate::windows::get_runtime_window_handle_for_generation(&resolved.id, generation)
        .ok_or_else(|| anyhow::anyhow!("surface_target_stale"))?;
    anyhow::ensure!(
        handle.update(cx, |_, window, _| window.is_owned_hidden())?,
        "owned_native_host_required"
    );
    match resolved.kind {
        AutomationWindowKind::Notes => anyhow::bail!("no_notes_window"),
        AutomationWindowKind::SnapOverlay => {
            let handle = handle
                .downcast::<crate::window_control::SnapOverlayView>()
                .ok_or_else(|| anyhow::anyhow!("snap_root_mismatch"))?;
            handle.update(cx, |view, window, _| view.close_owned(window))??;
        }
        _ => anyhow::bail!("surface_close_owner_unknown"),
    }
    Ok(())
}

include!("automation_surface_collector_overlays.rs");

/// Collect semantic elements for a resolved non-main automation window.
///
/// Returns `None` for window kinds that do not yet have a collector.
pub fn collect_surface_snapshot(
    resolved: &AutomationWindowInfo,
    limit: usize,
    cx: &gpui::App,
) -> Option<SurfaceElementSnapshot> {
    let current = current_surface_metadata(resolved)?;
    let resolved = &current;
    let mut snapshot = match resolved.kind {
        AutomationWindowKind::Notes => return None,
        AutomationWindowKind::SnapOverlay => collect_snap_snapshot(resolved, cx)?,
        _ => return None,
    };

    snapshot.total_count = snapshot.elements.len();
    if snapshot.elements.len() > limit {
        snapshot.elements.truncate(limit);
    }

    tracing::info!(
        target: "script_kit::automation",
        window_id = %resolved.id,
        kind = ?resolved.kind,
        element_count = snapshot.elements.len(),
        total_count = snapshot.total_count,
        warning_count = snapshot.warnings.len(),
        focused_semantic_id = ?snapshot.focused_semantic_id,
        selected_semantic_id = ?snapshot.selected_semantic_id,
        "automation.surface.snapshot_collected"
    );

    Some(snapshot)
}

#[allow(dead_code)] // The binary's owned evaluation calls this; the library has no caller.
pub(crate) fn append_root_layer_elements(
    snapshot: &mut SurfaceElementSnapshot,
    layers: &gpui_component::RootLayerSnapshot,
) {
    for (index, dialog) in layers.dialogs.iter().enumerate() {
        let mut item = element(
            &format!(
                "root-dialog:{}:{}",
                dialog.root_entity_id, dialog.generation
            ),
            ElementType::Panel,
            None,
            Some(dialog.generation.to_string()),
            Some(index + 1 == layers.dialogs.len()),
            None,
            Some(index),
        );
        item.kind = Some("rootDialog".into());
        item.source = Some("gpui-component.Root".into());
        item.status_kind = Some(
            if index + 1 == layers.dialogs.len() {
                "current"
            } else {
                "stacked"
            }
            .into(),
        );
        item.role = Some("dismissDialog".into());
        item.selectable = Some(index + 1 == layers.dialogs.len());
        item.action_disabled = (index + 1 != layers.dialogs.len())
            .then(|| "Only the current dialog can be dismissed".into());
        snapshot.elements.push(item);
    }
    for notification in &layers.notifications {
        let mut item = element(
            &format!("root-notification:{}", notification.entity_id),
            ElementType::Panel,
            notification.title.clone(),
            notification.message.clone(),
            None,
            None,
            None,
        )
        .redact_content(crate::protocol::ElementContentKind::UserContent);
        item.kind = Some("rootNotification".into());
        item.source = Some("gpui-component.Notification".into());
        item.status_kind = Some(
            if notification.closing {
                "closing"
            } else {
                "visible"
            }
            .into(),
        );
        item.role = Some("dismissNotification".into());
        item.selectable = Some(!notification.closing);
        item.action_disabled = notification
            .closing
            .then(|| "Notification is already closing".into());
        snapshot.elements.push(item);
    }
    if !layers.dialogs.is_empty() {
        snapshot.focused_semantic_id = None;
    }
    snapshot.total_count = snapshot.elements.len();
}

#[cfg(test)]
mod root_layer_projection_tests {
    #[test]
    fn projection_retains_dialog_lifetimes_and_notification_closing_state() {
        let layers = gpui_component::RootLayerSnapshot {
            revision: 7,
            dialogs: vec![
                gpui_component::RootDialogId {
                    root_entity_id: 12,
                    generation: 3,
                },
                gpui_component::RootDialogId {
                    root_entity_id: 12,
                    generation: 6,
                },
            ],
            notifications: vec![gpui_component::notification::NotificationLayerSnapshot {
                entity_id: 24,
                closing: true,
                title: Some("Saved".into()),
                message: None,
            }],
            notifications_expanded: false,
        };
        let mut snapshot = super::SurfaceElementSnapshot {
            focused_semantic_id: Some("input:notes-editor".into()),
            ..Default::default()
        };
        super::append_root_layer_elements(&mut snapshot, &layers);
        assert_eq!(snapshot.elements[0].semantic_id, "root-dialog:12:3");
        assert_eq!(snapshot.elements[0].status_kind.as_deref(), Some("stacked"));
        assert_eq!(snapshot.elements[1].semantic_id, "root-dialog:12:6");
        assert_eq!(snapshot.elements[1].status_kind.as_deref(), Some("current"));
        assert_eq!(snapshot.elements[2].semantic_id, "root-notification:24");
        assert_eq!(snapshot.elements[2].status_kind.as_deref(), Some("closing"));
        assert_eq!(snapshot.elements[0].selectable, Some(false));
        assert_eq!(snapshot.elements[1].selectable, Some(true));
        assert_eq!(snapshot.elements[1].role.as_deref(), Some("dismissDialog"));
        assert_eq!(snapshot.elements[2].selectable, Some(false));
        assert_eq!(
            snapshot.elements[2].role.as_deref(),
            Some("dismissNotification")
        );
        assert_eq!(snapshot.total_count, 3);
        assert!(snapshot.focused_semantic_id.is_none());
        assert_eq!(layers.revision, 7);
    }
}
