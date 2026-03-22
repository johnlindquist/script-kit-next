use crate::main_window_preflight::types::{
    MainWindowPreflightAction, MainWindowPreflightActionKind, MainWindowPreflightContextItem,
    MainWindowPreflightReceipt,
};
use crate::{AppView, GroupedListItem};

fn selected_result(app: &crate::ScriptListApp) -> Option<crate::scripts::SearchResult> {
    let grouped_items = &app.cached_grouped_items;
    let flat_results = &app.cached_grouped_flat_results;

    let mut ix = app.selected_index;
    while let Some(item) = grouped_items.get(ix) {
        match item {
            GroupedListItem::Item(result_ix) => return flat_results.get(*result_ix).cloned(),
            GroupedListItem::SectionHeader(..) => ix += 1,
        }
    }

    None
}

fn enter_action_kind(result: &crate::scripts::SearchResult) -> MainWindowPreflightActionKind {
    match result {
        crate::scripts::SearchResult::Script(_) => MainWindowPreflightActionKind::RunScript,
        crate::scripts::SearchResult::Scriptlet(_) => MainWindowPreflightActionKind::RunSnippet,
        crate::scripts::SearchResult::BuiltIn(_) => MainWindowPreflightActionKind::RunCommand,
        crate::scripts::SearchResult::App(_) => MainWindowPreflightActionKind::LaunchApp,
        crate::scripts::SearchResult::Window(_) => MainWindowPreflightActionKind::SwitchWindow,
        crate::scripts::SearchResult::Agent(_) => MainWindowPreflightActionKind::RunAgent,
        crate::scripts::SearchResult::Fallback(_) => MainWindowPreflightActionKind::RunFallback,
    }
}

fn build_tab_action(app: &crate::ScriptListApp) -> Option<MainWindowPreflightAction> {
    if app.filter_text.trim().is_empty() {
        return None;
    }

    Some(MainWindowPreflightAction {
        kind: MainWindowPreflightActionKind::AskAi,
        label: "Ask AI".to_string(),
        subject: app.filter_text.clone(),
        type_label: "AI".to_string(),
        source_name: None,
        description: Some(
            "Opens the AI window with the current query for review before submit.".to_string(),
        ),
    })
}

fn build_context_items() -> Vec<MainWindowPreflightContextItem> {
    use crate::ai::context_contract::ContextAttachmentKind;

    [
        ContextAttachmentKind::Current,
        ContextAttachmentKind::Selection,
        ContextAttachmentKind::Browser,
        ContextAttachmentKind::Window,
    ]
    .into_iter()
    .map(|kind| {
        let spec = kind.spec();
        MainWindowPreflightContextItem {
            label: spec.label.to_string(),
            source: spec.uri.to_string(),
            enabled: false,
        }
    })
    .collect()
}

pub(crate) fn build_main_window_preflight_receipt(
    app: &crate::ScriptListApp,
) -> Option<MainWindowPreflightReceipt> {
    if !matches!(app.current_view, AppView::ScriptList) {
        return None;
    }

    let result = selected_result(app)?;
    let mut warnings = Vec::new();

    if matches!(&result, crate::scripts::SearchResult::Agent(_)) {
        warnings.push(
            "Agent execution is not fully implemented in execute_selected yet.".to_string(),
        );
    }

    if app.filter_text.trim().is_empty() {
        warnings.push("Tab-to-AI is inactive until the filter has text.".to_string());
    }

    let enter_action = MainWindowPreflightAction {
        kind: enter_action_kind(&result),
        label: result.get_default_action_text().to_string(),
        subject: result.name().to_string(),
        type_label: result.type_label().to_string(),
        source_name: result.source_name().map(ToString::to_string),
        description: result.description().map(ToString::to_string),
    };

    Some(MainWindowPreflightReceipt {
        filter_text: app.filter_text.clone(),
        selected_index: app.selected_index,
        enter_action,
        tab_action: build_tab_action(app),
        context_items: build_context_items(),
        warnings,
    })
}

pub(crate) fn log_main_window_preflight_receipt(receipt: &MainWindowPreflightReceipt) {
    tracing::info!(
        event = "main_window_preflight_receipt",
        selected_index = receipt.selected_index,
        enter_label = %receipt.enter_action.label,
        enter_subject = %receipt.enter_action.subject,
        enter_type = %receipt.enter_action.type_label,
        tab_enabled = receipt.tab_action.is_some(),
        enabled_context_count = receipt
            .context_items
            .iter()
            .filter(|item| item.enabled)
            .count(),
        warnings = ?receipt.warnings,
        "Built main window preflight receipt"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_context_items_exposes_default_main_window_contract_sources() {
        let items = build_context_items();

        assert_eq!(items.len(), 4, "main-window rail should expose 4 context items");
        assert_eq!(items[0].label, "Current Context");
        assert_eq!(items[0].source, "kit://context?profile=minimal");
        assert_eq!(items[1].label, "Selection");
        assert_eq!(
            items[1].source,
            "kit://context?selectedText=1&frontmostApp=0&menuBar=0&browserUrl=0&focusedWindow=0"
        );
        assert_eq!(items[2].label, "Browser URL");
        assert_eq!(
            items[2].source,
            "kit://context?selectedText=0&frontmostApp=0&menuBar=0&browserUrl=1&focusedWindow=0"
        );
        assert_eq!(items[3].label, "Focused Window");
        assert_eq!(
            items[3].source,
            "kit://context?selectedText=0&frontmostApp=1&menuBar=0&browserUrl=0&focusedWindow=1"
        );
        assert!(
            items.iter().all(|item| !item.enabled),
            "contract rail should not claim context is attached when the main window has no toggle state"
        );
    }
}
