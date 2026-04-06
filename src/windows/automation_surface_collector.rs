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
        AutomationWindowKind::ActionsDialog => {
            collect_actions_dialog_snapshot(cx).unwrap_or_else(|| {
                panel_only_fallback(
                    "panel:actions-dialog",
                    resolved.title.clone(),
                    "panel_only_actions_dialog",
                )
            })
        }
        AutomationWindowKind::PromptPopup => {
            collect_prompt_popup_snapshot(cx).unwrap_or_else(|| {
                panel_only_fallback(
                    "panel:prompt-popup",
                    resolved.title.clone(),
                    "panel_only_prompt_popup",
                )
            })
        }
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

/// Fallback for surfaces that cannot be introspected.
fn panel_only_fallback(
    panel_id: &str,
    title: Option<String>,
    warning: &str,
) -> SurfaceElementSnapshot {
    SurfaceElementSnapshot {
        elements: vec![element(
            panel_id,
            ElementType::Panel,
            title,
            None,
            None,
            Some(true),
            None,
        )],
        total_count: 1,
        focused_semantic_id: Some(panel_id.to_string()),
        selected_semantic_id: None,
        warnings: vec![warning.to_string()],
    }
}

// ---------------------------------------------------------------------------
// Notes collector
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Detached ACP collector
// ---------------------------------------------------------------------------

fn collect_acp_detached_snapshot(
    _resolved: &AutomationWindowInfo,
    cx: &gpui::App,
) -> Option<SurfaceElementSnapshot> {
    let entity = crate::ai::acp::chat_window::get_detached_acp_view_entity()?;
    Some(collect_acp_detached_elements(&entity, 1000, cx))
}

/// Collect semantic elements from a live detached ACP entity.
///
/// Shared by the surface snapshot path (`getElements`) and the
/// [`DetachedAcpTransactionProvider`](super::automation_transaction_provider::DetachedAcpTransactionProvider)
/// so both see the same semantic model.
pub(crate) fn collect_acp_detached_elements(
    entity: &gpui::Entity<crate::ai::acp::view::AcpChatView>,
    limit: usize,
    cx: &gpui::App,
) -> SurfaceElementSnapshot {
    let state = entity.read(cx).collect_acp_state_snapshot(cx);

    let picker_open = state.picker.as_ref().map(|p| p.open).unwrap_or(false);

    let mut elements = vec![
        element(
            "panel:acp-detached",
            ElementType::Panel,
            None,
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

    if elements.len() > limit {
        elements.truncate(limit);
    }

    SurfaceElementSnapshot {
        total_count: elements.len(),
        elements,
        focused_semantic_id: Some("input:acp-composer".to_string()),
        selected_semantic_id: None,
        warnings: Vec::new(),
    }
}

// ---------------------------------------------------------------------------
// Actions dialog collector
// ---------------------------------------------------------------------------

/// Collect semantic elements from the live ActionsDialog entity.
///
/// Returns `None` if the actions window is not open or its entity cannot be
/// read, causing the caller to fall back to `panel_only_actions_dialog`.
fn collect_actions_dialog_snapshot(cx: &gpui::App) -> Option<SurfaceElementSnapshot> {
    let dialog_entity = crate::actions::get_actions_dialog_entity(cx)?;
    Some(collect_actions_dialog_elements(&dialog_entity, 1000, cx))
}

/// Collect semantic elements from a live ActionsDialog entity.
///
/// Shared by the surface snapshot path (`getElements`) and the
/// [`ActionsDialogTransactionProvider`](super::automation_transaction_provider::ActionsDialogTransactionProvider)
/// so both see the same semantic model.
pub(crate) fn collect_actions_dialog_elements(
    dialog_entity: &gpui::Entity<crate::actions::ActionsDialog>,
    limit: usize,
    cx: &gpui::App,
) -> SurfaceElementSnapshot {
    let dialog = dialog_entity.read(cx);

    let mut elements = Vec::new();

    // Search input
    let search_focused = !dialog.hide_search;
    elements.push(element(
        "input:actions-search",
        ElementType::Input,
        None,
        Some(dialog.search_text.clone()),
        None,
        Some(search_focused),
        None,
    ));

    // List of filtered actions
    let action_count = dialog.filtered_actions.len();
    elements.push(element(
        "list:actions",
        ElementType::List,
        Some(format!("{action_count} actions")),
        None,
        None,
        None,
        None,
    ));

    // Individual action choices
    let selected_action_idx = dialog
        .get_selected_filtered_index()
        .and_then(|fi| dialog.filtered_actions.get(fi).copied());
    let mut selected_semantic_id = None;

    for (filter_pos, &action_idx) in dialog.filtered_actions.iter().enumerate() {
        let Some(action) = dialog.actions.get(action_idx) else {
            continue;
        };
        let is_selected = selected_action_idx == Some(action_idx);
        let semantic_id = format!("choice:{}:{}", filter_pos, action.id);

        if is_selected {
            selected_semantic_id = Some(semantic_id.clone());
        }

        elements.push(element(
            &semantic_id,
            ElementType::Choice,
            Some(action.title.clone()),
            Some(action.id.clone()),
            Some(is_selected),
            None,
            Some(filter_pos),
        ));
    }

    let focused_semantic_id = if search_focused {
        Some("input:actions-search".to_string())
    } else {
        selected_semantic_id.clone()
    };

    if elements.len() > limit {
        elements.truncate(limit);
    }

    SurfaceElementSnapshot {
        total_count: elements.len(),
        elements,
        focused_semantic_id,
        selected_semantic_id,
        warnings: Vec::new(),
    }
}

// ---------------------------------------------------------------------------
// Prompt popup collector (mention picker, model selector, confirm)
// ---------------------------------------------------------------------------

/// Collect semantic elements from a known prompt popup type.
///
/// Tries each known popup kind in order and returns the first match.
/// Returns `None` if no known popup is open, causing the caller to fall
/// back to `panel_only_prompt_popup`.
fn collect_prompt_popup_snapshot(cx: &gpui::App) -> Option<SurfaceElementSnapshot> {
    if let Some(snapshot) = collect_mention_picker_snapshot(cx) {
        return Some(snapshot);
    }
    if let Some(snapshot) = collect_model_selector_snapshot(cx) {
        return Some(snapshot);
    }
    if let Some(snapshot) = collect_confirm_popup_snapshot(cx) {
        return Some(snapshot);
    }
    None
}

fn collect_mention_picker_snapshot(cx: &gpui::App) -> Option<SurfaceElementSnapshot> {
    let snap = crate::ai::acp::picker_popup::get_mention_popup_snapshot(cx)?;

    let mut elements = vec![element(
        "panel:mention-picker",
        ElementType::Panel,
        Some(format!("{:?}", snap.trigger)),
        None,
        None,
        None,
        None,
    )];

    let item_count = snap.items.len();
    elements.push(element(
        "list:mention-items",
        ElementType::List,
        Some(format!("{item_count} items")),
        None,
        None,
        None,
        None,
    ));

    let mut selected_semantic_id = None;
    for (idx, item) in snap.items.iter().enumerate() {
        let is_selected = idx == snap.selected_index;
        let semantic_id = format!("choice:{}:{}", idx, item.id);

        if is_selected {
            selected_semantic_id = Some(semantic_id.clone());
        }

        elements.push(element(
            &semantic_id,
            ElementType::Choice,
            Some(item.label.to_string()),
            Some(item.id.to_string()),
            Some(is_selected),
            None,
            Some(idx),
        ));
    }

    Some(SurfaceElementSnapshot {
        total_count: elements.len(),
        elements,
        focused_semantic_id: selected_semantic_id.clone(),
        selected_semantic_id,
        warnings: Vec::new(),
    })
}

fn collect_model_selector_snapshot(cx: &gpui::App) -> Option<SurfaceElementSnapshot> {
    let snap = crate::ai::acp::model_selector_popup::get_model_selector_popup_snapshot(cx)?;

    let mut elements = vec![element(
        "panel:model-selector",
        ElementType::Panel,
        None,
        None,
        None,
        None,
        None,
    )];

    let entry_count = snap.entries.len();
    elements.push(element(
        "list:model-entries",
        ElementType::List,
        Some(format!("{entry_count} models")),
        None,
        None,
        None,
        None,
    ));

    let mut selected_semantic_id = None;
    for (idx, entry) in snap.entries.iter().enumerate() {
        let is_selected = idx == snap.selected_index;
        let semantic_id = format!("choice:{}:{}", idx, entry.id);

        if is_selected {
            selected_semantic_id = Some(semantic_id.clone());
        }

        elements.push(element(
            &semantic_id,
            ElementType::Choice,
            Some(entry.display.to_string()),
            Some(entry.id.clone()),
            Some(is_selected),
            None,
            Some(idx),
        ));
    }

    Some(SurfaceElementSnapshot {
        total_count: elements.len(),
        elements,
        focused_semantic_id: selected_semantic_id.clone(),
        selected_semantic_id,
        warnings: Vec::new(),
    })
}

fn collect_confirm_popup_snapshot(cx: &gpui::App) -> Option<SurfaceElementSnapshot> {
    let snap = crate::confirm::get_confirm_popup_snapshot(cx)?;

    let confirm_focused = snap.focused_button == "confirm";
    let cancel_focused = snap.focused_button == "cancel";

    let elements = vec![
        element(
            "panel:confirm-dialog",
            ElementType::Panel,
            Some(snap.title),
            Some(snap.body),
            None,
            None,
            None,
        ),
        element(
            "button:0:confirm",
            ElementType::Button,
            Some(snap.confirm_text),
            Some("confirm".to_string()),
            None,
            Some(confirm_focused),
            Some(0),
        ),
        element(
            "button:1:cancel",
            ElementType::Button,
            Some(snap.cancel_text),
            Some("cancel".to_string()),
            None,
            Some(cancel_focused),
            Some(1),
        ),
    ];

    let focused_semantic_id = if confirm_focused {
        "button:0:confirm"
    } else {
        "button:1:cancel"
    };

    Some(SurfaceElementSnapshot {
        total_count: elements.len(),
        elements,
        focused_semantic_id: Some(focused_semantic_id.to_string()),
        selected_semantic_id: None,
        warnings: Vec::new(),
    })
}
