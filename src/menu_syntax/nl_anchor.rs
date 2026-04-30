use chrono::{Datelike, Duration, NaiveDate, NaiveTime, TimeZone};

use crate::menu_syntax::date::{DateGranularity, MenuSyntaxClock, RecurrenceWeekday, ResolvedDate};
use crate::menu_syntax::fragments::{
    MenuSyntaxFragment, MenuSyntaxFragmentRole, MenuSyntaxFragmentStatus,
};
use crate::menu_syntax::nl_phrase::{ConsumedTokens, NlParseOptions, NlToken, ParseHit};
use crate::menu_syntax::payload::DateRole;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum AnchorIntent {
    Start,
    Due,
    Relative,
}

#[derive(Debug, Clone)]
pub(super) struct ResolvedAnchor {
    pub date: chrono::DateTime<chrono_tz::Tz>,
    pub source: String,
    pub source_span: (usize, usize),
    pub token_indices: Vec<usize>,
    pub intent: AnchorIntent,
    pub all_day: bool,
    pub granularity: DateGranularity,
}

impl ResolvedAnchor {
    pub(super) fn to_resolved_date(&self, clock: &MenuSyntaxClock) -> ResolvedDate {
        ResolvedDate {
            role: if self.intent == AnchorIntent::Due {
                DateRole::Due
            } else {
                DateRole::Start
            },
            source: self.source.clone(),
            source_span: self.source_span,
            iso: self.date.to_rfc3339(),
            end_iso: None,
            relative: self.source.clone(),
            timezone: clock.timezone_label.clone(),
            all_day: self.all_day,
            granularity: self.granularity,
            confidence: 0.9,
        }
    }
}

pub(super) fn parse_anchor(
    tokens: &[NlToken],
    consumed: &ConsumedTokens,
    clock: &MenuSyntaxClock,
    options: &NlParseOptions,
) -> Option<ParseHit<ResolvedAnchor>> {
    if !options.anchor && !options.relative_date {
        return None;
    }
    for idx in 0..tokens.len() {
        if consumed.is_consumed(idx) {
            continue;
        }
        let mut start_idx = idx;
        let mut date_token_idx = idx;
        let mut intent = AnchorIntent::Start;
        if matches!(tokens[idx].lower.as_str(), "by" | "due" | "until") {
            intent = AnchorIntent::Due;
            start_idx = idx;
            date_token_idx = idx + 1;
        }
        let token = tokens.get(date_token_idx)?;

        if token.lower == "in" && options.relative_date {
            let dur_start = date_token_idx + 1;
            let duration = crate::menu_syntax::nl_duration::parse_relative_duration_after_in(
                tokens, dur_start,
            )?;
            let target = clock
                .now
                .checked_add_signed(Duration::seconds(duration.duration.seconds as i64))?;
            let end_idx = if tokens
                .get(dur_start)
                .and_then(|t| crate::menu_syntax::nl_duration::parse_duration_token(&t.lower))
                .is_some()
            {
                dur_start
            } else {
                dur_start + 1
            };
            let span = (tokens[date_token_idx].span.0, tokens[end_idx].span.1);
            let source = token_slice(tokens, date_token_idx, end_idx);
            return Some(anchor_hit(
                ResolvedAnchor {
                    date: target,
                    source: source.clone(),
                    source_span: span,
                    token_indices: (date_token_idx..=end_idx).collect(),
                    intent: AnchorIntent::Relative,
                    all_day: false,
                    granularity: if duration.duration.seconds < 86_400 {
                        DateGranularity::Minute
                    } else {
                        DateGranularity::Date
                    },
                },
                MenuSyntaxFragmentRole::Date,
                source,
                span,
            ));
        }

        if token.lower == "next" && options.relative_date {
            if let Some(next) = tokens.get(date_token_idx + 1) {
                let date = if next.lower == "week" {
                    Some(clock.now.date_naive() + Duration::days(7))
                } else {
                    parse_weekday(&next.lower).map(|weekday| next_weekday_date(clock, weekday))
                };
                if let Some(date) = date {
                    let span = (tokens[start_idx].span.0, next.span.1);
                    let source = token_slice(tokens, start_idx, date_token_idx + 1);
                    let mut anchor =
                        build_all_day_anchor(date, source.clone(), span, intent, clock);
                    anchor.token_indices = (start_idx..=date_token_idx + 1).collect();
                    return Some(anchor_hit(
                        anchor,
                        MenuSyntaxFragmentRole::Date,
                        source,
                        span,
                    ));
                }
            }
        }

        if let Some(date) = parse_date_anchor(&token.lower, clock) {
            let span = (tokens[start_idx].span.0, token.span.1);
            let source = token_slice(tokens, start_idx, date_token_idx);
            let mut anchor = build_all_day_anchor(date, source.clone(), span, intent, clock);
            anchor.token_indices = (start_idx..=date_token_idx).collect();
            return Some(anchor_hit(
                anchor,
                MenuSyntaxFragmentRole::Date,
                source,
                span,
            ));
        }
    }
    None
}

fn build_all_day_anchor(
    date: NaiveDate,
    source: String,
    source_span: (usize, usize),
    intent: AnchorIntent,
    clock: &MenuSyntaxClock,
) -> ResolvedAnchor {
    let naive = date.and_time(NaiveTime::MIN);
    let local = match clock.timezone.from_local_datetime(&naive) {
        chrono::LocalResult::Single(dt) => dt,
        chrono::LocalResult::Ambiguous(dt, _) => dt,
        chrono::LocalResult::None => clock.timezone.from_utc_datetime(&naive),
    };
    ResolvedAnchor {
        date: local,
        source,
        source_span,
        token_indices: Vec::new(),
        intent,
        all_day: true,
        granularity: DateGranularity::Date,
    }
}

fn anchor_hit(
    anchor: ResolvedAnchor,
    role: MenuSyntaxFragmentRole,
    source: String,
    span: (usize, usize),
) -> ParseHit<ResolvedAnchor> {
    let consumed = anchor.token_indices.clone();
    ParseHit {
        value: anchor,
        fragment: MenuSyntaxFragment {
            role,
            source,
            source_span: span,
            status: MenuSyntaxFragmentStatus::Resolved,
        },
        consumed,
    }
}

fn parse_date_anchor(source: &str, clock: &MenuSyntaxClock) -> Option<NaiveDate> {
    match source {
        "today" => Some(clock.now.date_naive()),
        "tomorrow" | "tom" => Some(clock.now.date_naive() + Duration::days(1)),
        other => parse_weekday(other).map(|weekday| next_weekday_date(clock, weekday)),
    }
}

pub(super) fn parse_weekday(source: &str) -> Option<RecurrenceWeekday> {
    match source {
        "mon" | "monday" => Some(RecurrenceWeekday::Mon),
        "tue" | "tues" | "tuesday" => Some(RecurrenceWeekday::Tue),
        "wed" | "wednesday" => Some(RecurrenceWeekday::Wed),
        "thu" | "thur" | "thurs" | "thursday" => Some(RecurrenceWeekday::Thu),
        "fri" | "friday" => Some(RecurrenceWeekday::Fri),
        "sat" | "saturday" => Some(RecurrenceWeekday::Sat),
        "sun" | "sunday" => Some(RecurrenceWeekday::Sun),
        _ => None,
    }
}

pub(super) fn next_weekday_date(clock: &MenuSyntaxClock, weekday: RecurrenceWeekday) -> NaiveDate {
    let today = clock.now.date_naive();
    let current = clock.now.weekday().num_days_from_monday() as i64;
    let target = weekday_index(weekday) as i64;
    let mut days = target - current;
    if days <= 0 {
        days += 7;
    }
    today + Duration::days(days)
}

pub(super) fn weekday_index(weekday: RecurrenceWeekday) -> u32 {
    match weekday {
        RecurrenceWeekday::Mon => 0,
        RecurrenceWeekday::Tue => 1,
        RecurrenceWeekday::Wed => 2,
        RecurrenceWeekday::Thu => 3,
        RecurrenceWeekday::Fri => 4,
        RecurrenceWeekday::Sat => 5,
        RecurrenceWeekday::Sun => 6,
    }
}

fn token_slice(tokens: &[NlToken], start: usize, end: usize) -> String {
    tokens[start..=end]
        .iter()
        .map(|token| token.text.as_str())
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menu_syntax::nl_phrase::{tokenize, ConsumedTokens, NlParseOptions};
    use chrono_tz::America::Denver;

    fn clock() -> MenuSyntaxClock {
        MenuSyntaxClock::fixed("2026-04-26T09:00:00", Denver).expect("fixed clock")
    }

    #[test]
    fn anchor_tomorrow_resolves_next_day_with_span() {
        let tokens = tokenize("Renew passport tomorrow");
        let hit = parse_anchor(
            &tokens,
            &ConsumedTokens::new(tokens.len()),
            &clock(),
            &NlParseOptions::calendar_default(),
        )
        .expect("anchor");
        assert_eq!(hit.value.source, "tomorrow");
        assert_eq!(hit.value.source_span, (15, 23));
        assert_eq!(hit.value.date.to_rfc3339(), "2026-04-27T00:00:00-06:00");
    }

    #[test]
    fn anchor_in_30_minutes_resolves_relative_now() {
        let tokens = tokenize("in 30 minutes");
        let hit = parse_anchor(
            &tokens,
            &ConsumedTokens::new(tokens.len()),
            &clock(),
            &NlParseOptions::calendar_default(),
        )
        .expect("anchor");
        assert_eq!(hit.value.source, "in 30 minutes");
        assert_eq!(hit.value.date.to_rfc3339(), "2026-04-26T09:30:00-06:00");
        assert_eq!(hit.value.intent, AnchorIntent::Relative);
    }

    #[test]
    fn anchor_next_week_resolves_fuzzy_date() {
        let tokens = tokenize("until next week");
        let hit = parse_anchor(
            &tokens,
            &ConsumedTokens::new(tokens.len()),
            &clock(),
            &NlParseOptions::calendar_default(),
        )
        .expect("anchor");
        assert_eq!(hit.value.source, "until next week");
        assert_eq!(hit.value.date.to_rfc3339(), "2026-05-03T00:00:00-06:00");
        assert_eq!(hit.value.intent, AnchorIntent::Due);
    }
}
