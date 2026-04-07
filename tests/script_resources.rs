//! Integration tests for the kit://scripts, kit://scriptlets, and kit://sdk-reference
//! MCP resources. These use fixture data rather than touching a real ~/.scriptkit directory.

use std::sync::Arc;

use script_kit_gpui::mcp_resources::{
    self, ScriptResourceEntry, ScriptletsResourceDocument, ScriptsResourceDocument,
    SdkReferenceDocument, SCRIPTLETS_RESOURCE_SCHEMA_VERSION, SCRIPTS_RESOURCE_SCHEMA_VERSION,
    SDK_REFERENCE_SCHEMA_VERSION,
};
use script_kit_gpui::scripts::{Script, Scriptlet};

fn fixture_scripts() -> Vec<Arc<Script>> {
    vec![
        Arc::new(Script {
            name: "hello-world".into(),
            path: "/tmp/test-scriptkit/scripts/hello-world.ts".into(),
            extension: "ts".into(),
            description: Some("A greeting script".into()),
            icon: None,
            alias: Some("hw".into()),
            shortcut: Some("opt h".into()),
            typed_metadata: None,
            schema: None,
            kit_name: Some("main".into()),
            body: None,
        }),
        Arc::new(Script {
            name: "fetch-data".into(),
            path: "/tmp/test-scriptkit/scripts/fetch-data.ts".into(),
            extension: "ts".into(),
            description: None,
            icon: None,
            alias: None,
            shortcut: None,
            typed_metadata: None,
            schema: None,
            kit_name: None,
            body: None,
        }),
    ]
}

fn fixture_scriptlets() -> Vec<Arc<Scriptlet>> {
    vec![
        Arc::new(Scriptlet {
            name: "Open GitHub".into(),
            description: Some("Opens GitHub in browser".into()),
            code: "open https://github.com".into(),
            tool: "open".into(),
            shortcut: None,
            keyword: Some("!gh".into()),
            group: Some("Productivity".into()),
            file_path: Some("/tmp/test-scriptkit/kit/main/extensions/urls.md#open-github".into()),
            command: Some("open-github".into()),
            alias: None,
        }),
        Arc::new(Scriptlet {
            name: "Paste Greeting".into(),
            description: None,
            code: "Hello from Script Kit!".into(),
            tool: "paste".into(),
            shortcut: Some("cmd shift g".into()),
            keyword: None,
            group: None,
            file_path: None,
            command: None,
            alias: None,
        }),
    ]
}

// =======================================================
// kit://scripts
// =======================================================

#[test]
fn kit_scripts_returns_schema_versioned_json() {
    let scripts = fixture_scripts();
    let content =
        mcp_resources::read_resource("kit://scripts", &scripts, &[], None).expect("should resolve");

    let doc: ScriptsResourceDocument =
        serde_json::from_str(&content.text).expect("valid JSON envelope");

    assert_eq!(doc.schema_version, SCRIPTS_RESOURCE_SCHEMA_VERSION);
    assert_eq!(doc.count, 2);
    assert_eq!(doc.scripts.len(), 2);
}

#[test]
fn kit_scripts_entries_carry_full_metadata() {
    let scripts = fixture_scripts();
    let content = mcp_resources::read_resource("kit://scripts", &scripts, &[], None).unwrap();
    let doc: ScriptsResourceDocument = serde_json::from_str(&content.text).unwrap();

    let first = &doc.scripts[0];
    assert_eq!(first.name, "hello-world");
    assert_eq!(first.path, "/tmp/test-scriptkit/scripts/hello-world.ts");
    assert_eq!(first.extension, "ts");
    assert_eq!(first.description, Some("A greeting script".into()));
    assert!(!first.has_schema);
}

#[test]
fn kit_scripts_empty_is_deterministic() {
    let content =
        mcp_resources::read_resource("kit://scripts", &[], &[], None).expect("should resolve");
    let doc: ScriptsResourceDocument = serde_json::from_str(&content.text).unwrap();

    assert_eq!(doc.schema_version, SCRIPTS_RESOURCE_SCHEMA_VERSION);
    assert_eq!(doc.count, 0);
    assert!(doc.scripts.is_empty());

    // Repeated reads produce identical output (idempotent)
    let content2 =
        mcp_resources::read_resource("kit://scripts", &[], &[], None).expect("should resolve");
    assert_eq!(content.text, content2.text);
}

#[test]
fn kit_scripts_json_uses_camel_case_envelope() {
    let scripts = fixture_scripts();
    let content = mcp_resources::read_resource("kit://scripts", &scripts, &[], None).unwrap();
    let value: serde_json::Value = serde_json::from_str(&content.text).unwrap();

    // Envelope fields are camelCase
    assert!(value.get("schemaVersion").is_some());
    assert!(value.get("count").is_some());
    assert!(value.get("scripts").is_some());

    // Entry fields present
    let first = &value["scripts"][0];
    assert!(first.get("has_schema").is_some());
}

// =======================================================
// kit://scriptlets
// =======================================================

#[test]
fn kit_scriptlets_returns_schema_versioned_json() {
    let scriptlets = fixture_scriptlets();
    let content = mcp_resources::read_resource("kit://scriptlets", &[], &scriptlets, None)
        .expect("should resolve");

    let doc: ScriptletsResourceDocument =
        serde_json::from_str(&content.text).expect("valid JSON envelope");

    assert_eq!(doc.schema_version, SCRIPTLETS_RESOURCE_SCHEMA_VERSION);
    assert_eq!(doc.count, 2);
    assert_eq!(doc.scriptlets.len(), 2);
}

#[test]
fn kit_scriptlets_entries_carry_optional_fields() {
    let scriptlets = fixture_scriptlets();
    let content = mcp_resources::read_resource("kit://scriptlets", &[], &scriptlets, None).unwrap();
    let doc: ScriptletsResourceDocument = serde_json::from_str(&content.text).unwrap();

    let first = &doc.scriptlets[0];
    assert_eq!(first.name, "Open GitHub");
    assert_eq!(first.tool, "open");
    assert_eq!(first.description, Some("Opens GitHub in browser".into()));
    assert_eq!(first.keyword, Some("!gh".into()));
    assert_eq!(first.group, Some("Productivity".into()));

    // Second entry has no keyword/group — they should be absent from JSON
    let second = &doc.scriptlets[1];
    assert_eq!(second.name, "Paste Greeting");
    assert!(second.keyword.is_none());
    assert!(second.group.is_none());
}

#[test]
fn kit_scriptlets_empty_is_deterministic() {
    let content =
        mcp_resources::read_resource("kit://scriptlets", &[], &[], None).expect("should resolve");
    let doc: ScriptletsResourceDocument = serde_json::from_str(&content.text).unwrap();
    assert_eq!(doc.count, 0);

    let content2 =
        mcp_resources::read_resource("kit://scriptlets", &[], &[], None).expect("should resolve");
    assert_eq!(content.text, content2.text);
}

// =======================================================
// kit://sdk-reference
// =======================================================

#[test]
fn sdk_reference_returns_versioned_document() {
    let content = mcp_resources::read_resource("kit://sdk-reference", &[], &[], None)
        .expect("should resolve");

    let doc: SdkReferenceDocument =
        serde_json::from_str(&content.text).expect("valid JSON document");

    assert_eq!(doc.schema_version, SDK_REFERENCE_SCHEMA_VERSION);
    assert_eq!(doc.sdk_package, "@scriptkit/sdk");
}

#[test]
fn sdk_reference_includes_core_prompt_functions() {
    let content = mcp_resources::read_resource("kit://sdk-reference", &[], &[], None).unwrap();
    let doc: SdkReferenceDocument = serde_json::from_str(&content.text).unwrap();

    let names: Vec<&str> = doc.functions.iter().map(|f| f.name.as_str()).collect();
    for expected in &["arg", "div", "editor", "term", "drop", "template"] {
        assert!(
            names.contains(expected),
            "SDK reference should include {expected}()"
        );
    }
}

#[test]
fn sdk_reference_includes_system_and_clipboard_functions() {
    let content = mcp_resources::read_resource("kit://sdk-reference", &[], &[], None).unwrap();
    let doc: SdkReferenceDocument = serde_json::from_str(&content.text).unwrap();

    let names: Vec<&str> = doc.functions.iter().map(|f| f.name.as_str()).collect();
    for expected in &["exec", "copy", "paste", "clipboard", "notify"] {
        assert!(
            names.contains(expected),
            "SDK reference should include {expected}()"
        );
    }
}

#[test]
fn sdk_reference_documents_script_directory_and_patterns() {
    let content = mcp_resources::read_resource("kit://sdk-reference", &[], &[], None).unwrap();
    let doc: SdkReferenceDocument = serde_json::from_str(&content.text).unwrap();

    assert!(doc.script_directory.contains("kit/main/scripts"));
    assert!(doc.scriptlet_pattern.contains("extensions"));
    assert!(doc.metadata_format.contains("export const metadata"));
}

#[test]
fn sdk_reference_is_idempotent() {
    let a =
        mcp_resources::read_resource("kit://sdk-reference", &[], &[], None).expect("first read");
    let b =
        mcp_resources::read_resource("kit://sdk-reference", &[], &[], None).expect("second read");
    assert_eq!(a.text, b.text);
}

#[test]
fn sdk_reference_roundtrips_through_serde() {
    let content = mcp_resources::read_resource("kit://sdk-reference", &[], &[], None).unwrap();
    let doc: SdkReferenceDocument = serde_json::from_str(&content.text).unwrap();
    let reserialized = serde_json::to_string_pretty(&doc).unwrap();
    assert_eq!(content.text, reserialized);
}

// =======================================================
// Resource listing includes the new resources
// =======================================================

#[test]
fn resource_list_includes_all_new_resources() {
    let resources = mcp_resources::get_resource_definitions();
    let uris: Vec<&str> = resources.iter().map(|r| r.uri.as_str()).collect();

    assert!(uris.contains(&"kit://scripts"), "missing kit://scripts");
    assert!(
        uris.contains(&"kit://scriptlets"),
        "missing kit://scriptlets"
    );
    assert!(
        uris.contains(&"kit://sdk-reference"),
        "missing kit://sdk-reference"
    );
}

#[test]
fn unknown_kit_uri_still_returns_error() {
    let result = mcp_resources::read_resource("kit://nonexistent", &[], &[], None);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Resource not found"));
}
