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

/// The human-readable label used in the main command list.
pub const GENERATE_SCRIPT_FROM_CURRENT_APP_LABEL: &str = "Generate Script from Current App";

/// Returns `None` when the raw input is empty, whitespace-only, or matches the
/// built-in label (case-insensitive). Otherwise returns the trimmed input.
pub fn normalize_generate_script_from_current_app_request(raw: Option<&str>) -> Option<&str> {
    let raw = raw.map(str::trim).filter(|text| !text.is_empty())?;

    if raw.eq_ignore_ascii_case(GENERATE_SCRIPT_FROM_CURRENT_APP_LABEL) {
        None
    } else {
        Some(raw)
    }
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

/// Returns `None` when the raw input is empty, whitespace-only, or matches the
/// built-in label (case-insensitive). Otherwise returns the trimmed input.
pub fn normalize_do_in_current_app_request(raw: Option<&str>) -> Option<&str> {
    let raw = raw.map(str::trim).filter(|text| !text.is_empty())?;

    if raw.eq_ignore_ascii_case(DO_IN_CURRENT_APP_LABEL) {
        None
    } else {
        Some(raw)
    }
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

    let (action, action_name) = if exact_matches.len() == 1 {
        (
            DoInCurrentAppAction::ExecuteEntry(exact_matches[0]),
            "execute_entry",
        )
    } else if filtered.is_empty() {
        (DoInCurrentAppAction::GenerateScript, "generate_script")
    } else {
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
#[derive(Debug, Clone, PartialEq, Eq)]
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
        let (action, receipt) =
            resolve_do_in_current_app_intent(&entries, Some("FILE -> new tab"));

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
}
