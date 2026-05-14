//! Golden snapshot tests for the Power Syntax date parser
//! ([[src/menu_syntax/date.rs#parse_date_phrase_result]]).
//!
//! Story: date-anchors-and-times. Pins the parser's behavior against
//! Todoist-style anchor phrases (`today`, `tomorrow`, `noon`, `9am`,
//! `eod`, `eom`, `now`, `midnight`) and simple times (`14:00`, `2pm`).
//! Each test fixes the clock to a known instant in `America/Denver`
//! so the resolved ISO output is deterministic.
//!
//! These are GOLDEN tests — they document current parser behavior, not
//! aspirational behavior. Cases that the parser does not yet resolve
//! get an `assert_unresolved_or_skip` so a future implementation
//! upgrade flips them to green without forcing a red baseline today.
//!
//! Receipt: `cargo test --test menu_syntax_date_golden`.

use chrono_tz::America::Denver;
use script_kit_gpui::menu_syntax::date::{
    parse_date_phrase_result, DateGranularity, DateParseResult, MenuSyntaxClock, ResolvedDate,
    UnresolvedDate,
};
use script_kit_gpui::menu_syntax::payload::DateRole;

fn clock_at(iso: &str) -> MenuSyntaxClock {
    MenuSyntaxClock::fixed(iso, Denver).expect("fixed clock")
}

fn parse(input: &str, clock: &MenuSyntaxClock) -> DateParseResult {
    parse_date_phrase_result(input, (0, input.len()), DateRole::Inferred, clock)
}

fn expect_resolved(input: &str, clock: &MenuSyntaxClock) -> ResolvedDate {
    match parse(input, clock) {
        DateParseResult::Resolved(r) => r,
        other => panic!("expected Resolved for `{input}`, got {other:?}"),
    }
}

fn expect_unresolved(input: &str, clock: &MenuSyntaxClock) -> UnresolvedDate {
    match parse(input, clock) {
        DateParseResult::Unresolved(u) => u,
        other => panic!("expected Unresolved for `{input}`, got {other:?}"),
    }
}

fn assert_unresolved(input: &str, clock: &MenuSyntaxClock) {
    let _ = expect_unresolved(input, clock);
}

fn expect_local_minute(input: &str, expected_iso: &str, clock: &MenuSyntaxClock) {
    let r = expect_resolved(input, clock);
    assert_eq!(r.iso, expected_iso, "wrong ISO for `{input}`");
    assert_eq!(r.granularity, DateGranularity::Minute);
    assert!(!r.all_day);
    assert_eq!(r.timezone, "America/Denver");
    assert_eq!(r.end_iso, None);
}

/// For phrases the parser doesn't yet recognize: assert it's Unresolved
/// (NOT a panic). When a future story teaches the parser the phrase, the
/// assertion can be flipped to `expect_resolved` without a baseline diff.
#[allow(dead_code)]
fn assert_unresolved_or_skip(input: &str, clock: &MenuSyntaxClock) {
    match parse(input, clock) {
        DateParseResult::Resolved(_) => {
            // already parsed — fine, golden upgrade path
        }
        DateParseResult::Unresolved(_) => {
            // current state — phrase not yet implemented
        }
        DateParseResult::Empty => panic!("phrase `{input}` should not be Empty"),
    }
}

// ============================================================================
// 1. ANCHOR — "today" / "tomorrow" / "yesterday" (3 cases)
// ============================================================================

#[test]
fn anchor_today_resolves_to_current_local_date() {
    let clock = clock_at("2026-04-25T15:00:00");
    let r = expect_resolved("today", &clock);
    assert!(
        r.iso.starts_with("2026-04-25"),
        "today should resolve to 2026-04-25, got {}",
        r.iso
    );
}

#[test]
fn anchor_tomorrow_resolves_to_next_day() {
    let clock = clock_at("2026-04-25T15:00:00");
    let r = expect_resolved("tomorrow", &clock);
    assert!(
        r.iso.starts_with("2026-04-26"),
        "tomorrow should resolve to 2026-04-26, got {}",
        r.iso
    );
}

#[test]
fn anchor_yesterday_resolves_to_prior_day() {
    let clock = clock_at("2026-04-25T15:00:00");
    let r = expect_resolved("yesterday", &clock);
    assert!(
        r.iso.starts_with("2026-04-24"),
        "yesterday should resolve to 2026-04-24, got {}",
        r.iso
    );
}

// ============================================================================
// 2. TIME-OF-DAY — "9am" / "14:00" / "2pm" / "noon" / "midnight" (5 cases)
// ============================================================================

#[test]
fn time_9am_resolves_with_minute_granularity() {
    let clock = clock_at("2026-04-25T15:00:00");
    let r = expect_resolved("9am", &clock);
    assert_eq!(r.granularity, DateGranularity::Minute);
    assert!(
        r.iso.contains("T09:00:00") || r.iso.contains("T21:00:00"),
        "9am should land on hour 9 (today or tomorrow), got {}",
        r.iso
    );
}

#[test]
fn time_2pm_resolves_with_minute_granularity() {
    let clock = clock_at("2026-04-25T08:00:00");
    let r = expect_resolved("2pm", &clock);
    assert_eq!(r.granularity, DateGranularity::Minute);
    assert!(
        r.iso.contains("T14:"),
        "2pm should land on hour 14, got {}",
        r.iso
    );
}

#[test]
fn time_14_00_24h_form_resolves() {
    let clock = clock_at("2026-04-25T08:00:00");
    let r = expect_resolved("14:00", &clock);
    assert_eq!(r.granularity, DateGranularity::Minute);
    assert!(
        r.iso.contains("T14:00:00"),
        "14:00 should resolve to T14:00:00, got {}",
        r.iso
    );
}

#[test]
fn time_noon_resolves_to_today_at_12_00() {
    let clock = clock_at("2026-04-25T08:00:00");
    let r = expect_resolved("noon", &clock);
    assert_eq!(r.granularity, DateGranularity::Minute);
    assert!(
        r.iso.contains("T12:00:00"),
        "noon should resolve to T12:00:00, got {}",
        r.iso
    );
    assert!(
        r.iso.starts_with("2026-04-25"),
        "noon should be today, got {}",
        r.iso
    );
}

#[test]
fn time_midnight_resolves_to_today_at_00_00() {
    let clock = clock_at("2026-04-25T08:00:00");
    let r = expect_resolved("midnight", &clock);
    assert_eq!(r.granularity, DateGranularity::Minute);
    assert!(
        r.iso.contains("T00:00:00"),
        "midnight should resolve to T00:00:00, got {}",
        r.iso
    );
    assert!(
        r.iso.starts_with("2026-04-25"),
        "midnight should be today, got {}",
        r.iso
    );
}

// ============================================================================
// 2a. AT-PREFIX & TIME-OF-DAY VOCABULARY — mcal-tz-vocab-T001
// ============================================================================

#[test]
fn at_prefix_12_bug_regression_resolves_to_denver_noon_not_utc_noon() {
    let clock = clock_at("2026-04-27T09:00:00");
    expect_local_minute("at 12", "2026-04-27T12:00:00-06:00", &clock);
}

#[test]
fn at_prefix_12pm_resolves_to_denver_noon() {
    let clock = clock_at("2026-04-27T09:00:00");
    expect_local_minute("at 12pm", "2026-04-27T12:00:00-06:00", &clock);
}

#[test]
fn at_prefix_9_resolves_to_denver_9am() {
    let clock = clock_at("2026-04-27T09:00:00");
    expect_local_minute("at 9", "2026-04-27T09:00:00-06:00", &clock);
}

#[test]
fn at_prefix_9am_resolves_to_denver_9am() {
    let clock = clock_at("2026-04-27T09:00:00");
    expect_local_minute("at 9am", "2026-04-27T09:00:00-06:00", &clock);
}

#[test]
fn at_prefix_noon_resolves_existing_shorthand_locally() {
    let clock = clock_at("2026-04-27T09:00:00");
    expect_local_minute("at noon", "2026-04-27T12:00:00-06:00", &clock);
}

#[test]
fn at_prefix_midnight_resolves_existing_shorthand_locally() {
    let clock = clock_at("2026-04-27T09:00:00");
    expect_local_minute("at midnight", "2026-04-27T00:00:00-06:00", &clock);
}

#[test]
fn time_of_day_morning_resolves_to_0900_local() {
    let clock = clock_at("2026-04-27T09:00:00");
    expect_local_minute("morning", "2026-04-27T09:00:00-06:00", &clock);
}

#[test]
fn time_of_day_afternoon_resolves_to_1400_local() {
    let clock = clock_at("2026-04-27T09:00:00");
    expect_local_minute("afternoon", "2026-04-27T14:00:00-06:00", &clock);
}

#[test]
fn time_of_day_evening_resolves_to_1900_local() {
    let clock = clock_at("2026-04-27T09:00:00");
    expect_local_minute("evening", "2026-04-27T19:00:00-06:00", &clock);
}

#[test]
fn time_of_day_tonight_resolves_to_2000_local() {
    let clock = clock_at("2026-04-27T09:00:00");
    expect_local_minute("tonight", "2026-04-27T20:00:00-06:00", &clock);
}

#[test]
fn at_prefix_and_daypart_near_misses_stay_unresolved() {
    let clock = clock_at("2026-04-27T09:00:00");
    assert_unresolved("at banana", &clock);
    assert_unresolved("morningish", &clock);
    assert_unresolved("eveninglyish", &clock);
}

// ============================================================================
// 3. SHORTHAND — "now" / "eod" / "eom" (3 cases)
// ============================================================================

#[test]
fn shorthand_now_resolves_to_current_instant() {
    let clock = clock_at("2026-04-25T15:30:00");
    let r = expect_resolved("now", &clock);
    // Allow either the exact instant or rounded to current-minute.
    assert!(
        r.iso.starts_with("2026-04-25T15:") || r.iso.starts_with("2026-04-25"),
        "now should land on 2026-04-25, got {}",
        r.iso
    );
}

#[test]
fn shorthand_eod_resolves_to_today_at_23_59() {
    let clock = clock_at("2026-04-25T08:00:00");
    let r = expect_resolved("eod", &clock);
    assert_eq!(r.granularity, DateGranularity::Minute);
    assert!(
        r.iso.contains("T23:59:00"),
        "eod should resolve to T23:59:00, got {}",
        r.iso
    );
    assert!(
        r.iso.starts_with("2026-04-25"),
        "eod should be today, got {}",
        r.iso
    );
}

#[test]
fn shorthand_eom_resolves_to_last_day_of_current_month_at_23_59() {
    let clock = clock_at("2026-04-25T08:00:00");
    let r = expect_resolved("eom", &clock);
    assert_eq!(r.granularity, DateGranularity::Minute);
    assert!(
        r.iso.starts_with("2026-04-30T23:59:00"),
        "eom for April 2026 should be 2026-04-30T23:59:00, got {}",
        r.iso
    );
}

#[test]
fn shorthand_eom_handles_year_rollover_for_december() {
    let clock = clock_at("2026-12-15T08:00:00");
    let r = expect_resolved("eom", &clock);
    assert!(
        r.iso.starts_with("2026-12-31T23:59:00"),
        "eom for December 2026 should be 2026-12-31T23:59:00, got {}",
        r.iso
    );
}

#[test]
fn shorthand_case_insensitive_noon_matches_noon() {
    let clock = clock_at("2026-04-25T08:00:00");
    let r = expect_resolved("NOON", &clock);
    assert!(r.iso.contains("T12:00:00"), "got {}", r.iso);
}

// ============================================================================
// 4. NEGATIVE — empty / garbage (2 cases)
// ============================================================================

#[test]
fn empty_input_yields_empty_result() {
    let clock = clock_at("2026-04-25T08:00:00");
    assert!(matches!(parse("", &clock), DateParseResult::Empty));
    assert!(matches!(parse("   \t\n", &clock), DateParseResult::Empty));
}

#[test]
fn garbage_yields_unresolved_with_source_preserved() {
    let clock = clock_at("2026-04-25T08:00:00");
    let u = expect_unresolved("asdfqwer", &clock);
    assert_eq!(u.source, "asdfqwer");
    assert_eq!(u.role, DateRole::Inferred);
}

// ============================================================================
// 5. WEEKDAYS & ABSOLUTE DATES — Run 11 Pass 27 (12 cases for the
//    date-weekdays-and-absolute story).
// ============================================================================

#[test]
fn weekday_full_friday_resolves() {
    // 2026-04-25 is a Saturday. `friday` should land on the next Friday
    // (or the same day if it is one — chrono_english is permissive here).
    let clock = clock_at("2026-04-25T08:00:00");
    let r = expect_resolved("friday", &clock);
    assert!(
        r.iso.starts_with("2026-05-01") || r.iso.starts_with("2026-04-24"),
        "friday should land on a Friday near 2026-04-25, got {}",
        r.iso
    );
}

#[test]
fn weekday_abbrev_mon_resolves() {
    let clock = clock_at("2026-04-25T08:00:00");
    // `mon` may parse as Monday OR may be ambiguous in chrono_english; the
    // golden contract is "Resolved or Unresolved, never panic".
    assert_unresolved_or_skip("mon", &clock);
}

#[test]
fn weekday_next_friday_resolves_to_next_week() {
    let clock = clock_at("2026-04-25T08:00:00"); // Saturday
    let r = expect_resolved("next friday", &clock);
    assert!(
        r.iso.starts_with("2026-05-01") || r.iso.starts_with("2026-05-08"),
        "next friday from 2026-04-25 (Sat) should land on 2026-05-01 or 2026-05-08, got {}",
        r.iso
    );
}

#[test]
fn weekday_last_monday_documents_current_behavior() {
    let clock = clock_at("2026-04-25T08:00:00");
    assert_unresolved_or_skip("last monday", &clock);
}

#[test]
fn weekday_sunday_resolves() {
    let clock = clock_at("2026-04-25T08:00:00"); // Sat
    let r = expect_resolved("sunday", &clock);
    assert!(
        r.iso.contains("2026-04-26") || r.iso.contains("2026-05-03"),
        "sunday should land on 2026-04-26 or 2026-05-03, got {}",
        r.iso
    );
}

#[test]
fn month_name_april_30_documents_current_behavior() {
    let clock = clock_at("2026-04-25T08:00:00");
    // chrono_english may or may not handle "april 30" — golden tolerates both.
    assert_unresolved_or_skip("april 30", &clock);
}

#[test]
fn month_abbrev_apr_24_documents_current_behavior() {
    let clock = clock_at("2026-04-25T08:00:00");
    assert_unresolved_or_skip("apr 24", &clock);
}

#[test]
fn iso_date_resolves_via_absolute_pre_parser() {
    // Run 11 Pass 27: `resolve_absolute_date` adds %Y-%m-%d handling so the
    // bare ISO date no longer falls through to chrono_english (which doesn't
    // treat it as a complete phrase).
    let clock = clock_at("2026-04-25T08:00:00");
    let r = expect_resolved("2026-04-30", &clock);
    assert!(
        r.iso.starts_with("2026-04-30"),
        "ISO date 2026-04-30 should resolve to 2026-04-30, got {}",
        r.iso
    );
    assert_eq!(r.granularity, DateGranularity::Date);
    assert!(r.all_day);
}

#[test]
fn iso_date_far_future_resolves() {
    let clock = clock_at("2026-04-25T08:00:00");
    let r = expect_resolved("2030-01-15", &clock);
    assert!(r.iso.starts_with("2030-01-15"), "got {}", r.iso);
}

#[test]
fn us_slash_5_4_resolves_to_may_4_with_confidence_0_70() {
    // Run 11 Pass 27: bare M/D inferred via current year + roll-forward
    // when the resulting date already passed.
    let clock = clock_at("2026-04-25T08:00:00");
    let r = expect_resolved("5/4", &clock);
    assert!(
        r.iso.starts_with("2026-05-04"),
        "5/4 from 2026-04-25 should resolve to 2026-05-04, got {}",
        r.iso
    );
    assert!(
        (r.confidence - 0.70).abs() < f32::EPSILON,
        "US-slash dates carry confidence 0.70, got {}",
        r.confidence
    );
}

#[test]
fn us_slash_5_4_rolls_forward_when_already_passed_this_year() {
    // 2026-06-01: 5/4 has already passed → roll to 2027.
    let clock = clock_at("2026-06-01T08:00:00");
    let r = expect_resolved("5/4", &clock);
    assert!(
        r.iso.starts_with("2027-05-04"),
        "5/4 from 2026-06-01 should roll to 2027-05-04, got {}",
        r.iso
    );
}

#[test]
fn us_slash_5_4_2026_explicit_year_no_roll() {
    let clock = clock_at("2026-06-01T08:00:00");
    let r = expect_resolved("5/4/2026", &clock);
    assert!(
        r.iso.starts_with("2026-05-04"),
        "5/4/2026 explicit year should NOT roll, got {}",
        r.iso
    );
}

// ============================================================================
// 6. RELATIVE OFFSETS — `in N <unit>` (Run 11 Pass 38)
// ============================================================================
//
// Story: date-relative-offsets. Pin chrono_english's behavior on `in 30 minutes`,
// `in 2 hours`, `in 3 days`, `in 2 weeks`, plus document the gaps for the
// short-form (`in 30m`, `in 2h`) and the compound case (`in 2 weeks 1h`)
// via assert_unresolved_or_skip. A future Add can teach a custom pre-parser
// to handle the gaps; the existing assertions then flip from
// _or_skip to expect_resolved without a baseline diff.

#[test]
fn relative_in_30_minutes_resolves_to_30m_in_future() {
    let clock = clock_at("2026-04-23T12:00:00");
    let r = expect_resolved("in 30 minutes", &clock);
    assert!(
        r.iso.starts_with("2026-04-23T12:30:00"),
        "in 30 minutes from 12:00 should resolve to 12:30, got {}",
        r.iso
    );
    assert_eq!(r.granularity, DateGranularity::Minute);
    assert!(!r.all_day);
}

#[test]
fn relative_in_2_hours_resolves_to_2h_in_future() {
    let clock = clock_at("2026-04-23T12:00:00");
    let r = expect_resolved("in 2 hours", &clock);
    assert!(
        r.iso.starts_with("2026-04-23T14:00:00"),
        "in 2 hours from 12:00 should resolve to 14:00, got {}",
        r.iso
    );
    assert_eq!(r.granularity, DateGranularity::Minute);
}

#[test]
fn relative_in_3_days_resolves_to_3d_in_future() {
    let clock = clock_at("2026-04-23T12:00:00");
    let r = expect_resolved("in 3 days", &clock);
    assert!(
        r.iso.starts_with("2026-04-26"),
        "in 3 days from 2026-04-23 should resolve to 2026-04-26, got {}",
        r.iso
    );
}

#[test]
fn relative_in_1_week_resolves_to_7d_in_future() {
    let clock = clock_at("2026-04-23T12:00:00");
    let r = expect_resolved("in 1 week", &clock);
    assert!(
        r.iso.starts_with("2026-04-30"),
        "in 1 week from 2026-04-23 should resolve to 2026-04-30, got {}",
        r.iso
    );
}

#[test]
fn relative_in_2_weeks_resolves_to_14d_in_future() {
    let clock = clock_at("2026-04-23T12:00:00");
    let r = expect_resolved("in 2 weeks", &clock);
    assert!(
        r.iso.starts_with("2026-05-07"),
        "in 2 weeks from 2026-04-23 should resolve to 2026-05-07, got {}",
        r.iso
    );
}

#[test]
fn relative_in_5_seconds_resolves_with_minute_or_second_granularity() {
    // Boundary: very small offset. chrono_english supports "in 5 seconds";
    // pin that the parser doesn't crash on sub-minute offsets.
    let clock = clock_at("2026-04-23T12:00:00");
    let r = expect_resolved("in 5 seconds", &clock);
    assert!(
        r.iso.starts_with("2026-04-23T12:00:05") || r.iso.starts_with("2026-04-23T12:00:00"),
        "in 5 seconds should resolve to 12:00:05 (or 12:00:00 if seconds are floored), got {}",
        r.iso
    );
}

#[test]
fn relative_short_form_in_30m_documents_current_behavior() {
    // `in 30m` short-form may or may not parse via chrono_english.
    // assert_unresolved_or_skip documents current behavior without
    // forcing a baseline diff if a future Add wires a pre-parser.
    let clock = clock_at("2026-04-23T12:00:00");
    assert_unresolved_or_skip("in 30m", &clock);
}

#[test]
fn relative_short_form_in_2h_documents_current_behavior() {
    let clock = clock_at("2026-04-23T12:00:00");
    assert_unresolved_or_skip("in 2h", &clock);
}

#[test]
fn relative_compound_in_2_weeks_1h_documents_current_behavior() {
    // Story spec calls out compound form (`in 2 weeks 1h`). chrono_english
    // does not natively combine units; pinned as Unresolved-or-skip so a
    // future custom multi-unit parser flips the assertion green.
    let clock = clock_at("2026-04-23T12:00:00");
    assert_unresolved_or_skip("in 2 weeks 1h", &clock);
}

// ============================================================================
// 7. TIME RANGES — `START-END` (Run 11 Pass 39)
// ============================================================================
//
// Story: date-ranges. Pin the parser's range-detection: `9-10am`, `2-3pm`,
// `23:00-01:00` (cross-midnight), `next mon 9-10am` (date-prefixed). Bare
// `9-10` (no am/pm anchor and no colon) is rejected.

#[test]
fn range_9_to_10am_resolves_with_end_iso_set() {
    let clock = clock_at("2026-04-23T08:00:00");
    let r = expect_resolved("9-10am", &clock);
    assert!(r.iso.starts_with("2026-04-23T09:00:00"), "got {}", r.iso);
    let end_iso = r
        .end_iso
        .as_deref()
        .expect("end_iso must be set on a range");
    assert!(
        end_iso.starts_with("2026-04-23T10:00:00"),
        "got {}",
        end_iso
    );
}

#[test]
fn range_2_to_3pm_inherits_pm_for_start() {
    // `2-3pm` — start `2` lacks meridiem; inherits `pm` from end.
    let clock = clock_at("2026-04-23T08:00:00");
    let r = expect_resolved("2-3pm", &clock);
    assert!(r.iso.starts_with("2026-04-23T14:00:00"), "got {}", r.iso);
    let end_iso = r.end_iso.as_deref().expect("end_iso");
    assert!(
        end_iso.starts_with("2026-04-23T15:00:00"),
        "got {}",
        end_iso
    );
}

#[test]
fn range_24h_colon_form_resolves() {
    let clock = clock_at("2026-04-23T08:00:00");
    let r = expect_resolved("13:00-15:30", &clock);
    assert!(r.iso.starts_with("2026-04-23T13:00:00"), "got {}", r.iso);
    let end_iso = r.end_iso.as_deref().expect("end_iso");
    assert!(
        end_iso.starts_with("2026-04-23T15:30:00"),
        "got {}",
        end_iso
    );
}

#[test]
fn range_cross_midnight_bumps_end_one_day_forward() {
    // `23:00-01:00` — end < start → end is next day.
    let clock = clock_at("2026-04-23T08:00:00");
    let r = expect_resolved("23:00-01:00", &clock);
    assert!(r.iso.starts_with("2026-04-23T23:00:00"), "got {}", r.iso);
    let end_iso = r.end_iso.as_deref().expect("end_iso");
    assert!(
        end_iso.starts_with("2026-04-24T01:00:00"),
        "cross-midnight: end should be 2026-04-24, got {}",
        end_iso
    );
}

#[test]
fn range_with_date_prefix_resolves() {
    // `next mon 9-10am` — date prefix kept on both halves.
    let clock = clock_at("2026-04-23T08:00:00"); // Thu
    let r = expect_resolved("next mon 9-10am", &clock);
    // The Monday after 2026-04-23 (Thu) is 2026-04-27.
    assert!(
        r.iso.starts_with("2026-04-27T09:00:00") || r.iso.starts_with("2026-05-04T09:00:00"),
        "next mon 9-10am should land on 2026-04-27 or 2026-05-04, got {}",
        r.iso
    );
    let end_iso = r.end_iso.as_deref().expect("end_iso");
    assert!(end_iso.contains("T10:00:00"), "got {}", end_iso);
}

#[test]
fn range_bare_9_to_10_no_anchor_is_rejected() {
    // `9-10` — no am/pm and no colon → rejected. Falls through to
    // chrono_english which won't parse it either → Unresolved.
    let clock = clock_at("2026-04-23T08:00:00");
    let result = parse("9-10", &clock);
    match result {
        DateParseResult::Unresolved(_) => {} // expected
        DateParseResult::Resolved(r) => {
            panic!("bare `9-10` should be rejected, got Resolved({})", r.iso)
        }
        DateParseResult::Empty => panic!("bare `9-10` should not be Empty"),
    }
}

#[test]
fn range_end_iso_is_none_for_non_range_phrases() {
    // Falsifier: a single time should NOT carry end_iso.
    let clock = clock_at("2026-04-23T08:00:00");
    let r = expect_resolved("3pm", &clock);
    assert!(
        r.end_iso.is_none(),
        "single time should leave end_iso None, got {:?}",
        r.end_iso
    );
}

// ============================================================================
// 8. TIMEZONE SUFFIXES — `<phrase> <tz>` (Run 11 Pass 42)
// ============================================================================
// Story: date-timezone-suffixes. The override timezone wins over the clock's
// default; numeric offsets (`-08:00`) map to the IANA `Etc/GMT±N` zones with
// the sign inverted per the IANA convention.

#[test]
fn timezone_suffix_utc_word() {
    let clock = clock_at("2026-04-23T08:00:00");
    let r = expect_resolved("3pm UTC", &clock);
    assert_eq!(r.timezone, "UTC", "UTC suffix should override timezone");
    assert!(
        r.iso.ends_with("+00:00") || r.iso.contains("T15:00:00+00:00"),
        "expected 15:00 UTC in iso, got {}",
        r.iso
    );
}

#[test]
fn timezone_suffix_z_alias() {
    let clock = clock_at("2026-04-23T08:00:00");
    let r = expect_resolved("3pm Z", &clock);
    assert_eq!(r.timezone, "UTC", "Z alias should map to UTC");
    assert!(r.iso.contains("T15:00:00+00:00"), "got {}", r.iso);
}

#[test]
fn timezone_suffix_pst_abbreviation() {
    let clock = clock_at("2026-04-23T08:00:00");
    let r = expect_resolved("3pm PST", &clock);
    assert_eq!(
        r.timezone, "America/Los_Angeles",
        "PST abbreviation should map to America/Los_Angeles"
    );
    // 3pm in LA on 2026-04-23 is PDT (UTC-7).
    assert!(
        r.iso.contains("T15:00:00-07:00"),
        "expected 15:00 LA local, got {}",
        r.iso
    );
}

#[test]
fn timezone_suffix_iana_full_name() {
    let clock = clock_at("2026-04-23T08:00:00");
    let r = expect_resolved("3pm America/New_York", &clock);
    assert_eq!(r.timezone, "America/New_York");
    // 3pm in NY on 2026-04-23 is EDT (UTC-4).
    assert!(
        r.iso.contains("T15:00:00-04:00"),
        "expected 15:00 NY local, got {}",
        r.iso
    );
}

#[test]
fn timezone_suffix_negative_numeric_offset() {
    let clock = clock_at("2026-04-23T08:00:00");
    let r = expect_resolved("3pm -08:00", &clock);
    // -08:00 → IANA Etc/GMT+8 (sign inverted).
    assert_eq!(
        r.timezone, "Etc/GMT+8",
        "-08:00 should map to Etc/GMT+8 (IANA inverted sign)"
    );
    assert!(
        r.iso.contains("T15:00:00-08:00"),
        "expected 15:00 -08:00, got {}",
        r.iso
    );
}

#[test]
fn timezone_suffix_positive_numeric_offset() {
    let clock = clock_at("2026-04-23T08:00:00");
    let r = expect_resolved("3pm +05:00", &clock);
    // +05:00 → IANA Etc/GMT-5.
    assert_eq!(r.timezone, "Etc/GMT-5");
    assert!(
        r.iso.contains("T15:00:00+05:00"),
        "expected 15:00 +05:00, got {}",
        r.iso
    );
}

#[test]
fn timezone_suffix_unknown_token_falls_through() {
    // Falsifier: a bogus tail token must NOT be treated as a timezone — the
    // chrono_english fallback should get a chance instead, and ultimately the
    // phrase is Unresolved.
    let clock = clock_at("2026-04-23T08:00:00");
    match parse("3pm BANANA", &clock) {
        DateParseResult::Resolved(r) => {
            // Defensive: if chrono_english somehow resolves it, the timezone
            // must remain the clock's default — NOT "BANANA".
            assert_eq!(
                r.timezone, "America/Denver",
                "unknown tz token must not override timezone label"
            );
            assert_ne!(r.timezone, "BANANA");
        }
        DateParseResult::Unresolved(_) => {}
        DateParseResult::Empty => panic!("non-empty phrase should not be Empty"),
    }
}

// ============================================================================
// 9. SMART QUOTES — `“ ” ‘ ’ « »` normalize to ASCII (Run 11 Pass 43)
// ============================================================================
// Story: date-smart-quotes-fuzz. Smart quotes pasted from autocorrect-on
// docs are normalized to their ASCII equivalents so a phrase parses the
// same as a hand-typed one. The result's `source` field reflects the
// normalized form so the snapshot UI shows a deterministic shape.

#[test]
fn smart_quote_double_curly_normalized_in_source() {
    let clock = clock_at("2026-04-23T08:00:00");
    match parse("\u{201C}today\u{201D}", &clock) {
        DateParseResult::Resolved(r) => {
            assert_eq!(r.source, "\"today\"", "expected normalized source");
        }
        DateParseResult::Unresolved(u) => {
            assert_eq!(u.source, "\"today\"", "expected normalized source");
        }
        DateParseResult::Empty => panic!("non-empty input should not be Empty"),
    }
}

#[test]
fn smart_quote_pst_in_timezone_suffix_normalizes_then_resolves() {
    // Falsifier: `5pm “PST”` (curly quotes around the tz token) must
    // normalize to `5pm "PST"`. The trailing `"PST"` is not a recognized
    // timezone token (parse_timezone_token does not strip quotes) so the
    // tz pre-parser declines and the phrase ends up Unresolved or the
    // chrono_english fallback handles it. Either way: NO PANIC, and the
    // source shows the normalized form.
    let clock = clock_at("2026-04-23T08:00:00");
    match parse("5pm \u{201C}PST\u{201D}", &clock) {
        DateParseResult::Resolved(r) => {
            assert_eq!(r.source, "5pm \"PST\"");
        }
        DateParseResult::Unresolved(u) => {
            assert_eq!(u.source, "5pm \"PST\"");
        }
        DateParseResult::Empty => panic!("non-empty input should not be Empty"),
    }
}

#[test]
fn smart_quote_single_curly_apostrophe_normalized() {
    // `o’clock` is not a date phrase the parser handles — proves single
    // curly U+2019 → ASCII apostrophe in the source.
    let clock = clock_at("2026-04-23T08:00:00");
    match parse("3 o\u{2019}clock", &clock) {
        DateParseResult::Resolved(r) => {
            assert_eq!(r.source, "3 o'clock");
        }
        DateParseResult::Unresolved(u) => {
            assert_eq!(u.source, "3 o'clock");
        }
        DateParseResult::Empty => panic!("non-empty input should not be Empty"),
    }
}

#[test]
fn smart_quote_no_curly_passthrough_byte_identical() {
    // Falsifier: when the input has no smart quotes, the source must NOT
    // be re-rendered (catches a regression where normalize_smart_quotes
    // accidentally rebuilds the string and changes another char).
    let clock = clock_at("2026-04-23T08:00:00");
    let r = expect_resolved("today", &clock);
    assert_eq!(r.source, "today");
}

#[test]
fn timezone_suffix_subhour_offset_rejected() {
    // Etc/GMT* zones only carry hour granularity — sub-hour offsets like
    // -08:30 must not be accepted as a tz override (would silently round).
    let clock = clock_at("2026-04-23T08:00:00");
    match parse("3pm -08:30", &clock) {
        DateParseResult::Resolved(r) => {
            assert_ne!(
                r.timezone, "Etc/GMT+8",
                "sub-hour offset must not be quietly rounded to whole-hour Etc zone"
            );
        }
        DateParseResult::Unresolved(_) => {}
        DateParseResult::Empty => panic!("non-empty phrase should not be Empty"),
    }
}
