//! Fuzzy search functionality for scripts, scriptlets, and other items
//!
//! This module provides fuzzy search functions using nucleo for high-performance
//! matching and scoring, plus ASCII case-folding helpers for efficiency.

mod apps;
mod ascii;
mod builtins;
mod highlight;
mod match_contract;
mod nucleo;
mod paths;
mod prefix_filters;
mod scriptlets;
mod scripts;
mod skills;
mod unified;
mod windows;

pub(crate) mod ghost;

pub use apps::fuzzy_search_apps;
pub use builtins::fuzzy_search_builtins;
pub use highlight::compute_match_indices_for_result;
pub use nucleo::NucleoCtx;
pub use scriptlets::fuzzy_search_scriptlets;
pub use scripts::fuzzy_search_scripts;
pub use skills::fuzzy_search_skills;
pub use unified::{
    fuzzy_search_unified, fuzzy_search_unified_all, fuzzy_search_unified_all_with_skills,
};
#[cfg(test)]
pub use unified::{fuzzy_search_unified_with_builtins, fuzzy_search_unified_with_windows};
pub use windows::{fuzzy_search_root_windows, fuzzy_search_windows};

pub(crate) use ascii::{
    contains_ignore_ascii_case, find_ignore_ascii_case, fuzzy_match_with_indices_ascii,
    is_ascii_pair, is_word_boundary_match, MIN_FUZZY_QUERY_LEN,
};
pub(crate) use match_contract::{
    better_match, better_match_evidence, byte_range_for_char_indices, low_tier_substring_match,
    match_evidence, match_tier_from_score, normalized_substring_match, primary_text_match,
    score_from_tier, TextMatch, TextMatchKind, MIN_BODY_EXACT_QUERY_LEN, TIER_ALIAS, TIER_BODY,
    TIER_DESCRIPTION, TIER_FILENAME, TIER_KEYWORD,
};

#[cfg(test)]
pub(crate) use ascii::{fuzzy_match_with_indices, is_exact_name_match, is_fuzzy_match};
pub(crate) use paths::{extract_filename, extract_scriptlet_display_path};
pub(crate) use prefix_filters::{
    app_passes_prefix_filter, builtin_passes_prefix_filter, parse_query_prefix,
    script_passes_prefix_filter, scriptlet_passes_prefix_filter, should_search_scriptlets,
    should_search_scripts, skill_passes_prefix_filter, window_passes_prefix_filter,
};

#[cfg(test)]
pub(crate) use super::types::{Script, Scriptlet, SearchResult};

#[cfg(test)]
mod tests;
