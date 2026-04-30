use std::collections::HashMap;

use serde_json::Value;

use super::capture_schema::{CaptureFieldSchema, FieldRequirement};
use super::payload::{DateRole, MenuSyntaxHandlerSpec};

const EXTRA_KEY: &str = "menuSyntax";

/// Parse a single field-requirement token (e.g. `"body"`, `"url"`,
/// `"kv:amount"`, `"date:start"`, `"tag"`) into a `FieldRequirement`. Token
/// vocabulary intentionally mirrors the lowercase enum name; `kv:KEY` and
/// `date:ROLE` use a colon-prefix to keep the namespace open for future
/// `attachment:`/`location:` etc. Unknown tokens return `None` (the
/// caller logs/drops; the doctor will warn separately).
pub fn parse_field_requirement_token(token: &str) -> Option<FieldRequirement> {
    let trimmed = token.trim();
    if trimmed.is_empty() {
        return None;
    }
    let lower = trimmed.to_ascii_lowercase();
    if let Some(key) = lower.strip_prefix("kv:") {
        let key = key.trim();
        if key.is_empty() {
            return None;
        }
        return Some(FieldRequirement::Kv(key.to_string()));
    }
    if let Some(role) = lower.strip_prefix("date:") {
        let role = match role.trim() {
            "due" => DateRole::Due,
            "at" => DateRole::At,
            "start" => DateRole::Start,
            "end" => DateRole::End,
            "inferred" | "any" => DateRole::Inferred,
            _ => return None,
        };
        return Some(FieldRequirement::DateRole(role));
    }
    Some(match lower.as_str() {
        "body" => FieldRequirement::Body,
        "url" => FieldRequirement::Url,
        "priority" => FieldRequirement::Priority,
        "duration" => FieldRequirement::Duration,
        "tag" | "tags" => FieldRequirement::Tag,
        "date" | "anydate" | "any-date" => FieldRequirement::AnyDate,
        _ => return None,
    })
}

/// Build a [[crate::menu_syntax::capture_schema::CaptureFieldSchema]] from a
/// `capture.v1` `MenuSyntaxHandlerSpec`'s `required`/`optional`/`forbidden`
/// string vectors. Returns `None` if the spec is not `capture.v1` OR has no
/// `targets` (a schema needs a target name). Unknown tokens are silently
/// dropped — the doctor surface flags them as Warnings.
pub fn dynamic_capture_schema_from_spec(
    spec: &MenuSyntaxHandlerSpec,
) -> Option<CaptureFieldSchema> {
    if spec.family != "capture.v1" {
        return None;
    }
    let target = spec.targets.iter().find(|t| !t.trim().is_empty())?;
    let parse_list = |list: &[String]| -> Vec<FieldRequirement> {
        let mut out: Vec<FieldRequirement> = Vec::with_capacity(list.len());
        for token in list {
            if let Some(req) = parse_field_requirement_token(token) {
                if !out.contains(&req) {
                    out.push(req);
                }
            }
        }
        out
    };
    let required = parse_list(&spec.required);
    let mut optional = parse_list(&spec.optional);
    optional.retain(|r| !required.contains(r));
    let mut forbidden = parse_list(&spec.forbidden);
    forbidden.retain(|r| !required.contains(r) && !optional.contains(r));
    Some(CaptureFieldSchema {
        target: target.trim().to_ascii_lowercase(),
        required,
        optional,
        forbidden,
    })
}

pub fn handler_specs_from_extra_map(extra: &HashMap<String, Value>) -> Vec<MenuSyntaxHandlerSpec> {
    match extra.get(EXTRA_KEY) {
        Some(value) => handler_specs_from_value(value),
        None => Vec::new(),
    }
}

pub fn handler_specs_from_value(value: &Value) -> Vec<MenuSyntaxHandlerSpec> {
    match value {
        Value::Array(items) => items
            .iter()
            .filter_map(|item| deserialize_spec(item))
            .collect(),
        Value::Object(_) => deserialize_spec(value).map(|s| vec![s]).unwrap_or_default(),
        _ => Vec::new(),
    }
}

fn deserialize_spec(value: &Value) -> Option<MenuSyntaxHandlerSpec> {
    serde_json::from_value::<MenuSyntaxHandlerSpec>(value.clone()).ok()
}

pub fn handler_specs_from_yaml_like_string(raw: &str) -> Vec<MenuSyntaxHandlerSpec> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }

    if let Ok(value) = serde_json::from_str::<Value>(trimmed) {
        return handler_specs_from_value(&value);
    }

    if let Ok(specs) = serde_yaml::from_str::<Vec<MenuSyntaxHandlerSpec>>(trimmed) {
        return specs;
    }

    if let Ok(spec) = serde_yaml::from_str::<MenuSyntaxHandlerSpec>(trimmed) {
        return vec![spec];
    }

    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn handles_single_spec_object() {
        let value = json!({
            "family": "capture.v1",
            "targets": ["todo"],
            "accepts": ["tags", "date", "priority"],
            "defaultHandler": true
        });
        let specs = handler_specs_from_value(&value);
        assert_eq!(specs.len(), 1);
        assert_eq!(specs[0].family, "capture.v1");
        assert_eq!(specs[0].targets, vec!["todo".to_string()]);
        assert!(specs[0].default_handler);
        assert!(specs[0].handles_capture_target("todo"));
        assert!(!specs[0].handles_capture_target("cal"));
    }

    #[test]
    fn handles_array_of_specs() {
        let value = json!([
            { "family": "capture.v1", "targets": ["todo"] },
            { "family": "capture.v1", "targets": ["note"], "label": "Append note" }
        ]);
        let specs = handler_specs_from_value(&value);
        assert_eq!(specs.len(), 2);
        assert_eq!(specs[0].targets, vec!["todo".to_string()]);
        assert_eq!(specs[1].label.as_deref(), Some("Append note"));
    }

    #[test]
    fn reads_from_extra_map() {
        let mut extra: HashMap<String, Value> = HashMap::new();
        extra.insert(
            "menuSyntax".to_string(),
            json!([{ "family": "capture.v1", "targets": ["cal"] }]),
        );
        let specs = handler_specs_from_extra_map(&extra);
        assert_eq!(specs.len(), 1);
        assert_eq!(specs[0].targets, vec!["cal".to_string()]);
    }

    #[test]
    fn returns_empty_when_missing_or_malformed() {
        assert!(handler_specs_from_extra_map(&HashMap::new()).is_empty());

        let mut extra: HashMap<String, Value> = HashMap::new();
        extra.insert("menuSyntax".to_string(), json!("not an object"));
        assert!(handler_specs_from_extra_map(&extra).is_empty());

        let bad = json!({ "targets": ["todo"] });
        assert!(handler_specs_from_value(&bad).is_empty());
    }

    #[test]
    fn wildcard_target_handles_anything_in_family() {
        let value = json!({ "family": "capture.v1", "targets": ["*"] });
        let specs = handler_specs_from_value(&value);
        assert_eq!(specs.len(), 1);
        assert!(specs[0].handles_capture_target("todo"));
        assert!(specs[0].handles_capture_target("link"));
    }

    #[test]
    fn round_trips_kv_enums_camelcase_field() {
        // Pass 22 (`grammar-capture-key-enum-data-source` follow-on):
        // the SDK shape uses camelCase `kvEnums`; our internal field is
        // snake_case `kv_enums`. The serde derive's camelCase rename
        // must bridge the two so authors writing the documented YAML/
        // JSON template see their declared enums survive a round-trip.
        let value = json!({
            "family": "capture.v1",
            "targets": ["link"],
            "kvEnums": {
                "env": ["prod", "staging", "dev"],
                "priority": ["P0", "P1", "P2"]
            }
        });
        let specs = handler_specs_from_value(&value);
        assert_eq!(specs.len(), 1);
        let env = specs[0].kv_enums.get("env").expect("env enum");
        assert_eq!(
            env,
            &vec!["prod".to_string(), "staging".to_string(), "dev".to_string()]
        );
        let priority = specs[0].kv_enums.get("priority").expect("priority enum");
        assert_eq!(
            priority,
            &vec!["P0".to_string(), "P1".to_string(), "P2".to_string()]
        );
        // And re-serializing yields the same camelCase key.
        let reserialized = serde_json::to_value(&specs[0]).unwrap();
        assert!(reserialized
            .get("kvEnums")
            .and_then(|v| v.get("env"))
            .is_some());
    }

    #[test]
    fn empty_kv_enums_is_skipped_during_serialization() {
        // Pass 22: the `skip_serializing_if = "BTreeMap::is_empty"`
        // attribute keeps existing serialized specs unchanged. Round-
        // tripping a spec without the field must NOT introduce a
        // `kvEnums:{}` blob in the output.
        let value = json!({ "family": "capture.v1", "targets": ["todo"] });
        let specs = handler_specs_from_value(&value);
        assert!(specs[0].kv_enums.is_empty());
        let reserialized = serde_json::to_value(&specs[0]).unwrap();
        assert!(
            reserialized.get("kvEnums").is_none(),
            "empty kvEnums must skip-serialize so existing specs stay byte-identical"
        );
    }

    #[test]
    fn parses_json_string_form() {
        let json_str = r#"[{"family":"capture.v1","targets":["note"]}]"#;
        let specs = handler_specs_from_yaml_like_string(json_str);
        assert_eq!(specs.len(), 1);
        assert_eq!(specs[0].targets, vec!["note".to_string()]);
    }

    #[test]
    fn parses_yaml_list_form() {
        let yaml = "- family: capture.v1\n  targets: [social]\n  label: Draft post";
        let specs = handler_specs_from_yaml_like_string(yaml);
        assert_eq!(specs.len(), 1);
        assert_eq!(specs[0].family, "capture.v1");
        assert_eq!(specs[0].targets, vec!["social".to_string()]);
        assert_eq!(specs[0].label.as_deref(), Some("Draft post"));
    }

    #[test]
    fn parses_yaml_single_object_form() {
        let yaml = "family: capture.v1\ntargets:\n  - link\n";
        let specs = handler_specs_from_yaml_like_string(yaml);
        assert_eq!(specs.len(), 1);
        assert_eq!(specs[0].targets, vec!["link".to_string()]);
    }

    #[test]
    fn empty_or_garbage_yaml_yields_empty() {
        assert!(handler_specs_from_yaml_like_string("").is_empty());
        assert!(handler_specs_from_yaml_like_string("   ").is_empty());
        assert!(handler_specs_from_yaml_like_string(";;;not valid;;").is_empty());
    }

    #[test]
    fn family_outside_capture_v1_does_not_match_capture_target() {
        let value = json!({ "family": "argv.v1", "targets": ["todo"] });
        let spec = handler_specs_from_value(&value).remove(0);
        assert!(!spec.handles_capture_target("todo"));
    }

    // ========================================================================
    // dynamic_capture_schema_from_spec + parse_field_requirement_token
    // ========================================================================

    #[test]
    fn parse_field_requirement_token_known_singletons() {
        assert_eq!(
            parse_field_requirement_token("body"),
            Some(FieldRequirement::Body)
        );
        assert_eq!(
            parse_field_requirement_token("URL"),
            Some(FieldRequirement::Url)
        );
        assert_eq!(
            parse_field_requirement_token(" Tag "),
            Some(FieldRequirement::Tag)
        );
        assert_eq!(
            parse_field_requirement_token("date"),
            Some(FieldRequirement::AnyDate)
        );
        assert_eq!(
            parse_field_requirement_token("any-date"),
            Some(FieldRequirement::AnyDate)
        );
    }

    #[test]
    fn parse_field_requirement_token_kv_namespace() {
        assert_eq!(
            parse_field_requirement_token("kv:amount"),
            Some(FieldRequirement::Kv("amount".to_string()))
        );
        assert_eq!(
            parse_field_requirement_token("KV:Title"),
            Some(FieldRequirement::Kv("title".to_string()))
        );
        assert_eq!(parse_field_requirement_token("kv:"), None);
        assert_eq!(parse_field_requirement_token("kv:   "), None);
    }

    #[test]
    fn parse_field_requirement_token_date_role_namespace() {
        assert_eq!(
            parse_field_requirement_token("date:start"),
            Some(FieldRequirement::DateRole(DateRole::Start))
        );
        assert_eq!(
            parse_field_requirement_token("date:end"),
            Some(FieldRequirement::DateRole(DateRole::End))
        );
        assert_eq!(
            parse_field_requirement_token("date:due"),
            Some(FieldRequirement::DateRole(DateRole::Due))
        );
        assert_eq!(
            parse_field_requirement_token("date:any"),
            Some(FieldRequirement::DateRole(DateRole::Inferred))
        );
        assert_eq!(parse_field_requirement_token("date:bogus"), None);
    }

    #[test]
    fn parse_field_requirement_token_unknown_yields_none() {
        assert_eq!(parse_field_requirement_token(""), None);
        assert_eq!(parse_field_requirement_token("    "), None);
        assert_eq!(parse_field_requirement_token("location"), None);
        assert_eq!(parse_field_requirement_token("attachment"), None);
    }

    #[test]
    fn dynamic_schema_for_expense_target_with_required_body_and_amount() {
        let value = json!({
            "family": "capture.v1",
            "targets": ["expense"],
            "required": ["body", "kv:amount"],
            "optional": ["tag"],
            "forbidden": ["url"],
        });
        let spec = handler_specs_from_value(&value).remove(0);
        let schema = dynamic_capture_schema_from_spec(&spec).expect("expected schema");
        assert_eq!(schema.target, "expense");
        assert_eq!(
            schema.required,
            vec![
                FieldRequirement::Body,
                FieldRequirement::Kv("amount".to_string())
            ]
        );
        assert_eq!(schema.optional, vec![FieldRequirement::Tag]);
        assert_eq!(schema.forbidden, vec![FieldRequirement::Url]);
    }

    #[test]
    fn dynamic_schema_drops_unknown_tokens_silently() {
        let value = json!({
            "family": "capture.v1",
            "targets": ["expense"],
            "required": ["body", "location", "kv:amount"],
        });
        let spec = handler_specs_from_value(&value).remove(0);
        let schema = dynamic_capture_schema_from_spec(&spec).expect("expected schema");
        // "location" is unknown — dropped — leaving body + amount.
        assert_eq!(
            schema.required,
            vec![
                FieldRequirement::Body,
                FieldRequirement::Kv("amount".to_string())
            ]
        );
    }

    #[test]
    fn dynamic_schema_target_is_lowercased() {
        let value = json!({
            "family": "capture.v1",
            "targets": ["EXPENSE"],
            "required": ["body"],
        });
        let spec = handler_specs_from_value(&value).remove(0);
        let schema = dynamic_capture_schema_from_spec(&spec).expect("expected schema");
        assert_eq!(schema.target, "expense");
    }

    #[test]
    fn dynamic_schema_returns_none_for_non_capture_family() {
        let value = json!({
            "family": "skill.v1",
            "slug": "review",
            "required": ["body"],
        });
        let spec = handler_specs_from_value(&value).remove(0);
        assert!(dynamic_capture_schema_from_spec(&spec).is_none());
    }

    #[test]
    fn dynamic_schema_returns_none_when_targets_missing_or_empty() {
        let value = json!({
            "family": "capture.v1",
            "targets": ["", "   "],
            "required": ["body"],
        });
        let spec = handler_specs_from_value(&value).remove(0);
        assert!(dynamic_capture_schema_from_spec(&spec).is_none());
    }

    #[test]
    fn dynamic_schema_with_no_required_yields_empty_required_vec() {
        let value = json!({
            "family": "capture.v1",
            "targets": ["expense"],
        });
        let spec = handler_specs_from_value(&value).remove(0);
        let schema = dynamic_capture_schema_from_spec(&spec).expect("expected schema");
        assert!(schema.required.is_empty());
        assert!(schema.optional.is_empty());
        assert!(schema.forbidden.is_empty());
    }
}
