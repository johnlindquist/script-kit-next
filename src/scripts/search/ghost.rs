use crate::scripts::types::SearchResult;

const AGENT_CHAT_HINT_SUFFIX: &str = " ⌘↵ Ask Agent Chat";
const AGENT_CHAT_HINT_CONFIDENCE: f32 = 0.55;
const AGENT_PROMPT_CONFIDENCE: f32 = 0.62;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PredictionRevision {
    pub query_rev: u64,
    pub catalog_rev: u64,
    pub context_rev: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GhostPredictionKind {
    CommandCompletion,
    AgentPromptCompletion,
    AgentChatHint,
}

#[derive(Clone, Debug)]
pub struct GhostPrediction {
    pub query: String,
    pub full_label: String,
    pub ghost_suffix: String,
    pub confidence: f32,
    pub revision: PredictionRevision,
    pub ghost_id: u64,
    pub kind: GhostPredictionKind,
}

impl GhostPrediction {
    pub fn accepts_tab(&self) -> bool {
        matches!(
            self.kind,
            GhostPredictionKind::CommandCompletion | GhostPredictionKind::AgentPromptCompletion
        )
    }

    pub fn kind_label(&self) -> &'static str {
        match self.kind {
            GhostPredictionKind::CommandCompletion => "command_completion",
            GhostPredictionKind::AgentPromptCompletion => "agent_prompt_completion",
            GhostPredictionKind::AgentChatHint => "agent_chat_hint",
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GhostContext {
    pub cwd_label: Option<String>,
    pub has_agents_md: bool,
    pub has_readme_md: bool,
}

impl GhostContext {
    pub fn from_cwd(cwd: &std::path::Path) -> Self {
        let cwd_label = cwd
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.to_string())
            .or_else(|| Some(cwd.display().to_string()));
        Self {
            cwd_label,
            has_agents_md: cwd.join("AGENTS.md").is_file(),
            has_readme_md: cwd.join("README.md").is_file(),
        }
    }
}

static GHOST_ID_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

fn next_ghost_id() -> u64 {
    GHOST_ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

pub fn compute_ghost_prediction(
    query: &str,
    flat_results: &[SearchResult],
) -> Option<GhostPrediction> {
    compute_ghost_prediction_with_revision(query, flat_results, PredictionRevision::default())
}

pub fn compute_ghost_prediction_with_revision(
    query: &str,
    flat_results: &[SearchResult],
    revision: PredictionRevision,
) -> Option<GhostPrediction> {
    compute_ghost_prediction_with_context(query, flat_results, revision, &GhostContext::default())
}

pub fn compute_ghost_prediction_with_context(
    query: &str,
    flat_results: &[SearchResult],
    revision: PredictionRevision,
    context: &GhostContext,
) -> Option<GhostPrediction> {
    if query.len() < 2 || query.ends_with(' ') {
        return None;
    }

    command_completion_prediction(query, flat_results, revision)
        .or_else(|| agent_prompt_completion_prediction(query, flat_results, revision, context))
        .or_else(|| agent_chat_hint_prediction(query, flat_results, revision))
}

fn command_completion_prediction(
    query: &str,
    flat_results: &[SearchResult],
    revision: PredictionRevision,
) -> Option<GhostPrediction> {
    let eligible: Vec<&SearchResult> = flat_results
        .iter()
        .filter(|r| is_eligible_for_ghost(r))
        .take(10)
        .collect();

    if eligible.is_empty() {
        return None;
    }

    let top = eligible[0];
    let label = top.name();

    let suffix = suffix_for_prefix(query, label)?;

    if suffix.is_empty() {
        return None;
    }

    let top_score = top.score();
    let second_score = eligible.get(1).map(|r| r.score()).unwrap_or(0);
    let top_tier = top.match_tier();

    if !dominant_enough(top_score, second_score, top_tier) {
        return None;
    }

    let gap = if second_score > 0 {
        (top_score - second_score) as f32 / top_score.max(1) as f32
    } else {
        1.0
    };

    Some(GhostPrediction {
        query: query.to_string(),
        full_label: label.to_string(),
        ghost_suffix: suffix,
        confidence: gap.clamp(0.0, 1.0),
        revision,
        ghost_id: next_ghost_id(),
        kind: GhostPredictionKind::CommandCompletion,
    })
}

fn agent_chat_hint_prediction(
    query: &str,
    flat_results: &[SearchResult],
    revision: PredictionRevision,
) -> Option<GhostPrediction> {
    let trimmed = query.trim();
    if !is_probably_natural_language_agent_query(trimmed) {
        return None;
    }
    if !has_send_to_ai_fallback(flat_results) {
        return None;
    }

    Some(GhostPrediction {
        query: query.to_string(),
        full_label: format!("{query}{AGENT_CHAT_HINT_SUFFIX}"),
        ghost_suffix: AGENT_CHAT_HINT_SUFFIX.to_string(),
        confidence: AGENT_CHAT_HINT_CONFIDENCE,
        revision,
        ghost_id: next_ghost_id(),
        kind: GhostPredictionKind::AgentChatHint,
    })
}

fn agent_prompt_completion_prediction(
    query: &str,
    flat_results: &[SearchResult],
    revision: PredictionRevision,
    context: &GhostContext,
) -> Option<GhostPrediction> {
    let trimmed = query.trim();
    if !has_send_to_ai_fallback(flat_results) || !is_safe_agent_prompt_seed(trimmed) {
        return None;
    }

    let completion = prompt_completion_for_seed(trimmed, context)?;
    let suffix = suffix_for_prefix(trimmed, &completion)?;
    if suffix.is_empty() {
        return None;
    }

    Some(GhostPrediction {
        query: query.to_string(),
        full_label: completion,
        ghost_suffix: suffix,
        confidence: AGENT_PROMPT_CONFIDENCE,
        revision,
        ghost_id: next_ghost_id(),
        kind: GhostPredictionKind::AgentPromptCompletion,
    })
}

fn suffix_for_prefix(query: &str, label: &str) -> Option<String> {
    let q_lower = query.to_lowercase();
    let l_lower = label.to_lowercase();
    if l_lower.starts_with(&q_lower) {
        Some(label[query.len()..].to_string())
    } else {
        None
    }
}

fn has_send_to_ai_fallback(flat_results: &[SearchResult]) -> bool {
    flat_results.iter().any(|result| {
        matches!(
            result,
            SearchResult::Fallback(fallback_match)
                if matches!(
                    &fallback_match.fallback,
                    crate::fallbacks::FallbackItem::Builtin(builtin)
                        if builtin.id == crate::fallbacks::builtins::SEND_TO_AI_FALLBACK_ID
                )
        )
    })
}

fn is_probably_natural_language_agent_query(query: &str) -> bool {
    if query.len() < 8 {
        return false;
    }
    if query.starts_with([':', '+', '@', '/', '~', '!', '#', '>', '|']) {
        return false;
    }
    if crate::scripts::input_detection::is_url(query)
        || crate::scripts::input_detection::is_math_expression(query)
        || crate::scripts::input_detection::is_file_path(query)
    {
        return false;
    }

    let word_count = query.split_whitespace().count();
    let first_word = query
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .trim_matches(|ch: char| !ch.is_alphanumeric())
        .to_ascii_lowercase();
    let starts_like_question_or_prompt = matches!(
        first_word.as_str(),
        "who"
            | "what"
            | "when"
            | "where"
            | "why"
            | "how"
            | "can"
            | "could"
            | "should"
            | "would"
            | "write"
            | "make"
            | "create"
            | "explain"
            | "summarize"
            | "translate"
            | "compare"
            | "tell"
            | "find"
    );

    query.ends_with('?') || (starts_like_question_or_prompt && word_count >= 2) || word_count >= 4
}

fn is_safe_agent_prompt_seed(query: &str) -> bool {
    if query.len() < 2 || query.len() > 48 {
        return false;
    }
    if query.starts_with([':', '+', '@', '/', '~', '!', '#', '>', '|']) {
        return false;
    }
    if crate::scripts::input_detection::is_url(query)
        || crate::scripts::input_detection::is_math_expression(query)
        || crate::scripts::input_detection::is_file_path(query)
    {
        return false;
    }
    query
        .chars()
        .all(|ch| ch.is_alphanumeric() || ch.is_whitespace() || matches!(ch, '-' | '_' | '?'))
}

fn prompt_completion_for_seed(query: &str, context: &GhostContext) -> Option<String> {
    let lower = query.to_ascii_lowercase();
    let words = lower.split_whitespace().collect::<Vec<_>>();
    let first = words.first().copied().unwrap_or_default();

    let base = match first {
        "fix" => Some("fix the issue in this project"),
        "debug" => Some("debug this issue in this project"),
        "test" => Some("test this change"),
        "review" => Some("review the current changes"),
        "explain" => Some("explain this project"),
        "summarize" => Some("summarize this project"),
        "write" => Some("write a Script Kit script"),
        "create" => Some("create a Script Kit script"),
        "make" => Some("make a Script Kit script"),
        "find" => Some("find the relevant code"),
        "where" => Some("where is this implemented"),
        "how" => Some("how does this work"),
        "why" => Some("why is this happening"),
        "what" => Some("what should I change next"),
        _ => None,
    }?;

    if !base.starts_with(&lower) {
        return None;
    }

    let doc_suffix = context_doc_suffix(context);
    Some(format!("{base}{doc_suffix}"))
}

fn context_doc_suffix(context: &GhostContext) -> &'static str {
    match (context.has_agents_md, context.has_readme_md) {
        (true, true) => " using AGENTS.md and README.md",
        (true, false) => " using AGENTS.md",
        (false, true) => " using README.md",
        (false, false) => "",
    }
}

fn dominant_enough(top_score: i32, second_score: i32, top_tier: i32) -> bool {
    if top_tier < 850 {
        return false;
    }
    let gap = top_score - second_score;
    gap > 200 || second_score == 0
}

fn is_eligible_for_ghost(result: &SearchResult) -> bool {
    !matches!(
        result,
        SearchResult::Fallback(_)
            | SearchResult::File(_)
            | SearchResult::Note(_)
            | SearchResult::Todo(_)
            | SearchResult::ClipboardHistory(_)
            | SearchResult::DictationHistory(_)
            | SearchResult::BrowserHistory(_)
            | SearchResult::BrowserTab(_)
            | SearchResult::ScriptIssue(_)
            | SearchResult::SpineProjection(_)
            | SearchResult::Agent(_)
    )
}

pub fn reconcile_typed_through(
    old_query: &str,
    new_query: &str,
    prediction: &GhostPrediction,
) -> Option<GhostPrediction> {
    if !prediction.accepts_tab() {
        return None;
    }
    if !new_query.starts_with(old_query) {
        return None;
    }
    let added = &new_query[old_query.len()..];
    let suffix_lower = prediction.ghost_suffix.to_lowercase();
    let added_lower = added.to_lowercase();
    if suffix_lower.starts_with(&added_lower) && added_lower.len() < suffix_lower.len() {
        let new_suffix = &prediction.ghost_suffix[added.len()..];
        Some(GhostPrediction {
            query: new_query.to_string(),
            full_label: prediction.full_label.clone(),
            ghost_suffix: new_suffix.to_string(),
            confidence: prediction.confidence,
            revision: prediction.revision,
            ghost_id: next_ghost_id(),
            kind: prediction.kind,
        })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suffix_extraction() {
        assert_eq!(
            suffix_for_prefix("clip", "Clipboard History"),
            Some("board History".to_string())
        );
        assert_eq!(
            suffix_for_prefix("Clip", "Clipboard History"),
            Some("board History".to_string())
        );
        assert_eq!(suffix_for_prefix("xyz", "Clipboard History"), None);
    }

    #[test]
    fn no_ghost_for_short_query() {
        assert!(compute_ghost_prediction("", &[]).is_none());
        assert!(compute_ghost_prediction("a", &[]).is_none());
    }

    #[test]
    fn no_ghost_for_trailing_space() {
        assert!(compute_ghost_prediction("clip ", &[]).is_none());
    }

    fn test_prediction(query: &str, label: &str, suffix: &str) -> GhostPrediction {
        GhostPrediction {
            query: query.to_string(),
            full_label: label.to_string(),
            ghost_suffix: suffix.to_string(),
            confidence: 0.8,
            revision: PredictionRevision::default(),
            ghost_id: 0,
            kind: GhostPredictionKind::CommandCompletion,
        }
    }

    #[test]
    fn typed_through_advances() {
        let pred = test_prediction("cli", "Clipboard History", "pboard History");
        let result = reconcile_typed_through("cli", "clip", &pred);
        assert!(result.is_some());
        let r = result.unwrap();
        assert_eq!(r.ghost_suffix, "board History");
        assert_eq!(r.query, "clip");
    }

    fn make_builtin_result(name: &str, score: i32) -> SearchResult {
        SearchResult::BuiltIn(crate::scripts::types::BuiltInMatch {
            entry: crate::builtins::BuiltInEntry {
                id: name.to_lowercase().replace(' ', "-"),
                name: name.to_string(),
                description: format!("Open {name}"),
                keywords: vec![],
                feature: crate::builtins::BuiltInFeature::ClipboardHistory,
                icon: None,
                group: crate::builtins::BuiltInGroup::Core,
            },
            score,
            match_evidence: None,
        })
    }

    #[test]
    fn ghost_prediction_with_dominant_prefix_match() {
        let results = vec![
            make_builtin_result("Clipboard History", 950_200),
            make_builtin_result("Clear Cache", 850_100),
        ];
        let pred = compute_ghost_prediction("cli", &results);
        assert!(
            pred.is_some(),
            "should produce ghost for dominant prefix match"
        );
        let p = pred.unwrap();
        assert_eq!(p.ghost_suffix, "pboard History");
        assert_eq!(p.full_label, "Clipboard History");
        assert!(p.confidence > 0.0);
    }

    #[test]
    fn no_ghost_when_no_prefix_match() {
        let results = vec![
            make_builtin_result("Process Manager", 950_200),
            make_builtin_result("Settings", 850_100),
        ];
        let pred = compute_ghost_prediction("cli", &results);
        assert!(
            pred.is_none(),
            "should not ghost when top result doesn't prefix-match"
        );
    }

    #[test]
    fn no_ghost_for_close_scores() {
        let results = vec![
            make_builtin_result("Clipboard History", 950_200),
            make_builtin_result("CLI Tools", 950_100),
        ];
        let pred = compute_ghost_prediction("cli", &results);
        assert!(
            pred.is_none(),
            "should not ghost when scores are too close (gap < 200)"
        );
    }

    #[test]
    fn ghost_only_for_eligible_result_types() {
        // Verify that eligible results produce ghost predictions
        let eligible = make_builtin_result("Settings", 950_500);
        assert!(
            is_eligible_for_ghost(&eligible),
            "BuiltIn should be eligible"
        );

        // Create a result with an eligible type but test edge case: single result, no competition
        let results = vec![make_builtin_result("Settings", 950_500)];
        let pred = compute_ghost_prediction("se", &results);
        assert!(
            pred.is_some(),
            "single dominant result should produce ghost"
        );
        assert_eq!(pred.unwrap().ghost_suffix, "ttings");
    }

    fn make_send_to_ai_fallback_result() -> SearchResult {
        let fallback = crate::fallbacks::builtins::get_builtin_fallbacks()
            .into_iter()
            .find(|fallback| fallback.id == crate::fallbacks::builtins::SEND_TO_AI_FALLBACK_ID)
            .expect("send-to-ai fallback should exist");
        SearchResult::Fallback(crate::scripts::types::FallbackMatch::new(
            crate::fallbacks::FallbackItem::Builtin(fallback),
            0,
        ))
    }

    #[test]
    fn natural_language_query_gets_agent_chat_hint() {
        let results = vec![make_send_to_ai_fallback_result()];
        let pred = compute_ghost_prediction("Who is the fastest man in the world?", &results)
            .expect("natural-language query should produce Agent Chat ghost hint");
        assert_eq!(pred.query, "Who is the fastest man in the world?");
        assert!(pred.ghost_suffix.contains("⌘↵"));
        assert!(pred.ghost_suffix.contains("Agent Chat"));
        assert!(pred
            .full_label
            .starts_with("Who is the fastest man in the world?"));
        assert_eq!(pred.kind, GhostPredictionKind::AgentChatHint);
        assert!(!pred.accepts_tab());
    }

    #[test]
    fn basic_agent_query_gets_prompt_completion() {
        let results = vec![make_send_to_ai_fallback_result()];
        let pred = compute_ghost_prediction("fix", &results)
            .expect("basic agent seed should produce prompt completion");
        assert_eq!(pred.full_label, "fix the issue in this project");
        assert_eq!(pred.ghost_suffix, " the issue in this project");
        assert_eq!(pred.kind, GhostPredictionKind::AgentPromptCompletion);
        assert!(pred.accepts_tab());
    }

    #[test]
    fn prompt_completion_uses_cwd_docs_when_present() {
        let mut dir = std::env::temp_dir();
        dir.push(format!(
            "script-kit-ghost-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time should be after epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).expect("create temp ghost test dir");
        std::fs::write(dir.join("AGENTS.md"), "agent instructions").expect("write AGENTS.md");
        std::fs::write(dir.join("README.md"), "project readme").expect("write README.md");

        let context = GhostContext::from_cwd(&dir);
        let results = vec![make_send_to_ai_fallback_result()];
        let pred = compute_ghost_prediction_with_context(
            "explain",
            &results,
            PredictionRevision::default(),
            &context,
        )
        .expect("cwd docs should enrich basic agent prompt completion");

        assert_eq!(
            pred.full_label,
            "explain this project using AGENTS.md and README.md"
        );
        assert_eq!(
            pred.ghost_suffix,
            " this project using AGENTS.md and README.md"
        );
        assert_eq!(pred.kind, GhostPredictionKind::AgentPromptCompletion);
        assert!(pred.accepts_tab());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn typed_through_advances_agent_prompt_completion() {
        let pred = GhostPrediction {
            query: "fix".to_string(),
            full_label: "fix the issue in this project".to_string(),
            ghost_suffix: " the issue in this project".to_string(),
            confidence: AGENT_PROMPT_CONFIDENCE,
            revision: PredictionRevision::default(),
            ghost_id: 0,
            kind: GhostPredictionKind::AgentPromptCompletion,
        };
        let result = reconcile_typed_through("fix", "fix the", &pred)
            .expect("typed-through prompt completion should reconcile like command completion");
        assert_eq!(result.ghost_suffix, " issue in this project");
        assert_eq!(result.kind, GhostPredictionKind::AgentPromptCompletion);
    }

    #[test]
    fn command_completion_wins_over_agent_chat_hint() {
        let results = vec![
            make_builtin_result("Clipboard History", 950_200),
            make_send_to_ai_fallback_result(),
        ];
        let pred = compute_ghost_prediction("cli", &results)
            .expect("dominant command prefix should still produce command completion");
        assert_eq!(pred.full_label, "Clipboard History");
        assert_eq!(pred.ghost_suffix, "pboard History");
        assert_eq!(pred.kind, GhostPredictionKind::CommandCompletion);
        assert!(pred.accepts_tab());
    }

    #[test]
    fn single_word_command_like_query_does_not_get_agent_chat_hint() {
        let results = vec![make_send_to_ai_fallback_result()];
        assert!(
            compute_ghost_prediction("quit", &results).is_none(),
            "single-word command-like input should not get decorative Agent Chat ghost text"
        );
    }

    #[test]
    fn natural_language_hint_requires_send_to_ai_fallback() {
        let results = vec![make_builtin_result("Process Manager", 950_200)];
        assert!(
            compute_ghost_prediction("Who is the fastest man in the world?", &results).is_none(),
            "Agent Chat hint should only appear when send-to-ai fallback is actually available"
        );
    }

    #[test]
    fn typed_through_does_not_reconcile_agent_chat_hint() {
        let pred = GhostPrediction {
            query: "Who is the fastest man in the world?".to_string(),
            full_label: "Who is the fastest man in the world? ⌘↵ Ask Agent Chat".to_string(),
            ghost_suffix: " ⌘↵ Ask Agent Chat".to_string(),
            confidence: AGENT_CHAT_HINT_CONFIDENCE,
            revision: PredictionRevision::default(),
            ghost_id: 0,
            kind: GhostPredictionKind::AgentChatHint,
        };
        assert!(reconcile_typed_through(
            "Who is the fastest man in the world?",
            "Who is the fastest man in the world? ",
            &pred,
        )
        .is_none());
    }

    #[test]
    fn ghost_serializes_in_state() {
        let pred = test_prediction("cli", "Clipboard History", "pboard History");
        let json = serde_json::json!({
            "query": pred.query,
            "fullLabel": pred.full_label,
            "ghostSuffix": pred.ghost_suffix,
            "confidence": pred.confidence,
        });
        assert_eq!(json["ghostSuffix"], "pboard History");
        assert_eq!(json["fullLabel"], "Clipboard History");
    }

    #[test]
    fn typed_through_rejects_mismatch() {
        let pred = test_prediction("cli", "Clipboard History", "pboard History");
        assert!(reconcile_typed_through("cli", "clx", &pred).is_none());
    }

    #[test]
    fn revision_stale_detection() {
        let rev1 = PredictionRevision {
            query_rev: 1,
            catalog_rev: 1,
            context_rev: 1,
        };
        let rev2 = PredictionRevision {
            query_rev: 2,
            catalog_rev: 1,
            context_rev: 1,
        };
        assert_ne!(rev1, rev2, "different query_rev should be stale");
        assert_eq!(rev1, rev1, "same revision should match");
    }

    #[test]
    fn ghost_ids_are_unique() {
        let results = vec![make_builtin_result("Settings", 950_500)];
        let p1 = compute_ghost_prediction("se", &results).unwrap();
        let p2 = compute_ghost_prediction("se", &results).unwrap();
        assert_ne!(
            p1.ghost_id, p2.ghost_id,
            "each prediction gets a unique ghost_id"
        );
    }
}
