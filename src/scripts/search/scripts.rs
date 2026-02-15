use std::cmp::Ordering;
use std::sync::Arc;

use super::super::types::{MatchIndices, Script, ScriptMatch};
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

        if score > 0 {
            matches.push(ScriptMatch {
                script: Arc::clone(script),
                score,
                filename,
                match_indices,
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
