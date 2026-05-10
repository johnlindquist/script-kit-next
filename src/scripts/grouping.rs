//! Result grouping for the main menu
//!
//! This module provides functions for grouping search results into
//! sections based on their source kit.
//!
//! When the filter is empty (grouped view), items are organized by their source kit:
//! - SUGGESTED (frecency-based recent items)
//! - {KIT_NAME} (e.g., CLEANSHOT, MAIN - containing scripts, scriptlets, AND agents from that kit)
//! - COMMANDS (built-ins and window controls)
//! - APPS (installed applications)
//!
//! Note: Scripts, scriptlets, and agents are all grouped under their source kit section.
//! The "main" kit appears last in the kit-based sections.

use std::sync::Arc;
use tracing::instrument;

use crate::app_launcher::AppInfo;
use crate::builtins::{menu_bar_items_to_entries, BuiltInEntry};
use crate::config::SuggestedConfig;
use crate::frecency::FrecencyStore;
use crate::list_item::GroupedListItem;
use crate::menu_bar::MenuBarItem;
use crate::plugins::PluginSkill;

use super::search::fuzzy_search_unified_all_with_skills;
use super::types::{
    FallbackMatch, MatchIndices, Script, ScriptIssueMatch, ScriptMatch, ScriptMatchKind, Scriptlet,
    SearchResult,
};
use super::validation::ValidationReport;

mod grouped_view;
mod search_mode;

/// Default maximum number of items to show in the RECENT section
pub const DEFAULT_MAX_RECENT_ITEMS: usize = 10;

/// Default suggested built-in names for new users without frecency data.
/// These appear in the SUGGESTED section when the user has no usage history.
/// Order matters - items appear in this order and must match built-in entry names.
pub const DEFAULT_SUGGESTED_ITEMS: &[&str] = &[
    "Agent Chat",
    "Do in Current App",
    "New Script",
    "Clipboard History",
    "Open Notes",
    "Search Files",
    "Search Browser Tabs",
    "Quick Terminal",
    "SDK Reference",
];

/// Maximum number of menu bar items to show in search results
/// This prevents menu bar actions from overwhelming the results
pub const MAX_MENU_BAR_ITEMS: usize = 5;

/// Minimum score required for a menu bar item to appear in results
/// This filters out weak matches that would clutter the list
pub const MIN_MENU_BAR_SCORE: i32 = 25;

/// Get grouped results with SUGGESTED/MAIN sections based on frecency.
#[instrument(level = "debug", skip_all, fields(filter_len = filter_text.len()))]
#[allow(clippy::too_many_arguments)]
pub fn get_grouped_results(
    scripts: &[Arc<Script>],
    scriptlets: &[Arc<Scriptlet>],
    builtins: &[BuiltInEntry],
    apps: &[AppInfo],
    skills: &[Arc<PluginSkill>],
    frecency_store: &FrecencyStore,
    filter_text: &str,
    suggested_config: &SuggestedConfig,
    menu_bar_items: &[MenuBarItem],
    menu_bar_bundle_id: Option<&str>,
) -> (Vec<GroupedListItem>, Vec<SearchResult>) {
    get_grouped_results_with_input_history(
        scripts,
        scriptlets,
        builtins,
        apps,
        skills,
        frecency_store,
        filter_text,
        suggested_config,
        menu_bar_items,
        menu_bar_bundle_id,
        None,
    )
}

#[instrument(level = "debug", skip_all, fields(filter_len = filter_text.len()))]
#[allow(clippy::too_many_arguments)]
pub(crate) fn get_grouped_results_with_input_history(
    scripts: &[Arc<Script>],
    scriptlets: &[Arc<Scriptlet>],
    builtins: &[BuiltInEntry],
    apps: &[AppInfo],
    skills: &[Arc<PluginSkill>],
    frecency_store: &FrecencyStore,
    filter_text: &str,
    suggested_config: &SuggestedConfig,
    menu_bar_items: &[MenuBarItem],
    menu_bar_bundle_id: Option<&str>,
    input_history: Option<&crate::input_history::InputHistory>,
) -> (Vec<GroupedListItem>, Vec<SearchResult>) {
    get_grouped_results_with_input_history_and_query(
        scripts,
        scriptlets,
        builtins,
        apps,
        skills,
        frecency_store,
        filter_text,
        suggested_config,
        menu_bar_items,
        menu_bar_bundle_id,
        input_history,
        None,
    )
}

/// Variant of [`get_grouped_results_with_input_history`] that also accepts an
/// optional [`crate::menu_syntax::AdvancedQuery`] for `:` prefix filtering.
///
/// When `advanced_query` is `Some`, the caller is expected to have already
/// substituted `filter_text` with the free-text portion (via
/// [`crate::menu_syntax::free_text_for_search`]). The fuzzy search runs against
/// `filter_text` and the predicate list post-filters the results before
/// search-mode frecency sorting or the grouped-view layout runs.
#[instrument(level = "debug", skip_all, fields(filter_len = filter_text.len()))]
#[allow(clippy::too_many_arguments)]
pub(crate) fn get_grouped_results_with_input_history_and_query(
    scripts: &[Arc<Script>],
    scriptlets: &[Arc<Scriptlet>],
    builtins: &[BuiltInEntry],
    apps: &[AppInfo],
    skills: &[Arc<PluginSkill>],
    frecency_store: &FrecencyStore,
    filter_text: &str,
    suggested_config: &SuggestedConfig,
    menu_bar_items: &[MenuBarItem],
    menu_bar_bundle_id: Option<&str>,
    input_history: Option<&crate::input_history::InputHistory>,
    advanced_query: Option<&crate::menu_syntax::AdvancedQuery>,
) -> (Vec<GroupedListItem>, Vec<SearchResult>) {
    // When filter is non-empty and we have menu bar items, include them in search.
    let all_builtins: Vec<BuiltInEntry>;
    let builtins_to_use: &[BuiltInEntry] = if let Some(bundle_id) =
        menu_bar_bundle_id.filter(|_| !filter_text.is_empty() && !menu_bar_items.is_empty())
    {
        let app_name = bundle_id.rsplit('.').next().unwrap_or(bundle_id);
        let menu_entries = menu_bar_items_to_entries(menu_bar_items, bundle_id, app_name);
        all_builtins = builtins.iter().cloned().chain(menu_entries).collect();
        &all_builtins
    } else {
        builtins
    };

    let results = fuzzy_search_unified_all_with_skills(
        scripts,
        scriptlets,
        builtins_to_use,
        apps,
        skills,
        filter_text,
    );

    let results = match advanced_query {
        Some(query) => crate::menu_syntax::apply_advanced_query(results, query),
        None => results,
    };

    if !filter_text.is_empty() {
        let preferred_result_key =
            input_history.and_then(|history| history.preferred_result_key(filter_text));
        return search_mode::build_search_mode_results(
            results,
            scripts,
            frecency_store,
            filter_text,
            preferred_result_key,
        );
    }

    grouped_view::build_grouped_view_results(results, frecency_store, suggested_config)
}

/// Pins a synthetic `SearchResult::ScriptIssue` row at `flat_results[0]` and
/// shifts every existing `GroupedListItem::Item(idx)` by +1 so the rest of
/// the list continues to point at the original results.
///
/// Called from [`get_grouped_results_with_validation`] when validation has
/// recorded one or more failed scripts and the surface should show the
/// launcher "Script Issues" repair row.
pub(crate) fn prepend_script_issues_row(
    grouped: &mut Vec<GroupedListItem>,
    flat_results: &mut Vec<SearchResult>,
    validation: &ValidationReport,
) {
    let failed_count = validation.failed_scripts.len();
    if failed_count == 0 {
        return;
    }

    let fatal_count = validation.fatal_count;
    let warning_count = validation.warning_count;

    let description = if fatal_count > 0 && warning_count > 0 {
        Some(format!(
            "{} failed · {} fatal · {} warning{}",
            failed_count,
            fatal_count,
            warning_count,
            if warning_count == 1 { "" } else { "s" }
        ))
    } else if fatal_count > 0 {
        Some(format!(
            "{} script{} excluded · {} fatal issue{}",
            failed_count,
            if failed_count == 1 { "" } else { "s" },
            fatal_count,
            if fatal_count == 1 { "" } else { "s" }
        ))
    } else {
        Some(format!(
            "{} script{} flagged",
            failed_count,
            if failed_count == 1 { "" } else { "s" }
        ))
    };

    let issue = ScriptIssueMatch {
        title: format!("Script Issues ({failed_count})"),
        description,
        failed_count,
        fatal_count,
        warning_count,
        score: i32::MAX, // pinned to the top regardless of sort
    };

    flat_results.insert(0, SearchResult::ScriptIssue(issue));

    for entry in grouped.iter_mut() {
        if let GroupedListItem::Item(idx) = entry {
            *idx += 1;
        }
    }

    grouped.insert(0, GroupedListItem::Item(0));
}

/// Validation-aware sibling of [`get_grouped_results_with_input_history`].
///
/// When `validation` is `Some` and it recorded failed scripts, a synthetic
/// `SearchResult::ScriptIssue` row is pinned at the top of the results so the
/// launcher surfaces "my script vanished" repair paths inline.
#[instrument(level = "debug", skip_all, fields(filter_len = filter_text.len()))]
#[allow(clippy::too_many_arguments)]
pub(crate) fn get_grouped_results_with_validation(
    scripts: &[Arc<Script>],
    scriptlets: &[Arc<Scriptlet>],
    builtins: &[BuiltInEntry],
    apps: &[AppInfo],
    skills: &[Arc<PluginSkill>],
    frecency_store: &FrecencyStore,
    filter_text: &str,
    suggested_config: &SuggestedConfig,
    menu_bar_items: &[MenuBarItem],
    menu_bar_bundle_id: Option<&str>,
    input_history: Option<&crate::input_history::InputHistory>,
    validation: Option<&ValidationReport>,
) -> (Vec<GroupedListItem>, Vec<SearchResult>) {
    get_grouped_results_with_validation_and_query(
        scripts,
        scriptlets,
        builtins,
        apps,
        skills,
        frecency_store,
        filter_text,
        suggested_config,
        menu_bar_items,
        menu_bar_bundle_id,
        input_history,
        validation,
        None,
    )
}

/// Validation-aware sibling of [`get_grouped_results_with_input_history_and_query`].
///
/// If `advanced_query` has predicates that reject `SearchResult::ScriptIssue`
/// (for example `:type:script` without `issue` anywhere), the synthetic issue
/// row is not prepended. Without this guard a filter like `:type:script git`
/// would leak an Issue-kind row into a script-only view.
#[instrument(level = "debug", skip_all, fields(filter_len = filter_text.len()))]
#[allow(clippy::too_many_arguments)]
pub(crate) fn get_grouped_results_with_validation_and_query(
    scripts: &[Arc<Script>],
    scriptlets: &[Arc<Scriptlet>],
    builtins: &[BuiltInEntry],
    apps: &[AppInfo],
    skills: &[Arc<PluginSkill>],
    frecency_store: &FrecencyStore,
    filter_text: &str,
    suggested_config: &SuggestedConfig,
    menu_bar_items: &[MenuBarItem],
    menu_bar_bundle_id: Option<&str>,
    input_history: Option<&crate::input_history::InputHistory>,
    validation: Option<&ValidationReport>,
    advanced_query: Option<&crate::menu_syntax::AdvancedQuery>,
) -> (Vec<GroupedListItem>, Vec<SearchResult>) {
    let (mut grouped, mut flat_results) = get_grouped_results_with_input_history_and_query(
        scripts,
        scriptlets,
        builtins,
        apps,
        skills,
        frecency_store,
        filter_text,
        suggested_config,
        menu_bar_items,
        menu_bar_bundle_id,
        input_history,
        advanced_query,
    );

    // Show the pinned row unconditionally when the grouped view is active
    // (empty filter) and there are failures. Also show during search when the
    // query hints at "issues" so authors can Cmd-F to the repair row.
    let filter_hints_issues = {
        let q = filter_text.trim().to_lowercase();
        ["issue", "issues", "failed", "validation", "hidden"]
            .iter()
            .any(|needle| q.contains(*needle))
    };

    let should_show = filter_text.is_empty() || filter_hints_issues;

    if should_show {
        if let Some(report) = validation {
            if !report.failed_scripts.is_empty() && !advanced_query_rejects_issue(advanced_query) {
                prepend_script_issues_row(&mut grouped, &mut flat_results, report);
            }
        }
    }

    (grouped, flat_results)
}

#[instrument(level = "debug", skip_all, fields(filter_len = filter_text.len()))]
#[allow(clippy::too_many_arguments)]
pub(crate) fn get_grouped_results_with_validation_query_and_root_files(
    scripts: &[Arc<Script>],
    scriptlets: &[Arc<Scriptlet>],
    builtins: &[BuiltInEntry],
    apps: &[AppInfo],
    skills: &[Arc<PluginSkill>],
    frecency_store: &FrecencyStore,
    filter_text: &str,
    suggested_config: &SuggestedConfig,
    menu_bar_items: &[MenuBarItem],
    menu_bar_bundle_id: Option<&str>,
    input_history: Option<&crate::input_history::InputHistory>,
    validation: Option<&ValidationReport>,
    advanced_query: Option<&crate::menu_syntax::AdvancedQuery>,
    root_file_search_mode: Option<crate::file_search::RootFileSectionMode>,
    root_file_search_loading: bool,
    root_file_results: &[crate::file_search::FileResult],
    root_recent_file_results: &[crate::file_search::FileResult],
) -> (Vec<GroupedListItem>, Vec<SearchResult>) {
    let (mut grouped, mut flat_results) = get_grouped_results_with_validation_and_query(
        scripts,
        scriptlets,
        builtins,
        apps,
        skills,
        frecency_store,
        filter_text,
        suggested_config,
        menu_bar_items,
        menu_bar_bundle_id,
        input_history,
        validation,
        advanced_query,
    );

    append_root_file_section(
        &mut grouped,
        &mut flat_results,
        root_file_search_mode,
        root_file_search_loading,
        root_file_results,
        root_recent_file_results,
        filter_text,
        frecency_store,
        advanced_query,
    );
    append_recent_root_file_section(
        &mut grouped,
        &mut flat_results,
        root_recent_file_results,
        filter_text,
        advanced_query,
    );

    (grouped, flat_results)
}

fn append_recent_root_file_section(
    grouped: &mut Vec<GroupedListItem>,
    flat_results: &mut Vec<SearchResult>,
    recent_file_results: &[crate::file_search::FileResult],
    filter_text: &str,
    advanced_query: Option<&crate::menu_syntax::AdvancedQuery>,
) {
    if advanced_query.is_some() || !filter_text.trim().is_empty() || recent_file_results.is_empty()
    {
        return;
    }

    let insertion_index = grouped
        .iter()
        .position(
            |item| matches!(item, GroupedListItem::SectionHeader(label, _) if label == "Suggested"),
        )
        .and_then(|suggested_index| {
            grouped
                .iter()
                .enumerate()
                .skip(suggested_index + 1)
                .find_map(|(index, item)| {
                    matches!(item, GroupedListItem::SectionHeader(label, _) if label != "Suggested")
                        .then_some(index)
                })
        })
        .or_else(|| {
            grouped
                .iter()
                .position(|item| matches!(item, GroupedListItem::SectionHeader(_, _)))
        })
        .unwrap_or(grouped.len());

    let mut recent_group = Vec::with_capacity(recent_file_results.len() + 1);
    recent_group.push(GroupedListItem::SectionHeader(
        "Recent Files".to_string(),
        None,
    ));
    for (rank, file) in recent_file_results.iter().enumerate() {
        let idx = flat_results.len();
        flat_results.push(SearchResult::File(crate::scripts::FileMatch {
            file: file.clone(),
            score: i32::MAX.saturating_sub(rank as i32),
        }));
        recent_group.push(GroupedListItem::Item(idx));
    }

    grouped.splice(insertion_index..insertion_index, recent_group);
}

fn append_root_file_section(
    grouped: &mut Vec<GroupedListItem>,
    flat_results: &mut Vec<SearchResult>,
    root_file_search_mode: Option<crate::file_search::RootFileSectionMode>,
    root_file_search_loading: bool,
    root_file_results: &[crate::file_search::FileResult],
    root_recent_file_results: &[crate::file_search::FileResult],
    filter_text: &str,
    frecency_store: &FrecencyStore,
    advanced_query: Option<&crate::menu_syntax::AdvancedQuery>,
) {
    let Some(mode) = root_file_search_mode else {
        return;
    };
    if advanced_query.is_some() {
        return;
    }

    let files = match mode {
        crate::file_search::RootFileSectionMode::GlobalQuery => {
            let merged = merge_root_global_file_results_with_recent(
                root_file_results,
                root_recent_file_results,
                filter_text,
            );
            crate::file_search::rank_root_file_results(
                &merged,
                filter_text,
                crate::file_search::ROOT_FILE_RENDER_LIMIT,
                |key| frecency_store.get_score(key),
            )
        }
        crate::file_search::RootFileSectionMode::DirectoryBrowse => {
            let child_filter = root_directory_browse_child_filter(filter_text);
            crate::file_search::root_directory_file_matches(
                root_file_results,
                child_filter.as_deref(),
                crate::file_search::ROOT_FILE_BROWSE_RENDER_LIMIT,
            )
        }
    };
    let handoff = root_file_search_handoff_result(filter_text, mode);
    if files.is_empty() && handoff.is_none() {
        return;
    }

    let promote = root_file_section_should_promote(mode, filter_text, &files, flat_results);
    let insertion_index = root_file_section_insertion_index(grouped, flat_results, promote);

    let mut file_group = Vec::with_capacity(files.len() + 2);
    file_group.push(GroupedListItem::SectionHeader(
        root_file_section_title(mode, root_file_search_loading).to_string(),
        None,
    ));
    for file_match in files {
        let idx = flat_results.len();
        flat_results.push(SearchResult::File(file_match));
        file_group.push(GroupedListItem::Item(idx));
    }
    if let Some(handoff) = handoff {
        let idx = flat_results.len();
        flat_results.push(handoff);
        file_group.push(GroupedListItem::Item(idx));
    }
    grouped.splice(insertion_index..insertion_index, file_group);
}

fn root_file_section_should_promote(
    mode: crate::file_search::RootFileSectionMode,
    filter_text: &str,
    files: &[crate::scripts::FileMatch],
    flat_results: &[SearchResult],
) -> bool {
    if mode != crate::file_search::RootFileSectionMode::GlobalQuery {
        return false;
    }

    let query = filter_text.trim();
    if !crate::file_search::root_file_global_query_is_eligible(query) {
        return false;
    }

    if top_launcher_result_strongly_matches_query(flat_results, query) {
        return false;
    }

    let Some(first_file) = files.first() else {
        return false;
    };

    crate::file_search::root_file_name_token_matches_query(&first_file.file.name, query)
}

fn top_launcher_result_strongly_matches_query(flat_results: &[SearchResult], query: &str) -> bool {
    flat_results
        .iter()
        .find(|result| {
            !matches!(
                result,
                SearchResult::ScriptIssue(_) | SearchResult::Fallback(_)
            )
        })
        .is_some_and(|result| {
            !matches!(result, SearchResult::File(_))
                && crate::file_search::root_file_name_token_matches_query(result.name(), query)
        })
}

fn root_file_section_insertion_index(
    grouped: &[GroupedListItem],
    flat_results: &[SearchResult],
    promote: bool,
) -> usize {
    if promote {
        return match grouped.first() {
            Some(GroupedListItem::Item(result_idx))
                if matches!(
                    flat_results.get(*result_idx),
                    Some(SearchResult::ScriptIssue(_))
                ) =>
            {
                1
            }
            _ => 0,
        };
    }

    grouped
        .iter()
        .position(|item| match item {
            GroupedListItem::Item(result_idx) => matches!(
                flat_results.get(*result_idx),
                Some(SearchResult::Fallback(_))
            ),
            GroupedListItem::SectionHeader(label, None) => {
                label.starts_with("Use \"") && label.ends_with("\" with...")
            }
            GroupedListItem::SectionHeader(_, Some(_)) => false,
        })
        .unwrap_or(grouped.len())
}

fn merge_root_global_file_results_with_recent(
    provider_results: &[crate::file_search::FileResult],
    recent_results: &[crate::file_search::FileResult],
    filter_text: &str,
) -> Vec<crate::file_search::FileResult> {
    let mut seen = std::collections::HashSet::new();
    let mut merged = Vec::with_capacity(provider_results.len() + recent_results.len());

    for file in provider_results
        .iter()
        .filter(|file| crate::file_search::root_global_file_result_is_eligible(file))
    {
        if seen.insert(file.path.clone()) {
            merged.push(file.clone());
        }
    }
    for file in recent_results.iter().filter(|file| {
        crate::file_search::root_global_file_result_is_eligible(file)
            && crate::file_search::root_file_name_seed_matches_query(&file.name, filter_text)
    }) {
        if seen.insert(file.path.clone()) {
            merged.push(file.clone());
        }
    }

    merged
}

fn root_file_section_title(
    mode: crate::file_search::RootFileSectionMode,
    loading: bool,
) -> &'static str {
    if !loading {
        return "Files";
    }

    match mode {
        crate::file_search::RootFileSectionMode::GlobalQuery => "Files · Searching...",
        crate::file_search::RootFileSectionMode::DirectoryBrowse => "Files · Loading folder...",
    }
}

fn root_file_search_handoff_result(
    filter_text: &str,
    mode: crate::file_search::RootFileSectionMode,
) -> Option<SearchResult> {
    let query = filter_text.trim();
    if crate::file_search::root_file_section_mode_for_query(query) != Some(mode) {
        return None;
    }

    let fallback = crate::fallbacks::builtins::get_builtin_fallbacks()
        .into_iter()
        .find(|fallback| fallback.id == crate::fallbacks::builtins::SEARCH_FILES_FALLBACK_ID)?;

    let (title, subtitle) = match mode {
        crate::file_search::RootFileSectionMode::GlobalQuery => (
            format!("Search Files for \"{query}\""),
            "Open full File Search".to_string(),
        ),
        crate::file_search::RootFileSectionMode::DirectoryBrowse => {
            let base = crate::file_search::root_directory_query_base(query)?;
            let label = crate::file_search::shorten_path(base.trim_end_matches('/'));
            (
                format!("Open File Search in \"{label}\""),
                "Browse the full folder".to_string(),
            )
        }
    };

    Some(SearchResult::Fallback(
        FallbackMatch::new(crate::fallbacks::FallbackItem::Builtin(fallback), 0)
            .with_display_overrides(title, subtitle),
    ))
}

fn root_directory_browse_child_filter(query: &str) -> Option<String> {
    let query = query.trim();
    let base = crate::file_search::root_directory_query_base(query)?;
    let child_filter = query.strip_prefix(&base)?.trim();
    (!child_filter.is_empty()).then(|| child_filter.to_string())
}

/// Incomplete menu-syntax hint row.
///
/// Returns a single non-selectable `GroupedListItem::SectionHeader(hint, None)`
/// and empty flat results. This is what renders when the user has typed a
/// power-syntax prefix that is not yet a complete invocation — for example
/// `:` (bare advanced query), `+` (bare capture prefix), or `+todo` (known
/// capture target without a body).
///
/// Selection maps through `GroupedListItem::Item(idx)` only, so a header is
/// automatically non-selectable. Do not reuse `SearchResult::ScriptIssue`:
/// that variant is selectable and routes to diagnostics.
pub(crate) fn build_menu_syntax_hint_results(
    hint: &str,
) -> (Vec<GroupedListItem>, Vec<SearchResult>) {
    (
        vec![GroupedListItem::SectionHeader(hint.to_string(), None)],
        Vec::new(),
    )
}

/// Capture-mode grouped results.
///
/// Replaces the normal launcher grouping entirely when the user typed a
/// `+<target>` or `<target>:` capture syntax. Do not mix with Suggested,
/// Favorites, Recent, menu-bar items, calculator, or fallbacks — capture
/// should render only handler scripts that opted into
/// `menuSyntax: [{ family: "capture.v1", targets: [...] }]`.
///
/// Returns a one-section layout:
/// - `SectionHeader("Capture <target>", None)` — always present
/// - `Item(i)` rows, one per handler script, in the order
///   `scripts_handling_capture` returns them (defaults first, then remaining)
///
/// When no handler scripts match, returns a single non-selectable help row
/// explaining that no scripts opted into `capture.v1/<target>`.
pub(crate) fn build_capture_mode_results(
    scripts: &[Arc<Script>],
    invocation: &crate::menu_syntax::CaptureInvocation,
) -> (Vec<GroupedListItem>, Vec<SearchResult>) {
    let handlers = crate::menu_syntax::rank_scripts_handling_capture(scripts, invocation);

    if handlers.is_empty() {
        return (
            vec![GroupedListItem::SectionHeader(
                format!("No scripts opted into capture.v1/{}", invocation.target),
                None,
            )],
            Vec::new(),
        );
    }

    let header = format!("Capture {}", invocation.target);
    let mut grouped: Vec<GroupedListItem> = Vec::with_capacity(handlers.len() + 1);
    grouped.push(GroupedListItem::SectionHeader(header, None));
    let mut flat_results: Vec<SearchResult> = Vec::with_capacity(handlers.len());

    for (idx, script) in handlers.into_iter().enumerate() {
        let filename = script
            .path
            .file_name()
            .map(|f: &std::ffi::OsStr| f.to_string_lossy().into_owned())
            .unwrap_or_default();
        flat_results.push(SearchResult::Script(ScriptMatch {
            script,
            score: i32::MAX,
            filename,
            match_indices: MatchIndices::default(),
            match_kind: ScriptMatchKind::Name,
            content_match: None,
        }));
        grouped.push(GroupedListItem::Item(idx));
    }

    (grouped, flat_results)
}

/// Returns `true` when `advanced_query` has predicates that would exclude a
/// synthetic `SearchResult::ScriptIssue` row from results. Only the predicates
/// are checked (no free-text substring match), so `:type:script` suppresses the
/// issue row while `:issues` keeps it.
fn advanced_query_rejects_issue(
    advanced_query: Option<&crate::menu_syntax::AdvancedQuery>,
) -> bool {
    let Some(query) = advanced_query else {
        return false;
    };
    if query.predicates.is_empty() {
        return false;
    }
    let synthetic = SearchResult::ScriptIssue(ScriptIssueMatch {
        title: String::new(),
        description: None,
        failed_count: 0,
        fatal_count: 0,
        warning_count: 0,
        score: 0,
    });
    !query
        .predicates
        .iter()
        .all(|p| crate::menu_syntax::matches_predicate(&synthetic, p))
}

#[cfg(test)]
mod advanced_query_tests {
    use super::*;
    use crate::file_search::{FileResult, FileType};
    use crate::menu_syntax::{parse, AdvancedQuery, MenuSyntaxParse};
    use crate::scripts::types::{BuiltInMatch, MatchIndices};

    fn issue_row() -> SearchResult {
        SearchResult::ScriptIssue(ScriptIssueMatch {
            title: "Script Issues (1)".into(),
            description: None,
            failed_count: 1,
            fatal_count: 1,
            warning_count: 0,
            score: i32::MAX,
        })
    }

    fn advanced_query_from(raw: &str) -> AdvancedQuery {
        match parse(raw) {
            MenuSyntaxParse::AdvancedQuery(q) => q,
            other => panic!("expected AdvancedQuery for {raw:?}, got {other:?}"),
        }
    }

    #[test]
    fn rejects_issue_under_type_script_predicate() {
        let query = advanced_query_from(":type:script git");
        assert!(advanced_query_rejects_issue(Some(&query)));
    }

    #[test]
    fn allows_issue_under_type_issue_predicate() {
        let query = advanced_query_from(":type:issue");
        assert!(!advanced_query_rejects_issue(Some(&query)));
    }

    #[test]
    fn no_advanced_query_never_rejects_issue() {
        assert!(!advanced_query_rejects_issue(None));
    }

    #[test]
    fn empty_predicates_never_reject_issue() {
        let query = advanced_query_from(": git");
        assert!(query.predicates.is_empty());
        assert!(!advanced_query_rejects_issue(Some(&query)));
    }

    #[test]
    fn apply_advanced_query_drops_issue_with_type_script() {
        let query = advanced_query_from(":type:script git");
        let results = vec![issue_row()];
        let filtered = crate::menu_syntax::apply_advanced_query(results, &query);
        assert!(
            filtered.is_empty(),
            ":type:script must not leak a ScriptIssue row through grouping"
        );
    }

    #[test]
    fn apply_advanced_query_keeps_issue_with_type_issue() {
        let query = advanced_query_from(":type:issue");
        let results = vec![issue_row()];
        let filtered = crate::menu_syntax::apply_advanced_query(results, &query);
        assert_eq!(filtered.len(), 1);
    }

    #[test]
    fn _compile_time_signature_and_marker() {
        // Compile-time witness that the new entry points exist and accept Option<&AdvancedQuery>.
        // Exercising them at runtime requires fuzzy-search fixtures that pull in heavy state;
        // that surface is covered by the existing scripts::grouping chunked tests.
        let _: fn(
            &[Arc<Script>],
            &[Arc<Scriptlet>],
            &[BuiltInEntry],
            &[AppInfo],
            &[Arc<PluginSkill>],
            &FrecencyStore,
            &str,
            &SuggestedConfig,
            &[MenuBarItem],
            Option<&str>,
            Option<&crate::input_history::InputHistory>,
            Option<&AdvancedQuery>,
        ) -> (Vec<GroupedListItem>, Vec<SearchResult>) =
            get_grouped_results_with_input_history_and_query;

        let _: fn(
            &[Arc<Script>],
            &[Arc<Scriptlet>],
            &[BuiltInEntry],
            &[AppInfo],
            &[Arc<PluginSkill>],
            &FrecencyStore,
            &str,
            &SuggestedConfig,
            &[MenuBarItem],
            Option<&str>,
            Option<&crate::input_history::InputHistory>,
            Option<&ValidationReport>,
            Option<&AdvancedQuery>,
        ) -> (Vec<GroupedListItem>, Vec<SearchResult>) =
            get_grouped_results_with_validation_and_query;

        let m = MatchIndices::default();
        let _ = m.clone();
    }

    // @lat: menu-syntax Advanced Query
    #[test]
    fn query_predicates_suppress_issue_row_prepend() {
        // Deep proof that `:type:script <text>` never prepends an issue row even
        // when validation reports failed scripts. We inspect the shared helper
        // to avoid spinning up a full frecency/scripts fixture.
        let query = advanced_query_from(":type:script something");
        assert!(advanced_query_rejects_issue(Some(&query)));
    }

    fn root_file(path: &str, name: &str) -> FileResult {
        root_file_with_type(path, name, FileType::Document)
    }

    fn root_file_with_type(path: &str, name: &str, file_type: FileType) -> FileResult {
        FileResult {
            path: path.to_string(),
            name: name.to_string(),
            size: 0,
            modified: 0,
            file_type,
        }
    }

    fn builtin_result(name: &str) -> SearchResult {
        SearchResult::BuiltIn(BuiltInMatch {
            entry: BuiltInEntry {
                id: format!("builtin/{}", name.to_lowercase().replace(' ', "-")),
                name: name.to_string(),
                description: "Test built-in".to_string(),
                keywords: Vec::new(),
                feature: crate::builtins::BuiltInFeature::AppLauncher,
                icon: None,
                group: crate::builtins::BuiltInGroup::Core,
            },
            score: i32::MAX,
        })
    }

    #[test]
    fn root_file_rows_append_files_section_for_eligible_search() {
        let frecency_store = FrecencyStore::new();
        let root_files = vec![root_file(
            "/Users/example/Desktop/fix spelling.png",
            "fix spelling.png",
        )];

        let (grouped, flat) = get_grouped_results_with_validation_query_and_root_files(
            &[],
            &[],
            &[],
            &[],
            &[],
            &frecency_store,
            "fix",
            &SuggestedConfig::default(),
            &[],
            None,
            None,
            None,
            None,
            Some(crate::file_search::RootFileSectionMode::GlobalQuery),
            false,
            &root_files,
            &[],
        );

        assert!(
            grouped
                .iter()
                .any(|item| matches!(item, GroupedListItem::SectionHeader(label, None) if label == "Files")),
            "eligible root queries should append a Files section"
        );
        assert!(
            flat.iter().any(|result| matches!(
                result,
                SearchResult::File(file) if file.file.path == "/Users/example/Desktop/fix spelling.png"
            )),
            "Files section should point at the ranked root file row"
        );
    }

    #[test]
    fn root_global_file_rows_seed_matching_recent_files_while_provider_loading() {
        let frecency_store = FrecencyStore::new();
        let recent_files = vec![root_file(
            "/Users/example/Desktop/design-notes.md",
            "design-notes.md",
        )];

        let (grouped, flat) = get_grouped_results_with_validation_query_and_root_files(
            &[],
            &[],
            &[],
            &[],
            &[],
            &frecency_store,
            "design",
            &SuggestedConfig::default(),
            &[],
            None,
            None,
            None,
            None,
            Some(crate::file_search::RootFileSectionMode::GlobalQuery),
            true,
            &[],
            &recent_files,
        );

        assert!(
            grouped
                .iter()
                .any(|item| matches!(item, GroupedListItem::SectionHeader(label, None) if label == "Files · Searching...")),
            "global root search should keep the loading Files header while recent seeds render"
        );
        assert!(
            flat.iter().any(|result| matches!(
                result,
                SearchResult::File(file) if file.file.path == "/Users/example/Desktop/design-notes.md"
            )),
            "matching recent files should seed non-empty global root file results before provider rows arrive"
        );
        assert!(
            flat.iter().any(|result| matches!(
                result,
                SearchResult::Fallback(fallback) if fallback.display_label() == "Search Files for \"design\""
            )),
            "seeded global file rows should keep the full File Search handoff"
        );
    }

    #[test]
    fn root_global_recent_seed_rejects_path_only_match_while_loading() {
        let frecency_store = FrecencyStore::new();
        let recent_files = vec![root_file(
            "/Users/example/Design/archive/readme.md",
            "readme.md",
        )];

        let (grouped, flat) = get_grouped_results_with_validation_query_and_root_files(
            &[],
            &[],
            &[],
            &[],
            &[],
            &frecency_store,
            "design",
            &SuggestedConfig::default(),
            &[],
            None,
            None,
            None,
            None,
            Some(crate::file_search::RootFileSectionMode::GlobalQuery),
            true,
            &[],
            &recent_files,
        );

        assert!(
            flat.iter().all(|result| !matches!(
                result,
                SearchResult::File(file) if file.file.path == "/Users/example/Design/archive/readme.md"
            )),
            "path-only recent files should not seed non-empty global root search"
        );
        assert!(
            grouped
                .iter()
                .any(|item| matches!(item, GroupedListItem::SectionHeader(label, None) if label == "Files · Searching...")),
            "the loading Files section should remain visible for the continuation row"
        );
        assert!(
            flat.iter().any(|result| matches!(
                result,
                SearchResult::Fallback(fallback) if fallback.display_label() == "Search Files for \"design\""
            )),
            "path-only recent rejection should still keep the dedicated File Search handoff"
        );
    }

    #[test]
    fn root_global_provider_path_only_match_still_renders() {
        let frecency_store = FrecencyStore::new();
        let root_files = vec![root_file(
            "/Users/example/Design/archive/readme.md",
            "readme.md",
        )];

        let (_grouped, flat) = get_grouped_results_with_validation_query_and_root_files(
            &[],
            &[],
            &[],
            &[],
            &[],
            &frecency_store,
            "design",
            &SuggestedConfig::default(),
            &[],
            None,
            None,
            None,
            None,
            Some(crate::file_search::RootFileSectionMode::GlobalQuery),
            false,
            &root_files,
            &[],
        );

        assert!(
            flat.iter().any(|result| matches!(
                result,
                SearchResult::File(file) if file.file.path == "/Users/example/Design/archive/readme.md"
            )),
            "provider-returned path-only matches should still render after the provider answers"
        );
    }

    #[test]
    fn root_global_file_rows_dedupe_provider_and_recent_by_path() {
        let frecency_store = FrecencyStore::new();
        let shared = root_file("/Users/example/Desktop/design-notes.md", "design-notes.md");
        let provider_files = vec![shared.clone()];
        let recent_files = vec![shared];

        let (_grouped, flat) = get_grouped_results_with_validation_query_and_root_files(
            &[],
            &[],
            &[],
            &[],
            &[],
            &frecency_store,
            "design",
            &SuggestedConfig::default(),
            &[],
            None,
            None,
            None,
            None,
            Some(crate::file_search::RootFileSectionMode::GlobalQuery),
            true,
            &provider_files,
            &recent_files,
        );

        let duplicate_count = flat
            .iter()
            .filter(|result| {
                matches!(
                    result,
                    SearchResult::File(file) if file.file.path == "/Users/example/Desktop/design-notes.md"
                )
            })
            .count();

        assert_eq!(
            duplicate_count, 1,
            "provider and recent rows with the same full path should render once"
        );
    }

    #[test]
    fn root_global_strong_filename_match_promotes_files_section() {
        let files = vec![SearchResult::File(crate::scripts::FileMatch {
            file: root_file("/Users/example/Desktop/design-notes.md", "design-notes.md"),
            score: 100,
        })];
        let grouped = vec![GroupedListItem::SectionHeader("Commands".to_string(), None)];
        let file_matches = files
            .iter()
            .filter_map(|result| match result {
                SearchResult::File(file) => Some(file.clone()),
                _ => None,
            })
            .collect::<Vec<_>>();

        assert!(root_file_section_should_promote(
            crate::file_search::RootFileSectionMode::GlobalQuery,
            "design",
            &file_matches,
            &[],
        ));
        assert_eq!(
            root_file_section_insertion_index(&grouped, &files, true),
            0,
            "strong filename matches should insert Files above ordinary launcher groups"
        );
    }

    #[test]
    fn root_directory_browse_never_promotes_files_section() {
        let files = vec![crate::scripts::FileMatch {
            file: root_file("/Users/example/dev/design-notes.md", "design-notes.md"),
            score: 100,
        }];

        assert!(!root_file_section_should_promote(
            crate::file_search::RootFileSectionMode::DirectoryBrowse,
            "design",
            &files,
            &[],
        ));
    }

    #[test]
    fn root_global_boundary_filename_token_match_promotes_files_section() {
        let files = vec![crate::scripts::FileMatch {
            file: root_file(
                "/Users/example/Desktop/client-design-notes.md",
                "client-design-notes.md",
            ),
            score: 100,
        }];

        assert!(root_file_section_should_promote(
            crate::file_search::RootFileSectionMode::GlobalQuery,
            "design",
            &files,
            &[],
        ));
    }

    #[test]
    fn root_global_camel_case_filename_token_match_promotes_files_section() {
        let files = vec![crate::scripts::FileMatch {
            file: root_file(
                "/Users/example/Desktop/ClientDesignNotes.md",
                "ClientDesignNotes.md",
            ),
            score: 100,
        }];

        assert!(root_file_section_should_promote(
            crate::file_search::RootFileSectionMode::GlobalQuery,
            "design",
            &files,
            &[],
        ));
    }

    #[test]
    fn root_global_recent_seed_accepts_camel_case_filename_token() {
        let frecency_store = FrecencyStore::new();
        let recent_files = vec![root_file(
            "/Users/example/Desktop/ClientDesignNotes.md",
            "ClientDesignNotes.md",
        )];

        let (grouped, flat) = get_grouped_results_with_validation_query_and_root_files(
            &[],
            &[],
            &[],
            &[],
            &[],
            &frecency_store,
            "design",
            &SuggestedConfig::default(),
            &[],
            None,
            None,
            None,
            None,
            Some(crate::file_search::RootFileSectionMode::GlobalQuery),
            true,
            &[],
            &recent_files,
        );

        assert!(
            grouped.iter().any(|item| matches!(
                item,
                GroupedListItem::SectionHeader(label, None) if label == "Files · Searching..."
            )),
            "global root search should keep the loading Files header while camel-case recent seeds render"
        );
        assert!(
            flat.iter().any(|result| matches!(
                result,
                SearchResult::File(file) if file.file.path == "/Users/example/Desktop/ClientDesignNotes.md"
            )),
            "camel-case filename token matches should seed non-empty global root file results before provider rows arrive"
        );
        assert!(
            flat.iter().any(|result| matches!(
                result,
                SearchResult::Fallback(fallback) if fallback.display_label() == "Search Files for \"design\""
            )),
            "seeded global file rows should keep the full File Search handoff"
        );
    }

    #[test]
    fn root_global_multiword_recent_seed_uses_filename_tokens() {
        let frecency_store = FrecencyStore::new();
        let recent_files = vec![root_file(
            "/Users/example/Desktop/client-design-notes.md",
            "client-design-notes.md",
        )];

        let (grouped, flat) = get_grouped_results_with_validation_query_and_root_files(
            &[],
            &[],
            &[],
            &[],
            &[],
            &frecency_store,
            "client notes",
            &SuggestedConfig::default(),
            &[],
            None,
            None,
            None,
            None,
            Some(crate::file_search::RootFileSectionMode::GlobalQuery),
            true,
            &[],
            &recent_files,
        );

        assert!(
            grouped.iter().any(|item| matches!(
                item,
                GroupedListItem::SectionHeader(label, None) if label == "Files · Searching..."
            )),
            "multi-word recent seeds should render in the loading Files section"
        );
        assert!(
            flat.iter().any(|result| matches!(
                result,
                SearchResult::File(file) if file.file.path == "/Users/example/Desktop/client-design-notes.md"
            )),
            "ordered multi-word filename tokens should seed non-empty global root file results"
        );
    }

    #[test]
    fn root_global_multiword_strong_match_promotes_files_section() {
        let files = vec![crate::scripts::FileMatch {
            file: root_file(
                "/Users/example/Desktop/client-design-notes.md",
                "client-design-notes.md",
            ),
            score: 100,
        }];

        assert!(root_file_section_should_promote(
            crate::file_search::RootFileSectionMode::GlobalQuery,
            "design notes",
            &files,
            &[],
        ));
    }

    #[test]
    fn root_global_multiword_mid_token_match_does_not_promote() {
        let files = vec![crate::scripts::FileMatch {
            file: root_file(
                "/Users/example/Desktop/redesign-notes.md",
                "redesign-notes.md",
            ),
            score: 100,
        }];

        assert!(!root_file_section_should_promote(
            crate::file_search::RootFileSectionMode::GlobalQuery,
            "design notes",
            &files,
            &[],
        ));
    }

    #[test]
    fn root_global_short_digit_recent_seed_uses_filename_tokens() {
        let frecency_store = FrecencyStore::new();
        let recent_files = vec![root_file(
            "/Users/example/Desktop/2026-q2-report.xlsx",
            "2026-q2-report.xlsx",
        )];

        let (grouped, flat) = get_grouped_results_with_validation_query_and_root_files(
            &[],
            &[],
            &[],
            &[],
            &[],
            &frecency_store,
            "q2",
            &SuggestedConfig::default(),
            &[],
            None,
            None,
            None,
            None,
            Some(crate::file_search::RootFileSectionMode::GlobalQuery),
            true,
            &[],
            &recent_files,
        );

        assert!(
            grouped.iter().any(|item| matches!(
                item,
                GroupedListItem::SectionHeader(label, None) if label == "Files · Searching..."
            )),
            "short digit recent seeds should render in the loading Files section"
        );
        assert!(
            flat.iter().any(|result| matches!(
                result,
                SearchResult::File(file) if file.file.name == "2026-q2-report.xlsx"
            )),
            "short digit filename tokens should seed non-empty global root file results"
        );
    }

    #[test]
    fn root_global_short_digit_filename_match_promotes_files_section() {
        let files = vec![crate::scripts::FileMatch {
            file: root_file("/Users/example/Desktop/Q2Report.pdf", "Q2Report.pdf"),
            score: 100,
        }];

        assert!(root_file_section_should_promote(
            crate::file_search::RootFileSectionMode::GlobalQuery,
            "q2",
            &files,
            &[],
        ));
    }

    #[test]
    fn root_global_two_letter_query_still_does_not_promote() {
        let files = vec![crate::scripts::FileMatch {
            file: root_file("/Users/example/Desktop/ai-notes.md", "ai-notes.md"),
            score: 100,
        }];

        assert!(!root_file_section_should_promote(
            crate::file_search::RootFileSectionMode::GlobalQuery,
            "ai",
            &files,
            &[],
        ));
    }

    #[test]
    fn root_global_mid_token_contains_does_not_promote_files_section() {
        let files = vec![crate::scripts::FileMatch {
            file: root_file(
                "/Users/example/Desktop/redesign-notes.md",
                "redesign-notes.md",
            ),
            score: 100,
        }];

        assert!(!root_file_section_should_promote(
            crate::file_search::RootFileSectionMode::GlobalQuery,
            "design",
            &files,
            &[],
        ));
    }

    #[test]
    fn root_global_strong_launcher_match_blocks_file_section_promotion() {
        let files = vec![crate::scripts::FileMatch {
            file: root_file(
                "/Users/example/Desktop/fix spelling.png",
                "fix spelling.png",
            ),
            score: 100,
        }];
        let launcher_results = vec![builtin_result("Fix Spelling and Grammar")];

        assert!(!root_file_section_should_promote(
            crate::file_search::RootFileSectionMode::GlobalQuery,
            "spelling",
            &files,
            &launcher_results,
        ));
    }

    #[test]
    fn root_global_weak_launcher_match_does_not_block_file_section_promotion() {
        let files = vec![crate::scripts::FileMatch {
            file: root_file("/Users/example/Desktop/design-notes.md", "design-notes.md"),
            score: 100,
        }];
        let launcher_results = vec![builtin_result("Redesign Theme")];

        assert!(root_file_section_should_promote(
            crate::file_search::RootFileSectionMode::GlobalQuery,
            "design",
            &files,
            &launcher_results,
        ));
    }

    #[test]
    fn root_file_rows_precede_fallback_rows_for_file_only_search() {
        let frecency_store = FrecencyStore::new();
        let root_files = vec![root_file(
            "/Users/example/Desktop/unique report name.pdf",
            "unique report name.pdf",
        )];

        let (grouped, flat) = get_grouped_results_with_validation_query_and_root_files(
            &[],
            &[],
            &[],
            &[],
            &[],
            &frecency_store,
            "unique report name",
            &SuggestedConfig::default(),
            &[],
            None,
            None,
            None,
            None,
            Some(crate::file_search::RootFileSectionMode::GlobalQuery),
            false,
            &root_files,
            &[],
        );

        let file_grouped_index = grouped
            .iter()
            .position(|item| {
                matches!(
                    item,
                    GroupedListItem::Item(idx)
                        if matches!(flat.get(*idx), Some(SearchResult::File(_)))
                )
            })
            .expect("file result should be grouped");
        let fallback_grouped_index = grouped
            .iter()
            .position(|item| {
                matches!(
                    item,
                    GroupedListItem::Item(idx)
                        if matches!(flat.get(*idx), Some(SearchResult::Fallback(_)))
                )
            })
            .expect("fallback result should still be grouped");

        assert!(
            file_grouped_index < fallback_grouped_index,
            "root file results should appear before fallback actions so Enter opens the file first"
        );
    }

    #[test]
    fn root_file_rows_do_not_append_for_advanced_queries() {
        let frecency_store = FrecencyStore::new();
        let root_files = vec![root_file(
            "/Users/example/Desktop/fix spelling.png",
            "fix spelling.png",
        )];
        let query = advanced_query_from(":type:file fix");

        let (grouped, flat) = get_grouped_results_with_validation_query_and_root_files(
            &[],
            &[],
            &[],
            &[],
            &[],
            &frecency_store,
            "fix",
            &SuggestedConfig::default(),
            &[],
            None,
            None,
            None,
            Some(&query),
            Some(crate::file_search::RootFileSectionMode::GlobalQuery),
            false,
            &root_files,
            &[],
        );

        assert!(
            !grouped
                .iter()
                .any(|item| matches!(item, GroupedListItem::SectionHeader(label, None) if label == "Files")),
            "advanced query mode should not mix in root Spotlight file rows"
        );
        assert!(
            flat.iter()
                .all(|result| !matches!(result, SearchResult::File(_))),
            "advanced query mode should not append file results"
        );
    }

    #[test]
    fn root_global_file_rows_exclude_application_bundles() {
        let frecency_store = FrecencyStore::new();
        let root_files = vec![
            root_file_with_type("/Applications/Zed.app", "Zed.app", FileType::Application),
            root_file_with_type(
                "/Users/example/Desktop/zed-notes.md",
                "zed-notes.md",
                FileType::Document,
            ),
        ];

        let (_grouped, flat) = get_grouped_results_with_validation_query_and_root_files(
            &[],
            &[],
            &[],
            &[],
            &[],
            &frecency_store,
            "zed",
            &SuggestedConfig::default(),
            &[],
            None,
            None,
            None,
            None,
            Some(crate::file_search::RootFileSectionMode::GlobalQuery),
            false,
            &root_files,
            &[],
        );

        let rendered_files = flat
            .iter()
            .filter_map(|result| match result {
                SearchResult::File(file) => Some(file.file.name.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>();

        assert_eq!(
            rendered_files,
            vec!["zed-notes.md"],
            "global root Files should not duplicate app launcher results as .app file rows"
        );
    }

    #[test]
    fn root_global_file_rows_exclude_app_bundle_contents() {
        let frecency_store = FrecencyStore::new();
        let root_files = vec![
            root_file_with_type(
                "/Applications/Zed.app/Contents/Info.plist",
                "Info.plist",
                FileType::Document,
            ),
            root_file_with_type(
                "/Users/example/Desktop/zed-notes.md",
                "zed-notes.md",
                FileType::Document,
            ),
        ];

        let (_grouped, flat) = get_grouped_results_with_validation_query_and_root_files(
            &[],
            &[],
            &[],
            &[],
            &[],
            &frecency_store,
            "zed",
            &SuggestedConfig::default(),
            &[],
            None,
            None,
            None,
            None,
            Some(crate::file_search::RootFileSectionMode::GlobalQuery),
            false,
            &root_files,
            &[],
        );

        let rendered_files = flat
            .iter()
            .filter_map(|result| match result {
                SearchResult::File(file) => Some(file.file.path.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>();

        assert_eq!(
            rendered_files,
            vec!["/Users/example/Desktop/zed-notes.md"],
            "global root Files should not render files nested inside .app bundles"
        );
    }

    #[test]
    fn root_global_app_bundle_filter_keeps_search_files_handoff() {
        let frecency_store = FrecencyStore::new();
        let root_files = vec![root_file_with_type(
            "/Applications/Zed.app",
            "Zed.app",
            FileType::Application,
        )];

        let (grouped, flat) = get_grouped_results_with_validation_query_and_root_files(
            &[],
            &[],
            &[],
            &[],
            &[],
            &frecency_store,
            "zed",
            &SuggestedConfig::default(),
            &[],
            None,
            None,
            None,
            None,
            Some(crate::file_search::RootFileSectionMode::GlobalQuery),
            false,
            &root_files,
            &[],
        );

        assert!(
            flat.iter().all(|result| !matches!(
                result,
                SearchResult::File(file) if file.file.name == "Zed.app"
            )),
            "filtered application bundles should not render as root global file rows"
        );
        assert!(
            grouped.iter().any(|item| {
                matches!(item, GroupedListItem::SectionHeader(label, None) if label == "Files")
            }),
            "the Files section should still be allowed to show the handoff row"
        );
        assert!(
            flat.iter().any(|result| matches!(
                result,
                SearchResult::Fallback(fallback) if fallback.display_label() == "Search Files for \"zed\""
            )),
            "app-bundle filtering should not remove the full File Search handoff"
        );
    }

    #[test]
    fn root_directory_browse_keeps_app_bundle_contents_for_explicit_paths() {
        let frecency_store = FrecencyStore::new();
        let root_files = vec![root_file_with_type(
            "/Applications/Zed.app/Contents/Info.plist",
            "Info.plist",
            FileType::Document,
        )];

        let (_grouped, flat) = get_grouped_results_with_validation_query_and_root_files(
            &[],
            &[],
            &[],
            &[],
            &[],
            &frecency_store,
            "/Applications/Zed.app/Contents/",
            &SuggestedConfig::default(),
            &[],
            None,
            None,
            None,
            None,
            Some(crate::file_search::RootFileSectionMode::DirectoryBrowse),
            false,
            &root_files,
            &[],
        );

        assert!(
            flat.iter().any(|result| matches!(
                result,
                SearchResult::File(file) if file.file.path == "/Applications/Zed.app/Contents/Info.plist"
            )),
            "explicit directory browse should still render already-collected direct children inside .app bundles"
        );
    }

    #[test]
    fn root_directory_browse_rows_append_files_section_for_path_query() {
        let frecency_store = FrecencyStore::new();
        let root_files = vec![
            root_file_with_type("/Users/example/dev/app", "app", FileType::Directory),
            root_file_with_type(
                "/Users/example/dev/Zed.app",
                "Zed.app",
                FileType::Application,
            ),
        ];

        let (grouped, flat) = get_grouped_results_with_validation_query_and_root_files(
            &[],
            &[],
            &[],
            &[],
            &[],
            &frecency_store,
            "~/dev/",
            &SuggestedConfig::default(),
            &[],
            None,
            None,
            None,
            None,
            Some(crate::file_search::RootFileSectionMode::DirectoryBrowse),
            false,
            &root_files,
            &[],
        );

        assert!(
            grouped
                .iter()
                .any(|item| matches!(item, GroupedListItem::SectionHeader(label, None) if label == "Files")),
            "directory path queries should append a Files section"
        );
        assert!(
            flat.iter().any(|result| matches!(
                result,
                SearchResult::File(file) if file.file.path == "/Users/example/dev/Zed.app"
            )),
            "directory browse should render provider-ordered rows, including app bundles"
        );
        assert!(
            flat.iter().any(|result| matches!(
                result,
                SearchResult::Fallback(fallback) if fallback.display_label() == "Open File Search in \"~/dev\""
            )),
            "directory browse should append a folder-scoped File Search handoff"
        );
    }

    #[test]
    fn root_directory_browse_does_not_mix_recent_files() {
        let frecency_store = FrecencyStore::new();
        let root_files = vec![root_file("/Users/example/dev/design.md", "design.md")];
        let recent_files = vec![root_file(
            "/Users/example/Desktop/design-notes.md",
            "design-notes.md",
        )];

        let (_grouped, flat) = get_grouped_results_with_validation_query_and_root_files(
            &[],
            &[],
            &[],
            &[],
            &[],
            &frecency_store,
            "~/dev/design",
            &SuggestedConfig::default(),
            &[],
            None,
            None,
            None,
            None,
            Some(crate::file_search::RootFileSectionMode::DirectoryBrowse),
            true,
            &root_files,
            &recent_files,
        );

        let rendered_paths = flat
            .iter()
            .filter_map(|result| match result {
                SearchResult::File(file) => Some(file.file.path.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>();

        assert_eq!(
            rendered_paths,
            vec!["/Users/example/dev/design.md"],
            "directory browse should render direct children only and ignore recent file seeds"
        );
    }

    #[test]
    fn root_directory_browse_rows_use_provider_order_without_fuzzy_filtering() {
        let frecency_store = FrecencyStore::new();
        let root_files = vec![
            root_file_with_type(
                "/Users/example/dev/beta.txt",
                "beta.txt",
                FileType::Document,
            ),
            root_file_with_type(
                "/Users/example/dev/alpha.txt",
                "alpha.txt",
                FileType::Document,
            ),
        ];

        let (_grouped, flat) = get_grouped_results_with_validation_query_and_root_files(
            &[],
            &[],
            &[],
            &[],
            &[],
            &frecency_store,
            "~/dev/",
            &SuggestedConfig::default(),
            &[],
            None,
            None,
            None,
            None,
            Some(crate::file_search::RootFileSectionMode::DirectoryBrowse),
            false,
            &root_files,
            &[],
        );

        let rendered_files = flat
            .iter()
            .filter_map(|result| match result {
                SearchResult::File(file) => Some(file.file.name.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>();

        assert_eq!(
            rendered_files,
            vec!["beta.txt", "alpha.txt"],
            "directory browse should preserve provider order instead of fuzzy re-ranking"
        );
    }

    #[test]
    fn root_directory_browse_rows_filter_by_child_fragment() {
        let frecency_store = FrecencyStore::new();
        let root_files = vec![
            root_file_with_type(
                "/Users/example/dev/beta-notes.md",
                "beta-notes.md",
                FileType::Document,
            ),
            root_file_with_type(
                "/Users/example/dev/alpha-report.md",
                "alpha-report.md",
                FileType::Document,
            ),
            root_file_with_type(
                "/Users/example/dev/alpha-folder",
                "alpha-folder",
                FileType::Directory,
            ),
        ];

        let (_grouped, flat) = get_grouped_results_with_validation_query_and_root_files(
            &[],
            &[],
            &[],
            &[],
            &[],
            &frecency_store,
            "~/dev/al",
            &SuggestedConfig::default(),
            &[],
            None,
            None,
            None,
            None,
            Some(crate::file_search::RootFileSectionMode::DirectoryBrowse),
            false,
            &root_files,
            &[],
        );

        let rendered_files = flat
            .iter()
            .filter_map(|result| match result {
                SearchResult::File(file) => Some(file.file.name.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>();

        assert_eq!(
            rendered_files,
            vec!["alpha-folder", "alpha-report.md"],
            "directory browse child fragments should filter direct children inline"
        );
        assert!(
            flat.iter().any(|result| matches!(
                result,
                SearchResult::Fallback(fallback) if fallback.display_label() == "Open File Search in \"~/dev\""
            )),
            "filtered directory browse should keep the handoff scoped to the containing folder"
        );
    }

    #[test]
    fn empty_root_appends_recent_files_section() {
        let frecency_store = FrecencyStore::new();
        let recent_files = vec![root_file(
            "/Users/example/Desktop/recent design notes.md",
            "recent design notes.md",
        )];

        let (grouped, flat) = get_grouped_results_with_validation_query_and_root_files(
            &[],
            &[],
            &[],
            &[],
            &[],
            &frecency_store,
            "",
            &SuggestedConfig::default(),
            &[],
            None,
            None,
            None,
            None,
            None,
            false,
            &[],
            &recent_files,
        );

        assert!(
            grouped
                .iter()
                .any(|item| matches!(item, GroupedListItem::SectionHeader(label, None) if label == "Recent Files")),
            "empty root should append a Recent Files section"
        );
        assert!(
            flat.iter().any(|result| matches!(
                result,
                SearchResult::File(file) if file.file.path == "/Users/example/Desktop/recent design notes.md"
            )),
            "Recent Files should render real SearchResult::File rows"
        );
    }

    #[test]
    fn recent_files_insert_after_icon_suggested_section() {
        let mut grouped = vec![
            GroupedListItem::SectionHeader("Suggested".to_string(), Some("StarFilled".to_string())),
            GroupedListItem::Item(0),
            GroupedListItem::SectionHeader("Commands".to_string(), Some("Terminal".to_string())),
            GroupedListItem::Item(1),
        ];
        let mut flat = vec![
            SearchResult::File(crate::scripts::FileMatch {
                file: root_file("/Users/example/Desktop/suggested.txt", "suggested.txt"),
                score: 10,
            }),
            SearchResult::File(crate::scripts::FileMatch {
                file: root_file("/Users/example/Desktop/command.txt", "command.txt"),
                score: 9,
            }),
        ];
        let recent_files = vec![root_file(
            "/Users/example/Desktop/recent design notes.md",
            "recent design notes.md",
        )];

        append_recent_root_file_section(&mut grouped, &mut flat, &recent_files, "", None);

        assert!(
            matches!(&grouped[2], GroupedListItem::SectionHeader(label, None) if label == "Recent Files"),
            "Recent Files should insert after Suggested items and before Commands"
        );
    }

    #[test]
    fn non_empty_search_does_not_append_recent_files_section() {
        let frecency_store = FrecencyStore::new();
        let recent_files = vec![root_file(
            "/Users/example/Desktop/recent design notes.md",
            "recent design notes.md",
        )];

        let (grouped, flat) = get_grouped_results_with_validation_query_and_root_files(
            &[],
            &[],
            &[],
            &[],
            &[],
            &frecency_store,
            "recent",
            &SuggestedConfig::default(),
            &[],
            None,
            None,
            None,
            None,
            None,
            false,
            &[],
            &recent_files,
        );

        assert!(
            !grouped
                .iter()
                .any(|item| matches!(item, GroupedListItem::SectionHeader(label, None) if label == "Recent Files")),
            "non-empty root search should use Files, not Recent Files"
        );
        assert!(
            flat.iter()
                .all(|result| !matches!(result, SearchResult::File(_))),
            "recent files should not leak into non-empty root search"
        );
    }

    #[test]
    fn advanced_query_does_not_append_recent_files_section() {
        let frecency_store = FrecencyStore::new();
        let recent_files = vec![root_file(
            "/Users/example/Desktop/recent design notes.md",
            "recent design notes.md",
        )];
        let query = advanced_query_from(":type:file");

        let (grouped, flat) = get_grouped_results_with_validation_query_and_root_files(
            &[],
            &[],
            &[],
            &[],
            &[],
            &frecency_store,
            "",
            &SuggestedConfig::default(),
            &[],
            None,
            None,
            None,
            Some(&query),
            None,
            false,
            &[],
            &recent_files,
        );

        assert!(
            !grouped
                .iter()
                .any(|item| matches!(item, GroupedListItem::SectionHeader(label, None) if label == "Recent Files")),
            "advanced query mode should not mix in recent root files"
        );
        assert!(
            flat.iter()
                .all(|result| !matches!(result, SearchResult::File(_))),
            "advanced query mode should not append recent file rows"
        );
    }

    #[test]
    fn recent_files_do_not_create_search_files_handoff_row() {
        let frecency_store = FrecencyStore::new();
        let recent_files = vec![root_file(
            "/Users/example/Desktop/recent design notes.md",
            "recent design notes.md",
        )];

        let (_grouped, flat) = get_grouped_results_with_validation_query_and_root_files(
            &[],
            &[],
            &[],
            &[],
            &[],
            &frecency_store,
            "",
            &SuggestedConfig::default(),
            &[],
            None,
            None,
            None,
            None,
            None,
            false,
            &[],
            &recent_files,
        );

        assert!(
            flat.iter().all(|result| !matches!(
                result,
                SearchResult::Fallback(fallback) if fallback.display_label().starts_with("Search Files for")
            )),
            "empty recent file rows should not create a Search Files continuation row"
        );
    }
}

#[cfg(test)]
mod capture_mode_tests {
    use super::*;
    use crate::menu_syntax::{parse, CaptureInvocation, MenuSyntaxParse};
    use crate::metadata_parser::TypedMetadata;
    use serde_json::json;
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn capture_from(raw: &str) -> CaptureInvocation {
        match parse(raw) {
            MenuSyntaxParse::Capture(c) => c,
            other => panic!("expected Capture for {raw:?}, got {other:?}"),
        }
    }

    fn script_with_menu_syntax(name: &str, menu_syntax: serde_json::Value) -> Arc<Script> {
        let mut extra: HashMap<String, serde_json::Value> = HashMap::new();
        extra.insert("menuSyntax".to_string(), menu_syntax);
        let mut meta = TypedMetadata::default();
        meta.extra = extra;
        Arc::new(Script {
            name: name.to_string(),
            path: PathBuf::from(format!("/tmp/{name}.ts")),
            extension: "ts".to_string(),
            description: Some(format!("{name} description")),
            icon: None,
            alias: None,
            shortcut: None,
            typed_metadata: Some(meta),
            schema: None,
            plugin_id: "main".to_string(),
            plugin_title: None,
            kit_name: None,
            body: None,
        })
    }

    fn plain_script(name: &str) -> Arc<Script> {
        Arc::new(Script {
            name: name.to_string(),
            path: PathBuf::from(format!("/tmp/{name}.ts")),
            extension: "ts".to_string(),
            description: None,
            icon: None,
            alias: None,
            shortcut: None,
            typed_metadata: None,
            schema: None,
            plugin_id: "main".to_string(),
            plugin_title: None,
            kit_name: None,
            body: None,
        })
    }

    #[test]
    fn zero_handlers_returns_single_help_header_and_no_flat_results() {
        let invocation = capture_from(";todo Renew passport");
        let scripts: Vec<Arc<Script>> = vec![plain_script("unrelated")];
        let (grouped, flat) = build_capture_mode_results(&scripts, &invocation);
        assert_eq!(flat.len(), 0, "no selectable results");
        assert_eq!(grouped.len(), 1, "exactly one help header");
        match &grouped[0] {
            GroupedListItem::SectionHeader(label, None) => {
                assert!(
                    label.contains("capture.v1/todo"),
                    "help header must name the target, got {label:?}"
                );
                assert!(
                    label.contains("No scripts opted"),
                    "help header must explain why"
                );
            }
            other => panic!("expected SectionHeader, got {other:?}"),
        }
    }

    // @lat: menu-syntax Capture Handler Filtering
    #[test]
    fn only_opted_in_handlers_appear_and_shape_is_header_then_items() {
        let todo_handler = script_with_menu_syntax(
            "todo-handler",
            json!([
                { "family": "capture.v1", "targets": ["todo"] }
            ]),
        );
        let note_handler = script_with_menu_syntax(
            "note-handler",
            json!([
                { "family": "capture.v1", "targets": ["note"] }
            ]),
        );
        let wildcard_handler = script_with_menu_syntax(
            "wildcard-handler",
            json!([
                { "family": "capture.v1", "targets": ["*"] }
            ]),
        );
        let unrelated = plain_script("unrelated");

        let scripts = vec![todo_handler, note_handler, wildcard_handler, unrelated];
        let invocation = capture_from(";todo Renew passport");
        let (grouped, flat) = build_capture_mode_results(&scripts, &invocation);

        assert_eq!(
            flat.len(),
            2,
            "todo + wildcard must match, note and plain must not"
        );
        // First item in grouped must be the section header.
        match &grouped[0] {
            GroupedListItem::SectionHeader(label, None) => {
                assert_eq!(label, "Capture todo");
            }
            other => panic!("first grouped entry must be the capture header, got {other:?}"),
        }
        // The rest must be Item rows in index-order.
        for (expected_idx, entry) in grouped.iter().skip(1).enumerate() {
            match entry {
                GroupedListItem::Item(i) => assert_eq!(*i, expected_idx),
                other => panic!("expected Item({expected_idx}), got {other:?}"),
            }
        }
        let names: Vec<&str> = flat
            .iter()
            .filter_map(|r| match r {
                SearchResult::Script(sm) => Some(sm.script.name.as_str()),
                _ => None,
            })
            .collect();
        assert!(names.contains(&"todo-handler"));
        assert!(names.contains(&"wildcard-handler"));
        assert!(!names.contains(&"note-handler"));
        assert!(!names.contains(&"unrelated"));
    }

    #[test]
    fn non_capture_family_never_matches_even_if_targets_include_target() {
        let impostor = script_with_menu_syntax(
            "impostor",
            json!([
                { "family": "query.v1", "targets": ["todo"] }
            ]),
        );
        let scripts = vec![impostor];
        let invocation = capture_from(";todo Renew passport");
        let (_grouped, flat) = build_capture_mode_results(&scripts, &invocation);
        assert_eq!(
            flat.len(),
            0,
            "non-capture family must never match capture mode"
        );
    }

    #[test]
    fn keyword_alias_matches_same_handlers_as_plus_alias() {
        let handler = script_with_menu_syntax(
            "note-handler",
            json!([{ "family": "capture.v1", "targets": ["note"] }]),
        );
        let scripts = vec![handler];
        let plus = capture_from(";note buy batteries");
        let keyword = capture_from("note: buy batteries");
        let (_, flat_plus) = build_capture_mode_results(&scripts, &plus);
        let (_, flat_keyword) = build_capture_mode_results(&scripts, &keyword);
        assert_eq!(flat_plus.len(), 1);
        assert_eq!(flat_keyword.len(), 1);
    }

    #[test]
    fn incomplete_hint_row_is_single_non_selectable_header() {
        let (grouped, flat) =
            build_menu_syntax_hint_results("Type a capture target: todo, cal, note, social, link");
        assert!(
            flat.is_empty(),
            "incomplete rows never yield selectable results"
        );
        assert_eq!(grouped.len(), 1);
        match &grouped[0] {
            GroupedListItem::SectionHeader(label, None) => {
                assert!(label.contains("todo"));
            }
            other => panic!("expected SectionHeader, got {other:?}"),
        }
        for entry in grouped.iter() {
            assert!(
                !matches!(entry, GroupedListItem::Item(_)),
                "hint rows must never be Item entries (Item maps to a selectable flat result)"
            );
        }
    }

    // @lat: menu-syntax Parser Boundary
    #[test]
    fn menu_syntax_parse_incomplete_wires_into_hint_helper() {
        use crate::menu_syntax::MenuSyntaxParse;
        match parse("+") {
            MenuSyntaxParse::Incomplete(s) => {
                let (grouped, flat) = build_menu_syntax_hint_results(&s.hint);
                assert!(flat.is_empty());
                assert_eq!(grouped.len(), 1);
                let GroupedListItem::SectionHeader(label, None) = &grouped[0] else {
                    panic!("expected header")
                };
                assert_eq!(label, &s.hint);
            }
            other => panic!("expected Incomplete for '+' , got {other:?}"),
        }
    }

    #[test]
    fn every_result_carries_max_score_for_deterministic_order() {
        let a = script_with_menu_syntax(
            "a",
            json!([{ "family": "capture.v1", "targets": ["todo"] }]),
        );
        let b = script_with_menu_syntax(
            "b",
            json!([{ "family": "capture.v1", "targets": ["todo"] }]),
        );
        let scripts = vec![a, b];
        let invocation = capture_from(";todo something");
        let (_grouped, flat) = build_capture_mode_results(&scripts, &invocation);
        for r in flat {
            match r {
                SearchResult::Script(sm) => {
                    assert_eq!(sm.score, i32::MAX);
                    assert_eq!(sm.match_kind, ScriptMatchKind::Name);
                    assert!(sm.content_match.is_none());
                }
                other => panic!("expected Script, got {other:?}"),
            }
        }
    }
}
