use std::sync::Arc;

use crate::scripts::{Script, Scriptlet, SearchResult};

use super::payload::{
    AdvancedQuery, ArtifactKind, CaptureInvocation, MenuSyntaxHandlerSpec, Predicate,
    ShortcutPredicate,
};

pub fn apply_advanced_query(
    results: Vec<SearchResult>,
    query: &AdvancedQuery,
) -> Vec<SearchResult> {
    if query.predicates.is_empty() {
        return results;
    }
    results
        .into_iter()
        .filter(|r| matches_all(r, &query.predicates))
        .collect()
}

fn matches_all(result: &SearchResult, predicates: &[Predicate]) -> bool {
    predicates.iter().all(|p| matches_predicate(result, p))
}

pub fn matches_predicate(result: &SearchResult, predicate: &Predicate) -> bool {
    match predicate {
        Predicate::Negate(inner) => !matches_predicate(result, inner),
        Predicate::Type(kind) => result_kind(result) == *kind,
        Predicate::Tag(tag) => matches_tag(result, tag),
        Predicate::HasShortcut(sp) => matches_shortcut(result, sp),
        Predicate::Source(s) => matches_source(result, s),
        Predicate::Plugin(s) => matches_plugin(result, s),
        Predicate::Name(s) => contains_ci(result.name(), s),
        Predicate::Desc(s) => matches_desc(result, s),
        Predicate::Alias(s) => matches_alias(result, s),
        Predicate::Has(field) => has_field(result, field),
        Predicate::MetaPath { path, value } => matches_meta_path(result, path, value),
    }
}

pub fn result_kind(result: &SearchResult) -> ArtifactKind {
    match result {
        SearchResult::Script(_) => ArtifactKind::Script,
        SearchResult::Scriptlet(_) => ArtifactKind::Scriptlet,
        SearchResult::Skill(_) => ArtifactKind::Skill,
        SearchResult::BuiltIn(_) => ArtifactKind::Builtin,
        SearchResult::App(_) => ArtifactKind::App,
        SearchResult::Window(_) => ArtifactKind::Window,
        SearchResult::File(_) => ArtifactKind::File,
        SearchResult::Note(_) => ArtifactKind::Note,
        // Brain rows have no dedicated ArtifactKind; the passive brain section
        // never renders alongside advanced queries, so map to Fallback like
        // SpineProjection.
        SearchResult::BrainHit(_) => ArtifactKind::Fallback,
        SearchResult::Todo(_) => ArtifactKind::Todo,
        SearchResult::AgentChatHistory(_) => ArtifactKind::AgentChatHistory,
        SearchResult::AiVault(_) => ArtifactKind::AiVault,
        SearchResult::ClipboardHistory(_) => ArtifactKind::ClipboardHistory,
        SearchResult::DictationHistory(_) => ArtifactKind::DictationHistory,
        SearchResult::BrowserTab(_) => ArtifactKind::BrowserTab,
        SearchResult::BrowserHistory(_) => ArtifactKind::BrowserHistory,
        SearchResult::Agent(_) => ArtifactKind::Agent,
        SearchResult::Fallback(_) => ArtifactKind::Fallback,
        SearchResult::ScriptIssue(_) => ArtifactKind::Issue,
        // Spine projections don't have a dedicated ArtifactKind; map to Fallback
        SearchResult::SpineProjection(_) => ArtifactKind::Fallback,
    }
}

fn matches_shortcut(result: &SearchResult, sp: &ShortcutPredicate) -> bool {
    let shortcut = result_shortcut(result);
    match sp {
        ShortcutPredicate::Any => shortcut.is_some(),
        ShortcutPredicate::None => shortcut.is_none(),
        ShortcutPredicate::Literal(s) => match shortcut {
            Some(actual) => actual.eq_ignore_ascii_case(s),
            None => false,
        },
    }
}

fn result_shortcut(result: &SearchResult) -> Option<&str> {
    match result {
        SearchResult::Script(sm) => sm.script.shortcut.as_deref(),
        SearchResult::Scriptlet(sm) => sm.scriptlet.shortcut.as_deref(),
        _ => None,
    }
}

fn matches_source(result: &SearchResult, s: &str) -> bool {
    if matches_plugin(result, s) {
        return true;
    }
    match result {
        SearchResult::Script(sm) => sm
            .script
            .kit_name
            .as_deref()
            .map(|t| contains_ci(t, s))
            .unwrap_or(false),
        _ => false,
    }
}

fn matches_plugin(result: &SearchResult, s: &str) -> bool {
    match result {
        SearchResult::Script(sm) => {
            matches_plugin_pair(&sm.script.plugin_id, sm.script.plugin_title.as_deref(), s)
        }
        SearchResult::Scriptlet(sm) => matches_plugin_pair(
            &sm.scriptlet.plugin_id,
            sm.scriptlet.plugin_title.as_deref(),
            s,
        ),
        SearchResult::Skill(sm) => {
            matches_plugin_pair(&sm.skill.plugin_id, Some(&sm.skill.plugin_title), s)
        }
        _ => false,
    }
}

fn matches_plugin_pair(plugin_id: &str, plugin_title: Option<&str>, query: &str) -> bool {
    contains_ci(plugin_id, query) || plugin_title.map(|t| contains_ci(t, query)).unwrap_or(false)
}

fn matches_desc(result: &SearchResult, s: &str) -> bool {
    result
        .description()
        .map(|d| contains_ci(d, s))
        .unwrap_or(false)
}

fn matches_alias(result: &SearchResult, s: &str) -> bool {
    match result {
        SearchResult::Script(sm) => sm
            .script
            .alias
            .as_deref()
            .map(|a| contains_ci(a, s))
            .unwrap_or(false),
        SearchResult::Scriptlet(sm) => sm
            .scriptlet
            .alias
            .as_deref()
            .map(|a| contains_ci(a, s))
            .unwrap_or(false),
        _ => false,
    }
}

fn matches_tag(result: &SearchResult, tag: &str) -> bool {
    match result {
        SearchResult::Script(sm) => sm
            .script
            .typed_metadata
            .as_ref()
            .map(|meta| {
                meta.tags
                    .iter()
                    .any(|candidate| tag_matches(candidate, tag))
                    || meta
                        .extra
                        .get("tags")
                        .map(|value| value_matches_tag(value, tag))
                        .unwrap_or(false)
            })
            .unwrap_or(false),
        SearchResult::Skill(sm) => {
            tag_matches(&sm.skill.plugin_id, tag) || tag_matches(&sm.skill.plugin_title, tag)
        }
        SearchResult::Scriptlet(sm) => sm
            .scriptlet
            .group
            .as_deref()
            .map(|group| tag_matches(group, tag))
            .unwrap_or(false),
        SearchResult::Todo(tm) => tm
            .hit
            .tags
            .iter()
            .any(|candidate| tag_matches(candidate, tag)),
        _ => false,
    }
}

fn value_matches_tag(value: &serde_json::Value, tag: &str) -> bool {
    match value {
        serde_json::Value::String(s) => tag_matches(s, tag),
        serde_json::Value::Array(items) => items.iter().any(|item| value_matches_tag(item, tag)),
        _ => false,
    }
}

fn tag_matches(candidate: &str, tag: &str) -> bool {
    let candidate = candidate.trim_start_matches('#');
    let tag = tag.trim_start_matches('#');
    candidate.eq_ignore_ascii_case(tag)
}

fn has_field(result: &SearchResult, field: &str) -> bool {
    let field_lower = field.to_ascii_lowercase();
    match result {
        SearchResult::Script(sm) => script_has_field(&sm.script, &field_lower),
        SearchResult::Scriptlet(sm) => scriptlet_has_field(&sm.scriptlet, &field_lower),
        _ => false,
    }
}

fn script_has_field(script: &Script, field: &str) -> bool {
    match field {
        "shortcut" => script.shortcut.is_some(),
        "alias" => script.alias.is_some(),
        "icon" => script.icon.is_some(),
        "description" | "desc" => script.description.is_some(),
        "schema" => script.schema.is_some(),
        "menusyntax" | "menu_syntax" => !script_menu_syntax_specs(script).is_empty(),
        other => script
            .typed_metadata
            .as_ref()
            .map(|m| m.extra.keys().any(|k| k.eq_ignore_ascii_case(other)))
            .unwrap_or(false),
    }
}

fn scriptlet_has_field(scriptlet: &Scriptlet, field: &str) -> bool {
    match field {
        "shortcut" => scriptlet.shortcut.is_some(),
        "alias" => scriptlet.alias.is_some(),
        "description" | "desc" => scriptlet.description.is_some(),
        "keyword" => scriptlet.keyword.is_some(),
        "group" => scriptlet.group.is_some(),
        "command" => scriptlet.command.is_some(),
        _ => false,
    }
}

fn matches_meta_path(result: &SearchResult, path: &str, query: &str) -> bool {
    let SearchResult::Script(sm) = result else {
        return false;
    };
    let Some(meta) = sm.script.typed_metadata.as_ref() else {
        return false;
    };
    let mut parts = path.split('.');
    let first = match parts.next() {
        Some(f) if !f.is_empty() => f,
        _ => return false,
    };
    let Some((_, root_value)) = meta
        .extra
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(first))
    else {
        return false;
    };

    let mut current: serde_json::Value = root_value.clone();
    for part in parts {
        current = match current {
            serde_json::Value::Object(map) => {
                match map.iter().find(|(k, _)| k.eq_ignore_ascii_case(part)) {
                    Some((_, v)) => v.clone(),
                    None => return false,
                }
            }
            _ => return false,
        };
    }

    value_matches_query(&current, query)
}

fn value_matches_query(value: &serde_json::Value, query: &str) -> bool {
    match value {
        serde_json::Value::String(s) => contains_ci(s, query),
        serde_json::Value::Bool(b) => {
            let q = query.to_ascii_lowercase();
            (*b && matches!(q.as_str(), "true" | "yes" | "1"))
                || (!*b && matches!(q.as_str(), "false" | "no" | "0"))
        }
        serde_json::Value::Number(n) => n.to_string() == query,
        serde_json::Value::Array(items) => items.iter().any(|v| value_matches_query(v, query)),
        serde_json::Value::Null => query.eq_ignore_ascii_case("null"),
        serde_json::Value::Object(_) => false,
    }
}

fn contains_ci(haystack: &str, needle: &str) -> bool {
    haystack
        .to_ascii_lowercase()
        .contains(&needle.to_ascii_lowercase())
}

pub fn script_menu_syntax_specs(script: &Script) -> Vec<MenuSyntaxHandlerSpec> {
    match script.typed_metadata.as_ref() {
        Some(meta) => super::metadata::handler_specs_from_extra_map(&meta.extra),
        None => Vec::new(),
    }
}

pub fn script_handles_capture(script: &Script, invocation: &CaptureInvocation) -> bool {
    script_menu_syntax_specs(script)
        .iter()
        .any(|spec| spec.handles_capture_target(&invocation.target))
}

pub fn capture_accepts_for_target_from_scripts(
    scripts: &[Arc<Script>],
    target: &str,
) -> Vec<String> {
    scripts
        .iter()
        .flat_map(|script| script_menu_syntax_specs(script).into_iter())
        .find(|spec| spec.handles_capture_target(target))
        .map(|spec| spec.accepts)
        .unwrap_or_default()
}

/// Run 13 Pass 2 (user bug report) — first concrete capture target a script
/// declares via its `menuSyntax` metadata. Used by the main script-list
/// Enter path to pivot to the power-syntax composer (`+target `) instead
/// of running the script process directly, which would crash on the
/// missing `KIT_MENU_SYNTAX_PAYLOAD_PATH` env var. Returns `None` when the
/// script is not a menu-syntax capture handler at all, or when its only
/// declared targets are wildcard `*` (no concrete pivot target).
pub fn first_concrete_capture_target_for_script(script: &Script) -> Option<String> {
    for spec in script_menu_syntax_specs(script) {
        if spec.family != "capture.v1" {
            continue;
        }
        for target in &spec.targets {
            if target == "*" || target.is_empty() {
                continue;
            }
            return Some(target.clone());
        }
    }
    None
}

/// Run 13 Pass 4 — symmetric to [`first_concrete_capture_target_for_script`]
/// but for `command.v1` handlers. Returns the first non-empty `head` slug
/// declared by any of the script's command handlers. Used by the main
/// script-list Enter path to pivot to the command composer (`!head `)
/// instead of running the script process directly.
pub fn first_command_head_for_script(script: &Script) -> Option<String> {
    for spec in script_menu_syntax_specs(script) {
        if spec.family != "command.v1" {
            continue;
        }
        if let Some(head) = spec.head.as_deref() {
            let head = head.trim();
            if !head.is_empty() {
                return Some(head.to_string());
            }
        }
    }
    None
}

pub fn scripts_handling_capture(
    scripts: &[Arc<Script>],
    invocation: &CaptureInvocation,
) -> Vec<Arc<Script>> {
    scripts
        .iter()
        .filter(|s: &&Arc<Script>| script_handles_capture(s.as_ref(), invocation))
        .cloned()
        .collect()
}

/// Find the first `command.v1` handler spec whose `head` matches `head`
/// case-insensitively across the loaded scripts. Returns the spec by value
/// because the hint surface needs to render schema-derived rows from it.
pub fn script_command_schema_for(
    scripts: &[Arc<Script>],
    head: &str,
) -> Option<MenuSyntaxHandlerSpec> {
    scripts
        .iter()
        .flat_map(|script| script_menu_syntax_specs(script).into_iter())
        .find(|spec| spec.handles_command_head(head))
}

pub fn registered_capture_targets_from_scripts(scripts: &[Arc<Script>]) -> Vec<String> {
    let mut targets: Vec<String> = Vec::new();
    for script in scripts {
        for spec in script_menu_syntax_specs(script) {
            if spec.family != "capture.v1" {
                continue;
            }
            for target in spec.targets {
                let slug = target.trim().to_ascii_lowercase();
                if slug.is_empty() || slug == "*" {
                    continue;
                }
                if !targets.iter().any(|existing| existing == &slug) {
                    targets.push(slug);
                }
            }
        }
    }
    targets.sort();
    targets
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metadata_parser::TypedMetadata;
    use crate::scripts::{
        MatchIndices, ScriptMatch, ScriptMatchKind, ScriptletMatch, SearchResult, TodoMatch,
    };
    use serde_json::json;
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::sync::Arc;

    fn make_script_with_extra(
        name: &str,
        extra: HashMap<String, serde_json::Value>,
    ) -> Arc<Script> {
        let mut meta = TypedMetadata::default();
        meta.extra = extra;
        Arc::new(Script {
            name: name.to_string(),
            path: PathBuf::from(format!("/tmp/{name}.ts")),
            extension: "ts".to_string(),
            description: Some(format!("{name} description")),
            icon: None,
            alias: None,
            shortcut: None,
            typed_metadata: Some(meta),
            schema: None,
            plugin_id: "main".to_string(),
            plugin_title: None,
            kit_name: None,
            body: None,
        })
    }

    fn script_match(script: Arc<Script>) -> SearchResult {
        SearchResult::Script(ScriptMatch {
            script,
            score: 100,
            filename: "x".to_string(),
            match_indices: MatchIndices::default(),
            match_kind: ScriptMatchKind::Name,
            content_match: None,
            match_evidence: None,
        })
    }

    fn scriptlet_match(scriptlet: Arc<Scriptlet>) -> SearchResult {
        SearchResult::Scriptlet(ScriptletMatch {
            scriptlet,
            score: 100,
            display_file_path: None,
            match_indices: MatchIndices::default(),
            match_evidence: None,
        })
    }

    fn todo_match(title: &str, tags: &[&str]) -> SearchResult {
        SearchResult::Todo(TodoMatch {
            hit: crate::menu_syntax::RootTodoSearchHit {
                stable_key: format!("todo/test/{title}"),
                title: title.to_string(),
                body: title.to_string(),
                subtitle: "Captured todo".to_string(),
                tags: tags.iter().map(|tag| tag.to_string()).collect(),
                priority: None,
                due: None,
                created_at: None,
                path: PathBuf::from("/tmp/todos.jsonl"),
                line_number: Some(1),
                raw_line: title.to_string(),
            },
            score: 100,
        })
    }

    #[test]
    fn type_predicate_filters_by_artifact_kind() {
        let s = make_script_with_extra("foo", HashMap::new());
        let results = vec![script_match(s)];

        let query = AdvancedQuery {
            free_text: String::new(),
            predicates: vec![Predicate::Type(ArtifactKind::Script)],
            source_filters: Default::default(),
            raw: ":type:script".to_string(),
        };
        assert_eq!(apply_advanced_query(results.clone(), &query).len(), 1);

        let query2 = AdvancedQuery {
            free_text: String::new(),
            predicates: vec![Predicate::Type(ArtifactKind::Skill)],
            source_filters: Default::default(),
            raw: ":type:skill".to_string(),
        };
        assert_eq!(apply_advanced_query(results, &query2).len(), 0);
    }

    #[test]
    fn tag_predicate_filters_todo_result_tags() {
        let results = vec![
            todo_match("Buy milk", &["home", "errands"]),
            todo_match("Renew passport", &["admin"]),
        ];
        let query = AdvancedQuery {
            free_text: String::new(),
            predicates: vec![Predicate::Tag("errands".to_string())],
            source_filters: Default::default(),
            raw: "todo: #errands".to_string(),
        };

        let filtered = apply_advanced_query(results, &query);

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name(), "Buy milk");
    }

    #[test]
    fn shortcut_any_filters_scripts_with_shortcut() {
        let s_with = {
            let mut s = make_script_with_extra("has", HashMap::new());
            Arc::make_mut(&mut s).shortcut = Some("cmd+g".to_string());
            s
        };
        let s_without = make_script_with_extra("bare", HashMap::new());

        let results = vec![script_match(s_with), script_match(s_without)];
        let query = AdvancedQuery {
            free_text: String::new(),
            predicates: vec![Predicate::HasShortcut(ShortcutPredicate::Any)],
            source_filters: Default::default(),
            raw: ":shortcut:true".to_string(),
        };
        let filtered = apply_advanced_query(results, &query);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name(), "has");
    }

    #[test]
    fn has_shortcut_matches_scriptlet_or_snippet_shortcut_rows() {
        let script_with = {
            let mut script = make_script_with_extra("script shortcut", HashMap::new());
            Arc::make_mut(&mut script).shortcut = Some("cmd+1".to_string());
            script
        };
        let script_without = make_script_with_extra("script bare", HashMap::new());
        let scriptlet_with = Arc::new(Scriptlet {
            name: "Run Snippet".to_string(),
            description: Some("Snippet with shortcut".to_string()),
            code: "console.log('snippet')".to_string(),
            tool: "ts".to_string(),
            shortcut: Some("cmd+2".to_string()),
            keyword: None,
            group: Some("Snippets".to_string()),
            plugin_id: "main".to_string(),
            plugin_title: None,
            file_path: Some("/tmp/snippets.md#run-snippet".to_string()),
            command: None,
            alias: None,
        });
        let scriptlet_without = Arc::new(Scriptlet {
            name: "Bare Snippet".to_string(),
            description: None,
            code: "console.log('bare')".to_string(),
            tool: "ts".to_string(),
            shortcut: None,
            keyword: None,
            group: Some("Snippets".to_string()),
            plugin_id: "main".to_string(),
            plugin_title: None,
            file_path: Some("/tmp/snippets.md#bare-snippet".to_string()),
            command: None,
            alias: None,
        });

        let query = AdvancedQuery {
            free_text: String::new(),
            predicates: vec![Predicate::Has("shortcut".to_string())],
            source_filters: Default::default(),
            raw: "has:shortcut".to_string(),
        };
        let filtered = apply_advanced_query(
            vec![
                script_match(script_with),
                script_match(script_without),
                scriptlet_match(scriptlet_with),
                scriptlet_match(scriptlet_without),
            ],
            &query,
        );
        let names: Vec<_> = filtered.iter().map(|result| result.name()).collect();

        assert_eq!(names, vec!["script shortcut", "Run Snippet"]);
    }

    #[test]
    fn meta_path_matches_nested_value() {
        let mut extra = HashMap::new();
        extra.insert(
            "domain".to_string(),
            json!({ "kind": "calendar", "team": "platform" }),
        );
        let s = make_script_with_extra("cal", extra);
        let results = vec![script_match(s)];

        let query = AdvancedQuery {
            free_text: String::new(),
            predicates: vec![Predicate::MetaPath {
                path: "domain.kind".to_string(),
                value: "calendar".to_string(),
            }],
            source_filters: Default::default(),
            raw: ":meta.domain.kind:calendar".to_string(),
        };
        assert_eq!(apply_advanced_query(results.clone(), &query).len(), 1);

        let query_miss = AdvancedQuery {
            free_text: String::new(),
            predicates: vec![Predicate::MetaPath {
                path: "domain.kind".to_string(),
                value: "analytics".to_string(),
            }],
            source_filters: Default::default(),
            raw: ":meta.domain.kind:analytics".to_string(),
        };
        assert_eq!(apply_advanced_query(results, &query_miss).len(), 0);
    }

    #[test]
    fn has_menu_syntax_filters_to_opted_in_scripts() {
        let mut with_extra = HashMap::new();
        with_extra.insert(
            "menuSyntax".to_string(),
            json!([{ "family": "capture.v1", "targets": ["todo"] }]),
        );
        let with = make_script_with_extra("todo-handler", with_extra);
        let without = make_script_with_extra("plain", HashMap::new());

        let results = vec![script_match(with), script_match(without)];
        let query = AdvancedQuery {
            free_text: String::new(),
            predicates: vec![Predicate::Has("menuSyntax".to_string())],
            source_filters: Default::default(),
            raw: ":has:menuSyntax".to_string(),
        };
        let filtered = apply_advanced_query(results, &query);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name(), "todo-handler");
    }

    #[test]
    fn demo_refine_fixture_matches_tag_has_meta_and_shortcut_predicates() {
        let mut extra = HashMap::new();
        extra.insert(
            "menuSyntax".to_string(),
            json!([{ "family": "capture.v1", "targets": ["fixture"] }]),
        );
        extra.insert(
            "domain".to_string(),
            json!({ "kind": "fixture", "team": "launcher", "localFirst": true }),
        );
        extra.insert("category".to_string(), json!("menu-syntax-demo"));

        let script = {
            let mut s = make_script_with_extra("Power Syntax Refine Fixture", extra);
            let mut_script = Arc::make_mut(&mut s);
            mut_script.alias = Some("ps-refine".to_string());
            mut_script.shortcut = Some("cmd+shift+;".to_string());
            if let Some(meta) = mut_script.typed_metadata.as_mut() {
                meta.tags = vec!["power-syntax".to_string(), "demo".to_string()];
            }
            s
        };

        let results = vec![script_match(script)];
        let query = crate::menu_syntax::query::parse_advanced_query(
            ":#power-syntax has:menuSyntax meta.domain.kind:fixture shortcut:any",
        );
        let filtered = apply_advanced_query(results.clone(), &query);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name(), "Power Syntax Refine Fixture");

        let alias_query = crate::menu_syntax::query::parse_advanced_query(":alias:ps-refine");
        assert_eq!(apply_advanced_query(results, &alias_query).len(), 1);
    }

    #[test]
    fn negation_inverts_predicate() {
        let s = make_script_with_extra("foo", HashMap::new());
        let results = vec![script_match(s)];
        let query = AdvancedQuery {
            free_text: String::new(),
            predicates: vec![Predicate::Negate(Box::new(Predicate::Type(
                ArtifactKind::Skill,
            )))],
            source_filters: Default::default(),
            raw: ":-type:skill".to_string(),
        };
        assert_eq!(apply_advanced_query(results, &query).len(), 1);
    }

    #[test]
    fn source_and_plugin_predicates_are_not_aliases() {
        let s = {
            let mut s = make_script_with_extra("foo", HashMap::new());
            let mut_script = Arc::make_mut(&mut s);
            mut_script.kit_name = Some("my-kenv".to_string());
            mut_script.plugin_id = "core".to_string();
            mut_script.plugin_title = Some("Core".to_string());
            s
        };
        let results = vec![script_match(s)];

        let source_match = AdvancedQuery {
            free_text: String::new(),
            predicates: vec![Predicate::Source("my-kenv".to_string())],
            source_filters: Default::default(),
            raw: ":source:my-kenv".to_string(),
        };
        assert_eq!(
            apply_advanced_query(results.clone(), &source_match).len(),
            1
        );

        let plugin_misses_kit = AdvancedQuery {
            free_text: String::new(),
            predicates: vec![Predicate::Plugin("my-kenv".to_string())],
            source_filters: Default::default(),
            raw: ":plugin:my-kenv".to_string(),
        };
        assert_eq!(
            apply_advanced_query(results.clone(), &plugin_misses_kit).len(),
            0
        );

        let plugin_matches_id = AdvancedQuery {
            free_text: String::new(),
            predicates: vec![Predicate::Plugin("core".to_string())],
            source_filters: Default::default(),
            raw: ":plugin:core".to_string(),
        };
        assert_eq!(apply_advanced_query(results, &plugin_matches_id).len(), 1);
    }

    #[test]
    fn has_field_matches_extra_keys_case_insensitively() {
        let mut extra = HashMap::new();
        extra.insert("fooBar".to_string(), json!("value"));
        let s = make_script_with_extra("foo", extra);
        let results = vec![script_match(s)];

        for probe in ["fooBar", "foobar", "FOOBAR", "Foobar"] {
            let query = AdvancedQuery {
                free_text: String::new(),
                predicates: vec![Predicate::Has(probe.to_string())],
                source_filters: Default::default(),
                raw: format!(":has:{probe}"),
            };
            assert_eq!(
                apply_advanced_query(results.clone(), &query).len(),
                1,
                "expected has:{probe} to match case-insensitively"
            );
        }
    }

    #[test]
    fn meta_path_is_case_insensitive_through_nested_objects() {
        let mut extra = HashMap::new();
        extra.insert("Domain".to_string(), json!({ "Kind": "Calendar" }));
        let s = make_script_with_extra("cal", extra);
        let results = vec![script_match(s)];

        let query = AdvancedQuery {
            free_text: String::new(),
            predicates: vec![Predicate::MetaPath {
                path: "domain.kind".to_string(),
                value: "calendar".to_string(),
            }],
            source_filters: Default::default(),
            raw: ":meta.domain.kind:calendar".to_string(),
        };
        assert_eq!(apply_advanced_query(results, &query).len(), 1);
    }

    #[test]
    fn scripts_handling_capture_filters_to_capture_opted_in() {
        let mut todo_extra = HashMap::new();
        todo_extra.insert(
            "menuSyntax".to_string(),
            json!([{ "family": "capture.v1", "targets": ["todo"] }]),
        );
        let todo_script = make_script_with_extra("Add Todo", todo_extra);

        let mut cal_extra = HashMap::new();
        cal_extra.insert(
            "menuSyntax".to_string(),
            json!([{ "family": "capture.v1", "targets": ["cal"] }]),
        );
        let cal_script = make_script_with_extra("Calendar", cal_extra);

        let plain_script = make_script_with_extra("Plain", HashMap::new());

        let scripts = vec![todo_script.clone(), cal_script.clone(), plain_script];
        let invocation = CaptureInvocation {
            target: "todo".to_string(),
            alias_form: crate::menu_syntax::payload::CaptureAlias::CapturePrefix,
            body: "x".to_string(),
            tags: vec![],
            priority: None,
            url: None,
            duration: None,
            kv: vec![],
            date_phrases: vec![],
            raw: ";todo x".to_string(),
        };

        let handlers = scripts_handling_capture(&scripts, &invocation);
        assert_eq!(handlers.len(), 1);
        assert_eq!(handlers[0].name, "Add Todo");
    }

    #[test]
    fn registered_capture_targets_from_scripts_collects_custom_targets() {
        let mut extra = HashMap::new();
        extra.insert(
            "menuSyntax".to_string(),
            json!([
                { "family": "capture.v1", "targets": ["github", "*"] },
                { "family": "query.v1", "targets": ["ignored"] }
            ]),
        );
        let script = make_script_with_extra("GitHub", extra);

        let targets = registered_capture_targets_from_scripts(&[script]);
        assert_eq!(targets, vec!["github".to_string()]);
    }

    #[test]
    fn tag_predicate_matches_script_metadata_tags() {
        let mut meta = TypedMetadata::default();
        meta.tags = vec!["script-kit".to_string(), "work".to_string()];
        let script = Arc::new(Script {
            name: "Tagged".to_string(),
            path: PathBuf::from("/tmp/tagged.ts"),
            extension: "ts".to_string(),
            typed_metadata: Some(meta),
            plugin_id: "main".to_string(),
            ..Default::default()
        });
        let results = vec![script_match(script)];

        let query = AdvancedQuery {
            free_text: String::new(),
            predicates: vec![Predicate::Tag("script-kit".to_string())],
            source_filters: Default::default(),
            raw: ":#script-kit".to_string(),
        };
        assert_eq!(apply_advanced_query(results.clone(), &query).len(), 1);

        let miss = AdvancedQuery {
            free_text: String::new(),
            predicates: vec![Predicate::Tag("personal".to_string())],
            source_filters: Default::default(),
            raw: ":#personal".to_string(),
        };
        assert_eq!(apply_advanced_query(results, &miss).len(), 0);
    }
}
