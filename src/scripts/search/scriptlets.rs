use std::cmp::Ordering;
use std::sync::Arc;

use super::super::types::{MatchIndices, Scriptlet, ScriptletMatch};
use super::{
    contains_ignore_ascii_case, extract_scriptlet_display_path, find_ignore_ascii_case,
    is_exact_name_match, is_word_boundary_match, NucleoCtx, MIN_FUZZY_QUERY_LEN,
};

/// Fuzzy search scriptlets by query string
/// Searches across name, file_path with anchor (e.g., "url.md#open-github"), description, and code
/// Returns results sorted by relevance score (highest first)
/// Match indices are provided to enable UI highlighting of matched characters
///
/// H1 Optimization: Accepts Arc<Scriptlet> to avoid expensive clones during filter operations.
/// Each ScriptletMatch contains an Arc::clone which is just a refcount bump.
pub fn fuzzy_search_scriptlets(scriptlets: &[Arc<Scriptlet>], query: &str) -> Vec<ScriptletMatch> {
    if query.is_empty() {
        // If no query, return all scriptlets with equal score, sorted by name
        return scriptlets
            .iter()
            .map(|s| {
                let display_file_path = extract_scriptlet_display_path(&s.file_path);
                ScriptletMatch {
                    scriptlet: Arc::clone(s),
                    score: 0,
                    display_file_path,
                    match_indices: MatchIndices::default(),
                }
            })
            .collect();
    }

    let query_lower = query.to_lowercase();
    let mut matches = Vec::new();

    // Create nucleo context once for all scriptlets - reuses buffer across calls
    let mut nucleo = NucleoCtx::new(&query_lower);
    // Check if query is ASCII once for all items
    let query_is_ascii = query_lower.is_ascii();

    // Gate nucleo fuzzy matching on minimum query length to reduce noise
    let use_nucleo = query_lower.len() >= MIN_FUZZY_QUERY_LEN;

    for scriptlet in scriptlets {
        let mut score = 0i32;
        // Lazy match indices - don't compute during scoring
        let match_indices = MatchIndices::default();

        let display_file_path = extract_scriptlet_display_path(&scriptlet.file_path);

        // Exact name match boost: if the query IS the full name, always rank first
        if query_is_ascii
            && scriptlet.name.is_ascii()
            && is_exact_name_match(&scriptlet.name, &query_lower)
        {
            score += 500;
        }

        // Score by name match - highest priority
        // Only use ASCII fast-path when both strings are ASCII
        if query_is_ascii && scriptlet.name.is_ascii() {
            if let Some(pos) = find_ignore_ascii_case(&scriptlet.name, &query_lower) {
                // Bonus for exact substring match at start of name
                score += if pos == 0 { 100 } else { 75 };
                // Extra bonus for word-boundary matches (e.g., "new" in "New Tab")
                if pos > 0 && is_word_boundary_match(&scriptlet.name, pos) {
                    score += 20;
                }
            }
        }

        // Fuzzy character matching in name using nucleo (handles Unicode)
        // Only for queries >= MIN_FUZZY_QUERY_LEN to avoid noisy single-char matches
        if use_nucleo {
            if let Some(nucleo_s) = nucleo.score(&scriptlet.name) {
                // Scale nucleo score to match existing weights (~50 for fuzzy match)
                score += 50 + (nucleo_s / 20) as i32;
            }
        }

        // Score by file_path match - high priority (allows searching by ".md", anchor names)
        if let Some(ref fp) = display_file_path {
            // File paths are typically ASCII
            if query_is_ascii && fp.is_ascii() {
                if let Some(pos) = find_ignore_ascii_case(fp, &query_lower) {
                    // Bonus for exact substring match at start of file_path
                    score += if pos == 0 { 60 } else { 45 };
                }
            }

            // Fuzzy character matching in file_path using nucleo (handles Unicode)
            if use_nucleo {
                if let Some(nucleo_s) = nucleo.score(fp) {
                    // Scale nucleo score to match existing weights (~35 for file_path fuzzy match)
                    score += 35 + (nucleo_s / 30) as i32;
                }
            }
        }

        // Score by description match - medium priority
        // Substring match + nucleo fuzzy for catching typos and partial matches
        if let Some(ref desc) = scriptlet.description {
            if query_is_ascii && desc.is_ascii() && contains_ignore_ascii_case(desc, &query_lower) {
                score += 25;
            }
            // Fuzzy match on description using nucleo (catches typos and partial terms)
            if use_nucleo {
                if let Some(nucleo_s) = nucleo.score(desc) {
                    score += 15 + (nucleo_s / 30) as i32;
                }
            }
        }

        // CRITICAL OPTIMIZATION: Only search code when query is long enough (>=4 chars)
        // and no other matches were found. Code search is the biggest performance hit
        // because scriptlet.code can be very large.
        // Code is typically ASCII
        if query_lower.len() >= 4
            && score == 0
            && query_is_ascii
            && scriptlet.code.is_ascii()
            && contains_ignore_ascii_case(&scriptlet.code, &query_lower)
        {
            score += 5;
        }

        // Bonus for keyword match - allows users to search by trigger keyword
        // Keywords are typically short ASCII strings
        if let Some(ref keyword) = scriptlet.keyword {
            if query_is_ascii && keyword.is_ascii() {
                if let Some(pos) = find_ignore_ascii_case(keyword, &query_lower) {
                    // Strong bonus for keyword match (keywords are explicit triggers)
                    score += if pos == 0 { 80 } else { 60 };
                }
            }
        }

        // Bonus for alias match - allows users to search by alias
        if let Some(ref alias) = scriptlet.alias {
            if query_is_ascii && alias.is_ascii() {
                if let Some(pos) = find_ignore_ascii_case(alias, &query_lower) {
                    // Strong bonus for alias match (aliases are explicit shortcuts)
                    score += if pos == 0 { 80 } else { 60 };
                }
            }
        }

        // Bonus for keyboard shortcut match - find scriptlets by their hotkey
        if let Some(ref shortcut) = scriptlet.shortcut {
            if query_is_ascii && shortcut.is_ascii() {
                if let Some(pos) = find_ignore_ascii_case(shortcut, &query_lower) {
                    // Strong bonus for shortcut match (shortcuts are explicit bindings)
                    score += if pos == 0 { 80 } else { 60 };
                }
            }
        }

        // Bonus for group name match - allows searching by group (e.g., "productivity")
        if let Some(ref group) = scriptlet.group {
            if group != "main" && query_is_ascii && group.is_ascii() {
                if let Some(pos) = find_ignore_ascii_case(group, &query_lower) {
                    // Moderate bonus for group name match (helps find all snippets from a group)
                    score += if pos == 0 { 30 } else { 20 };
                }
            }
        }

        // Bonus for tool type match
        // Tool types are ASCII (snippet, template, etc.)
        if query_is_ascii
            && scriptlet.tool.is_ascii()
            && contains_ignore_ascii_case(&scriptlet.tool, &query_lower)
        {
            score += 10;
        }

        if score > 0 {
            matches.push(ScriptletMatch {
                scriptlet: Arc::clone(scriptlet),
                score,
                display_file_path,
                match_indices,
            });
        }
    }

    // Sort by score (highest first), then by name for ties
    matches.sort_by(|a, b| match b.score.cmp(&a.score) {
        Ordering::Equal => a.scriptlet.name.cmp(&b.scriptlet.name),
        other => other,
    });

    matches
}
