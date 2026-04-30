use crate::menu_syntax::date::{
    resolve_date_phrase, MenuSyntaxClock, RecurrenceFrequency, RecurrenceWeekday,
    ResolvedRecurrence,
};
use crate::menu_syntax::fragments::{
    MenuSyntaxFragment, MenuSyntaxFragmentRole, MenuSyntaxFragmentStatus,
};
use crate::menu_syntax::nl_anchor::parse_weekday;
use crate::menu_syntax::nl_phrase::{ConsumedTokens, NlParseOptions, NlToken, ParseHit};

#[derive(Debug, Clone)]
pub(super) struct ResolvedRecurrencePhrase {
    pub recurrence: ResolvedRecurrence,
    pub anchor_weekdays: Vec<RecurrenceWeekday>,
    #[allow(dead_code)]
    pub source: String,
    #[allow(dead_code)]
    pub source_span: (usize, usize),
    #[allow(dead_code)]
    pub token_indices: Vec<usize>,
}

pub(super) fn parse_recurrence(
    tokens: &[NlToken],
    consumed: &ConsumedTokens,
    clock: &MenuSyntaxClock,
    options: &NlParseOptions,
) -> Option<ParseHit<ResolvedRecurrencePhrase>> {
    if !(options.recurrence
        || options.daily
        || options.multi_weekday
        || options.monthly
        || options.yearly)
    {
        return None;
    }
    for idx in 0..tokens.len() {
        if consumed.is_consumed(idx) {
            continue;
        }
        if tokens[idx].lower != "every" && !is_lead_in(&tokens[idx].lower) {
            continue;
        }
        if let Some(hit) = parse_with_optional_lead_in(tokens, consumed, idx, clock, options) {
            return Some(hit);
        }
    }
    for idx in 0..tokens.len() {
        if consumed.is_consumed(idx)
            || tokens[idx].lower == "every"
            || is_lead_in(&tokens[idx].lower)
        {
            continue;
        }
        if let Some(hit) = parse_with_optional_lead_in(tokens, consumed, idx, clock, options) {
            return Some(hit);
        }
    }
    None
}

#[derive(Debug, Clone)]
struct RecurrenceCore {
    end_idx: usize,
    frequency: RecurrenceFrequency,
    weekdays: Vec<RecurrenceWeekday>,
    interval: Option<u16>,
}

fn parse_with_optional_lead_in(
    tokens: &[NlToken],
    consumed: &ConsumedTokens,
    idx: usize,
    clock: &MenuSyntaxClock,
    options: &NlParseOptions,
) -> Option<ParseHit<ResolvedRecurrencePhrase>> {
    if is_lead_in(&tokens[idx].lower) {
        let core_start = idx + 1;
        if core_start >= tokens.len() || consumed.is_consumed(core_start) {
            return None;
        }
        let core = parse_core(tokens, consumed, core_start, options)?;
        return Some(build_hit(tokens, idx, core, clock, consumed));
    }
    let core = parse_core(tokens, consumed, idx, options)?;
    Some(build_hit(tokens, idx, core, clock, consumed))
}

fn is_lead_in(token: &str) -> bool {
    matches!(
        token,
        "repeat" | "repeats" | "repeating" | "recurring" | "recurs"
    )
}

fn parse_core(
    tokens: &[NlToken],
    consumed: &ConsumedTokens,
    idx: usize,
    options: &NlParseOptions,
) -> Option<RecurrenceCore> {
    if consumed.is_consumed(idx) {
        return None;
    }
    match tokens[idx].lower.as_str() {
        "daily" if options.daily => Some(core(idx, idx, RecurrenceFrequency::Daily, vec![], None)),
        "weekly" if options.recurrence => {
            Some(core(idx, idx, RecurrenceFrequency::Weekly, vec![], None))
        }
        "biweekly" | "fortnightly" if options.recurrence => {
            Some(core(idx, idx, RecurrenceFrequency::Weekly, vec![], Some(2)))
        }
        "bimonthly" if options.monthly => Some(core(
            idx,
            idx,
            RecurrenceFrequency::Monthly,
            vec![],
            Some(2),
        )),
        "monthly" if options.monthly => {
            Some(core(idx, idx, RecurrenceFrequency::Monthly, vec![], None))
        }
        "yearly" | "annually" if options.yearly => {
            Some(core(idx, idx, RecurrenceFrequency::Yearly, vec![], None))
        }
        "every" => parse_every(tokens, consumed, idx, options),
        _ => None,
    }
}

fn core(
    _start_idx: usize,
    end_idx: usize,
    frequency: RecurrenceFrequency,
    weekdays: Vec<RecurrenceWeekday>,
    interval: Option<u16>,
) -> RecurrenceCore {
    RecurrenceCore {
        end_idx,
        frequency,
        weekdays,
        interval,
    }
}

fn parse_every(
    tokens: &[NlToken],
    consumed: &ConsumedTokens,
    idx: usize,
    options: &NlParseOptions,
) -> Option<RecurrenceCore> {
    let next_idx = idx + 1;
    if next_idx >= tokens.len() || consumed.is_consumed(next_idx) {
        return None;
    }
    let next = normalized(&tokens[next_idx].lower);
    match next.as_str() {
        "day" if options.daily => {
            return Some(core(
                idx,
                next_idx,
                RecurrenceFrequency::Daily,
                vec![],
                None,
            ));
        }
        "weekday" if options.recurrence || options.multi_weekday => {
            return Some(core(
                idx,
                next_idx,
                RecurrenceFrequency::Weekly,
                vec![
                    RecurrenceWeekday::Mon,
                    RecurrenceWeekday::Tue,
                    RecurrenceWeekday::Wed,
                    RecurrenceWeekday::Thu,
                    RecurrenceWeekday::Fri,
                ],
                None,
            ));
        }
        "weekend" if options.recurrence || options.multi_weekday => {
            return Some(core(
                idx,
                next_idx,
                RecurrenceFrequency::Weekly,
                vec![RecurrenceWeekday::Sat, RecurrenceWeekday::Sun],
                None,
            ));
        }
        "week" if options.recurrence => {
            return Some(core(
                idx,
                next_idx,
                RecurrenceFrequency::Weekly,
                vec![],
                None,
            ));
        }
        "month" if options.monthly => {
            return Some(core(
                idx,
                next_idx,
                RecurrenceFrequency::Monthly,
                vec![],
                None,
            ));
        }
        "year" if options.yearly => {
            return Some(core(
                idx,
                next_idx,
                RecurrenceFrequency::Yearly,
                vec![],
                None,
            ));
        }
        "other" => return parse_every_other(tokens, consumed, idx, options),
        _ => {}
    }

    if let Some(interval) = parse_interval_number(&next) {
        return parse_numeric_interval(tokens, idx, next_idx, interval, options);
    }

    parse_weekday_list(tokens, consumed, idx, options)
}

fn parse_every_other(
    tokens: &[NlToken],
    consumed: &ConsumedTokens,
    idx: usize,
    options: &NlParseOptions,
) -> Option<RecurrenceCore> {
    let unit_idx = idx + 2;
    if unit_idx >= tokens.len() || consumed.is_consumed(unit_idx) {
        return None;
    }
    let unit = normalized(&tokens[unit_idx].lower);
    match unit.as_str() {
        "day" if options.daily => Some(core(
            idx,
            unit_idx,
            RecurrenceFrequency::Daily,
            vec![],
            Some(2),
        )),
        "week" if options.recurrence => Some(core(
            idx,
            unit_idx,
            RecurrenceFrequency::Weekly,
            vec![],
            Some(2),
        )),
        "month" if options.monthly => Some(core(
            idx,
            unit_idx,
            RecurrenceFrequency::Monthly,
            vec![],
            Some(2),
        )),
        "year" if options.yearly => Some(core(
            idx,
            unit_idx,
            RecurrenceFrequency::Yearly,
            vec![],
            Some(2),
        )),
        _ if options.recurrence => parse_weekday(&unit).map(|weekday| {
            core(
                idx,
                unit_idx,
                RecurrenceFrequency::Weekly,
                vec![weekday],
                Some(2),
            )
        }),
        _ => None,
    }
}

fn parse_numeric_interval(
    tokens: &[NlToken],
    start_idx: usize,
    number_idx: usize,
    interval: u16,
    options: &NlParseOptions,
) -> Option<RecurrenceCore> {
    if interval == 0 || interval > 30 || number_idx + 1 >= tokens.len() {
        return None;
    }
    let unit_idx = number_idx + 1;
    let unit = normalized(&tokens[unit_idx].lower);
    let interval = (interval > 1).then_some(interval);
    match unit.as_str() {
        "day" | "days" if options.daily => Some(core(
            start_idx,
            unit_idx,
            RecurrenceFrequency::Daily,
            vec![],
            interval,
        )),
        "week" | "weeks" if options.recurrence => Some(core(
            start_idx,
            unit_idx,
            RecurrenceFrequency::Weekly,
            vec![],
            interval,
        )),
        "month" | "months" if options.monthly => Some(core(
            start_idx,
            unit_idx,
            RecurrenceFrequency::Monthly,
            vec![],
            interval,
        )),
        "year" | "years" if options.yearly => Some(core(
            start_idx,
            unit_idx,
            RecurrenceFrequency::Yearly,
            vec![],
            interval,
        )),
        _ => None,
    }
}

fn parse_weekday_list(
    tokens: &[NlToken],
    consumed: &ConsumedTokens,
    idx: usize,
    options: &NlParseOptions,
) -> Option<RecurrenceCore> {
    if !(options.recurrence || options.multi_weekday) {
        return None;
    }
    let mut weekdays = Vec::new();
    let mut end_idx = idx;
    let mut cursor = idx + 1;
    let mut previous_had_comma = false;
    loop {
        if cursor >= tokens.len() || consumed.is_consumed(cursor) {
            break;
        }
        if normalized(&tokens[cursor].lower) == "and" {
            let next_idx = cursor + 1;
            if next_idx >= tokens.len() || consumed.is_consumed(next_idx) {
                break;
            }
            let next = normalized(&tokens[next_idx].lower);
            let Some(weekday) = parse_weekday(&next) else {
                break;
            };
            weekdays.push(weekday);
            end_idx = next_idx;
            previous_had_comma = tokens[next_idx].lower.ends_with(',');
            cursor = next_idx + 1;
            continue;
        }
        if !weekdays.is_empty() && !previous_had_comma {
            break;
        }
        let token = normalized(&tokens[cursor].lower);
        let Some(weekday) = parse_weekday(&token) else {
            break;
        };
        weekdays.push(weekday);
        end_idx = cursor;
        previous_had_comma = tokens[cursor].lower.ends_with(',');
        cursor += 1;
    }

    if weekdays.is_empty() {
        return None;
    }
    if weekdays.len() == 1 && !options.recurrence {
        return None;
    }
    Some(core(
        idx,
        end_idx,
        RecurrenceFrequency::Weekly,
        weekdays,
        None,
    ))
}

fn normalized(token: &str) -> String {
    token.trim_end_matches(',').to_string()
}

fn parse_interval_number(token: &str) -> Option<u16> {
    if token.is_empty() || !token.chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }
    token.parse::<u16>().ok()
}

fn parse_count_terminator(tokens: &[NlToken], next_idx: usize) -> Option<(usize, u16)> {
    if next_idx + 2 >= tokens.len() || normalized(&tokens[next_idx].lower) != "for" {
        return None;
    }
    let count = parse_interval_number(&normalized(&tokens[next_idx + 1].lower))?;
    if !(1..=999).contains(&count) {
        return None;
    }
    let unit = normalized(&tokens[next_idx + 2].lower);
    if matches!(
        unit.as_str(),
        "day"
            | "days"
            | "week"
            | "weeks"
            | "month"
            | "months"
            | "year"
            | "years"
            | "time"
            | "times"
            | "occurrence"
            | "occurrences"
    ) {
        Some((next_idx + 2, count))
    } else {
        None
    }
}

fn parse_until_terminator(
    tokens: &[NlToken],
    next_idx: usize,
    clock: &MenuSyntaxClock,
    consumed: &ConsumedTokens,
) -> Option<(usize, String)> {
    if next_idx + 1 >= tokens.len() || normalized(&tokens[next_idx].lower) != "until" {
        return None;
    }
    let mut best = None;
    for end_idx in next_idx + 1..tokens.len() {
        if (next_idx + 1..=end_idx).any(|idx| consumed.is_consumed(idx)) {
            break;
        }
        let raw = tokens[next_idx + 1..=end_idx]
            .iter()
            .map(|token| token.text.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        if let Some(resolved) = resolve_date_phrase(&raw, clock) {
            if let Some(until) = format_until(&resolved.iso) {
                best = Some((end_idx, until));
            }
        }
    }
    best
}

fn format_until(iso: &str) -> Option<String> {
    chrono::DateTime::parse_from_rfc3339(iso).ok().map(|dt| {
        dt.with_timezone(&chrono::Utc)
            .format("%Y%m%dT%H%M%SZ")
            .to_string()
    })
}

fn build_hit(
    tokens: &[NlToken],
    start_idx: usize,
    core: RecurrenceCore,
    clock: &MenuSyntaxClock,
    consumed_tokens: &ConsumedTokens,
) -> ParseHit<ResolvedRecurrencePhrase> {
    let mut end_idx = core.end_idx;
    let mut count = None;
    let mut until = None;
    if let Some((count_end_idx, parsed_count)) = parse_count_terminator(tokens, end_idx + 1) {
        end_idx = count_end_idx;
        count = Some(parsed_count);
    }
    if let Some((until_end_idx, parsed_until)) =
        parse_until_terminator(tokens, end_idx + 1, clock, consumed_tokens)
    {
        end_idx = until_end_idx;
        until = Some(parsed_until);
    }

    let span = (tokens[start_idx].span.0, tokens[end_idx].span.1);
    let source = tokens[start_idx..=end_idx]
        .iter()
        .map(|token| token.text.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    let rrule = rrule(
        core.frequency,
        &core.weekdays,
        core.interval,
        count,
        until.as_deref(),
    );
    let label = label(
        core.frequency,
        &core.weekdays,
        core.interval,
        count,
        until.as_deref(),
    );
    let recurrence = ResolvedRecurrence {
        source: source.clone(),
        source_span: span,
        frequency: core.frequency,
        weekdays: core.weekdays.clone(),
        rrule,
        label,
    };
    let consumed = (start_idx..=end_idx).collect::<Vec<_>>();
    ParseHit {
        value: ResolvedRecurrencePhrase {
            recurrence,
            anchor_weekdays: core.weekdays,
            source: source.clone(),
            source_span: span,
            token_indices: consumed.clone(),
        },
        fragment: MenuSyntaxFragment {
            role: MenuSyntaxFragmentRole::Recurrence,
            source,
            source_span: span,
            status: MenuSyntaxFragmentStatus::Resolved,
        },
        consumed,
    }
}

fn rrule(
    frequency: RecurrenceFrequency,
    weekdays: &[RecurrenceWeekday],
    interval: Option<u16>,
    count: Option<u16>,
    until: Option<&str>,
) -> String {
    let freq = match frequency {
        RecurrenceFrequency::Daily => "DAILY",
        RecurrenceFrequency::Weekly => "WEEKLY",
        RecurrenceFrequency::Monthly => "MONTHLY",
        RecurrenceFrequency::Yearly => "YEARLY",
    };
    let mut parts = vec![format!("FREQ={freq}")];
    if let Some(interval) = interval.filter(|interval| *interval > 1) {
        parts.push(format!("INTERVAL={interval}"));
    }
    if !weekdays.is_empty() {
        parts.push(format!(
            "BYDAY={}",
            weekdays
                .iter()
                .map(|weekday| weekday_rrule(*weekday))
                .collect::<Vec<_>>()
                .join(",")
        ));
    }
    if let Some(count) = count {
        parts.push(format!("COUNT={count}"));
    }
    if let Some(until) = until {
        parts.push(format!("UNTIL={until}"));
    }
    parts.join(";")
}

fn label(
    frequency: RecurrenceFrequency,
    weekdays: &[RecurrenceWeekday],
    interval: Option<u16>,
    count: Option<u16>,
    until: Option<&str>,
) -> String {
    let base = match frequency {
        RecurrenceFrequency::Daily => interval_label("day", "days", interval),
        RecurrenceFrequency::Monthly => interval_label("month", "months", interval),
        RecurrenceFrequency::Yearly => interval_label("year", "years", interval),
        RecurrenceFrequency::Weekly
            if weekdays == [RecurrenceWeekday::Sat, RecurrenceWeekday::Sun]
                && interval.is_none() =>
        {
            "every weekend".to_string()
        }
        RecurrenceFrequency::Weekly if weekdays.is_empty() => {
            interval_label("week", "weeks", interval)
        }
        RecurrenceFrequency::Weekly if weekdays.len() == 1 => {
            if interval == Some(2) {
                format!("every other {}", weekday_label(weekdays[0]))
            } else {
                format!("every {}", weekday_label(weekdays[0]))
            }
        }
        RecurrenceFrequency::Weekly => format!("every {}", join_weekday_labels(weekdays)),
    };
    let mut out = base;
    if let Some(count) = count {
        out.push_str(&format!(" for {count} occurrences"));
    }
    if let Some(until) = until {
        out.push_str(&format!(" until {until}"));
    }
    out
}

fn interval_label(singular: &str, plural: &str, interval: Option<u16>) -> String {
    match interval {
        Some(2) => format!("every other {singular}"),
        Some(interval) if interval > 1 => format!("every {interval} {plural}"),
        _ => format!("every {singular}"),
    }
}

fn join_weekday_labels(weekdays: &[RecurrenceWeekday]) -> String {
    let labels = weekdays
        .iter()
        .map(|weekday| weekday_label(*weekday))
        .collect::<Vec<_>>();
    match labels.as_slice() {
        [] => String::new(),
        [one] => (*one).to_string(),
        [first, second] => format!("{first} and {second}"),
        _ => {
            let Some((last, rest)) = labels.split_last() else {
                return String::new();
            };
            format!("{}, and {last}", rest.join(", ")).replace(", and", " and")
        }
    }
}

pub(super) fn weekday_rrule(weekday: RecurrenceWeekday) -> &'static str {
    match weekday {
        RecurrenceWeekday::Mon => "MO",
        RecurrenceWeekday::Tue => "TU",
        RecurrenceWeekday::Wed => "WE",
        RecurrenceWeekday::Thu => "TH",
        RecurrenceWeekday::Fri => "FR",
        RecurrenceWeekday::Sat => "SA",
        RecurrenceWeekday::Sun => "SU",
    }
}

fn weekday_label(weekday: RecurrenceWeekday) -> &'static str {
    match weekday {
        RecurrenceWeekday::Mon => "Monday",
        RecurrenceWeekday::Tue => "Tuesday",
        RecurrenceWeekday::Wed => "Wednesday",
        RecurrenceWeekday::Thu => "Thursday",
        RecurrenceWeekday::Fri => "Friday",
        RecurrenceWeekday::Sat => "Saturday",
        RecurrenceWeekday::Sun => "Sunday",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menu_syntax::nl_phrase::{tokenize, ConsumedTokens, NlParseOptions};
    use chrono_tz::America::Denver;

    fn clock() -> MenuSyntaxClock {
        MenuSyntaxClock::fixed("2026-04-26T09:00:00", Denver).expect("fixed clock")
    }

    fn parse(input: &str) -> ResolvedRecurrence {
        let tokens = tokenize(input);
        parse_recurrence(
            &tokens,
            &ConsumedTokens::new(tokens.len()),
            &clock(),
            &NlParseOptions::calendar_default(),
        )
        .expect("recurrence")
        .value
        .recurrence
    }

    #[test]
    fn every_day_resolves_daily_frequency_no_weekdays() {
        let recurrence = parse("every day");
        assert_eq!(recurrence.frequency, RecurrenceFrequency::Daily);
        assert!(recurrence.weekdays.is_empty());
        assert_eq!(recurrence.rrule, "FREQ=DAILY");
    }

    #[test]
    fn every_weekday_resolves_byday_mo_tu_we_th_fr() {
        let recurrence = parse("every weekday");
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
        assert_eq!(recurrence.rrule, "FREQ=WEEKLY;BYDAY=MO,TU,WE,TH,FR");
    }

    #[test]
    fn every_tuesday_and_thursday_resolves_byday_tu_th() {
        let recurrence = parse("every tuesday and thursday");
        assert_eq!(
            recurrence.weekdays,
            vec![RecurrenceWeekday::Tue, RecurrenceWeekday::Thu]
        );
        assert_eq!(recurrence.rrule, "FREQ=WEEKLY;BYDAY=TU,TH");
    }

    #[test]
    fn every_month_resolves_monthly_frequency() {
        let recurrence = parse("every month");
        assert_eq!(recurrence.frequency, RecurrenceFrequency::Monthly);
        assert_eq!(recurrence.rrule, "FREQ=MONTHLY");
    }

    #[test]
    fn weekly_resolves_without_byday() {
        let recurrence = parse("weekly");
        assert_eq!(recurrence.rrule, "FREQ=WEEKLY");
    }
}
