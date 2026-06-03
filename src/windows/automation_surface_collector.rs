//! Secondary-surface semantic element collectors.
//!
//! Provides [`collect_surface_snapshot`] which returns semantic elements for
//! non-main automation windows (Notes, AcpDetached, ActionsDialog, PromptPopup).
//!
//! Used by both `getElements` and `inspectAutomationWindow` so agents see one
//! consistent semantic model regardless of which protocol command they use.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use crate::menu_syntax::TriggerPickerSnapshot;
use crate::protocol::{AutomationWindowInfo, AutomationWindowKind, ElementInfo, ElementType};

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

#[derive(Clone, Debug, Default)]
struct PromptPopupElementSnapshot {
    elements: Vec<ElementInfo>,
    focused_semantic_id: Option<String>,
    selected_semantic_id: Option<String>,
}

fn prompt_popup_semantic_cache() -> &'static Mutex<HashMap<String, PromptPopupElementSnapshot>> {
    static CACHE: OnceLock<Mutex<HashMap<String, PromptPopupElementSnapshot>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn actions_dialog_semantic_cache() -> &'static Mutex<HashMap<String, PromptPopupElementSnapshot>> {
    static CACHE: OnceLock<Mutex<HashMap<String, PromptPopupElementSnapshot>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

#[allow(dead_code)] // Binary app_impl uses this; lib-only builds do not.
pub(crate) fn upsert_actions_dialog_snapshot(
    window_id: &str,
    dialog_entity: &gpui::Entity<crate::actions::ActionsDialog>,
    cx: &gpui::App,
) {
    let snapshot = collect_actions_dialog_elements(dialog_entity, 1000, cx);
    if let Ok(mut cache) = actions_dialog_semantic_cache().lock() {
        cache.insert(
            window_id.to_string(),
            PromptPopupElementSnapshot {
                elements: snapshot.elements,
                focused_semantic_id: snapshot.focused_semantic_id,
                selected_semantic_id: snapshot.selected_semantic_id,
            },
        );
    }
}

#[allow(dead_code)] // Binary app_impl uses this; lib-only builds do not.
pub(crate) fn remove_actions_dialog_snapshot(window_id: &str) {
    if let Ok(mut cache) = actions_dialog_semantic_cache().lock() {
        cache.remove(window_id);
    }
}

#[allow(dead_code)] // Binary app_impl uses this; lib-only builds do not.
pub(crate) fn upsert_menu_syntax_prompt_popup_snapshot(
    window_id: &str,
    snapshot: &TriggerPickerSnapshot,
    selected_row_id: Option<&str>,
) {
    let mut elements = Vec::new();
    elements.push(element(
        "panel:menu-syntax-trigger-popup",
        ElementType::Panel,
        Some("Menu Syntax".to_string()),
        None,
        None,
        None,
        None,
    ));
    elements.push(element(
        "list:menu-syntax-trigger-popup",
        ElementType::List,
        Some(format!("{} rows", snapshot.rows.len())),
        None,
        None,
        None,
        None,
    ));

    let mut selected_semantic_id = None;
    for (idx, row) in snapshot.rows.iter().enumerate() {
        let is_selected = selected_row_id == Some(row.id.as_str());
        let semantic_id = format!("choice:{}:{}", idx, row.id);
        if is_selected {
            selected_semantic_id = Some(semantic_id.clone());
        }

        let mut info = element(
            &semantic_id,
            ElementType::Choice,
            Some(row.title.clone()),
            row.token.clone(),
            Some(is_selected),
            None,
            Some(idx),
        );
        info.role = row.subtitle.clone();
        info.kind = Some(format!("{:?}", row.kind));
        info.source_name = row.detail.clone().or_else(|| row.example.clone());
        info.selectable = Some(row.enabled);
        elements.push(info);
    }

    let focused_semantic_id = selected_semantic_id.clone();
    if let Ok(mut cache) = prompt_popup_semantic_cache().lock() {
        cache.insert(
            window_id.to_string(),
            PromptPopupElementSnapshot {
                elements,
                focused_semantic_id,
                selected_semantic_id,
            },
        );
    }
}

#[allow(dead_code)] // Binary app_impl uses this; lib-only builds do not.
pub(crate) fn remove_menu_syntax_prompt_popup_snapshot(window_id: &str) {
    if let Ok(mut cache) = prompt_popup_semantic_cache().lock() {
        cache.remove(window_id);
    }
}

#[allow(dead_code)] // Binary dictation overlay uses this; lib-only builds do not.
pub(crate) fn upsert_dictation_microphone_prompt_popup_snapshot(
    window_id: &str,
    snapshot: &crate::dictation::DictationMicrophonePopupSnapshot,
) {
    let mut elements = Vec::new();
    elements.push(element(
        "panel:dictation-microphone-popup",
        ElementType::Panel,
        Some("Dictation Microphones".to_string()),
        None,
        None,
        None,
        None,
    ));
    elements.push(element(
        "list:dictation-microphones",
        ElementType::List,
        Some(format!("{} rows", snapshot.rows.len())),
        None,
        None,
        None,
        None,
    ));

    let mut selected_semantic_id = None;
    for (idx, row) in snapshot.rows.iter().enumerate() {
        let is_selected = snapshot.selected_row_id.as_deref() == Some(row.row_id.as_str());
        if is_selected {
            selected_semantic_id = Some(row.semantic_id.clone());
        }

        let mut info = element(
            &row.semantic_id,
            ElementType::Choice,
            Some(row.title.clone()),
            Some(row.row_id.clone()),
            Some(is_selected),
            None,
            Some(idx),
        );
        info.role = Some(row.subtitle.clone());
        info.kind = Some("DictationMicrophone".to_string());
        info.selectable = Some(true);
        elements.push(info);
    }

    let focused_semantic_id = selected_semantic_id.clone();
    if let Ok(mut cache) = prompt_popup_semantic_cache().lock() {
        cache.insert(
            window_id.to_string(),
            PromptPopupElementSnapshot {
                elements,
                focused_semantic_id,
                selected_semantic_id,
            },
        );
    }
}

#[allow(dead_code)] // Binary dictation overlay uses this; lib-only builds do not.
pub(crate) fn remove_dictation_microphone_prompt_popup_snapshot(window_id: &str) {
    if let Ok(mut cache) = prompt_popup_semantic_cache().lock() {
        cache.remove(window_id);
    }
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
        AutomationWindowKind::Notes => collect_notes_snapshot(resolved, cx).unwrap_or_else(|| {
            panel_only_fallback(
                "panel:notes-window",
                resolved.title.clone(),
                "panel_only_notes",
            )
        }),
        AutomationWindowKind::AcpDetached => collect_acp_detached_snapshot(resolved, cx)
            .unwrap_or_else(|| {
                panel_only_fallback(
                    "panel:acp-detached",
                    resolved.title.clone(),
                    "panel_only_acp_detached",
                )
            }),
        AutomationWindowKind::ActionsDialog => collect_actions_dialog_snapshot(cx)
            .or_else(|| collect_cached_actions_dialog_snapshot(&resolved.id))
            .unwrap_or_else(|| {
                panel_only_fallback(
                    "panel:actions-dialog",
                    resolved.title.clone(),
                    "panel_only_actions_dialog",
                )
            }),
        AutomationWindowKind::PromptPopup => collect_cached_prompt_popup_snapshot(&resolved.id)
            .or_else(|| collect_prompt_popup_snapshot(cx))
            .unwrap_or_else(|| {
                panel_only_fallback(
                    "panel:prompt-popup",
                    resolved.title.clone(),
                    "panel_only_prompt_popup",
                )
            }),
        AutomationWindowKind::Dictation => collect_dictation_snapshot(resolved),
        AutomationWindowKind::MiniAi
            if resolved.id == crate::inline_agent::window::INLINE_AGENT_WINDOW_AUTOMATION_ID =>
        {
            collect_inline_agent_snapshot(resolved)
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

fn collect_dictation_snapshot(resolved: &AutomationWindowInfo) -> SurfaceElementSnapshot {
    let state = crate::dictation::snapshot_overlay_state().unwrap_or_default();
    let phase = format!("{:?}", state.phase);
    let target = state.target.overlay_label().to_string();

    let mut panel = element(
        "panel:dictation-overlay",
        ElementType::Panel,
        resolved
            .title
            .clone()
            .or_else(|| Some("Dictation".to_string())),
        None,
        None,
        Some(resolved.focused),
        None,
    );
    panel.kind = Some("overlay".to_string());
    panel.status_kind = Some(phase.clone());

    let mut signal = element(
        "panel:dictation-signal-band",
        ElementType::Panel,
        Some(phase),
        None,
        None,
        None,
        None,
    );
    signal.kind = Some("signal".to_string());

    let mut target_badge = element(
        "button:dictation-target",
        ElementType::Button,
        Some(target),
        None,
        None,
        None,
        Some(0),
    );
    target_badge.role = Some("target".to_string());
    target_badge.selectable = Some(crate::dictation::can_cycle_dictation_target());

    SurfaceElementSnapshot {
        elements: vec![panel, signal, target_badge],
        total_count: 3,
        focused_semantic_id: Some("panel:dictation-overlay".to_string()),
        selected_semantic_id: None,
        warnings: Vec::new(),
        quality: SnapshotQuality::Full,
    }
}

fn collect_inline_agent_snapshot(resolved: &AutomationWindowInfo) -> SurfaceElementSnapshot {
    use crate::inline_agent::automation::{
        INLINE_AGENT_ACTION_APPEND_ID, INLINE_AGENT_ACTION_CHAT_ID, INLINE_AGENT_ACTION_COPY_ID,
        INLINE_AGENT_ACTION_REPLACE_ID, INLINE_AGENT_ACTION_RETRY_ID, INLINE_AGENT_ACTION_STOP_ID,
        INLINE_AGENT_APP_BADGE_ID, INLINE_AGENT_COLLAPSE_ID, INLINE_AGENT_COMPACT_ID,
        INLINE_AGENT_EXPANDED_COMPOSER_ID, INLINE_AGENT_EXPANDED_ID, INLINE_AGENT_HEADER_ID,
        INLINE_AGENT_INPUT_ID, INLINE_AGENT_METRICS_ID, INLINE_AGENT_OUTPUT_PREVIEW_ID,
        INLINE_AGENT_THINKING_BAR_ID, INLINE_AGENT_THINKING_LABEL_ID, INLINE_AGENT_TURN_LIST_ID,
    };
    use crate::inline_agent::{InlineAgentMode, InlineAgentRunState};

    let plan = crate::inline_agent::window::inline_agent_current_window_snapshot();
    let mode = plan
        .as_ref()
        .map(|plan| plan.mode)
        .unwrap_or(InlineAgentMode::Compact);
    let run_state = plan
        .as_ref()
        .map(|plan| plan.run_state.clone())
        .unwrap_or(InlineAgentRunState::Idle);

    let mut elements = Vec::new();
    let root_id = match mode {
        InlineAgentMode::Compact => INLINE_AGENT_COMPACT_ID,
        InlineAgentMode::Expanded => INLINE_AGENT_EXPANDED_ID,
    };
    let mut root = element(
        root_id,
        ElementType::Panel,
        resolved.title.clone(),
        None,
        None,
        None,
        None,
    );
    root.kind = Some(
        match mode {
            InlineAgentMode::Compact => "compact",
            InlineAgentMode::Expanded => "expanded",
        }
        .to_string(),
    );
    root.status_kind = Some(inline_agent_run_state_kind(&run_state).to_string());
    elements.push(root);

    elements.push(element(
        INLINE_AGENT_HEADER_ID,
        ElementType::Panel,
        Some("Inline Agent header".to_string()),
        None,
        None,
        None,
        None,
    ));
    elements.push(element(
        INLINE_AGENT_APP_BADGE_ID,
        ElementType::Panel,
        Some("Source app".to_string()),
        None,
        None,
        None,
        None,
    ));
    elements.push(element(
        INLINE_AGENT_METRICS_ID,
        ElementType::Panel,
        Some("Captured text metrics".to_string()),
        None,
        None,
        None,
        None,
    ));
    elements.push(element(
        match mode {
            InlineAgentMode::Compact => INLINE_AGENT_INPUT_ID,
            InlineAgentMode::Expanded => INLINE_AGENT_EXPANDED_COMPOSER_ID,
        },
        ElementType::Input,
        Some(crate::inline_agent::types::INLINE_AGENT_INPUT_PLACEHOLDER.to_string()),
        None,
        None,
        Some(true),
        None,
    ));

    if matches!(
        run_state,
        InlineAgentRunState::Thinking { .. } | InlineAgentRunState::Streaming { .. }
    ) {
        elements.push(element(
            INLINE_AGENT_THINKING_BAR_ID,
            ElementType::Panel,
            Some("Thinking".to_string()),
            None,
            None,
            None,
            None,
        ));
        elements.push(element(
            INLINE_AGENT_THINKING_LABEL_ID,
            ElementType::Panel,
            Some("Thinking...".to_string()),
            None,
            None,
            None,
            None,
        ));
    }

    if inline_agent_has_output_preview(&run_state) {
        elements.push(element(
            INLINE_AGENT_OUTPUT_PREVIEW_ID,
            ElementType::Panel,
            Some("Output preview".to_string()),
            None,
            None,
            None,
            None,
        ));
    }

    let button_specs = [
        (INLINE_AGENT_ACTION_REPLACE_ID, "Replace"),
        (INLINE_AGENT_ACTION_APPEND_ID, "Append"),
        (INLINE_AGENT_ACTION_COPY_ID, "Copy"),
        (INLINE_AGENT_ACTION_CHAT_ID, "Chat"),
        (INLINE_AGENT_ACTION_STOP_ID, "Stop"),
        (INLINE_AGENT_ACTION_RETRY_ID, "Retry"),
    ];
    for (index, (id, label)) in button_specs.into_iter().enumerate() {
        let mut info = element(
            id,
            ElementType::Button,
            Some(label.to_string()),
            None,
            None,
            None,
            Some(index),
        );
        info.action_disabled =
            inline_agent_action_disabled_reason(id, mode, &run_state).map(str::to_string);
        elements.push(info);
    }

    if mode == InlineAgentMode::Expanded {
        elements.push(element(
            INLINE_AGENT_TURN_LIST_ID,
            ElementType::List,
            Some("Inline Agent turns".to_string()),
            None,
            None,
            None,
            None,
        ));
        elements.push(element(
            INLINE_AGENT_COLLAPSE_ID,
            ElementType::Button,
            Some("Collapse".to_string()),
            None,
            None,
            None,
            None,
        ));
    }

    SurfaceElementSnapshot {
        total_count: elements.len(),
        elements,
        focused_semantic_id: Some(
            match mode {
                InlineAgentMode::Compact => INLINE_AGENT_INPUT_ID,
                InlineAgentMode::Expanded => INLINE_AGENT_EXPANDED_COMPOSER_ID,
            }
            .to_string(),
        ),
        selected_semantic_id: None,
        warnings: Vec::new(),
        quality: SnapshotQuality::Full,
    }
}

fn inline_agent_run_state_kind(state: &crate::inline_agent::InlineAgentRunState) -> &'static str {
    match state {
        crate::inline_agent::InlineAgentRunState::Idle => "idle",
        crate::inline_agent::InlineAgentRunState::Thinking { .. } => "thinking",
        crate::inline_agent::InlineAgentRunState::Streaming { .. } => "streaming",
        crate::inline_agent::InlineAgentRunState::Completed { .. } => "completed",
        crate::inline_agent::InlineAgentRunState::Error { .. } => "error",
        crate::inline_agent::InlineAgentRunState::Applying { .. } => "applying",
        crate::inline_agent::InlineAgentRunState::Applied { .. } => "applied",
    }
}

fn inline_agent_has_output_preview(state: &crate::inline_agent::InlineAgentRunState) -> bool {
    match state {
        crate::inline_agent::InlineAgentRunState::Streaming { partial_output, .. } => {
            !partial_output.is_empty()
        }
        _ => state.latest_complete_output().is_some(),
    }
}

fn inline_agent_action_disabled_reason(
    id: &str,
    mode: crate::inline_agent::InlineAgentMode,
    state: &crate::inline_agent::InlineAgentRunState,
) -> Option<&'static str> {
    use crate::inline_agent::automation::{
        INLINE_AGENT_ACTION_APPEND_ID, INLINE_AGENT_ACTION_CHAT_ID, INLINE_AGENT_ACTION_COPY_ID,
        INLINE_AGENT_ACTION_REPLACE_ID, INLINE_AGENT_ACTION_RETRY_ID, INLINE_AGENT_ACTION_STOP_ID,
    };
    let active = matches!(
        state,
        crate::inline_agent::InlineAgentRunState::Thinking { .. }
            | crate::inline_agent::InlineAgentRunState::Streaming { .. }
    );
    let retryable = matches!(
        state,
        crate::inline_agent::InlineAgentRunState::Error {
            retryable: true,
            ..
        }
    );
    let has_output = state.latest_complete_output().is_some();

    match id {
        INLINE_AGENT_ACTION_STOP_ID if !active => Some("inactive"),
        INLINE_AGENT_ACTION_RETRY_ID if !retryable => Some("not-retryable"),
        INLINE_AGENT_ACTION_REPLACE_ID
        | INLINE_AGENT_ACTION_APPEND_ID
        | INLINE_AGENT_ACTION_COPY_ID
            if active =>
        {
            Some("active-turn")
        }
        INLINE_AGENT_ACTION_REPLACE_ID
        | INLINE_AGENT_ACTION_APPEND_ID
        | INLINE_AGENT_ACTION_COPY_ID
            if !has_output =>
        {
            Some("no-output")
        }
        INLINE_AGENT_ACTION_CHAT_ID if mode == crate::inline_agent::InlineAgentMode::Expanded => {
            Some("already-expanded")
        }
        _ => None,
    }
}

/// Fallback for surfaces that cannot be fully introspected.
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
        quality: SnapshotQuality::PanelOnly,
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
        quality: SnapshotQuality::Full,
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
        quality: SnapshotQuality::Full,
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

fn collect_cached_actions_dialog_snapshot(window_id: &str) -> Option<SurfaceElementSnapshot> {
    let cached = actions_dialog_semantic_cache()
        .lock()
        .ok()
        .and_then(|cache| cache.get(window_id).cloned())?;
    Some(SurfaceElementSnapshot {
        total_count: cached.elements.len(),
        elements: cached.elements,
        focused_semantic_id: cached.focused_semantic_id,
        selected_semantic_id: cached.selected_semantic_id,
        warnings: Vec::new(),
        quality: SnapshotQuality::Full,
    })
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

    // Individual action choices. Use grouped visual indexes so semantic ids
    // match rowGeometry, which also includes section headers.
    let mut selected_semantic_id = None;

    for (visual_index, grouped_item) in dialog.grouped_items.iter().enumerate() {
        match grouped_item {
            crate::actions::GroupedActionItem::SectionHeader(label) => {
                elements.push(element(
                    &format!("section:{visual_index}"),
                    ElementType::Panel,
                    Some(label.clone()),
                    None,
                    Some(dialog.selected_index == visual_index),
                    None,
                    Some(visual_index),
                ));
            }
            crate::actions::GroupedActionItem::Item(filter_idx) => {
                let Some(&action_idx) = dialog.filtered_actions.get(*filter_idx) else {
                    continue;
                };
                let Some(action) = dialog.actions.get(action_idx) else {
                    continue;
                };
                let is_selected = dialog.selected_index == visual_index;
                let semantic_id = format!("choice:{visual_index}:{}", action.id);

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
                    Some(visual_index),
                ));
            }
        }
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
        quality: SnapshotQuality::Full,
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
    if let Some(snapshot) = collect_history_popup_snapshot(cx) {
        return Some(snapshot);
    }
    if let Some(snapshot) = collect_confirm_popup_snapshot(cx) {
        return Some(snapshot);
    }
    None
}

fn collect_cached_prompt_popup_snapshot(window_id: &str) -> Option<SurfaceElementSnapshot> {
    let cached = prompt_popup_semantic_cache()
        .lock()
        .ok()?
        .get(window_id)
        .cloned()?;
    Some(SurfaceElementSnapshot {
        total_count: cached.elements.len(),
        elements: cached.elements,
        focused_semantic_id: cached.focused_semantic_id,
        selected_semantic_id: cached.selected_semantic_id,
        warnings: Vec::new(),
        quality: SnapshotQuality::Full,
    })
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
        quality: SnapshotQuality::Full,
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
        quality: SnapshotQuality::Full,
    })
}

fn collect_history_popup_snapshot(cx: &gpui::App) -> Option<SurfaceElementSnapshot> {
    let snap = crate::ai::acp::history_popup::get_history_popup_snapshot(cx)?;

    let mut elements = vec![element(
        "panel:history-popup",
        ElementType::Panel,
        Some(snap.title.to_string()),
        Some(snap.query.to_string()),
        None,
        None,
        None,
    )];

    let entry_count = snap.entries.len();
    elements.push(element(
        "list:history-entries",
        ElementType::List,
        Some(format!("{entry_count} sessions")),
        None,
        None,
        None,
        None,
    ));

    let mut selected_semantic_id = None;
    for (idx, entry) in snap.entries.iter().enumerate() {
        let is_selected = idx == snap.selected_index;
        let semantic_id = format!("choice:{}:{}", idx, entry.hit.entry.session_id);

        if is_selected {
            selected_semantic_id = Some(semantic_id.clone());
        }

        elements.push(element(
            &semantic_id,
            ElementType::Choice,
            Some(entry.title.to_string()),
            Some(entry.hit.entry.session_id.clone()),
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
        quality: SnapshotQuality::Full,
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
        quality: SnapshotQuality::Full,
    })
}
