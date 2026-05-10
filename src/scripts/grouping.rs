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
    MatchIndices, Script, ScriptIssueMatch, ScriptMatch, ScriptMatchKind, Scriptlet, SearchResult,
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
    root_file_results: &[crate::file_search::FileResult],
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
        root_file_results,
        filter_text,
        frecency_store,
        advanced_query,
    );

    (grouped, flat_results)
}

fn append_root_file_section(
    grouped: &mut Vec<GroupedListItem>,
    flat_results: &mut Vec<SearchResult>,
    root_file_results: &[crate::file_search::FileResult],
    filter_text: &str,
    frecency_store: &FrecencyStore,
    advanced_query: Option<&crate::menu_syntax::AdvancedQuery>,
) {
    if advanced_query.is_some() || !crate::file_search::should_search_root_files(filter_text) {
        return;
    }

    let files = crate::file_search::rank_root_file_results(
        root_file_results,
        filter_text,
        crate::file_search::ROOT_FILE_RENDER_LIMIT,
        |key| frecency_store.get_score(key),
    );
    if files.is_empty() {
        return;
    }

    grouped.push(GroupedListItem::SectionHeader("Files".to_string(), None));
    for file_match in files {
        let idx = flat_results.len();
        flat_results.push(SearchResult::File(file_match));
        grouped.push(GroupedListItem::Item(idx));
    }
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
    use crate::scripts::types::MatchIndices;

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
        FileResult {
            path: path.to_string(),
            name: name.to_string(),
            size: 0,
            modified: 0,
            file_type: FileType::Document,
        }
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
            &root_files,
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
            &root_files,
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
