use std::cmp::Ordering;
use std::sync::Arc;

use crate::plugins::PluginSkill;
use crate::scripts::types::{MatchIndices, SkillMatch};

use super::{
    contains_ignore_ascii_case, find_ignore_ascii_case, is_exact_name_match,
    is_word_boundary_match, NucleoCtx, MIN_FUZZY_QUERY_LEN,
};

const SCORE_EXACT_NAME_MATCH: i32 = 500;
const SCORE_NAME_PREFIX: i32 = 100;
const SCORE_NAME_SUBSTRING: i32 = 75;
const SCORE_WORD_BOUNDARY: i32 = 20;
const SCORE_NAME_FUZZY_BASE: i32 = 50;
const SCORE_NAME_FUZZY_DIV: u32 = 20;
const SCORE_DESC_SUBSTRING: i32 = 25;
const SCORE_DESC_FUZZY_BASE: i32 = 15;
const SCORE_DESC_FUZZY_DIV: u32 = 30;
const SCORE_PLUGIN_PREFIX: i32 = 30;
const SCORE_PLUGIN_SUBSTRING: i32 = 20;

/// Fuzzy search plugin skills by query string.
///
/// Searches across title, description, and plugin title/id.
/// Returns results sorted by relevance score (highest first).
pub fn fuzzy_search_skills(skills: &[Arc<PluginSkill>], query: &str) -> Vec<SkillMatch> {
    if query.is_empty() {
        return skills
            .iter()
            .map(|s| SkillMatch {
                skill: Arc::clone(s),
                score: 0,
                match_indices: MatchIndices::default(),
            })
            .collect();
    }

    let query_lower = query.to_lowercase();
    let mut matches = Vec::new();
    let mut nucleo = NucleoCtx::new(&query_lower);
    let query_is_ascii = query_lower.is_ascii();
    let use_nucleo = query_lower.len() >= MIN_FUZZY_QUERY_LEN;

    for skill in skills {
        let mut score = 0i32;

        // Exact name match boost
        if query_is_ascii
            && skill.title.is_ascii()
            && is_exact_name_match(&skill.title, &query_lower)
        {
            score += SCORE_EXACT_NAME_MATCH;
        }

        // Score by title match — highest priority
        if query_is_ascii && skill.title.is_ascii() {
            if let Some(pos) = find_ignore_ascii_case(&skill.title, &query_lower) {
                score += if pos == 0 {
                    SCORE_NAME_PREFIX
                } else {
                    SCORE_NAME_SUBSTRING
                };
                if pos > 0 && is_word_boundary_match(&skill.title, pos) {
                    score += SCORE_WORD_BOUNDARY;
                }
            }
        }

        // Fuzzy matching on title via nucleo
        if use_nucleo {
            if let Some(nucleo_s) = nucleo.score(&skill.title) {
                score += SCORE_NAME_FUZZY_BASE + (nucleo_s / SCORE_NAME_FUZZY_DIV) as i32;
            }
        }

        // Score by description match
        if !skill.description.is_empty() {
            if query_is_ascii
                && skill.description.is_ascii()
                && contains_ignore_ascii_case(&skill.description, &query_lower)
            {
                score += SCORE_DESC_SUBSTRING;
            }
            if use_nucleo {
                if let Some(nucleo_s) = nucleo.score(&skill.description) {
                    score += SCORE_DESC_FUZZY_BASE + (nucleo_s / SCORE_DESC_FUZZY_DIV) as i32;
                }
            }
        }

        // Score by plugin title/id match
        let plugin_label = if skill.plugin_title.is_empty() {
            &skill.plugin_id
        } else {
            &skill.plugin_title
        };
        if query_is_ascii && plugin_label.is_ascii() {
            if let Some(pos) = find_ignore_ascii_case(plugin_label, &query_lower) {
                score += if pos == 0 {
                    SCORE_PLUGIN_PREFIX
                } else {
                    SCORE_PLUGIN_SUBSTRING
                };
            }
        }

        if score > 0 {
            matches.push(SkillMatch {
                skill: Arc::clone(skill),
                score,
                match_indices: MatchIndices::default(),
            });
        }
    }

    matches.sort_by(|a, b| match b.score.cmp(&a.score) {
        Ordering::Equal => a.skill.title.cmp(&b.skill.title),
        other => other,
    });

    matches
}
