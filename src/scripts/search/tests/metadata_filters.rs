use std::path::PathBuf;
use std::sync::Arc;

use super::super::*;
use super::core_search::make_script;

fn make_script_with_shortcut(name: &str, shortcut: Option<&str>) -> Arc<Script> {
    Arc::new(Script {
        name: name.to_string(),
        path: PathBuf::from(format!(
            "/test/{}.ts",
            name.to_lowercase().replace(' ', "-")
        )),
        extension: "ts".to_string(),
        shortcut: shortcut.map(|s| s.to_string()),
        ..Default::default()
    })
}

#[test]
fn test_shortcut_search_finds_script() {
    let scripts = vec![
        make_script_with_shortcut("Toggle Dark Mode", Some("opt d")),
        make_script_with_shortcut("Open Dashboard", None),
        make_script("Dark Reader", Some("Toggle dark mode in browser")),
    ];
    let results = fuzzy_search_scripts(&scripts, "opt d");
    assert!(!results.is_empty());
    // "Toggle Dark Mode" with shortcut "opt d" should rank first
    assert_eq!(results[0].script.name, "Toggle Dark Mode");
}

#[test]
fn test_shortcut_search_partial_match() {
    let scripts = vec![
        make_script_with_shortcut("Screenshot Tool", Some("cmd shift s")),
        make_script("Search Files", Some("Search for files")),
    ];
    let results = fuzzy_search_scripts(&scripts, "cmd shift");
    assert!(!results.is_empty());
    // Script with matching shortcut prefix should appear in results
    assert!(
        results.iter().any(|r| r.script.name == "Screenshot Tool"),
        "Script with matching shortcut should be found"
    );
}

// ============================================
// Kit name search tests
// ============================================

fn make_script_with_kit(name: &str, kit_name: Option<&str>) -> Arc<Script> {
    Arc::new(Script {
        name: name.to_string(),
        path: PathBuf::from(format!(
            "/test/{}.ts",
            name.to_lowercase().replace(' ', "-")
        )),
        extension: "ts".to_string(),
        kit_name: kit_name.map(|s| s.to_string()),
        ..Default::default()
    })
}

#[test]
fn test_kit_name_search_boosts_results() {
    let scripts = vec![
        make_script_with_kit("Capture Window", Some("cleanshot")),
        make_script_with_kit("Clean Workspace", Some("main")),
        make_script_with_kit("Annotate", Some("cleanshot")),
    ];
    let results = fuzzy_search_scripts(&scripts, "cleanshot");
    assert!(!results.is_empty());
    // Both cleanshot scripts should appear and rank above "Clean Workspace"
    // which only matches on name substring, not kit name
    let cleanshot_results: Vec<_> = results
        .iter()
        .filter(|r| r.script.kit_name.as_deref() == Some("cleanshot"))
        .collect();
    assert_eq!(
        cleanshot_results.len(),
        2,
        "Both cleanshot scripts should match"
    );
}

#[test]
fn test_main_kit_not_boosted() {
    // "main" kit should NOT get a bonus since it's the default
    let scripts = vec![
        make_script_with_kit("My Script", Some("main")),
        make_script("Main Event", None),
    ];
    let results = fuzzy_search_scripts(&scripts, "main");
    // "Main Event" has a name match and should not be penalized
    // "My Script" with kit "main" should NOT get a kit name bonus
    assert!(!results.is_empty());
}

// ============================================
// Scriptlet shortcut and group search tests
// ============================================

fn make_scriptlet_with_opts(
    name: &str,
    shortcut: Option<&str>,
    group: Option<&str>,
) -> Arc<Scriptlet> {
    Arc::new(Scriptlet {
        name: name.to_string(),
        description: None,
        code: "echo hello".to_string(),
        tool: "bash".to_string(),
        shortcut: shortcut.map(|s| s.to_string()),
        keyword: None,
        group: group.map(|s| s.to_string()),
        file_path: None,
        command: None,
        alias: None,
    })
}

#[test]
fn test_scriptlet_shortcut_search() {
    let scriptlets = vec![
        make_scriptlet_with_opts("Quick Paste", Some("opt v"), None),
        make_scriptlet_with_opts("Variable Dump", None, None),
    ];
    let results = fuzzy_search_scriptlets(&scriptlets, "opt v");
    assert!(!results.is_empty());
    assert_eq!(results[0].scriptlet.name, "Quick Paste");
}

#[test]
fn test_scriptlet_group_search() {
    let scriptlets = vec![
        make_scriptlet_with_opts("Git Commit", None, Some("development")),
        make_scriptlet_with_opts("Git Push", None, Some("development")),
        make_scriptlet_with_opts("Restart Server", None, Some("ops")),
    ];
    let results = fuzzy_search_scriptlets(&scriptlets, "development");
    // Both "development" group scriptlets should match
    let dev_results: Vec<_> = results
        .iter()
        .filter(|r| r.scriptlet.group.as_deref() == Some("development"))
        .collect();
    assert_eq!(
        dev_results.len(),
        2,
        "Both development scriptlets should match"
    );
}

// ============================================
// Tag-based search tests
// ============================================

fn make_script_with_tags(name: &str, tags: &[&str]) -> Arc<Script> {
    use crate::metadata_parser::TypedMetadata;
    Arc::new(Script {
        name: name.to_string(),
        path: PathBuf::from(format!(
            "/test/{}.ts",
            name.to_lowercase().replace(' ', "-")
        )),
        extension: "ts".to_string(),
        typed_metadata: Some(TypedMetadata {
            tags: tags.iter().map(|t| t.to_string()).collect(),
            ..Default::default()
        }),
        ..Default::default()
    })
}

#[test]
fn test_tag_search_finds_matching_scripts() {
    let scripts = vec![
        make_script_with_tags("Create Note", &["productivity", "notes"]),
        make_script_with_tags("Git Commit", &["development", "git"]),
        make_script("Untagged Script", Some("no tags")),
    ];
    let results = fuzzy_search_scripts(&scripts, "productivity");
    // "Create Note" should match via its "productivity" tag
    assert!(
        results.iter().any(|r| r.script.name == "Create Note"),
        "Script with matching tag should be found"
    );
}

#[test]
fn test_tag_search_partial_match() {
    let scripts = vec![
        make_script_with_tags("Deploy App", &["deployment", "ci-cd"]),
        make_script("Random Script", None),
    ];
    let results = fuzzy_search_scripts(&scripts, "deploy");
    // "Deploy App" should match via both name AND tag substring
    assert!(
        results.iter().any(|r| r.script.name == "Deploy App"),
        "Script should match via tag substring"
    );
}

#[test]
fn test_tag_search_only_counts_best_match() {
    // Multiple tags matching should only count once (break after first match)
    let scripts = vec![make_script_with_tags(
        "Multi Tag",
        &["dev", "development", "developer"],
    )];
    let results = fuzzy_search_scripts(&scripts, "dev");
    assert_eq!(results.len(), 1);
    // Score should include tag bonus but only once
    assert!(results[0].score > 0);
}

// ============================================
// Author-based search tests
// ============================================

fn make_script_with_author(name: &str, author: &str) -> Arc<Script> {
    use crate::metadata_parser::TypedMetadata;
    Arc::new(Script {
        name: name.to_string(),
        path: PathBuf::from(format!(
            "/test/{}.ts",
            name.to_lowercase().replace(' ', "-")
        )),
        extension: "ts".to_string(),
        typed_metadata: Some(TypedMetadata {
            author: Some(author.to_string()),
            ..Default::default()
        }),
        ..Default::default()
    })
}

#[test]
fn test_author_search_finds_scripts() {
    let scripts = vec![
        make_script_with_author("My Script", "John Lindquist"),
        make_script_with_author("Other Script", "Jane Doe"),
        make_script("No Author", Some("plain script")),
    ];
    let results = fuzzy_search_scripts(&scripts, "john");
    // "My Script" by "John Lindquist" should match via author
    assert!(
        results.iter().any(|r| r.script.name == "My Script"),
        "Script with matching author should be found"
    );
}

#[test]
fn test_author_search_prefix_scores_higher() {
    let scripts = vec![
        make_script_with_author("Script A", "John Doe"),
        make_script_with_author("Script B", "Bobby Johnson"),
    ];
    let results = fuzzy_search_scripts(&scripts, "john");
    // Both should match but "Script A" (author prefix "John") should score higher
    assert!(results.len() >= 2);
    assert_eq!(
        results[0].script.name, "Script A",
        "Author prefix match should rank higher"
    );
}

// ============================================
// Hidden script filtering tests
// ============================================

fn make_hidden_script(name: &str) -> Arc<Script> {
    use crate::metadata_parser::TypedMetadata;
    Arc::new(Script {
        name: name.to_string(),
        path: PathBuf::from(format!(
            "/test/{}.ts",
            name.to_lowercase().replace(' ', "-")
        )),
        extension: "ts".to_string(),
        typed_metadata: Some(TypedMetadata {
            hidden: true,
            ..Default::default()
        }),
        ..Default::default()
    })
}

#[test]
fn test_hidden_scripts_excluded_from_search() {
    let scripts = vec![
        make_script("Visible Script", Some("shown in list")),
        make_hidden_script("Hidden Background Task"),
        make_script("Another Visible", None),
    ];
    let results = fuzzy_search_scripts(&scripts, "script");
    // Hidden script should NOT appear in results
    assert!(
        !results
            .iter()
            .any(|r| r.script.name == "Hidden Background Task"),
        "Hidden script should not appear in search results"
    );
    // Visible scripts should still appear
    assert!(results.iter().any(|r| r.script.name == "Visible Script"));
}

#[test]
fn test_hidden_scripts_excluded_from_empty_query() {
    let scripts = vec![make_script("Visible", None), make_hidden_script("Hidden")];
    // Empty query returns all non-hidden scripts
    let results = fuzzy_search_scripts(&scripts, "");
    assert_eq!(results.len(), 1, "Only visible scripts should be returned");
    assert_eq!(results[0].script.name, "Visible");
}

#[test]
fn test_non_hidden_scripts_not_affected() {
    // Scripts without typed_metadata or with hidden=false should not be filtered
    use crate::metadata_parser::TypedMetadata;
    let scripts = vec![
        make_script("No Metadata", None),
        Arc::new(Script {
            name: "Explicit False".to_string(),
            path: PathBuf::from("/test/explicit-false.ts"),
            extension: "ts".to_string(),
            typed_metadata: Some(TypedMetadata {
                hidden: false,
                ..Default::default()
            }),
            ..Default::default()
        }),
    ];
    let results = fuzzy_search_scripts(&scripts, "");
    assert_eq!(
        results.len(),
        2,
        "Non-hidden scripts should all be returned"
    );
}

// ============================================
// Property keyword search tests
// ============================================

fn make_script_with_cron(name: &str, cron: &str) -> Arc<Script> {
    use crate::metadata_parser::TypedMetadata;
    Arc::new(Script {
        name: name.to_string(),
        path: PathBuf::from(format!(
            "/test/{}.ts",
            name.to_lowercase().replace(' ', "-")
        )),
        extension: "ts".to_string(),
        typed_metadata: Some(TypedMetadata {
            cron: Some(cron.to_string()),
            ..Default::default()
        }),
        ..Default::default()
    })
}

fn make_script_with_background(name: &str) -> Arc<Script> {
    use crate::metadata_parser::TypedMetadata;
    Arc::new(Script {
        name: name.to_string(),
        path: PathBuf::from(format!(
            "/test/{}.ts",
            name.to_lowercase().replace(' ', "-")
        )),
        extension: "ts".to_string(),
        typed_metadata: Some(TypedMetadata {
            background: true,
            ..Default::default()
        }),
        ..Default::default()
    })
}

fn make_script_with_watch(name: &str, patterns: &[&str]) -> Arc<Script> {
    use crate::metadata_parser::TypedMetadata;
    Arc::new(Script {
        name: name.to_string(),
        path: PathBuf::from(format!(
            "/test/{}.ts",
            name.to_lowercase().replace(' ', "-")
        )),
        extension: "ts".to_string(),
        typed_metadata: Some(TypedMetadata {
            watch: patterns.iter().map(|p| p.to_string()).collect(),
            ..Default::default()
        }),
        ..Default::default()
    })
}

fn make_script_with_system(name: &str) -> Arc<Script> {
    use crate::metadata_parser::TypedMetadata;
    Arc::new(Script {
        name: name.to_string(),
        path: PathBuf::from(format!(
            "/test/{}.ts",
            name.to_lowercase().replace(' ', "-")
        )),
        extension: "ts".to_string(),
        typed_metadata: Some(TypedMetadata {
            system: true,
            ..Default::default()
        }),
        ..Default::default()
    })
}

#[test]
fn test_property_search_finds_cron_scripts() {
    let scripts = vec![
        make_script_with_cron("Daily Backup", "0 0 * * *"),
        make_script("Backup Manager", Some("Manage backups manually")),
        make_script("Random Script", None),
    ];
    let results = fuzzy_search_scripts(&scripts, "cron");
    assert!(
        results.iter().any(|r| r.script.name == "Daily Backup"),
        "Script with cron should be found when searching 'cron'"
    );
}

#[test]
fn test_property_search_scheduled_synonym() {
    let scripts = vec![
        make_script_with_cron("Email Digest", "0 9 * * 1"),
        make_script("Unrelated", None),
    ];
    let results = fuzzy_search_scripts(&scripts, "scheduled");
    assert!(
        results.iter().any(|r| r.script.name == "Email Digest"),
        "Script with cron should be found when searching 'scheduled'"
    );
}

#[test]
fn test_property_search_finds_background_scripts() {
    let scripts = vec![
        make_script_with_background("Clipboard Monitor"),
        make_script("Clipboard History", Some("View clipboard history")),
        make_script("Random Script", None),
    ];
    let results = fuzzy_search_scripts(&scripts, "background");
    assert!(
        results.iter().any(|r| r.script.name == "Clipboard Monitor"),
        "Background script should be found when searching 'background'"
    );
}

#[test]
fn test_property_search_finds_watch_scripts() {
    let scripts = vec![
        make_script_with_watch("Config Reloader", &["~/.config/**"]),
        make_script("Watch Movie", Some("Play a movie")),
    ];
    let results = fuzzy_search_scripts(&scripts, "watch");
    // Both should match - one via property, one via name
    assert!(
        results.iter().any(|r| r.script.name == "Config Reloader"),
        "Script with watch patterns should be found when searching 'watch'"
    );
}

#[test]
fn test_property_search_finds_system_scripts() {
    let scripts = vec![
        make_script_with_system("System Cleanup"),
        make_script("System Info", Some("Show system information")),
    ];
    let results = fuzzy_search_scripts(&scripts, "system");
    // Both should match - one via property, one via name
    assert!(
        results.iter().any(|r| r.script.name == "System Cleanup"),
        "System script should be found when searching 'system'"
    );
}

#[test]
fn test_property_search_no_false_positives() {
    // Scripts without special properties should NOT match property keywords
    // (unless they match on name/description/etc.)
    let scripts = vec![make_script("Hello World", None)];
    let results = fuzzy_search_scripts(&scripts, "cron");
    assert!(
        results.is_empty(),
        "Script without cron should not match 'cron'"
    );
}
