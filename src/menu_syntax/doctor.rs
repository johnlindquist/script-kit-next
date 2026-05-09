//! Power Syntax `metadata.menuSyntax` doctor.
//!
//! Pure validation library that takes a `serde_json::Value` (whatever a
//! script's `metadata.menuSyntax` deserialized to) and returns actionable
//! diagnostics: unknown families, malformed targets, duplicate command
//! heads, accepts tokens that don't match any payload field, etc. The
//! diagnostics carry JSON paths so a future `kit menu-syntax doctor`
//! CLI subcommand can print machine-actionable output.
//!
//! The CLI wiring lives in a follow-up commit; this module is the engine
//! the CLI calls. Receipt: `cargo test --test menu_syntax_doctor`.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

use super::metadata::parse_field_requirement_token;
use super::payload::{is_known_capture_target, KNOWN_CAPTURE_TARGETS};

/// Severity of a single diagnostic. Errors should fail the CLI with a
/// non-zero exit code; warnings are advisory and pass.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DoctorSeverity {
    Error,
    Warning,
}

/// One diagnostic emitted by the doctor. `path` is a JSONPath-like pointer
/// into the input (e.g. `$[2].targets[0]`) so the CLI can show authors
/// exactly which field is at fault.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DoctorIssue {
    pub path: String,
    pub severity: DoctorSeverity,
    pub message: String,
}

impl DoctorIssue {
    fn err(path: impl Into<String>, msg: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            severity: DoctorSeverity::Error,
            message: msg.into(),
        }
    }

    fn warn(path: impl Into<String>, msg: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            severity: DoctorSeverity::Warning,
            message: msg.into(),
        }
    }
}

/// Aggregate doctor result. `has_errors()` is the CLI exit-code source.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DoctorReport {
    pub issues: Vec<DoctorIssue>,
}

impl DoctorReport {
    pub fn has_errors(&self) -> bool {
        self.issues
            .iter()
            .any(|i| i.severity == DoctorSeverity::Error)
    }

    pub fn errors(&self) -> impl Iterator<Item = &DoctorIssue> {
        self.issues
            .iter()
            .filter(|i| i.severity == DoctorSeverity::Error)
    }
}

const KNOWN_FAMILIES: &[&str] = &["capture.v1", "command.v1", "skill.v1"];

const KNOWN_CAPTURE_ACCEPTS: &[&str] = &[
    "tags",
    "date",
    "dateRange",
    "duration",
    "recurrence",
    "relativeDate",
    "daily",
    "multiWeekday",
    "monthly",
    "yearly",
    "url",
    "priority",
    "kv",
];

/// Validate a `metadata.menuSyntax` value (Object or Array form).
/// Returns a `DoctorReport` with one entry per problem found.
pub fn validate(value: &Value) -> DoctorReport {
    validate_at_path(value, "$")
}

/// Validate a `metadata.menuSyntax` value rooted at `base_path`.
///
/// The CLI uses this when authors pass a wrapper object such as
/// `{ "menuSyntax": [...] }`, preserving actionable paths like
/// `$.menuSyntax[0].targets` instead of reporting everything at `$`.
pub fn validate_at_path(value: &Value, base_path: &str) -> DoctorReport {
    let mut issues = Vec::new();
    match value {
        Value::Array(items) => {
            let mut command_heads: BTreeMap<String, Vec<usize>> = BTreeMap::new();
            for (idx, item) in items.iter().enumerate() {
                let path = format!("{base_path}[{idx}]");
                validate_one(item, &path, &mut issues, Some((&mut command_heads, idx)));
            }
            for (head, indices) in command_heads {
                if indices.len() > 1 {
                    let label = indices
                        .iter()
                        .map(|i| format!("{base_path}[{i}]"))
                        .collect::<Vec<_>>()
                        .join(", ");
                    issues.push(DoctorIssue::err(
                        label,
                        format!(
                            "duplicate command.v1 head `{head}` (registered {} times)",
                            indices.len()
                        ),
                    ));
                }
            }
        }
        Value::Object(_) => {
            validate_one(value, base_path, &mut issues, None);
        }
        Value::Null => {
            issues.push(DoctorIssue::warn(
                base_path,
                "menuSyntax is null — drop the field or replace with an array",
            ));
        }
        _ => {
            issues.push(DoctorIssue::err(
                base_path,
                format!(
                    "menuSyntax must be an array (or single-spec object); got {}",
                    json_type_label(value)
                ),
            ));
        }
    }
    DoctorReport { issues }
}

fn json_type_label(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

fn validate_one(
    item: &Value,
    path: &str,
    issues: &mut Vec<DoctorIssue>,
    mut command_track: Option<(&mut BTreeMap<String, Vec<usize>>, usize)>,
) {
    let Value::Object(obj) = item else {
        issues.push(DoctorIssue::err(
            path,
            format!("expected object, got {}", json_type_label(item)),
        ));
        return;
    };

    let family = match obj.get("family") {
        Some(Value::String(s)) => s.clone(),
        Some(other) => {
            issues.push(DoctorIssue::err(
                format!("{path}.family"),
                format!("family must be a string, got {}", json_type_label(other)),
            ));
            return;
        }
        None => {
            issues.push(DoctorIssue::err(
                format!("{path}.family"),
                "missing required field `family` (expected one of capture.v1, command.v1, skill.v1)",
            ));
            return;
        }
    };

    if !KNOWN_FAMILIES.iter().any(|f| *f == family) {
        issues.push(DoctorIssue::err(
            format!("{path}.family"),
            format!(
                "unknown family `{family}` (expected one of {})",
                KNOWN_FAMILIES.join(", ")
            ),
        ));
        return;
    }

    match family.as_str() {
        "capture.v1" => validate_capture(obj, path, issues),
        "command.v1" => {
            validate_command(obj, path, issues);
            if let Some((tracker, idx)) = command_track.as_mut() {
                if let Some(Value::String(head)) = obj.get("head") {
                    let lowered = head.to_ascii_lowercase();
                    tracker.entry(lowered).or_default().push(*idx);
                }
            }
        }
        "skill.v1" => validate_skill(obj, path, issues),
        _ => unreachable!("KNOWN_FAMILIES gate"),
    }
}

fn validate_capture(
    obj: &serde_json::Map<String, Value>,
    path: &str,
    issues: &mut Vec<DoctorIssue>,
) {
    // targets: required, non-empty array of strings.
    let Some(targets) = obj.get("targets") else {
        issues.push(DoctorIssue::err(
            format!("{path}.targets"),
            "capture.v1 requires `targets` (array of slug strings, or [\"*\"] for wildcard)",
        ));
        return;
    };
    let Value::Array(targets_arr) = targets else {
        issues.push(DoctorIssue::err(
            format!("{path}.targets"),
            format!("targets must be an array, got {}", json_type_label(targets)),
        ));
        return;
    };
    if targets_arr.is_empty() {
        issues.push(DoctorIssue::err(
            format!("{path}.targets"),
            "targets is empty — capture handler will never match anything",
        ));
    }

    let mut seen_targets: BTreeSet<String> = BTreeSet::new();
    for (i, t) in targets_arr.iter().enumerate() {
        let target_path = format!("{path}.targets[{i}]");
        let Value::String(slug) = t else {
            issues.push(DoctorIssue::err(
                target_path,
                format!("target must be a string, got {}", json_type_label(t)),
            ));
            continue;
        };
        if slug.is_empty() {
            issues.push(DoctorIssue::err(target_path, "target slug is empty"));
            continue;
        }
        if slug != "*" && slug.chars().any(|c| c.is_whitespace()) {
            issues.push(DoctorIssue::err(
                target_path.clone(),
                format!("target slug `{slug}` contains whitespace"),
            ));
        }
        let lowered = slug.to_ascii_lowercase();
        if !seen_targets.insert(lowered.clone()) {
            issues.push(DoctorIssue::warn(
                target_path.clone(),
                format!("duplicate target slug `{slug}` (case-insensitive)"),
            ));
        }
        if slug != "*" && !is_known_capture_target(slug) {
            issues.push(DoctorIssue::warn(
                target_path,
                format!(
                    "target `{slug}` is not a built-in (built-ins: {}); make sure your handler is the intended owner",
                    KNOWN_CAPTURE_TARGETS.join(", ")
                ),
            ));
        }
    }

    for field in ["required", "optional", "forbidden"] {
        validate_capture_requirement_list(obj, path, field, issues);
    }

    if let Some(accepts) = obj.get("accepts") {
        let Value::Array(arr) = accepts else {
            issues.push(DoctorIssue::err(
                format!("{path}.accepts"),
                format!("accepts must be an array, got {}", json_type_label(accepts)),
            ));
            return;
        };
        for (i, a) in arr.iter().enumerate() {
            let accept_path = format!("{path}.accepts[{i}]");
            let Value::String(token) = a else {
                issues.push(DoctorIssue::err(
                    accept_path,
                    format!("accepts entry must be a string, got {}", json_type_label(a)),
                ));
                continue;
            };
            if !KNOWN_CAPTURE_ACCEPTS.contains(&token.as_str()) {
                issues.push(DoctorIssue::err(
                    accept_path,
                    format!(
                        "unknown accepts token `{token}` (expected one of {})",
                        KNOWN_CAPTURE_ACCEPTS.join(", ")
                    ),
                ));
            }
        }
    }
}

fn validate_capture_requirement_list(
    obj: &serde_json::Map<String, Value>,
    path: &str,
    field: &str,
    issues: &mut Vec<DoctorIssue>,
) {
    let Some(value) = obj.get(field) else {
        return;
    };
    let Value::Array(arr) = value else {
        issues.push(DoctorIssue::err(
            format!("{path}.{field}"),
            format!("{field} must be an array, got {}", json_type_label(value)),
        ));
        return;
    };

    let mut seen: BTreeSet<String> = BTreeSet::new();
    for (i, item) in arr.iter().enumerate() {
        let item_path = format!("{path}.{field}[{i}]");
        let Value::String(token) = item else {
            issues.push(DoctorIssue::err(
                item_path,
                format!(
                    "{field} entry must be a string, got {}",
                    json_type_label(item)
                ),
            ));
            continue;
        };
        let normalized = token.trim().to_ascii_lowercase();
        if normalized.is_empty() {
            issues.push(DoctorIssue::warn(
                item_path,
                format!("empty {field} token will be ignored"),
            ));
            continue;
        }
        if parse_field_requirement_token(token).is_none() {
            issues.push(DoctorIssue::warn(
                item_path,
                format!("unknown {field} token `{token}` will be ignored by the capture schema"),
            ));
            continue;
        }
        if !seen.insert(normalized) {
            issues.push(DoctorIssue::warn(
                item_path,
                format!("duplicate {field} token `{token}`"),
            ));
        }
    }
}

fn validate_command(
    obj: &serde_json::Map<String, Value>,
    path: &str,
    issues: &mut Vec<DoctorIssue>,
) {
    let Some(head_value) = obj.get("head") else {
        issues.push(DoctorIssue::err(
            format!("{path}.head"),
            "command.v1 requires `head` (the bare slug after `!`, e.g. \"deploy\")",
        ));
        return;
    };
    let Value::String(head) = head_value else {
        issues.push(DoctorIssue::err(
            format!("{path}.head"),
            format!("head must be a string, got {}", json_type_label(head_value)),
        ));
        return;
    };
    if head.is_empty() {
        issues.push(DoctorIssue::err(format!("{path}.head"), "head is empty"));
    }
    if head.starts_with('>') {
        issues.push(DoctorIssue::err(
            format!("{path}.head"),
            format!(
                "head should NOT include the leading `>` — use `\"{}\"`",
                head.trim_start_matches('>')
            ),
        ));
    }
    if head.chars().any(|c| c.is_whitespace()) {
        issues.push(DoctorIssue::err(
            format!("{path}.head"),
            format!("head `{head}` contains whitespace; use a single slug"),
        ));
    }

    if let Some(args) = obj.get("args") {
        validate_named_array(args, &format!("{path}.args"), "name", issues);
    }
    if let Some(flags) = obj.get("flags") {
        validate_named_array(flags, &format!("{path}.flags"), "name", issues);
    }
}

fn validate_skill(obj: &serde_json::Map<String, Value>, path: &str, issues: &mut Vec<DoctorIssue>) {
    let Some(slug_value) = obj.get("slug") else {
        issues.push(DoctorIssue::err(
            format!("{path}.slug"),
            "skill.v1 requires `slug` (the bare slug after `/`, e.g. \"review\")",
        ));
        return;
    };
    let Value::String(slug) = slug_value else {
        issues.push(DoctorIssue::err(
            format!("{path}.slug"),
            format!("slug must be a string, got {}", json_type_label(slug_value)),
        ));
        return;
    };
    if slug.is_empty() {
        issues.push(DoctorIssue::err(format!("{path}.slug"), "slug is empty"));
    }
    if slug.starts_with('/') {
        issues.push(DoctorIssue::err(
            format!("{path}.slug"),
            format!(
                "slug should NOT include the leading `/` — use `\"{}\"`",
                slug.trim_start_matches('/')
            ),
        ));
    }
}

fn validate_named_array(
    value: &Value,
    path: &str,
    name_field: &str,
    issues: &mut Vec<DoctorIssue>,
) {
    let Value::Array(arr) = value else {
        issues.push(DoctorIssue::err(
            path,
            format!("{path} must be an array, got {}", json_type_label(value)),
        ));
        return;
    };
    let mut names: BTreeSet<String> = BTreeSet::new();
    for (i, item) in arr.iter().enumerate() {
        let item_path = format!("{path}[{i}]");
        let Value::Object(obj) = item else {
            issues.push(DoctorIssue::err(
                item_path,
                format!("entry must be an object, got {}", json_type_label(item)),
            ));
            continue;
        };
        match obj.get(name_field) {
            Some(Value::String(name)) => {
                if name.is_empty() {
                    issues.push(DoctorIssue::err(
                        format!("{item_path}.{name_field}"),
                        format!("{name_field} is empty"),
                    ));
                } else if !names.insert(name.clone()) {
                    issues.push(DoctorIssue::warn(
                        format!("{item_path}.{name_field}"),
                        format!("duplicate {name_field} `{name}` in {path}"),
                    ));
                }
            }
            Some(other) => issues.push(DoctorIssue::err(
                format!("{item_path}.{name_field}"),
                format!(
                    "{name_field} must be a string, got {}",
                    json_type_label(other)
                ),
            )),
            None => issues.push(DoctorIssue::err(
                format!("{item_path}.{name_field}"),
                format!("missing required `{name_field}` field"),
            )),
        }
    }
}
