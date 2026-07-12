use nucleo_matcher::pattern::Pattern;
use nucleo_matcher::{Matcher, Utf32Str};

use super::super::types::{
    MatchEvidence, MatchEvidenceField, MatchIndices, ScriptMatchKind, SearchResult,
};
use super::{find_ignore_ascii_case, fuzzy_match_with_indices_ascii, is_ascii_pair};

/// Reusable highlight matcher that keeps ASCII fast-path behavior and
/// falls back to Unicode-safe nucleo indices when needed.
pub(crate) struct SearchHighlightMatchCtx {
    query_lower: String,
    unicode_ctx: Option<UnicodeHighlightCtx>,
}

impl SearchHighlightMatchCtx {
    pub(crate) fn new(query: &str) -> Self {
        Self {
            query_lower: query.trim().to_lowercase(),
            unicode_ctx: None,
        }
    }

    #[inline]
    pub(crate) fn indices_for(&mut self, haystack: &str) -> (bool, Vec<usize>) {
        if self.query_lower.is_empty() {
            return (false, Vec::new());
        }

        if is_ascii_pair(haystack, &self.query_lower) {
            if let Some(start) = find_ignore_ascii_case(haystack, &self.query_lower) {
                let end = start + self.query_lower.len();
                return (true, (start..end).collect());
            }
            return fuzzy_match_with_indices_ascii(haystack, &self.query_lower);
        }

        self.unicode_ctx
            .get_or_insert_with(|| UnicodeHighlightCtx::new(&self.query_lower))
            .indices_for(haystack)
    }

    /// Contiguous-substring-only indices: no character-subsequence fuzzy
    /// fallback. Long-form text rows use this so a sentence query can never
    /// paint scattered per-character highlights.
    pub(crate) fn contiguous_indices_for(&mut self, haystack: &str) -> (bool, Vec<usize>) {
        if self.query_lower.is_empty() {
            return (false, Vec::new());
        }

        if is_ascii_pair(haystack, &self.query_lower) {
            if let Some(start) = find_ignore_ascii_case(haystack, &self.query_lower) {
                let end = start + self.query_lower.len();
                return (true, (start..end).collect());
            }
            return (false, Vec::new());
        }

        let hay: Vec<char> = haystack.chars().flat_map(|ch| ch.to_lowercase()).collect();
        let needle: Vec<char> = self.query_lower.chars().collect();
        if needle.is_empty() || hay.len() < needle.len() {
            return (false, Vec::new());
        }
        for start in 0..=(hay.len() - needle.len()) {
            if hay[start..start + needle.len()] == needle[..] {
                return (true, (start..start + needle.len()).collect());
            }
        }
        (false, Vec::new())
    }

    /// Mode-aware indices for long-form passive rows: sentence queries only
    /// highlight contiguous phrase occurrences, single tokens keep today's
    /// behavior.
    pub(crate) fn long_text_indices_for(
        &mut self,
        haystack: &str,
        sentence_mode: bool,
    ) -> (bool, Vec<usize>) {
        if sentence_mode {
            self.contiguous_indices_for(haystack)
        } else {
            self.indices_for(haystack)
        }
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

fn indices_from_evidence(
    evidence: Option<&MatchEvidence>,
    rendered_name: &str,
    rendered_description: Option<&str>,
    rendered_filename: Option<&str>,
) -> Option<MatchIndices> {
    let evidence = evidence?;
    let mut indices = MatchIndices::default();

    match evidence.field {
        MatchEvidenceField::Name if evidence.text == rendered_name => {
            indices.name_indices = evidence.indices.clone();
        }
        MatchEvidenceField::Description if rendered_description == Some(evidence.text.as_str()) => {
            indices.description_indices = evidence.indices.clone();
        }
        MatchEvidenceField::Filename if rendered_filename == Some(evidence.text.as_str()) => {
            indices.filename_indices = evidence.indices.clone();
        }
        MatchEvidenceField::Content
        | MatchEvidenceField::Alias
        | MatchEvidenceField::Shortcut
        | MatchEvidenceField::Keyword
        | MatchEvidenceField::Source
        | MatchEvidenceField::Tool
        | MatchEvidenceField::WindowApp
        | MatchEvidenceField::SkillId
        | MatchEvidenceField::PluginTitle
        | MatchEvidenceField::Name
        | MatchEvidenceField::Description
        | MatchEvidenceField::Filename => {}
    }

    Some(indices)
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
    if query.trim().is_empty() {
        return MatchIndices::default();
    }

    let mut highlight_ctx = SearchHighlightMatchCtx::new(query);
    // Sentence queries (2+ words) must never fall back to nucleo
    // char-subsequence highlighting on long-form rows.
    let sentence_mode = super::sentence::compile_long_text_query(query)
        .is_some_and(|compiled| compiled.is_sentence());

    match result {
        SearchResult::Script(sm) => {
            if let Some(indices) = indices_from_evidence(
                sm.match_evidence.as_ref(),
                &sm.script.name,
                sm.script.description.as_deref(),
                Some(&sm.filename),
            ) {
                return indices;
            }

            let mut indices = MatchIndices::default();

            match sm.match_kind {
                ScriptMatchKind::Name => {
                    let (name_matched, name_indices) = highlight_ctx.indices_for(&sm.script.name);
                    if name_matched {
                        indices.name_indices = name_indices;
                    }
                }
                ScriptMatchKind::Description => {
                    if let Some(ref desc) = sm.script.description {
                        let (desc_matched, desc_indices) = highlight_ctx.indices_for(desc);
                        if desc_matched {
                            indices.description_indices = desc_indices;
                        }
                    }
                }
                ScriptMatchKind::Filename => {
                    let (filename_matched, filename_indices) =
                        highlight_ctx.indices_for(&sm.filename);
                    if filename_matched {
                        indices.filename_indices = filename_indices;
                    }
                }
                ScriptMatchKind::Content => {}
            }

            indices
        }
        SearchResult::Scriptlet(sm) => {
            if let Some(indices) = indices_from_evidence(
                sm.match_evidence.as_ref(),
                &sm.scriptlet.name,
                sm.scriptlet.description.as_deref(),
                sm.display_file_path.as_deref(),
            ) {
                return indices;
            }

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
            if let Some(indices) = indices_from_evidence(
                bm.match_evidence.as_ref(),
                &bm.entry.name,
                Some(&bm.entry.description),
                None,
            ) {
                return indices;
            }

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
            if let Some(indices) =
                indices_from_evidence(am.match_evidence.as_ref(), &am.app.name, None, None)
            {
                return indices;
            }

            let mut indices = MatchIndices::default();

            let (name_matched, name_indices) = highlight_ctx.indices_for(&am.app.name);
            if name_matched {
                indices.name_indices = name_indices;
            }

            indices
        }
        SearchResult::Window(wm) => {
            if let Some(indices) = indices_from_evidence(
                wm.match_evidence.as_ref(),
                &wm.window.title,
                Some(&wm.subtitle),
                None,
            ) {
                return indices;
            }

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
        SearchResult::File(fm) => {
            let mut indices = MatchIndices::default();

            let (name_matched, name_indices) = highlight_ctx.indices_for(&fm.file.name);
            if name_matched {
                indices.name_indices = name_indices;
            }

            if indices.name_indices.is_empty() {
                let (path_matched, path_indices) = highlight_ctx.indices_for(&fm.file.path);
                if path_matched {
                    indices.filename_indices = path_indices;
                }
            }

            indices
        }
        SearchResult::Note(nm) => {
            let mut indices = MatchIndices::default();

            let (name_matched, name_indices) = highlight_ctx.indices_for(&nm.title);
            if name_matched {
                indices.name_indices = name_indices;
            }

            if indices.name_indices.is_empty() {
                let (desc_matched, desc_indices) = highlight_ctx.indices_for(&nm.subtitle);
                if desc_matched {
                    indices.description_indices = desc_indices;
                }
            }

            indices
        }
        SearchResult::BrainHit(bm) => {
            let mut indices = MatchIndices::default();

            let (name_matched, name_indices) = highlight_ctx.indices_for(&bm.hit.title);
            if name_matched {
                indices.name_indices = name_indices;
            }

            if indices.name_indices.is_empty() {
                let (desc_matched, desc_indices) = highlight_ctx.indices_for(&bm.subtitle);
                if desc_matched {
                    indices.description_indices = desc_indices;
                }
            }

            indices
        }
        SearchResult::BrainInboxItem(bm) => {
            let mut indices = MatchIndices::default();

            let (name_matched, name_indices) = highlight_ctx.indices_for(&bm.item.title);
            if name_matched {
                indices.name_indices = name_indices;
            }

            if indices.name_indices.is_empty() {
                let (desc_matched, desc_indices) = highlight_ctx.indices_for(&bm.subtitle);
                if desc_matched {
                    indices.description_indices = desc_indices;
                }
            }

            indices
        }
        SearchResult::Todo(tm) => {
            let mut indices = MatchIndices::default();

            let (name_matched, name_indices) = highlight_ctx.indices_for(&tm.hit.title);
            if name_matched {
                indices.name_indices = name_indices;
            }

            if indices.name_indices.is_empty() {
                let (desc_matched, desc_indices) = highlight_ctx.indices_for(&tm.hit.subtitle);
                if desc_matched {
                    indices.description_indices = desc_indices;
                }
            }

            if indices.name_indices.is_empty() && indices.description_indices.is_empty() {
                let (body_matched, body_indices) = highlight_ctx.indices_for(&tm.hit.body);
                if body_matched {
                    indices.filename_indices = body_indices;
                }
            }

            indices
        }
        SearchResult::AgentChatHistory(am) => {
            let mut indices = MatchIndices::default();

            // Evidence-first: highlight exactly the word ranges that made
            // the row qualify. The stored source texts guard against
            // composed subtitles that don't start with the matched field.
            if let Some(evidence) = am.evidence.as_ref() {
                let rendered_title = am.entry.title_display();
                if !evidence.title_indices.is_empty()
                    && rendered_title.starts_with(evidence.title_text.as_str())
                {
                    indices.name_indices = evidence.title_indices.clone();
                }
                if !evidence.subtitle_indices.is_empty()
                    && am.subtitle.starts_with(evidence.subtitle_text.as_str())
                {
                    indices.description_indices = evidence.subtitle_indices.clone();
                }
                return indices;
            }

            // No evidence (e.g. empty-query recency rows): contiguous
            // substring only — never scattered fuzzy characters.
            let (name_matched, name_indices) =
                highlight_ctx.contiguous_indices_for(am.entry.title_display());
            if name_matched {
                indices.name_indices = name_indices;
            }

            if indices.name_indices.is_empty() {
                let (preview_matched, preview_indices) =
                    highlight_ctx.contiguous_indices_for(am.entry.preview_display());
                if preview_matched {
                    indices.description_indices = preview_indices;
                }
            }

            indices
        }
        SearchResult::AiVault(am) => {
            let mut indices = MatchIndices::default();

            let (name_matched, name_indices) = highlight_ctx.indices_for(&am.hit.safe_title);
            if name_matched {
                indices.name_indices = name_indices;
            }

            if indices.name_indices.is_empty() {
                let (desc_matched, desc_indices) = highlight_ctx.indices_for(&am.subtitle);
                if desc_matched {
                    indices.description_indices = desc_indices;
                }
            }

            indices
        }
        SearchResult::ClipboardHistory(cm) => {
            let mut indices = MatchIndices::default();

            let (name_matched, name_indices) =
                highlight_ctx.long_text_indices_for(&cm.title, sentence_mode);
            if name_matched {
                indices.name_indices = name_indices;
            }

            if indices.name_indices.is_empty() {
                let (desc_matched, desc_indices) =
                    highlight_ctx.long_text_indices_for(&cm.subtitle, sentence_mode);
                if desc_matched {
                    indices.description_indices = desc_indices;
                }
            }

            indices
        }
        SearchResult::DictationHistory(dm) => {
            let mut indices = MatchIndices::default();

            if let Some(evidence) = dm.evidence.as_ref() {
                if !evidence.title_indices.is_empty()
                    && dm.preview.starts_with(evidence.title_text.as_str())
                {
                    indices.name_indices = evidence.title_indices.clone();
                }
                if !evidence.subtitle_indices.is_empty()
                    && dm.subtitle.starts_with(evidence.subtitle_text.as_str())
                {
                    indices.description_indices = evidence.subtitle_indices.clone();
                }
                return indices;
            }

            let (name_matched, name_indices) = highlight_ctx.contiguous_indices_for(&dm.preview);
            if name_matched {
                indices.name_indices = name_indices;
            }

            if indices.name_indices.is_empty() {
                let (desc_matched, desc_indices) =
                    highlight_ctx.contiguous_indices_for(&dm.subtitle);
                if desc_matched {
                    indices.description_indices = desc_indices;
                }
            }

            indices
        }
        SearchResult::BrowserHistory(bm) => {
            let mut indices = MatchIndices::default();

            let (name_matched, name_indices) =
                highlight_ctx.long_text_indices_for(&bm.hit.title, sentence_mode);
            if name_matched {
                indices.name_indices = name_indices;
            }

            if indices.name_indices.is_empty() {
                let (desc_matched, desc_indices) =
                    highlight_ctx.long_text_indices_for(&bm.subtitle, sentence_mode);
                if desc_matched {
                    indices.description_indices = desc_indices;
                }
            }

            if indices.name_indices.is_empty() && indices.description_indices.is_empty() {
                let (url_matched, url_indices) =
                    highlight_ctx.long_text_indices_for(&bm.hit.url, sentence_mode);
                if url_matched {
                    indices.filename_indices = url_indices;
                }
            }

            indices
        }
        SearchResult::BrowserTab(bm) => {
            let mut indices = MatchIndices::default();

            let (name_matched, name_indices) =
                highlight_ctx.long_text_indices_for(&bm.hit.title, sentence_mode);
            if name_matched {
                indices.name_indices = name_indices;
            }

            if indices.name_indices.is_empty() {
                let (desc_matched, desc_indices) =
                    highlight_ctx.long_text_indices_for(&bm.subtitle, sentence_mode);
                if desc_matched {
                    indices.description_indices = desc_indices;
                }
            }

            if indices.name_indices.is_empty() && indices.description_indices.is_empty() {
                let (url_matched, url_indices) =
                    highlight_ctx.long_text_indices_for(&bm.hit.url, sentence_mode);
                if url_matched {
                    indices.filename_indices = url_indices;
                }
            }

            indices
        }
        SearchResult::Skill(sm) => {
            if let Some(indices) = indices_from_evidence(
                sm.match_evidence.as_ref(),
                &sm.skill.title,
                (!sm.skill.description.is_empty()).then_some(sm.skill.description.as_str()),
                None,
            ) {
                return indices;
            }

            let mut indices = MatchIndices::default();

            let (name_matched, name_indices) = highlight_ctx.indices_for(&sm.skill.title);
            if name_matched {
                indices.name_indices = name_indices;
            }

            if !sm.skill.description.is_empty() {
                let (desc_matched, desc_indices) = highlight_ctx.indices_for(&sm.skill.description);
                if desc_matched {
                    indices.description_indices = desc_indices;
                }
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
            if fm
                .stable_selection_key_override
                .as_deref()
                .is_some_and(|key| key.starts_with("fallback/root-file-search-handoff/"))
            {
                return indices;
            }

            let fallback_label = fm.display_label();

            // Try name match for fallback items
            let (name_matched, name_indices) = highlight_ctx.indices_for(&fallback_label);
            if name_matched {
                indices.name_indices = name_indices;
            }

            indices
        }
        // Script issues row is synthetic and not matched against the query
        SearchResult::ScriptIssue(_) => MatchIndices::default(),
        // Spine projections are not matched against the query
        SearchResult::SpineProjection(_) => MatchIndices::default(),
        // Flow matches carry indices precomputed by the flow fuzzy scorer
        SearchResult::Flow(fm) => fm.match_indices.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::compute_match_indices_for_result;
    use crate::fallbacks::builtins::{BuiltinFallback, FallbackAction, FallbackCondition};
    use crate::fallbacks::FallbackItem;
    use crate::scripts::{FallbackMatch, SearchResult};

    #[test]
    fn fallback_label_highlight_ignores_trailing_query_space() {
        let result = SearchResult::Fallback(
            FallbackMatch::new(
                FallbackItem::Builtin(BuiltinFallback::new(
                    "search-files",
                    "Search Files",
                    "Search for files matching this query",
                    "Search",
                    FallbackAction::SearchFiles,
                    FallbackCondition::Always,
                    10,
                )),
                0,
            )
            .with_display_overrides(
                "Search Files for \"what is codex\"",
                "Open full File Search",
            ),
        );

        let indices = compute_match_indices_for_result(&result, "what is codex ");

        assert!(
            !indices.name_indices.is_empty(),
            "trailing spaces in the input should not clear the fallback row text highlight"
        );
    }

    #[test]
    fn fallback_label_highlight_prefers_contiguous_substring_over_fuzzy_prefix() {
        let result = SearchResult::Fallback(
            FallbackMatch::new(
                FallbackItem::Builtin(BuiltinFallback::new(
                    "create-event",
                    "The event",
                    "Create an event",
                    "Calendar",
                    FallbackAction::SearchFiles,
                    FallbackCondition::Always,
                    10,
                )),
                0,
            )
            .with_display_overrides("The event", "Create an event"),
        );

        let indices = compute_match_indices_for_result(&result, "event");

        assert_eq!(
            indices.name_indices,
            vec![4, 5, 6, 7, 8],
            "contiguous substring should beat earliest fuzzy chars like the 'e' in 'The'"
        );
    }

    #[test]
    fn root_file_handoff_fallback_does_not_highlight_typed_query() {
        let result = SearchResult::Fallback(
            FallbackMatch::new(
                FallbackItem::Builtin(BuiltinFallback::new(
                    "search-files",
                    "Search Files",
                    "Search for files matching this query",
                    "Search",
                    FallbackAction::SearchFiles,
                    FallbackCondition::Always,
                    10,
                )),
                0,
            )
            .with_display_overrides(
                "Search Files for \"why is this\"",
                "Open full File Search · preview matches filename words",
            )
            .with_stable_selection_key("fallback/root-file-search-handoff/global"),
        );

        let indices = compute_match_indices_for_result(&result, "why is this");

        assert!(
            indices.name_indices.is_empty() && indices.description_indices.is_empty(),
            "root file handoff copy changes every keystroke, so it should not flash fuzzy highlights"
        );
    }
}
