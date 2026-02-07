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

/// Check if a substring match starts at a word boundary in the haystack.
///
/// A word boundary is:
/// - Position 0 (start of string)
/// - After a non-alphanumeric character (space, dash, underscore, etc.)
/// - At a camelCase transition (lowercase followed by uppercase)
///
/// This is used to give bonus points when the query matches at meaningful
/// word starts, making searches more intuitive (e.g., "new" ranks "New Tab"
/// higher than "Renewal").
#[inline]
pub(crate) fn is_word_boundary_match(haystack: &str, match_pos: usize) -> bool {
    if match_pos == 0 {
        return true;
    }
    let bytes = haystack.as_bytes();
    if match_pos >= bytes.len() {
        return false;
    }
    let prev = bytes[match_pos - 1];
    let curr = bytes[match_pos];
    // After non-alphanumeric (space, dash, underscore, dot, slash, etc.)
    if !prev.is_ascii_alphanumeric() {
        return true;
    }
    // camelCase boundary: lowercase followed by uppercase
    if prev.is_ascii_lowercase() && curr.is_ascii_uppercase() {
        return true;
    }
    false
}

/// Check if query is an exact case-insensitive match for the full haystack.
/// Used to give a massive boost when the user types an exact name.
#[inline]
pub(crate) fn is_exact_name_match(haystack: &str, query_lower: &str) -> bool {
    haystack.len() == query_lower.len()
        && haystack
            .as_bytes()
            .iter()
            .zip(query_lower.as_bytes())
            .all(|(h, q)| h.to_ascii_lowercase() == *q)
}

/// Minimum query length for nucleo fuzzy matching.
/// Very short queries (1 char) generate too many low-quality fuzzy matches
/// across all items, reducing result quality. Require at least 2 chars.
pub(crate) const MIN_FUZZY_QUERY_LEN: usize = 2;

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
