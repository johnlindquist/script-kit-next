//! Secondary-surface semantic element collectors.
//!
//! Provides [`collect_surface_snapshot`] which returns semantic elements for
//! non-main automation windows (Notes, AcpDetached, ActionsDialog, PromptPopup).
//!
//! Used by both `getElements` and `inspectAutomationWindow` so agents see one
//! consistent semantic model regardless of which protocol command they use.

use crate::protocol::{AutomationWindowInfo, AutomationWindowKind, ElementInfo, ElementType};

/// Lightweight snapshot of semantic elements from a non-main surface.
#[derive(Clone, Debug, Default)]
pub struct SurfaceElementSnapshot {
    pub elements: Vec<ElementInfo>,
    pub total_count: usize,
    pub focused_semantic_id: Option<String>,
    pub selected_semantic_id: Option<String>,
    pub warnings: Vec<String>,
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
        selected,
        focused,
        index,
    }
}

/// Collect semantic elements for a resolved non-main automation window.
///
/// Returns `None` for window kinds that do not yet have a collector.
pub fn collect_surface_snapshot(
    resolved: &AutomationWindowInfo,
    limit: usize,
    cx: &gpui::App,
) -> Option<SurfaceElementSnapshot> {
    let mut snapshot = match resolved.kind {
        AutomationWindowKind::Notes => collect_notes_snapshot(resolved, cx)?,
        AutomationWindowKind::AcpDetached => collect_acp_detached_snapshot(resolved, cx)?,
        AutomationWindowKind::ActionsDialog => Some(SurfaceElementSnapshot {
            elements: vec![element(
                "panel:actions-dialog",
                ElementType::Panel,
                resolved.title.clone(),
                None,
                None,
                Some(true),
                None,
            )],
            total_count: 1,
            focused_semantic_id: Some("panel:actions-dialog".to_string()),
            selected_semantic_id: None,
            warnings: vec!["panel_only_actions_dialog".to_string()],
        })?,
        AutomationWindowKind::PromptPopup => Some(SurfaceElementSnapshot {
            elements: vec![element(
                "panel:prompt-popup",
                ElementType::Panel,
                resolved.title.clone(),
                None,
                None,
                Some(true),
                None,
            )],
            total_count: 1,
            focused_semantic_id: Some("panel:prompt-popup".to_string()),
            selected_semantic_id: None,
            warnings: vec!["panel_only_prompt_popup".to_string()],
        })?,
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
        "automation.surface.snapshot_collected"
    );

    Some(snapshot)
}

fn collect_notes_snapshot(
    resolved: &AutomationWindowInfo,
    cx: &gpui::App,
) -> Option<SurfaceElementSnapshot> {
    let text = crate::notes::get_notes_editor_text(cx)?;

    Some(SurfaceElementSnapshot {
        elements: vec![
            element(
                "panel:notes-window",
                ElementType::Panel,
                resolved.title.clone(),
                None,
                None,
                None,
                None,
            ),
            element(
                "input:notes-editor",
                ElementType::Input,
                None,
                Some(text),
                None,
                Some(true),
                None,
            ),
        ],
        total_count: 2,
        focused_semantic_id: Some("input:notes-editor".to_string()),
        selected_semantic_id: None,
        warnings: Vec::new(),
    })
}

fn collect_acp_detached_snapshot(
    resolved: &AutomationWindowInfo,
    cx: &gpui::App,
) -> Option<SurfaceElementSnapshot> {
    let entity = crate::ai::acp::chat_window::get_detached_acp_view_entity()?;
    let state = entity.read(cx).collect_acp_state_snapshot(cx);

    let picker_open = state.picker.as_ref().map(|p| p.open).unwrap_or(false);

    let mut elements = vec![
        element(
            "panel:acp-detached",
            ElementType::Panel,
            resolved.title.clone(),
            None,
            None,
            None,
            None,
        ),
        element(
            "input:acp-composer",
            ElementType::Input,
            None,
            Some(state.input_text.clone()),
            None,
            Some(true),
            None,
        ),
        element(
            "list:acp-messages",
            ElementType::List,
            Some(format!("{} messages", state.message_count)),
            None,
            None,
            None,
            None,
        ),
    ];

    if picker_open {
        elements.push(element(
            "panel:acp-picker",
            ElementType::Panel,
            Some("open".to_string()),
            None,
            None,
            None,
            None,
        ));
    }

    Some(SurfaceElementSnapshot {
        total_count: elements.len(),
        elements,
        focused_semantic_id: Some("input:acp-composer".to_string()),
        selected_semantic_id: None,
        warnings: Vec::new(),
    })
}
