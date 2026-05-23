use std::cmp::Ordering;
use std::ops::Range;
use std::sync::Arc;

use super::super::types::{MatchIndices, Script, ScriptContentMatch, ScriptMatch, ScriptMatchKind};
use super::{
    better_match, byte_range_for_char_indices, extract_filename, find_ignore_ascii_case,
    low_tier_substring_match, normalized_substring_match, primary_text_match, score_from_tier,
    NucleoCtx, TextMatch, TextMatchKind, MIN_BODY_EXACT_QUERY_LEN, TIER_ALIAS, TIER_BODY,
    TIER_DESCRIPTION, TIER_FILENAME, TIER_KEYWORD,
};

const SCORE_EXACT_NAME_MATCH: i32 = 500;
const SCORE_NAME_PREFIX: i32 = 100;
const SCORE_NAME_SUBSTRING: i32 = 75;
const SCORE_WORD_BOUNDARY: i32 = 20;
const SCORE_NAME_FUZZY_BASE: i32 = 50;
const SCORE_NAME_FUZZY_DIV: u32 = 20;
const SCORE_FILENAME_PREFIX: i32 = 60;
const SCORE_FILENAME_SUBSTRING: i32 = 45;
const SCORE_FILENAME_FUZZY_BASE: i32 = 35;
const SCORE_FILENAME_FUZZY_DIV: u32 = 30;
const SCORE_ALIAS_PREFIX: i32 = 80;
const SCORE_ALIAS_SUBSTRING: i32 = 60;
const SCORE_SHORTCUT_PREFIX: i32 = 80;
const SCORE_SHORTCUT_SUBSTRING: i32 = 60;
const SCORE_KIT_PREFIX: i32 = 30;
const SCORE_KIT_SUBSTRING: i32 = 20;
const SCORE_TAG_PREFIX: i32 = 40;
const SCORE_TAG_SUBSTRING: i32 = 25;
const SCORE_AUTHOR_PREFIX: i32 = 30;
const SCORE_AUTHOR_SUBSTRING: i32 = 20;
const SCORE_PROPERTY_KEYWORD: i32 = 35;
const SCORE_DESC_SUBSTRING: i32 = 25;
const SCORE_DESC_FUZZY_BASE: i32 = 15;
const SCORE_DESC_FUZZY_DIV: u32 = 30;
const SCORE_PATH_SUBSTRING: i32 = 10;
const SCORE_CONTENT_MATCH: i32 = 5;
const CURRENT_APP_RECIPE_HEADER_PREFIX: &str = "// Current-App-Recipe-";

fn is_legacy_ai_vault_wrapper_script(script: &Script) -> bool {
    let name_is_vault = script.name.trim().eq_ignore_ascii_case("vault");
    let alias_is_vault = script
        .alias
        .as_deref()
        .is_some_and(|alias| alias.trim().eq_ignore_ascii_case("vault"));
    let description = script
        .description
        .as_deref()
        .unwrap_or("")
        .to_ascii_lowercase();
    let mentions_ai_conversations =
        description.contains("ai conversation") || description.contains("agent chat conversation");

    name_is_vault
        && alias_is_vault
        && mentions_ai_conversations
        && (description.contains("resume") || description.contains("past"))
}

fn legacy_ai_vault_direct_match_score(
    script: &Script,
    filename: &str,
    query_lower: &str,
) -> Option<i32> {
    if !query_lower.is_ascii() {
        return None;
    }

    let mut best = None::<i32>;
    let mut consider = |candidate: &str| {
        if !candidate.is_ascii() {
            return;
        }

        let candidate = candidate.trim().to_ascii_lowercase();
        let score = if candidate == query_lower {
            Some(score_from_tier(1000, 900))
        } else if query_lower.len() >= 3 && candidate.starts_with(query_lower) {
            Some(score_from_tier(950, 900))
        } else if query_lower.len() >= 3 && candidate.contains(query_lower) {
            Some(score_from_tier(850, 900))
        } else {
            None
        };

        if let Some(score) = score {
            best = Some(best.map_or(score, |current| current.max(score)));
        }
    };

    consider(&script.name);
    consider(filename);
    if let Some(alias) = script.alias.as_deref() {
        consider(alias);
    }
    consider("vault");
    consider("ai-vault");
    consider("aivault");

    best
}

fn metadata_match(candidate: &str, query_lower: &str, tier: i32) -> Option<TextMatch> {
    low_tier_substring_match(candidate, query_lower, tier)
}

fn property_keyword_match(
    typed_meta: &crate::metadata_parser::TypedMetadata,
    query_lower: &str,
) -> Option<TextMatch> {
    let mut best = None;
    let mut consider = |enabled: bool, keyword: &str| {
        if enabled {
            better_match(
                &mut best,
                low_tier_substring_match(keyword, query_lower, TIER_KEYWORD),
            );
        }
    };
    consider(
        typed_meta.cron.is_some() || typed_meta.schedule.is_some(),
        "cron",
    );
    consider(
        typed_meta.cron.is_some() || typed_meta.schedule.is_some(),
        "scheduled",
    );
    consider(
        typed_meta.cron.is_some() || typed_meta.schedule.is_some(),
        "schedule",
    );
    consider(typed_meta.background, "background");
    consider(typed_meta.background, "bg");
    consider(typed_meta.system, "system");
    consider(!typed_meta.watch.is_empty(), "watch");
    consider(!typed_meta.watch.is_empty(), "watching");
    best
}

fn script_match_kind_for_candidate(
    current_kind: ScriptMatchKind,
    current: &Option<TextMatch>,
    candidate: Option<TextMatch>,
    candidate_kind: ScriptMatchKind,
) -> (Option<TextMatch>, ScriptMatchKind) {
    let Some(candidate) = candidate else {
        return (current.clone(), current_kind);
    };
    let replace = match current {
        None => true,
        Some(existing) => {
            candidate.tier > existing.tier
                || (candidate.tier == existing.tier && candidate.score > existing.score)
        }
    };
    if replace {
        (Some(candidate), candidate_kind)
    } else {
        (current.clone(), current_kind)
    }
}

/// Fuzzy search scripts by query string
/// Searches across name, filename (e.g., "my-script.ts"), description, and path
/// Returns results sorted by relevance score (highest first)
/// Match indices are provided to enable UI highlighting of matched characters
///
/// H1 Optimization: Accepts Arc<Script> to avoid expensive clones during filter operations.
/// Each ScriptMatch contains an Arc::clone which is just a refcount bump.
pub fn fuzzy_search_scripts(scripts: &[Arc<Script>], query: &str) -> Vec<ScriptMatch> {
    if query.is_empty() {
        // If no query, return all scripts with equal score, sorted by name
        // Filter out hidden scripts (metadata = { hidden: true })
        return scripts
            .iter()
            .filter(|s| !s.typed_metadata.as_ref().is_some_and(|m| m.hidden))
            .map(|s| {
                let filename = extract_filename(&s.path);
                ScriptMatch {
                    script: Arc::clone(s),
                    score: 0,
                    filename,
                    match_indices: MatchIndices::default(),
                    match_kind: ScriptMatchKind::default(),
                    content_match: None,
                }
            })
            .collect();
    }

    let query_lower = query.to_lowercase();
    let mut matches = Vec::new();

    // Create nucleo context once for all scripts - reuses buffer across calls
    let mut nucleo = NucleoCtx::new(&query_lower);
    // Check if query is ASCII once for all items
    for script in scripts {
        // Skip hidden scripts - they should not appear in search results or grouped view
        // Hidden flag comes from typed metadata: metadata = { hidden: true }
        if script.typed_metadata.as_ref().is_some_and(|m| m.hidden) {
            continue;
        }

        let mut best: Option<TextMatch> = None;
        let mut match_kind = ScriptMatchKind::Name;
        // Lazy match indices - don't compute during scoring, will be computed on-demand
        let match_indices = MatchIndices::default();

        let filename = extract_filename(&script.path);

        if is_legacy_ai_vault_wrapper_script(script) {
            if let Some(score) = legacy_ai_vault_direct_match_score(script, &filename, &query_lower)
            {
                matches.push(ScriptMatch {
                    script: Arc::clone(script),
                    score,
                    filename,
                    match_indices,
                    match_kind: ScriptMatchKind::Name,
                    content_match: None,
                });
            }
            continue;
        }

        let (next, kind) = script_match_kind_for_candidate(
            match_kind,
            &best,
            primary_text_match(&script.name, &query_lower, &mut nucleo),
            ScriptMatchKind::Name,
        );
        best = next;
        match_kind = kind;

        let (next, kind) = script_match_kind_for_candidate(
            match_kind,
            &best,
            low_tier_substring_match(&filename, &query_lower, TIER_FILENAME),
            ScriptMatchKind::Filename,
        );
        best = next;
        match_kind = kind;

        // Score by alias match - allows users to search by trigger alias
        if let Some(ref alias) = script.alias {
            better_match(&mut best, metadata_match(alias, &query_lower, TIER_ALIAS));
        }

        // Score by keyboard shortcut match - allows finding scripts by their hotkey
        // Users may type "opt i" or "cmd shift k" to find which script has that shortcut
        if let Some(ref shortcut) = script.shortcut {
            better_match(
                &mut best,
                metadata_match(shortcut, &query_lower, TIER_ALIAS),
            );
        }

        // Score by kit name match - allows searching by kit (e.g., "cleanshot")
        if let Some(ref kit_name) = script.kit_name {
            if kit_name != "main" {
                better_match(
                    &mut best,
                    metadata_match(kit_name, &query_lower, TIER_KEYWORD),
                );
            }
        }

        // Score by tag match - allows searching by category (e.g., "productivity", "utility")
        // Tags come from typed metadata: metadata = { tags: ["productivity", "notes"] }
        if let Some(ref typed_meta) = script.typed_metadata {
            for tag in &typed_meta.tags {
                better_match(&mut best, metadata_match(tag, &query_lower, TIER_KEYWORD));
            }
        }

        // Score by author match - allows finding scripts by creator
        // Author comes from typed metadata: metadata = { author: "John Lindquist" }
        if let Some(ref typed_meta) = script.typed_metadata {
            if let Some(ref author) = typed_meta.author {
                better_match(
                    &mut best,
                    metadata_match(author, &query_lower, TIER_KEYWORD),
                );
            }
        }

        // Score by script property keyword match - allows finding scripts by behavior
        // Users can type "cron", "scheduled", "background", "system", or "watch" to find
        // scripts with those runtime properties. This makes special scripts discoverable.
        if let Some(ref typed_meta) = script.typed_metadata {
            better_match(&mut best, property_keyword_match(typed_meta, &query_lower));
        }

        if let Some(ref desc) = script.description {
            let (next, kind) = script_match_kind_for_candidate(
                match_kind,
                &best,
                normalized_substring_match(desc, &query_lower, TIER_DESCRIPTION),
                ScriptMatchKind::Description,
            );
            best = next;
            match_kind = kind;
        }

        let mut content_match = None;
        if let Some(ref body) = script.body {
            if let Some(hit) = find_best_content_line(body, &query_lower) {
                if crate::logging::filter_perf_trace_enabled() {
                    crate::logging::log(
                        "FILTER_PERF",
                        &format!(
                            "[CONTENT_MATCH] script='{}' line={} current_tier={} bonus={}",
                            script.name,
                            hit.line_number,
                            best.as_ref().map(|m| m.tier).unwrap_or(0),
                            SCORE_CONTENT_MATCH
                        ),
                    );
                }
                let candidate = TextMatch {
                    kind: TextMatchKind::Substring,
                    tier: TIER_BODY,
                    score: score_from_tier(TIER_BODY, SCORE_CONTENT_MATCH),
                    indices: hit.line_match_indices.clone(),
                };
                let replace = best
                    .as_ref()
                    .is_none_or(|current| candidate.tier > current.tier);
                if replace {
                    best = Some(candidate);
                    match_kind = ScriptMatchKind::Content;
                    content_match = Some(hit);
                }
            }
        }

        if let Some(best) = best {
            matches.push(ScriptMatch {
                script: Arc::clone(script),
                score: best.score,
                filename,
                match_indices,
                match_kind,
                content_match,
            });
        }
    }

    // Sort by score (highest first), then by name for ties
    matches.sort_by(|a, b| match b.score.cmp(&a.score) {
        Ordering::Equal => a.script.name.cmp(&b.script.name),
        other => other,
    });

    matches
}

/// Candidate for the best matching line within a script body.
#[derive(Debug)]
struct ContentLineCandidate {
    score: u32,
    line_number: usize,
    line_text: String,
    line_match_indices: Vec<usize>,
    byte_range: Range<usize>,
}

/// Scan body text line-by-line.
///
/// Body matches are exact-only and low-tier. This avoids source identifiers or
/// imports admitting launcher rows through sparse fuzzy matches.
/// Returns the best-scoring line with its 1-based line number and match indices.
fn find_best_content_line(body: &str, query_lower: &str) -> Option<ScriptContentMatch> {
    if query_lower.chars().count() < MIN_BODY_EXACT_QUERY_LEN {
        return None;
    }

    let mut best: Option<ContentLineCandidate> = None;
    let mut byte_offset = 0usize;

    for (idx, segment) in body.split_inclusive('\n').enumerate() {
        let line = segment.strip_suffix('\n').unwrap_or(segment);
        let trimmed = line.trim();
        if trimmed.is_empty() {
            byte_offset += segment.len();
            continue;
        }
        if should_skip_body_search_line(trimmed) {
            byte_offset += segment.len();
            continue;
        }

        let Some(candidate) =
            exact_content_line_match(line, trimmed, query_lower, byte_offset, idx + 1)
        else {
            byte_offset += segment.len();
            continue;
        };

        let replace = match &best {
            None => true,
            Some(current) => {
                candidate.score > current.score
                    || (candidate.score == current.score
                        && candidate.line_number < current.line_number)
            }
        };
        if replace {
            best = Some(candidate);
        }

        byte_offset += segment.len();
    }

    best.map(|c| ScriptContentMatch {
        line_number: c.line_number,
        line_text: c.line_text,
        line_match_indices: c.line_match_indices,
        byte_range: c.byte_range,
    })
}

fn exact_content_line_match(
    raw_line: &str,
    trimmed_line: &str,
    query_lower: &str,
    body_byte_offset: usize,
    line_number: usize,
) -> Option<ContentLineCandidate> {
    let trimmed_start = raw_line.find(trimmed_line)?;
    let line_match_indices = if trimmed_line.is_ascii() && query_lower.is_ascii() {
        let match_start = find_ignore_ascii_case(trimmed_line, query_lower)?;
        (match_start..match_start + query_lower.chars().count()).collect()
    } else {
        low_tier_substring_match(trimmed_line, query_lower, TIER_BODY)?.indices
    };
    let line_relative_range = byte_range_for_char_indices(trimmed_line, &line_match_indices)?;

    Some(ContentLineCandidate {
        score: query_lower.chars().count() as u32,
        line_number,
        line_text: trimmed_line.to_string(),
        line_match_indices,
        byte_range: (body_byte_offset + trimmed_start + line_relative_range.start)
            ..(body_byte_offset + trimmed_start + line_relative_range.end),
    })
}

fn should_skip_body_search_line(trimmed_line: &str) -> bool {
    trimmed_line.starts_with(CURRENT_APP_RECIPE_HEADER_PREFIX)
}
