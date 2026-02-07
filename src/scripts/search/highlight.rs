use nucleo_matcher::pattern::Pattern;
use nucleo_matcher::{Matcher, Utf32Str};

use super::super::types::{MatchIndices, SearchResult};
use super::{fuzzy_match_with_indices_ascii, is_ascii_pair};

/// Reusable highlight matcher that keeps ASCII fast-path behavior and
/// falls back to Unicode-safe nucleo indices when needed.
struct SearchHighlightMatchCtx {
    query_lower: String,
    unicode_ctx: Option<UnicodeHighlightCtx>,
}

impl SearchHighlightMatchCtx {
    fn new(query: &str) -> Self {
        Self {
            query_lower: query.to_lowercase(),
            unicode_ctx: None,
        }
    }

    #[inline]
    fn indices_for(&mut self, haystack: &str) -> (bool, Vec<usize>) {
        if self.query_lower.is_empty() {
            return (false, Vec::new());
        }

        if is_ascii_pair(haystack, &self.query_lower) {
            return fuzzy_match_with_indices_ascii(haystack, &self.query_lower);
        }

        self.unicode_ctx
            .get_or_insert_with(|| UnicodeHighlightCtx::new(&self.query_lower))
            .indices_for(haystack)
    }
}

/// Unicode-safe fuzzy index matcher backed by nucleo Pattern::indices.
struct UnicodeHighlightCtx {
    pattern: Pattern,
    matcher: Matcher,
    haystack_buf: Vec<char>,
    indices_buf: Vec<u32>,
}

impl UnicodeHighlightCtx {
    fn new(query_lower: &str) -> Self {
        Self {
            pattern: Pattern::parse(
                query_lower,
                nucleo_matcher::pattern::CaseMatching::Ignore,
                nucleo_matcher::pattern::Normalization::Smart,
            ),
            matcher: Matcher::new(nucleo_matcher::Config::DEFAULT),
            haystack_buf: Vec::with_capacity(64),
            indices_buf: Vec::with_capacity(query_lower.chars().count()),
        }
    }

    #[inline]
    fn indices_for(&mut self, haystack: &str) -> (bool, Vec<usize>) {
        self.haystack_buf.clear();
        self.indices_buf.clear();

        let utf32 = Utf32Str::new(haystack, &mut self.haystack_buf);
        if self
            .pattern
            .indices(utf32, &mut self.matcher, &mut self.indices_buf)
            .is_none()
        {
            return (false, Vec::new());
        }

        // Pattern::indices can append unsorted duplicates when multiple atoms
        // contribute. Normalize once before passing to rendering.
        self.indices_buf.sort_unstable();
        self.indices_buf.dedup();

        let mut indices = Vec::with_capacity(self.indices_buf.len());
        indices.extend(self.indices_buf.iter().map(|idx| *idx as usize));
        (true, indices)
    }
}

/// Compute match indices for a search result on-demand (lazy evaluation)
///
/// This function is called by the UI layer only for visible rows, avoiding
/// the cost of computing indices for all results during the scoring phase.
///
/// # Arguments
/// * `result` - The search result to compute indices for
/// * `query` - The original search query (will be lowercased internally)
///
/// # Returns
/// MatchIndices containing the character positions that match the query
pub fn compute_match_indices_for_result(result: &SearchResult, query: &str) -> MatchIndices {
    if query.is_empty() {
        return MatchIndices::default();
    }

    let mut highlight_ctx = SearchHighlightMatchCtx::new(query);

    match result {
        SearchResult::Script(sm) => {
            let mut indices = MatchIndices::default();

            // Try name first
            let (name_matched, name_indices) = highlight_ctx.indices_for(&sm.script.name);
            if name_matched {
                indices.name_indices = name_indices;
            }

            // Also compute description indices for highlighting
            if let Some(ref desc) = sm.script.description {
                let (desc_matched, desc_indices) = highlight_ctx.indices_for(desc);
                if desc_matched {
                    indices.description_indices = desc_indices;
                }
            }

            // If name didn't match, fall back to filename
            if indices.name_indices.is_empty() {
                let (filename_matched, filename_indices) = highlight_ctx.indices_for(&sm.filename);
                if filename_matched {
                    indices.filename_indices = filename_indices;
                }
            }

            indices
        }
        SearchResult::Scriptlet(sm) => {
            let mut indices = MatchIndices::default();

            // Try name first
            let (name_matched, name_indices) = highlight_ctx.indices_for(&sm.scriptlet.name);
            if name_matched {
                indices.name_indices = name_indices;
            }

            // Also compute description indices for highlighting
            if let Some(ref desc) = sm.scriptlet.description {
                let (desc_matched, desc_indices) = highlight_ctx.indices_for(desc);
                if desc_matched {
                    indices.description_indices = desc_indices;
                }
            }

            // If name didn't match, fall back to file path
            if indices.name_indices.is_empty() {
                if let Some(ref fp) = sm.display_file_path {
                    let (fp_matched, fp_indices) = highlight_ctx.indices_for(fp);
                    if fp_matched {
                        indices.filename_indices = fp_indices;
                    }
                }
            }

            indices
        }
        SearchResult::BuiltIn(bm) => {
            let mut indices = MatchIndices::default();

            let (name_matched, name_indices) = highlight_ctx.indices_for(&bm.entry.name);
            if name_matched {
                indices.name_indices = name_indices;
            }

            // Also compute description indices for highlighting
            let (desc_matched, desc_indices) = highlight_ctx.indices_for(&bm.entry.description);
            if desc_matched {
                indices.description_indices = desc_indices;
            }

            indices
        }
        SearchResult::App(am) => {
            let mut indices = MatchIndices::default();

            let (name_matched, name_indices) = highlight_ctx.indices_for(&am.app.name);
            if name_matched {
                indices.name_indices = name_indices;
            }

            indices
        }
        SearchResult::Window(wm) => {
            let mut indices = MatchIndices::default();

            // Try app name first, then title
            let (app_matched, app_indices) = highlight_ctx.indices_for(&wm.window.app);
            if app_matched {
                indices.name_indices = app_indices;
                return indices;
            }

            let (title_matched, title_indices) = highlight_ctx.indices_for(&wm.window.title);
            if title_matched {
                indices.filename_indices = title_indices;
            }

            indices
        }
        SearchResult::Agent(am) => {
            let mut indices = MatchIndices::default();

            // Try name first
            let (name_matched, name_indices) = highlight_ctx.indices_for(&am.agent.name);
            if name_matched {
                indices.name_indices = name_indices;
                return indices;
            }

            // Fall back to description
            if let Some(ref desc) = am.agent.description {
                let (desc_matched, desc_indices) = highlight_ctx.indices_for(desc);
                if desc_matched {
                    indices.filename_indices = desc_indices;
                }
            }

            indices
        }
        SearchResult::Fallback(fm) => {
            let mut indices = MatchIndices::default();

            // Try name match for fallback items
            let (name_matched, name_indices) = highlight_ctx.indices_for(fm.fallback.name());
            if name_matched {
                indices.name_indices = name_indices;
            }

            indices
        }
    }
}
