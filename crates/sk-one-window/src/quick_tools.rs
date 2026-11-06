//! Small local answers. Recognition has no side effects; a coin is tossed only after dispatch.
use crate::Row;
use unicode_segmentation::UnicodeSegmentation;

pub const COIN: &str = "quick-tool:coin";

pub fn coin_row(words: &str) -> Option<Row> {
    let said = words
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();
    matches!(
        said.as_str(),
        "flip a coin" | "toss a coin" | "coin flip" | "coin toss" | "heads or tails"
    )
    .then(|| {
        Row::new(COIN, "Flip a coin")
            .with_sub("Show heads or tails here")
            .with_enter_label("Flip a coin")
    })
}

/// A deliberate, local number draw. Searching for it consumes no randomness.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NumberDraw {
    low: i32,
    high: i32,
}
impl NumberDraw {
    pub(crate) fn die(sides: u8) -> Self {
        debug_assert!(sides >= 2);
        Self {
            low: 1,
            high: i32::from(sides),
        }
    }
    pub fn request(words: &str) -> Option<Self> {
        let lower = words.to_ascii_lowercase();
        let tokens: Vec<_> = lower.split_whitespace().collect();
        let rest = tokens
            .strip_prefix(&["pick", "a", "random", "number"][..])
            .or_else(|| tokens.strip_prefix(&["random", "number"][..]))
            .or_else(|| tokens.strip_prefix(&["pick", "a", "number"][..]))?;
        let (low, high) = match rest {
            [] => (1, 100),
            [a, "to", b] | ["from", a, "to", b] | ["between", a, "and", b] => {
                (a.parse().ok()?, b.parse().ok()?)
            }
            _ => return None,
        };
        (low <= high).then_some(Self { low, high })
    }
    pub fn bounds(self) -> (i32, i32) {
        (self.low, self.high)
    }
    pub fn key(self) -> String {
        format!("quick-tool:random:{}:{}", self.low, self.high)
    }
    pub fn row(self) -> Row {
        Row::new(
            self.key(),
            format!("Pick a random number · {} to {}", self.low, self.high),
        )
        .with_sub("Draw once here · choose a range with “random number 1 to 6”")
        .with_enter_label("Draw a number")
    }
    pub fn draw(self, mut next: impl FnMut() -> u32) -> i32 {
        let width = (i64::from(self.high) - i64::from(self.low) + 1) as u64;
        let population = 1u64 << 32;
        let limit = population - population % width;
        loop {
            let n = u64::from(next());
            if n < limit {
                return (i64::from(self.low) + (n % width) as i64) as i32;
            }
        }
    }
}

pub fn word_count_label(text: &str) -> String {
    let n = text.unicode_words().count();
    format!("{n} {}", if n == 1 { "word" } else { "words" })
}

/// Capitalize each word, preserving punctuation, apostrophes, spaces and line breaks. This is
/// Unicode word casing, not a particular publication’s editorial style for minor words.
pub fn title_case(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    for word in input.split_word_bounds() {
        let mut at_start = true;
        for ch in word.chars() {
            if ch.is_alphabetic() {
                if at_start {
                    result.extend(ch.to_uppercase());
                } else {
                    result.extend(ch.to_lowercase());
                }
                at_start = false;
            } else {
                result.push(ch);
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn a_word_count_keeps_apostrophes_and_ignores_separators() {
        assert_eq!(word_count_label("  don’t stop, café!  "), "3 words");
        assert_eq!(word_count_label("🦉"), "0 words");
    }
    #[test]
    fn number_draw_is_bounded_inclusive_and_rejects_the_biased_tail() {
        let d = NumberDraw::request("pick a random number between -2 and 3").unwrap();
        let mut values = [u32::MAX, 5].into_iter();
        assert_eq!(d.draw(|| values.next().unwrap()), 3);
        assert_eq!(d.draw(|| 0), -2);
        assert_eq!(NumberDraw::request("random number").unwrap().high, 100);
        for invalid in [
            "random number 9 to 2",
            "search random number",
            "random number 3 to 4 extra",
        ] {
            assert!(NumberDraw::request(invalid).is_none());
        }
    }

    #[test]
    fn casing_preserves_content_other_than_case() {
        assert_eq!(title_case("how are you?"), "How Are You?");
        assert_eq!(
            title_case("  don't-stop, café!\néCOLE 🦉"),
            "  Don't-Stop, Café!\nÉcole 🦉"
        );
    }
    #[test]
    fn a_coin_request_offers_a_deliberate_action_not_a_search_hit_inside_a_sentence() {
        assert_eq!(coin_row("  Flip   a coin ").unwrap().key, COIN);
        assert!(coin_row("search for coin flip images").is_none());
    }
}
