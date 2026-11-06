//! The ↵ button while a sentence is typed forward.
//!
//! ↵ names what Enter runs. While each key only extends the words, it changes when that truly
//! changes (a stronger name took the top, or Jev's reading settled) and never goes back to words
//! it showed earlier in the sentence. A name that owned Enter and lost it to another is passed over
//! if it comes back strong later, unless the user finished typing its exact name. Jev's reading becomes ↵'s words only
//! once the keys have paused, or a key that isn't typing (an arrow, Enter) says the sentence is
//! done, so a reading of half a sentence never flashes on the button and goes stale on the next
//! key.

use crate::Ms;
use crate::contract::READING_SETTLE_MS;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Steady {
    /// The words as they were last; a change that doesn't extend them starts a new sentence.
    words: String,
    /// What owned Enter in this sentence, in order: a name by its key, or Jev (`None`).
    owners: Vec<Option<String>>,
    /// Names that lost Enter and came back; they don't own it again in this sentence.
    passed: Vec<String>,
    /// When the words last changed because someone typed.
    typed_at: Option<Ms>,
}

impl Steady {
    /// The words changed.
    pub fn on_text(&mut self, now: Ms, text: &str, typed: bool) {
        if typed {
            self.typed_at = Some(now);
        }
        if text.trim().is_empty() || !text.starts_with(&self.words) {
            self.owners.clear();
            self.passed.clear();
        }
        self.words = text.to_string();
    }

    /// A key that isn't typing: the sentence is as long as it's going to get.
    pub fn key(&mut self) {
        self.typed_at = None;
    }

    /// Whether `key` may own Enter now.
    pub fn may_own(&self, key: &str) -> bool {
        !self.passed.iter().any(|p| p == key)
    }

    /// The names for the current words came, strongest first. Returns the one that owns Enter.
    pub fn names<'a>(
        &mut self,
        strong: impl IntoIterator<Item = (&'a str, bool)>,
    ) -> Option<&'a str> {
        let owner = strong.into_iter().find_map(|(key, explicit)| {
            if explicit {
                self.passed.retain(|p| p != key);
                return Some(key);
            }
            if !self.may_own(key) {
                return None;
            }
            let last = self.owners.last().and_then(Option::as_deref) == Some(key);
            if !last && self.owners.iter().flatten().any(|o| o == key) {
                self.passed.push(key.to_string());
                return None;
            }
            Some(key)
        });
        if self.owners.last().map(Option::as_deref) != Some(owner) {
            self.owners.push(owner.map(str::to_string));
        }
        owner
    }

    /// Whether a reading may name ↵ now: the keys have paused.
    pub fn settled(&self, now: Ms) -> bool {
        self.typed_at
            .is_none_or(|at| now >= at.saturating_add(READING_SETTLE_MS))
    }

    /// When a reading waiting for the keys to pause may name ↵.
    pub fn settles_at(&self, now: Ms) -> Option<Ms> {
        self.typed_at
            .map(|at| at.saturating_add(READING_SETTLE_MS))
            .filter(|t| *t > now)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn typed<'a>(s: &mut Steady, now: Ms, text: &str, strong: &[&'a str]) -> Option<&'a str> {
        s.on_text(now, text, true);
        s.names(
            strong
                .iter()
                .map(|&key| (key, key.eq_ignore_ascii_case(text))),
        )
    }

    #[test]
    fn a_stronger_name_takes_the_top() {
        let mut s = Steady::default();
        assert_eq!(typed(&mut s, 0, "s", &["Safari"]), Some("Safari"));
        assert_eq!(typed(&mut s, 10, "sl", &["Slack", "Safari"]), Some("Slack"));
    }

    #[test]
    fn a_name_that_lost_enter_doesnt_take_it_back() {
        let mut s = Steady::default();
        typed(&mut s, 0, "s", &["Safari"]);
        typed(&mut s, 10, "sl", &["Slack"]);
        assert_eq!(
            typed(&mut s, 20, "sla", &["Safari", "Slack"]),
            Some("Slack")
        );
        assert_eq!(typed(&mut s, 30, "slac", &["Safari"]), None);
        assert!(!s.may_own("Safari"));
    }

    #[test]
    fn a_name_that_lost_enter_to_jev_doesnt_take_it_back() {
        let mut s = Steady::default();
        typed(&mut s, 0, "w", &["WhatsApp"]);
        typed(&mut s, 10, "wh", &[]);
        assert_eq!(typed(&mut s, 20, "whe", &["WhatsApp"]), None);
        assert_eq!(
            typed(&mut s, 30, "when", &["Weather"]),
            Some("Weather"),
            "a name new to the sentence still takes the top"
        );
    }

    #[test]
    fn editing_back_starts_a_new_sentence() {
        let mut s = Steady::default();
        typed(&mut s, 0, "s", &["Safari"]);
        typed(&mut s, 10, "sl", &["Slack"]);
        typed(&mut s, 20, "s", &["Safari"]);
        assert!(s.may_own("Safari"));
    }

    #[test]
    fn a_reading_waits_for_the_keys_to_pause() {
        let mut s = Steady::default();
        assert!(s.settled(0), "nothing typed yet");
        s.on_text(100, "when does", true);
        assert!(!s.settled(100 + READING_SETTLE_MS - 1));
        assert_eq!(s.settles_at(200), Some(100 + READING_SETTLE_MS));
        assert!(s.settled(100 + READING_SETTLE_MS));
        s.on_text(900, "when does lebron", false);
        assert!(s.settled(900), "words the window set aren't typing");
        s.on_text(1_000, "when does lebron j", true);
        s.key();
        assert!(s.settled(1_000), "an arrow or Enter ends the sentence");
    }
}
