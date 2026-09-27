use super::AppView;

/// Convert an `AppView` variant to the prompt-type string that the canonical
/// source-type detector in `crate::ai` understands.
pub(super) fn app_view_to_prompt_type_str(view: &AppView) -> &'static str {
    match view {
        AppView::ScriptList => "ScriptList",
        AppView::ClipboardHistoryView { .. } => "ClipboardHistory",
        AppView::ArgPrompt { .. } => "ArgPrompt",
        AppView::MiniPrompt { .. } => "MiniPrompt",
        AppView::MicroPrompt { .. } => "MicroPrompt",
        AppView::DivPrompt { .. } => "DivPrompt",
        AppView::FormPrompt { .. } => "FormPrompt",
        AppView::EditorPrompt { .. } => "EditorPrompt",
        AppView::SelectPrompt { .. } => "SelectPrompt",
        AppView::PathPrompt { .. } => "PathPrompt",
        AppView::DropPrompt { .. } => "DropPrompt",
        AppView::TemplatePrompt { .. } => "TemplatePrompt",
        AppView::TermPrompt { .. } => "TermPrompt",
        AppView::EnvPrompt { .. } => "EnvPrompt",
        AppView::ChatPrompt { .. } => "ChatPrompt",
        AppView::NamingPrompt { .. } => "NamingPrompt",
        _ => "Other",
    }
}

/// Early source type detection using only the view and UI snapshot: no
/// desktop context required.
pub(super) fn detect_tab_ai_source_type_early(
    source_view: &AppView,
    ui: &crate::ai::TabAiUiSnapshot,
) -> Option<crate::ai::TabAiSourceType> {
    let prompt_type = app_view_to_prompt_type_str(source_view);
    match prompt_type {
        "ScriptList" if ui.focused_semantic_id.is_some() || ui.selected_semantic_id.is_some() => {
            Some(crate::ai::TabAiSourceType::ScriptListItem)
        }
        "ClipboardHistory" => Some(crate::ai::TabAiSourceType::ClipboardEntry),
        "ArgPrompt" | "MiniPrompt" | "MicroPrompt" | "DivPrompt" | "FormPrompt"
        | "EditorPrompt" | "SelectPrompt" | "PathPrompt" | "DropPrompt" | "TemplatePrompt"
        | "TermPrompt" | "EnvPrompt" | "ChatPrompt" | "NamingPrompt" => {
            Some(crate::ai::TabAiSourceType::RunningCommand)
        }
        // Desktop / DesktopSelection require the deferred capture's selected_text.
        _ => None,
    }
}

/// Detect source type by delegating to the canonical mapping in
/// `crate::ai::detect_tab_ai_source_type_from_prompt`.
pub(super) fn detect_tab_ai_source_type(
    source_view: &AppView,
    desktop: &crate::context_snapshot::AiContextSnapshot,
    focused_target: Option<&crate::ai::TabAiTargetContext>,
) -> Option<crate::ai::TabAiSourceType> {
    crate::ai::detect_tab_ai_source_type_from_prompt(
        app_view_to_prompt_type_str(source_view),
        desktop,
        focused_target,
    )
}

/// Build an apply-back hint from the detected source type.
pub(super) fn build_tab_ai_apply_back_hint(
    source_type: Option<&crate::ai::TabAiSourceType>,
) -> Option<crate::ai::TabAiApplyBackHint> {
    crate::ai::build_tab_ai_apply_back_hint_from_source(source_type)
}
