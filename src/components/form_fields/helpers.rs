// --- Text indexing helpers (char-indexed cursor/selection) --------------------

pub(crate) fn char_len(s: &str) -> usize {
    s.chars().count()
}

/// Convert a character index (0..=char_len) into a byte index (0..=s.len()).
/// If char_idx is past the end, returns s.len().
pub(crate) fn byte_idx_from_char_idx(s: &str, char_idx: usize) -> usize {
    if char_idx == 0 {
        return 0;
    }
    s.char_indices()
        .nth(char_idx)
        .map(|(byte_idx, _)| byte_idx)
        .unwrap_or_else(|| s.len())
}

/// Remove a char range [start_char, end_char) from a String (char indices).
pub(crate) fn drain_char_range(s: &mut String, start_char: usize, end_char: usize) {
    let start_b = byte_idx_from_char_idx(s, start_char);
    let end_b = byte_idx_from_char_idx(s, end_char);
    if start_b < end_b && start_b <= s.len() && end_b <= s.len() {
        s.drain(start_b..end_b);
    }
}

/// Slice a &str by char indices [start_char, end_char).
pub(crate) fn slice_by_char_range(s: &str, start_char: usize, end_char: usize) -> &str {
    let start_b = byte_idx_from_char_idx(s, start_char);
    let end_b = byte_idx_from_char_idx(s, end_char);
    &s[start_b..end_b]
}

fn is_partial_number_value(candidate: &str) -> bool {
    if candidate.is_empty() {
        return true;
    }

    let mut chars = candidate.chars().peekable();
    if matches!(chars.peek(), Some('+' | '-')) {
        chars.next();
    }

    let mut saw_decimal = false;
    for ch in chars {
        match ch {
            '0'..='9' => {}
            '.' if !saw_decimal => {
                saw_decimal = true;
            }
            _ => return false,
        }
    }

    true
}

fn is_partial_email_value(candidate: &str) -> bool {
    let mut at_sign_count = 0usize;
    for ch in candidate.chars() {
        if ch.is_control() || ch.is_whitespace() {
            return false;
        }
        if ch == '@' {
            at_sign_count += 1;
            if at_sign_count > 1 {
                return false;
            }
        }
    }
    true
}

pub(crate) fn form_field_type_allows_candidate_value(
    field_type: Option<&str>,
    candidate: &str,
) -> bool {
    match field_type {
        Some(field_type) if field_type.eq_ignore_ascii_case("number") => {
            is_partial_number_value(candidate)
        }
        Some(field_type) if field_type.eq_ignore_ascii_case("email") => {
            is_partial_email_value(candidate)
        }
        _ => true,
    }
}

// Tunables for click-to-position. These are *approximations*.
#[allow(dead_code)]
pub(crate) const TEXTFIELD_CHAR_WIDTH_PX: f32 = 8.0;
#[allow(dead_code)]
pub(crate) const TEXTAREA_LINE_HEIGHT_PX: f32 = 24.0;
#[allow(dead_code)]
pub(crate) const INPUT_PADDING_X_PX: f32 = 12.0;
#[allow(dead_code)]
pub(crate) const TEXTAREA_PADDING_Y_PX: f32 = 8.0;
