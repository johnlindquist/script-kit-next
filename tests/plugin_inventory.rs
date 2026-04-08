use std::fs;

use script_kit_gpui::plugins::{
    discover_plugin_skills, discover_plugins_in, read_plugin_manifest, synthesize_plugin_manifest,
    PluginIndex,
};

/// Helper: create a temp dir tree and return the container path.
fn make_container(tmp: &std::path::Path, plugins: &[(&str, Option<&str>)]) -> std::path::PathBuf {
    let container = tmp.join("kit");
    for (id, manifest_json) in plugins {
        let plugin_root = container.join(id);
        fs::create_dir_all(plugin_root.join("scripts")).expect("create scripts dir");
        fs::create_dir_all(plugin_root.join("extensions")).expect("create extensions dir");
        if let Some(json) = manifest_json {
            fs::write(plugin_root.join("plugin.json"), json).expect("write plugin.json");
        }
    }
    container
}

// ── discover_plugins ────────────────────────────────────────────────

#[test]
fn discover_finds_both_plugin_roots_sorted_by_id() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let manifest_main = r#"{"id":"main","title":"Main","description":"User plugin"}"#;
    let manifest_tools = r#"{"id":"tools","title":"Tools","description":"Dev tools"}"#;
    let container = make_container(
        tmp.path(),
        &[
            ("main", Some(manifest_main)),
            ("tools", Some(manifest_tools)),
        ],
    );

    let index = discover_plugins_in(&container).expect("discover");
    assert_eq!(
        index.plugins.len(),
        2,
        "should discover exactly two plugins"
    );
    assert_eq!(index.plugins[0].id, "main");
    assert_eq!(index.plugins[1].id, "tools");
}

#[test]
fn discover_returns_empty_index_for_missing_container() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let container = tmp.path().join("nonexistent");
    let index = discover_plugins_in(&container).expect("discover");
    assert!(index.plugins.is_empty());
}

#[test]
fn discover_skips_files_in_container() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let container = make_container(tmp.path(), &[("alpha", None)]);
    // Drop a loose file next to the plugin dir
    fs::write(container.join("README.md"), "ignore me").expect("write file");

    let index = discover_plugins_in(&container).expect("discover");
    assert_eq!(index.plugins.len(), 1);
    assert_eq!(index.plugins[0].id, "alpha");
}

// ── read_plugin_manifest ────────────────────────────────────────────

#[test]
fn manifest_prefers_plugin_json() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path().join("myplugin");
    fs::create_dir_all(&root).expect("mkdir");
    let json = r#"{"id":"myplugin","title":"My Plugin","description":"test","version":"1.0.0"}"#;
    fs::write(root.join("plugin.json"), json).expect("write");

    let manifest = read_plugin_manifest(&root).expect("read");
    assert_eq!(manifest.id, "myplugin");
    assert_eq!(manifest.title, "My Plugin");
    assert_eq!(manifest.version, "1.0.0");
}

#[test]
fn manifest_falls_back_to_package_json() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path().join("pkg-plugin");
    fs::create_dir_all(&root).expect("mkdir");
    let pkg = r#"{"name":"pkg-plugin","description":"From package.json","version":"2.0.0","author":{"name":"Alice"},"repository":{"url":"https://github.com/example/repo"}}"#;
    fs::write(root.join("package.json"), pkg).expect("write");

    let manifest = read_plugin_manifest(&root).expect("read");
    assert_eq!(manifest.id, "pkg-plugin");
    assert_eq!(manifest.title, "pkg-plugin");
    assert_eq!(manifest.description, "From package.json");
    assert_eq!(manifest.version, "2.0.0");
    assert_eq!(manifest.author, "Alice");
    assert_eq!(manifest.repo_url, "https://github.com/example/repo");
}

#[test]
fn manifest_synthesizes_from_directory_name() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path().join("bare-dir");
    fs::create_dir_all(&root).expect("mkdir");

    let manifest = read_plugin_manifest(&root).expect("read");
    assert_eq!(manifest.id, "bare-dir");
    assert_eq!(manifest.title, "bare-dir");
    assert!(manifest.description.is_empty());
}

#[test]
fn synthesize_from_package_json_string_author() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path().join("str-author");
    fs::create_dir_all(&root).expect("mkdir");
    let pkg = r#"{"name":"str-author","author":"Bob","repository":"https://example.com/repo.git"}"#;
    fs::write(root.join("package.json"), pkg).expect("write");

    let manifest = synthesize_plugin_manifest(&root).expect("synth");
    assert_eq!(manifest.author, "Bob");
    assert_eq!(manifest.repo_url, "https://example.com/repo.git");
}

// ── discover_plugin_skills ──────────────────────────────────────────

#[test]
fn skills_found_under_plugin_roots() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let container = tmp.path().join("kit");

    // Plugin "authoring" with two skills
    let authoring = container.join("authoring");
    let skill_a = authoring.join("skills").join("scriptlets");
    let skill_b = authoring.join("skills").join("config");
    fs::create_dir_all(&skill_a).expect("mkdir");
    fs::create_dir_all(&skill_b).expect("mkdir");
    fs::write(skill_a.join("SKILL.md"), "# Scriptlets skill").expect("write");
    fs::write(skill_b.join("SKILL.md"), "# Config skill").expect("write");
    let manifest = r#"{"id":"authoring","title":"Authoring"}"#;
    fs::write(authoring.join("plugin.json"), manifest).expect("write");

    // Plugin "tools" with no skills dir
    let tools = container.join("tools");
    fs::create_dir_all(tools.join("scripts")).expect("mkdir");
    let manifest2 = r#"{"id":"tools","title":"Tools"}"#;
    fs::write(tools.join("plugin.json"), manifest2).expect("write");

    let index = discover_plugins_in(&container).expect("discover");
    assert_eq!(index.plugins.len(), 2);

    let skills = discover_plugin_skills(&index).expect("skills");
    assert_eq!(skills.len(), 2, "should find exactly two skills");

    // Sorted by (plugin_id, skill_id)
    assert_eq!(skills[0].plugin_id, "authoring");
    assert_eq!(skills[0].skill_id, "config");
    assert_eq!(skills[1].plugin_id, "authoring");
    assert_eq!(skills[1].skill_id, "scriptlets");
}

#[test]
fn skills_ignores_dirs_without_skill_md() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let container = tmp.path().join("kit");
    let plugin = container.join("main");
    let skill_dir = plugin.join("skills").join("draft");
    fs::create_dir_all(&skill_dir).expect("mkdir");
    // No SKILL.md inside
    fs::write(skill_dir.join("notes.txt"), "wip").expect("write");
    let manifest = r#"{"id":"main","title":"Main"}"#;
    fs::write(plugin.join("plugin.json"), manifest).expect("write");

    let index = discover_plugins_in(&container).expect("discover");
    let skills = discover_plugin_skills(&index).expect("skills");
    assert!(skills.is_empty(), "no SKILL.md means no skill discovered");
}

#[test]
fn skills_records_plugin_id_on_every_skill() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let container = tmp.path().join("kit");

    // Two plugins each with one skill
    for (pid, sid) in &[("alpha", "s1"), ("beta", "s2")] {
        let plugin = container.join(pid);
        let skill = plugin.join("skills").join(sid);
        fs::create_dir_all(&skill).expect("mkdir");
        fs::write(skill.join("SKILL.md"), format!("# {sid}")).expect("write");
        let m = format!(r#"{{"id":"{pid}","title":"{pid}"}}"#);
        fs::write(plugin.join("plugin.json"), m).expect("write");
    }

    let index = discover_plugins_in(&container).expect("discover");
    let skills = discover_plugin_skills(&index).expect("skills");
    assert_eq!(skills.len(), 2);
    assert_eq!(skills[0].plugin_id, "alpha");
    assert_eq!(skills[0].skill_id, "s1");
    assert_eq!(skills[1].plugin_id, "beta");
    assert_eq!(skills[1].skill_id, "s2");
}

// ── Skill metadata parsing ──────────────────────────────────────────

#[test]
fn skills_parse_title_from_frontmatter() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let container = tmp.path().join("kit");
    let plugin = container.join("authoring");
    let skill = plugin.join("skills").join("scriptlets");
    fs::create_dir_all(&skill).expect("mkdir");
    fs::write(
        skill.join("SKILL.md"),
        "---\ntitle: Scriptlet Authoring\ndescription: Create markdown extension bundles\n---\n# Body",
    )
    .expect("write");
    let manifest = r#"{"id":"authoring","title":"Authoring"}"#;
    fs::write(plugin.join("plugin.json"), manifest).expect("write");

    let index = discover_plugins_in(&container).expect("discover");
    let skills = discover_plugin_skills(&index).expect("skills");
    assert_eq!(skills.len(), 1);
    assert_eq!(skills[0].title, "Scriptlet Authoring");
    assert_eq!(skills[0].description, "Create markdown extension bundles");
    assert_eq!(skills[0].plugin_title, "Authoring");
    assert_eq!(skills[0].plugin_id, "authoring");
}

#[test]
fn skills_title_falls_back_to_h1_then_slug() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let container = tmp.path().join("kit");

    // Skill with H1 but no frontmatter title
    let plugin_a = container.join("alpha");
    let skill_a = plugin_a.join("skills").join("review");
    fs::create_dir_all(&skill_a).expect("mkdir");
    fs::write(skill_a.join("SKILL.md"), "# Code Review\nBody text").expect("write");
    let m_a = r#"{"id":"alpha","title":"Alpha"}"#;
    fs::write(plugin_a.join("plugin.json"), m_a).expect("write");

    // Skill with no frontmatter and no H1 — falls back to slug
    let plugin_b = container.join("beta");
    let skill_b = plugin_b.join("skills").join("bare-slug");
    fs::create_dir_all(&skill_b).expect("mkdir");
    fs::write(skill_b.join("SKILL.md"), "Just text, no heading").expect("write");
    let m_b = r#"{"id":"beta","title":"Beta"}"#;
    fs::write(plugin_b.join("plugin.json"), m_b).expect("write");

    let index = discover_plugins_in(&container).expect("discover");
    let skills = discover_plugin_skills(&index).expect("skills");
    assert_eq!(skills.len(), 2);

    // alpha/review: title from H1
    assert_eq!(skills[0].plugin_id, "alpha");
    assert_eq!(skills[0].title, "Code Review");

    // beta/bare-slug: title falls back to skill_id
    assert_eq!(skills[1].plugin_id, "beta");
    assert_eq!(skills[1].title, "bare-slug");
}

#[test]
fn skills_duplicate_slugs_across_plugins_preserved() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let container = tmp.path().join("kit");

    // Two plugins both define a "review" skill
    for pid in &["alpha", "beta"] {
        let plugin = container.join(pid);
        let skill = plugin.join("skills").join("review");
        fs::create_dir_all(&skill).expect("mkdir");
        fs::write(
            skill.join("SKILL.md"),
            format!("---\ntitle: {pid} Review\n---\n# Body"),
        )
        .expect("write");
        let m = format!(r#"{{"id":"{pid}","title":"{pid}"}}"#);
        fs::write(plugin.join("plugin.json"), m).expect("write");
    }

    let index = discover_plugins_in(&container).expect("discover");
    let skills = discover_plugin_skills(&index).expect("skills");

    // Both skills must be preserved — no dedup by bare skill_id
    assert_eq!(skills.len(), 2, "duplicate skill slugs must not collapse");
    assert_eq!(skills[0].plugin_id, "alpha");
    assert_eq!(skills[0].skill_id, "review");
    assert_eq!(skills[0].title, "alpha Review");
    assert_eq!(skills[1].plugin_id, "beta");
    assert_eq!(skills[1].skill_id, "review");
    assert_eq!(skills[1].title, "beta Review");
}

// ── PluginIndex default ─────────────────────────────────────────────

#[test]
fn plugin_index_default_is_empty() {
    let index = PluginIndex::default();
    assert!(index.plugins.is_empty());
}

// ── PluginManifest serde roundtrip ──────────────────────────────────

#[test]
fn manifest_serde_roundtrip() {
    let json = r#"{"id":"test","title":"Test","description":"desc","version":"0.1.0","author":"me","repoUrl":"https://example.com"}"#;
    let manifest: script_kit_gpui::plugins::PluginManifest =
        serde_json::from_str(json).expect("deserialize");
    assert_eq!(manifest.id, "test");
    assert_eq!(manifest.repo_url, "https://example.com");

    let serialized = serde_json::to_string(&manifest).expect("serialize");
    let roundtripped: script_kit_gpui::plugins::PluginManifest =
        serde_json::from_str(&serialized).expect("roundtrip");
    assert_eq!(manifest, roundtripped);
}
