//! No-panic fuzz harness for the Power Syntax date parser
//! ([[src/menu_syntax/date.rs#parse_date_phrase_result]]).
//!
//! Story: date-smart-quotes-fuzz. Feeds 1000 deterministic random short
//! strings (drawn from a small alphabet that includes ASCII digits,
//! letters, punctuation the parser cares about, and Unicode smart
//! quotes) through `parse_date_phrase_result` and asserts no panic. Uses
//! a hand-rolled xorshift64 PRNG with a fixed seed so the corpus is
//! reproducible across CI runs without pulling in the `proptest` crate.
//!
//! Receipt: `cargo test --test menu_syntax_date_proptest`.

use chrono_tz::America::Denver;
use script_kit_gpui::menu_syntax::{parse_date_phrase_result, DateRole, MenuSyntaxClock};

const ALPHABET: &[char] = &[
    // ASCII digits
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
    // Letters (a sample — full alphabet is unnecessary noise)
    't', 'o', 'd', 'a', 'y', 'p', 'm', 'n', 'x', 'i', ' ', '-', '+', ':', '/', '.',
    // Punctuation and whitespace the parser splits on
    ',', '\t', '"', '\'',
    // Smart-quote glyphs — fuzz hits the normalize_smart_quotes path
    '\u{201C}', '\u{201D}', '\u{2018}', '\u{2019}', '\u{00AB}', '\u{00BB}',
    // Less common Unicode to widen the surface
    'é', '京',
];

/// xorshift64 — deterministic, fast, zero deps. Same algorithm Marsaglia
/// published; sufficient for fuzz-coverage purposes (NOT for crypto).
struct XorShift64(u64);
impl XorShift64 {
    fn new(seed: u64) -> Self {
        Self(if seed == 0 {
            0xdead_beef_cafe_babe
        } else {
            seed
        })
    }
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
    fn pick<'a, T>(&mut self, slice: &'a [T]) -> &'a T {
        let n = slice.len() as u64;
        &slice[(self.next() % n) as usize]
    }
    fn range(&mut self, lo: usize, hi: usize) -> usize {
        let span = (hi - lo) as u64;
        lo + (self.next() % span) as usize
    }
}

#[test]
fn parser_does_not_panic_on_1000_random_short_strings() {
    let clock = MenuSyntaxClock::fixed("2026-04-23T08:00:00", Denver).expect("fixed clock");
    let mut rng = XorShift64::new(0xa1b2_c3d4_e5f6_0789);
    for i in 0..1000 {
        let len = rng.range(0, 33); // 0..=32 chars
        let mut s = String::with_capacity(len * 4);
        for _ in 0..len {
            s.push(*rng.pick(ALPHABET));
        }
        // Each call must return without panicking. The result is
        // discarded — what we're proving is no-panic, not correctness.
        let _ = parse_date_phrase_result(&s, (0, s.len()), DateRole::Inferred, &clock);
        // Every 100th input, also exercise the empty-prefix span path so
        // the (0,0) span doesn't get all the iteration coverage.
        if i % 100 == 0 {
            let _ = parse_date_phrase_result(&s, (0, 0), DateRole::Due, &clock);
        }
    }
}

/// Falsifier: exercise the obvious smart-quote inputs explicitly so a
/// regression in `normalize_smart_quotes` is caught even if the random
/// fuzz loop happens to skip the relevant chars.
#[test]
fn smart_quote_corpus_does_not_panic() {
    let clock = MenuSyntaxClock::fixed("2026-04-23T08:00:00", Denver).expect("fixed clock");
    let corpus = [
        "\u{201C}today\u{201D}",
        "\u{2018}tomorrow\u{2019}",
        "\u{00AB}noon\u{00BB}",
        "due:\u{201C}tomorrow\u{201D}",
        "\u{201C}\u{201D}",
        "\u{201C}",
        "\u{201D}",
        "5pm \u{201C}PST\u{201D}",
    ];
    for s in corpus {
        let _ = parse_date_phrase_result(s, (0, s.len()), DateRole::Inferred, &clock);
    }
}
