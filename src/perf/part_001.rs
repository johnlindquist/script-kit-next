impl Default for KeyEventPerfGuard {
    fn default() -> Self {
        Self::new()
    }
}
impl KeyEventPerfGuard {
    #[inline]
    pub fn new() -> Self {
        let start = start_key_event();
        Self {
            start,
            _timing: TimingGuard::key_event(),
        }
    }
}
impl Drop for KeyEventPerfGuard {
    fn drop(&mut self) {
        end_key_event(self.start);
        // Optional but useful: warns when key repeat is very high.
        log_key_rate();
    }
}
/// RAII guard that records scroll timing into ScrollTimer
/// and logs slow scroll operations via TimingGuard.
pub struct ScrollPerfGuard {
    _timing: TimingGuard,
}
impl Default for ScrollPerfGuard {
    fn default() -> Self {
        Self::new()
    }
}
impl ScrollPerfGuard {
    #[inline]
    pub fn new() -> Self {
        start_scroll();
        Self {
            _timing: TimingGuard::scroll(),
        }
    }
}
impl Drop for ScrollPerfGuard {
    fn drop(&mut self) {
        let _ = end_scroll();
    }
}
// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_key_event_tracker() {
        let mut tracker = KeyEventTracker::new();

        // Simulate some key events
        for _ in 0..5 {
            let start = tracker.start_event();
            thread::sleep(Duration::from_micros(100));
            tracker.end_event(start);
        }

        assert_eq!(tracker.total_events, 5);
        assert!(tracker.avg_processing_time_us() >= 100);
    }

    #[test]
    fn test_scroll_timer() {
        let mut timer = ScrollTimer::new();

        let _start = timer.start();
        thread::sleep(Duration::from_micros(100));
        let duration = timer.end();

        assert!(duration.as_micros() >= 100);
        assert_eq!(timer.total_ops, 1);
    }

    #[test]
    fn test_frame_timer() {
        let mut timer = FrameTimer::new();

        // First frame has no previous
        assert!(timer.mark_frame().is_none());

        thread::sleep(Duration::from_millis(16));
        let duration = timer.mark_frame();

        assert!(duration.is_some());
        assert!(duration.unwrap().as_millis() >= 16);
    }

    #[test]
    fn test_timing_guard() {
        // Just ensure it doesn't panic
        {
            let _guard = TimingGuard::key_event();
            thread::sleep(Duration::from_micros(100));
        }
    }
}
