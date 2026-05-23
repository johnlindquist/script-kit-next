use nucleo_matcher::pattern::Pattern;
use nucleo_matcher::{Matcher, Utf32Str};

/// Context for nucleo fuzzy matching that reuses allocations across calls.
///
/// This struct is designed for hot-path scoring where avoiding allocations
/// is critical (e.g., searching thousands of scripts per keystroke).
///
/// Usage:
/// ```ignore
/// let mut ctx = NucleoCtx::new(query);
/// for item in items {
///     if let Some(score) = ctx.score(&item.name) {
///         // matched with score
///     }
/// }
/// ```
pub struct NucleoCtx {
    pattern: Pattern,
    matcher: Matcher,
    buf: Vec<char>,
    indices_buf: Vec<u32>,
}

impl NucleoCtx {
    /// Create a new NucleoCtx for the given query string.
    /// The query is parsed with case-insensitive matching and smart normalization.
    pub fn new(query: &str) -> Self {
        let pattern = Pattern::parse(
            query,
            nucleo_matcher::pattern::CaseMatching::Ignore,
            nucleo_matcher::pattern::Normalization::Smart,
        );
        let indices_cap = query.chars().count();
        Self {
            pattern,
            matcher: Matcher::new(nucleo_matcher::Config::DEFAULT),
            buf: Vec::with_capacity(64), // Pre-allocate for typical strings
            indices_buf: Vec::with_capacity(indices_cap),
        }
    }

    /// Score a haystack string against this context's pattern.
    /// Returns Some(score) if matched, None otherwise.
    /// Reuses internal buffer to avoid allocations.
    #[inline]
    pub fn score(&mut self, haystack: &str) -> Option<u32> {
        self.buf.clear();
        let utf32 = Utf32Str::new(haystack, &mut self.buf);
        self.pattern.score(utf32, &mut self.matcher)
    }

    /// Score a haystack only when the fuzzy hit is compact enough to be useful
    /// in the launcher. This keeps ordered-but-sparse matches from turning
    /// unrelated long labels into candidates while preserving direct
    /// substring/prefix paths in the caller.
    #[inline]
    pub fn compact_score(&mut self, haystack: &str, query: &str) -> Option<u32> {
        let score = self.score(haystack)?;

        if fuzzy_match_is_compact(haystack, query, &self.indices(haystack)?) {
            Some(score)
        } else {
            None
        }
    }

    /// Score a haystack and return the matched character indices.
    /// Returns None if no match. Indices are sorted and deduplicated.
    #[inline]
    pub fn indices(&mut self, haystack: &str) -> Option<Vec<usize>> {
        self.buf.clear();
        self.indices_buf.clear();
        let utf32 = Utf32Str::new(haystack, &mut self.buf);
        self.pattern
            .indices(utf32, &mut self.matcher, &mut self.indices_buf)?;
        self.indices_buf.sort_unstable();
        self.indices_buf.dedup();
        Some(self.indices_buf.iter().map(|idx| *idx as usize).collect())
    }
}

fn fuzzy_match_is_compact(haystack: &str, query: &str, indices: &[usize]) -> bool {
    let query_len = query.chars().count();
    if query_len < 4 {
        return true;
    }

    let Some(first) = indices.first().copied() else {
        return false;
    };
    let Some(last) = indices.last().copied() else {
        return false;
    };

    let span = last.saturating_sub(first).saturating_add(1);
    if span <= query_len.saturating_add(1) {
        return true;
    }

    fuzzy_match_is_structured_abbreviation(haystack, indices)
}

fn fuzzy_match_is_structured_abbreviation(haystack: &str, indices: &[usize]) -> bool {
    let Some(first) = indices.first().copied() else {
        return false;
    };

    if !is_word_start(haystack, first) {
        return false;
    }

    let mut run_count = 1;
    let mut previous = first;

    for current in indices.iter().copied().skip(1) {
        if current == previous.saturating_add(1) {
            previous = current;
            continue;
        }

        if !is_word_start(haystack, current) {
            return false;
        }

        run_count += 1;
        previous = current;
    }

    run_count >= 2
}

fn is_word_start(haystack: &str, char_index: usize) -> bool {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn compact_match(query: &str, haystack: &str) -> bool {
        let mut ctx = NucleoCtx::new(query);
        ctx.compact_score(haystack, query).is_some()
    }

    #[test]
    fn compact_score_keeps_word_contiguous_match() {
        assert!(compact_match("posit", "Reset Window Positions"));
    }

    #[test]
    fn compact_score_rejects_sparse_permission_assistant_match() {
        assert!(!compact_match(
            "posit",
            "Accessibility Permission Assistant"
        ));
    }

    #[test]
    fn compact_score_rejects_sparse_description_match() {
        assert!(!compact_match(
            "posit",
            "Open the Permission Assistant for Screen Recording"
        ));
    }

    #[test]
    fn compact_score_rejects_single_word_ordered_letters() {
        assert!(!compact_match("posi", "Change Tone (Professional)"));
    }

    #[test]
    fn compact_score_rejects_mid_word_sparse_chunks() {
        assert!(!compact_match("posi", "AirPort Base Station Agent"));
        assert!(!compact_match("posi", "PeopleMessageService"));
        assert!(!compact_match(
            "posi",
            "Initialize git and sync the Script Kit workspace to GitHub"
        ));
    }

    #[test]
    fn compact_score_preserves_common_compact_abbreviations() {
        assert!(compact_match("gcal", "Google Calendar"));
    }

    #[test]
    fn compact_score_leaves_very_short_queries_unchanged() {
        assert!(compact_match("gc", "Google Calendar"));
    }
}
