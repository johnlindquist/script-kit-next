//! Golden tests for the menu syntax natural-language recurrence parser.
//!
//! Receipt: `cargo test --test menu_syntax_recurrence_golden`.

use chrono_tz::America::Denver;
use script_kit_gpui::menu_syntax::{
    parse_capture, resolve_capture_dates_with_accepts, CaptureParse, MenuSyntaxClock,
    ResolvedCaptureInvocation,
};

fn clock() -> MenuSyntaxClock {
    MenuSyntaxClock::fixed("2026-04-26T09:00:00", Denver).expect("fixed clock")
}

fn resolve(body: &str) -> ResolvedCaptureInvocation {
    let input = format!(";mcal {body}");
    let invocation = match parse_capture(&input) {
        CaptureParse::Ok(invocation) => invocation,
        CaptureParse::Incomplete(state) => panic!("incomplete: {state:?}"),
    };
    resolve_capture_dates_with_accepts(&invocation, &clock(), &[])
}

fn parse(body: &str) -> Option<String> {
    resolve(body).recurrence.map(|recurrence| recurrence.rrule)
}

#[test]
fn recurrence_golden_rrules() {
    const CASES: &[(&str, Option<&str>)] = &[
        ("Team sync tomorrow 9am weekly", Some("FREQ=WEEKLY")),
        (
            "Team sync tomorrow 9am biweekly",
            Some("FREQ=WEEKLY;INTERVAL=2"),
        ),
        (
            "Team sync tomorrow 9am bimonthly",
            Some("FREQ=MONTHLY;INTERVAL=2"),
        ),
        (
            "Team sync tomorrow 9am fortnightly",
            Some("FREQ=WEEKLY;INTERVAL=2"),
        ),
        ("Team sync tomorrow 9am every week", Some("FREQ=WEEKLY")),
        (
            "Team sync tomorrow 9am every weekend",
            Some("FREQ=WEEKLY;BYDAY=SA,SU"),
        ),
        (
            "Team sync tomorrow 9am every other day",
            Some("FREQ=DAILY;INTERVAL=2"),
        ),
        (
            "Team sync tomorrow 9am every other week",
            Some("FREQ=WEEKLY;INTERVAL=2"),
        ),
        (
            "Team sync tomorrow 9am every other month",
            Some("FREQ=MONTHLY;INTERVAL=2"),
        ),
        (
            "Team sync tomorrow 9am every other year",
            Some("FREQ=YEARLY;INTERVAL=2"),
        ),
        (
            "Team sync tomorrow 9am every other mon",
            Some("FREQ=WEEKLY;INTERVAL=2;BYDAY=MO"),
        ),
        ("Team sync tomorrow 9am every 1 day", Some("FREQ=DAILY")),
        (
            "Team sync tomorrow 9am every 2 days",
            Some("FREQ=DAILY;INTERVAL=2"),
        ),
        (
            "Team sync tomorrow 9am every 2 weeks",
            Some("FREQ=WEEKLY;INTERVAL=2"),
        ),
        (
            "Team sync tomorrow 9am every 2 months",
            Some("FREQ=MONTHLY;INTERVAL=2"),
        ),
        (
            "Team sync tomorrow 9am every 2 years",
            Some("FREQ=YEARLY;INTERVAL=2"),
        ),
        (
            "Team sync tomorrow 9am every 30 weeks",
            Some("FREQ=WEEKLY;INTERVAL=30"),
        ),
        (
            "Team sync tomorrow 9am every mon, tue and wed",
            Some("FREQ=WEEKLY;BYDAY=MO,TU,WE"),
        ),
        (
            "Team sync tomorrow 9am repeat every week",
            Some("FREQ=WEEKLY"),
        ),
        ("Team sync tomorrow 9am repeats weekly", Some("FREQ=WEEKLY")),
        (
            "Team sync tomorrow 9am repeating biweekly",
            Some("FREQ=WEEKLY;INTERVAL=2"),
        ),
        (
            "Team sync tomorrow 9am recurring every mon",
            Some("FREQ=WEEKLY;BYDAY=MO"),
        ),
        (
            "Team sync tomorrow 9am recurs every other month",
            Some("FREQ=MONTHLY;INTERVAL=2"),
        ),
        (
            "Team sync tomorrow 9am every week for 4 weeks",
            Some("FREQ=WEEKLY;COUNT=4"),
        ),
        (
            "Team sync tomorrow 9am every week for 4 times",
            Some("FREQ=WEEKLY;COUNT=4"),
        ),
        (
            "Team sync tomorrow 9am every week for 4 occurrences",
            Some("FREQ=WEEKLY;COUNT=4"),
        ),
        (
            "Team sync tomorrow 9am every week until 2026-05-31",
            Some("FREQ=WEEKLY;UNTIL=20260531T060000Z"),
        ),
        ("Team sync tomorrow 9am repeat", None),
        ("Team sync tomorrow 9am every 0 weeks", None),
        ("Team sync tomorrow 9am every 31 weeks", None),
        ("Team sync tomorrow 9am every 2 bananas", None),
    ];

    for (body, expected) in CASES {
        assert_eq!(parse(body).as_deref(), *expected, "{body}");
    }
}

#[test]
fn recurrence_count_takes_precedence_over_duration() {
    let resolved = resolve("Team sync tomorrow 9am every week for 4 weeks");
    assert_eq!(
        resolved
            .recurrence
            .as_ref()
            .map(|recurrence| recurrence.rrule.as_str()),
        Some("FREQ=WEEKLY;COUNT=4")
    );
    assert!(
        resolved.duration_resolved.is_none(),
        "for 4 weeks should be consumed as recurrence count"
    );
}

#[test]
fn recurrence_weekend_label_is_normalized() {
    let resolved = resolve("Team sync tomorrow 9am every weekend");
    assert_eq!(
        resolved
            .recurrence
            .as_ref()
            .map(|recurrence| recurrence.label.as_str()),
        Some("every weekend")
    );
}

#[test]
fn recurrence_comma_weekday_label_is_normalized() {
    let resolved = resolve("Team sync tomorrow 9am every mon, tue and wed");
    assert_eq!(
        resolved
            .recurrence
            .as_ref()
            .map(|recurrence| recurrence.label.as_str()),
        Some("every Monday, Tuesday and Wednesday")
    );
}
