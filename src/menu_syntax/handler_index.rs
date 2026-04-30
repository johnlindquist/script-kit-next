use std::sync::Arc;

use crate::scripts::Script;

use super::filter::script_menu_syntax_specs;
use super::payload::{CaptureInvocation, MenuSyntaxHandlerSpec};

/// One script paired with the specific `menuSyntax` handler spec that caused it
/// to match, plus the ranking score the sort used. Sorting is stable by the
/// tuple (`HandlerScore`, script name ascending) so ties fall back to
/// alphabetical name order for deterministic rendering in snapshots and tests.
#[derive(Debug, Clone)]
pub struct RankedHandler {
    pub script: Arc<Script>,
    pub spec: MenuSyntaxHandlerSpec,
    pub score: HandlerScore,
}

/// Per-handler priority tuple. Higher values sort FIRST. The tuple is
/// lexicographic: exact-target > default_handler > user-authored >
/// accepts-boost. `accepts_boost` is small by design (max `MAX_ACCEPTS_BOOST`)
/// so it only breaks ties inside a priority bucket, never crosses buckets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HandlerScore {
    pub exact_target: u8,
    pub default_handler: u8,
    pub user_authored: u8,
    pub accepts_boost: u8,
}

/// Cap `accepts_boost` at 3 so even a handler that accepts `["date","url",
/// "tag","priority","duration"]` cannot beat the next priority bucket. Keeps
/// the ranking readable: buckets first, boosts only break within-bucket ties.
const MAX_ACCEPTS_BOOST: u8 = 3;

/// Ordered list of known `accepts` tokens we recognize inside a
/// [`CaptureInvocation`]. Any other token the handler declares simply gets no
/// boost — the classifier is intentionally permissive, not a parser.
const KNOWN_ACCEPTS: &[&str] = &["date", "url", "tag", "tags", "priority", "duration", "kv"];

/// Rank all handlers in `scripts` that opt into `capture.v1` and either match
/// the invocation's target exactly or declare a wildcard `*`. Returns rows
/// sorted by [`HandlerScore`] descending, then by script name ascending for a
/// deterministic tie-break.
///
/// A single script may appear more than once if it declares multiple specs for
/// the same target (e.g. one exact + one wildcard). Callers that only want one
/// row per script should dedupe by `script.path` after calling this.
pub fn rank_handlers_for_target(
    scripts: &[Arc<Script>],
    invocation: &CaptureInvocation,
) -> Vec<RankedHandler> {
    let mut ranked: Vec<RankedHandler> = Vec::new();

    for script in scripts {
        for spec in script_menu_syntax_specs(script) {
            if spec.family != "capture.v1" {
                continue;
            }
            let Some(score) = score_spec(&spec, script, invocation) else {
                continue;
            };
            ranked.push(RankedHandler {
                script: script.clone(),
                spec,
                score,
            });
        }
    }

    ranked.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then_with(|| a.script.name.cmp(&b.script.name))
    });
    ranked
}

/// Convenience: like [`rank_handlers_for_target`] but returns only the script
/// references in priority order. Used by surfaces that already dedupe on
/// script identity (e.g. `build_capture_mode_results`).
pub fn rank_scripts_handling_capture(
    scripts: &[Arc<Script>],
    invocation: &CaptureInvocation,
) -> Vec<Arc<Script>> {
    let ranked = rank_handlers_for_target(scripts, invocation);
    let mut seen_paths: Vec<std::path::PathBuf> = Vec::with_capacity(ranked.len());
    let mut out: Vec<Arc<Script>> = Vec::with_capacity(ranked.len());
    for entry in ranked {
        let path = entry.script.path.clone();
        if seen_paths.iter().any(|p| p == &path) {
            continue;
        }
        seen_paths.push(path);
        out.push(entry.script);
    }
    out
}

fn score_spec(
    spec: &MenuSyntaxHandlerSpec,
    script: &Script,
    invocation: &CaptureInvocation,
) -> Option<HandlerScore> {
    let mut exact_target: u8 = 0;
    let mut wildcard = false;
    for target in &spec.targets {
        if target == "*" {
            wildcard = true;
        } else if target.eq_ignore_ascii_case(&invocation.target) {
            exact_target = 1;
            break;
        }
    }
    if exact_target == 0 && !wildcard {
        return None;
    }

    let default_handler = if spec.default_handler { 1 } else { 0 };
    let user_authored = if script_is_user_authored(script) {
        1
    } else {
        0
    };
    let accepts_boost = accepts_boost_for(spec, invocation);

    Some(HandlerScore {
        exact_target,
        default_handler,
        user_authored,
        accepts_boost,
    })
}

/// A script is "user-authored" when it lives in a plugin other than the
/// shipped `main` plugin. Shipped capture examples under
/// `scripts/examples/menu-syntax/` load with `plugin_id == "main"`, so they
/// sort below a user's own handlers for the same target.
fn script_is_user_authored(script: &Script) -> bool {
    !script.plugin_id.eq_ignore_ascii_case("main")
}

fn accepts_boost_for(spec: &MenuSyntaxHandlerSpec, invocation: &CaptureInvocation) -> u8 {
    if spec.accepts.is_empty() {
        return 0;
    }
    let mut boost: u8 = 0;
    for accept in &spec.accepts {
        let lc = accept.to_ascii_lowercase();
        if !KNOWN_ACCEPTS.iter().any(|k| *k == lc) {
            continue;
        }
        if invocation_has(&lc, invocation) {
            boost = boost.saturating_add(1);
            if boost >= MAX_ACCEPTS_BOOST {
                return MAX_ACCEPTS_BOOST;
            }
        }
    }
    boost
}

fn invocation_has(accept: &str, invocation: &CaptureInvocation) -> bool {
    match accept {
        "date" => !invocation.date_phrases.is_empty(),
        "url" => invocation.url.is_some(),
        "tag" | "tags" => !invocation.tags.is_empty(),
        "priority" => invocation.priority.is_some(),
        "duration" => invocation.duration.is_some(),
        "kv" => !invocation.kv.is_empty(),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menu_syntax::payload::CaptureAlias;
    use crate::metadata_parser::TypedMetadata;
    use serde_json::json;
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn invocation(target: &str, body: &str) -> CaptureInvocation {
        CaptureInvocation {
            target: target.to_string(),
            alias_form: CaptureAlias::Plus,
            body: body.to_string(),
            tags: Vec::new(),
            priority: None,
            url: None,
            duration: None,
            kv: Vec::new(),
            date_phrases: Vec::new(),
            raw: format!("+{target} {body}"),
        }
    }

    fn script_with_menu_syntax(
        name: &str,
        plugin_id: &str,
        spec_json: serde_json::Value,
    ) -> Arc<Script> {
        let mut extra: HashMap<String, serde_json::Value> = HashMap::new();
        extra.insert("menuSyntax".to_string(), spec_json);
        let mut meta = TypedMetadata::default();
        meta.extra = extra;
        Arc::new(Script {
            name: name.to_string(),
            path: PathBuf::from(format!("/tmp/{name}.ts")),
            extension: "ts".to_string(),
            description: None,
            icon: None,
            alias: None,
            shortcut: None,
            typed_metadata: Some(meta),
            schema: None,
            plugin_id: plugin_id.to_string(),
            plugin_title: None,
            kit_name: None,
            body: None,
        })
    }

    #[test]
    fn empty_catalog_returns_empty_ranking() {
        let inv = invocation("todo", "buy milk");
        assert!(rank_handlers_for_target(&[], &inv).is_empty());
    }

    #[test]
    fn scripts_without_menu_syntax_are_ignored() {
        let plain = Arc::new(Script {
            name: "plain".into(),
            path: PathBuf::from("/tmp/plain.ts"),
            extension: "ts".into(),
            description: None,
            icon: None,
            alias: None,
            shortcut: None,
            typed_metadata: None,
            schema: None,
            plugin_id: "main".into(),
            plugin_title: None,
            kit_name: None,
            body: None,
        });
        let inv = invocation("todo", "x");
        assert!(rank_handlers_for_target(&[plain], &inv).is_empty());
    }

    #[test]
    fn non_capture_family_is_ignored() {
        let script = script_with_menu_syntax(
            "query handler",
            "main",
            json!([{ "family": "query.v1", "targets": ["todo"] }]),
        );
        let inv = invocation("todo", "x");
        assert!(rank_handlers_for_target(&[script], &inv).is_empty());
    }

    #[test]
    fn exact_target_outranks_wildcard_same_plugin() {
        let exact = script_with_menu_syntax(
            "Exact Todo",
            "main",
            json!([{ "family": "capture.v1", "targets": ["todo"] }]),
        );
        let wild = script_with_menu_syntax(
            "Wildcard Handler",
            "main",
            json!([{ "family": "capture.v1", "targets": ["*"] }]),
        );
        let inv = invocation("todo", "x");
        let ranked = rank_handlers_for_target(&[wild.clone(), exact.clone()], &inv);
        assert_eq!(ranked.len(), 2);
        assert_eq!(ranked[0].script.name, "Exact Todo");
        assert_eq!(ranked[1].script.name, "Wildcard Handler");
    }

    #[test]
    fn default_handler_outranks_non_default_exact() {
        let default_h = script_with_menu_syntax(
            "Default Todo",
            "main",
            json!([{ "family": "capture.v1", "targets": ["todo"], "defaultHandler": true }]),
        );
        let plain_h = script_with_menu_syntax(
            "Plain Todo",
            "main",
            json!([{ "family": "capture.v1", "targets": ["todo"] }]),
        );
        let inv = invocation("todo", "x");
        let ranked = rank_handlers_for_target(&[plain_h, default_h.clone()], &inv);
        assert_eq!(ranked[0].script.name, "Default Todo");
    }

    #[test]
    fn user_plugin_outranks_shipped_main_when_neither_is_default() {
        let user = script_with_menu_syntax(
            "User Todo",
            "my-plugin",
            json!([{ "family": "capture.v1", "targets": ["todo"] }]),
        );
        let shipped = script_with_menu_syntax(
            "Shipped Todo",
            "main",
            json!([{ "family": "capture.v1", "targets": ["todo"] }]),
        );
        let inv = invocation("todo", "x");
        let ranked = rank_handlers_for_target(&[shipped, user.clone()], &inv);
        assert_eq!(ranked[0].script.name, "User Todo");
    }

    #[test]
    fn shipped_default_still_beats_user_non_default() {
        let user = script_with_menu_syntax(
            "User Todo",
            "my-plugin",
            json!([{ "family": "capture.v1", "targets": ["todo"] }]),
        );
        let shipped_default = script_with_menu_syntax(
            "Shipped Default Todo",
            "main",
            json!([{ "family": "capture.v1", "targets": ["todo"], "defaultHandler": true }]),
        );
        let inv = invocation("todo", "x");
        let ranked = rank_handlers_for_target(&[user, shipped_default.clone()], &inv);
        assert_eq!(ranked[0].script.name, "Shipped Default Todo");
    }

    #[test]
    fn accepts_boost_breaks_tie_within_bucket() {
        let with_accepts = script_with_menu_syntax(
            "Accepts Url",
            "my-plugin",
            json!([{ "family": "capture.v1", "targets": ["link"], "accepts": ["url"] }]),
        );
        let plain = script_with_menu_syntax(
            "Plain Link",
            "my-plugin",
            json!([{ "family": "capture.v1", "targets": ["link"] }]),
        );
        let mut inv = invocation("link", "https://zed.dev");
        inv.url = Some("https://zed.dev".into());
        let ranked = rank_handlers_for_target(&[plain, with_accepts.clone()], &inv);
        assert_eq!(ranked[0].script.name, "Accepts Url");
    }

    #[test]
    fn accepts_boost_does_not_cross_priority_buckets() {
        // Even with every possible accepts match, a wildcard user handler must
        // not outrank an exact-target shipped handler.
        let shipped_exact = script_with_menu_syntax(
            "Shipped Exact Todo",
            "main",
            json!([{ "family": "capture.v1", "targets": ["todo"] }]),
        );
        let user_wildcard_with_accepts = script_with_menu_syntax(
            "User Wildcard Handler",
            "my-plugin",
            json!([{
                "family": "capture.v1",
                "targets": ["*"],
                "accepts": ["date","url","tag","priority","duration","kv"]
            }]),
        );
        let mut inv = invocation("todo", "buy milk tomorrow");
        inv.date_phrases.push(super::super::payload::DatePhrase {
            role: super::super::payload::DateRole::Due,
            source: "tomorrow".into(),
            source_span: (0, 8),
        });
        inv.url = Some("https://x".into());
        inv.tags.push("x".into());
        inv.priority = Some(1);
        inv.duration = Some("30m".into());
        inv.kv.push(("k".into(), "v".into()));
        let ranked =
            rank_handlers_for_target(&[user_wildcard_with_accepts, shipped_exact.clone()], &inv);
        assert_eq!(ranked[0].script.name, "Shipped Exact Todo");
    }

    #[test]
    fn accepts_boost_caps_at_maximum() {
        let spec = MenuSyntaxHandlerSpec {
            family: "capture.v1".into(),
            targets: vec!["todo".into()],
            accepts: vec![
                "date".into(),
                "url".into(),
                "tag".into(),
                "priority".into(),
                "duration".into(),
                "kv".into(),
            ],
            ..Default::default()
        };
        let mut inv = invocation("todo", "x");
        inv.date_phrases.push(super::super::payload::DatePhrase {
            role: super::super::payload::DateRole::Due,
            source: "tomorrow".into(),
            source_span: (0, 8),
        });
        inv.url = Some("x".into());
        inv.tags.push("x".into());
        inv.priority = Some(1);
        inv.duration = Some("30m".into());
        inv.kv.push(("k".into(), "v".into()));
        assert_eq!(accepts_boost_for(&spec, &inv), MAX_ACCEPTS_BOOST);
    }

    #[test]
    fn unknown_accepts_tokens_are_ignored() {
        let spec = MenuSyntaxHandlerSpec {
            family: "capture.v1".into(),
            targets: vec!["todo".into()],
            accepts: vec!["unsupported".into(), "date".into()],
            ..Default::default()
        };
        let mut inv = invocation("todo", "x");
        inv.date_phrases.push(super::super::payload::DatePhrase {
            role: super::super::payload::DateRole::Due,
            source: "tomorrow".into(),
            source_span: (0, 8),
        });
        assert_eq!(accepts_boost_for(&spec, &inv), 1);
    }

    #[test]
    fn name_alphabetical_tiebreak_is_stable() {
        let a = script_with_menu_syntax(
            "AAA Todo",
            "main",
            json!([{ "family": "capture.v1", "targets": ["todo"] }]),
        );
        let b = script_with_menu_syntax(
            "BBB Todo",
            "main",
            json!([{ "family": "capture.v1", "targets": ["todo"] }]),
        );
        let inv = invocation("todo", "x");
        let ranked = rank_handlers_for_target(&[b, a.clone()], &inv);
        assert_eq!(ranked[0].script.name, "AAA Todo");
        assert_eq!(ranked[1].script.name, "BBB Todo");
    }

    #[test]
    fn rank_scripts_handling_capture_dedupes_by_path() {
        let multi_spec = script_with_menu_syntax(
            "Todo Both",
            "main",
            json!([
                { "family": "capture.v1", "targets": ["todo"] },
                { "family": "capture.v1", "targets": ["*"] }
            ]),
        );
        let inv = invocation("todo", "x");
        let ranked = rank_handlers_for_target(&[multi_spec.clone()], &inv);
        assert_eq!(ranked.len(), 2, "both specs score independently");
        let flat = rank_scripts_handling_capture(&[multi_spec], &inv);
        assert_eq!(flat.len(), 1, "dedupes to one script");
        assert_eq!(flat[0].name, "Todo Both");
    }

    #[test]
    fn wildcard_only_matches_when_no_exact_target() {
        let wildcard_only = script_with_menu_syntax(
            "Wildcard Handler",
            "main",
            json!([{ "family": "capture.v1", "targets": ["*"] }]),
        );
        let inv_link = invocation("link", "https://zed.dev");
        let ranked = rank_handlers_for_target(&[wildcard_only.clone()], &inv_link);
        assert_eq!(ranked.len(), 1);
        assert_eq!(ranked[0].score.exact_target, 0);
    }

    #[test]
    fn case_insensitive_target_match() {
        let h = script_with_menu_syntax(
            "Mixed Case",
            "main",
            json!([{ "family": "capture.v1", "targets": ["ToDo"] }]),
        );
        let inv = invocation("todo", "x");
        let ranked = rank_handlers_for_target(&[h], &inv);
        assert_eq!(ranked.len(), 1);
        assert_eq!(ranked[0].score.exact_target, 1);
    }
}
