//! Run 11 Pass 40 — attacker probe of the two pre-parsers added in
//! Pass 38 ([[src/menu_syntax/date.rs#resolve_relative_offset]]) and
//! Pass 39 ([[src/menu_syntax/date.rs#resolve_time_range]]). Combined
//! attack — both surfaces are fresh as primary targets.
//!
//! Categories: Boundary, Composition, Resurrection. Actions: 22.

use chrono_tz::America::Denver;
use script_kit_gpui::menu_syntax::date::{
    parse_date_phrase_result, DateGranularity, DateParseResult, MenuSyntaxClock, ResolvedDate,
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

// ============================================================================
// BOUNDARY (8 actions)
// ============================================================================

#[test]
fn boundary_01_in_0_minutes_resolves_to_exactly_now() {
    let clock = clock_at("2026-04-23T12:00:00");
    let r = expect_resolved("in 0 minutes", &clock);
    // Zero offset → same as now; allow same minute (12:00:00..12:00:59).
    assert!(r.iso.starts_with("2026-04-23T12:00:"), "got {}", r.iso);
}

#[test]
fn boundary_02_in_negative_days_currently_parses_negative_offset() {
    // Pinned current behavior: `i64::parse` accepts `-3` so the helper
    // produces a PAST timestamp. Whether negative offsets are a feature
    // or a bug is undecided — pinning current behavior here so a future
    // call-site validation that rejects negatives flips this test in
    // place rather than silently changing semantics.
    let clock = clock_at("2026-04-23T12:00:00");
    let r = expect_resolved("in -3 days", &clock);
    assert!(
        r.iso.starts_with("2026-04-20"),
        "in -3 days from 2026-04-23 should be 2026-04-20 with current behavior, got {}",
        r.iso
    );
}

#[test]
fn boundary_03_in_huge_overflow_returns_unresolved() {
    // `i64::MAX seconds` overflow: `n.checked_mul(secs_per)` should None.
    let clock = clock_at("2026-04-23T12:00:00");
    match parse("in 99999999999999999999 days", &clock) {
        DateParseResult::Unresolved(_) => {} // expected — i64 parse fails
        DateParseResult::Resolved(r) => panic!("overflow should not resolve; got {}", r.iso),
        DateParseResult::Empty => panic!("non-empty input"),
    }
}

#[test]
fn boundary_04_in_extra_whitespace_between_tokens_resolves() {
    // `split_whitespace` collapses runs of whitespace; pin that.
    let clock = clock_at("2026-04-23T12:00:00");
    let r = expect_resolved("in  3   days", &clock);
    assert!(r.iso.starts_with("2026-04-26"), "got {}", r.iso);
}

#[test]
fn boundary_05_casing_uppercase_in_30_minutes_resolves() {
    // `to_ascii_lowercase` happens before prefix strip — uppercase input
    // round-trips.
    let clock = clock_at("2026-04-23T12:00:00");
    let r = expect_resolved("IN 30 MINUTES", &clock);
    assert!(r.iso.starts_with("2026-04-23T12:30:00"), "got {}", r.iso);
}

#[test]
fn boundary_06_range_with_whitespace_around_dash_currently_handled_or_unresolved() {
    // `9 - 10am` — split-whitespace on left gives ("9", "9"); rsplit on `-`
    // splits at the lone dash. Pin current behavior either way: the helper
    // either resolves it (because the trailing dash split + trim works) OR
    // rejects (depending on how looks_like_time tolerates the trailing
    // space). Acceptable outcomes for current code: Resolved with 09:00 +
    // 10:00, OR Unresolved.
    let clock = clock_at("2026-04-23T08:00:00");
    let result = parse("9 - 10am", &clock);
    match result {
        DateParseResult::Resolved(r) => {
            assert!(
                r.iso.contains("T09:00:00") || r.iso.contains("T9:"),
                "got {}",
                r.iso
            );
        }
        DateParseResult::Unresolved(_) => {} // also acceptable current behavior
        DateParseResult::Empty => panic!("non-empty input"),
    }
}

#[test]
fn boundary_07_range_dash_inside_date_prefix_5_4_dash_5_5() {
    // `5/4-5/5` — left of rightmost dash is `5/4-5`, which `looks_like_time`
    // rejects (it has `/`). Right is `5`, which passes looks_like_time
    // (digits only) but lacks meridiem AND colon → range rejected. Falls
    // through to Unresolved (or chrono_english may resolve `5/4-5/5` as a
    // garbage phrase — pin Unresolved for safety).
    let clock = clock_at("2026-04-23T08:00:00");
    let result = parse("5/4-5/5", &clock);
    match result {
        DateParseResult::Unresolved(_) => {}
        DateParseResult::Resolved(r) => {
            // If a future change makes this resolve as something
            // (e.g. range of two slash-dates) the test would catch it
            // and the author can decide whether to pin or flip.
            panic!(
                "5/4-5/5 currently expected Unresolved (range rejects, slash-pair confuses); got Resolved({})",
                r.iso
            );
        }
        DateParseResult::Empty => panic!("non-empty"),
    }
}

#[test]
fn boundary_08_range_cross_month_boundary_bumps_to_next_month() {
    // `23:00-01:00` at end of month — cross-midnight bump should land
    // on day 1 of next month.
    let clock = clock_at("2026-04-30T20:00:00");
    let r = expect_resolved("23:00-01:00", &clock);
    assert!(r.iso.starts_with("2026-04-30T23:00:00"), "got {}", r.iso);
    let end = r.end_iso.as_deref().expect("end_iso");
    assert!(
        end.starts_with("2026-05-01T01:00:00"),
        "cross-month: end should be 2026-05-01, got {}",
        end
    );
}

// ============================================================================
// COMPOSITION (8 actions)
// ============================================================================

#[test]
fn composition_09_in_1_minute_1_second_two_pair_compound_resolves() {
    let clock = clock_at("2026-04-23T12:00:00");
    let r = expect_resolved("in 1 minute 1 second", &clock);
    // 60 + 1 = 61 seconds offset → 12:01:01.
    assert!(r.iso.starts_with("2026-04-23T12:01:01"), "got {}", r.iso);
    assert_eq!(r.granularity, DateGranularity::Minute);
}

#[test]
fn composition_10_in_3_pair_compound_resolves() {
    let clock = clock_at("2026-04-23T12:00:00");
    let r = expect_resolved("in 5 hours 30 minutes 15 seconds", &clock);
    // 5*3600 + 30*60 + 15 = 19815 sec = 17:30:15.
    assert!(r.iso.starts_with("2026-04-23T17:30:15"), "got {}", r.iso);
}

#[test]
fn composition_11_5_pair_compound_exceeds_4_cap_is_rejected() {
    // 5 pairs > 4-pair cap → resolve_relative_offset returns None →
    // chrono_english fallback also fails → Unresolved.
    let clock = clock_at("2026-04-23T12:00:00");
    match parse("in 1 m 1 m 1 m 1 m 1 m", &clock) {
        DateParseResult::Unresolved(_) => {}
        DateParseResult::Resolved(r) => panic!(
            "5-pair compound should hit the 4-pair cap; got Resolved({})",
            r.iso
        ),
        DateParseResult::Empty => panic!("non-empty"),
    }
}

#[test]
fn composition_12_range_next_mon_9_10am_lands_on_correct_weekday() {
    // 2026-04-23 is Thursday; next Monday is 2026-04-27 (or 2026-05-04
    // depending on chrono_english's "next" semantics).
    let clock = clock_at("2026-04-23T08:00:00");
    let r = expect_resolved("next mon 9-10am", &clock);
    assert!(
        r.iso.starts_with("2026-04-27T09:00:00") || r.iso.starts_with("2026-05-04T09:00:00"),
        "got {}",
        r.iso
    );
    let end = r.end_iso.as_deref().expect("end_iso");
    assert!(end.contains("T10:00:00"), "got {}", end);
}

#[test]
fn composition_13_range_no_prefix_on_weekend_resolves_today() {
    // No prefix → range resolves "today" via the chrono_english fallback
    // path. Saturday 2026-04-25.
    let clock = clock_at("2026-04-25T08:00:00");
    let r = expect_resolved("9-10am", &clock);
    assert!(r.iso.starts_with("2026-04-25T09:00:00"), "got {}", r.iso);
    let end = r.end_iso.as_deref().expect("end_iso");
    assert!(end.starts_with("2026-04-25T10:00:00"), "got {}", end);
}

#[test]
fn composition_14_relative_offset_carries_confidence_0_9() {
    let clock = clock_at("2026-04-23T12:00:00");
    let r = expect_resolved("in 30m", &clock);
    assert!(
        (r.confidence - 0.9).abs() < f32::EPSILON,
        "relative offsets should carry confidence 0.9, got {}",
        r.confidence
    );
}

#[test]
fn composition_15_range_inherits_confidence_from_start_resolved_path() {
    // Range output is built FROM the start_resolved struct, so its
    // confidence reflects how the start half was resolved (chrono_english
    // path = 0.9). Pin: range confidence == start-half confidence.
    let clock = clock_at("2026-04-23T08:00:00");
    let r = expect_resolved("9-10am", &clock);
    assert!(
        (r.confidence - 0.9).abs() < f32::EPSILON,
        "range confidence inherits from start-half resolution; got {}",
        r.confidence
    );
}

#[test]
fn composition_16_range_source_field_is_full_phrase_not_start_half() {
    // Pin that the range output's `source` field is overwritten to the
    // FULL range phrase (not just the start half). This matters for
    // payload consumers that key off `source` to deduplicate or display
    // the original input.
    let clock = clock_at("2026-04-23T08:00:00");
    let r = expect_resolved("9-10am", &clock);
    assert_eq!(r.source, "9-10am", "source must be the full range phrase");
    assert_eq!(
        r.relative, "9-10am",
        "relative must also be the full phrase"
    );
    assert_eq!(r.source_span, (0, 6), "source_span covers the full range");
}

// ============================================================================
// RESURRECTION (6 actions)
// ============================================================================

#[test]
fn resurrection_17_relative_idempotent_under_repeated_calls() {
    let clock = clock_at("2026-04-23T12:00:00");
    let a = expect_resolved("in 30m", &clock);
    let b = expect_resolved("in 30m", &clock);
    let c = expect_resolved("in 30m", &clock);
    assert_eq!(a.iso, b.iso);
    assert_eq!(b.iso, c.iso);
}

#[test]
fn resurrection_18_range_idempotent_under_repeated_calls() {
    let clock = clock_at("2026-04-23T08:00:00");
    let a = expect_resolved("9-10am", &clock);
    let b = expect_resolved("9-10am", &clock);
    assert_eq!(a.iso, b.iso);
    assert_eq!(a.end_iso, b.end_iso);
}

#[test]
fn resurrection_19_clone_clock_yields_equal_offset() {
    let clock1 = clock_at("2026-04-23T12:00:00");
    let clock2 = clock1.clone();
    let r1 = expect_resolved("in 2 hours", &clock1);
    let r2 = expect_resolved("in 2 hours", &clock2);
    assert_eq!(r1.iso, r2.iso);
}

#[test]
fn resurrection_20_different_clocks_produce_different_offsets() {
    // Sanity: changing the clock changes the result. Pins that the
    // helper doesn't accidentally cache a base time.
    let c1 = clock_at("2026-04-23T12:00:00");
    let c2 = clock_at("2026-05-01T12:00:00");
    let r1 = expect_resolved("in 2 hours", &c1);
    let r2 = expect_resolved("in 2 hours", &c2);
    assert_ne!(r1.iso, r2.iso);
    assert!(r1.iso.starts_with("2026-04-23"));
    assert!(r2.iso.starts_with("2026-05-01"));
}

#[test]
fn resurrection_21_range_pure_dispatch_alternating_inputs() {
    // Alternating two range inputs against the same clock — neither
    // call leaks state into the other.
    let clock = clock_at("2026-04-23T08:00:00");
    for _ in 0..3 {
        let a = expect_resolved("9-10am", &clock);
        let b = expect_resolved("13:00-15:30", &clock);
        assert!(a.iso.contains("T09:00:00"));
        assert!(b.iso.contains("T13:00:00"));
    }
}

#[test]
fn resurrection_22_relative_in_seconds_unit_yields_minute_granularity() {
    // Pin: any sub-day unit (including `s`/`seconds`) tags granularity
    // as Minute. Falsifier: a future change that adds a Second
    // granularity tag would flip this test.
    let clock = clock_at("2026-04-23T12:00:00");
    let r = expect_resolved("in 5 seconds", &clock);
    assert_eq!(r.granularity, DateGranularity::Minute);
}
