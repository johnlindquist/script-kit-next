use std::ops::Range;

use super::{
    list::{SpineListAction, SpineListRow},
    SpineCursorProjection, SpineParse, SpineSegment, SpineSegmentKind, SpineSegmentResolution,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpineInputSpanTone {
    Resolved,
    Unknown,
    Hint,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpineInputSpan {
    pub range: Range<usize>,
    pub tone: SpineInputSpanTone,
    pub is_active: bool,
}

pub fn input_spans_for_parse(
    parse: &SpineParse,
    projection: Option<&SpineCursorProjection>,
) -> Vec<SpineInputSpan> {
    let active_index = projection.map(|p| p.active_segment_index);

    let spans = parse
        .segments
        .iter()
        .enumerate()
        .filter_map(|(index, segment)| {
            let tone = tone_for_segment(index, segment)?;
            Some(SpineInputSpan {
                range: segment.byte_range.clone(),
                tone,
                is_active: active_index == Some(index),
            })
        })
        .collect::<Vec<_>>();

    normalize_spine_input_spans(parse.input.as_str(), spans)
}

/// Generate accent ranges for completed Spine segments.
/// Non-active resolved/hint segments get the full range colored.
/// Context mentions with sub_queries get only the prefix (@file:) colored.
/// Active segments are not colored (the user is currently editing them).
pub fn accent_ranges_for_parse(
    parse: &SpineParse,
    projection: Option<&SpineCursorProjection>,
) -> Vec<(Range<usize>, &'static str)> {
    accent_ranges_for_parse_with_resolved_tokens(parse, projection, &|_| false)
}

/// Like [`accent_ranges_for_parse`], but tokens recognized by
/// `is_resolved_token` (e.g. registered in the spine mention alias registry
/// after the user picked a concrete file/clipboard entry) are treated as
/// completed: the FULL token gets the accent color, even while the cursor
/// sits inside it. In-progress subsearch typing (`@file:quer`) keeps the
/// prefix-only treatment because those tokens are not registered yet.
pub fn accent_ranges_for_parse_with_resolved_tokens(
    parse: &SpineParse,
    projection: Option<&SpineCursorProjection>,
    is_resolved_token: &dyn Fn(&str) -> bool,
) -> Vec<(Range<usize>, &'static str)> {
    let active_index = projection.map(|p| p.active_segment_index);
    let mut ranges = Vec::new();

    for (index, segment) in parse.segments.iter().enumerate() {
        if matches!(segment.kind, SpineSegmentKind::ContextMention { .. })
            && is_resolved_token(segment.raw.trim())
        {
            ranges.push((segment.byte_range.clone(), "spine.context.completed"));
            continue;
        }

        let is_active = active_index == Some(index);
        if is_active {
            // For active context mentions with sub_query, color just the prefix
            if let Some(prefix_range) = context_prefix_byte_range(segment) {
                ranges.push((prefix_range, "spine.context.completed"));
                continue;
            }
            // A completed segment (exact catalog match, e.g. `@selection` or
            // `/rewrite`) is accent colored immediately — completion must not
            // wait for the cursor to move out of the segment.
            if matches!(
                tone_for_segment(index, segment),
                Some(SpineInputSpanTone::Resolved)
            ) {
                if let Some(role) = completed_segment_role(&segment.kind) {
                    ranges.push((segment.byte_range.clone(), role));
                }
            }
            continue;
        }

        let tone = match tone_for_segment(index, segment) {
            Some(t) => t,
            None => continue,
        };

        if !matches!(
            tone,
            SpineInputSpanTone::Resolved | SpineInputSpanTone::Hint
        ) {
            continue;
        }

        // For context mentions with sub_query, only color the prefix
        if let Some(prefix_range) = context_prefix_byte_range(segment) {
            ranges.push((prefix_range, "spine.context.completed"));
            continue;
        }

        let Some(role) = completed_segment_role(&segment.kind) else {
            continue;
        };

        ranges.push((segment.byte_range.clone(), role));
    }

    ranges
}

fn completed_segment_role(kind: &SpineSegmentKind) -> Option<&'static str> {
    match kind {
        SpineSegmentKind::ContextMention { .. } => Some("spine.context.completed"),
        SpineSegmentKind::SlashCommand { .. } => Some("spine.command.completed"),
        SpineSegmentKind::Profile { .. } => Some("spine.profile.completed"),
        SpineSegmentKind::Style { .. } => Some("spine.style.completed"),
        SpineSegmentKind::Capture { .. } => Some("spine.capture.completed"),
        SpineSegmentKind::ProjectCwd { .. } => Some("spine.cwd.completed"),
        // Flow tokens reuse the command role: same "resolved command word"
        // visual language as `/rewrite`.
        SpineSegmentKind::Flow { .. } => Some("spine.command.completed"),
        _ => None,
    }
}

pub fn spine_input_span_role_name(span: &SpineInputSpan) -> &'static str {
    match (span.tone, span.is_active) {
        (SpineInputSpanTone::Resolved, false) => "spineResolved",
        (SpineInputSpanTone::Resolved, true) => "spineResolvedActive",
        (SpineInputSpanTone::Unknown, false) => "spineUnknown",
        (SpineInputSpanTone::Unknown, true) => "spineUnknownActive",
        (SpineInputSpanTone::Hint, false) => "spineHint",
        (SpineInputSpanTone::Hint, true) => "spineHintActive",
    }
}

fn tone_for_segment(segment_index: usize, segment: &SpineSegment) -> Option<SpineInputSpanTone> {
    if !decorates_segment_kind(&segment.kind) {
        return None;
    }
    match &segment.resolution {
        SpineSegmentResolution::Resolved { .. } => Some(SpineInputSpanTone::Resolved),
        SpineSegmentResolution::Unknown { .. } => Some(SpineInputSpanTone::Unknown),
        SpineSegmentResolution::Unresolved => unresolved_segment_tone(segment_index, segment),
    }
}

/// For context mentions with sub_queries (@file:readme), return the byte range
/// of just the `@file:` prefix portion for accent coloring.
pub fn context_prefix_byte_range(segment: &SpineSegment) -> Option<Range<usize>> {
    match &segment.kind {
        SpineSegmentKind::ContextMention {
            context_type,
            sub_query: Some(_),
        } => {
            let prefix_len = 1 + context_type.len() + 1; // @ + type + :
            let start = segment.byte_range.start;
            let end = start + prefix_len;
            if end <= segment.byte_range.end {
                Some(start..end)
            } else {
                None
            }
        }
        _ => None,
    }
}

fn decorates_segment_kind(kind: &SpineSegmentKind) -> bool {
    matches!(
        kind,
        SpineSegmentKind::ContextMention { .. }
            | SpineSegmentKind::SlashCommand { .. }
            | SpineSegmentKind::Profile { .. }
            | SpineSegmentKind::Style { .. }
            | SpineSegmentKind::Capture { .. }
            | SpineSegmentKind::ListFilter { .. }
            | SpineSegmentKind::ProjectCwd { .. }
            | SpineSegmentKind::Flow { .. }
            | SpineSegmentKind::ModeExit { .. }
    )
}

fn unresolved_segment_tone(
    segment_index: usize,
    segment: &SpineSegment,
) -> Option<SpineInputSpanTone> {
    match &segment.kind {
        SpineSegmentKind::ContextMention {
            sub_query: Some(_), ..
        } => Some(SpineInputSpanTone::Hint),
        SpineSegmentKind::ContextMention { .. }
        | SpineSegmentKind::SlashCommand { .. }
        | SpineSegmentKind::Profile { .. }
        | SpineSegmentKind::Style { .. } => match prompt_catalog_match(segment_index, segment) {
            CatalogMatch::Exact => Some(SpineInputSpanTone::Resolved),
            CatalogMatch::Partial => Some(SpineInputSpanTone::Hint),
            CatalogMatch::None if !segment_query(segment).trim().is_empty() => {
                Some(SpineInputSpanTone::Unknown)
            }
            CatalogMatch::None => Some(SpineInputSpanTone::Hint),
        },
        SpineSegmentKind::Capture { .. }
        | SpineSegmentKind::ListFilter { .. }
        | SpineSegmentKind::ProjectCwd { .. }
        | SpineSegmentKind::Flow { .. }
        | SpineSegmentKind::ModeExit { .. } => Some(SpineInputSpanTone::Hint),
        SpineSegmentKind::FreeText => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CatalogMatch {
    Exact,
    Partial,
    None,
}

fn prompt_catalog_match(segment_index: usize, segment: &SpineSegment) -> CatalogMatch {
    let rows = prompt_catalog_rows_for_segment(segment_index, segment);
    if rows.is_empty() {
        return CatalogMatch::None;
    }
    if rows
        .iter()
        .any(|row| row_replacement_matches_segment(row, segment.raw.as_str()))
    {
        return CatalogMatch::Exact;
    }
    CatalogMatch::Partial
}

fn prompt_catalog_rows_for_segment(
    segment_index: usize,
    segment: &SpineSegment,
) -> Vec<SpineListRow> {
    let range = segment.byte_range.clone();
    let query = segment_query(segment);
    match &segment.kind {
        SpineSegmentKind::ContextMention {
            sub_query: None, ..
        } => super::catalog_context::build_context_root_rows(query, segment_index, range),
        SpineSegmentKind::SlashCommand { .. } => {
            super::catalog_slash::build_slash_command_rows(query, segment_index, range)
        }
        SpineSegmentKind::Profile { .. } => {
            super::catalog_profile::build_profile_rows(query, segment_index, range)
        }
        SpineSegmentKind::Style { .. } => {
            // Span-tone matching only inspects row replacements, never the
            // action label, so the auto-submit label variant is irrelevant.
            super::catalog_style::build_style_rows(query, segment_index, range, false)
        }
        _ => Vec::new(),
    }
}

fn row_replacement_matches_segment(row: &SpineListRow, raw: &str) -> bool {
    match &row.action {
        SpineListAction::ResolveSegment { replacement, .. }
        | SpineListAction::InsertSegmentText {
            text: replacement, ..
        } => replacement.as_ref() == raw,
        _ => false,
    }
}

fn segment_query(segment: &SpineSegment) -> &str {
    match &segment.kind {
        SpineSegmentKind::FreeText => segment.raw.as_str(),
        SpineSegmentKind::ContextMention {
            context_type,
            sub_query: None,
        } => context_type.as_str(),
        SpineSegmentKind::ContextMention {
            sub_query: Some(sub_query),
            ..
        } => sub_query.as_str(),
        SpineSegmentKind::SlashCommand { command } => command.as_str(),
        SpineSegmentKind::Flow { query } => query.as_str(),
        SpineSegmentKind::Profile { profile_id } => profile_id.as_str(),
        SpineSegmentKind::Style { style_id } => style_id.as_str(),
        SpineSegmentKind::Capture { target, .. } => target.as_str(),
        SpineSegmentKind::ListFilter { query } => query.as_str(),
        SpineSegmentKind::ProjectCwd {
            sub_query: Some(sq),
        } => sq.as_str(),
        SpineSegmentKind::ProjectCwd { sub_query: None } => "",
        SpineSegmentKind::ModeExit { rest, .. } => rest.as_str(),
    }
}

fn normalize_spine_input_spans(raw: &str, mut spans: Vec<SpineInputSpan>) -> Vec<SpineInputSpan> {
    spans.retain(|span| valid_utf8_range(raw, span.range.clone()) && !span.range.is_empty());
    spans.sort_by(|a, b| {
        a.range
            .start
            .cmp(&b.range.start)
            .then(a.range.end.cmp(&b.range.end))
            .then(b.is_active.cmp(&a.is_active))
            .then(spine_tone_rank(a.tone).cmp(&spine_tone_rank(b.tone)))
    });

    let mut out: Vec<SpineInputSpan> = Vec::new();
    for span in spans {
        if out
            .last()
            .is_some_and(|previous| ranges_overlap(&span.range, &previous.range))
        {
            continue;
        }
        out.push(span);
    }
    out
}

fn spine_tone_rank(tone: SpineInputSpanTone) -> u8 {
    match tone {
        SpineInputSpanTone::Resolved => 0,
        SpineInputSpanTone::Unknown => 1,
        SpineInputSpanTone::Hint => 2,
    }
}

fn ranges_overlap(a: &Range<usize>, b: &Range<usize>) -> bool {
    a.start < b.end && b.start < a.end
}

fn valid_utf8_range(raw: &str, range: Range<usize>) -> bool {
    range.start <= range.end
        && range.end <= raw.len()
        && raw.is_char_boundary(range.start)
        && raw.is_char_boundary(range.end)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spine::{parse_spine, project_cursor};

    fn span_texts<'a>(
        raw: &'a str,
        spans: &'a [SpineInputSpan],
    ) -> Vec<(&'a str, SpineInputSpanTone, bool)> {
        spans
            .iter()
            .map(|span| (&raw[span.range.clone()], span.tone, span.is_active))
            .collect()
    }

    #[test]
    fn exact_prompt_catalog_segments_are_resolved() {
        let raw = "@selection /rewrite";
        let parse = parse_spine(raw);
        let projection = project_cursor(&parse, raw.len());
        let spans = input_spans_for_parse(&parse, Some(&projection));
        let texts = span_texts(raw, &spans);
        assert!(texts.contains(&("@selection", SpineInputSpanTone::Resolved, false)));
        assert!(texts.contains(&("/rewrite", SpineInputSpanTone::Resolved, true)));
    }

    #[test]
    fn unknown_prompt_segment_gets_warning_tone() {
        let raw = "@madeup";
        let parse = parse_spine(raw);
        let projection = project_cursor(&parse, raw.len());
        let spans = input_spans_for_parse(&parse, Some(&projection));
        assert_eq!(
            span_texts(raw, &spans),
            vec![("@madeup", SpineInputSpanTone::Unknown, true)]
        );
    }

    #[test]
    fn partial_prompt_segment_is_hint_not_unknown() {
        let raw = "/rew";
        let parse = parse_spine(raw);
        let projection = project_cursor(&parse, raw.len());
        let spans = input_spans_for_parse(&parse, Some(&projection));
        assert_eq!(
            span_texts(raw, &spans),
            vec![("/rew", SpineInputSpanTone::Hint, true)]
        );
    }

    #[test]
    fn active_segment_moves_with_projection() {
        let raw = "@selection /rewrite";
        let parse = parse_spine(raw);

        let first_projection = project_cursor(&parse, 2);
        let first_spans = input_spans_for_parse(&parse, Some(&first_projection));
        assert!(span_texts(raw, &first_spans).contains(&(
            "@selection",
            SpineInputSpanTone::Resolved,
            true
        )));

        let slash_cursor = raw.find("/rewrite").unwrap() + 2;
        let second_projection = project_cursor(&parse, slash_cursor);
        let second_spans = input_spans_for_parse(&parse, Some(&second_projection));
        assert!(span_texts(raw, &second_spans).contains(&(
            "/rewrite",
            SpineInputSpanTone::Resolved,
            true
        )));
    }

    #[test]
    fn spans_are_valid_utf8_boundaries() {
        let raw =
            "@file:\u{65e5}\u{672c}\u{8a9e} /rewrite \u{6587}\u{7ae0}\u{3092}\u{4fee}\u{6b63}";
        let parse = parse_spine(raw);
        let projection = project_cursor(&parse, raw.len());
        for span in input_spans_for_parse(&parse, Some(&projection)) {
            assert!(raw.is_char_boundary(span.range.start), "{span:?}");
            assert!(raw.is_char_boundary(span.range.end), "{span:?}");
        }
    }

    #[test]
    fn resolved_file_token_gets_full_range_accent() {
        // After a file is selected, the compact token is registered in the
        // mention alias registry; the WHOLE token must be accent colored,
        // even with the cursor at the end of the input (active segment).
        let token = "@file:AddPass_Icon%402x.png";
        for raw in [token.to_string(), format!("{token} ")] {
            let parse = parse_spine(&raw);
            let projection = project_cursor(&parse, raw.len());
            let ranges = accent_ranges_for_parse_with_resolved_tokens(
                &parse,
                Some(&projection),
                &|candidate| candidate == token,
            );
            assert_eq!(ranges.len(), 1, "raw={raw:?} ranges={ranges:?}");
            assert_eq!(&raw[ranges[0].0.clone()], token, "raw={raw:?}");
            assert_eq!(ranges[0].1, "spine.context.completed");
        }
    }

    #[test]
    fn active_exact_match_mention_gets_full_range_accent() {
        // `@selection ` with the cursor at the end is a completed sigil; the
        // accent must not wait for the cursor to leave the segment.
        let raw = "@selection ";
        let parse = parse_spine(raw);
        let projection = project_cursor(&parse, raw.len());
        let ranges = accent_ranges_for_parse(&parse, Some(&projection));
        assert_eq!(ranges.len(), 1, "ranges={ranges:?}");
        assert_eq!(&raw[ranges[0].0.clone()], "@selection");
        assert_eq!(ranges[0].1, "spine.context.completed");
    }

    #[test]
    fn active_partial_mention_has_no_accent() {
        // Mid-typing (`@selec`) is not complete — no accent yet.
        let raw = "@selec";
        let parse = parse_spine(raw);
        let projection = project_cursor(&parse, raw.len());
        let ranges = accent_ranges_for_parse(&parse, Some(&projection));
        assert!(ranges.is_empty(), "ranges={ranges:?}");
    }

    #[test]
    fn unresolved_file_subsearch_keeps_prefix_only_accent() {
        // While the user is still typing a subsearch query the token is not
        // registered, so only the `@file:` prefix is accent colored.
        let raw = "@file:addpa";
        let parse = parse_spine(raw);
        let projection = project_cursor(&parse, raw.len());
        let ranges =
            accent_ranges_for_parse_with_resolved_tokens(&parse, Some(&projection), &|_| false);
        assert_eq!(ranges.len(), 1, "ranges={ranges:?}");
        assert_eq!(&raw[ranges[0].0.clone()], "@file:");
    }

    #[test]
    fn plain_text_has_no_spans() {
        let raw = "hello world";
        let parse = parse_spine(raw);
        let projection = project_cursor(&parse, raw.len());
        assert!(input_spans_for_parse(&parse, Some(&projection)).is_empty());
    }
}
