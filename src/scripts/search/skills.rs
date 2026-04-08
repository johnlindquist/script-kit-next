use std::cmp::Ordering;
use std::sync::Arc;

use crate::plugins::PluginSkill;

use super::super::types::{MatchIndices, SkillMatch};
use super::{
    contains_ignore_ascii_case, find_ignore_ascii_case, is_exact_name_match,
    is_word_boundary_match, NucleoCtx, MIN_FUZZY_QUERY_LEN,
};

/// Fuzzy search plugin skills by query string.
/// Matches against title, skill_id, plugin_title, and description.
/// Returns results sorted by relevance score (highest first).
pub fn fuzzy_search_skills(skills: &[Arc<PluginSkill>], query: &str) -> Vec<SkillMatch> {
    if query.is_empty() {
        return skills
            .iter()
            .map(|s| SkillMatch {
                skill: s.clone(),
                score: 0,
                match_indices: MatchIndices::default(),
            })
            .collect();
    }

    let query_lower = query.to_lowercase();
    let mut matches: Vec<(usize, i32, MatchIndices)> = Vec::with_capacity(skills.len());

    let mut nucleo = NucleoCtx::new(&query_lower);
    let query_is_ascii = query_lower.is_ascii();
    let use_nucleo = query_lower.len() >= MIN_FUZZY_QUERY_LEN;

    for (index, skill) in skills.iter().enumerate() {
        let mut score = 0i32;
        let mut name_indices = Vec::new();

        // Exact title match
        if query_is_ascii
            && skill.title.is_ascii()
            && is_exact_name_match(&skill.title, &query_lower)
        {
            score += 500;
        }

        // Substring match in title
        if query_is_ascii && skill.title.is_ascii() {
            if let Some(pos) = find_ignore_ascii_case(&skill.title, &query_lower) {
                score += if pos == 0 { 100 } else { 75 };
                if pos > 0 && is_word_boundary_match(&skill.title, pos) {
                    score += 20;
                }
                // Record matched character indices for highlighting
                for i in 0..query_lower.len() {
                    name_indices.push(pos + i);
                }
            }
        }

        // Nucleo fuzzy match in title
        if use_nucleo {
            if let Some(nucleo_s) = nucleo.score(&skill.title) {
                score += 50 + (nucleo_s / 20) as i32;
            }
        }

        // Substring match in skill_id
        if query_is_ascii
            && skill.skill_id.is_ascii()
            && contains_ignore_ascii_case(&skill.skill_id, &query_lower)
        {
            score += 30;
        }

        // Substring match in plugin_title
        if query_is_ascii
            && skill.plugin_title.is_ascii()
            && contains_ignore_ascii_case(&skill.plugin_title, &query_lower)
        {
            score += 15;
        }

        // Substring match in description
        if !skill.description.is_empty()
            && query_is_ascii
            && skill.description.is_ascii()
            && contains_ignore_ascii_case(&skill.description, &query_lower)
        {
            score += 10;
        }

        if score > 0 {
            matches.push((
                index,
                score,
                MatchIndices {
                    name_indices,
                    filename_indices: Vec::new(),
                    description_indices: Vec::new(),
                },
            ));
        }
    }

    matches.sort_by(
        |(a_idx, a_score, _), (b_idx, b_score, _)| match b_score.cmp(a_score) {
            Ordering::Equal => skills[*a_idx].title.cmp(&skills[*b_idx].title),
            other => other,
        },
    );

    matches
        .into_iter()
        .map(|(index, score, match_indices)| SkillMatch {
            skill: skills[index].clone(),
            score,
            match_indices,
        })
        .collect()
}
