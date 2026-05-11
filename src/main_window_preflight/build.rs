use crate::list_item::GroupedListItem;
use crate::main_window_preflight::types::{
    MainWindowPreflightAction, MainWindowPreflightActionKind, MainWindowPreflightReceipt,
    MainWindowPreflightResultRole, MainWindowPreflightVisibleResult, RootPassiveFrameReceipt,
    RootPassiveSourceReceipt,
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

fn visible_row_fingerprint(app: &crate::ScriptListApp) -> String {
    app.main_menu_result_caches
        .grouped_items()
        .iter()
        .enumerate()
        .map(|(grouped_index, item)| match item {
            GroupedListItem::SectionHeader(label, icon) => {
                format!("h:{grouped_index}:{label}:{icon:?}")
            }
            GroupedListItem::Item(flat_index) => app
                .main_menu_result_caches
                .search_result_for_flat_index(*flat_index)
                .map(|result| {
                    format!(
                        "i:{grouped_index}:{flat_index}:{}:{:?}:{:?}:{}:{}",
                        result.stable_selection_key().unwrap_or_default(),
                        result_role(result),
                        enter_action_kind(result),
                        result.type_label(),
                        result.source_name().unwrap_or("")
                    )
                })
                .unwrap_or_else(|| format!("i:{grouped_index}:{flat_index}:missing")),
        })
        .collect::<Vec<_>>()
        .join("|")
}

fn result_role(result: &crate::scripts::SearchResult) -> MainWindowPreflightResultRole {
    match result {
        crate::scripts::SearchResult::Script(_)
        | crate::scripts::SearchResult::Scriptlet(_)
        | crate::scripts::SearchResult::Skill(_)
        | crate::scripts::SearchResult::BuiltIn(_)
        | crate::scripts::SearchResult::App(_)
        | crate::scripts::SearchResult::Window(_) => MainWindowPreflightResultRole::Primary,
        crate::scripts::SearchResult::File(_) => MainWindowPreflightResultRole::RootFile,
        crate::scripts::SearchResult::Note(_)
        | crate::scripts::SearchResult::AcpHistory(_)
        | crate::scripts::SearchResult::ClipboardHistory(_)
        | crate::scripts::SearchResult::DictationHistory(_)
        | crate::scripts::SearchResult::BrowserTab(_)
        | crate::scripts::SearchResult::BrowserHistory(_) => {
            MainWindowPreflightResultRole::RootPassive
        }
        crate::scripts::SearchResult::Fallback(_) => MainWindowPreflightResultRole::Fallback,
        crate::scripts::SearchResult::ScriptIssue(_) => MainWindowPreflightResultRole::ScriptIssue,
        crate::scripts::SearchResult::Agent(_) => MainWindowPreflightResultRole::Agent,
    }
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

fn visible_result_receipts(app: &crate::ScriptListApp) -> Vec<MainWindowPreflightVisibleResult> {
    app.main_menu_result_caches
        .grouped_items()
        .iter()
        .enumerate()
        .filter_map(|(grouped_index, item)| {
            let GroupedListItem::Item(flat_index) = item else {
                return None;
            };
            let result = app
                .main_menu_result_caches
                .search_result_for_flat_index(*flat_index)?;
            Some(MainWindowPreflightVisibleResult {
                visible_rank: 0,
                grouped_index,
                stable_key: result.stable_selection_key(),
                role: result_role(result),
                action_kind: enter_action_kind(result),
                type_label: result.type_label().to_string(),
                source_name: result.source_name().map(ToString::to_string),
            })
        })
        .enumerate()
        .map(|(visible_rank, mut receipt)| {
            receipt.visible_rank = visible_rank;
            receipt
        })
        .collect()
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
        notes: RootPassiveSourceReceipt {
            enabled: frame.key.notes_options.enabled,
            frame_count: frame.note_hits.len(),
            cache_generation: 0,
            frame_generation: 0,
            refreshing: false,
        },
        clipboard_history: RootPassiveSourceReceipt {
            enabled: frame.key.clipboard_history_options.enabled,
            frame_count: frame.clipboard_history_hits.len(),
            cache_generation: 0,
            frame_generation: 0,
            refreshing: false,
        },
        dictation_history: RootPassiveSourceReceipt {
            enabled: frame.key.dictation_history_options.enabled,
            frame_count: frame.dictation_history_hits.len(),
            cache_generation: 0,
            frame_generation: 0,
            refreshing: false,
        },
        acp_history: RootPassiveSourceReceipt {
            enabled: frame.key.acp_history_options.enabled,
            frame_count: frame.acp_history_hits.len(),
            cache_generation: 0,
            frame_generation: 0,
            refreshing: false,
        },
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
    let visible_results = visible_result_receipts(app);
    let selected_result_role = result_role(&result);

    Some(MainWindowPreflightReceipt {
        filter_text: app.filter_text.clone(),
        selected_index: app.selected_index,
        selected_result_key: result.stable_selection_key(),
        selected_result_role,
        visible_results,
        visible_result_key_fingerprint: visible_result_keys(app).join("|"),
        visible_row_fingerprint: visible_row_fingerprint(app),
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
        selected_result_role = ?receipt.selected_result_role,
        visible_result_key_fingerprint = %receipt.visible_result_key_fingerprint,
        visible_row_fingerprint = %receipt.visible_row_fingerprint,
        visible_result_count = receipt.visible_result_count,
        enter_label = %receipt.enter_action.label,
        enter_subject = %receipt.enter_action.subject,
        enter_type = %receipt.enter_action.type_label,
        tab_enabled = receipt.tab_action.is_some(),
        warnings = ?receipt.warnings,
        "Built main window preflight receipt"
    );
}
