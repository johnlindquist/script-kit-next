use chrono::{Duration, NaiveDate, NaiveDateTime, NaiveTime, TimeZone};

use crate::menu_syntax::date::{DateGranularity, MenuSyntaxClock, ResolvedDate};
use crate::menu_syntax::fragments::{
    MenuSyntaxFragment, MenuSyntaxFragmentRole, MenuSyntaxFragmentStatus,
};
use crate::menu_syntax::nl_anchor::{next_weekday_date, ResolvedAnchor};
use crate::menu_syntax::nl_phrase::{ConsumedTokens, NlParseOptions, NlToken, ParseHit};
use crate::menu_syntax::nl_recurrence::ResolvedRecurrencePhrase;
use crate::menu_syntax::payload::DateRole;

#[derive(Debug, Clone, Copy)]
pub(super) struct ParsedTime {
    pub hour: u32,
    pub minute: u32,
    pub meridiem: Option<Meridiem>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Meridiem {
    Am,
    Pm,
}

#[derive(Debug, Clone)]
pub(super) struct ResolvedTimePhrase {
    pub date: ResolvedDate,
}

pub(super) fn parse_time_or_range(
    tokens: &[NlToken],
    consumed: &ConsumedTokens,
    anchor: Option<&ResolvedAnchor>,
    recurrence: Option<&ResolvedRecurrencePhrase>,
    clock: &MenuSyntaxClock,
    options: &NlParseOptions,
) -> Option<ParseHit<ResolvedTimePhrase>> {
    if options.date_range {
        if let Some(hit) = parse_range(tokens, consumed, anchor, recurrence, clock) {
            return Some(hit);
        }
    }
    if options.time {
        parse_single_time(tokens, consumed, anchor, recurrence, clock)
    } else {
        None
    }
}

fn parse_range(
    tokens: &[NlToken],
    consumed: &ConsumedTokens,
    anchor: Option<&ResolvedAnchor>,
    recurrence: Option<&ResolvedRecurrencePhrase>,
    clock: &MenuSyntaxClock,
) -> Option<ParseHit<ResolvedTimePhrase>> {
    let (start_idx, connector_idx, end_idx, ignored_idx) = find_range(tokens, consumed)?;
    let start = parse_time_token_or_shorthand(&tokens[start_idx].lower)?;
    let end = parse_time_token_or_shorthand(&tokens[end_idx].lower)?;
    let date_anchor = date_for_time(anchor, recurrence, start, clock);
    let (start_dt, end_dt) = resolve_time_range(date_anchor, start, end, clock);
    let time_span_start = ignored_idx
        .map(|idx| tokens[idx].span.0)
        .unwrap_or(tokens[start_idx].span.0);
    let anchor_absorbed = anchor
        .filter(|anchor| {
            anchor
                .token_indices
                .iter()
                .all(|idx| !consumed.is_consumed(*idx))
        })
        .filter(|anchor| anchor.source_span.1 <= tokens[start_idx].span.0);
    let span_start = anchor_absorbed
        .map(|anchor| anchor.source_span.0)
        .unwrap_or(time_span_start);
    let span = (span_start, tokens[end_idx].span.1);
    let source = token_span_source(tokens, span);
    let mut consumed_tokens = vec![start_idx, connector_idx, end_idx];
    if let Some(idx) = ignored_idx {
        consumed_tokens.push(idx);
    }
    if let Some(anchor) = anchor_absorbed {
        consumed_tokens.extend(anchor.token_indices.iter().copied());
    }
    Some(ParseHit {
        value: ResolvedTimePhrase {
            date: resolved_date(source.clone(), span, start_dt, Some(end_dt), clock),
        },
        fragment: fragment(MenuSyntaxFragmentRole::DateRange, source, span),
        consumed: consumed_tokens,
    })
}

fn parse_single_time(
    tokens: &[NlToken],
    consumed: &ConsumedTokens,
    anchor: Option<&ResolvedAnchor>,
    recurrence: Option<&ResolvedRecurrencePhrase>,
    clock: &MenuSyntaxClock,
) -> Option<ParseHit<ResolvedTimePhrase>> {
    let (idx, time, ignored_idx) = find_single_time(
        tokens,
        consumed,
        anchor.and_then(|a| a.token_indices.last().copied()),
    )?;
    let date_anchor = date_for_time(anchor, recurrence, time, clock);
    let start_dt = local_dt(
        date_anchor,
        materialize_hour(time, None),
        time.minute,
        clock,
    );
    let time_span_start = ignored_idx
        .map(|idx| tokens[idx].span.0)
        .unwrap_or(tokens[idx].span.0);
    let anchor_absorbed = anchor
        .filter(|anchor| {
            anchor
                .token_indices
                .iter()
                .all(|idx| !consumed.is_consumed(*idx))
        })
        .filter(|anchor| anchor.source_span.1 <= tokens[idx].span.0);
    let span_start = anchor_absorbed
        .map(|anchor| anchor.source_span.0)
        .unwrap_or(time_span_start);
    let span = (span_start, tokens[idx].span.1);
    let source = token_span_source(tokens, span);
    let mut consumed_tokens = vec![idx];
    if let Some(idx) = ignored_idx {
        consumed_tokens.push(idx);
    }
    if let Some(anchor) = anchor_absorbed {
        consumed_tokens.extend(anchor.token_indices.iter().copied());
    }
    Some(ParseHit {
        value: ResolvedTimePhrase {
            date: resolved_date(source.clone(), span, start_dt, None, clock),
        },
        fragment: fragment(MenuSyntaxFragmentRole::Date, source, span),
        consumed: consumed_tokens,
    })
}

fn date_for_time(
    anchor: Option<&ResolvedAnchor>,
    recurrence: Option<&ResolvedRecurrencePhrase>,
    time: ParsedTime,
    clock: &MenuSyntaxClock,
) -> NaiveDate {
    if let Some(anchor) = anchor {
        return anchor.date.date_naive();
    }
    if let Some(recurrence) = recurrence {
        if let Some(first) = recurrence.anchor_weekdays.first().copied() {
            return next_weekday_date(clock, first);
        }
        let hour = materialize_hour(time, None);
        let candidate = local_dt(clock.now.date_naive(), hour, time.minute, clock);
        if candidate <= clock.now {
            return clock.now.date_naive() + Duration::days(1);
        }
    }
    clock.now.date_naive()
}

pub(super) fn parse_time_token(source: &str) -> Option<ParsedTime> {
    let (body, meridiem) = if let Some(head) = source.strip_suffix("am") {
        (head, Some(Meridiem::Am))
    } else if let Some(head) = source.strip_suffix("pm") {
        (head, Some(Meridiem::Pm))
    } else {
        (source, None)
    };
    if body.is_empty() {
        return None;
    }
    let (hour, minute) = if let Some((hour, minute)) = body.split_once(':') {
        (hour.parse::<u32>().ok()?, minute.parse::<u32>().ok()?)
    } else {
        (body.parse::<u32>().ok()?, 0)
    };
    if hour > 23 || minute > 59 {
        return None;
    }
    if meridiem.is_some() && !(1..=12).contains(&hour) {
        return None;
    }
    Some(ParsedTime {
        hour,
        minute,
        meridiem,
    })
}

fn find_range(
    tokens: &[NlToken],
    consumed: &ConsumedTokens,
) -> Option<(usize, usize, usize, Option<usize>)> {
    for connector_idx in 0..tokens.len() {
        if consumed.is_consumed(connector_idx) || !is_range_connector(&tokens[connector_idx].lower)
        {
            continue;
        }
        if connector_idx == 0 || connector_idx + 1 >= tokens.len() {
            continue;
        }
        let mut start_idx = connector_idx - 1;
        let mut ignored_idx = None;
        if tokens[start_idx].lower == "from" && connector_idx >= 2 {
            ignored_idx = Some(start_idx);
            start_idx -= 1;
        } else if start_idx >= 1 && matches!(tokens[start_idx - 1].lower.as_str(), "from" | "at") {
            ignored_idx = Some(start_idx - 1);
        }
        let end_idx = connector_idx + 1;
        if [start_idx, end_idx]
            .iter()
            .all(|idx| !consumed.is_consumed(*idx))
            && parse_time_token_or_shorthand(&tokens[start_idx].lower).is_some()
            && parse_time_token_or_shorthand(&tokens[end_idx].lower).is_some()
        {
            return Some((start_idx, connector_idx, end_idx, ignored_idx));
        }
    }
    None
}

fn is_range_connector(source: &str) -> bool {
    matches!(source, "til" | "until" | "to" | "-" | "–" | "—")
}

fn find_single_time(
    tokens: &[NlToken],
    consumed: &ConsumedTokens,
    anchor_idx: Option<usize>,
) -> Option<(usize, ParsedTime, Option<usize>)> {
    for (idx, token) in tokens.iter().enumerate() {
        if consumed.is_consumed(idx) {
            continue;
        }
        if token.lower == "at" {
            if let Some(next) = tokens.get(idx + 1) {
                if !consumed.is_consumed(idx + 1) {
                    if let Some(time) = parse_time_token_or_shorthand(&next.lower) {
                        return Some((idx + 1, time, Some(idx)));
                    }
                }
            }
        }
    }
    if let Some(anchor_idx) = anchor_idx {
        if let Some(next) = tokens.get(anchor_idx + 1) {
            if !consumed.is_consumed(anchor_idx + 1) {
                if let Some(time) = parse_time_token_or_shorthand(&next.lower) {
                    return Some((anchor_idx + 1, time, None));
                }
            }
        }
    }
    // Standalone bare time: only kicks in when no day anchor is present, so
    // "Morning coffee tomorrow" keeps "Morning" in subject. Allows shorthand
    // dayparts (noon, morning, tonight, ...) and unambiguous clock tokens
    // (3pm, 14:00) but not bare digits-only tokens.
    if anchor_idx.is_none() {
        for (idx, token) in tokens.iter().enumerate() {
            if consumed.is_consumed(idx) || token.lower == "at" {
                continue;
            }
            if let Some(time) = parse_bare_standalone_time_token(&token.lower) {
                return Some((idx, time, None));
            }
        }
    }
    None
}

fn parse_time_token_or_shorthand(source: &str) -> Option<ParsedTime> {
    parse_time_token(source).or_else(|| parse_time_of_day_shorthand_token(source))
}

fn parse_time_of_day_shorthand_token(source: &str) -> Option<ParsedTime> {
    match source {
        "noon" => Some(ParsedTime {
            hour: 12,
            minute: 0,
            meridiem: Some(Meridiem::Pm),
        }),
        "midnight" => Some(ParsedTime {
            hour: 12,
            minute: 0,
            meridiem: Some(Meridiem::Am),
        }),
        "morning" => Some(ParsedTime {
            hour: 9,
            minute: 0,
            meridiem: Some(Meridiem::Am),
        }),
        "afternoon" => Some(ParsedTime {
            hour: 2,
            minute: 0,
            meridiem: Some(Meridiem::Pm),
        }),
        "evening" => Some(ParsedTime {
            hour: 7,
            minute: 0,
            meridiem: Some(Meridiem::Pm),
        }),
        "tonight" => Some(ParsedTime {
            hour: 8,
            minute: 0,
            meridiem: Some(Meridiem::Pm),
        }),
        _ => None,
    }
}

fn parse_bare_standalone_time_token(source: &str) -> Option<ParsedTime> {
    if !is_bare_standalone_time_source(source) {
        return None;
    }
    parse_time_token_or_shorthand(source)
}

fn is_bare_standalone_time_source(source: &str) -> bool {
    parse_time_of_day_shorthand_token(source).is_some()
        || source.ends_with("am")
        || source.ends_with("pm")
        || source.contains(':')
}

fn resolve_time_range(
    date: NaiveDate,
    start: ParsedTime,
    end: ParsedTime,
    clock: &MenuSyntaxClock,
) -> (
    chrono::DateTime<chrono_tz::Tz>,
    chrono::DateTime<chrono_tz::Tz>,
) {
    let inherited_end = end.meridiem.or(start.meridiem);
    let inherited_start = start.meridiem.or(end.meridiem);
    let start_dt = local_dt(
        date,
        materialize_hour(start, inherited_start),
        start.minute,
        clock,
    );
    let end_hour = if end.meridiem.is_none()
        && start.meridiem == Some(Meridiem::Pm)
        && (1..=7).contains(&end.hour)
    {
        materialize_hour(end, Some(Meridiem::Am))
    } else {
        materialize_hour(end, inherited_end)
    };
    let mut end_dt = local_dt(date, end_hour, end.minute, clock);
    if end_dt <= start_dt {
        end_dt += Duration::days(1);
    }
    (start_dt, end_dt)
}

fn materialize_hour(time: ParsedTime, inherited: Option<Meridiem>) -> u32 {
    match time.meridiem.or(inherited) {
        Some(Meridiem::Am) => {
            if time.hour == 12 {
                0
            } else {
                time.hour
            }
        }
        Some(Meridiem::Pm) => {
            if time.hour == 12 {
                12
            } else {
                time.hour + 12
            }
        }
        None => match time.hour {
            1..=7 => time.hour + 12,
            8..=11 => time.hour,
            12 => 12,
            13..=23 => time.hour,
            _ => 0,
        },
    }
}

fn local_dt(
    date: NaiveDate,
    hour: u32,
    minute: u32,
    clock: &MenuSyntaxClock,
) -> chrono::DateTime<chrono_tz::Tz> {
    let Some(time) = NaiveTime::from_hms_opt(hour, minute, 0) else {
        return clock
            .timezone
            .from_utc_datetime(&date.and_time(NaiveTime::MIN));
    };
    let naive = NaiveDateTime::new(date, time);
    match clock.timezone.from_local_datetime(&naive) {
        chrono::LocalResult::Single(dt) => dt,
        chrono::LocalResult::Ambiguous(dt, _) => dt,
        chrono::LocalResult::None => clock.timezone.from_utc_datetime(&naive),
    }
}

fn resolved_date(
    source: String,
    source_span: (usize, usize),
    start: chrono::DateTime<chrono_tz::Tz>,
    end: Option<chrono::DateTime<chrono_tz::Tz>>,
    clock: &MenuSyntaxClock,
) -> ResolvedDate {
    ResolvedDate {
        role: DateRole::Inferred,
        source,
        source_span,
        iso: start.to_rfc3339(),
        end_iso: end.map(|dt| dt.to_rfc3339()),
        relative: "natural language".to_string(),
        timezone: clock.timezone_label.clone(),
        all_day: false,
        granularity: DateGranularity::Minute,
        confidence: 0.85,
    }
}

fn fragment(
    role: MenuSyntaxFragmentRole,
    source: String,
    source_span: (usize, usize),
) -> MenuSyntaxFragment {
    MenuSyntaxFragment {
        role,
        source,
        source_span,
        status: MenuSyntaxFragmentStatus::Resolved,
    }
}

fn token_span_source(tokens: &[NlToken], span: (usize, usize)) -> String {
    tokens
        .iter()
        .filter(|token| token.span.0 >= span.0 && token.span.1 <= span.1)
        .map(|token| token.text.as_str())
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menu_syntax::nl_anchor::parse_anchor;
    use crate::menu_syntax::nl_phrase::{tokenize, ConsumedTokens, NlParseOptions};
    use crate::menu_syntax::nl_recurrence::parse_recurrence;
    use chrono_tz::America::Denver;

    fn clock() -> MenuSyntaxClock {
        MenuSyntaxClock::fixed("2026-04-26T09:00:00", Denver).expect("fixed clock")
    }

    #[test]
    fn range_span_merges_anchor_at_connector() {
        let tokens = tokenize("Lunch tomorrow at 12pm til 1pm");
        let consumed = ConsumedTokens::new(tokens.len());
        let anchor = parse_anchor(
            &tokens,
            &consumed,
            &clock(),
            &NlParseOptions::calendar_default(),
        )
        .unwrap()
        .value;
        let hit = parse_time_or_range(
            &tokens,
            &consumed,
            Some(&anchor),
            None,
            &clock(),
            &NlParseOptions::calendar_default(),
        )
        .expect("time");
        assert_eq!(hit.value.date.source, "tomorrow at 12pm til 1pm");
        assert_eq!(hit.value.date.source_span, (6, 30));
    }

    #[test]
    fn tom_short_single_time_span_includes_anchor() {
        let tokens = tokenize("Lunch tom 12pm");
        let consumed = ConsumedTokens::new(tokens.len());
        let anchor = parse_anchor(
            &tokens,
            &consumed,
            &clock(),
            &NlParseOptions::calendar_default(),
        )
        .unwrap()
        .value;
        let hit = parse_time_or_range(
            &tokens,
            &consumed,
            Some(&anchor),
            None,
            &clock(),
            &NlParseOptions::calendar_default(),
        )
        .expect("time");
        assert_eq!(hit.value.date.source, "tom 12pm");
    }

    #[test]
    fn every_day_at_8am_chooses_next_non_past_occurrence() {
        let tokens = tokenize("every day at 8am");
        let mut consumed = ConsumedTokens::new(tokens.len());
        let recurrence = parse_recurrence(
            &tokens,
            &consumed,
            &clock(),
            &NlParseOptions::calendar_default(),
        )
        .unwrap();
        for idx in &recurrence.consumed {
            consumed.mark(*idx);
        }
        let hit = parse_time_or_range(
            &tokens,
            &consumed,
            None,
            Some(&recurrence.value),
            &clock(),
            &NlParseOptions::calendar_default(),
        )
        .expect("time");
        assert_eq!(hit.value.date.iso, "2026-04-27T08:00:00-06:00");
    }

    #[test]
    fn cross_midnight_range_still_rolls_end_forward() {
        let tokens = tokenize("tomorrow 11pm til 1");
        let consumed = ConsumedTokens::new(tokens.len());
        let anchor = parse_anchor(
            &tokens,
            &consumed,
            &clock(),
            &NlParseOptions::calendar_default(),
        )
        .unwrap()
        .value;
        let hit = parse_time_or_range(
            &tokens,
            &consumed,
            Some(&anchor),
            None,
            &clock(),
            &NlParseOptions::calendar_default(),
        )
        .expect("time");
        assert_eq!(
            hit.value.date.end_iso.as_deref(),
            Some("2026-04-28T01:00:00-06:00")
        );
    }
}
