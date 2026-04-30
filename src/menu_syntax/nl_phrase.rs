use crate::menu_syntax::date::{
    MenuSyntaxClock, ResolvedDate, ResolvedDuration, ResolvedRecurrence,
};
use crate::menu_syntax::fragments::MenuSyntaxFragment;
use crate::menu_syntax::payload::{CaptureInvocation, DateRole};

#[derive(Debug, Clone, PartialEq)]
pub struct CaptureNlResolution {
    pub subject: String,
    pub fragments: Vec<MenuSyntaxFragment>,
    pub date: Option<ResolvedDate>,
    pub duration: Option<ResolvedDuration>,
    pub recurrence: Option<ResolvedRecurrence>,
}

#[derive(Debug, Clone)]
pub(super) struct NlToken {
    pub text: String,
    pub lower: String,
    pub span: (usize, usize),
}

pub(super) fn tokenize(input: &str) -> Vec<NlToken> {
    let mut tokens = Vec::new();
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i].is_ascii_whitespace() {
            i += 1;
            continue;
        }
        let start = i;
        while i < bytes.len() && !bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        let text = input[start..i].to_string();
        tokens.push(NlToken {
            lower: text.to_ascii_lowercase(),
            text,
            span: (start, i),
        });
    }
    tokens
}

#[derive(Debug, Clone, Default)]
pub(super) struct ConsumedTokens {
    bits: Vec<bool>,
}

impl ConsumedTokens {
    pub(super) fn new(len: usize) -> Self {
        Self {
            bits: vec![false; len],
        }
    }

    pub(super) fn mark(&mut self, idx: usize) {
        if let Some(bit) = self.bits.get_mut(idx) {
            *bit = true;
        }
    }

    #[allow(dead_code)]
    pub(super) fn mark_range(&mut self, range: std::ops::RangeInclusive<usize>) {
        for idx in range {
            self.mark(idx);
        }
    }

    pub(super) fn is_consumed(&self, idx: usize) -> bool {
        self.bits.get(idx).copied().unwrap_or(false)
    }
}

#[derive(Debug, Clone)]
pub(super) struct ParseHit<T> {
    pub value: T,
    pub fragment: MenuSyntaxFragment,
    pub consumed: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct NlParseOptions {
    pub anchor: bool,
    pub relative_date: bool,
    pub time: bool,
    pub date_range: bool,
    pub duration: bool,
    pub recurrence: bool,
    pub daily: bool,
    pub multi_weekday: bool,
    pub monthly: bool,
    pub yearly: bool,
}

impl Default for NlParseOptions {
    fn default() -> Self {
        Self {
            anchor: false,
            relative_date: false,
            time: false,
            date_range: false,
            duration: false,
            recurrence: false,
            daily: false,
            multi_weekday: false,
            monthly: false,
            yearly: false,
        }
    }
}

impl NlParseOptions {
    pub fn calendar_default() -> Self {
        Self {
            anchor: true,
            relative_date: true,
            time: true,
            date_range: true,
            duration: true,
            recurrence: true,
            daily: true,
            multi_weekday: true,
            monthly: true,
            yearly: true,
        }
    }

    pub fn from_target_accepts(target: &str, accepts: &[String]) -> Self {
        let calendar = target.eq_ignore_ascii_case("cal") || target.eq_ignore_ascii_case("mcal");
        if calendar {
            return Self::calendar_default();
        }
        let has = |needle: &str| accepts.iter().any(|accept| accept == needle);
        let wants_recurrence = has("recurrence")
            || has("daily")
            || has("multiWeekday")
            || has("monthly")
            || has("yearly");
        Self {
            anchor: has("date") || has("dateRange") || has("relativeDate") || wants_recurrence,
            relative_date: has("relativeDate"),
            time: has("date") || has("dateRange") || wants_recurrence,
            date_range: has("dateRange"),
            duration: has("duration") || has("relativeDate"),
            recurrence: has("recurrence") || has("multiWeekday"),
            daily: has("daily") || has("recurrence"),
            multi_weekday: has("multiWeekday") || has("recurrence"),
            monthly: has("monthly"),
            yearly: has("yearly"),
        }
    }
}

pub fn resolve_capture_nl_phrase(
    invocation: &CaptureInvocation,
    clock: &MenuSyntaxClock,
) -> CaptureNlResolution {
    resolve_capture_nl_phrase_with_options(invocation, clock, &NlParseOptions::calendar_default())
}

pub fn resolve_capture_nl_phrase_with_accepts(
    invocation: &CaptureInvocation,
    clock: &MenuSyntaxClock,
    accepts: &[String],
) -> CaptureNlResolution {
    let options = NlParseOptions::from_target_accepts(&invocation.target, accepts);
    resolve_capture_nl_phrase_with_options(invocation, clock, &options)
}

pub fn resolve_capture_nl_phrase_with_options(
    invocation: &CaptureInvocation,
    clock: &MenuSyntaxClock,
    options: &NlParseOptions,
) -> CaptureNlResolution {
    let tokens = tokenize(&invocation.body);
    let mut consumed = ConsumedTokens::new(tokens.len());
    let mut fragments = Vec::new();

    let anchor_hit =
        crate::menu_syntax::nl_anchor::parse_anchor(&tokens, &consumed, clock, options);
    let anchor = anchor_hit.as_ref().map(|hit| hit.value.clone());

    let recurrence_hit =
        crate::menu_syntax::nl_recurrence::parse_recurrence(&tokens, &consumed, clock, options);
    let recurrence_phrase = recurrence_hit.as_ref().map(|hit| hit.value.clone());
    if let Some(hit) = recurrence_hit.as_ref() {
        for idx in &hit.consumed {
            consumed.mark(*idx);
        }
        fragments.push(hit.fragment.clone());
    }

    let time_hit = crate::menu_syntax::nl_time::parse_time_or_range(
        &tokens,
        &consumed,
        anchor.as_ref(),
        recurrence_phrase.as_ref(),
        clock,
        options,
    );
    let mut date = time_hit.as_ref().map(|hit| hit.value.date.clone());
    if let Some(hit) = time_hit.as_ref() {
        for idx in &hit.consumed {
            consumed.mark(*idx);
        }
        fragments.push(hit.fragment.clone());
    }

    let duration_hit = crate::menu_syntax::nl_duration::parse_duration(&tokens, &consumed);
    let duration = if options.duration {
        duration_hit.as_ref().map(|hit| hit.value.duration.clone())
    } else {
        None
    };
    if let Some(hit) = duration_hit.as_ref().filter(|_| options.duration) {
        for idx in &hit.consumed {
            consumed.mark(*idx);
        }
        fragments.push(hit.fragment.clone());
        if let Some(resolved_date) = date.as_mut() {
            if let Ok(start) = chrono::DateTime::parse_from_rfc3339(&resolved_date.iso) {
                resolved_date.end_iso = Some(
                    (start.with_timezone(&clock.timezone)
                        + chrono::Duration::seconds(hit.value.duration.seconds as i64))
                    .to_rfc3339(),
                );
            }
        }
    } else if let Some(for_idx) = tokens.iter().position(|t| t.lower == "for") {
        consumed.mark(for_idx);
        let span = if let Some(next) = tokens.get(for_idx + 1) {
            consumed.mark(for_idx + 1);
            (tokens[for_idx].span.0, next.span.1)
        } else {
            tokens[for_idx].span
        };
        fragments.push(crate::menu_syntax::nl_duration::unresolved_fragment(
            &invocation.body[span.0..span.1],
            span,
        ));
    }

    if date.is_none() {
        if let Some(hit) = anchor_hit.as_ref().filter(|_| options.anchor) {
            let absorbed = hit.consumed.iter().any(|idx| consumed.is_consumed(*idx));
            if !absorbed {
                for idx in &hit.consumed {
                    consumed.mark(*idx);
                }
                fragments.push(hit.fragment.clone());
                let mut resolved = hit.value.to_resolved_date(clock);
                if hit.value.intent == crate::menu_syntax::nl_anchor::AnchorIntent::Due {
                    resolved.role = DateRole::Due;
                }
                date = Some(resolved);
            }
        }
    }

    fragments.sort_by_key(|fragment| fragment.source_span.0);

    let subject = tokens
        .iter()
        .enumerate()
        .filter(|(idx, _)| !consumed.is_consumed(*idx))
        .map(|(_, token)| token.text.as_str())
        .collect::<Vec<_>>()
        .join(" ");

    CaptureNlResolution {
        subject,
        fragments,
        date,
        duration,
        recurrence: recurrence_phrase.map(|phrase| phrase.recurrence),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menu_syntax::capture::{parse_capture, CaptureParse};
    use crate::menu_syntax::date::{RecurrenceFrequency, RecurrenceWeekday};
    use crate::menu_syntax::fragments::{MenuSyntaxFragmentRole, MenuSyntaxFragmentStatus};
    use chrono_tz::America::Denver;

    fn clock() -> MenuSyntaxClock {
        MenuSyntaxClock::fixed("2026-04-26T09:00:00", Denver).expect("fixed clock")
    }

    fn monday_clock() -> MenuSyntaxClock {
        MenuSyntaxClock::fixed("2026-04-27T09:00:00", Denver).expect("fixed clock")
    }

    fn monday_clock_at(local: &str) -> MenuSyntaxClock {
        MenuSyntaxClock::fixed(local, Denver).expect("fixed clock")
    }

    fn parse_ok(input: &str) -> CaptureInvocation {
        match parse_capture(input) {
            CaptureParse::Ok(inv) => inv,
            CaptureParse::Incomplete(s) => panic!("expected ok, got {s:?}"),
        }
    }

    fn custom_inv(target: &str, body: &str) -> CaptureInvocation {
        CaptureInvocation {
            target: target.to_string(),
            alias_form: crate::menu_syntax::payload::CaptureAlias::Keyword,
            body: body.to_string(),
            tags: vec![],
            priority: None,
            url: None,
            duration: None,
            kv: vec![],
            date_phrases: vec![],
            raw: format!(";{target} {body}"),
        }
    }

    #[test]
    fn tomorrow_at_noon_til_one_resolves_range_and_subject() {
        let inv = parse_ok(";mcal Lunch with Ryan tomorrow at 12pm til 1pm");
        let resolution = resolve_capture_nl_phrase(&inv, &clock());
        let date = resolution.date.expect("date");
        assert_eq!(resolution.subject, "Lunch with Ryan");
        assert_eq!(date.iso, "2026-04-27T12:00:00-06:00");
        assert_eq!(date.end_iso.as_deref(), Some("2026-04-27T13:00:00-06:00"));
    }

    #[test]
    fn daterange_fragment_span_includes_anchor_tomorrow() {
        let inv = parse_ok(";mcal Lunch with Ryan tomorrow at 12pm til 1pm");
        let resolution = resolve_capture_nl_phrase(&inv, &clock());
        let date = resolution.date.expect("date");
        assert_eq!(date.source, "tomorrow at 12pm til 1pm");
        assert_eq!(date.source_span, (16, 40));
        assert!(resolution.fragments.iter().any(|fragment| {
            fragment.role == MenuSyntaxFragmentRole::DateRange
                && fragment.source == "tomorrow at 12pm til 1pm"
                && fragment.source_span == (16, 40)
        }));
    }

    #[test]
    fn daterange_fragment_span_includes_anchor_tom() {
        let inv = parse_ok(";mcal Lunch with Ryan tom 12pm for 30mins");
        let resolution = resolve_capture_nl_phrase(&inv, &clock());
        let date = resolution.date.expect("date");
        assert_eq!(date.source, "tom 12pm");
        assert_eq!(date.source_span, (16, 24));
        assert!(resolution.fragments.iter().any(|fragment| {
            fragment.role == MenuSyntaxFragmentRole::Date
                && fragment.source == "tom 12pm"
                && fragment.source_span == (16, 24)
        }));
    }

    #[test]
    fn tom_short_noon_for_30mins_resolves_duration_and_end() {
        let inv = parse_ok(";mcal Lunch with Ryan tom 12pm for 30mins");
        let resolution = resolve_capture_nl_phrase(&inv, &clock());
        let date = resolution.date.expect("date");
        let duration = resolution.duration.expect("duration");
        assert_eq!(resolution.subject, "Lunch with Ryan");
        assert_eq!(date.iso, "2026-04-27T12:00:00-06:00");
        assert_eq!(date.end_iso.as_deref(), Some("2026-04-27T12:30:00-06:00"));
        assert_eq!(duration.minutes, 30);
        assert_eq!(duration.seconds, 1800);
    }

    #[test]
    fn every_mon_from_one_til_two_resolves_weekly_recurrence() {
        let inv = parse_ok(";mcal Lunch w/ Ryan every mon from 1 til 2");
        let resolution = resolve_capture_nl_phrase(&inv, &clock());
        let date = resolution.date.expect("date");
        let recurrence = resolution.recurrence.expect("recurrence");
        assert_eq!(resolution.subject, "Lunch w/ Ryan");
        assert_eq!(date.iso, "2026-04-27T13:00:00-06:00");
        assert_eq!(date.end_iso.as_deref(), Some("2026-04-27T14:00:00-06:00"));
        assert_eq!(recurrence.rrule, "FREQ=WEEKLY;BYDAY=MO");
        assert_eq!(recurrence.label, "every Monday");
    }

    #[test]
    fn til_until_to_and_dash_are_equivalent_range_connectors() {
        for connector in ["til", "until", "to", "-"] {
            let inv = parse_ok(&format!(";mcal Lunch tomorrow 12pm {connector} 1pm"));
            let date = resolve_capture_nl_phrase(&inv, &clock()).date.unwrap();
            assert_eq!(date.end_iso.as_deref(), Some("2026-04-27T13:00:00-06:00"));
        }
    }

    #[test]
    fn duration_compact_and_spaced_units_parse() {
        for duration in ["30m", "30min", "30mins", "1h", "2hrs"] {
            let inv = parse_ok(&format!(";mcal Lunch tom 12pm for {duration}"));
            assert!(
                resolve_capture_nl_phrase(&inv, &clock()).duration.is_some(),
                "{duration}"
            );
        }
    }

    #[test]
    fn bare_one_to_two_defaults_to_pm() {
        let inv = parse_ok(";mcal Lunch tomorrow 1 til 2");
        let date = resolve_capture_nl_phrase(&inv, &clock()).date.unwrap();
        assert_eq!(date.iso, "2026-04-27T13:00:00-06:00");
        assert_eq!(date.end_iso.as_deref(), Some("2026-04-27T14:00:00-06:00"));
    }

    #[test]
    fn explicit_pm_cross_midnight_infers_end_am() {
        let inv = parse_ok(";mcal Late deploy tomorrow 11pm til 1");
        let date = resolve_capture_nl_phrase(&inv, &clock()).date.unwrap();
        assert_eq!(date.iso, "2026-04-27T23:00:00-06:00");
        assert_eq!(date.end_iso.as_deref(), Some("2026-04-28T01:00:00-06:00"));
    }

    #[test]
    fn w_slash_stays_subject_not_syntax() {
        let inv = parse_ok(";mcal Lunch w/ Ryan tom 12pm for 30mins");
        let resolution = resolve_capture_nl_phrase(&inv, &clock());
        assert_eq!(resolution.subject, "Lunch w/ Ryan");
    }

    #[test]
    fn unresolved_recognized_fragment_is_reported_muted() {
        let inv = parse_ok(";mcal Lunch tomorrow 12pm for later");
        let resolution = resolve_capture_nl_phrase(&inv, &clock());
        assert!(resolution.fragments.iter().any(|fragment| {
            fragment.status == MenuSyntaxFragmentStatus::Unresolved
                && fragment.role == MenuSyntaxFragmentRole::Unresolved
        }));
    }

    #[test]
    fn todo_renew_passport_tomorrow_resolves_anchor_only() {
        let inv = parse_ok(";todo Renew passport tomorrow");
        let accepts = vec!["date".to_string()];
        let resolution = resolve_capture_nl_phrase_with_accepts(&inv, &clock(), &accepts);
        let date = resolution.date.expect("date");
        assert_eq!(resolution.subject, "Renew passport");
        assert_eq!(date.iso, "2026-04-27T00:00:00-06:00");
        assert!(date.all_day);
    }

    #[test]
    fn reminder_walk_dog_every_day_at_8am_combines_anchor_recurrence_time() {
        let inv = custom_inv("reminder", "Walk dog every day at 8am");
        let accepts = vec!["daily".to_string(), "date".to_string()];
        let resolution = resolve_capture_nl_phrase_with_accepts(&inv, &clock(), &accepts);
        let date = resolution.date.expect("date");
        let recurrence = resolution.recurrence.expect("recurrence");
        assert_eq!(resolution.subject, "Walk dog");
        assert_eq!(date.iso, "2026-04-27T08:00:00-06:00");
        assert_eq!(recurrence.frequency, RecurrenceFrequency::Daily);
        assert!(recurrence.weekdays.is_empty());
    }

    #[test]
    fn snooze_in_30_minutes_resolves_relative_now() {
        let inv = custom_inv("snooze", "in 30 minutes");
        let accepts = vec!["relativeDate".to_string()];
        let resolution = resolve_capture_nl_phrase_with_accepts(&inv, &clock(), &accepts);
        let date = resolution.date.expect("date");
        assert_eq!(resolution.subject, "");
        assert_eq!(date.iso, "2026-04-26T09:30:00-06:00");
        assert_eq!(
            date.granularity,
            crate::menu_syntax::date::DateGranularity::Minute
        );
    }

    #[test]
    fn defer_until_next_week_fuzzy_no_time() {
        let inv = custom_inv("defer", "until next week");
        let accepts = vec!["date".to_string(), "relativeDate".to_string()];
        let resolution = resolve_capture_nl_phrase_with_accepts(&inv, &clock(), &accepts);
        let date = resolution.date.expect("date");
        assert_eq!(resolution.subject, "");
        assert_eq!(date.iso, "2026-05-03T00:00:00-06:00");
        assert!(date.all_day);
    }

    #[test]
    fn todo_submit_form_by_friday_p1_preserves_priority_and_sets_due_date() {
        let inv = parse_ok(";todo Submit form by friday p1");
        assert_eq!(inv.priority, Some(1));
        let accepts = vec!["date".to_string(), "priority".to_string()];
        let resolution = resolve_capture_nl_phrase_with_accepts(&inv, &clock(), &accepts);
        let date = resolution.date.expect("date");
        assert_eq!(resolution.subject, "Submit form");
        assert_eq!(date.role, DateRole::Due);
        assert_eq!(date.iso, "2026-05-01T00:00:00-06:00");
    }

    #[test]
    fn standalone_at_noon_for_30_min_resolves_today_and_subject() {
        let inv = parse_ok(";mcal Lunch w/ Mindy at noon for 30 min");
        let resolution = resolve_capture_nl_phrase(&inv, &monday_clock());
        let date = resolution.date.expect("date");
        let duration = resolution.duration.expect("duration");
        assert_eq!(resolution.subject, "Lunch w/ Mindy");
        assert_eq!(date.iso, "2026-04-27T12:00:00-06:00");
        assert_eq!(date.end_iso.as_deref(), Some("2026-04-27T12:30:00-06:00"));
        assert_eq!(duration.minutes, 30);
        assert_eq!(duration.seconds, 1800);
        assert!(resolution.fragments.iter().any(|fragment| {
            fragment.role == MenuSyntaxFragmentRole::Date && fragment.source == "at noon"
        }));
    }

    #[test]
    fn standalone_noon_without_at_resolves_same_as_at_noon() {
        let clock = monday_clock();
        let with_at = parse_ok(";mcal Lunch w/ Mindy at noon");
        let bare = parse_ok(";mcal Lunch w/ Mindy noon");
        let with_at_date = resolve_capture_nl_phrase(&with_at, &clock).date.unwrap();
        let bare_resolution = resolve_capture_nl_phrase(&bare, &clock);
        let bare_date = bare_resolution.date.unwrap();
        assert_eq!(bare_resolution.subject, "Lunch w/ Mindy");
        assert_eq!(bare_date.iso, with_at_date.iso);
        assert_eq!(bare_date.iso, "2026-04-27T12:00:00-06:00");
    }

    #[test]
    fn standalone_morning_resolves_to_0900_today() {
        let inv = parse_ok(";mcal Breakfast meeting morning");
        let resolution = resolve_capture_nl_phrase(&inv, &monday_clock());
        let date = resolution.date.expect("date");
        assert_eq!(resolution.subject, "Breakfast meeting");
        assert_eq!(date.iso, "2026-04-27T09:00:00-06:00");
    }

    #[test]
    fn standalone_afternoon_evening_tonight_resolve_to_pinned_defaults() {
        let clock = monday_clock();
        for (word, expected_iso) in [
            ("afternoon", "2026-04-27T14:00:00-06:00"),
            ("evening", "2026-04-27T19:00:00-06:00"),
            ("tonight", "2026-04-27T20:00:00-06:00"),
        ] {
            let inv = parse_ok(&format!(";mcal Thing {word}"));
            let resolution = resolve_capture_nl_phrase(&inv, &clock);
            let date = resolution.date.expect(word);
            assert_eq!(resolution.subject, "Thing", "{word}");
            assert_eq!(date.iso, expected_iso, "{word}");
            assert_eq!(date.source, word, "{word}");
        }
    }

    #[test]
    fn standalone_t001_at_prefix_vocab_resolves_today() {
        let clock = monday_clock();
        for (phrase, expected_iso) in [
            ("at 12", "2026-04-27T12:00:00-06:00"),
            ("at 12pm", "2026-04-27T12:00:00-06:00"),
            ("at 9", "2026-04-27T09:00:00-06:00"),
            ("at 9am", "2026-04-27T09:00:00-06:00"),
            ("at noon", "2026-04-27T12:00:00-06:00"),
            ("at midnight", "2026-04-27T00:00:00-06:00"),
        ] {
            let inv = parse_ok(&format!(";mcal Thing {phrase}"));
            let resolution = resolve_capture_nl_phrase(&inv, &clock);
            let date = resolution.date.expect(phrase);
            assert_eq!(resolution.subject, "Thing", "{phrase}");
            assert_eq!(date.iso, expected_iso, "{phrase}");
            assert_eq!(date.source, phrase, "{phrase}");
        }
        let inv = custom_inv("mcal", "Thing at 14:00");
        let resolution = resolve_capture_nl_phrase(&inv, &clock);
        let date = resolution.date.expect("at 14:00");
        assert_eq!(resolution.subject, "Thing");
        assert_eq!(date.iso, "2026-04-27T14:00:00-06:00");
        assert_eq!(date.source, "at 14:00");
    }

    #[test]
    fn standalone_at_3pm_resolves_to_today_1500() {
        let inv = parse_ok(";mcal Lunch at 3pm");
        let resolution = resolve_capture_nl_phrase(&inv, &monday_clock());
        let date = resolution.date.expect("date");
        assert_eq!(resolution.subject, "Lunch");
        assert_eq!(date.iso, "2026-04-27T15:00:00-06:00");
        assert_eq!(date.source, "at 3pm");
    }

    #[test]
    fn standalone_bare_3pm_resolves_to_today_1500() {
        let inv = parse_ok(";mcal Lunch 3pm");
        let resolution = resolve_capture_nl_phrase(&inv, &monday_clock());
        let date = resolution.date.expect("date");
        assert_eq!(resolution.subject, "Lunch");
        assert_eq!(date.iso, "2026-04-27T15:00:00-06:00");
        assert_eq!(date.source, "3pm");
    }

    #[test]
    fn standalone_at_banana_stays_unresolved() {
        let inv = parse_ok(";mcal Lunch at banana");
        let resolution = resolve_capture_nl_phrase(&inv, &monday_clock());
        assert_eq!(resolution.subject, "Lunch at banana");
        assert!(resolution.date.is_none());
        assert!(resolution.fragments.is_empty());
    }

    #[test]
    fn morningish_inside_subject_does_not_match_morning() {
        let inv = parse_ok(";mcal Coffee morningish");
        let resolution = resolve_capture_nl_phrase(&inv, &monday_clock());
        assert_eq!(resolution.subject, "Coffee morningish");
        assert!(resolution.date.is_none());
        assert!(resolution.fragments.is_empty());
    }

    #[test]
    fn subject_with_morning_only_at_end_extracts_anchor() {
        let inv = parse_ok(";mcal Coffee morning");
        let resolution = resolve_capture_nl_phrase(&inv, &monday_clock());
        let date = resolution.date.expect("date");
        assert_eq!(resolution.subject, "Coffee");
        assert_eq!(date.iso, "2026-04-27T09:00:00-06:00");
        assert_eq!(date.source, "morning");
    }

    #[test]
    fn standalone_past_time_stays_today_not_tomorrow() {
        let inv = parse_ok(";mcal Lunch at noon for 30 min");
        let clock = monday_clock_at("2026-04-27T14:00:00");
        let resolution = resolve_capture_nl_phrase(&inv, &clock);
        let date = resolution.date.expect("date");
        assert_eq!(resolution.subject, "Lunch");
        assert_eq!(date.iso, "2026-04-27T12:00:00-06:00");
        assert_eq!(date.end_iso.as_deref(), Some("2026-04-27T12:30:00-06:00"));
    }

    #[test]
    fn todo_daily_standup_every_weekday_at_9am_recurrence_without_range_end() {
        let inv = parse_ok(";todo Daily standup every weekday at 9am");
        let accepts = vec![
            "date".to_string(),
            "recurrence".to_string(),
            "multiWeekday".to_string(),
        ];
        let resolution = resolve_capture_nl_phrase_with_accepts(&inv, &clock(), &accepts);
        let date = resolution.date.expect("date");
        let recurrence = resolution.recurrence.expect("recurrence");
        assert_eq!(resolution.subject, "Daily standup");
        assert_eq!(date.iso, "2026-04-27T09:00:00-06:00");
        assert_eq!(date.end_iso, None);
        assert_eq!(
            recurrence.weekdays,
            vec![
                RecurrenceWeekday::Mon,
                RecurrenceWeekday::Tue,
                RecurrenceWeekday::Wed,
                RecurrenceWeekday::Thu,
                RecurrenceWeekday::Fri,
            ]
        );
    }
}
