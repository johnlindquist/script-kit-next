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
