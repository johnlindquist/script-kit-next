use super::{find_ignore_ascii_case, is_word_boundary_match, NucleoCtx};

pub(crate) const TIER_EXACT_PRIMARY: i32 = 1000;
pub(crate) const TIER_PREFIX_PRIMARY: i32 = 950;
pub(crate) const TIER_WORD_BOUNDARY_PRIMARY: i32 = 900;
pub(crate) const TIER_SUBSTRING_PRIMARY: i32 = 850;
pub(crate) const TIER_ACRONYM_PRIMARY: i32 = 800;
pub(crate) const TIER_COMPACT_FUZZY_PRIMARY: i32 = 700;
pub(crate) const TIER_ALIAS: i32 = 650;
pub(crate) const TIER_KEYWORD: i32 = 550;
pub(crate) const TIER_DESCRIPTION: i32 = 450;
pub(crate) const TIER_FILENAME: i32 = 375;
pub(crate) const TIER_PATH: i32 = 250;
pub(crate) const TIER_BODY: i32 = 150;

pub(crate) const MIN_PRIMARY_FUZZY_QUERY_LEN: usize = 4;
pub(crate) const MIN_BODY_EXACT_QUERY_LEN: usize = 5;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum TextMatchKind {
    Exact,
    Prefix,
    WordBoundary,
    Substring,
    Acronym,
    CompactFuzzy,
}

#[derive(Clone, Debug)]
pub(crate) struct TextMatch {
    pub(crate) kind: TextMatchKind,
    pub(crate) tier: i32,
    pub(crate) score: i32,
    pub(crate) indices: Vec<usize>,
}

pub(crate) fn score_from_tier(tier: i32, bonus: i32) -> i32 {
    tier.saturating_mul(1000)
        .saturating_add(bonus.clamp(0, 999))
}

pub(crate) fn match_tier_from_score(score: i32) -> i32 {
    if score <= 0 {
        0
    } else {
        score / 1000
    }
}

pub(crate) fn exact_substring_match(
    haystack: &str,
    query_lower: &str,
    tier: i32,
) -> Option<TextMatch> {
    if query_lower.is_empty() {
        return None;
    }

    let indices = substring_indices(haystack, query_lower)?;
    let start = *indices.first()?;
    let kind = if haystack.chars().count() == indices.len() {
        TextMatchKind::Exact
    } else if start == 0 {
        TextMatchKind::Prefix
    } else if char_index_is_word_start(haystack, start) {
        TextMatchKind::WordBoundary
    } else {
        TextMatchKind::Substring
    };
    let adjusted_tier = match kind {
        TextMatchKind::Exact => tier.max(TIER_EXACT_PRIMARY),
        TextMatchKind::Prefix => tier.max(TIER_PREFIX_PRIMARY),
        TextMatchKind::WordBoundary => tier.max(TIER_WORD_BOUNDARY_PRIMARY),
        TextMatchKind::Substring => tier,
        TextMatchKind::Acronym | TextMatchKind::CompactFuzzy => tier,
    };

    Some(TextMatch {
        kind,
        tier: adjusted_tier,
        score: score_from_tier(
            adjusted_tier,
            900usize.saturating_sub(start).min(900) as i32,
        ),
        indices,
    })
}

pub(crate) fn low_tier_substring_match(
    haystack: &str,
    query_lower: &str,
    tier: i32,
) -> Option<TextMatch> {
    let indices = substring_indices(haystack, query_lower)?;
    let start = *indices.first()?;
    Some(TextMatch {
        kind: if start == 0 {
            TextMatchKind::Prefix
        } else {
            TextMatchKind::Substring
        },
        tier,
        score: score_from_tier(tier, 900usize.saturating_sub(start).min(900) as i32),
        indices,
    })
}

pub(crate) fn normalized_substring_match(
    haystack: &str,
    query_lower: &str,
    tier: i32,
) -> Option<TextMatch> {
    if let Some(exact) = low_tier_substring_match(haystack, query_lower, tier) {
        return Some(exact);
    }
    let indices = normalized_indices_for_query(haystack, query_lower)?;
    Some(TextMatch {
        kind: TextMatchKind::Substring,
        tier,
        score: score_from_tier(tier, 900),
        indices,
    })
}

pub(crate) fn primary_text_match(
    haystack: &str,
    query_lower: &str,
    nucleo: &mut NucleoCtx,
) -> Option<TextMatch> {
    if let Some(exact) = exact_substring_match(haystack, query_lower, TIER_SUBSTRING_PRIMARY) {
        return Some(exact);
    }

    if query_lower.chars().count() < MIN_PRIMARY_FUZZY_QUERY_LEN {
        return None;
    }

    let score = nucleo.score(haystack)?;
    let indices = nucleo.indices(haystack)?;
    if indices.is_empty() {
        return None;
    }

    let query_len = query_lower.chars().count();
    let first = *indices.first()?;
    let last = *indices.last()?;
    let span = last.saturating_sub(first).saturating_add(1);
    if span <= query_len.saturating_add(1) {
        let tier = TIER_COMPACT_FUZZY_PRIMARY;
        return Some(TextMatch {
            kind: TextMatchKind::CompactFuzzy,
            tier,
            score: score_from_tier(tier, (score / 20).min(999) as i32),
            indices,
        });
    }

    if fuzzy_indices_are_structured_abbreviation(haystack, &indices) {
        let tier = TIER_ACRONYM_PRIMARY;
        return Some(TextMatch {
            kind: TextMatchKind::Acronym,
            tier,
            score: score_from_tier(tier, (score / 20).min(999) as i32),
            indices,
        });
    }

    None
}

pub(crate) fn better_match(current: &mut Option<TextMatch>, candidate: Option<TextMatch>) {
    let Some(candidate) = candidate else {
        return;
    };
    let replace = match current {
        None => true,
        Some(existing) => {
            candidate.tier > existing.tier
                || (candidate.tier == existing.tier && candidate.score > existing.score)
        }
    };
    if replace {
        *current = Some(candidate);
    }
}

fn substring_indices(haystack: &str, query_lower: &str) -> Option<Vec<usize>> {
    if haystack.is_ascii() && query_lower.is_ascii() {
        let start = find_ignore_ascii_case(haystack, query_lower)?;
        return char_indices_for_span(haystack, start, query_lower.chars().count());
    }

    normalized_indices_for_query(haystack, query_lower)
}

pub(crate) fn char_indices_for_span(
    haystack: &str,
    byte_start: usize,
    char_len: usize,
) -> Option<Vec<usize>> {
    let start_char = haystack[..byte_start].chars().count();
    Some((start_char..start_char + char_len).collect())
}

pub(crate) fn byte_range_for_char_indices(
    haystack: &str,
    indices: &[usize],
) -> Option<std::ops::Range<usize>> {
    let first = *indices.first()?;
    let last = *indices.last()?;
    let mut offsets: Vec<usize> = haystack.char_indices().map(|(idx, _)| idx).collect();
    offsets.push(haystack.len());
    Some(*offsets.get(first)?..*offsets.get(last + 1)?)
}

fn normalized_indices_for_query(haystack: &str, query_lower: &str) -> Option<Vec<usize>> {
    let haystack_norm = normalized_chars_with_original_indices(haystack);
    let query_norm = normalized_query_chars(query_lower);
    if query_norm.is_empty() || query_norm.len() > haystack_norm.len() {
        return None;
    }

    for start in 0..=(haystack_norm.len() - query_norm.len()) {
        if haystack_norm[start..start + query_norm.len()]
            .iter()
            .map(|(ch, _)| *ch)
            .eq(query_norm.iter().copied())
        {
            let mut indices = haystack_norm[start..start + query_norm.len()]
                .iter()
                .map(|(_, original_index)| *original_index)
                .collect::<Vec<_>>();
            indices.sort_unstable();
            indices.dedup();
            return Some(indices);
        }
    }

    None
}

fn normalized_query_chars(value: &str) -> Vec<char> {
    value
        .chars()
        .flat_map(|ch| fold_search_char(ch).into_iter())
        .collect()
}

fn normalized_chars_with_original_indices(value: &str) -> Vec<(char, usize)> {
    value
        .chars()
        .enumerate()
        .flat_map(|(index, ch)| {
            fold_search_char(ch)
                .into_iter()
                .map(move |folded| (folded, index))
        })
        .collect()
}

fn fold_search_char(ch: char) -> Vec<char> {
    let folded = match ch {
        'À' | 'Á' | 'Â' | 'Ã' | 'Ä' | 'Å' | 'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' => "a",
        'Ç' | 'ç' => "c",
        'È' | 'É' | 'Ê' | 'Ë' | 'è' | 'é' | 'ê' | 'ë' => "e",
        'Ì' | 'Í' | 'Î' | 'Ï' | 'ì' | 'í' | 'î' | 'ï' | 'İ' => "i",
        'Ñ' | 'ñ' => "n",
        'Ò' | 'Ó' | 'Ô' | 'Õ' | 'Ö' | 'Ø' | 'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ø' => "o",
        'Ù' | 'Ú' | 'Û' | 'Ü' | 'ù' | 'ú' | 'û' | 'ü' => "u",
        'Ý' | 'Ÿ' | 'ý' | 'ÿ' => "y",
        'Æ' | 'æ' => "ae",
        'Œ' | 'œ' => "oe",
        'ß' => "ss",
        _ => return ch.to_lowercase().collect(),
    };
    folded.chars().collect()
}

fn byte_pos_is_word_boundary(haystack: &str, byte_pos: usize) -> bool {
    if haystack.is_ascii() {
        return is_word_boundary_match(haystack, byte_pos);
    }

    if byte_pos == 0 {
        return true;
    }
    let mut previous: Option<char> = None;
    for (idx, current) in haystack.char_indices() {
        if idx == byte_pos {
            let Some(previous) = previous else {
                return false;
            };
            return !char::is_alphanumeric(previous)
                || (char::is_lowercase(previous) && char::is_uppercase(current));
        }
        previous = Some(current);
    }
    false
}

fn fuzzy_indices_are_structured_abbreviation(haystack: &str, indices: &[usize]) -> bool {
    let Some(first) = indices.first().copied() else {
        return false;
    };

    if !char_index_is_word_start(haystack, first) {
        return false;
    }

    let mut previous = first;
    let mut run_count = 1;
    for current in indices.iter().copied().skip(1) {
        if current == previous.saturating_add(1) {
            previous = current;
            continue;
        }
        if !char_index_is_word_start(haystack, current) {
            return false;
        }
        run_count += 1;
        previous = current;
    }

    run_count >= 2
}

fn char_index_is_word_start(haystack: &str, char_index: usize) -> bool {
    if char_index == 0 {
        return true;
    }

    let mut previous: Option<char> = None;
    for (index, current) in haystack.chars().enumerate() {
        if index == char_index {
            let Some(previous) = previous else {
                return false;
            };
            return !previous.is_alphanumeric()
                || (previous.is_lowercase() && current.is_uppercase());
        }
        previous = Some(current);
    }

    false
}
