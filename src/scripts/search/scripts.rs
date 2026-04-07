use std::cmp::Ordering;
use std::ops::Range;
use std::sync::Arc;

use super::super::types::{MatchIndices, Script, ScriptContentMatch, ScriptMatch, ScriptMatchKind};
use super::{
    contains_ignore_ascii_case, extract_filename, find_ignore_ascii_case, is_exact_name_match,
    is_word_boundary_match, NucleoCtx, MIN_FUZZY_QUERY_LEN,
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
    let query_is_ascii = query_lower.is_ascii();

    // Gate nucleo fuzzy matching on minimum query length to reduce noise
    let use_nucleo = query_lower.len() >= MIN_FUZZY_QUERY_LEN;

    for script in scripts {
        // Skip hidden scripts - they should not appear in search results or grouped view
        // Hidden flag comes from typed metadata: metadata = { hidden: true }
        if script.typed_metadata.as_ref().is_some_and(|m| m.hidden) {
            continue;
        }

        let mut score = 0i32;
        // Lazy match indices - don't compute during scoring, will be computed on-demand
        let match_indices = MatchIndices::default();

        let filename = extract_filename(&script.path);

        // Exact name match boost: if the query IS the full name, always rank first
        if query_is_ascii
            && script.name.is_ascii()
            && is_exact_name_match(&script.name, &query_lower)
        {
            score += SCORE_EXACT_NAME_MATCH;
        }

        // Score by name match - highest priority
        // Only use ASCII fast-path when both strings are ASCII
        if query_is_ascii && script.name.is_ascii() {
            if let Some(pos) = find_ignore_ascii_case(&script.name, &query_lower) {
                // Bonus for exact substring match at start of name
                score += if pos == 0 {
                    SCORE_NAME_PREFIX
                } else {
                    SCORE_NAME_SUBSTRING
                };
                // Extra bonus for word-boundary matches (e.g., "new" in "New Tab")
                if pos > 0 && is_word_boundary_match(&script.name, pos) {
                    score += SCORE_WORD_BOUNDARY;
                }
            }
        }

        // Fuzzy character matching in name using nucleo (handles Unicode correctly)
        // Only for queries >= MIN_FUZZY_QUERY_LEN to avoid noisy single-char matches
        if use_nucleo {
            if let Some(nucleo_s) = nucleo.score(&script.name) {
                // Scale nucleo score (0-1000+) to match existing weights (~50 for fuzzy match)
                score += SCORE_NAME_FUZZY_BASE + (nucleo_s / SCORE_NAME_FUZZY_DIV) as i32;
            }
        }

        // Score by filename match - high priority (allows searching by ".ts", ".js", etc.)
        // Filenames are typically ASCII
        if query_is_ascii && filename.is_ascii() {
            if let Some(pos) = find_ignore_ascii_case(&filename, &query_lower) {
                // Bonus for exact substring match at start of filename
                score += if pos == 0 {
                    SCORE_FILENAME_PREFIX
                } else {
                    SCORE_FILENAME_SUBSTRING
                };
            }
        }

        // Fuzzy character matching in filename using nucleo (handles Unicode)
        if use_nucleo {
            if let Some(nucleo_s) = nucleo.score(&filename) {
                // Scale nucleo score to match existing weights (~35 for filename fuzzy match)
                score += SCORE_FILENAME_FUZZY_BASE + (nucleo_s / SCORE_FILENAME_FUZZY_DIV) as i32;
            }
        }

        // Score by alias match - allows users to search by trigger alias
        if let Some(ref alias) = script.alias {
            if query_is_ascii && alias.is_ascii() {
                if let Some(pos) = find_ignore_ascii_case(alias, &query_lower) {
                    // Strong bonus for alias match (aliases are explicit shortcuts)
                    score += if pos == 0 {
                        SCORE_ALIAS_PREFIX
                    } else {
                        SCORE_ALIAS_SUBSTRING
                    };
                }
            }
        }

        // Score by keyboard shortcut match - allows finding scripts by their hotkey
        // Users may type "opt i" or "cmd shift k" to find which script has that shortcut
        if let Some(ref shortcut) = script.shortcut {
            if query_is_ascii && shortcut.is_ascii() {
                if let Some(pos) = find_ignore_ascii_case(shortcut, &query_lower) {
                    // Strong bonus for shortcut match (shortcuts are explicit bindings)
                    score += if pos == 0 {
                        SCORE_SHORTCUT_PREFIX
                    } else {
                        SCORE_SHORTCUT_SUBSTRING
                    };
                }
            }
        }

        // Score by kit name match - allows searching by kit (e.g., "cleanshot")
        if let Some(ref kit_name) = script.kit_name {
            if kit_name != "main" && query_is_ascii && kit_name.is_ascii() {
                if let Some(pos) = find_ignore_ascii_case(kit_name, &query_lower) {
                    // Moderate bonus for kit name match (helps find all scripts from a kit)
                    score += if pos == 0 {
                        SCORE_KIT_PREFIX
                    } else {
                        SCORE_KIT_SUBSTRING
                    };
                }
            }
        }

        // Score by tag match - allows searching by category (e.g., "productivity", "utility")
        // Tags come from typed metadata: metadata = { tags: ["productivity", "notes"] }
        if let Some(ref typed_meta) = script.typed_metadata {
            for tag in &typed_meta.tags {
                if query_is_ascii && tag.is_ascii() {
                    if let Some(pos) = find_ignore_ascii_case(tag, &query_lower) {
                        // Moderate bonus for tag match (helps discover scripts by category)
                        score += if pos == 0 {
                            SCORE_TAG_PREFIX
                        } else {
                            SCORE_TAG_SUBSTRING
                        };
                        break; // Only count best tag match once
                    }
                }
            }
        }

        // Score by author match - allows finding scripts by creator
        // Author comes from typed metadata: metadata = { author: "John Lindquist" }
        if let Some(ref typed_meta) = script.typed_metadata {
            if let Some(ref author) = typed_meta.author {
                if query_is_ascii && author.is_ascii() {
                    if let Some(pos) = find_ignore_ascii_case(author, &query_lower) {
                        // Moderate bonus for author match
                        score += if pos == 0 {
                            SCORE_AUTHOR_PREFIX
                        } else {
                            SCORE_AUTHOR_SUBSTRING
                        };
                    }
                }
            }
        }

        // Score by script property keyword match - allows finding scripts by behavior
        // Users can type "cron", "scheduled", "background", "system", or "watch" to find
        // scripts with those runtime properties. This makes special scripts discoverable.
        if let Some(ref typed_meta) = script.typed_metadata {
            let ql = query_lower.as_str();
            if (typed_meta.cron.is_some() || typed_meta.schedule.is_some())
                && (contains_ignore_ascii_case("cron", ql)
                    || contains_ignore_ascii_case("scheduled", ql)
                    || contains_ignore_ascii_case("schedule", ql))
            {
                score += SCORE_PROPERTY_KEYWORD;
            }
            if typed_meta.background
                && (contains_ignore_ascii_case("background", ql)
                    || contains_ignore_ascii_case("bg", ql))
            {
                score += SCORE_PROPERTY_KEYWORD;
            }
            if typed_meta.system && contains_ignore_ascii_case("system", ql) {
                score += SCORE_PROPERTY_KEYWORD;
            }
            if !typed_meta.watch.is_empty()
                && (contains_ignore_ascii_case("watch", ql)
                    || contains_ignore_ascii_case("watching", ql))
            {
                score += SCORE_PROPERTY_KEYWORD;
            }
        }

        // Score by description match - medium priority
        // Substring match + nucleo fuzzy for catching typos and partial matches
        if let Some(ref desc) = script.description {
            if query_is_ascii && desc.is_ascii() && contains_ignore_ascii_case(desc, &query_lower) {
                score += SCORE_DESC_SUBSTRING;
            }
            // Fuzzy match on description using nucleo (catches typos and partial terms)
            if use_nucleo {
                if let Some(nucleo_s) = nucleo.score(desc) {
                    score += SCORE_DESC_FUZZY_BASE + (nucleo_s / SCORE_DESC_FUZZY_DIV) as i32;
                }
            }
        }

        // Score by path match - lower priority
        // Paths are typically ASCII
        let path_str = script.path.to_string_lossy();
        if query_is_ascii
            && path_str.is_ascii()
            && contains_ignore_ascii_case(&path_str, &query_lower)
        {
            score += SCORE_PATH_SUBSTRING;
        }

        // Determine match kind based on which tier contributed the most
        let match_kind = if score >= SCORE_EXACT_NAME_MATCH
            || score >= SCORE_NAME_PREFIX
            || score >= SCORE_NAME_FUZZY_BASE
        {
            ScriptMatchKind::Name
        } else if score >= SCORE_DESC_SUBSTRING {
            ScriptMatchKind::Description
        } else if score >= SCORE_FILENAME_PREFIX {
            ScriptMatchKind::Filename
        } else {
            ScriptMatchKind::Name // default
        };

        // Determine whether a primary text field already won
        let primary_text_match = matches!(
            match_kind,
            ScriptMatchKind::Name | ScriptMatchKind::Description | ScriptMatchKind::Filename
        ) && score > 0;

        // Content body search — lowest-priority tier (+5)
        // Always search body so content bonus stacks with other tiers
        let mut content_match = None;
        if let Some(ref body) = script.body {
            if let Some(hit) = find_best_content_line(body, &query_lower) {
                score += SCORE_CONTENT_MATCH;
                crate::logging::log(
                    "FILTER_PERF",
                    &format!(
                        "[CONTENT_MATCH] script='{}' line={} primary_text_match={} bonus={}",
                        script.name, hit.line_number, primary_text_match, SCORE_CONTENT_MATCH
                    ),
                );
                // Only surface the snippet row when no stronger field won
                if !primary_text_match {
                    content_match = Some(hit);
                }
            }
        }

        let final_match_kind = if content_match.is_some() {
            ScriptMatchKind::Content
        } else {
            match_kind
        };

        if score > 0 {
            matches.push(ScriptMatch {
                script: Arc::clone(script),
                score,
                filename,
                match_indices,
                match_kind: final_match_kind,
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

/// Candidate for the best fuzzy-matching line within a script body.
#[derive(Debug)]
struct ContentLineCandidate {
    score: u32,
    line_number: usize,
    line_text: String,
    line_match_indices: Vec<usize>,
    byte_range: Range<usize>,
}

/// Scan body text line-by-line with nucleo fuzzy matching.
/// Returns the best-scoring line with its 1-based line number and match indices.
fn find_best_content_line(body: &str, query_lower: &str) -> Option<ScriptContentMatch> {
    let mut best: Option<ContentLineCandidate> = None;
    let mut ctx = NucleoCtx::new(query_lower);
    let mut byte_offset = 0usize;

    for (idx, segment) in body.split_inclusive('\n').enumerate() {
        let line = segment.strip_suffix('\n').unwrap_or(segment);
        let trimmed = line.trim();
        if trimmed.is_empty() {
            byte_offset += segment.len();
            continue;
        }

        let Some(score) = ctx.score(trimmed) else {
            byte_offset += segment.len();
            continue;
        };

        let Some(line_match_indices) = ctx.indices(trimmed) else {
            byte_offset += segment.len();
            continue;
        };

        let byte_range = match matched_span_byte_range(line, trimmed, &line_match_indices) {
            Some(line_relative_range) => {
                (byte_offset + line_relative_range.start)..(byte_offset + line_relative_range.end)
            }
            None => {
                byte_offset += segment.len();
                continue;
            }
        };

        let candidate = ContentLineCandidate {
            score,
            line_number: idx + 1,
            line_text: trimmed.to_string(),
            line_match_indices,
            byte_range,
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

fn matched_span_byte_range(
    raw_line: &str,
    trimmed_line: &str,
    line_match_indices: &[usize],
) -> Option<Range<usize>> {
    let &first_char = line_match_indices.first()?;
    let &last_char = line_match_indices.last()?;
    if first_char > last_char {
        return None;
    }

    let trimmed_start = raw_line.find(trimmed_line)?;
    let mut offsets: Vec<usize> = trimmed_line
        .char_indices()
        .map(|(byte_idx, _)| byte_idx)
        .collect();
    offsets.push(trimmed_line.len());

    let start = *offsets.get(first_char)?;
    let end = *offsets.get(last_char + 1)?;
    Some((trimmed_start + start)..(trimmed_start + end))
}
