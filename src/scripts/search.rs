//! Fuzzy search functionality for scripts, scriptlets, and other items
//!
//! This module provides fuzzy search functions using nucleo for high-performance
//! matching and scoring, plus ASCII case-folding helpers for efficiency.

use std::cmp::Ordering;
use std::sync::Arc;

use nucleo_matcher::pattern::Pattern;
use nucleo_matcher::{Matcher, Utf32Str};

use crate::app_launcher::AppInfo;
use crate::builtins::BuiltInEntry;
use crate::window_control::WindowInfo;

use super::types::{
    AppMatch, BuiltInMatch, MatchIndices, Script, ScriptMatch, Scriptlet, ScriptletMatch,
    SearchResult, WindowMatch,
};

// ============================================
// ASCII CASE-FOLDING HELPERS (Performance-optimized)
// ============================================
// These functions avoid heap allocations by doing case-insensitive
// comparisons byte-by-byte instead of calling to_lowercase().
//
// IMPORTANT: These functions ONLY work correctly for ASCII text. For Unicode,
// they degrade to case-sensitive matching. Use the is_ascii_pair() helper to
// gate their usage, or rely on nucleo for Unicode-safe fuzzy matching.

/// Check if both strings are ASCII, enabling ASCII fast-path optimizations.
/// When both are ASCII, we can use byte-level case-insensitive comparison.
/// When either contains non-ASCII, we should rely on nucleo for matching.
#[inline]
pub(crate) fn is_ascii_pair(a: &str, b: &str) -> bool {
    a.is_ascii() && b.is_ascii()
}

/// Check if haystack contains needle using ASCII case-insensitive matching.
/// `needle_lower` must already be lowercase.
/// Returns true if needle is found anywhere in haystack.
/// No allocation - O(n*m) worst case but typically much faster.
///
/// WARNING: Only use when both strings are ASCII (check with is_ascii_pair()).
/// For non-ASCII text, this degrades to case-sensitive matching.
#[inline]
pub(crate) fn contains_ignore_ascii_case(haystack: &str, needle_lower: &str) -> bool {
    let h = haystack.as_bytes();
    let n = needle_lower.as_bytes();
    if n.is_empty() {
        return true;
    }
    if n.len() > h.len() {
        return false;
    }
    'outer: for i in 0..=(h.len() - n.len()) {
        for j in 0..n.len() {
            if h[i + j].to_ascii_lowercase() != n[j] {
                continue 'outer;
            }
        }
        return true;
    }
    false
}

/// Find the position of needle in haystack using ASCII case-insensitive matching.
/// `needle_lower` must already be lowercase.
/// Returns Some(position) if found, None otherwise.
/// No allocation - O(n*m) worst case.
///
/// WARNING: Only use when both strings are ASCII (check with is_ascii_pair()).
/// For non-ASCII text, this degrades to case-sensitive matching.
#[inline]
pub(crate) fn find_ignore_ascii_case(haystack: &str, needle_lower: &str) -> Option<usize> {
    let h = haystack.as_bytes();
    let n = needle_lower.as_bytes();
    if n.is_empty() {
        return Some(0);
    }
    if n.len() > h.len() {
        return None;
    }
    'outer: for i in 0..=(h.len() - n.len()) {
        for j in 0..n.len() {
            if h[i + j].to_ascii_lowercase() != n[j] {
                continue 'outer;
            }
        }
        return Some(i);
    }
    None
}

/// Perform fuzzy matching without allocating a lowercase copy of haystack.
/// `pattern_lower` must already be lowercase.
/// Returns (matched, indices) where matched is true if all pattern chars found in order.
/// The indices are positions in the original haystack.
#[inline]
pub(crate) fn fuzzy_match_with_indices_ascii(
    haystack: &str,
    pattern_lower: &str,
) -> (bool, Vec<usize>) {
    let mut indices = Vec::new();
    let mut pattern_chars = pattern_lower.chars().peekable();

    for (idx, ch) in haystack.chars().enumerate() {
        if let Some(&p) = pattern_chars.peek() {
            if ch.to_ascii_lowercase() == p {
                indices.push(idx);
                pattern_chars.next();
            }
        }
    }

    let matched = pattern_chars.peek().is_none();
    (matched, if matched { indices } else { Vec::new() })
}

/// Check if a pattern is a fuzzy match for haystack (characters appear in order)
#[allow(dead_code)]
pub(crate) fn is_fuzzy_match(haystack: &str, pattern: &str) -> bool {
    let mut pattern_chars = pattern.chars().peekable();
    for ch in haystack.chars() {
        if let Some(&p) = pattern_chars.peek() {
            if ch.eq_ignore_ascii_case(&p) {
                pattern_chars.next();
            }
        }
    }
    pattern_chars.peek().is_none()
}

/// Perform fuzzy matching and return the indices of matched characters
/// Returns (matched, indices) where matched is true if all pattern chars found in order
#[allow(dead_code)]
pub(crate) fn fuzzy_match_with_indices(haystack: &str, pattern: &str) -> (bool, Vec<usize>) {
    let mut indices = Vec::new();
    let mut pattern_chars = pattern.chars().peekable();

    for (idx, ch) in haystack.chars().enumerate() {
        if let Some(&p) = pattern_chars.peek() {
            if ch.eq_ignore_ascii_case(&p) {
                indices.push(idx);
                pattern_chars.next();
            }
        }
    }

    let matched = pattern_chars.peek().is_none();
    (matched, if matched { indices } else { Vec::new() })
}

/// Score a haystack against a nucleo pattern.
/// Returns Some(score) if the pattern matches, None otherwise.
/// Score range is typically 0-1000+ (higher = better match).
///
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
pub(crate) struct NucleoCtx {
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

    let query_lower = query.to_lowercase();

    match result {
        SearchResult::Script(sm) => {
            let mut indices = MatchIndices::default();

            // Try name first
            let (name_matched, name_indices) =
                fuzzy_match_with_indices_ascii(&sm.script.name, &query_lower);
            if name_matched {
                indices.name_indices = name_indices;
                return indices;
            }

            // Fall back to filename
            let (filename_matched, filename_indices) =
                fuzzy_match_with_indices_ascii(&sm.filename, &query_lower);
            if filename_matched {
                indices.filename_indices = filename_indices;
            }

            indices
        }
        SearchResult::Scriptlet(sm) => {
            let mut indices = MatchIndices::default();

            // Try name first
            let (name_matched, name_indices) =
                fuzzy_match_with_indices_ascii(&sm.scriptlet.name, &query_lower);
            if name_matched {
                indices.name_indices = name_indices;
                return indices;
            }

            // Fall back to file path
            if let Some(ref fp) = sm.display_file_path {
                let (fp_matched, fp_indices) = fuzzy_match_with_indices_ascii(fp, &query_lower);
                if fp_matched {
                    indices.filename_indices = fp_indices;
                }
            }

            indices
        }
        SearchResult::BuiltIn(bm) => {
            let mut indices = MatchIndices::default();

            let (name_matched, name_indices) =
                fuzzy_match_with_indices_ascii(&bm.entry.name, &query_lower);
            if name_matched {
                indices.name_indices = name_indices;
            }

            indices
        }
        SearchResult::App(am) => {
            let mut indices = MatchIndices::default();

            let (name_matched, name_indices) =
                fuzzy_match_with_indices_ascii(&am.app.name, &query_lower);
            if name_matched {
                indices.name_indices = name_indices;
            }

            indices
        }
        SearchResult::Window(wm) => {
            let mut indices = MatchIndices::default();

            // Try app name first, then title
            let (app_matched, app_indices) =
                fuzzy_match_with_indices_ascii(&wm.window.app, &query_lower);
            if app_matched {
                indices.name_indices = app_indices;
                return indices;
            }

            let (title_matched, title_indices) =
                fuzzy_match_with_indices_ascii(&wm.window.title, &query_lower);
            if title_matched {
                indices.filename_indices = title_indices;
            }

            indices
        }
        SearchResult::Agent(am) => {
            let mut indices = MatchIndices::default();

            // Try name first
            let (name_matched, name_indices) =
                fuzzy_match_with_indices_ascii(&am.agent.name, &query_lower);
            if name_matched {
                indices.name_indices = name_indices;
                return indices;
            }

            // Fall back to description
            if let Some(ref desc) = am.agent.description {
                let (desc_matched, desc_indices) =
                    fuzzy_match_with_indices_ascii(desc, &query_lower);
                if desc_matched {
                    indices.filename_indices = desc_indices;
                }
            }

            indices
        }
    }
}

/// Extract filename from a path for display
pub(crate) fn extract_filename(path: &std::path::Path) -> String {
    path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string()
}

/// Extract display-friendly file path from scriptlet file_path
/// Converts "/path/to/file.md#slug" to "file.md#slug"
pub(crate) fn extract_scriptlet_display_path(file_path: &Option<String>) -> Option<String> {
    file_path.as_ref().map(|fp| {
        // Split on # to get path and anchor
        let parts: Vec<&str> = fp.splitn(2, '#').collect();
        let path_part = parts[0];
        let anchor = parts.get(1);

        // Extract just the filename from the path
        let filename = std::path::Path::new(path_part)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(path_part);

        // Reconstruct with anchor if present
        match anchor {
            Some(a) => format!("{}#{}", filename, a),
            None => filename.to_string(),
        }
    })
}

/// Fuzzy search scripts by query string
/// Searches across name, filename (e.g., "my-script.ts"), description, and path
/// Returns results sorted by relevance score (highest first)
/// Match indices are provided to enable UI highlighting of matched characters
///
/// H1 Optimization: Accepts Arc<Script> to avoid expensive clones during filter operations.
/// Each ScriptMatch contains an Arc::clone which is just a refcount bump.
pub fn fuzzy_search_scripts(scripts: &[Arc<Script>], query: &str) -> Vec<ScriptMatch> {
    if query.is_empty() {
        // If no query, return all scripts with equal score, sorted by name
        return scripts
            .iter()
            .map(|s| {
                let filename = extract_filename(&s.path);
                ScriptMatch {
                    script: Arc::clone(s),
                    score: 0,
                    filename,
                    match_indices: MatchIndices::default(),
                }
            })
            .collect();
    }

    let query_lower = query.to_lowercase();
    let mut matches = Vec::new();

    // Create nucleo context once for all scripts - reuses buffer across calls
    let mut nucleo = NucleoCtx::new(&query_lower);
    // Check if query is ASCII once for all items
    let query_is_ascii = query_lower.is_ascii();

    for script in scripts {
        let mut score = 0i32;
        // Lazy match indices - don't compute during scoring, will be computed on-demand
        let match_indices = MatchIndices::default();

        let filename = extract_filename(&script.path);

        // Score by name match - highest priority
        // Only use ASCII fast-path when both strings are ASCII
        if query_is_ascii && script.name.is_ascii() {
            if let Some(pos) = find_ignore_ascii_case(&script.name, &query_lower) {
                // Bonus for exact substring match at start of name
                score += if pos == 0 { 100 } else { 75 };
            }
        }

        // Fuzzy character matching in name using nucleo (handles Unicode correctly)
        if let Some(nucleo_s) = nucleo.score(&script.name) {
            // Scale nucleo score (0-1000+) to match existing weights (~50 for fuzzy match)
            score += 50 + (nucleo_s / 20) as i32;
        }

        // Score by filename match - high priority (allows searching by ".ts", ".js", etc.)
        // Filenames are typically ASCII
        if query_is_ascii && filename.is_ascii() {
            if let Some(pos) = find_ignore_ascii_case(&filename, &query_lower) {
                // Bonus for exact substring match at start of filename
                score += if pos == 0 { 60 } else { 45 };
            }
        }

        // Fuzzy character matching in filename using nucleo (handles Unicode)
        if let Some(nucleo_s) = nucleo.score(&filename) {
            // Scale nucleo score to match existing weights (~35 for filename fuzzy match)
            score += 35 + (nucleo_s / 30) as i32;
        }

        // Score by description match - medium priority
        // Only use ASCII fast-path when both are ASCII
        if let Some(ref desc) = script.description {
            if query_is_ascii && desc.is_ascii() && contains_ignore_ascii_case(desc, &query_lower) {
                score += 25;
            }
        }

        // Score by path match - lower priority
        // Paths are typically ASCII
        let path_str = script.path.to_string_lossy();
        if query_is_ascii
            && path_str.is_ascii()
            && contains_ignore_ascii_case(&path_str, &query_lower)
        {
            score += 10;
        }

        if score > 0 {
            matches.push(ScriptMatch {
                script: Arc::clone(script),
                score,
                filename,
                match_indices,
            });
        }
    }

    // Sort by score (highest first), then by name for ties
    matches.sort_by(|a, b| match b.score.cmp(&a.score) {
        Ordering::Equal => a.script.name.cmp(&b.script.name),
        other => other,
    });

    matches
}

/// Fuzzy search scriptlets by query string
/// Searches across name, file_path with anchor (e.g., "url.md#open-github"), description, and code
/// Returns results sorted by relevance score (highest first)
/// Match indices are provided to enable UI highlighting of matched characters
///
/// H1 Optimization: Accepts Arc<Scriptlet> to avoid expensive clones during filter operations.
/// Each ScriptletMatch contains an Arc::clone which is just a refcount bump.
pub fn fuzzy_search_scriptlets(scriptlets: &[Arc<Scriptlet>], query: &str) -> Vec<ScriptletMatch> {
    if query.is_empty() {
        // If no query, return all scriptlets with equal score, sorted by name
        return scriptlets
            .iter()
            .map(|s| {
                let display_file_path = extract_scriptlet_display_path(&s.file_path);
                ScriptletMatch {
                    scriptlet: Arc::clone(s),
                    score: 0,
                    display_file_path,
                    match_indices: MatchIndices::default(),
                }
            })
            .collect();
    }

    let query_lower = query.to_lowercase();
    let mut matches = Vec::new();

    // Create nucleo context once for all scriptlets - reuses buffer across calls
    let mut nucleo = NucleoCtx::new(&query_lower);
    // Check if query is ASCII once for all items
    let query_is_ascii = query_lower.is_ascii();

    for scriptlet in scriptlets {
        let mut score = 0i32;
        // Lazy match indices - don't compute during scoring
        let match_indices = MatchIndices::default();

        let display_file_path = extract_scriptlet_display_path(&scriptlet.file_path);

        // Score by name match - highest priority
        // Only use ASCII fast-path when both strings are ASCII
        if query_is_ascii && scriptlet.name.is_ascii() {
            if let Some(pos) = find_ignore_ascii_case(&scriptlet.name, &query_lower) {
                // Bonus for exact substring match at start of name
                score += if pos == 0 { 100 } else { 75 };
            }
        }

        // Fuzzy character matching in name using nucleo (handles Unicode)
        if let Some(nucleo_s) = nucleo.score(&scriptlet.name) {
            // Scale nucleo score to match existing weights (~50 for fuzzy match)
            score += 50 + (nucleo_s / 20) as i32;
        }

        // Score by file_path match - high priority (allows searching by ".md", anchor names)
        if let Some(ref fp) = display_file_path {
            // File paths are typically ASCII
            if query_is_ascii && fp.is_ascii() {
                if let Some(pos) = find_ignore_ascii_case(fp, &query_lower) {
                    // Bonus for exact substring match at start of file_path
                    score += if pos == 0 { 60 } else { 45 };
                }
            }

            // Fuzzy character matching in file_path using nucleo (handles Unicode)
            if let Some(nucleo_s) = nucleo.score(fp) {
                // Scale nucleo score to match existing weights (~35 for file_path fuzzy match)
                score += 35 + (nucleo_s / 30) as i32;
            }
        }

        // Score by description match - medium priority
        // Only use ASCII fast-path when both are ASCII
        if let Some(ref desc) = scriptlet.description {
            if query_is_ascii && desc.is_ascii() && contains_ignore_ascii_case(desc, &query_lower) {
                score += 25;
            }
        }

        // CRITICAL OPTIMIZATION: Only search code when query is long enough (>=4 chars)
        // and no other matches were found. Code search is the biggest performance hit
        // because scriptlet.code can be very large.
        // Code is typically ASCII
        if query_lower.len() >= 4
            && score == 0
            && query_is_ascii
            && scriptlet.code.is_ascii()
            && contains_ignore_ascii_case(&scriptlet.code, &query_lower)
        {
            score += 5;
        }

        // Bonus for tool type match
        // Tool types are ASCII (snippet, template, etc.)
        if query_is_ascii
            && scriptlet.tool.is_ascii()
            && contains_ignore_ascii_case(&scriptlet.tool, &query_lower)
        {
            score += 10;
        }

        if score > 0 {
            matches.push(ScriptletMatch {
                scriptlet: Arc::clone(scriptlet),
                score,
                display_file_path,
                match_indices,
            });
        }
    }

    // Sort by score (highest first), then by name for ties
    matches.sort_by(|a, b| match b.score.cmp(&a.score) {
        Ordering::Equal => a.scriptlet.name.cmp(&b.scriptlet.name),
        other => other,
    });

    matches
}

/// Fuzzy search built-in entries by query string
/// Searches across name, description, and keywords
/// Returns results sorted by relevance score (highest first)
///
/// Scoring priorities (name matches are weighted MUCH higher than keywords):
/// - Name prefix match (starts with query): 200 points
/// - Name substring match (contains query): 150 points
/// - Name fuzzy match (nucleo): 100 + scaled nucleo score
/// - Description match: 25 points
/// - Keyword exact match: 40 points (much lower than name)
/// - Keyword fuzzy match: 20 + scaled nucleo score
pub fn fuzzy_search_builtins(entries: &[BuiltInEntry], query: &str) -> Vec<BuiltInMatch> {
    if query.is_empty() {
        // If no query, return all entries with equal score, sorted by name
        return entries
            .iter()
            .map(|e| BuiltInMatch {
                entry: e.clone(),
                score: 0,
            })
            .collect();
    }

    let query_lower = query.to_lowercase();
    let mut matches = Vec::new();

    // Create nucleo context once for all entries - reuses buffer across calls
    let mut nucleo = NucleoCtx::new(&query_lower);
    // Check if query is ASCII once for all items
    // Note: Built-in names, descriptions, and keywords are typically ASCII
    let query_is_ascii = query_lower.is_ascii();

    for entry in entries {
        let mut score = 0i32;
        let mut name_matched = false;

        // Score by name match - HIGHEST priority (significantly higher than keywords)
        // Built-in names are ASCII
        if query_is_ascii && entry.name.is_ascii() {
            if let Some(pos) = find_ignore_ascii_case(&entry.name, &query_lower) {
                // Bonus for exact substring match at start of name (prefix match is best)
                score += if pos == 0 { 200 } else { 150 };
                name_matched = true;
            }
        }

        // Fuzzy character matching in name using nucleo (handles Unicode)
        if let Some(nucleo_s) = nucleo.score(&entry.name) {
            // Scale nucleo score - name fuzzy matches are worth more than keyword matches
            score += 100 + (nucleo_s / 15) as i32;
            name_matched = true;
        }

        // Score by description match - medium priority
        // Built-in descriptions are ASCII
        if query_is_ascii
            && entry.description.is_ascii()
            && contains_ignore_ascii_case(&entry.description, &query_lower)
        {
            score += 25;
        }

        // Score by keyword match - LOWER priority than name matches
        // Keywords help find items but shouldn't outrank name matches
        // Keywords are ASCII
        if query_is_ascii {
            for keyword in &entry.keywords {
                if keyword.is_ascii() && contains_ignore_ascii_case(keyword, &query_lower) {
                    // Keyword matches are worth less than name matches
                    // This ensures "Scratch Pad" (name match) beats "Lock Screen" (keyword match on "screen")
                    score += 40;
                    break; // Only count once even if multiple keywords match
                }
            }
        }

        // Fuzzy match on keywords using nucleo (handles Unicode)
        // Only add keyword fuzzy score if we didn't already match the name well
        // This prevents keywords from inflating scores when the name is already a good match
        if !name_matched {
            for keyword in &entry.keywords {
                if let Some(nucleo_s) = nucleo.score(keyword) {
                    // Scale nucleo score - keyword fuzzy is worth less than name fuzzy
                    score += 20 + (nucleo_s / 40) as i32;
                    break; // Only count once
                }
            }
        }

        if score > 0 {
            matches.push(BuiltInMatch {
                entry: entry.clone(),
                score,
            });
        }
    }

    // Sort by score (highest first), then by name for ties
    matches.sort_by(|a, b| match b.score.cmp(&a.score) {
        Ordering::Equal => a.entry.name.cmp(&b.entry.name),
        other => other,
    });

    matches
}

/// Fuzzy search applications by query string
/// Searches across name and bundle_id
/// Returns results sorted by relevance score (highest first)
pub fn fuzzy_search_apps(apps: &[AppInfo], query: &str) -> Vec<AppMatch> {
    if query.is_empty() {
        // If no query, return all apps with equal score, sorted by name
        return apps
            .iter()
            .map(|a| AppMatch {
                app: a.clone(),
                score: 0,
            })
            .collect();
    }

    let query_lower = query.to_lowercase();
    let mut matches = Vec::new();

    // Create nucleo context once for all apps - reuses buffer across calls
    let mut nucleo = NucleoCtx::new(&query_lower);
    // Check if query is ASCII once for all items
    let query_is_ascii = query_lower.is_ascii();

    for app in apps {
        let mut score = 0i32;

        // Score by name match - highest priority
        // App names can have Unicode (e.g., "日本語アプリ")
        if query_is_ascii && app.name.is_ascii() {
            if let Some(pos) = find_ignore_ascii_case(&app.name, &query_lower) {
                // Bonus for exact substring match at start of name
                score += if pos == 0 { 100 } else { 75 };
            }
        }

        // Fuzzy character matching in name using nucleo (handles Unicode)
        if let Some(nucleo_s) = nucleo.score(&app.name) {
            // Scale nucleo score to match existing weights (~50 for fuzzy match)
            score += 50 + (nucleo_s / 20) as i32;
        }

        // Score by bundle_id match - lower priority
        // Bundle IDs are always ASCII (e.g., "com.apple.Safari")
        if let Some(ref bundle_id) = app.bundle_id {
            if query_is_ascii
                && bundle_id.is_ascii()
                && contains_ignore_ascii_case(bundle_id, &query_lower)
            {
                score += 15;
            }
        }

        // Score by path match - lowest priority
        // Paths are typically ASCII
        let path_str = app.path.to_string_lossy();
        if query_is_ascii
            && path_str.is_ascii()
            && contains_ignore_ascii_case(&path_str, &query_lower)
        {
            score += 5;
        }

        if score > 0 {
            matches.push(AppMatch {
                app: app.clone(),
                score,
            });
        }
    }

    // Sort by score (highest first), then by name for ties
    matches.sort_by(|a, b| match b.score.cmp(&a.score) {
        Ordering::Equal => a.app.name.cmp(&b.app.name),
        other => other,
    });

    matches
}

/// Fuzzy search windows by query string
/// Searches across app name and window title
/// Returns results sorted by relevance score (highest first)
///
/// Scoring priorities:
/// - App name match at start: 100 points
/// - App name match elsewhere: 75 points
/// - Window title match at start: 90 points  
/// - Window title match elsewhere: 65 points
/// - Fuzzy match on app name: 50 points
/// - Fuzzy match on window title: 40 points
pub fn fuzzy_search_windows(windows: &[WindowInfo], query: &str) -> Vec<WindowMatch> {
    if query.is_empty() {
        // If no query, return all windows with equal score, sorted by app name then title
        let mut matches: Vec<WindowMatch> = windows
            .iter()
            .map(|w| WindowMatch {
                window: w.clone(),
                score: 0,
            })
            .collect();
        matches.sort_by(|a, b| match a.window.app.cmp(&b.window.app) {
            Ordering::Equal => a.window.title.cmp(&b.window.title),
            other => other,
        });
        return matches;
    }

    let query_lower = query.to_lowercase();
    let mut matches = Vec::new();

    // Create nucleo context once for all windows - reuses buffer across calls
    let mut nucleo = NucleoCtx::new(&query_lower);
    // Check if query is ASCII once for all items
    let query_is_ascii = query_lower.is_ascii();

    for window in windows {
        let mut score = 0i32;

        // Score by app name match - highest priority
        // App names can have Unicode
        if query_is_ascii && window.app.is_ascii() {
            if let Some(pos) = find_ignore_ascii_case(&window.app, &query_lower) {
                // Bonus for exact substring match at start of app name
                score += if pos == 0 { 100 } else { 75 };
            }
        }

        // Score by window title match - high priority
        // Window titles can have Unicode content
        if query_is_ascii && window.title.is_ascii() {
            if let Some(pos) = find_ignore_ascii_case(&window.title, &query_lower) {
                // Bonus for exact substring match at start of title
                score += if pos == 0 { 90 } else { 65 };
            }
        }

        // Fuzzy character matching in app name using nucleo (handles Unicode)
        if let Some(nucleo_s) = nucleo.score(&window.app) {
            // Scale nucleo score to match existing weights (~50 for app name fuzzy match)
            score += 50 + (nucleo_s / 20) as i32;
        }

        // Fuzzy character matching in window title using nucleo (handles Unicode)
        if let Some(nucleo_s) = nucleo.score(&window.title) {
            // Scale nucleo score to match existing weights (~40 for title fuzzy match)
            score += 40 + (nucleo_s / 25) as i32;
        }

        if score > 0 {
            matches.push(WindowMatch {
                window: window.clone(),
                score,
            });
        }
    }

    // Sort by score (highest first), then by app name, then by title for ties
    matches.sort_by(|a, b| match b.score.cmp(&a.score) {
        Ordering::Equal => match a.window.app.cmp(&b.window.app) {
            Ordering::Equal => a.window.title.cmp(&b.window.title),
            other => other,
        },
        other => other,
    });

    matches
}

/// Perform unified fuzzy search across scripts, scriptlets, and built-ins
/// Returns combined and ranked results sorted by relevance
/// Built-ins appear at the TOP of results (before scripts) when scores are equal
///
/// H1 Optimization: Accepts Arc<Script> and Arc<Scriptlet> to avoid expensive clones.
pub fn fuzzy_search_unified(
    scripts: &[Arc<Script>],
    scriptlets: &[Arc<Scriptlet>],
    query: &str,
) -> Vec<SearchResult> {
    fuzzy_search_unified_with_builtins(scripts, scriptlets, &[], query)
}

/// Perform unified fuzzy search across scripts, scriptlets, and built-ins
/// Returns combined and ranked results sorted by relevance
/// Built-ins appear at the TOP of results (before scripts) when scores are equal
///
/// H1 Optimization: Accepts Arc<Script> and Arc<Scriptlet> to avoid expensive clones.
pub fn fuzzy_search_unified_with_builtins(
    scripts: &[Arc<Script>],
    scriptlets: &[Arc<Scriptlet>],
    builtins: &[BuiltInEntry],
    query: &str,
) -> Vec<SearchResult> {
    // Use the new function with empty apps list for backwards compatibility
    fuzzy_search_unified_all(scripts, scriptlets, builtins, &[], query)
}

/// Perform unified fuzzy search across scripts, scriptlets, built-ins, and apps
/// Returns combined and ranked results sorted by relevance
/// Built-ins appear at the TOP of results (before scripts) when scores are equal
/// Apps appear after built-ins but before scripts when scores are equal
///
/// H1 Optimization: Accepts Arc<Script> and Arc<Scriptlet> to avoid expensive clones.
pub fn fuzzy_search_unified_all(
    scripts: &[Arc<Script>],
    scriptlets: &[Arc<Scriptlet>],
    builtins: &[BuiltInEntry],
    apps: &[AppInfo],
    query: &str,
) -> Vec<SearchResult> {
    let mut results = Vec::new();

    // Search built-ins first (they should appear at top when scores are equal)
    let builtin_matches = fuzzy_search_builtins(builtins, query);
    for bm in builtin_matches {
        results.push(SearchResult::BuiltIn(bm));
    }

    // Search apps (appear after built-ins but before scripts)
    let app_matches = fuzzy_search_apps(apps, query);
    for am in app_matches {
        results.push(SearchResult::App(am));
    }

    // Search scripts
    let script_matches = fuzzy_search_scripts(scripts, query);
    for sm in script_matches {
        results.push(SearchResult::Script(sm));
    }

    // Search scriptlets
    let scriptlet_matches = fuzzy_search_scriptlets(scriptlets, query);
    for sm in scriptlet_matches {
        results.push(SearchResult::Scriptlet(sm));
    }

    // Sort by score (highest first), then by type (builtins first, apps, windows, scripts, scriptlets, agents), then by name
    results.sort_by(|a, b| {
        match b.score().cmp(&a.score()) {
            Ordering::Equal => {
                // Prefer builtins over apps over windows over scripts over scriptlets over agents when scores are equal
                let type_order = |r: &SearchResult| -> i32 {
                    match r {
                        SearchResult::BuiltIn(_) => 0, // Built-ins first
                        SearchResult::App(_) => 1,     // Apps second
                        SearchResult::Window(_) => 2,  // Windows third
                        SearchResult::Script(_) => 3,
                        SearchResult::Scriptlet(_) => 4,
                        SearchResult::Agent(_) => 5, // Agents last
                    }
                };
                let type_order_a = type_order(a);
                let type_order_b = type_order(b);
                match type_order_a.cmp(&type_order_b) {
                    Ordering::Equal => a.name().cmp(b.name()),
                    other => other,
                }
            }
            other => other,
        }
    });

    results
}

/// Perform unified fuzzy search across scripts, scriptlets, built-ins, apps, and windows
/// Returns combined and ranked results sorted by relevance
/// Order by type when scores are equal: Built-ins > Apps > Windows > Scripts > Scriptlets
///
/// H1 Optimization: Accepts Arc<Script> and Arc<Scriptlet> to avoid expensive clones.
pub fn fuzzy_search_unified_with_windows(
    scripts: &[Arc<Script>],
    scriptlets: &[Arc<Scriptlet>],
    builtins: &[BuiltInEntry],
    apps: &[AppInfo],
    windows: &[WindowInfo],
    query: &str,
) -> Vec<SearchResult> {
    let mut results = Vec::new();

    // Search built-ins first (they should appear at top when scores are equal)
    let builtin_matches = fuzzy_search_builtins(builtins, query);
    for bm in builtin_matches {
        results.push(SearchResult::BuiltIn(bm));
    }

    // Search apps (appear after built-ins)
    let app_matches = fuzzy_search_apps(apps, query);
    for am in app_matches {
        results.push(SearchResult::App(am));
    }

    // Search windows (appear after apps)
    let window_matches = fuzzy_search_windows(windows, query);
    for wm in window_matches {
        results.push(SearchResult::Window(wm));
    }

    // Search scripts
    let script_matches = fuzzy_search_scripts(scripts, query);
    for sm in script_matches {
        results.push(SearchResult::Script(sm));
    }

    // Search scriptlets
    let scriptlet_matches = fuzzy_search_scriptlets(scriptlets, query);
    for sm in scriptlet_matches {
        results.push(SearchResult::Scriptlet(sm));
    }

    // Sort by score (highest first), then by type (builtins first, apps, windows, scripts, scriptlets, agents), then by name
    results.sort_by(|a, b| {
        match b.score().cmp(&a.score()) {
            Ordering::Equal => {
                // Prefer builtins over apps over windows over scripts over scriptlets over agents when scores are equal
                let type_order = |r: &SearchResult| -> i32 {
                    match r {
                        SearchResult::BuiltIn(_) => 0, // Built-ins first
                        SearchResult::App(_) => 1,     // Apps second
                        SearchResult::Window(_) => 2,  // Windows third
                        SearchResult::Script(_) => 3,
                        SearchResult::Scriptlet(_) => 4,
                        SearchResult::Agent(_) => 5, // Agents last
                    }
                };
                let type_order_a = type_order(a);
                let type_order_b = type_order(b);
                match type_order_a.cmp(&type_order_b) {
                    Ordering::Equal => a.name().cmp(b.name()),
                    other => other,
                }
            }
            other => other,
        }
    });

    results
}
