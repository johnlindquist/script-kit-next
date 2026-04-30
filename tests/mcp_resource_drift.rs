//! Drift audits for hand-written MCP reference resources.
//!
//! Source-of-truth principle for Oracle-Session `mcp-resource-doc-drift-tests`:
//! tests MUST NOT scrape Rust source or duplicate dispatch match arms. Each
//! resource exposes one runtime accessor (e.g.
//! `stdin_commands::all_external_command_verbs`) and the resource payload
//! wraps its canonical declaration list in `<!-- drift-audit:<marker>:start -->`
//! / `:end` markers. This harness parses the marker block, normalizes each
//! declared entry, and diffs it against the accessor-derived expected set.
//!
//! A failure looks like:
//!
//! ```text
//! MCP resource drift: kit://trigger-builtins
//! expected source: src/builtins/trigger_registry.rs::all_trigger_builtin_command_ids()
//! payload rule: only lines inside `<!-- drift-audit:trigger-builtin-ids:start -->` / ...
//! missing from resource payload:
//!   - builtin/current-app-commands
//! unexpected in resource payload: <none>
//! ```

use std::collections::BTreeSet;
use std::fmt;
use std::sync::Arc;

use script_kit_gpui::builtins::trigger_registry;
use script_kit_gpui::config::normalize_builtin_identifier;
use script_kit_gpui::mcp_resources;
use script_kit_gpui::scripts::{Script, Scriptlet};
use script_kit_gpui::stdin_commands;

trait ResourceDriftAudit {
    fn resource_id(&self) -> &'static str;
    fn expected_source(&self) -> &'static str;
    fn marker(&self) -> &'static str;
    fn expected_set(&self) -> BTreeSet<String>;
    fn payload_set(&self) -> BTreeSet<String>;

    fn ignore_expected_entries(&self) -> BTreeSet<String> {
        BTreeSet::new()
    }

    fn ignore_payload_entries(&self) -> BTreeSet<String> {
        BTreeSet::new()
    }

    fn audit(&self) -> Result<(), DriftReport> {
        let expected = subtract(self.expected_set(), self.ignore_expected_entries());
        let payload = subtract(self.payload_set(), self.ignore_payload_entries());

        let missing = expected
            .difference(&payload)
            .cloned()
            .collect::<BTreeSet<_>>();
        let unexpected = payload
            .difference(&expected)
            .cloned()
            .collect::<BTreeSet<_>>();

        if missing.is_empty() && unexpected.is_empty() {
            Ok(())
        } else {
            Err(DriftReport {
                resource_id: self.resource_id(),
                expected_source: self.expected_source(),
                marker: self.marker(),
                missing,
                unexpected,
            })
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DriftReport {
    resource_id: &'static str,
    expected_source: &'static str,
    marker: &'static str,
    missing: BTreeSet<String>,
    unexpected: BTreeSet<String>,
}

impl fmt::Display for DriftReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "MCP resource drift: {}", self.resource_id)?;
        writeln!(f, "expected source: {}", self.expected_source)?;
        writeln!(
            f,
            "payload rule: only lines inside `<!-- drift-audit:{0}:start -->` / \
             `<!-- drift-audit:{0}:end -->` matching `- `name`:` count",
            self.marker
        )?;
        write_set(f, "missing from resource payload", &self.missing)?;
        write_set(f, "unexpected in resource payload", &self.unexpected)?;
        Ok(())
    }
}

fn write_set(f: &mut fmt::Formatter<'_>, title: &str, values: &BTreeSet<String>) -> fmt::Result {
    if values.is_empty() {
        writeln!(f, "{title}: <none>")?;
        return Ok(());
    }
    writeln!(f, "{title}:")?;
    for value in values {
        writeln!(f, "  - {value}")?;
    }
    Ok(())
}

fn subtract(mut values: BTreeSet<String>, ignored: BTreeSet<String>) -> BTreeSet<String> {
    for value in ignored {
        values.remove(&value);
    }
    values
}

fn assert_audit_ok<A: ResourceDriftAudit>(audit: A) {
    if let Err(report) = audit.audit() {
        panic!("{report}");
    }
}

fn read_resource_text(uri: &str) -> String {
    let scripts: Vec<Arc<Script>> = Vec::new();
    let scriptlets: Vec<Arc<Scriptlet>> = Vec::new();
    mcp_resources::read_resource(uri, &scripts, &scriptlets, None)
        .unwrap_or_else(|err| panic!("{uri} should resolve: {err}"))
        .text
}

fn declared_payload_entries(
    payload: &str,
    marker: &str,
    normalize: fn(&str) -> String,
) -> BTreeSet<String> {
    marked_section(payload, marker)
        .lines()
        .filter_map(declared_backtick_entry)
        .map(normalize)
        .filter(|value| !value.is_empty())
        .collect()
}

fn marked_section<'a>(payload: &'a str, marker: &str) -> &'a str {
    let start = format!("<!-- drift-audit:{marker}:start -->");
    let end = format!("<!-- drift-audit:{marker}:end -->");
    let Some((_, after_start)) = payload.split_once(start.as_str()) else {
        return "";
    };
    let Some((section, _)) = after_start.split_once(end.as_str()) else {
        return "";
    };
    section
}

fn declared_backtick_entry(line: &str) -> Option<&str> {
    let line = line.trim_start();
    let rest = line.strip_prefix("- `")?;
    let (name, suffix) = rest.split_once('`')?;
    if suffix.starts_with(':') {
        Some(name.trim())
    } else {
        None
    }
}

fn normalize_stdin_verb(raw: &str) -> String {
    raw.trim().to_string()
}

fn normalize_builtin_command_id(raw: &str) -> String {
    let identifier = normalize_builtin_identifier(raw.trim());
    let identifier = identifier.replace('_', "-").to_ascii_lowercase();
    format!("builtin/{identifier}")
}

fn set_from_slice(values: &[&str], normalize: fn(&str) -> String) -> BTreeSet<String> {
    values.iter().copied().map(normalize).collect()
}

struct StdinVerbResource;

impl ResourceDriftAudit for StdinVerbResource {
    fn resource_id(&self) -> &'static str {
        mcp_resources::STDIN_COMMANDS_REFERENCE_URI
    }

    fn expected_source(&self) -> &'static str {
        "src/stdin_commands/mod.rs::all_external_command_verbs()"
    }

    fn marker(&self) -> &'static str {
        "stdin-verbs"
    }

    fn expected_set(&self) -> BTreeSet<String> {
        set_from_slice(
            stdin_commands::all_external_command_verbs(),
            normalize_stdin_verb,
        )
    }

    fn payload_set(&self) -> BTreeSet<String> {
        declared_payload_entries(
            &read_resource_text(self.resource_id()),
            self.marker(),
            normalize_stdin_verb,
        )
    }
}

struct TriggerBuiltinResource;

impl ResourceDriftAudit for TriggerBuiltinResource {
    fn resource_id(&self) -> &'static str {
        mcp_resources::TRIGGER_BUILTINS_REFERENCE_URI
    }

    fn expected_source(&self) -> &'static str {
        "src/builtins/trigger_registry.rs::all_trigger_builtin_command_ids()"
    }

    fn marker(&self) -> &'static str {
        "trigger-builtin-ids"
    }

    fn expected_set(&self) -> BTreeSet<String> {
        set_from_slice(
            trigger_registry::all_trigger_builtin_command_ids(),
            normalize_builtin_command_id,
        )
    }

    fn payload_set(&self) -> BTreeSet<String> {
        declared_payload_entries(
            &read_resource_text(self.resource_id()),
            self.marker(),
            normalize_builtin_command_id,
        )
    }
}

#[test]
fn kit_stdin_commands_resource_matches_external_command_verbs() {
    assert_audit_ok(StdinVerbResource);
}

#[test]
fn kit_trigger_builtins_resource_matches_trigger_builtin_registry() {
    assert_audit_ok(TriggerBuiltinResource);
}

#[test]
fn stdin_commands_resource_is_listed_in_definitions() {
    let uris: Vec<String> = mcp_resources::get_resource_definitions()
        .into_iter()
        .map(|r| r.uri)
        .collect();
    assert!(
        uris.iter()
            .any(|u| u == mcp_resources::STDIN_COMMANDS_REFERENCE_URI),
        "kit://stdin-commands must appear in get_resource_definitions()"
    );
    assert!(
        uris.iter()
            .any(|u| u == mcp_resources::TRIGGER_BUILTINS_REFERENCE_URI),
        "kit://trigger-builtins must appear in get_resource_definitions()"
    );
}

#[test]
fn drift_audit_parser_rejects_entries_outside_marker_block() {
    let payload = "\
<!-- drift-audit:stdin-verbs:start -->
- `run`: Dispatches Run.
- `show`: Dispatches Show.
<!-- drift-audit:stdin-verbs:end -->

Example outside the marker block:
- `triggerBuiltin`: must not be counted here.

```json
{\"type\":\"setFilter\"}
```
";
    let entries = declared_payload_entries(payload, "stdin-verbs", normalize_stdin_verb);
    let expected: BTreeSet<String> = ["run", "show"].into_iter().map(String::from).collect();
    assert_eq!(entries, expected);
}

#[test]
fn drift_audit_parser_ignores_json_examples() {
    let payload = "\
<!-- drift-audit:trigger-builtin-ids:start -->
- `builtin/design-gallery`: Opens the gallery.
<!-- drift-audit:trigger-builtin-ids:end -->

Example:
```json
{\"builtinId\":\"builtin/clipboard-history\"}
```
";
    let entries =
        declared_payload_entries(payload, "trigger-builtin-ids", normalize_builtin_command_id);
    let expected: BTreeSet<String> = ["builtin/design-gallery"]
        .into_iter()
        .map(String::from)
        .collect();
    assert_eq!(entries, expected);
}
