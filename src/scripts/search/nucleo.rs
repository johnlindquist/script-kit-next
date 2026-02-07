use nucleo_matcher::pattern::Pattern;
use nucleo_matcher::{Matcher, Utf32Str};

/// DEPRECATED: Prefer using NucleoCtx::score() to avoid per-call allocations.
#[inline]
pub(crate) fn nucleo_score(
    haystack: &str,
    pattern: &Pattern,
    matcher: &mut Matcher,
) -> Option<u32> {
    let mut haystack_buf = Vec::new();
    let haystack_utf32 = Utf32Str::new(haystack, &mut haystack_buf);
    pattern.score(haystack_utf32, matcher)
}

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
        Self {
            pattern,
            matcher: Matcher::new(nucleo_matcher::Config::DEFAULT),
            buf: Vec::with_capacity(64), // Pre-allocate for typical strings
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
}
