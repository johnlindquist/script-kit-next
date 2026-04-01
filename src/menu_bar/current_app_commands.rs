//! Frontmost-menu snapshot contract for the "Current App Commands" built-in.
//!
//! Captures the menu bar of the frontmost (non–Script Kit) application as a
//! self-contained snapshot that can be converted into searchable `BuiltInEntry`
//! items without holding any platform handles.

use crate::builtins::BuiltInEntry;
use crate::menu_bar::MenuBarItem;

/// A point-in-time capture of the frontmost application's menu bar.
#[derive(Debug, Clone)]
pub struct FrontmostMenuSnapshot {
    /// Localised display name (e.g. "Safari").
    pub app_name: String,
    /// Bundle identifier (e.g. "com.apple.Safari").
    pub bundle_id: String,
    /// Top-level menu bar items with full hierarchy.
    pub items: Vec<MenuBarItem>,
}

/// A machine-readable receipt for a frontmost-menu snapshot capture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrontmostMenuSnapshotReceipt {
    pub app_name: String,
    pub bundle_id: String,
    pub top_level_menu_count: usize,
    pub leaf_entry_count: usize,
    pub placeholder: String,
    pub source: &'static str,
}

impl FrontmostMenuSnapshot {
    /// Convert the snapshot into flat, searchable built-in entries.
    ///
    /// Delegates to [`crate::builtins::menu_bar_items_to_entries`] which skips
    /// the Apple menu, separators, and disabled items.
    pub fn into_entries(self) -> Vec<BuiltInEntry> {
        self.into_entries_with_receipt().0
    }

    /// Convert the snapshot into entries and an audit-friendly receipt.
    pub fn into_entries_with_receipt(self) -> (Vec<BuiltInEntry>, FrontmostMenuSnapshotReceipt) {
        let entries = crate::builtins::menu_bar_items_to_entries(
            &self.items,
            &self.bundle_id,
            &self.app_name,
        );

        let receipt = FrontmostMenuSnapshotReceipt {
            app_name: self.app_name.clone(),
            bundle_id: self.bundle_id.clone(),
            top_level_menu_count: self.items.len(),
            leaf_entry_count: entries.len(),
            placeholder: self.placeholder(),
            source: "frontmost_app_tracker",
        };

        (entries, receipt)
    }

    /// Placeholder text for the filter input (e.g. "Search Safari commands…").
    pub fn placeholder(&self) -> String {
        format!("Search {} commands\u{2026}", self.app_name)
    }
}

// ---------------------------------------------------------------------------
// Label normalization
// ---------------------------------------------------------------------------

/// Generic helper for built-in entries whose label may appear in the main filter.
/// Returns `None` when the raw input is empty, whitespace-only, or matches the
/// built-in label (case-insensitive). Otherwise returns the trimmed input.
pub fn normalize_builtin_labeled_request<'a>(
    raw: Option<&'a str>,
    builtin_label: &str,
) -> Option<&'a str> {
    let raw = raw.map(str::trim).filter(|text| !text.is_empty())?;
    if raw.eq_ignore_ascii_case(builtin_label) {
        None
    } else {
        Some(raw)
    }
}

/// The human-readable label used in the main command list.
pub const GENERATE_SCRIPT_WITH_AI_LABEL: &str = "Generate Script with AI";

/// Returns `None` when the raw input is empty, whitespace-only, or matches the
/// built-in label (case-insensitive). Otherwise returns the trimmed input.
pub fn normalize_generate_script_request(raw: Option<&str>) -> Option<&str> {
    normalize_builtin_labeled_request(raw, GENERATE_SCRIPT_WITH_AI_LABEL)
}

/// The human-readable label used in the main command list.
pub const GENERATE_SCRIPT_FROM_CURRENT_APP_LABEL: &str = "Generate Script from Current App";

/// Returns `None` when the raw input is empty, whitespace-only, or matches the
/// built-in label (case-insensitive). Otherwise returns the trimmed input.
pub fn normalize_generate_script_from_current_app_request(raw: Option<&str>) -> Option<&str> {
    normalize_builtin_labeled_request(raw, GENERATE_SCRIPT_FROM_CURRENT_APP_LABEL)
}

// ---------------------------------------------------------------------------
// "Do in Current App" intent router
// ---------------------------------------------------------------------------

/// The human-readable label used in the main command list.
pub const DO_IN_CURRENT_APP_LABEL: &str = "Do in Current App";

/// The action selected by the "Do in Current App" intent router.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DoInCurrentAppAction {
    OpenCommandPalette,
    ExecuteEntry(usize),
    GenerateScript,
}

/// A machine-readable receipt for the "Do in Current App" router.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoInCurrentAppReceipt {
    pub normalized_query: String,
    pub filtered_entries: usize,
    pub exact_matches: usize,
    pub action: &'static str,
}

/// Compatibility-only label kept so old phrases like
/// "Current App Commands: Close Duplicate Tabs" still normalize through the
/// primary current-app router after the visible command is removed.
pub const CURRENT_APP_COMMANDS_COMPAT_LABEL: &str = "Current App Commands";

/// Returns the effective user request for the primary current-app command.
///
/// Behavior:
/// - empty / whitespace-only -> None
/// - exact "Do in Current App" -> None
/// - exact "Current App Commands" -> None
/// - "Do in Current App <request>" -> Some("<request>")
/// - "Current App Commands <request>" -> Some("<request>")
/// - anything else -> Some(trimmed raw input)
pub fn normalize_do_in_current_app_request(raw: Option<&str>) -> Option<&str> {
    tracing::info!(raw_input = ?raw, "do_in_current_app.normalize_request.entry");
    let raw = raw.map(str::trim).filter(|text| !text.is_empty())?;

    let raw_lower = raw.to_ascii_lowercase();

    for label in [DO_IN_CURRENT_APP_LABEL, CURRENT_APP_COMMANDS_COMPAT_LABEL] {
        let label_lower = label.to_ascii_lowercase();

        if raw_lower == label_lower {
            tracing::info!(
                raw = %raw,
                label = %label,
                "do_in_current_app.normalize_request → None (matches label, treated as empty)"
            );
            return None;
        }

        if raw_lower.starts_with(&label_lower) {
            let rest = raw[label.len()..]
                .trim_start_matches(|ch: char| {
                    ch.is_ascii_whitespace()
                        || matches!(ch, ':' | '-' | '\u{2014}' | '\u{2013}')
                })
                .trim();

            if rest.is_empty() {
                tracing::info!(
                    raw = %raw,
                    label = %label,
                    "do_in_current_app.normalize_request → None (label prefix with empty query)"
                );
                return None;
            }

            tracing::info!(
                raw = %raw,
                label = %label,
                rest = %rest,
                "do_in_current_app.normalize_request → Some (stripped labeled query)"
            );
            return Some(rest);
        }
    }

    tracing::info!(raw = %raw, "do_in_current_app.normalize_request → Some (real query)");
    Some(raw)
}

fn normalize_intent_match_text(text: &str) -> String {
    let mut normalized = String::with_capacity(text.len());
    let mut last_was_space = false;

    for ch in text.chars() {
        let ch = if ch == '→' { ' ' } else { ch };

        if ch.is_ascii_alphanumeric() {
            normalized.push(ch.to_ascii_lowercase());
            last_was_space = false;
        } else if !last_was_space {
            normalized.push(' ');
            last_was_space = true;
        }
    }

    normalized.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn entry_exactly_matches_query(entry: &BuiltInEntry, normalized_query: &str) -> bool {
    if normalized_query.is_empty() {
        return false;
    }

    if normalize_intent_match_text(entry.leaf_name()) == normalized_query {
        return true;
    }

    if normalize_intent_match_text(&entry.name) == normalized_query {
        return true;
    }

    entry
        .keywords
        .iter()
        .any(|keyword| normalize_intent_match_text(keyword) == normalized_query)
}

/// Resolve a free-text request against the current app's menu entries.
pub fn resolve_do_in_current_app_intent(
    entries: &[BuiltInEntry],
    raw_query: Option<&str>,
) -> (DoInCurrentAppAction, DoInCurrentAppReceipt) {
    let normalized_query = normalize_do_in_current_app_request(raw_query)
        .map(normalize_intent_match_text)
        .unwrap_or_default();

    if normalized_query.is_empty() {
        tracing::info!(
            total_entries = entries.len(),
            "do_in_current_app.resolve → OpenCommandPalette (empty query, showing all entries)"
        );
        return (
            DoInCurrentAppAction::OpenCommandPalette,
            DoInCurrentAppReceipt {
                normalized_query,
                filtered_entries: entries.len(),
                exact_matches: 0,
                action: "open_command_palette",
            },
        );
    }

    let filtered: Vec<(usize, &BuiltInEntry)> = entries
        .iter()
        .enumerate()
        .filter(|(_, entry)| {
            crate::builtins::menu_bar_entry_matches_query(entry, &normalized_query)
        })
        .collect();

    let exact_matches: Vec<usize> = filtered
        .iter()
        .filter(|(_, entry)| entry_exactly_matches_query(entry, &normalized_query))
        .map(|(idx, _)| *idx)
        .collect();

    tracing::info!(
        normalized_query = %normalized_query,
        total_entries = entries.len(),
        filtered_count = filtered.len(),
        exact_match_count = exact_matches.len(),
        "do_in_current_app.resolve.match_results"
    );

    let (action, action_name) = if exact_matches.len() == 1 {
        tracing::info!(
            entry_index = exact_matches[0],
            entry_name = %entries[exact_matches[0]].name,
            "do_in_current_app.resolve → ExecuteEntry (single exact match)"
        );
        (
            DoInCurrentAppAction::ExecuteEntry(exact_matches[0]),
            "execute_entry",
        )
    } else if filtered.is_empty() {
        tracing::info!("do_in_current_app.resolve → GenerateScript (no menu matches)");
        (DoInCurrentAppAction::GenerateScript, "generate_script")
    } else {
        tracing::info!(
            filtered_count = filtered.len(),
            exact_match_count = exact_matches.len(),
            "do_in_current_app.resolve → OpenCommandPalette (multiple matches, no single exact)"
        );
        (
            DoInCurrentAppAction::OpenCommandPalette,
            "open_command_palette",
        )
    };

    (
        action,
        DoInCurrentAppReceipt {
            normalized_query,
            filtered_entries: filtered.len(),
            exact_matches: exact_matches.len(),
            action: action_name,
        },
    )
}

// ---------------------------------------------------------------------------
// Script prompt builder
// ---------------------------------------------------------------------------

/// Maximum number of menu items to include in AI script-generation prompts.
const MAX_SCRIPT_PROMPT_MENU_ITEMS: usize = 20;

/// Maximum number of characters of selected text to include in the prompt.
const MAX_SELECTED_TEXT_CHARS: usize = 1_500;

/// A machine-readable receipt for a script-generation prompt built from the
/// frontmost app snapshot.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CurrentAppScriptPromptReceipt {
    pub app_name: String,
    pub bundle_id: String,
    pub total_menu_items: usize,
    pub included_menu_items: usize,
    pub included_user_request: bool,
    pub included_selected_text: bool,
    pub included_browser_url: bool,
}

/// Build a deterministic AI prompt for script generation from a frontmost-app
/// snapshot plus optional user request, selected text, and browser URL.
///
/// Returns the assembled prompt string and a structured receipt for logging.
/// This function is side-effect free and reuses `snapshot.into_entries_with_receipt()`
/// rather than re-walking platform state.
pub fn build_generate_script_prompt_from_snapshot(
    snapshot: FrontmostMenuSnapshot,
    user_request: Option<&str>,
    selected_text: Option<&str>,
    browser_url: Option<&str>,
) -> (String, CurrentAppScriptPromptReceipt) {
    let (entries, snapshot_receipt) = snapshot.into_entries_with_receipt();

    let user_request = user_request.map(str::trim).filter(|text| !text.is_empty());

    let selected_text = selected_text.map(str::trim).filter(|text| !text.is_empty());

    let browser_url = browser_url.map(str::trim).filter(|url| !url.is_empty());

    let menu_lines: Vec<String> = entries
        .iter()
        .take(MAX_SCRIPT_PROMPT_MENU_ITEMS)
        .map(|entry| {
            let shortcut_suffix = match &entry.feature {
                crate::builtins::BuiltInFeature::MenuBarAction(info) => info
                    .shortcut
                    .as_ref()
                    .map(|shortcut| format!(" ({shortcut})"))
                    .unwrap_or_default(),
                _ => String::new(),
            };
            format!("- {}{}", entry.name, shortcut_suffix)
        })
        .collect();

    let mut sections = Vec::new();
    sections.push(
        "Generate a Script Kit script that automates what I am doing in the current app."
            .to_string(),
    );

    if let Some(request) = user_request {
        sections.push(format!("User Request:\n{}", request));
    }

    sections.push(format!("Frontmost App: {}", snapshot_receipt.app_name));
    sections.push(format!("Bundle ID: {}", snapshot_receipt.bundle_id));

    if !menu_lines.is_empty() {
        sections.push(format!(
            "Enabled Menu Commands (showing {} of {}):\n{}",
            menu_lines.len(),
            snapshot_receipt.leaf_entry_count,
            menu_lines.join("\n")
        ));
    }

    if let Some(text) = selected_text {
        let truncated: String = text.chars().take(MAX_SELECTED_TEXT_CHARS).collect();
        sections.push(format!("Selected Text:\n```text\n{}\n```", truncated));
    }

    if let Some(url) = browser_url {
        sections.push(format!("Focused Browser URL:\n{}", url));
    }

    sections.push(
        "Requirements:\n\
         - Prefer the smallest useful working script.\n\
         - Reuse existing app or menu semantics when possible.\n\
         - Call out required permissions.\n\
         - If the task is ambiguous, pick the safest reasonable default and say what you assumed."
            .to_string(),
    );

    let prompt = sections.join("\n\n");

    let receipt = CurrentAppScriptPromptReceipt {
        app_name: snapshot_receipt.app_name,
        bundle_id: snapshot_receipt.bundle_id,
        total_menu_items: snapshot_receipt.leaf_entry_count,
        included_menu_items: menu_lines.len(),
        included_user_request: user_request.is_some(),
        included_selected_text: selected_text.is_some(),
        included_browser_url: browser_url.is_some(),
    };

    (prompt, receipt)
}

// ---------------------------------------------------------------------------
// Trace Current App Intent
// ---------------------------------------------------------------------------

/// The human-readable label used in the main command list.
pub const TRACE_CURRENT_APP_INTENT_LABEL: &str = "Trace Current App Intent";

/// Returns the effective user request for the trace command.
///
/// Behavior:
/// - empty / whitespace-only -> None
/// - exact label -> None
/// - "Trace Current App Intent <request>" -> Some("<request>")
/// - anything else -> Some(trimmed raw input)
pub fn normalize_trace_current_app_intent_request(raw: Option<&str>) -> Option<String> {
    let raw = raw.map(str::trim).filter(|text| !text.is_empty())?;
    let label = TRACE_CURRENT_APP_INTENT_LABEL;
    let raw_lower = raw.to_ascii_lowercase();
    let label_lower = label.to_ascii_lowercase();

    if raw_lower == label_lower {
        return None;
    }

    if raw_lower.starts_with(&label_lower) {
        let rest = raw[label.len()..]
            .trim_start_matches(|ch: char| {
                ch.is_ascii_whitespace() || matches!(ch, ':' | '-' | '\u{2014}' | '\u{2013}')
            })
            .trim();

        if rest.is_empty() {
            None
        } else {
            Some(rest.to_string())
        }
    } else {
        Some(raw.to_string())
    }
}

/// Schema version for [`CurrentAppIntentTraceReceipt`].
pub const CURRENT_APP_INTENT_TRACE_SCHEMA_VERSION: u32 = 1;

/// A compact candidate entry included in [`CurrentAppIntentTraceReceipt`].
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CurrentAppIntentTraceCandidate {
    pub entry_id: String,
    pub name: String,
    pub leaf_name: String,
    pub shortcut: Option<String>,
}

/// A machine-readable dry-run receipt for a free-text current-app request.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CurrentAppIntentTraceReceipt {
    pub schema_version: u32,
    pub source: String,
    pub app_name: String,
    pub bundle_id: String,
    pub raw_query: String,
    pub effective_query: String,
    pub normalized_query: String,
    pub top_level_menu_count: usize,
    pub leaf_entry_count: usize,
    pub filtered_entries: usize,
    pub exact_matches: usize,
    pub action: String,
    pub selected_entry: Option<CurrentAppIntentTraceCandidate>,
    pub candidates: Vec<CurrentAppIntentTraceCandidate>,
    pub prompt_receipt: Option<CurrentAppScriptPromptReceipt>,
    pub prompt_preview: Option<String>,
}

fn intent_trace_candidate(entry: &BuiltInEntry) -> CurrentAppIntentTraceCandidate {
    let shortcut = match &entry.feature {
        crate::builtins::BuiltInFeature::MenuBarAction(info) => info.shortcut.clone(),
        _ => None,
    };

    CurrentAppIntentTraceCandidate {
        entry_id: entry.id.clone(),
        name: entry.name.clone(),
        leaf_name: entry.leaf_name().to_string(),
        shortcut,
    }
}

/// Build a deterministic dry-run trace for how a current-app request would resolve.
///
/// This function never executes the chosen command. It only reports:
/// - normalized input
/// - candidate menu matches
/// - final router action
/// - script prompt preview/receipt when the router would fall back to AI generation
pub fn build_current_app_intent_trace_receipt(
    snapshot: FrontmostMenuSnapshot,
    raw_query: Option<&str>,
) -> CurrentAppIntentTraceReceipt {
    let raw_query_string = raw_query.unwrap_or_default().to_string();
    let effective_query = normalize_trace_current_app_intent_request(raw_query).unwrap_or_default();

    let (entries, snapshot_receipt) = snapshot.clone().into_entries_with_receipt();
    let (action, intent_receipt) =
        resolve_do_in_current_app_intent(&entries, Some(effective_query.as_str()));

    let filtered: Vec<&BuiltInEntry> = entries
        .iter()
        .filter(|entry| {
            crate::builtins::menu_bar_entry_matches_query(
                entry,
                intent_receipt.normalized_query.as_str(),
            )
        })
        .collect();

    let candidates = filtered
        .iter()
        .take(8)
        .map(|entry| intent_trace_candidate(entry))
        .collect::<Vec<_>>();

    let selected_entry = match &action {
        DoInCurrentAppAction::ExecuteEntry(idx) => entries.get(*idx).map(intent_trace_candidate),
        DoInCurrentAppAction::OpenCommandPalette | DoInCurrentAppAction::GenerateScript => None,
    };

    let (prompt_receipt, prompt_preview) = match &action {
        DoInCurrentAppAction::GenerateScript => {
            let request = (!effective_query.is_empty()).then_some(effective_query.as_str());
            let (prompt, receipt) =
                build_generate_script_prompt_from_snapshot(snapshot, request, None, None);
            (Some(receipt), Some(prompt))
        }
        DoInCurrentAppAction::OpenCommandPalette | DoInCurrentAppAction::ExecuteEntry(_) => {
            (None, None)
        }
    };

    CurrentAppIntentTraceReceipt {
        schema_version: CURRENT_APP_INTENT_TRACE_SCHEMA_VERSION,
        source: snapshot_receipt.source.to_string(),
        app_name: snapshot_receipt.app_name,
        bundle_id: snapshot_receipt.bundle_id,
        raw_query: raw_query_string,
        effective_query,
        normalized_query: intent_receipt.normalized_query,
        top_level_menu_count: snapshot_receipt.top_level_menu_count,
        leaf_entry_count: snapshot_receipt.leaf_entry_count,
        filtered_entries: intent_receipt.filtered_entries,
        exact_matches: intent_receipt.exact_matches,
        action: intent_receipt.action.to_string(),
        selected_entry,
        candidates,
        prompt_receipt,
        prompt_preview,
    }
}

// ---------------------------------------------------------------------------
// Turn This Into a Command
// ---------------------------------------------------------------------------

/// The human-readable label used in the main command list.
pub const TURN_THIS_INTO_A_COMMAND_LABEL: &str = "Turn This Into a Command";

/// Schema version for [`CurrentAppCommandRecipe`].
pub const CURRENT_APP_COMMAND_RECIPE_SCHEMA_VERSION: u32 = 1;

/// A machine-readable recipe that packages current-app routing + prompt state
/// for later script creation or auditing.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrentAppCommandRecipe {
    pub schema_version: u32,
    pub recipe_type: String,
    pub raw_query: String,
    pub effective_query: String,
    pub suggested_script_name: String,
    pub trace: CurrentAppIntentTraceReceipt,
    pub prompt_receipt: CurrentAppScriptPromptReceipt,
    pub prompt: String,
}

/// Returns the effective user request for the "Turn This Into a Command" command.
///
/// Behavior:
/// - empty / whitespace-only -> None
/// - exact label -> None
/// - "Turn This Into a Command <request>" -> Some("<request>")
/// - anything else -> Some(trimmed raw input)
pub fn normalize_turn_this_into_a_command_request(raw: Option<&str>) -> Option<String> {
    let raw = raw.map(str::trim).filter(|text| !text.is_empty())?;
    let label = TURN_THIS_INTO_A_COMMAND_LABEL;
    let raw_lower = raw.to_ascii_lowercase();
    let label_lower = label.to_ascii_lowercase();

    if raw_lower == label_lower {
        return None;
    }

    if raw_lower.starts_with(&label_lower) {
        let rest = raw[label.len()..]
            .trim_start_matches(|ch: char| {
                ch.is_ascii_whitespace() || matches!(ch, ':' | '-' | '\u{2014}' | '\u{2013}')
            })
            .trim();

        if rest.is_empty() {
            None
        } else {
            Some(rest.to_string())
        }
    } else {
        Some(raw.to_string())
    }
}

fn title_case_words(text: &str) -> String {
    text.split_whitespace()
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => {
                    let mut word = String::new();
                    word.extend(first.to_uppercase());
                    word.push_str(chars.as_str());
                    word
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Suggest a stable, human-readable script name from app + request.
pub fn suggest_current_app_command_name(app_name: &str, raw_request: &str) -> String {
    let normalized = normalize_intent_match_text(raw_request);
    if normalized.is_empty() {
        return format!("{app_name} Command");
    }

    format!("{app_name} {}", title_case_words(&normalized))
}

/// Build a deterministic recipe for turning a current-app request into a reusable command.
///
/// This does not execute anything. It packages:
/// - a normalized request
/// - a route trace
/// - a generation prompt
/// - a suggested script name
pub fn build_current_app_command_recipe(
    snapshot: FrontmostMenuSnapshot,
    raw_query: Option<&str>,
    selected_text: Option<&str>,
    browser_url: Option<&str>,
) -> CurrentAppCommandRecipe {
    let raw_query_string = raw_query.unwrap_or_default().to_string();
    let effective_query = normalize_turn_this_into_a_command_request(raw_query).unwrap_or_default();

    let request = (!effective_query.is_empty()).then_some(effective_query.as_str());

    let mut trace = build_current_app_intent_trace_receipt(snapshot.clone(), request);
    trace.raw_query = raw_query_string.clone();

    let (prompt, prompt_receipt) =
        build_generate_script_prompt_from_snapshot(snapshot, request, selected_text, browser_url);

    // Keep the nested trace aligned with the actual prompt carried by the recipe.
    // This matters when selected_text/browser_url are present.
    if trace.action == "generate_script" {
        trace.prompt_receipt = Some(prompt_receipt.clone());
        trace.prompt_preview = Some(prompt.clone());
    }

    CurrentAppCommandRecipe {
        schema_version: CURRENT_APP_COMMAND_RECIPE_SCHEMA_VERSION,
        recipe_type: "currentAppCommand".to_string(),
        raw_query: raw_query_string,
        effective_query: effective_query.clone(),
        suggested_script_name: suggest_current_app_command_name(
            &prompt_receipt.app_name,
            &effective_query,
        ),
        trace,
        prompt_receipt,
        prompt,
    }
}

// ---------------------------------------------------------------------------
// Recipe → generation prompt
// ---------------------------------------------------------------------------

/// Builds a generation prompt from a pre-built recipe, embedding the recipe as
/// a base64-encoded header so the generated script can be round-tripped back to
/// recipe tooling (verify, replay) later.
pub fn build_generated_script_prompt_from_recipe(recipe: &CurrentAppCommandRecipe) -> String {
    use base64::Engine as _;

    let recipe_json = match serde_json::to_string_pretty(recipe) {
        Ok(json) => json,
        Err(error) => {
            tracing::warn!(
                error = %error,
                "current_app_recipe.embed_serialize_failed"
            );
            "{}".to_string()
        }
    };

    let recipe_base64 = base64::engine::general_purpose::STANDARD.encode(recipe_json.as_bytes());

    format!(
        "{prompt}\n\n\
         OUTPUT CONTRACT:\n\
         - Return only runnable Script Kit TypeScript.\n\
         - Bias toward direct menu-command automation before brittle click/coordinate automation.\n\
         - Keep the script as small as possible.\n\
         - Put these exact header lines at the top of the generated file:\n\
         \x20 // Current-App-Recipe-Base64: {recipe_base64}\n\
         \x20 // Current-App-Recipe-Name: {script_name}\n\
         - If Accessibility is required, say so in a comment near the top.\n\
         - If the task needs AI, isolate that in one function instead of spreading it across the file.\n\
         - Do not include prose outside the code.",
        prompt = recipe.prompt,
        script_name = recipe.suggested_script_name,
    )
}

// ---------------------------------------------------------------------------
// Verify Current App Recipe
// ---------------------------------------------------------------------------

/// The human-readable label used in the main command list.
pub const VERIFY_CURRENT_APP_RECIPE_LABEL: &str = "Verify Current App Recipe";

/// Schema version for [`CurrentAppCommandRecipeVerification`].
pub const CURRENT_APP_COMMAND_RECIPE_VERIFICATION_SCHEMA_VERSION: u32 = 1;

/// A machine-readable drift report comparing a stored recipe against live context.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrentAppCommandRecipeVerification {
    pub schema_version: u32,
    pub verification_type: String,
    pub status: String,
    pub expected_app_name: String,
    pub actual_app_name: String,
    pub expected_bundle_id: String,
    pub actual_bundle_id: String,
    pub expected_effective_query: String,
    pub actual_effective_query: String,
    pub expected_route: String,
    pub actual_route: String,
    pub app_name_matches: bool,
    pub bundle_id_matches: bool,
    pub effective_query_matches: bool,
    pub route_matches: bool,
    pub prompt_matches: bool,
    pub selected_text_expected: bool,
    pub selected_text_present: bool,
    pub browser_url_expected: bool,
    pub browser_url_present: bool,
    pub warning_count: usize,
    pub warnings: Vec<String>,
    pub live_recipe: CurrentAppCommandRecipe,
}

/// Parse a JSON string into a [`CurrentAppCommandRecipe`], returning
/// descriptive errors for empty input, invalid JSON, wrong recipe type,
/// and unsupported schema versions.
pub fn parse_current_app_command_recipe_json(
    input: &str,
) -> Result<CurrentAppCommandRecipe, String> {
    let trimmed = input.trim();

    if trimmed.is_empty() {
        return Err(
            "Clipboard is empty. Run \"Turn This Into a Command\" first, then try again."
                .to_string(),
        );
    }

    let value: serde_json::Value = serde_json::from_str(trimmed)
        .map_err(|e| format!("Clipboard does not contain valid JSON: {e}"))?;

    let recipe_type = value
        .get("recipeType")
        .and_then(|v| v.as_str())
        .unwrap_or_default();

    if recipe_type != "currentAppCommand" {
        return Err(format!(
            "Clipboard JSON has recipeType={recipe_type:?}. Expected \"currentAppCommand\" from \"Turn This Into a Command\"."
        ));
    }

    let recipe: CurrentAppCommandRecipe = serde_json::from_value(value)
        .map_err(|e| format!("Clipboard JSON is not a valid currentAppCommand recipe: {e}"))?;

    if recipe.schema_version != CURRENT_APP_COMMAND_RECIPE_SCHEMA_VERSION {
        return Err(format!(
            "Unsupported recipe schema_version {}. Expected {}.",
            recipe.schema_version, CURRENT_APP_COMMAND_RECIPE_SCHEMA_VERSION
        ));
    }

    Ok(recipe)
}

/// Load a [`CurrentAppCommandRecipe`] from the system clipboard (macOS only).
#[cfg(target_os = "macos")]
pub fn load_current_app_command_recipe_from_clipboard() -> Result<CurrentAppCommandRecipe, String> {
    let output = std::process::Command::new("pbpaste")
        .output()
        .map_err(|e| format!("Failed to read clipboard with pbpaste: {e}"))?;

    if !output.status.success() {
        return Err(format!("pbpaste exited with status {}.", output.status));
    }

    let clipboard_text = String::from_utf8(output.stdout)
        .map_err(|e| format!("Clipboard text is not valid UTF-8: {e}"))?;

    parse_current_app_command_recipe_json(&clipboard_text)
}

/// Stub for non-macOS platforms — always returns an error.
#[cfg(not(target_os = "macos"))]
pub fn load_current_app_command_recipe_from_clipboard() -> Result<CurrentAppCommandRecipe, String> {
    Err("Verify Current App Recipe is only supported on macOS.".to_string())
}

/// Compare a stored recipe against a live frontmost-app snapshot and optional
/// context, returning a deterministic drift report.
pub fn verify_current_app_command_recipe(
    stored_recipe: &CurrentAppCommandRecipe,
    snapshot: FrontmostMenuSnapshot,
    selected_text: Option<&str>,
    browser_url: Option<&str>,
) -> CurrentAppCommandRecipeVerification {
    let replay_query = if stored_recipe.raw_query.trim().is_empty() {
        stored_recipe.effective_query.as_str()
    } else {
        stored_recipe.raw_query.as_str()
    };

    let live_recipe =
        build_current_app_command_recipe(snapshot, Some(replay_query), selected_text, browser_url);

    let app_name_matches =
        stored_recipe.prompt_receipt.app_name == live_recipe.prompt_receipt.app_name;
    let bundle_id_matches =
        stored_recipe.prompt_receipt.bundle_id == live_recipe.prompt_receipt.bundle_id;
    let effective_query_matches = stored_recipe.effective_query == live_recipe.effective_query;
    let route_matches = stored_recipe.trace.action == live_recipe.trace.action;
    let prompt_matches = stored_recipe.prompt == live_recipe.prompt;

    let selected_text_present = selected_text
        .map(str::trim)
        .map(|text| !text.is_empty())
        .unwrap_or(false);

    let browser_url_present = browser_url
        .map(str::trim)
        .map(|url| !url.is_empty())
        .unwrap_or(false);

    let selected_text_expected = stored_recipe.prompt_receipt.included_selected_text;
    let browser_url_expected = stored_recipe.prompt_receipt.included_browser_url;

    let mut warnings = Vec::new();

    if !app_name_matches {
        warnings.push(format!(
            "Recipe expected app {:?}, but the current app is {:?}.",
            stored_recipe.prompt_receipt.app_name, live_recipe.prompt_receipt.app_name,
        ));
    }

    if !bundle_id_matches {
        warnings.push(format!(
            "Recipe expected bundle_id {:?}, but the current bundle_id is {:?}.",
            stored_recipe.prompt_receipt.bundle_id, live_recipe.prompt_receipt.bundle_id,
        ));
    }

    if !effective_query_matches {
        warnings.push(format!(
            "Recipe effective_query changed from {:?} to {:?}.",
            stored_recipe.effective_query, live_recipe.effective_query,
        ));
    }

    if !route_matches {
        warnings.push(format!(
            "Recipe route changed from {:?} to {:?}.",
            stored_recipe.trace.action, live_recipe.trace.action,
        ));
    }

    if selected_text_expected && !selected_text_present {
        warnings.push(
            "Recipe expected selected text, but no selected text is available in the current context."
                .to_string(),
        );
    }

    if browser_url_expected && !browser_url_present {
        warnings.push(
            "Recipe expected a focused browser URL, but no browser URL is available in the current context."
                .to_string(),
        );
    }

    if !prompt_matches {
        warnings.push(
            "The regenerated prompt does not exactly match the stored recipe prompt.".to_string(),
        );
    }

    let status = if warnings.is_empty() {
        "match".to_string()
    } else {
        "drift".to_string()
    };

    CurrentAppCommandRecipeVerification {
        schema_version: CURRENT_APP_COMMAND_RECIPE_VERIFICATION_SCHEMA_VERSION,
        verification_type: "currentAppCommandVerification".to_string(),
        status,
        expected_app_name: stored_recipe.prompt_receipt.app_name.clone(),
        actual_app_name: live_recipe.prompt_receipt.app_name.clone(),
        expected_bundle_id: stored_recipe.prompt_receipt.bundle_id.clone(),
        actual_bundle_id: live_recipe.prompt_receipt.bundle_id.clone(),
        expected_effective_query: stored_recipe.effective_query.clone(),
        actual_effective_query: live_recipe.effective_query.clone(),
        expected_route: stored_recipe.trace.action.clone(),
        actual_route: live_recipe.trace.action.clone(),
        app_name_matches,
        bundle_id_matches,
        effective_query_matches,
        route_matches,
        prompt_matches,
        selected_text_expected,
        selected_text_present,
        browser_url_expected,
        browser_url_present,
        warning_count: warnings.len(),
        warnings,
        live_recipe,
    }
}

/// Build a human-readable HUD message from a verification result.
pub fn build_current_app_command_verification_hud_message(
    verification: &CurrentAppCommandRecipeVerification,
) -> String {
    if verification.warning_count == 0 {
        format!(
            "Recipe verified: {}",
            verification.live_recipe.suggested_script_name
        )
    } else {
        format!(
            "Recipe drift detected: {} warning{}",
            verification.warning_count,
            if verification.warning_count == 1 {
                ""
            } else {
                "s"
            }
        )
    }
}

// ---------------------------------------------------------------------------
// Replay Current App Recipe
// ---------------------------------------------------------------------------

/// The human-readable label used in the main command list.
pub const REPLAY_CURRENT_APP_RECIPE_LABEL: &str = "Replay Current App Recipe";

/// Schema version for [`ReplayCurrentAppRecipeReceipt`].
pub const REPLAY_CURRENT_APP_RECIPE_SCHEMA_VERSION: u32 = 1;

/// A machine-readable replay receipt for a current-app recipe.
///
/// This receipt is intentionally pure data:
/// - `verification` says whether the stored recipe still matches live context
/// - `action` says what replay would do when the recipe is safe to run
/// - `selected_entry_index` points back into the live `entries` slice owned by the caller
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplayCurrentAppRecipeReceipt {
    pub schema_version: u32,
    pub replay_type: String,
    pub action: String,
    pub selected_entry_index: Option<usize>,
    pub selected_entry: Option<CurrentAppIntentTraceCandidate>,
    pub verification: CurrentAppCommandRecipeVerification,
}

/// Build a deterministic replay receipt for a stored current-app recipe.
///
/// Behavior:
/// - if verification reports any drift, action becomes `blocked_by_drift`
/// - otherwise, re-resolve the live entries using the verified live recipe
///   and report one of:
///   - `execute_entry`
///   - `open_command_palette`
///   - `generate_script`
pub fn build_replay_current_app_recipe_receipt(
    stored_recipe: &CurrentAppCommandRecipe,
    entries: &[BuiltInEntry],
    snapshot: FrontmostMenuSnapshot,
    selected_text: Option<&str>,
    browser_url: Option<&str>,
) -> ReplayCurrentAppRecipeReceipt {
    let verification =
        verify_current_app_command_recipe(stored_recipe, snapshot, selected_text, browser_url);

    if verification.warning_count > 0 {
        return ReplayCurrentAppRecipeReceipt {
            schema_version: REPLAY_CURRENT_APP_RECIPE_SCHEMA_VERSION,
            replay_type: "currentAppRecipeReplay".to_string(),
            action: "blocked_by_drift".to_string(),
            selected_entry_index: None,
            selected_entry: None,
            verification,
        };
    }

    let (action, _) = resolve_do_in_current_app_intent(
        entries,
        Some(verification.live_recipe.effective_query.as_str()),
    );

    let (action_name, selected_entry_index, selected_entry) = match action {
        DoInCurrentAppAction::ExecuteEntry(idx) => (
            "execute_entry".to_string(),
            Some(idx),
            entries.get(idx).map(intent_trace_candidate),
        ),
        DoInCurrentAppAction::OpenCommandPalette => {
            ("open_command_palette".to_string(), None, None)
        }
        DoInCurrentAppAction::GenerateScript => ("generate_script".to_string(), None, None),
    };

    ReplayCurrentAppRecipeReceipt {
        schema_version: REPLAY_CURRENT_APP_RECIPE_SCHEMA_VERSION,
        replay_type: "currentAppRecipeReplay".to_string(),
        action: action_name,
        selected_entry_index,
        selected_entry,
        verification,
    }
}

/// Build a human-readable status message from a replay receipt.
pub fn build_replay_current_app_recipe_hud_message(
    receipt: &ReplayCurrentAppRecipeReceipt,
) -> String {
    match receipt.action.as_str() {
        "blocked_by_drift" => format!(
            "Recipe drift detected: {} warning{}",
            receipt.verification.warning_count,
            if receipt.verification.warning_count == 1 {
                ""
            } else {
                "s"
            }
        ),
        "execute_entry" => receipt
            .selected_entry
            .as_ref()
            .map(|entry| format!("Replayed recipe: {}", entry.leaf_name))
            .unwrap_or_else(|| "Replayed current app recipe".to_string()),
        "open_command_palette" => {
            let filter = receipt.verification.live_recipe.effective_query.trim();
            if filter.is_empty() {
                "Opened Current App Commands".to_string()
            } else {
                format!("Opened Current App Commands: {}", filter)
            }
        }
        "generate_script" => format!(
            "Replayed recipe into script: {}",
            receipt.verification.live_recipe.suggested_script_name
        ),
        _ => "Replayed current app recipe".to_string(),
    }
}

// ---------------------------------------------------------------------------
// Platform loader
// ---------------------------------------------------------------------------

/// Load a [`FrontmostMenuSnapshot`] from the current frontmost application.
///
/// On macOS this reads the pre-cached menu items from the frontmost-app tracker.
/// On other platforms it returns a deterministic "unsupported" error.
#[cfg(target_os = "macos")]
pub fn load_frontmost_menu_snapshot() -> anyhow::Result<FrontmostMenuSnapshot> {
    use anyhow::Context;

    let tracked_app = crate::frontmost_app_tracker::get_last_real_app()
        .context("No frontmost application tracked — is the app tracker running?")?;

    let items = crate::frontmost_app_tracker::get_cached_menu_items();

    tracing::info!(
        app_name = %tracked_app.name,
        bundle_id = %tracked_app.bundle_id,
        item_count = items.len(),
        "frontmost_menu_snapshot.loaded"
    );

    Ok(FrontmostMenuSnapshot {
        app_name: tracked_app.name,
        bundle_id: tracked_app.bundle_id,
        items,
    })
}

/// Stub for non-macOS platforms — always returns an error.
#[cfg(not(target_os = "macos"))]
pub fn load_frontmost_menu_snapshot() -> anyhow::Result<FrontmostMenuSnapshot> {
    anyhow::bail!("Current App Commands requires macOS (Accessibility APIs unavailable)")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menu_bar::{KeyboardShortcut, MenuBarItem, ModifierFlags};

    fn apple_menu() -> MenuBarItem {
        MenuBarItem {
            title: "Apple".into(),
            enabled: true,
            shortcut: None,
            children: vec![],
            ax_element_path: vec![0],
        }
    }

    fn leaf(title: &str, path: Vec<usize>) -> MenuBarItem {
        MenuBarItem {
            title: title.into(),
            enabled: true,
            shortcut: None,
            children: vec![],
            ax_element_path: path,
        }
    }

    fn leaf_with_shortcut(title: &str, key: &str, path: Vec<usize>) -> MenuBarItem {
        MenuBarItem {
            title: title.into(),
            enabled: true,
            shortcut: Some(KeyboardShortcut::new(key.into(), ModifierFlags::COMMAND)),
            children: vec![],
            ax_element_path: path,
        }
    }

    fn menu(title: &str, children: Vec<MenuBarItem>, path: Vec<usize>) -> MenuBarItem {
        MenuBarItem {
            title: title.into(),
            enabled: true,
            shortcut: None,
            children,
            ax_element_path: path,
        }
    }

    #[test]
    fn into_entries_skips_apple_menu() {
        let snap = FrontmostMenuSnapshot {
            app_name: "TestApp".into(),
            bundle_id: "com.test.app".into(),
            items: vec![
                apple_menu(),
                menu("File", vec![leaf("New Tab", vec![1, 0])], vec![1]),
            ],
        };

        let entries = snap.into_entries();
        assert_eq!(entries.len(), 1);
        assert!(entries[0].name.contains("New Tab"));
    }

    #[test]
    fn into_entries_empty_menu_returns_empty() {
        let snap = FrontmostMenuSnapshot {
            app_name: "Empty".into(),
            bundle_id: "com.test.empty".into(),
            items: vec![],
        };
        assert!(snap.into_entries().is_empty());
    }

    #[test]
    fn placeholder_includes_app_name() {
        let snap = FrontmostMenuSnapshot {
            app_name: "Safari".into(),
            bundle_id: "com.apple.Safari".into(),
            items: vec![],
        };
        assert_eq!(snap.placeholder(), "Search Safari commands\u{2026}");
    }

    #[test]
    fn into_entries_with_receipt_reports_counts() {
        let snap = FrontmostMenuSnapshot {
            app_name: "TestApp".into(),
            bundle_id: "com.test.app".into(),
            items: vec![
                apple_menu(),
                menu("File", vec![leaf("New Tab", vec![1, 0])], vec![1]),
            ],
        };

        let (entries, receipt) = snap.into_entries_with_receipt();

        assert_eq!(entries.len(), 1);
        assert_eq!(receipt.app_name, "TestApp");
        assert_eq!(receipt.bundle_id, "com.test.app");
        assert_eq!(receipt.top_level_menu_count, 2);
        assert_eq!(receipt.leaf_entry_count, 1);
        assert_eq!(receipt.placeholder, "Search TestApp commands\u{2026}");
        assert_eq!(receipt.source, "frontmost_app_tracker");
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn loader_returns_snapshot_or_error() {
        // This test just ensures the loader doesn't panic.
        // It may return Ok or Err depending on whether the tracker is running.
        let _result = load_frontmost_menu_snapshot();
    }

    #[cfg(not(target_os = "macos"))]
    #[test]
    fn loader_returns_unsupported_error() {
        let err = load_frontmost_menu_snapshot().unwrap_err();
        assert!(
            err.to_string().contains("macOS"),
            "Expected macOS-specific error, got: {}",
            err
        );
    }

    // -----------------------------------------------------------------------
    // Script prompt builder tests
    // -----------------------------------------------------------------------

    #[test]
    fn generate_script_prompt_includes_user_request_and_optional_context() {
        let snap = FrontmostMenuSnapshot {
            app_name: "Safari".into(),
            bundle_id: "com.apple.Safari".into(),
            items: vec![
                apple_menu(),
                menu(
                    "File",
                    vec![leaf_with_shortcut("New Tab", "T", vec![1, 0])],
                    vec![1],
                ),
            ],
        };

        let (prompt, receipt) = build_generate_script_prompt_from_snapshot(
            snap,
            Some("close duplicate tabs"),
            Some("pricing"),
            Some("https://example.com/pricing"),
        );

        assert_eq!(receipt.app_name, "Safari");
        assert_eq!(receipt.bundle_id, "com.apple.Safari");
        assert_eq!(receipt.total_menu_items, 1);
        assert_eq!(receipt.included_menu_items, 1);
        assert!(receipt.included_user_request);
        assert!(receipt.included_selected_text);
        assert!(receipt.included_browser_url);

        assert!(prompt.contains("User Request:\nclose duplicate tabs"));
        assert!(prompt.contains("Frontmost App: Safari"));
        assert!(prompt.contains("Bundle ID: com.apple.Safari"));
        assert!(prompt.contains("Selected Text:\n```text\npricing\n```"));
        assert!(prompt.contains("Focused Browser URL:\nhttps://example.com/pricing"));
    }

    #[test]
    fn generate_script_prompt_omits_empty_optional_inputs() {
        let snap = FrontmostMenuSnapshot {
            app_name: "Finder".into(),
            bundle_id: "com.apple.finder".into(),
            items: vec![],
        };

        let (prompt, receipt) =
            build_generate_script_prompt_from_snapshot(snap, None, Some("  "), Some(""));

        assert!(!receipt.included_user_request);
        assert!(!receipt.included_selected_text);
        assert!(!receipt.included_browser_url);
        assert!(!prompt.contains("User Request:"));
        assert!(!prompt.contains("Selected Text:"));
        assert!(!prompt.contains("Focused Browser URL:"));
    }

    #[test]
    fn generate_script_prompt_truncates_selected_text() {
        let long_text: String = "x".repeat(2_000);
        let snap = FrontmostMenuSnapshot {
            app_name: "TextEdit".into(),
            bundle_id: "com.apple.TextEdit".into(),
            items: vec![],
        };

        let (prompt, receipt) =
            build_generate_script_prompt_from_snapshot(snap, None, Some(&long_text), None);

        assert!(receipt.included_selected_text);
        // The truncated text should be exactly MAX_SELECTED_TEXT_CHARS characters
        let expected_truncated: String = "x".repeat(MAX_SELECTED_TEXT_CHARS);
        assert!(prompt.contains(&expected_truncated));
        // But not the full 2000
        assert!(!prompt.contains(&long_text));
    }

    #[test]
    fn generate_script_prompt_truncates_long_menu_lists() {
        let children: Vec<MenuBarItem> = (0..25)
            .map(|idx| leaf(&format!("Item {}", idx), vec![1, idx]))
            .collect();

        let snap = FrontmostMenuSnapshot {
            app_name: "BigApp".into(),
            bundle_id: "com.example.BigApp".into(),
            items: vec![apple_menu(), menu("File", children, vec![1])],
        };

        let (prompt, receipt) = build_generate_script_prompt_from_snapshot(snap, None, None, None);

        assert_eq!(receipt.total_menu_items, 25);
        assert_eq!(receipt.included_menu_items, MAX_SCRIPT_PROMPT_MENU_ITEMS);
        assert!(prompt.contains("Enabled Menu Commands (showing 20 of 25):"));
    }

    #[test]
    fn generate_script_prompt_includes_shortcut_suffix() {
        let snap = FrontmostMenuSnapshot {
            app_name: "Safari".into(),
            bundle_id: "com.apple.Safari".into(),
            items: vec![
                apple_menu(),
                menu(
                    "File",
                    vec![leaf_with_shortcut("New Tab", "T", vec![1, 0])],
                    vec![1],
                ),
            ],
        };

        let (prompt, _receipt) = build_generate_script_prompt_from_snapshot(snap, None, None, None);

        // The entry name from menu_bar_items_to_entries includes the path,
        // and the shortcut should be appended in parentheses
        assert!(
            prompt.contains("(⌘T)"),
            "Prompt should include shortcut suffix, got:\n{}",
            prompt
        );
    }

    // -----------------------------------------------------------------------
    // normalize_generate_script_from_current_app_request
    // -----------------------------------------------------------------------

    #[test]
    fn normalize_generate_script_from_current_app_request_drops_builtin_label() {
        assert_eq!(
            normalize_generate_script_from_current_app_request(Some(
                GENERATE_SCRIPT_FROM_CURRENT_APP_LABEL
            )),
            None
        );
        assert_eq!(
            normalize_generate_script_from_current_app_request(Some(
                "  generate script from current app  "
            )),
            None
        );
        assert_eq!(
            normalize_generate_script_from_current_app_request(Some("close duplicate tabs")),
            Some("close duplicate tabs")
        );
        assert_eq!(
            normalize_generate_script_from_current_app_request(Some("   ")),
            None
        );
        assert_eq!(
            normalize_generate_script_from_current_app_request(None),
            None
        );
    }

    // -----------------------------------------------------------------------
    // normalize_generate_script_request (Generate Script with AI)
    // -----------------------------------------------------------------------

    #[test]
    fn normalize_generate_script_request_drops_builtin_label() {
        assert_eq!(
            normalize_generate_script_request(Some(GENERATE_SCRIPT_WITH_AI_LABEL)),
            None
        );
        assert_eq!(
            normalize_generate_script_request(Some("  generate script with ai  ")),
            None
        );
        assert_eq!(
            normalize_generate_script_request(Some("build a clipboard cleanup script")),
            Some("build a clipboard cleanup script")
        );
        assert_eq!(normalize_generate_script_request(Some("   ")), None);
        assert_eq!(normalize_generate_script_request(None), None);
    }

    // -----------------------------------------------------------------------
    // normalize_builtin_labeled_request (generic helper)
    // -----------------------------------------------------------------------

    #[test]
    fn normalize_builtin_labeled_request_generic() {
        assert_eq!(
            normalize_builtin_labeled_request(Some("My Command"), "My Command"),
            None
        );
        assert_eq!(
            normalize_builtin_labeled_request(Some("  my command  "), "My Command"),
            None
        );
        assert_eq!(
            normalize_builtin_labeled_request(Some("real input"), "My Command"),
            Some("real input")
        );
        assert_eq!(
            normalize_builtin_labeled_request(Some(""), "My Command"),
            None
        );
        assert_eq!(normalize_builtin_labeled_request(None, "My Command"), None);
    }

    // -----------------------------------------------------------------------
    // normalize_do_in_current_app_request
    // -----------------------------------------------------------------------

    #[test]
    fn normalize_do_in_current_app_request_drops_builtin_label() {
        assert_eq!(
            normalize_do_in_current_app_request(Some(DO_IN_CURRENT_APP_LABEL)),
            None
        );
        assert_eq!(
            normalize_do_in_current_app_request(Some("  do in current app  ")),
            None
        );
        assert_eq!(
            normalize_do_in_current_app_request(Some("close duplicate tabs")),
            Some("close duplicate tabs")
        );
        assert_eq!(normalize_do_in_current_app_request(Some("   ")), None);
        assert_eq!(normalize_do_in_current_app_request(None), None);
    }

    #[test]
    fn normalize_do_in_current_app_request_drops_compat_label() {
        assert_eq!(
            normalize_do_in_current_app_request(Some("Current App Commands")),
            None
        );
        assert_eq!(
            normalize_do_in_current_app_request(Some("current app commands")),
            None
        );
    }

    #[test]
    fn normalize_do_in_current_app_request_strips_label_prefix() {
        assert_eq!(
            normalize_do_in_current_app_request(Some(
                "Do in Current App close duplicate tabs"
            )),
            Some("close duplicate tabs")
        );
        assert_eq!(
            normalize_do_in_current_app_request(Some(
                "Current App Commands: close duplicate tabs"
            )),
            Some("close duplicate tabs")
        );
        assert_eq!(
            normalize_do_in_current_app_request(Some(
                "Current App Commands - open new window"
            )),
            Some("open new window")
        );
        // Label prefix with only separator chars → None
        assert_eq!(
            normalize_do_in_current_app_request(Some("Do in Current App:  ")),
            None
        );
    }

    // -----------------------------------------------------------------------
    // resolve_do_in_current_app_intent
    // -----------------------------------------------------------------------

    #[test]
    fn resolve_do_in_current_app_intent_unique_leaf_match_executes_entry() {
        let snap = FrontmostMenuSnapshot {
            app_name: "Safari".into(),
            bundle_id: "com.apple.Safari".into(),
            items: vec![
                apple_menu(),
                menu(
                    "File",
                    vec![
                        leaf_with_shortcut("New Tab", "T", vec![1, 0]),
                        leaf("Close Window", vec![1, 1]),
                    ],
                    vec![1],
                ),
            ],
        };

        let entries = snap.into_entries();
        let (action, receipt) = resolve_do_in_current_app_intent(&entries, Some("new tab"));

        assert_eq!(action, DoInCurrentAppAction::ExecuteEntry(0));
        assert_eq!(receipt.filtered_entries, 1);
        assert_eq!(receipt.exact_matches, 1);
        assert_eq!(receipt.action, "execute_entry");
    }

    #[test]
    fn resolve_do_in_current_app_intent_ambiguous_query_opens_palette() {
        let snap = FrontmostMenuSnapshot {
            app_name: "Safari".into(),
            bundle_id: "com.apple.Safari".into(),
            items: vec![
                apple_menu(),
                menu(
                    "File",
                    vec![leaf("New Tab", vec![1, 0]), leaf("New Window", vec![1, 1])],
                    vec![1],
                ),
            ],
        };

        let entries = snap.into_entries();
        let (action, receipt) = resolve_do_in_current_app_intent(&entries, Some("new"));

        assert_eq!(action, DoInCurrentAppAction::OpenCommandPalette);
        assert_eq!(receipt.filtered_entries, 2);
        assert_eq!(receipt.action, "open_command_palette");
    }

    #[test]
    fn resolve_do_in_current_app_intent_no_matches_generates_script() {
        let snap = FrontmostMenuSnapshot {
            app_name: "Safari".into(),
            bundle_id: "com.apple.Safari".into(),
            items: vec![
                apple_menu(),
                menu("File", vec![leaf("New Tab", vec![1, 0])], vec![1]),
            ],
        };

        let entries = snap.into_entries();
        let (action, receipt) =
            resolve_do_in_current_app_intent(&entries, Some("close duplicate tabs"));

        assert_eq!(action, DoInCurrentAppAction::GenerateScript);
        assert_eq!(receipt.filtered_entries, 0);
        assert_eq!(receipt.exact_matches, 0);
        assert_eq!(receipt.action, "generate_script");
    }

    #[test]
    fn resolve_do_in_current_app_intent_empty_query_opens_palette() {
        let snap = FrontmostMenuSnapshot {
            app_name: "Safari".into(),
            bundle_id: "com.apple.Safari".into(),
            items: vec![
                apple_menu(),
                menu("File", vec![leaf("New Tab", vec![1, 0])], vec![1]),
            ],
        };

        let entries = snap.into_entries();
        let (action, receipt) =
            resolve_do_in_current_app_intent(&entries, Some(DO_IN_CURRENT_APP_LABEL));

        assert_eq!(action, DoInCurrentAppAction::OpenCommandPalette);
        assert_eq!(receipt.action, "open_command_palette");
    }

    // -----------------------------------------------------------------------
    // resolve_do_in_current_app_intent: edge-case routing
    // -----------------------------------------------------------------------

    #[test]
    fn resolve_do_in_current_app_intent_exact_shortcut_keyword_executes_entry() {
        let snap = FrontmostMenuSnapshot {
            app_name: "Safari".into(),
            bundle_id: "com.apple.Safari".into(),
            items: vec![
                apple_menu(),
                menu(
                    "File",
                    vec![leaf_with_shortcut("New Tab", "T", vec![1, 0])],
                    vec![1],
                ),
            ],
        };

        let entries = snap.into_entries();
        let (action, receipt) = resolve_do_in_current_app_intent(&entries, Some("cmd+t"));

        assert_eq!(action, DoInCurrentAppAction::ExecuteEntry(0));
        assert_eq!(receipt.filtered_entries, 1);
        assert_eq!(receipt.exact_matches, 1);
        assert_eq!(receipt.action, "execute_entry");
    }

    #[test]
    fn resolve_do_in_current_app_intent_normalizes_path_punctuation_and_case() {
        let snap = FrontmostMenuSnapshot {
            app_name: "Safari".into(),
            bundle_id: "com.apple.Safari".into(),
            items: vec![
                apple_menu(),
                menu("File", vec![leaf("New Tab", vec![1, 0])], vec![1]),
            ],
        };

        let entries = snap.into_entries();
        let (action, receipt) = resolve_do_in_current_app_intent(&entries, Some("FILE -> new tab"));

        assert_eq!(action, DoInCurrentAppAction::ExecuteEntry(0));
        assert_eq!(receipt.filtered_entries, 1);
        assert_eq!(receipt.exact_matches, 1);
        assert_eq!(receipt.action, "execute_entry");
    }

    #[test]
    fn generate_script_prompt_omits_builtin_label_request() {
        let snap = FrontmostMenuSnapshot {
            app_name: "Safari".into(),
            bundle_id: "com.apple.Safari".into(),
            items: vec![apple_menu()],
        };

        let request = normalize_generate_script_from_current_app_request(Some(
            GENERATE_SCRIPT_FROM_CURRENT_APP_LABEL,
        ));

        let (prompt, receipt) =
            build_generate_script_prompt_from_snapshot(snap, request, None, None);

        assert!(!receipt.included_user_request);
        assert!(
            !prompt.contains("User Request:"),
            "Prompt should omit User Request when input matches the built-in label"
        );
    }

    // -----------------------------------------------------------------------
    // Turn This Into a Command
    // -----------------------------------------------------------------------

    #[test]
    fn normalize_turn_this_into_a_command_request_strips_label_prefix() {
        assert_eq!(
            normalize_turn_this_into_a_command_request(Some(TURN_THIS_INTO_A_COMMAND_LABEL)),
            None
        );

        assert_eq!(
            normalize_turn_this_into_a_command_request(Some(
                "Turn This Into a Command close duplicate tabs"
            )),
            Some("close duplicate tabs".to_string())
        );

        assert_eq!(
            normalize_turn_this_into_a_command_request(Some("Turn This Into a Command: new tab")),
            Some("new tab".to_string())
        );
    }

    #[test]
    fn suggest_current_app_command_name_title_cases_request() {
        assert_eq!(
            suggest_current_app_command_name("Safari", "close duplicate tabs"),
            "Safari Close Duplicate Tabs"
        );

        assert_eq!(
            suggest_current_app_command_name("Finder", ""),
            "Finder Command"
        );
    }

    #[test]
    fn build_current_app_command_recipe_contains_trace_and_prompt() {
        let snap = FrontmostMenuSnapshot {
            app_name: "Safari".into(),
            bundle_id: "com.apple.Safari".into(),
            items: vec![
                apple_menu(),
                menu(
                    "File",
                    vec![leaf_with_shortcut("New Tab", "T", vec![1, 0])],
                    vec![1],
                ),
            ],
        };

        let recipe = build_current_app_command_recipe(
            snap,
            Some("Turn This Into a Command close duplicate tabs"),
            None,
            None,
        );

        assert_eq!(
            recipe.schema_version,
            CURRENT_APP_COMMAND_RECIPE_SCHEMA_VERSION
        );
        assert_eq!(recipe.recipe_type, "currentAppCommand");
        assert_eq!(
            recipe.raw_query,
            "Turn This Into a Command close duplicate tabs"
        );
        assert_eq!(recipe.effective_query, "close duplicate tabs");
        assert_eq!(
            recipe.trace.raw_query,
            "Turn This Into a Command close duplicate tabs"
        );
        assert_eq!(recipe.trace.effective_query, "close duplicate tabs");
        assert_eq!(recipe.suggested_script_name, "Safari Close Duplicate Tabs");
        assert_eq!(recipe.trace.action, "generate_script");
        assert!(recipe
            .prompt
            .contains("User Request:\nclose duplicate tabs"));
        assert_eq!(recipe.prompt_receipt.app_name, "Safari");
        assert_eq!(recipe.prompt_receipt.bundle_id, "com.apple.Safari");
    }

    #[test]
    fn current_app_command_recipe_serializes_with_camel_case_fields() {
        let snap = FrontmostMenuSnapshot {
            app_name: "Safari".into(),
            bundle_id: "com.apple.Safari".into(),
            items: vec![
                apple_menu(),
                menu(
                    "File",
                    vec![leaf_with_shortcut("New Tab", "T", vec![1, 0])],
                    vec![1],
                ),
            ],
        };

        let recipe = build_current_app_command_recipe(
            snap,
            Some("Turn This Into a Command new tab"),
            None,
            None,
        );

        let value = serde_json::to_value(&recipe).expect("recipe should serialize");
        assert!(value.get("schemaVersion").is_some());
        assert!(value.get("recipeType").is_some());
        assert!(value.get("suggestedScriptName").is_some());
        assert!(value.get("trace").is_some());
        assert!(value.get("promptReceipt").is_some());
    }

    // -----------------------------------------------------------------------
    // Replay Current App Recipe receipt tests
    // -----------------------------------------------------------------------

    #[test]
    fn replay_current_app_recipe_receipt_executes_entry_on_exact_match() {
        let snapshot = FrontmostMenuSnapshot {
            app_name: "Safari".into(),
            bundle_id: "com.apple.Safari".into(),
            items: vec![
                apple_menu(),
                menu(
                    "File",
                    vec![
                        leaf_with_shortcut("New Tab", "T", vec![1, 0]),
                        leaf("Close Window", vec![1, 1]),
                    ],
                    vec![1],
                ),
            ],
        };

        let stored_recipe =
            build_current_app_command_recipe(snapshot.clone(), Some("new tab"), None, None);
        let entries = snapshot.clone().into_entries();

        let receipt =
            build_replay_current_app_recipe_receipt(&stored_recipe, &entries, snapshot, None, None);

        assert_eq!(receipt.verification.status, "match");
        assert_eq!(receipt.action, "execute_entry");
        assert_eq!(receipt.selected_entry_index, Some(0));
        assert_eq!(
            receipt
                .selected_entry
                .as_ref()
                .map(|entry| entry.leaf_name.as_str()),
            Some("New Tab")
        );
    }

    #[test]
    fn replay_current_app_recipe_receipt_opens_palette_for_ambiguous_recipe() {
        let snapshot = FrontmostMenuSnapshot {
            app_name: "Safari".into(),
            bundle_id: "com.apple.Safari".into(),
            items: vec![
                apple_menu(),
                menu(
                    "File",
                    vec![leaf("New Tab", vec![1, 0]), leaf("New Window", vec![1, 1])],
                    vec![1],
                ),
            ],
        };

        let stored_recipe =
            build_current_app_command_recipe(snapshot.clone(), Some("new"), None, None);
        let entries = snapshot.clone().into_entries();

        let receipt =
            build_replay_current_app_recipe_receipt(&stored_recipe, &entries, snapshot, None, None);

        assert_eq!(receipt.verification.status, "match");
        assert_eq!(receipt.action, "open_command_palette");
        assert_eq!(receipt.selected_entry_index, None);
        assert!(receipt.selected_entry.is_none());
    }

    #[test]
    fn replay_current_app_recipe_receipt_generates_script_when_no_direct_match_exists() {
        let snapshot = FrontmostMenuSnapshot {
            app_name: "Safari".into(),
            bundle_id: "com.apple.Safari".into(),
            items: vec![
                apple_menu(),
                menu("File", vec![leaf("New Tab", vec![1, 0])], vec![1]),
            ],
        };

        let stored_recipe = build_current_app_command_recipe(
            snapshot.clone(),
            Some("close duplicate tabs"),
            None,
            None,
        );
        let entries = snapshot.clone().into_entries();

        let receipt =
            build_replay_current_app_recipe_receipt(&stored_recipe, &entries, snapshot, None, None);

        assert_eq!(receipt.verification.status, "match");
        assert_eq!(receipt.action, "generate_script");
        assert_eq!(receipt.selected_entry_index, None);
        assert!(receipt.selected_entry.is_none());
    }

    #[test]
    fn replay_current_app_recipe_receipt_blocks_when_drift_detected() {
        let stored_snapshot = FrontmostMenuSnapshot {
            app_name: "Safari".into(),
            bundle_id: "com.apple.Safari".into(),
            items: vec![
                apple_menu(),
                menu("File", vec![leaf("New Tab", vec![1, 0])], vec![1]),
            ],
        };

        let live_snapshot = FrontmostMenuSnapshot {
            app_name: "Finder".into(),
            bundle_id: "com.apple.finder".into(),
            items: vec![
                apple_menu(),
                menu("File", vec![leaf("New Finder Window", vec![1, 0])], vec![1]),
            ],
        };

        let stored_recipe =
            build_current_app_command_recipe(stored_snapshot, Some("new tab"), None, None);
        let live_entries = live_snapshot.clone().into_entries();

        let receipt = build_replay_current_app_recipe_receipt(
            &stored_recipe,
            &live_entries,
            live_snapshot,
            None,
            None,
        );

        assert_eq!(receipt.verification.status, "drift");
        assert_eq!(receipt.action, "blocked_by_drift");
        assert!(receipt.verification.warning_count > 0);
    }

    // -----------------------------------------------------------------------
    // Recipe construction contract: context flags and fields
    // -----------------------------------------------------------------------

    #[test]
    fn build_current_app_command_recipe_marks_context_flags() {
        let snap = FrontmostMenuSnapshot {
            app_name: "Safari".into(),
            bundle_id: "com.apple.Safari".into(),
            items: vec![
                apple_menu(),
                menu("File", vec![leaf("New Tab", vec![1, 0])], vec![1]),
            ],
        };

        let recipe = build_current_app_command_recipe(
            snap,
            Some("close duplicate tabs"),
            Some("tab 1\ntab 2"),
            Some("https://example.com"),
        );

        assert_eq!(recipe.recipe_type, "currentAppCommand");
        assert_eq!(recipe.effective_query, "close duplicate tabs");
        assert_eq!(recipe.trace.action, "generate_script");
        assert!(recipe.prompt_receipt.included_user_request);
        assert!(recipe.prompt_receipt.included_selected_text);
        assert!(recipe.prompt_receipt.included_browser_url);
        assert_eq!(recipe.suggested_script_name, "Safari Close Duplicate Tabs");
    }

    // -----------------------------------------------------------------------
    // Prompt contract stability: output contract and recipe header
    // -----------------------------------------------------------------------

    #[test]
    fn generated_script_prompt_from_recipe_embeds_contract_and_recipe_header() {
        let snap = FrontmostMenuSnapshot {
            app_name: "Safari".into(),
            bundle_id: "com.apple.Safari".into(),
            items: vec![
                apple_menu(),
                menu("File", vec![leaf("New Tab", vec![1, 0])], vec![1]),
            ],
        };

        let recipe = build_current_app_command_recipe(
            snap,
            Some("close duplicate tabs"),
            None,
            Some("https://example.com"),
        );

        let prompt = build_generated_script_prompt_from_recipe(&recipe);

        assert!(
            prompt.contains("OUTPUT CONTRACT:"),
            "prompt must contain OUTPUT CONTRACT section"
        );
        assert!(
            prompt.contains("Return only runnable Script Kit TypeScript."),
            "prompt must require runnable Script Kit TypeScript"
        );
        assert!(
            prompt.contains("Current-App-Recipe-Base64:"),
            "prompt must embed base64-encoded recipe header"
        );
        assert!(
            prompt.contains("Bias toward direct menu-command automation"),
            "prompt must bias toward menu-command automation"
        );
    }
}
