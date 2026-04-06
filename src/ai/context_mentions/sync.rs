use crate::ai::message_parts::AiContextPart;
use std::collections::HashSet;
use std::ops::Range;

/// The result of reconciling inline `@mention` tokens in composer text
/// against the currently attached context parts and ownership set.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct InlineMentionSyncPlan {
    /// All context parts that the current text *desires* (one per resolved mention).
    pub(crate) desired_parts: Vec<AiContextPart>,
    /// Canonical tokens present in the current text.
    pub(crate) desired_tokens: HashSet<String>,
    /// Indices into the attached-parts slice that should be removed (stale).
    pub(crate) stale_indices: Vec<usize>,
    /// Parts that are new and should be added to the attachment list.
    pub(crate) added_parts: Vec<AiContextPart>,
    /// Canonical tokens for the newly added parts.
    pub(crate) added_tokens: Vec<String>,
}

fn char_to_byte_offset(text: &str, char_idx: usize) -> usize {
    text.char_indices()
        .nth(char_idx)
        .map(|(byte_idx, _)| byte_idx)
        .unwrap_or(text.len())
}

/// Replace a character-level range in `text` with `replacement`.
pub(crate) fn replace_text_in_char_range(
    text: &str,
    range: Range<usize>,
    replacement: &str,
) -> String {
    let start_byte = char_to_byte_offset(text, range.start);
    let end_byte = char_to_byte_offset(text, range.end);
    let mut next = String::with_capacity(text.len() - (end_byte - start_byte) + replacement.len());
    next.push_str(&text[..start_byte]);
    next.push_str(replacement);
    next.push_str(&text[end_byte..]);
    next
}

/// Compute the caret position (in chars) after a replacement.
pub(crate) fn caret_after_replacement(range: &Range<usize>, replacement: &str) -> usize {
    range.start + replacement.chars().count()
}

/// Decide whether accepting an inline picker item should claim ownership
/// of the resulting canonical token.
///
/// Returns:
/// - `false` when the part has no inline token,
/// - `true` when the token is already owned by inline text,
/// - `false` when the same part is already attached from a non-inline source,
/// - `true` otherwise (brand-new inline attachment).
pub(crate) fn should_claim_inline_mention_ownership(
    part: &AiContextPart,
    attached_parts: &[AiContextPart],
    inline_owned_tokens: &HashSet<String>,
) -> bool {
    let Some(token) = super::part_to_inline_token(part) else {
        return false;
    };

    if inline_owned_tokens.contains(&token) {
        return true;
    }

    !attached_parts.iter().any(|existing| existing == part)
}

/// Build a plan that describes which parts to add, which to remove, and what
/// the desired ownership set should look like — without mutating any state.
///
/// Both ACP and the AI window call this then apply the plan to their own
/// state/thread structures.
pub(crate) fn build_inline_mention_sync_plan(
    text: &str,
    attached_parts: &[AiContextPart],
    inline_owned_tokens: &HashSet<String>,
) -> InlineMentionSyncPlan {
    let parsed = super::parse_inline_context_mentions(text);

    let desired_parts: Vec<AiContextPart> = parsed.iter().map(|m| m.part.clone()).collect();
    let desired_tokens: HashSet<String> =
        parsed.iter().map(|m| m.canonical_token.clone()).collect();

    // Indices of attached parts whose owning token has disappeared from text.
    let stale_indices: Vec<usize> = attached_parts
        .iter()
        .enumerate()
        .filter_map(|(ix, part)| {
            let token = super::part_to_inline_token(part)?;
            (inline_owned_tokens.contains(&token) && !desired_tokens.contains(&token)).then_some(ix)
        })
        .collect();

    // Parts for tokens that are in the text but not yet attached.
    let existing_tokens: HashSet<String> = attached_parts
        .iter()
        .filter_map(super::part_to_inline_token)
        .collect();

    let mut added_parts = Vec::new();
    let mut added_tokens = Vec::new();
    for mention in &parsed {
        if existing_tokens.contains(&mention.canonical_token)
            || added_tokens.contains(&mention.canonical_token)
        {
            continue;
        }
        added_tokens.push(mention.canonical_token.clone());
        added_parts.push(mention.part.clone());
    }

    tracing::info!(
        target: "ai",
        event = "inline_mention_sync_plan_built",
        desired_count = desired_parts.len(),
        stale_count = stale_indices.len(),
        added_count = added_parts.len(),
        desired_tokens = ?desired_tokens,
    );

    InlineMentionSyncPlan {
        desired_parts,
        desired_tokens,
        stale_indices,
        added_parts,
        added_tokens,
    }
}

/// Return the indices of attached parts that should be shown as visible chips
/// (i.e. those NOT already represented by an inline `@mention` token).
pub(crate) fn visible_context_chip_indices(
    text: &str,
    attached_parts: &[AiContextPart],
) -> Vec<usize> {
    let inline_tokens: HashSet<String> = super::parse_inline_context_mentions(text)
        .into_iter()
        .map(|m| m.canonical_token)
        .collect();

    let indices: Vec<usize> = attached_parts
        .iter()
        .enumerate()
        .filter_map(|(ix, part)| match super::part_to_inline_token(part) {
            Some(token) if inline_tokens.contains(&token) => None,
            _ => Some(ix),
        })
        .collect();

    tracing::debug!(
        target: "ai",
        event = "visible_context_chip_indices_computed",
        visible_count = indices.len(),
        attached_count = attached_parts.len(),
    );

    indices
}

/// Atomically remove an inline mention at the cursor position, consuming one
/// trailing space when present. Returns `(new_text, new_cursor)` or `None`
/// if the cursor is not inside/adjacent to a mention.
pub(crate) fn remove_inline_mention_at_cursor(
    text: &str,
    cursor: usize,
    delete_forward: bool,
) -> Option<(String, usize)> {
    let range = super::mention_range_for_atomic_delete(text, cursor, delete_forward)?;

    let chars: Vec<char> = text.chars().collect();
    let mut end_char = range.end;
    // Consume one trailing space when present.
    if chars.get(end_char) == Some(&' ') {
        end_char += 1;
    }

    let start_byte = char_to_byte_offset(text, range.start);
    let end_byte = char_to_byte_offset(text, end_char);

    let mut next = String::with_capacity(text.len() - (end_byte - start_byte));
    next.push_str(&text[..start_byte]);
    next.push_str(&text[end_byte..]);

    Some((next, range.start))
}
