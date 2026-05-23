use std::cmp::Ordering;
use std::sync::Arc;

use crate::plugins::PluginSkill;

use super::super::types::{MatchIndices, SkillMatch};
use super::{
    better_match, low_tier_substring_match, primary_text_match, NucleoCtx, TIER_DESCRIPTION,
    TIER_FILENAME, TIER_KEYWORD,
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
    for (index, skill) in skills.iter().enumerate() {
        let mut best = None;
        let mut name_indices = Vec::new();

        let title_match = primary_text_match(&skill.title, &query_lower, &mut nucleo);
        if let Some(title_match) = title_match {
            name_indices = title_match.indices.clone();
            best = Some(title_match);
        }

        better_match(
            &mut best,
            low_tier_substring_match(&skill.skill_id, &query_lower, TIER_FILENAME),
        );

        better_match(
            &mut best,
            low_tier_substring_match(&skill.plugin_title, &query_lower, TIER_KEYWORD),
        );

        if !skill.description.is_empty() {
            better_match(
                &mut best,
                low_tier_substring_match(&skill.description, &query_lower, TIER_DESCRIPTION),
            );
        }

        if let Some(best) = best {
            matches.push((
                index,
                best.score,
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
