use crate::menu_syntax::payload::{
    object_refs_for_raw_capture, CaptureInvocation, CaptureObjectKind, DateRole,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldRequirement {
    Body,
    Url,
    Priority,
    Duration,
    Tag,
    AnyDate,
    DateRole(DateRole),
    Kv(String),
    ObjectSelection,
    SnippetTriggerOrSelection,
    SnippetNameOrSelection,
}

impl FieldRequirement {
    pub fn enum_values(&self) -> &'static [&'static str] {
        match self {
            FieldRequirement::Priority => &["p1", "p2", "p3", "p4"],
            _ => &[],
        }
    }

    pub fn label(&self) -> String {
        match self {
            FieldRequirement::Body => "body".to_string(),
            FieldRequirement::Url => "url".to_string(),
            FieldRequirement::Priority => "priority".to_string(),
            FieldRequirement::Duration => "duration".to_string(),
            FieldRequirement::Tag => "tag".to_string(),
            FieldRequirement::AnyDate => "date".to_string(),
            FieldRequirement::DateRole(role) => match role {
                DateRole::Due => "due date".to_string(),
                DateRole::At => "time".to_string(),
                DateRole::Start => "start time".to_string(),
                DateRole::End => "end time".to_string(),
                DateRole::Inferred => "date".to_string(),
            },
            FieldRequirement::Kv(key) => key.clone(),
            FieldRequirement::ObjectSelection => "selected object".to_string(),
            FieldRequirement::SnippetTriggerOrSelection => {
                "trigger or selected snippet".to_string()
            }
            FieldRequirement::SnippetNameOrSelection => "name or selected snippet".to_string(),
        }
    }

    pub fn is_satisfied(&self, payload: &CaptureInvocation) -> bool {
        match self {
            FieldRequirement::Body => body_requirement_is_satisfied(payload),
            FieldRequirement::Url => url_requirement_is_satisfied(payload),
            FieldRequirement::Priority => payload.priority.is_some(),
            FieldRequirement::Duration => payload.duration.is_some(),
            FieldRequirement::Tag => !payload.tags.is_empty(),
            FieldRequirement::AnyDate => !payload.date_phrases.is_empty(),
            FieldRequirement::DateRole(role) => {
                payload.date_phrases.iter().any(|p| &p.role == role)
            }
            FieldRequirement::Kv(key) => payload
                .kv
                .iter()
                .any(|(k, v)| k.eq_ignore_ascii_case(key) && !v.trim().is_empty()),
            FieldRequirement::ObjectSelection => has_resolved_object_ref(payload, None),
            FieldRequirement::SnippetTriggerOrSelection => {
                snippet_trigger_or_selection_requirement_is_satisfied(payload)
            }
            FieldRequirement::SnippetNameOrSelection => {
                snippet_name_or_selection_requirement_is_satisfied(payload)
            }
        }
    }
}

fn body_requirement_is_satisfied(payload: &CaptureInvocation) -> bool {
    if !payload.body.trim().is_empty() {
        return true;
    }
    if matches!(
        payload.target.to_ascii_lowercase().as_str(),
        "reminder" | "snooze" | "defer"
    ) {
        return has_resolved_object_ref(payload, Some(CaptureObjectKind::Todo));
    }
    if matches!(
        payload.target.to_ascii_lowercase().as_str(),
        "note" | "notes"
    ) && has_resolved_object_ref(payload, Some(CaptureObjectKind::Note))
        && note_payload_has_update_fragment(payload)
    {
        return true;
    }
    if payload.target.eq_ignore_ascii_case("snippet") {
        if let Ok(draft) = crate::menu_syntax::parse_snippet_scriptlet_capture(payload) {
            if !matches!(
                draft.operation,
                crate::menu_syntax::SnippetScriptletOperation::Create
            ) {
                return draft.lookup.is_some();
            }
        }
    }
    false
}

fn url_requirement_is_satisfied(payload: &CaptureInvocation) -> bool {
    if payload.url.is_some() {
        return true;
    }
    if !payload.target.eq_ignore_ascii_case("link") {
        return false;
    }
    if !matches!(
        first_app_owned_operation_word(&payload.body),
        Some("update" | "delete")
    ) {
        return false;
    }
    has_resolved_object_ref(payload, Some(CaptureObjectKind::Link))
}

fn snippet_trigger_or_selection_requirement_is_satisfied(payload: &CaptureInvocation) -> bool {
    let has_trigger = payload
        .kv
        .iter()
        .any(|(k, v)| k.eq_ignore_ascii_case("trigger") && !v.trim().is_empty());
    if has_trigger {
        return true;
    }
    if !payload.target.eq_ignore_ascii_case("snippet") {
        return false;
    }
    matches!(
        first_app_owned_operation_word(&payload.body),
        Some("update" | "delete")
    ) && has_resolved_object_ref(payload, Some(CaptureObjectKind::Snippet))
}

fn snippet_name_or_selection_requirement_is_satisfied(payload: &CaptureInvocation) -> bool {
    if !payload.target.eq_ignore_ascii_case("snippet") {
        return false;
    }
    let Ok(draft) = crate::menu_syntax::parse_snippet_scriptlet_capture(payload) else {
        return false;
    };
    match draft.operation {
        crate::menu_syntax::SnippetScriptletOperation::Create => draft
            .name
            .as_deref()
            .map(|name| !name.trim().is_empty())
            .unwrap_or(false),
        crate::menu_syntax::SnippetScriptletOperation::Update
        | crate::menu_syntax::SnippetScriptletOperation::Delete => draft.lookup.is_some(),
    }
}

fn note_payload_has_update_fragment(payload: &CaptureInvocation) -> bool {
    payload.url.is_some()
        || !payload.tags.is_empty()
        || !payload.kv.is_empty()
        || !payload.date_phrases.is_empty()
        || matches!(
            first_app_owned_operation_word(&payload.body),
            Some("update" | "delete")
        )
}

fn selected_link_ref_url(payload: &CaptureInvocation) -> Option<String> {
    if !payload.target.eq_ignore_ascii_case("link") {
        return None;
    }
    if !matches!(
        first_app_owned_operation_word(&payload.body),
        Some("update" | "delete")
    ) {
        return None;
    }
    object_refs_for_raw_capture(&payload.target, &payload.raw)
        .into_iter()
        .find(|object_ref| {
            object_ref.resolved
                && object_ref.role == "primary"
                && object_ref.kind == CaptureObjectKind::Link
        })
        .map(|object_ref| object_ref.id)
}

fn first_app_owned_operation_word(body: &str) -> Option<&'static str> {
    let first = body.split_whitespace().next()?.to_ascii_lowercase();
    match first.as_str() {
        "add" | "create" | "save" => Some("create"),
        "update" => Some("update"),
        "remove" | "rm" | "delete" => Some("delete"),
        _ => None,
    }
}

fn has_resolved_object_ref(payload: &CaptureInvocation, kind: Option<CaptureObjectKind>) -> bool {
    object_refs_for_raw_capture(&payload.target, &payload.raw)
        .into_iter()
        .any(|object_ref| {
            object_ref.resolved
                && object_ref.role == "primary"
                && kind.map(|kind| object_ref.kind == kind).unwrap_or(true)
        })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureFieldSchema {
    pub target: String,
    pub required: Vec<FieldRequirement>,
    pub optional: Vec<FieldRequirement>,
    pub forbidden: Vec<FieldRequirement>,
}

impl CaptureFieldSchema {
    pub fn missing_required(&self, payload: &CaptureInvocation) -> Vec<FieldRequirement> {
        self.required
            .iter()
            .filter(|req| !req.is_satisfied(payload))
            .cloned()
            .collect()
    }
}

pub fn builtin_schema(target: &str) -> Option<CaptureFieldSchema> {
    let target_lc = target.to_ascii_lowercase();
    match target_lc.as_str() {
        "todo" => Some(CaptureFieldSchema {
            target: "todo".to_string(),
            required: vec![FieldRequirement::Body],
            optional: vec![
                FieldRequirement::Tag,
                FieldRequirement::Priority,
                FieldRequirement::AnyDate,
                FieldRequirement::Url,
                FieldRequirement::ObjectSelection,
            ],
            forbidden: vec![],
        }),
        "reminder" => Some(CaptureFieldSchema {
            target: "reminder".to_string(),
            required: vec![FieldRequirement::Body],
            optional: vec![
                FieldRequirement::Tag,
                FieldRequirement::AnyDate,
                FieldRequirement::Duration,
                FieldRequirement::ObjectSelection,
            ],
            forbidden: vec![FieldRequirement::Url],
        }),
        "snooze" => Some(CaptureFieldSchema {
            target: "snooze".to_string(),
            required: vec![FieldRequirement::Body, FieldRequirement::AnyDate],
            optional: vec![
                FieldRequirement::Tag,
                FieldRequirement::Duration,
                FieldRequirement::ObjectSelection,
            ],
            forbidden: vec![FieldRequirement::Priority, FieldRequirement::Url],
        }),
        "defer" => Some(CaptureFieldSchema {
            target: "defer".to_string(),
            required: vec![FieldRequirement::Body, FieldRequirement::AnyDate],
            optional: vec![
                FieldRequirement::Tag,
                FieldRequirement::Priority,
                FieldRequirement::ObjectSelection,
            ],
            forbidden: vec![FieldRequirement::Url, FieldRequirement::Duration],
        }),
        "note" => Some(CaptureFieldSchema {
            target: "note".to_string(),
            required: vec![FieldRequirement::Body],
            optional: vec![
                FieldRequirement::Tag,
                FieldRequirement::Url,
                FieldRequirement::AnyDate,
                FieldRequirement::Kv("title".to_string()),
                FieldRequirement::Kv("alias".to_string()),
                FieldRequirement::Kv("aliases".to_string()),
                FieldRequirement::ObjectSelection,
            ],
            forbidden: vec![FieldRequirement::Priority, FieldRequirement::Duration],
        }),
        "notes" => builtin_schema("note").map(|mut schema| {
            schema.target = "notes".to_string();
            schema
        }),
        "link" => Some(CaptureFieldSchema {
            target: "link".to_string(),
            required: vec![FieldRequirement::Url],
            optional: vec![
                FieldRequirement::Body,
                FieldRequirement::Tag,
                FieldRequirement::Kv("title".to_string()),
                FieldRequirement::ObjectSelection,
            ],
            forbidden: vec![FieldRequirement::Priority, FieldRequirement::Duration],
        }),
        "snippet" => Some(CaptureFieldSchema {
            target: "snippet".to_string(),
            required: vec![
                FieldRequirement::Body,
                FieldRequirement::SnippetNameOrSelection,
            ],
            optional: vec![
                FieldRequirement::Kv("keyword".to_string()),
                FieldRequirement::Kv("description".to_string()),
                FieldRequirement::Kv("alias".to_string()),
                FieldRequirement::Kv("shortcut".to_string()),
                FieldRequirement::Kv("author".to_string()),
                FieldRequirement::Kv("enter".to_string()),
                FieldRequirement::Kv("icon".to_string()),
                FieldRequirement::Kv("placeholder".to_string()),
                FieldRequirement::Kv("cron".to_string()),
                FieldRequirement::Kv("schedule".to_string()),
                FieldRequirement::Kv("hidden".to_string()),
                FieldRequirement::Kv("background".to_string()),
                FieldRequirement::Kv("system".to_string()),
                FieldRequirement::Kv("fallback".to_string()),
                FieldRequirement::Kv("fallback_label".to_string()),
                FieldRequirement::Kv("tags".to_string()),
                FieldRequirement::Kv("watch".to_string()),
                FieldRequirement::Kv("tool".to_string()),
                FieldRequirement::ObjectSelection,
            ],
            forbidden: vec![FieldRequirement::Priority, FieldRequirement::Duration],
        }),
        "cal" | "mcal" => Some(CaptureFieldSchema {
            target: target_lc,
            required: vec![FieldRequirement::Body, FieldRequirement::AnyDate],
            optional: vec![
                FieldRequirement::Tag,
                FieldRequirement::Duration,
                FieldRequirement::Kv("location".to_string()),
            ],
            forbidden: vec![FieldRequirement::Priority, FieldRequirement::Url],
        }),
        "social" => Some(CaptureFieldSchema {
            target: "social".to_string(),
            required: vec![FieldRequirement::Body],
            optional: vec![FieldRequirement::Tag, FieldRequirement::Url],
            forbidden: vec![FieldRequirement::Priority, FieldRequirement::Duration],
        }),
        _ => None,
    }
}

pub fn builtin_target_slugs() -> &'static [&'static str] {
    &["todo", "note", "link", "snippet", "cal", "social"]
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationResult {
    Ready,
    Incomplete {
        missing: Vec<FieldRequirement>,
    },
    Malformed {
        field: FieldRequirement,
        reason: String,
    },
}

pub fn validate(payload: &CaptureInvocation, schema: &CaptureFieldSchema) -> ValidationResult {
    if let Some(url) = payload.url.as_deref() {
        if !is_well_formed_url(url) {
            return ValidationResult::Malformed {
                field: FieldRequirement::Url,
                reason: format!("URL must start with http:// or https://, got `{url}`"),
            };
        }
    }
    if let Some(url) = selected_link_ref_url(payload) {
        if !is_well_formed_url(&url) {
            return ValidationResult::Malformed {
                field: FieldRequirement::Url,
                reason: format!("URL must start with http:// or https://, got `{url}`"),
            };
        }
    }
    for (key, value) in &payload.kv {
        if key.eq_ignore_ascii_case("amount")
            && !value.trim().is_empty()
            && !looks_like_amount(value)
        {
            return ValidationResult::Malformed {
                field: FieldRequirement::Kv(key.clone()),
                reason: format!("amount must be numeric, got `{value}`"),
            };
        }
    }
    // Forbidden fields take Malformed precedence over Incomplete: a payload
    // shipping a field the schema explicitly disallows is wrong-shape, not
    // missing-shape, so the author sees the actual error first. See
    // [[removed-docs Syntax#Capture Payload Validation]].
    for forbidden in &schema.forbidden {
        if forbidden.is_satisfied(payload) {
            let label = forbidden.label();
            return ValidationResult::Malformed {
                field: forbidden.clone(),
                reason: format!("{label} is not allowed for ;{}", schema.target),
            };
        }
    }
    let missing = schema.missing_required(payload);
    if missing.is_empty() {
        ValidationResult::Ready
    } else {
        ValidationResult::Incomplete { missing }
    }
}

fn is_well_formed_url(s: &str) -> bool {
    let lower = s.trim().to_ascii_lowercase();
    lower.starts_with("http://") || lower.starts_with("https://")
}

fn looks_like_amount(v: &str) -> bool {
    let trimmed = v
        .trim()
        .trim_start_matches('$')
        .trim_start_matches('-')
        .trim_start_matches('+');
    if trimmed.is_empty() {
        return false;
    }
    // f64::parse accepts "NaN", "inf", "infinity" — currency values can be
    // neither, so reject explicitly. Closes Run 11 Pass 16 [?]
    // `validate-amount-accepts-nan-inf-as-numeric`.
    match trimmed.parse::<f64>() {
        Ok(n) => n.is_finite(),
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menu_syntax::payload::{CaptureAlias, DatePhrase};

    fn empty_invocation(target: &str) -> CaptureInvocation {
        CaptureInvocation {
            target: target.to_string(),
            alias_form: CaptureAlias::CapturePrefix,
            body: String::new(),
            tags: vec![],
            priority: None,
            url: None,
            duration: None,
            kv: vec![],
            date_phrases: vec![],
            raw: format!("+{}", target),
        }
    }

    fn invocation_with_body(target: &str, body: &str) -> CaptureInvocation {
        let mut inv = empty_invocation(target);
        inv.body = body.to_string();
        inv.raw = format!("+{} {}", target, body);
        inv
    }

    #[test]
    fn cal_requires_body_and_date() {
        let schema = builtin_schema("cal").expect("cal schema must be registered");
        assert_eq!(schema.target, "cal");
        assert!(schema.required.contains(&FieldRequirement::Body));
        assert!(schema.required.contains(&FieldRequirement::AnyDate));
        assert_eq!(schema.required.len(), 2);
    }

    #[test]
    fn link_requires_url_not_body() {
        let schema = builtin_schema("link").expect("link schema must be registered");
        assert_eq!(schema.required, vec![FieldRequirement::Url]);
        assert!(schema.optional.contains(&FieldRequirement::Body));
    }

    #[test]
    fn todo_requires_only_body() {
        let schema = builtin_schema("todo").expect("todo schema must be registered");
        assert_eq!(schema.required, vec![FieldRequirement::Body]);
        assert!(schema.optional.contains(&FieldRequirement::Priority));
        assert!(schema.optional.contains(&FieldRequirement::AnyDate));
    }

    #[test]
    fn cal_missing_required_lists_body_and_date() {
        let schema = builtin_schema("cal").unwrap();
        let missing = schema.missing_required(&empty_invocation("cal"));
        assert_eq!(missing.len(), 2);
        assert!(missing.contains(&FieldRequirement::Body));
        assert!(missing.contains(&FieldRequirement::AnyDate));
    }

    #[test]
    fn cal_with_body_only_still_missing_date() {
        let schema = builtin_schema("cal").unwrap();
        let missing = schema.missing_required(&invocation_with_body("cal", "Design review"));
        assert_eq!(missing, vec![FieldRequirement::AnyDate]);
    }

    #[test]
    fn cal_with_body_and_date_is_complete() {
        let schema = builtin_schema("cal").unwrap();
        let mut inv = invocation_with_body("cal", "Design review");
        inv.date_phrases.push(DatePhrase {
            role: DateRole::Start,
            source: "friday 2pm".to_string(),
            source_span: (0, 10),
        });
        assert!(schema.missing_required(&inv).is_empty());
    }

    #[test]
    fn link_without_url_is_missing_url() {
        let schema = builtin_schema("link").unwrap();
        let missing = schema.missing_required(&invocation_with_body("link", "Some title"));
        assert_eq!(missing, vec![FieldRequirement::Url]);
    }

    #[test]
    fn link_with_url_is_complete_even_without_body() {
        let schema = builtin_schema("link").unwrap();
        let mut inv = empty_invocation("link");
        inv.url = Some("https://zed.dev".to_string());
        assert!(schema.missing_required(&inv).is_empty());
    }

    #[test]
    fn unknown_target_has_no_schema() {
        assert!(builtin_schema("github").is_none());
        assert!(builtin_schema("expense").is_none());
        assert!(builtin_schema("").is_none());
    }

    #[test]
    fn shipped_dynamic_targets_do_not_have_builtin_schema() {
        for target in ["gcal", "github", "expense", "fixture"] {
            assert!(
                builtin_schema(target).is_none(),
                "`{target}` should not gain a builtin schema by accident"
            );
        }
    }

    #[test]
    fn builtin_target_slugs_match_known_targets() {
        let slugs: Vec<&str> = builtin_target_slugs().to_vec();
        assert_eq!(slugs.len(), 6);
        assert_eq!(
            slugs,
            vec!["todo", "note", "link", "snippet", "cal", "social"]
        );
        for slug in &slugs {
            assert!(
                builtin_schema(slug).is_some(),
                "builtin slug {} must have a schema",
                slug
            );
        }
    }

    #[test]
    fn mcal_uses_calendar_schema_but_is_not_core_builtin_slug() {
        let cal = builtin_schema("cal").expect("cal schema");
        let mcal = builtin_schema("mcal").expect("mcal schema");

        assert_eq!(mcal.target, "mcal");
        assert_eq!(mcal.required, cal.required);
        assert_eq!(mcal.optional, cal.optional);
        assert_eq!(mcal.forbidden, cal.forbidden);
        assert!(
            !builtin_target_slugs().contains(&"mcal"),
            "mcal is schema-known, but not one of the core builtin slugs"
        );
    }

    #[test]
    fn field_requirement_label_is_human_readable() {
        assert_eq!(FieldRequirement::Body.label(), "body");
        assert_eq!(FieldRequirement::Url.label(), "url");
        assert_eq!(FieldRequirement::AnyDate.label(), "date");
        assert_eq!(
            FieldRequirement::DateRole(DateRole::Start).label(),
            "start time"
        );
        assert_eq!(FieldRequirement::Kv("amount".to_string()).label(), "amount");
    }

    #[test]
    fn target_lookup_is_case_insensitive() {
        assert!(builtin_schema("CAL").is_some());
        assert!(builtin_schema("Todo").is_some());
        assert!(builtin_schema("LinK").is_some());
    }

    #[test]
    fn todo_alias_schemas_accept_expected_temporal_fields() {
        let reminder = builtin_schema("reminder").expect("reminder schema");
        assert_eq!(reminder.target, "reminder");
        assert!(reminder.required.contains(&FieldRequirement::Body));
        assert!(reminder.optional.contains(&FieldRequirement::AnyDate));
        assert!(reminder.optional.contains(&FieldRequirement::Duration));

        let snooze = builtin_schema("snooze").expect("snooze schema");
        assert!(snooze.required.contains(&FieldRequirement::AnyDate));
        assert!(snooze.optional.contains(&FieldRequirement::ObjectSelection));

        let defer = builtin_schema("defer").expect("defer schema");
        assert!(defer.required.contains(&FieldRequirement::AnyDate));
        assert!(defer.optional.contains(&FieldRequirement::Priority));
    }

    #[test]
    fn selected_todo_ref_satisfies_todo_alias_body_requirement() {
        let schema = builtin_schema("snooze").expect("snooze schema");
        let mut inv = empty_invocation("snooze");
        inv.raw = ";snooze @todo:todo/tmp/sk/menu-syntax/todos.jsonl:1 in 30 minutes".to_string();
        inv.date_phrases.push(DatePhrase {
            role: DateRole::Inferred,
            source: "in 30 minutes".to_string(),
            source_span: (55, 68),
        });
        let missing = schema.missing_required(&inv);
        assert!(
            missing.is_empty(),
            "selected todo should satisfy body: {missing:?}"
        );
    }

    #[test]
    fn unresolved_todo_query_does_not_satisfy_todo_alias_body_requirement() {
        let schema = builtin_schema("snooze").expect("snooze schema");
        let mut inv = empty_invocation("snooze");
        inv.raw = ";snooze @Review in 30 minutes".to_string();
        inv.date_phrases.push(DatePhrase {
            role: DateRole::Inferred,
            source: "in 30 minutes".to_string(),
            source_span: (16, 29),
        });
        let missing = schema.missing_required(&inv);
        assert_eq!(missing, vec![FieldRequirement::Body]);
    }

    #[test]
    fn notes_alias_uses_note_schema() {
        let note = builtin_schema("note").expect("note schema");
        let notes = builtin_schema("notes").expect("notes schema");
        assert_eq!(notes.target, "notes");
        assert_eq!(notes.required, note.required);
        assert_eq!(notes.optional, note.optional);
        assert_eq!(notes.forbidden, note.forbidden);
    }

    #[test]
    fn snippet_schema_requires_body_and_accepts_snippet_fields() {
        let schema = builtin_schema("snippet").expect("snippet schema");
        assert!(schema.required.contains(&FieldRequirement::Body));
        assert!(schema
            .required
            .contains(&FieldRequirement::SnippetNameOrSelection));
        assert!(schema
            .optional
            .contains(&FieldRequirement::Kv("keyword".to_string())));
        assert!(schema
            .optional
            .contains(&FieldRequirement::Kv("description".to_string())));
        assert!(schema
            .optional
            .contains(&FieldRequirement::Kv("tool".to_string())));
        assert!(schema.optional.contains(&FieldRequirement::ObjectSelection));
    }

    #[test]
    fn link_update_selected_ref_satisfies_url_requirement() {
        let schema = builtin_schema("link").unwrap();
        let mut inv = invocation_with_body("link", "update");
        inv.raw = ";link update @link:https://example.com title:New".to_string();
        inv.kv.push(("title".to_string(), "New".to_string()));

        assert_eq!(validate(&inv, &schema), ValidationResult::Ready);
    }

    #[test]
    fn link_create_selected_ref_does_not_satisfy_url_requirement() {
        let schema = builtin_schema("link").unwrap();
        let mut inv = empty_invocation("link");
        inv.raw = ";link @link:https://example.com".to_string();

        assert_eq!(schema.missing_required(&inv), vec![FieldRequirement::Url]);
    }

    #[test]
    fn link_update_bad_selected_ref_url_is_malformed() {
        let schema = builtin_schema("link").unwrap();
        let mut inv = invocation_with_body("link", "update");
        inv.raw = ";link update @link:not-a-url".to_string();

        assert_eq!(
            validate(&inv, &schema),
            ValidationResult::Malformed {
                field: FieldRequirement::Url,
                reason: "URL must start with http:// or https://, got `not-a-url`".to_string(),
            }
        );
    }

    #[test]
    fn snippet_remove_selected_ref_is_ready_without_body() {
        let schema = builtin_schema("snippet").unwrap();
        let mut inv = invocation_with_body("snippet", "remove");
        inv.raw = ";snippet remove @snippet:fj".to_string();

        assert_eq!(validate(&inv, &schema), ValidationResult::Ready);
    }

    #[test]
    fn snippet_remove_without_name_or_selection_is_incomplete() {
        let schema = builtin_schema("snippet").unwrap();
        let inv = invocation_with_body("snippet", "remove");

        assert_eq!(
            schema.missing_required(&inv),
            vec![FieldRequirement::SnippetNameOrSelection]
        );
    }

    #[test]
    fn snippet_create_selected_ref_does_not_satisfy_name_requirement() {
        let schema = builtin_schema("snippet").unwrap();
        let mut inv = invocation_with_body("snippet", "add");
        inv.raw = ";snippet add @snippet:fj -- const x = 1".to_string();
        inv.body = "add const x = 1".to_string();

        assert_eq!(
            schema.missing_required(&inv),
            vec![FieldRequirement::SnippetNameOrSelection]
        );
    }

    #[test]
    fn selected_note_ref_with_date_satisfies_note_update_body_requirement() {
        let schema = builtin_schema("note").unwrap();
        let mut inv = empty_invocation("note");
        inv.raw = ";note @note:550e8400-e29b-41d4-a716-446655440000 due:tomorrow".to_string();
        inv.date_phrases
            .push(crate::menu_syntax::payload::DatePhrase {
                role: DateRole::Due,
                source: "tomorrow".to_string(),
                source_span: (46, 54),
            });

        assert_eq!(validate(&inv, &schema), ValidationResult::Ready);
    }

    #[test]
    fn selected_note_ref_without_mutation_still_needs_body_or_fields() {
        let schema = builtin_schema("note").unwrap();
        let mut inv = empty_invocation("note");
        inv.raw = ";note @note:550e8400-e29b-41d4-a716-446655440000".to_string();

        assert_eq!(schema.missing_required(&inv), vec![FieldRequirement::Body]);
    }

    #[test]
    fn note_allows_date_url_and_metadata_kv_but_forbids_priority_duration() {
        let schema = builtin_schema("note").unwrap();

        assert!(schema.optional.contains(&FieldRequirement::AnyDate));
        assert!(schema.optional.contains(&FieldRequirement::Url));
        assert!(schema.optional.contains(&FieldRequirement::ObjectSelection));
        assert!(schema
            .optional
            .contains(&FieldRequirement::Kv("title".to_string())));
        assert!(schema
            .optional
            .contains(&FieldRequirement::Kv("alias".to_string())));
        assert!(schema.forbidden.contains(&FieldRequirement::Priority));
        assert!(schema.forbidden.contains(&FieldRequirement::Duration));
    }

    #[test]
    fn whitespace_only_body_does_not_satisfy_body_requirement() {
        let schema = builtin_schema("todo").unwrap();
        let mut inv = empty_invocation("todo");
        inv.body = "   \t  ".to_string();
        let missing = schema.missing_required(&inv);
        assert_eq!(missing, vec![FieldRequirement::Body]);
    }

    fn cal_with_body_and_date() -> CaptureInvocation {
        let mut inv = invocation_with_body("cal", "Design review");
        inv.date_phrases.push(DatePhrase {
            role: DateRole::Start,
            source: "friday 2pm".to_string(),
            source_span: (0, 10),
        });
        inv
    }

    #[test]
    fn validate_ready_when_all_required_present_and_well_formed() {
        let schema = builtin_schema("cal").unwrap();
        let inv = cal_with_body_and_date();
        assert_eq!(validate(&inv, &schema), ValidationResult::Ready);
    }

    #[test]
    fn validate_incomplete_missing_body_for_todo() {
        let schema = builtin_schema("todo").unwrap();
        let inv = empty_invocation("todo");
        match validate(&inv, &schema) {
            ValidationResult::Incomplete { missing } => {
                assert_eq!(missing, vec![FieldRequirement::Body]);
            }
            other => panic!("expected Incomplete missing body, got {other:?}"),
        }
    }

    #[test]
    fn validate_incomplete_missing_date_for_cal_with_body_only() {
        let schema = builtin_schema("cal").unwrap();
        let inv = invocation_with_body("cal", "Design review");
        match validate(&inv, &schema) {
            ValidationResult::Incomplete { missing } => {
                assert_eq!(missing, vec![FieldRequirement::AnyDate]);
            }
            other => panic!("expected Incomplete missing date, got {other:?}"),
        }
    }

    #[test]
    fn validate_malformed_amount_kv_beats_incomplete() {
        // Malformed wins over Incomplete — surface the bad field even when other
        // required fields are also missing so authors see the actual error.
        let schema = CaptureFieldSchema {
            target: "expense".to_string(),
            required: vec![
                FieldRequirement::Body,
                FieldRequirement::Kv("amount".to_string()),
            ],
            optional: vec![],
            forbidden: vec![],
        };
        let mut inv = empty_invocation("expense");
        inv.kv
            .push(("amount".to_string(), "not-a-number".to_string()));
        match validate(&inv, &schema) {
            ValidationResult::Malformed { field, reason } => {
                assert_eq!(field, FieldRequirement::Kv("amount".to_string()));
                assert!(
                    reason.contains("amount"),
                    "reason should name field: {reason}"
                );
                assert!(
                    reason.contains("not-a-number"),
                    "reason should quote bad value: {reason}"
                );
            }
            other => panic!("expected Malformed amount, got {other:?}"),
        }
    }

    #[test]
    fn validate_malformed_url_for_link_with_garbage_url() {
        let schema = builtin_schema("link").unwrap();
        let mut inv = empty_invocation("link");
        inv.url = Some("ftp://nope".to_string());
        match validate(&inv, &schema) {
            ValidationResult::Malformed { field, reason } => {
                assert_eq!(field, FieldRequirement::Url);
                assert!(
                    reason.contains("http"),
                    "reason should mention scheme: {reason}"
                );
            }
            other => panic!("expected Malformed url, got {other:?}"),
        }
    }

    #[test]
    fn validate_amount_accepts_decimals_and_signs() {
        let schema = CaptureFieldSchema {
            target: "expense".to_string(),
            required: vec![
                FieldRequirement::Body,
                FieldRequirement::Kv("amount".to_string()),
            ],
            optional: vec![],
            forbidden: vec![],
        };
        for good in ["18.50", "$18.50", "-5", "+12.0", "0", "100"] {
            let mut inv = invocation_with_body("expense", "Lunch");
            inv.kv.push(("amount".to_string(), good.to_string()));
            assert_eq!(
                validate(&inv, &schema),
                ValidationResult::Ready,
                "amount `{good}` should be Ready"
            );
        }
    }

    #[test]
    fn validate_empty_amount_kv_falls_through_to_incomplete() {
        // Empty value is "not provided", so it's Incomplete (missing), not Malformed.
        let schema = CaptureFieldSchema {
            target: "expense".to_string(),
            required: vec![
                FieldRequirement::Body,
                FieldRequirement::Kv("amount".to_string()),
            ],
            optional: vec![],
            forbidden: vec![],
        };
        let mut inv = invocation_with_body("expense", "Lunch");
        inv.kv.push(("amount".to_string(), "  ".to_string()));
        match validate(&inv, &schema) {
            ValidationResult::Incomplete { missing } => {
                assert!(missing
                    .iter()
                    .any(|m| matches!(m, FieldRequirement::Kv(k) if k == "amount")));
            }
            other => panic!("expected Incomplete (empty amount counts as missing), got {other:?}"),
        }
    }

    #[test]
    fn validate_amount_rejects_nan_and_infinity() {
        // Closes Run 11 Pass 16 [?] validate-amount-accepts-nan-inf-as-numeric.
        // f64::parse accepts these tokens; currency cannot be NaN or infinite.
        let schema = CaptureFieldSchema {
            target: "expense".to_string(),
            required: vec![
                FieldRequirement::Body,
                FieldRequirement::Kv("amount".to_string()),
            ],
            optional: vec![],
            forbidden: vec![],
        };
        for bad in [
            "NaN",
            "nan",
            "inf",
            "-inf",
            "infinity",
            "+infinity",
            "Infinity",
        ] {
            let mut inv = invocation_with_body("expense", "Lunch");
            inv.kv.push(("amount".to_string(), bad.to_string()));
            match validate(&inv, &schema) {
                ValidationResult::Malformed { field, reason } => {
                    assert_eq!(field, FieldRequirement::Kv("amount".to_string()));
                    assert!(
                        reason.contains(bad),
                        "reason should quote bad value `{bad}`: {reason}"
                    );
                }
                other => panic!("expected Malformed for amount=`{bad}`, got {other:?}"),
            }
        }
    }

    #[test]
    fn validate_malformed_when_forbidden_field_present() {
        // Closes Run 11 Pass 16 [?] validate-ignores-schema-forbidden-fields.
        // `cal` forbids Priority + Url. Setting either must Malformed even when
        // all required fields are also satisfied (otherwise the gate would let
        // a wrong-shape payload reach the handler).
        let schema = builtin_schema("cal").unwrap();
        let mut inv = invocation_with_body("cal", "Design review");
        inv.date_phrases
            .push(crate::menu_syntax::payload::DatePhrase {
                role: crate::menu_syntax::payload::DateRole::Inferred,
                source: "friday 2pm".to_string(),
                source_span: (0, 10),
            });
        inv.priority = Some(1u8);
        match validate(&inv, &schema) {
            ValidationResult::Malformed { field, reason } => {
                assert_eq!(field, FieldRequirement::Priority);
                assert!(
                    reason.contains("priority"),
                    "reason should name field: {reason}"
                );
                assert!(
                    reason.contains(";cal"),
                    "reason should name target: {reason}"
                );
            }
            other => panic!("expected Malformed for forbidden Priority on +cal, got {other:?}"),
        }
    }

    #[test]
    fn validate_forbidden_url_on_cal_beats_incomplete() {
        // Forbidden takes precedence over Incomplete: even when other required
        // fields are still missing, the wrong-shape field surfaces first.
        let schema = builtin_schema("cal").unwrap();
        let mut inv = empty_invocation("cal"); // no body, no date_phrase
        inv.url = Some("https://example.com".to_string()); // well-formed but forbidden
        match validate(&inv, &schema) {
            ValidationResult::Malformed { field, .. } => {
                assert_eq!(field, FieldRequirement::Url);
            }
            other => panic!("expected Malformed forbidden Url on +cal, got {other:?}"),
        }
    }

    #[test]
    fn validate_ready_unaffected_when_no_forbidden_fields_set() {
        // Defensive: a payload that uses NONE of the forbidden fields must
        // continue to validate Ready. Falsifier from the [?] story.
        let schema = builtin_schema("cal").unwrap();
        let mut inv = invocation_with_body("cal", "Design review");
        inv.date_phrases
            .push(crate::menu_syntax::payload::DatePhrase {
                role: crate::menu_syntax::payload::DateRole::Inferred,
                source: "friday 2pm".to_string(),
                source_span: (0, 10),
            });
        // priority/url left at None — no forbidden field is satisfied.
        assert_eq!(validate(&inv, &schema), ValidationResult::Ready);
    }

    #[test]
    fn empty_kv_value_does_not_satisfy_kv_requirement() {
        let req = FieldRequirement::Kv("amount".to_string());
        let mut inv = empty_invocation("expense");
        inv.kv.push(("amount".to_string(), "  ".to_string()));
        assert!(!req.is_satisfied(&inv));
        inv.kv.clear();
        inv.kv.push(("amount".to_string(), "18.50".to_string()));
        assert!(req.is_satisfied(&inv));
    }
}
