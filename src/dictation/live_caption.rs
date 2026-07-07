//! Word-by-word live caption reveal for the dictation overlay.
//!
//! Whisper partials arrive in bursts every ~1.5 s and may rewrite their own
//! tail, so rendering them directly makes the transcript pop in and out.
//! [`LiveCaption`] sits between the raw partial stream and the renderer: it
//! merges each partial into a committed word list and then reveals words one
//! at a time on a paced clock, so the user reads a steady caption stream
//! instead of watching lines appear and vanish.
//!
//! Pacing contract:
//! - one word per [`REVEAL_INTERVAL`] while the backlog is small,
//! - one word per [`REVEAL_INTERVAL_CATCHUP`] once the backlog exceeds
//!   [`CATCHUP_BACKLOG_WORDS`] (keeps up with fast speech),
//! - anything beyond [`SNAP_BACKLOG_WORDS`] is revealed immediately so the
//!   caption can never fall unboundedly behind the speaker.
//!
//! When a newer partial rewrites words that are already on screen, the
//! reveal pointer rewinds to the first divergent word and the replacement
//! words stream back in — a targeted correction instead of a full repaint.

use std::time::{Duration, Instant};

/// Relaxed reveal pace: one word roughly every 110 ms reads as live captions.
pub(crate) const REVEAL_INTERVAL: Duration = Duration::from_millis(110);
/// Catch-up pace once the backlog exceeds [`CATCHUP_BACKLOG_WORDS`].
pub(crate) const REVEAL_INTERVAL_CATCHUP: Duration = Duration::from_millis(55);
/// Backlog size at which the reveal clock switches to the catch-up pace.
pub(crate) const CATCHUP_BACKLOG_WORDS: usize = 5;
/// Backlog size beyond which pending words are revealed instantly.
pub(crate) const SNAP_BACKLOG_WORDS: usize = 14;

/// Marker prefix the runtime puts on partials whose leading audio scrolled
/// out of the transcription window.
const TRUNCATED_PREFIX: char = '\u{2026}';

/// Paced word-reveal state between raw dictation partials and the renderer.
#[derive(Debug, Default)]
pub(crate) struct LiveCaption {
    /// Committed words merged across all partials seen this session.
    words: Vec<String>,
    /// Number of committed words currently visible.
    revealed: usize,
    /// Bumped whenever the newest visible word changes so the renderer can
    /// restart the fade-in animation for exactly that word.
    generation: u64,
    /// When the most recent word was revealed (paces the clock).
    last_reveal_at: Option<Instant>,
}

impl LiveCaption {
    /// Merge a raw partial transcript into the committed word list.
    ///
    /// Rewinds the reveal pointer to the first divergent word when the new
    /// partial rewrites text that is already visible.
    pub(crate) fn set_target(&mut self, raw: &str) {
        let trimmed = raw.trim();
        let truncated = trimmed.starts_with(TRUNCATED_PREFIX);
        let body = trimmed.trim_start_matches(TRUNCATED_PREFIX);
        let incoming: Vec<String> = body.split_whitespace().map(str::to_string).collect();

        if incoming.is_empty() {
            return;
        }

        let merged = if truncated {
            merge_truncated_window(&self.words, &incoming)
        } else {
            incoming
        };

        let divergence = common_word_prefix_len(&self.words, &merged);
        self.revealed = self.revealed.min(divergence);
        self.words = merged;
    }

    /// Advance the reveal clock. Returns `true` when the visible text changed.
    pub(crate) fn tick(&mut self, now: Instant) -> bool {
        let mut changed = false;

        // Never fall unboundedly behind: snap the oldest backlog straight in.
        let backlog = self.words.len() - self.revealed;
        if backlog > SNAP_BACKLOG_WORDS {
            self.revealed = self.words.len() - SNAP_BACKLOG_WORDS;
            self.generation = self.generation.wrapping_add(1);
            self.last_reveal_at = Some(now);
            changed = true;
        }

        let backlog = self.words.len() - self.revealed;
        if backlog == 0 {
            return changed;
        }

        let interval = if backlog > CATCHUP_BACKLOG_WORDS {
            REVEAL_INTERVAL_CATCHUP
        } else {
            REVEAL_INTERVAL
        };
        let due = self
            .last_reveal_at
            .is_none_or(|at| now.duration_since(at) >= interval);
        if due {
            self.revealed += 1;
            self.generation = self.generation.wrapping_add(1);
            self.last_reveal_at = Some(now);
            changed = true;
        }

        changed
    }

    /// Reveal everything immediately (processing phases show the full text).
    pub(crate) fn reveal_all(&mut self) {
        if self.revealed != self.words.len() {
            self.revealed = self.words.len();
            self.generation = self.generation.wrapping_add(1);
        }
    }

    /// Visible caption text (the revealed prefix of the committed words).
    pub(crate) fn visible_text(&self) -> String {
        self.words[..self.revealed].join(" ")
    }

    /// Char offset into [`Self::visible_text`] where the newest word begins.
    ///
    /// Everything before this offset is stable and must not re-animate.
    pub(crate) fn fresh_char_offset(&self) -> usize {
        if self.revealed == 0 {
            return 0;
        }
        let stable_chars: usize = self.words[..self.revealed - 1]
            .iter()
            .map(|word| word.chars().count())
            .sum();
        // Joining spaces between the stable words plus the one before the
        // fresh word.
        stable_chars + self.revealed - 1
    }

    /// Animation restart key for the newest revealed word.
    pub(crate) fn generation(&self) -> u64 {
        self.generation
    }

    /// Full committed text including words not yet revealed.
    ///
    /// The overlay sizes its transcript block from this so the window grows
    /// before the paced reveal reaches the new words, never during.
    pub(crate) fn target_text(&self) -> String {
        self.words.join(" ")
    }

    /// When the newest visible word appeared (drives the render-time fade).
    pub(crate) fn last_reveal_at(&self) -> Option<Instant> {
        self.last_reveal_at
    }

    /// True when nothing is visible yet (renderer falls back to the waveform).
    pub(crate) fn is_empty(&self) -> bool {
        self.revealed == 0
    }

    /// Number of committed words not yet revealed (test/diagnostic hook).
    #[cfg(test)]
    pub(crate) fn backlog(&self) -> usize {
        self.words.len() - self.revealed
    }
}

/// Count the words shared at the start of two word lists.
fn common_word_prefix_len(previous: &[String], current: &[String]) -> usize {
    previous
        .iter()
        .zip(current.iter())
        .take_while(|(a, b)| a == b)
        .count()
}

/// Merge a truncated-window partial into the committed words.
///
/// The partial no longer starts at the beginning of the session, so align it
/// by the longest suffix of `existing` that matches a prefix of `incoming`.
/// The final committed word is allowed to differ (whisper rewrites its own
/// tail), so alignment is retried with the last word dropped before giving
/// up. With no alignment the incoming words replace the committed list — the
/// pre-window prefix already scrolled out of the visible preview anyway.
fn merge_truncated_window(existing: &[String], incoming: &[String]) -> Vec<String> {
    for trim in 0..=1usize.min(existing.len()) {
        let anchored = &existing[..existing.len() - trim];
        let max_overlap = anchored.len().min(incoming.len());
        for overlap in (1..=max_overlap).rev() {
            if anchored[anchored.len() - overlap..] == incoming[..overlap] {
                let mut merged = anchored[..anchored.len() - overlap].to_vec();
                merged.extend_from_slice(incoming);
                return merged;
            }
        }
    }
    incoming.to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn revealed_text(caption: &LiveCaption) -> String {
        caption.visible_text()
    }

    fn drain(caption: &mut LiveCaption, mut now: Instant) -> Instant {
        // Tick far apart until the backlog is empty.
        for _ in 0..1000 {
            if caption.backlog() == 0 {
                break;
            }
            now += REVEAL_INTERVAL;
            caption.tick(now);
        }
        now
    }

    #[test]
    fn reveals_one_word_per_interval() {
        let mut caption = LiveCaption::default();
        let start = Instant::now();
        caption.set_target("hello brave new world");

        assert!(caption.tick(start), "first word reveals immediately");
        assert_eq!(revealed_text(&caption), "hello");

        // Too soon: nothing new.
        assert!(!caption.tick(start + Duration::from_millis(30)));
        assert_eq!(revealed_text(&caption), "hello");

        assert!(caption.tick(start + REVEAL_INTERVAL));
        assert_eq!(revealed_text(&caption), "hello brave");
    }

    #[test]
    fn generation_bumps_on_each_reveal() {
        let mut caption = LiveCaption::default();
        let start = Instant::now();
        caption.set_target("one two");
        let g0 = caption.generation();
        caption.tick(start);
        let g1 = caption.generation();
        caption.tick(start + REVEAL_INTERVAL);
        let g2 = caption.generation();
        assert_ne!(g0, g1);
        assert_ne!(g1, g2);
    }

    #[test]
    fn catches_up_faster_with_large_backlog() {
        let mut caption = LiveCaption::default();
        let start = Instant::now();
        caption.set_target("w1 w2 w3 w4 w5 w6 w7 w8");
        caption.tick(start);
        assert_eq!(caption.backlog(), 7);

        // Backlog > CATCHUP_BACKLOG_WORDS: the catch-up interval applies.
        assert!(caption.tick(start + REVEAL_INTERVAL_CATCHUP));
        assert_eq!(revealed_text(&caption), "w1 w2");
    }

    #[test]
    fn snaps_when_backlog_exceeds_ceiling() {
        let mut caption = LiveCaption::default();
        let start = Instant::now();
        let words: Vec<String> = (0..30).map(|i| format!("w{i}")).collect();
        caption.set_target(&words.join(" "));
        caption.tick(start);
        assert!(
            caption.backlog() <= SNAP_BACKLOG_WORDS,
            "backlog {} must collapse to the snap ceiling",
            caption.backlog()
        );
        assert!(!caption.is_empty());
    }

    #[test]
    fn tail_rewrite_rewinds_only_divergent_words() {
        let mut caption = LiveCaption::default();
        let mut now = Instant::now();
        caption.set_target("the quick brown fax");
        now = drain(&mut caption, now);
        assert_eq!(revealed_text(&caption), "the quick brown fax");

        // A newer partial fixes the last word: only it re-streams.
        caption.set_target("the quick brown fox jumps");
        assert_eq!(revealed_text(&caption), "the quick brown");
        now += REVEAL_INTERVAL;
        caption.tick(now);
        assert_eq!(revealed_text(&caption), "the quick brown fox");
        now += REVEAL_INTERVAL;
        caption.tick(now);
        assert_eq!(revealed_text(&caption), "the quick brown fox jumps");
    }

    #[test]
    fn truncated_window_appends_via_overlap() {
        let mut caption = LiveCaption::default();
        let mut now = Instant::now();
        caption.set_target("alpha beta gamma delta");
        now = drain(&mut caption, now);

        // The window slid: the partial lost "alpha" but overlaps the tail.
        caption.set_target("\u{2026}beta gamma delta epsilon");
        assert_eq!(
            revealed_text(&caption),
            "alpha beta gamma delta",
            "already-revealed words stay visible through a window slide"
        );
        now += REVEAL_INTERVAL;
        caption.tick(now);
        assert_eq!(revealed_text(&caption), "alpha beta gamma delta epsilon");
    }

    #[test]
    fn truncated_window_aligns_even_when_last_word_was_rewritten() {
        let existing: Vec<String> = ["a", "b", "c", "d"].map(str::to_string).to_vec();
        let incoming: Vec<String> = ["c", "delta", "e"].map(str::to_string).to_vec();
        let merged = merge_truncated_window(&existing, &incoming);
        assert_eq!(merged, ["a", "b", "c", "delta", "e"]);
    }

    #[test]
    fn truncated_window_without_alignment_replaces() {
        let existing: Vec<String> = ["x", "y"].map(str::to_string).to_vec();
        let incoming: Vec<String> = ["p", "q", "r"].map(str::to_string).to_vec();
        let merged = merge_truncated_window(&existing, &incoming);
        assert_eq!(merged, ["p", "q", "r"]);
    }

    #[test]
    fn empty_partial_keeps_existing_words() {
        let mut caption = LiveCaption::default();
        let now = Instant::now();
        caption.set_target("keep these words");
        caption.tick(now);
        caption.set_target("   ");
        assert_eq!(revealed_text(&caption), "keep");
        assert_eq!(caption.backlog(), 2);
    }

    #[test]
    fn fresh_char_offset_points_at_newest_word() {
        let mut caption = LiveCaption::default();
        let mut now = Instant::now();
        caption.set_target("hello world");
        now = drain(&mut caption, now);
        let _ = now;
        let text = caption.visible_text();
        let offset = caption.fresh_char_offset();
        assert_eq!(&text[offset..], "world");
        assert_eq!(text.chars().take(offset).collect::<String>(), "hello ");
    }

    #[test]
    fn reveal_all_shows_everything() {
        let mut caption = LiveCaption::default();
        caption.set_target("full text shown at once");
        caption.reveal_all();
        assert_eq!(revealed_text(&caption), "full text shown at once");
        assert_eq!(caption.backlog(), 0);
    }

    #[test]
    fn target_text_includes_unrevealed_words() {
        let mut caption = LiveCaption::default();
        caption.set_target("words not yet revealed");
        // Nothing revealed yet, but the sizing target sees the whole text so
        // the overlay can grow before the paced reveal reaches it.
        assert!(caption.is_empty());
        assert_eq!(caption.target_text(), "words not yet revealed");
    }
}
