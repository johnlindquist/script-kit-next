use std::cmp::Ordering;

use crate::builtins::{BuiltInEntry, BuiltInGroup};

use super::super::types::BuiltInMatch;
use super::{
    contains_ignore_ascii_case, find_ignore_ascii_case, is_exact_name_match,
    is_word_boundary_match, NucleoCtx, MIN_FUZZY_QUERY_LEN,
};

/// Fuzzy search built-in entries by query string
/// Searches across name, description, and keywords
/// Returns results sorted by relevance score (highest first)
///
/// Scoring priorities (name matches are weighted MUCH higher than keywords):
/// - Name prefix match (starts with query): 200 points
/// - Name substring match (contains query): 150 points
/// - Name fuzzy match (nucleo): 100 + scaled nucleo score
/// - Description match: 25 points
/// - Keyword exact match: 40 points (much lower than name)
/// - Keyword fuzzy match: 20 + scaled nucleo score
pub fn fuzzy_search_builtins(entries: &[BuiltInEntry], query: &str) -> Vec<BuiltInMatch> {
    if query.is_empty() {
        // If no query, return all entries with equal score, sorted by name
        return entries
            .iter()
            .map(|e| BuiltInMatch {
                entry: e.clone(),
                score: 0,
            })
            .collect();
    }

    let query_lower = query.to_lowercase();
    let mut matches = Vec::new();

    // Create nucleo context once for all entries - reuses buffer across calls
    let mut nucleo = NucleoCtx::new(&query_lower);
    // Check if query is ASCII once for all items
    // Note: Built-in names, descriptions, and keywords are typically ASCII
    let query_is_ascii = query_lower.is_ascii();

    // Gate nucleo fuzzy matching on minimum query length to reduce noise
    let use_nucleo = query_lower.len() >= MIN_FUZZY_QUERY_LEN;

    for entry in entries {
        let mut score = 0i32;
        let mut name_matched = false;
        let mut leaf_name_matched = false;

        // Exact name match boost for non-menu-bar items
        if entry.group != BuiltInGroup::MenuBar
            && query_is_ascii
            && entry.name.is_ascii()
            && is_exact_name_match(&entry.name, &query_lower)
        {
            score += 500;
        }

        // For menu bar items, prioritize matching the LEAF name (actual menu item)
        // e.g., for "Shell → New Tab", matching "New Tab" should score high
        // IMPORTANT: We only accept SUBSTRING matches for menu bar items, not fuzzy
        // This prevents "how are" from matching "Clear the scrollback" via scattered chars
        if entry.group == BuiltInGroup::MenuBar {
            let leaf_name = entry.leaf_name();
            if query_is_ascii && leaf_name.is_ascii() {
                if let Some(pos) = find_ignore_ascii_case(leaf_name, &query_lower) {
                    // Strong bonus for matching the actual menu item name
                    // Prefix match on leaf name is the best possible match for menu items
                    score += if pos == 0 { 300 } else { 200 };
                    leaf_name_matched = true;
                    name_matched = true;
                }
            }
            // NO fuzzy matching for menu bar leaf names - too many false positives
            // Menu items should only match if query is a substring of the leaf name
        }

        // Score by full name match - HIGHEST priority for non-menu-bar items
        // For menu bar items, this adds to the leaf name score for path matches
        if query_is_ascii && entry.name.is_ascii() {
            if let Some(pos) = find_ignore_ascii_case(&entry.name, &query_lower) {
                // Bonus for exact substring match at start of name (prefix match is best)
                // Reduced bonus for menu bar items since leaf name match is primary
                let bonus = if entry.group == BuiltInGroup::MenuBar {
                    if pos == 0 {
                        50
                    } else {
                        25
                    }
                } else if pos == 0 {
                    200
                } else {
                    150
                };
                score += bonus;
                // Extra bonus for word-boundary matches in non-menu-bar items
                if pos > 0
                    && entry.group != BuiltInGroup::MenuBar
                    && is_word_boundary_match(&entry.name, pos)
                {
                    score += 20;
                }
                name_matched = true;
            }
        }

        // Fuzzy character matching in full name using nucleo (handles Unicode)
        // Skip for menu bar items entirely - they should only match on substring, not fuzzy
        // This prevents "how are" from matching menu items via scattered character matches
        if use_nucleo && !leaf_name_matched && entry.group != BuiltInGroup::MenuBar {
            if let Some(nucleo_s) = nucleo.score(&entry.name) {
                // Scale nucleo score - name fuzzy matches are worth more than keyword matches
                score += 100 + (nucleo_s / 15) as i32;
                name_matched = true;
            }
        }

        // Score by description match - medium priority
        // Built-in descriptions are ASCII
        if query_is_ascii
            && entry.description.is_ascii()
            && contains_ignore_ascii_case(&entry.description, &query_lower)
        {
            score += 25;
        }

        // Score by keyword match - LOWER priority than name matches
        // Keywords help find items but shouldn't outrank name matches
        // Keywords are ASCII
        if query_is_ascii {
            for keyword in &entry.keywords {
                if keyword.is_ascii() && contains_ignore_ascii_case(keyword, &query_lower) {
                    // Keyword matches are worth less than name matches
                    // This ensures "Scratch Pad" (name match) beats "Lock Screen" (keyword match on "screen")
                    score += 40;
                    break; // Only count once even if multiple keywords match
                }
            }
        }

        // Fuzzy match on keywords using nucleo (handles Unicode)
        // Only add keyword fuzzy score if we didn't already match the name well
        // This prevents keywords from inflating scores when the name is already a good match
        // Skip for menu bar items - they should only match on substring, not fuzzy
        if use_nucleo && !name_matched && entry.group != BuiltInGroup::MenuBar {
            for keyword in &entry.keywords {
                if let Some(nucleo_s) = nucleo.score(keyword) {
                    // Scale nucleo score - keyword fuzzy is worth less than name fuzzy
                    score += 20 + (nucleo_s / 40) as i32;
                    break; // Only count once
                }
            }
        }

        // Apply penalty for menu bar items that did NOT match their leaf name well
        // This prevents random path matches from cluttering results
        // But if they matched the leaf name, they should compete fairly
        if entry.group == BuiltInGroup::MenuBar && !leaf_name_matched {
            // Heavy penalty for menu bar items that only matched on path/keywords
            score = (score / 4).saturating_sub(50);
        }

        if score > 0 {
            matches.push(BuiltInMatch {
                entry: entry.clone(),
                score,
            });
        }
    }

    // Sort by score (highest first), then by name for ties
    matches.sort_by(|a, b| match b.score.cmp(&a.score) {
        Ordering::Equal => a.entry.name.cmp(&b.entry.name),
        other => other,
    });

    matches
}
