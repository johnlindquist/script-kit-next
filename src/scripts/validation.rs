//! Startup-time validation for loaded scripts.
//!
//! Oracle-Session `script-metadata-validation-fail-fast` PR1: introduce the
//! validation surface + collision detection. The goal is to make metadata
//! problems — especially duplicate `shortcut`, `alias`, `keyword`, or
//! `trigger` declarations — visible at load time instead of silently racing
//! at dispatch.
//!
//! This PR is the foundation: it defines the report types and a single
//! `validate_script_catalog` entry point that takes an already-loaded
//! `Vec<Arc<Script>>` and produces a [`ScriptCatalogReport`]. Follow-ups
//! plumb typed-metadata parse errors through the loader and expose a
//! `kit://failed-scripts` MCP resource + menu-bar badge count.
//!
//! Usage:
//!
//! ```ignore
//! let scripts = scripts::read_scripts();
//! let report = scripts::validate_script_catalog(scripts);
//! if !report.validation.failed_scripts.is_empty() {
//!     tracing::warn!(
//!         fatal = report.validation.fatal_count,
//!         warnings = report.validation.warning_count,
//!         "script_validation_found_failures",
//!     );
//! }
//! ```

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use super::types::Script;

/// Current schema version of the `ValidationReport` payload.
pub const VALIDATION_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ValidationSeverity {
    Fatal,
    Warning,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum BindingKind {
    Shortcut,
    Alias,
    Keyword,
    Trigger,
}

impl BindingKind {
    pub fn as_metadata_field(self) -> MetadataField {
        match self {
            BindingKind::Shortcut => MetadataField::Shortcut,
            BindingKind::Alias => MetadataField::Alias,
            BindingKind::Keyword => MetadataField::Keyword,
            BindingKind::Trigger => MetadataField::Trigger,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum MetadataField {
    Metadata,
    Schema,
    Name,
    Alias,
    Keyword,
    Trigger,
    Shortcut,
    Cron,
    Schedule,
    Watch,
    Unknown,
}

/// Discriminated failure kind. Serialized tag-first so operator tooling can
/// switch on `kind.kind` without knowing the full enum shape.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ScriptValidationKind {
    /// Typed-metadata parser reported a non-fatal error but failed to produce
    /// a usable `TypedMetadata`. The raw detail is the parser message.
    MetadataParse { detail: String },
    /// Schema parser error. Pulls straight from `schema_parser::SchemaParseResult`.
    SchemaParse { detail: String },
    /// The field declared a value that failed shape/grammar validation.
    InvalidValue { value: String, reason: String },
    /// Two or more scripts declared the same binding (shortcut/alias/keyword/trigger).
    DuplicateBinding { binding: BindingKind, value: String },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RelatedScript {
    pub path: PathBuf,
    pub name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ScriptValidationIssue {
    pub severity: ValidationSeverity,
    pub path: PathBuf,
    pub script_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<MetadataField>,
    pub message: String,
    pub kind: ScriptValidationKind,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub related: Vec<RelatedScript>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FailedScript {
    pub path: PathBuf,
    pub name: String,
    pub fatal: Arc<[ScriptValidationIssue]>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationReport {
    pub schema_version: u32,
    pub total_candidates: usize,
    pub valid_count: usize,
    pub fatal_count: usize,
    pub warning_count: usize,
    pub failed_scripts: Arc<[FailedScript]>,
    pub warnings: Arc<[ScriptValidationIssue]>,
}

/// Bundles the kept scripts + the validation report into one immutable
/// artifact the startup/index publisher can consume atomically.
#[derive(Clone, Debug)]
pub struct ScriptCatalogReport {
    pub scripts: Arc<[Arc<Script>]>,
    pub validation: Arc<ValidationReport>,
}

/// Normalize a binding value for collision comparison.
///
/// Shortcuts: lowercase + collapse internal whitespace so `"Cmd Shift K"` and
/// `"cmd  shift k"` collide. Alias/keyword: lowercase + trim. This is
/// deliberately loose — we'd rather false-positive a collision than miss
/// one. Per-binding grammar validation is a follow-up.
fn normalize_binding(kind: BindingKind, raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    match kind {
        BindingKind::Shortcut => {
            let mut out = String::with_capacity(trimmed.len());
            let mut first = true;
            for token in trimmed.split_whitespace() {
                if !first {
                    out.push(' ');
                }
                first = false;
                for ch in token.chars() {
                    out.extend(ch.to_lowercase());
                }
            }
            Some(out)
        }
        BindingKind::Alias | BindingKind::Keyword | BindingKind::Trigger => {
            Some(trimmed.to_lowercase())
        }
    }
}

fn script_bindings(script: &Script) -> Vec<(BindingKind, &str)> {
    let mut out = Vec::with_capacity(4);
    if let Some(v) = script.shortcut.as_deref() {
        out.push((BindingKind::Shortcut, v));
    }
    if let Some(v) = script.alias.as_deref() {
        out.push((BindingKind::Alias, v));
    }
    if let Some(meta) = script.typed_metadata.as_ref() {
        if let Some(v) = meta.keyword.as_deref() {
            out.push((BindingKind::Keyword, v));
        }
        if let Some(v) = meta.extra.get("trigger").and_then(|v| v.as_str()) {
            out.push((BindingKind::Trigger, v));
        }
    }
    out
}

/// Detect duplicate shortcut/alias/keyword/trigger declarations across
/// the catalog. Emits one `DuplicateBinding` issue per offending script so
/// both sides show up in the failure report with pointers at each other.
pub fn detect_binding_collisions(scripts: &[Arc<Script>]) -> Vec<ScriptValidationIssue> {
    let mut buckets: HashMap<(BindingKind, String), Vec<RelatedScript>> = HashMap::new();
    for script in scripts {
        for (kind, raw) in script_bindings(script) {
            if let Some(value) = normalize_binding(kind, raw) {
                buckets
                    .entry((kind, value))
                    .or_default()
                    .push(RelatedScript {
                        path: script.path.clone(),
                        name: script.name.clone(),
                    });
            }
        }
    }

    let mut out = Vec::new();
    for ((binding, value), owners) in buckets {
        if owners.len() < 2 {
            continue;
        }
        for owner in &owners {
            let related: Vec<RelatedScript> = owners
                .iter()
                .filter(|peer| peer.path != owner.path)
                .cloned()
                .collect();
            out.push(ScriptValidationIssue {
                severity: ValidationSeverity::Fatal,
                path: owner.path.clone(),
                script_name: owner.name.clone(),
                field: Some(binding.as_metadata_field()),
                message: format!(
                    "{:?} `{}` is declared by {} scripts",
                    binding,
                    value,
                    owners.len()
                ),
                kind: ScriptValidationKind::DuplicateBinding {
                    binding,
                    value: value.clone(),
                },
                related,
            });
        }
    }
    // Sort for deterministic output — buckets iterate in hash order.
    out.sort_by(|a, b| {
        a.path
            .cmp(&b.path)
            .then_with(|| format!("{:?}", a.kind).cmp(&format!("{:?}", b.kind)))
    });
    out
}

/// Validate a catalog of already-loaded scripts. This is the entry point
/// the loader wraps via `read_scripts_report()`.
///
/// Fatal issues (currently: duplicate bindings) move a script into
/// `failed_scripts` and exclude it from the returned `scripts` slice so the
/// index never sees ambiguous dispatch. Warning issues stay in the kept set
/// but surface in the report for the MCP resource + menu-bar badge.
pub fn validate_script_catalog(scripts: Vec<Arc<Script>>) -> ScriptCatalogReport {
    let mut by_path: HashMap<PathBuf, Vec<ScriptValidationIssue>> = HashMap::new();
    for issue in detect_binding_collisions(&scripts) {
        by_path.entry(issue.path.clone()).or_default().push(issue);
    }

    let total_candidates = scripts.len();
    let mut kept: Vec<Arc<Script>> = Vec::with_capacity(scripts.len());
    let mut failed: Vec<FailedScript> = Vec::new();
    let mut warnings: Vec<ScriptValidationIssue> = Vec::new();

    for script in scripts {
        let issues = by_path.remove(&script.path).unwrap_or_default();
        let (fatal_issues, warn_issues): (Vec<_>, Vec<_>) = issues
            .into_iter()
            .partition(|i| i.severity == ValidationSeverity::Fatal);
        warnings.extend(warn_issues);

        if fatal_issues.is_empty() {
            kept.push(script);
        } else {
            failed.push(FailedScript {
                path: script.path.clone(),
                name: script.name.clone(),
                fatal: Arc::from(fatal_issues),
            });
        }
    }

    failed.sort_by(|a, b| a.path.cmp(&b.path));

    let fatal_count: usize = failed.iter().map(|f| f.fatal.len()).sum();
    let warning_count = warnings.len();
    let valid_count = kept.len();

    let validation = Arc::new(ValidationReport {
        schema_version: VALIDATION_SCHEMA_VERSION,
        total_candidates,
        valid_count,
        fatal_count,
        warning_count,
        failed_scripts: Arc::from(failed),
        warnings: Arc::from(warnings),
    });

    ScriptCatalogReport {
        scripts: Arc::from(kept),
        validation,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metadata_parser::TypedMetadata;

    fn make_script(name: &str, path: &str) -> Arc<Script> {
        Arc::new(Script {
            name: name.to_string(),
            path: PathBuf::from(path),
            extension: "ts".to_string(),
            ..Script::default()
        })
    }

    fn with_shortcut(mut script: Script, shortcut: &str) -> Script {
        script.shortcut = Some(shortcut.to_string());
        script
    }

    fn with_alias(mut script: Script, alias: &str) -> Script {
        script.alias = Some(alias.to_string());
        script
    }

    fn with_keyword(mut script: Script, keyword: &str) -> Script {
        script.typed_metadata = Some(TypedMetadata {
            keyword: Some(keyword.to_string()),
            ..TypedMetadata::default()
        });
        script
    }

    fn arc(script: Script) -> Arc<Script> {
        Arc::new(script)
    }

    #[test]
    fn empty_catalog_reports_zero_issues() {
        let report = validate_script_catalog(Vec::new());
        assert_eq!(report.validation.total_candidates, 0);
        assert_eq!(report.validation.valid_count, 0);
        assert_eq!(report.validation.fatal_count, 0);
        assert_eq!(report.validation.warning_count, 0);
        assert!(report.validation.failed_scripts.is_empty());
        assert!(report.scripts.is_empty());
    }

    #[test]
    fn single_script_with_bindings_passes() {
        let s = arc(with_shortcut(
            (*make_script("solo", "/tmp/solo.ts")).clone(),
            "cmd shift k",
        ));
        let report = validate_script_catalog(vec![s]);
        assert_eq!(report.validation.valid_count, 1);
        assert_eq!(report.validation.fatal_count, 0);
        assert!(report.validation.failed_scripts.is_empty());
    }

    #[test]
    fn duplicate_shortcut_excludes_both_scripts() {
        let a = arc(with_shortcut(
            (*make_script("a", "/tmp/a.ts")).clone(),
            "cmd shift k",
        ));
        let b = arc(with_shortcut(
            (*make_script("b", "/tmp/b.ts")).clone(),
            "Cmd Shift K",
        ));
        let report = validate_script_catalog(vec![a, b]);
        assert_eq!(report.validation.total_candidates, 2);
        assert_eq!(report.validation.valid_count, 0);
        assert_eq!(report.validation.fatal_count, 2);
        assert_eq!(report.validation.failed_scripts.len(), 2);

        let first = &report.validation.failed_scripts[0];
        assert_eq!(first.fatal.len(), 1);
        assert_eq!(first.fatal[0].related.len(), 1);
        assert!(matches!(
            first.fatal[0].kind,
            ScriptValidationKind::DuplicateBinding {
                binding: BindingKind::Shortcut,
                ..
            }
        ));
    }

    #[test]
    fn duplicate_alias_normalizes_case() {
        let a = arc(with_alias((*make_script("a", "/tmp/a.ts")).clone(), "GC"));
        let b = arc(with_alias((*make_script("b", "/tmp/b.ts")).clone(), "gc"));
        let report = validate_script_catalog(vec![a, b]);
        assert_eq!(report.validation.fatal_count, 2);
        assert!(report
            .validation
            .failed_scripts
            .iter()
            .all(|f| f.fatal.iter().any(|i| matches!(
                i.kind,
                ScriptValidationKind::DuplicateBinding {
                    binding: BindingKind::Alias,
                    ..
                }
            ))));
    }

    #[test]
    fn duplicate_keyword_from_typed_metadata_collides() {
        let a = arc(with_keyword(
            (*make_script("a", "/tmp/a.ts")).clone(),
            "!note",
        ));
        let b = arc(with_keyword(
            (*make_script("b", "/tmp/b.ts")).clone(),
            "!note",
        ));
        let report = validate_script_catalog(vec![a, b]);
        assert_eq!(report.validation.fatal_count, 2);
    }

    #[test]
    fn unique_bindings_across_kinds_do_not_collide() {
        let a = arc(with_shortcut(
            (*make_script("a", "/tmp/a.ts")).clone(),
            "cmd shift k",
        ));
        // Alias "cmd shift k" should NOT collide with shortcut "cmd shift k"
        // because the (kind, value) bucket is kind-scoped.
        let b = arc(with_alias(
            (*make_script("b", "/tmp/b.ts")).clone(),
            "cmd shift k",
        ));
        let report = validate_script_catalog(vec![a, b]);
        assert_eq!(report.validation.valid_count, 2);
        assert_eq!(report.validation.fatal_count, 0);
    }

    #[test]
    fn empty_binding_values_are_skipped() {
        let a = arc(with_shortcut(
            (*make_script("a", "/tmp/a.ts")).clone(),
            "   ",
        ));
        let b = arc(with_shortcut((*make_script("b", "/tmp/b.ts")).clone(), ""));
        // Both shortcuts normalize to None — no collision, both kept.
        let report = validate_script_catalog(vec![a, b]);
        assert_eq!(report.validation.valid_count, 2);
        assert_eq!(report.validation.fatal_count, 0);
    }

    #[test]
    fn trigger_collision_from_extra_field() {
        let mut a = (*make_script("a", "/tmp/a.ts")).clone();
        let mut extra_a = std::collections::HashMap::new();
        extra_a.insert(
            "trigger".to_string(),
            serde_json::Value::String("open".into()),
        );
        a.typed_metadata = Some(TypedMetadata {
            extra: extra_a,
            ..TypedMetadata::default()
        });

        let mut b = (*make_script("b", "/tmp/b.ts")).clone();
        let mut extra_b = std::collections::HashMap::new();
        extra_b.insert(
            "trigger".to_string(),
            serde_json::Value::String("OPEN".into()),
        );
        b.typed_metadata = Some(TypedMetadata {
            extra: extra_b,
            ..TypedMetadata::default()
        });

        let report = validate_script_catalog(vec![arc(a), arc(b)]);
        assert_eq!(report.validation.fatal_count, 2);
        assert!(report
            .validation
            .failed_scripts
            .iter()
            .all(|f| f.fatal.iter().any(|i| matches!(
                i.kind,
                ScriptValidationKind::DuplicateBinding {
                    binding: BindingKind::Trigger,
                    ..
                }
            ))));
    }

    #[test]
    fn three_way_shortcut_collision_lists_all_peers() {
        let a = arc(with_shortcut(
            (*make_script("a", "/tmp/a.ts")).clone(),
            "cmd k",
        ));
        let b = arc(with_shortcut(
            (*make_script("b", "/tmp/b.ts")).clone(),
            "cmd k",
        ));
        let c = arc(with_shortcut(
            (*make_script("c", "/tmp/c.ts")).clone(),
            "cmd k",
        ));
        let report = validate_script_catalog(vec![a, b, c]);
        assert_eq!(report.validation.fatal_count, 3);
        for failed in report.validation.failed_scripts.iter() {
            assert_eq!(
                failed.fatal[0].related.len(),
                2,
                "each failure should list the 2 peers it collides with"
            );
        }
    }

    #[test]
    fn report_is_serializable() {
        let a = arc(with_shortcut(
            (*make_script("a", "/tmp/a.ts")).clone(),
            "cmd k",
        ));
        let b = arc(with_shortcut(
            (*make_script("b", "/tmp/b.ts")).clone(),
            "cmd k",
        ));
        let report = validate_script_catalog(vec![a, b]);
        let json = serde_json::to_string(&*report.validation)
            .expect("validation report must serialize cleanly");
        assert!(json.contains("\"schemaVersion\":1"));
        assert!(json.contains("\"duplicateBinding\""));
        assert!(json.contains("\"shortcut\""));
    }
}
