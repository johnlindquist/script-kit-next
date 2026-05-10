use crate::main_window_preflight::types::{
    MainWindowPreflightAction, MainWindowPreflightActionKind, MainWindowPreflightReceipt,
    RootPassiveFrameReceipt, RootPassiveSourceReceipt,
};
use crate::AppView;

fn selected_result(app: &crate::ScriptListApp) -> Option<crate::scripts::SearchResult> {
    app.main_menu_result_caches
        .cloned_first_search_result_at_or_after_grouped_item(app.selected_index)
}

fn visible_result_keys(app: &crate::ScriptListApp) -> Vec<String> {
    app.main_menu_result_caches
        .grouped_search_results()
        .filter_map(|result| result.stable_selection_key())
        .collect()
}

fn enter_action_kind(result: &crate::scripts::SearchResult) -> MainWindowPreflightActionKind {
    match result {
        crate::scripts::SearchResult::Script(_) => MainWindowPreflightActionKind::RunScript,
        crate::scripts::SearchResult::Scriptlet(_) => MainWindowPreflightActionKind::RunSnippet,
        crate::scripts::SearchResult::BuiltIn(_) => MainWindowPreflightActionKind::RunCommand,
        crate::scripts::SearchResult::App(_) => MainWindowPreflightActionKind::LaunchApp,
        crate::scripts::SearchResult::Window(_) => MainWindowPreflightActionKind::SwitchWindow,
        crate::scripts::SearchResult::File(_) => MainWindowPreflightActionKind::OpenFile,
        crate::scripts::SearchResult::Note(_) => MainWindowPreflightActionKind::RunCommand,
        crate::scripts::SearchResult::AcpHistory(_) => MainWindowPreflightActionKind::RunCommand,
        crate::scripts::SearchResult::ClipboardHistory(_) => {
            MainWindowPreflightActionKind::RunCommand
        }
        crate::scripts::SearchResult::DictationHistory(_) => {
            MainWindowPreflightActionKind::RunCommand
        }
        crate::scripts::SearchResult::BrowserTab(_) => MainWindowPreflightActionKind::SwitchWindow,
        crate::scripts::SearchResult::BrowserHistory(_) => {
            MainWindowPreflightActionKind::RunCommand
        }
        crate::scripts::SearchResult::Agent(_) => MainWindowPreflightActionKind::RunAgent,
        crate::scripts::SearchResult::Skill(_) => MainWindowPreflightActionKind::OpenSkill,
        crate::scripts::SearchResult::Fallback(_) => MainWindowPreflightActionKind::RunFallback,
        crate::scripts::SearchResult::ScriptIssue(_) => {
            MainWindowPreflightActionKind::InspectIssues
        }
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

fn build_root_passive_frame_receipt(app: &crate::ScriptListApp) -> Option<RootPassiveFrameReceipt> {
    let frame = app.root_passive_frame.as_ref()?;
    let browser_tabs_status = crate::browser_tabs::root_browser_tabs_snapshot_status();
    let browser_history_status = crate::browser_history::root_browser_history_snapshot_status();

    Some(RootPassiveFrameReceipt {
        query: frame.key.query.clone(),
        browser_tabs: RootPassiveSourceReceipt {
            enabled: frame.key.browser_tabs_options.enabled,
            frame_count: frame.browser_tab_hits.len(),
            cache_generation: browser_tabs_status.generation,
            frame_generation: frame.browser_tabs_snapshot_generation,
            refreshing: browser_tabs_status.refreshing,
        },
        browser_history: RootPassiveSourceReceipt {
            enabled: frame.key.browser_history_options.enabled,
            frame_count: frame.browser_history_hits.len(),
            cache_generation: browser_history_status.generation,
            frame_generation: frame.browser_history_snapshot_generation,
            refreshing: browser_history_status.refreshing,
        },
    })
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
        warnings
            .push("Agent execution is not fully implemented in execute_selected yet.".to_string());
    }

    if app.filter_text.trim().is_empty() {
        warnings.push("Tab-to-AI is inactive until the filter has text.".to_string());
    }

    let enter_action = MainWindowPreflightAction {
        kind: enter_action_kind(&result),
        label: app.main_window_primary_action_label(),
        subject: result.name().to_string(),
        type_label: result.type_label().to_string(),
        source_name: result.source_name().map(ToString::to_string),
        description: result.description().map(ToString::to_string),
    };

    Some(MainWindowPreflightReceipt {
        filter_text: app.filter_text.clone(),
        selected_index: app.selected_index,
        selected_result_key: result.stable_selection_key(),
        visible_result_key_fingerprint: visible_result_keys(app).join("|"),
        visible_result_count: app.main_menu_result_caches.grouped_search_results().count(),
        root_passive_frame: build_root_passive_frame_receipt(app),
        enter_action,
        tab_action: build_tab_action(app),
        warnings,
    })
}

pub(crate) fn log_main_window_preflight_receipt(receipt: &MainWindowPreflightReceipt) {
    tracing::info!(
        event = "main_window_preflight_receipt",
        selected_index = receipt.selected_index,
        selected_result_key = ?receipt.selected_result_key,
        visible_result_key_fingerprint = %receipt.visible_result_key_fingerprint,
        visible_result_count = receipt.visible_result_count,
        enter_label = %receipt.enter_action.label,
        enter_subject = %receipt.enter_action.subject,
        enter_type = %receipt.enter_action.type_label,
        tab_enabled = receipt.tab_action.is_some(),
        warnings = ?receipt.warnings,
        "Built main window preflight receipt"
    );
}
