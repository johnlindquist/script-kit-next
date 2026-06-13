//! Surface-neutral Spine parse/projection helpers.
//!
//! Lifts the parse + cursor-projection + ownership-decision logic out of the
//! main-menu `ScriptListApp` so other surfaces (Agent Chat composer, future Notes /
//! Quick-Terminal composers) can share the same Spine state machine without
//! duplicating it.

use crate::spine::{
    parse_spine, project_cursor, SpineCursorProjection, SpineParse, SpineSegmentKind,
};

#[derive(Debug, Clone, Default)]
pub(crate) struct SpineInputProjection {
    pub parse: SpineParse,
    pub projection: Option<SpineCursorProjection>,
}

pub(crate) fn byte_offset_for_char_cursor(text: &str, cursor_chars: usize) -> usize {
    text.char_indices()
        .nth(cursor_chars)
        .map(|(byte, _)| byte)
        .unwrap_or(text.len())
}

#[allow(dead_code)]
pub(crate) fn char_cursor_for_byte_offset(text: &str, byte_offset: usize) -> usize {
    let clamped = byte_offset.min(text.len());
    text[..clamped].chars().count()
}

pub(crate) fn project_text_at_char_cursor(text: &str, cursor_chars: usize) -> SpineInputProjection {
    let parse = parse_spine(text);
    let cursor_byte = byte_offset_for_char_cursor(text, cursor_chars);
    let projection = if parse.segments.is_empty() {
        None
    } else {
        Some(project_cursor(&parse, cursor_byte))
    };
    SpineInputProjection { parse, projection }
}

pub(crate) fn project_text_at_byte_cursor(text: &str, cursor_byte: usize) -> SpineInputProjection {
    let parse = parse_spine(text);
    let clamped = cursor_byte.min(text.len());
    let projection = if parse.segments.is_empty() {
        None
    } else {
        Some(project_cursor(&parse, clamped))
    };
    SpineInputProjection { parse, projection }
}

/// Returns `true` when the current cursor projection should drive a Spine
/// list. Only active sigil segments own a list; free-text tails stay plain
/// editor text and use Cmd+Enter for direct handoff.
pub(crate) fn projection_owns_prompt_builder_list(
    projection: Option<&SpineCursorProjection>,
    _parse: &SpineParse,
) -> bool {
    let Some(proj) = projection else {
        return false;
    };
    !matches!(proj.active_segment_kind, SpineSegmentKind::FreeText)
}
