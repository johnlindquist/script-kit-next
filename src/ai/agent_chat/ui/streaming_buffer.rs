//! Smooth assistant streaming text by buffering RPC chunks and draining on char boundaries.

const DEFAULT_TICK_MS: usize = 16;
const CATCH_UP_WINDOW_MS: usize = 200;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct StreamingTextBuffer {
    pending: String,
}

impl StreamingTextBuffer {
    pub(crate) fn push_chunk(&mut self, chunk: String) {
        if !chunk.is_empty() {
            self.pending.push_str(&chunk);
        }
    }

    pub(crate) fn drain_next(&mut self, chars_budget: usize) -> Option<String> {
        if self.pending.is_empty() || chars_budget == 0 {
            return None;
        }

        let take_chars = chars_budget.min(self.pending.chars().count());
        let byte_end = self
            .pending
            .char_indices()
            .nth(take_chars)
            .map(|(index, _)| index)
            .unwrap_or_else(|| self.pending.len());
        Some(self.pending.drain(..byte_end).collect())
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }

    pub(crate) fn flush_all(&mut self) -> String {
        std::mem::take(&mut self.pending)
    }

    pub(crate) fn drain_budget_for_tick(&self) -> usize {
        Self::budget_for_backlog_chars(self.pending.chars().count(), DEFAULT_TICK_MS)
    }

    pub(crate) fn budget_for_backlog_chars(backlog_chars: usize, tick_ms: usize) -> usize {
        if backlog_chars == 0 || tick_ms == 0 {
            return 0;
        }

        // Drain about one 200ms catch-up window of backlog per second-feel:
        // each 16ms frame reveals ceil(backlog * 16/200). Larger backlogs
        // therefore drain faster while small backlogs still advance one char
        // per tick, keeping catch-up latency bounded without slab jumps.
        let budget = backlog_chars
            .saturating_mul(tick_ms)
            .div_ceil(CATCH_UP_WINDOW_MS);
        budget.clamp(1, backlog_chars)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_and_drain_preserves_order() {
        let mut buffer = StreamingTextBuffer::default();
        buffer.push_chunk("hel".to_string());
        buffer.push_chunk("lo".to_string());

        assert_eq!(buffer.drain_next(2).as_deref(), Some("he"));
        assert_eq!(buffer.drain_next(10).as_deref(), Some("llo"));
        assert!(buffer.is_empty());
        assert_eq!(buffer.drain_next(1), None);
    }

    #[test]
    fn budget_scales_with_backlog() {
        assert_eq!(StreamingTextBuffer::budget_for_backlog_chars(0, 16), 0);
        assert_eq!(StreamingTextBuffer::budget_for_backlog_chars(1, 16), 1);
        assert_eq!(StreamingTextBuffer::budget_for_backlog_chars(25, 16), 2);
        assert_eq!(StreamingTextBuffer::budget_for_backlog_chars(200, 16), 16);
    }

    #[test]
    fn flush_all_drains_everything() {
        let mut buffer = StreamingTextBuffer::default();
        buffer.push_chunk("abc".to_string());
        buffer.push_chunk("def".to_string());

        assert_eq!(buffer.flush_all(), "abcdef");
        assert!(buffer.is_empty());
    }

    #[test]
    fn drain_respects_utf8_boundaries() {
        let mut buffer = StreamingTextBuffer::default();
        buffer.push_chunk("a🦀é中b".to_string());

        assert_eq!(buffer.drain_next(2).as_deref(), Some("a🦀"));
        assert_eq!(buffer.drain_next(2).as_deref(), Some("é中"));
        assert_eq!(buffer.drain_next(2).as_deref(), Some("b"));
        assert!(buffer.is_empty());
    }
}
