//! Plugin runtime ownership tests.
//!
//! Validates that script loading and scriptlet-command loading both consume
//! `discover_plugins()` and attach `plugin_id` and plugin-title fallback data
//! to every runtime entrypoint. Also validates that duplicate scriptlet groups
//! across different plugins preserve source attribution.

use std::fs;
use std::sync::Arc;

use script_kit_gpui::plugins::discover_plugins_in;
use script_kit_gpui::scripts::{
    MatchIndices, Script, ScriptMatch, ScriptMatchKind, Scriptlet, ScriptletMatch, SearchResult,
};

/// Helper: write a plugin.json manifest to a plugin root.
fn write_manifest(root: &std::path::Path, id: &str, title: &str) {
    let json = serde_json::json!({
        "id": id,
        "title": title,
        "description": format!("Test plugin: {title}"),
    });
    fs::write(
        root.join("plugin.json"),
        serde_json::to_string_pretty(&json).unwrap(),
    )
    .expect("write plugin.json");
}

/// Helper: create a minimal .ts script file with a Name metadata comment.
fn write_script(scripts_dir: &std::path::Path, filename: &str, name: &str) {
    fs::create_dir_all(scripts_dir).expect("create scripts dir");
    let content = format!("// Name: {name}\nconsole.log('hello');\n");
    fs::write(scripts_dir.join(filename), content).expect("write script");
}

/// Helper: create a minimal .md scriptlet bundle with one scriptlet.
fn write_extension(extensions_dir: &std::path::Path, filename: &str, group: &str, cmd_name: &str) {
    fs::create_dir_all(extensions_dir).expect("create extensions dir");
    let content = format!("# {group}\n\n## {cmd_name}\n\n```bash\necho hello\n```\n");
    fs::write(extensions_dir.join(filename), content).expect("write extension");
}

// ── Script plugin_id population ────────────────────────────────────

#[test]
fn script_struct_carries_plugin_id_and_title() {
    let script = Script {
        name: "hello".to_string(),
        plugin_id: "tools".to_string(),
        plugin_title: Some("Dev Tools".to_string()),
        ..Default::default()
    };

    assert_eq!(script.plugin_id, "tools");
    assert_eq!(script.plugin_title.as_deref(), Some("Dev Tools"));
}

#[test]
fn scriptlet_struct_carries_plugin_id_and_title() {
    let scriptlet = Scriptlet {
        name: "Open GitHub".to_string(),
        description: None,
        code: "echo hello".to_string(),
        tool: "bash".to_string(),
        shortcut: None,
        keyword: None,
        group: Some("Quick Links".to_string()),
        plugin_id: "quicklinks".to_string(),
        plugin_title: Some("Quick Links".to_string()),
        file_path: None,
        command: Some("open-github".to_string()),
        alias: None,
    };

    assert_eq!(scriptlet.plugin_id, "quicklinks");
    assert_eq!(scriptlet.plugin_title.as_deref(), Some("Quick Links"));
}

// ── SearchResult::source_name() resolves to plugin identity ────────

#[test]
fn source_name_prefers_plugin_title_for_scripts() {
    let script = Arc::new(Script {
        name: "hello".to_string(),
        plugin_id: "tools".to_string(),
        plugin_title: Some("Dev Tools".to_string()),
        kit_name: Some("tools".to_string()),
        ..Default::default()
    });

    let result = SearchResult::Script(ScriptMatch {
        script,
        score: 100,
        filename: "hello.ts".to_string(),
        match_indices: MatchIndices::default(),
        match_kind: ScriptMatchKind::Name,
        content_match: None,
    });

    assert_eq!(result.source_name(), Some("Dev Tools"));
}

#[test]
fn source_name_falls_back_to_plugin_id_for_scripts() {
    let script = Arc::new(Script {
        name: "hello".to_string(),
        plugin_id: "tools".to_string(),
        plugin_title: None,
        kit_name: Some("tools".to_string()),
        ..Default::default()
    });

    let result = SearchResult::Script(ScriptMatch {
        script,
        score: 100,
        filename: "hello.ts".to_string(),
        match_indices: MatchIndices::default(),
        match_kind: ScriptMatchKind::Name,
        content_match: None,
    });

    assert_eq!(result.source_name(), Some("tools"));
}

#[test]
fn source_name_falls_back_to_kit_name_when_plugin_id_empty() {
    let script = Arc::new(Script {
        name: "hello".to_string(),
        plugin_id: String::new(),
        plugin_title: None,
        kit_name: Some("legacy-kit".to_string()),
        ..Default::default()
    });

    let result = SearchResult::Script(ScriptMatch {
        script,
        score: 100,
        filename: "hello.ts".to_string(),
        match_indices: MatchIndices::default(),
        match_kind: ScriptMatchKind::Name,
        content_match: None,
    });

    assert_eq!(result.source_name(), Some("legacy-kit"));
}

#[test]
fn source_name_prefers_plugin_title_for_scriptlets() {
    let scriptlet = Arc::new(Scriptlet {
        name: "Open GitHub".to_string(),
        description: None,
        code: "open https://github.com".to_string(),
        tool: "open".to_string(),
        shortcut: None,
        keyword: None,
        group: Some("Quick Links".to_string()),
        plugin_id: "quicklinks".to_string(),
        plugin_title: Some("Quick Links Plugin".to_string()),
        file_path: None,
        command: Some("open-github".to_string()),
        alias: None,
    });

    let result = SearchResult::Scriptlet(ScriptletMatch {
        scriptlet,
        score: 100,
        display_file_path: None,
        match_indices: MatchIndices::default(),
    });

    assert_eq!(result.source_name(), Some("Quick Links Plugin"));
}

#[test]
fn source_name_falls_back_to_plugin_id_for_scriptlets() {
    let scriptlet = Arc::new(Scriptlet {
        name: "Open GitHub".to_string(),
        description: None,
        code: "open https://github.com".to_string(),
        tool: "open".to_string(),
        shortcut: None,
        keyword: None,
        group: Some("Quick Links".to_string()),
        plugin_id: "quicklinks".to_string(),
        plugin_title: None,
        file_path: None,
        command: Some("open-github".to_string()),
        alias: None,
    });

    let result = SearchResult::Scriptlet(ScriptletMatch {
        scriptlet,
        score: 100,
        display_file_path: None,
        match_indices: MatchIndices::default(),
    });

    assert_eq!(result.source_name(), Some("quicklinks"));
}

#[test]
fn source_name_falls_back_to_group_when_plugin_id_empty() {
    let scriptlet = Arc::new(Scriptlet {
        name: "Open GitHub".to_string(),
        description: None,
        code: "open https://github.com".to_string(),
        tool: "open".to_string(),
        shortcut: None,
        keyword: None,
        group: Some("Legacy Group".to_string()),
        plugin_id: String::new(),
        plugin_title: None,
        file_path: None,
        command: Some("open-github".to_string()),
        alias: None,
    });

    let result = SearchResult::Scriptlet(ScriptletMatch {
        scriptlet,
        score: 100,
        display_file_path: None,
        match_indices: MatchIndices::default(),
    });

    assert_eq!(result.source_name(), Some("Legacy Group"));
}

// ── Duplicate extension groups across plugins preserve attribution ──

#[test]
fn duplicate_group_names_across_plugins_keep_distinct_plugin_ids() {
    // Two plugins with the same H1 group name in their scriptlet bundles
    let scriptlet_a = Arc::new(Scriptlet {
        name: "Copy URL".to_string(),
        description: None,
        code: "echo url".to_string(),
        tool: "bash".to_string(),
        shortcut: None,
        keyword: None,
        group: Some("Quick Links".to_string()),
        plugin_id: "alpha-plugin".to_string(),
        plugin_title: Some("Alpha".to_string()),
        file_path: None,
        command: Some("copy-url".to_string()),
        alias: None,
    });

    let scriptlet_b = Arc::new(Scriptlet {
        name: "Open Docs".to_string(),
        description: None,
        code: "echo docs".to_string(),
        tool: "bash".to_string(),
        shortcut: None,
        keyword: None,
        group: Some("Quick Links".to_string()),
        plugin_id: "beta-plugin".to_string(),
        plugin_title: Some("Beta".to_string()),
        file_path: None,
        command: Some("open-docs".to_string()),
        alias: None,
    });

    // Both share group "Quick Links" but source_name resolves to their distinct plugins
    let result_a = SearchResult::Scriptlet(ScriptletMatch {
        scriptlet: scriptlet_a,
        score: 100,
        display_file_path: None,
        match_indices: MatchIndices::default(),
    });
    let result_b = SearchResult::Scriptlet(ScriptletMatch {
        scriptlet: scriptlet_b,
        score: 100,
        display_file_path: None,
        match_indices: MatchIndices::default(),
    });

    assert_eq!(result_a.source_name(), Some("Alpha"));
    assert_eq!(result_b.source_name(), Some("Beta"));
    // They are distinct despite sharing the same group name
    assert_ne!(result_a.source_name(), result_b.source_name());
}

// ── Plugin discovery populates plugin_id on scripts ────────────────

#[test]
fn discover_plugins_populates_manifest_on_roots() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let container = tmp.path().join("kit");

    // Create two plugin roots with manifests
    let alpha = container.join("alpha");
    fs::create_dir_all(alpha.join("scripts")).expect("mkdir");
    write_manifest(&alpha, "alpha", "Alpha Plugin");

    let beta = container.join("beta");
    fs::create_dir_all(beta.join("scripts")).expect("mkdir");
    write_manifest(&beta, "beta", "Beta Plugin");

    let index = discover_plugins_in(&container).expect("discover");
    assert_eq!(index.plugins.len(), 2);

    assert_eq!(index.plugins[0].id, "alpha");
    assert_eq!(index.plugins[0].manifest.title, "Alpha Plugin");
    assert_eq!(index.plugins[1].id, "beta");
    assert_eq!(index.plugins[1].manifest.title, "Beta Plugin");
}

// ── Source-level regression: loader uses discover_plugins ───────────

const LOADER_SOURCE: &str = include_str!("../src/scripts/loader.rs");
const SCRIPTLET_LOADER_SOURCE: &str = include_str!("../src/scripts/scriptlet_loader/loading.rs");

#[test]
fn script_loader_uses_discover_plugins() {
    assert!(
        LOADER_SOURCE.contains("crate::plugins::discover_plugins()"),
        "read_scripts() must consume discover_plugins() for plugin-scoped loading"
    );
}

#[test]
fn script_loader_sets_plugin_id_on_scripts() {
    assert!(
        LOADER_SOURCE.contains("plugin_id: plugin.id.clone()"),
        "read_scripts() must set plugin_id from the discovered plugin"
    );
}

#[test]
fn script_loader_sets_plugin_title_on_scripts() {
    assert!(
        LOADER_SOURCE.contains("plugin_title: Some(plugin.manifest.title.clone())"),
        "read_scripts() must set plugin_title from the plugin manifest"
    );
}

#[test]
fn scriptlet_loader_uses_discover_plugins() {
    assert!(
        SCRIPTLET_LOADER_SOURCE.contains("crate::plugins::discover_plugins()"),
        "load_scriptlets() must consume discover_plugins() for plugin-scoped loading"
    );
}

#[test]
fn scriptlet_loader_sets_plugin_id_on_scriptlets() {
    assert!(
        SCRIPTLET_LOADER_SOURCE.contains("plugin_id: plugin.id.clone()"),
        "load_scriptlets() must set plugin_id from the discovered plugin"
    );
}

#[test]
fn scriptlet_loader_sets_plugin_title_on_scriptlets() {
    assert!(
        SCRIPTLET_LOADER_SOURCE.contains("plugin_title: Some(plugin.manifest.title.clone())"),
        "load_scriptlets() must set plugin_title from the plugin manifest"
    );
}

// ── ACP skill enumeration uses discover_plugin_skills ──────────────

const ACP_VIEW_SOURCE: &str = include_str!("../src/ai/acp/view.rs");

#[test]
fn acp_view_uses_discover_plugin_skills_for_slash_commands() {
    assert!(
        ACP_VIEW_SOURCE.contains("crate::plugins::discover_plugins()"),
        "ACP view must use discover_plugins() for skill enumeration"
    );
    assert!(
        ACP_VIEW_SOURCE.contains("crate::plugins::discover_plugin_skills("),
        "ACP view must use discover_plugin_skills() for skill enumeration"
    );
}

#[test]
fn acp_view_does_not_manually_scan_kit_container_for_skills() {
    // The old pattern manually scanned kit/*/skills/ — this is now
    // replaced by discover_plugin_skills() which routes through the
    // canonical plugin index.
    assert!(
        !ACP_VIEW_SOURCE.contains("kit_container"),
        "ACP view must not use manual kit_container scanning for skills"
    );
}
