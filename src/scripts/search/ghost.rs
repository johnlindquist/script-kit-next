use crate::scripts::types::SearchResult;

#[derive(Clone, Debug)]
pub struct GhostPrediction {
    pub query: String,
    pub full_label: String,
    pub ghost_suffix: String,
    pub confidence: f32,
}

pub fn compute_ghost_prediction(
    query: &str,
    flat_results: &[SearchResult],
) -> Option<GhostPrediction> {
    if query.len() < 2 || query.ends_with(' ') {
        return None;
    }

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

    #[test]
    fn typed_through_advances() {
        let pred = GhostPrediction {
            query: "cli".to_string(),
            full_label: "Clipboard History".to_string(),
            ghost_suffix: "pboard History".to_string(),
            confidence: 0.8,
        };
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

    #[test]
    fn ghost_serializes_in_state() {
        let pred = GhostPrediction {
            query: "cli".to_string(),
            full_label: "Clipboard History".to_string(),
            ghost_suffix: "pboard History".to_string(),
            confidence: 0.85,
        };
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
        let pred = GhostPrediction {
            query: "cli".to_string(),
            full_label: "Clipboard History".to_string(),
            ghost_suffix: "pboard History".to_string(),
            confidence: 0.8,
        };
        assert!(reconcile_typed_through("cli", "clx", &pred).is_none());
    }
}
