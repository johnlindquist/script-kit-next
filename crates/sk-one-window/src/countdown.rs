//! One app-local timer. Reading a sentence never starts it. Every mutation checks its identity.
use crate::{Ms, Row};

pub const MAX_SECONDS: u64 = 24 * 60 * 60;
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Request {
    Start(u64),
    List,
    Stop,
}
pub fn request(text: &str) -> Option<Request> {
    let lower = text.trim().to_ascii_lowercase();
    if !lower.contains("timer") && lower != "time remaining" {
        return None;
    }
    let bytes = lower.as_bytes();
    if bytes.iter().enumerate().any(|(i, c)| {
        *c == b'-'
            && (i == 0
                || !bytes[i - 1].is_ascii_digit()
                || !bytes.get(i + 1).is_some_and(u8::is_ascii_alphabetic))
    }) {
        return None;
    }
    let folded = lower.replace('-', " ");
    let said = folded.split_whitespace().collect::<Vec<_>>().join(" ");
    match said.as_str() {
        "timer" | "timers" | "time remaining" | "timer status" => return Some(Request::List),
        "stop timer" | "cancel timer" | "stop the timer" | "cancel the timer" => {
            return Some(Request::Stop);
        }
        _ => {}
    }
    let span = said
        .strip_suffix(" timer")
        .or_else(|| said.strip_prefix("set a timer for "))
        .or_else(|| said.strip_prefix("timer for "))
        .or_else(|| said.strip_prefix("timer "))?;
    let span = span
        .strip_prefix("start a ")
        .or_else(|| span.strip_prefix("start "))
        .unwrap_or(span);
    let unit = span.find(|c: char| !c.is_ascii_digit())?;
    let number = span[..unit].parse::<u64>().ok()?;
    let factor = match span[unit..].trim() {
        "s" | "sec" | "second" | "seconds" => 1,
        "m" | "min" | "minute" | "minutes" => 60,
        "h" | "hr" | "hour" | "hours" => 3600,
        _ => return None,
    };
    let seconds = number.checked_mul(factor)?;
    (1..=MAX_SECONDS)
        .contains(&seconds)
        .then_some(Request::Start(seconds))
}
pub fn duration_words(seconds: u64) -> String {
    let (n, name) = if seconds.is_multiple_of(3600) {
        (seconds / 3600, "hour")
    } else if seconds.is_multiple_of(60) {
        (seconds / 60, "minute")
    } else {
        (seconds, "second")
    };
    format!("{n} {name}{}", if n == 1 { "" } else { "s" })
}
pub fn remaining_words(ms: Ms) -> String {
    let s = ms.div_ceil(1000);
    if s >= 3600 {
        format!("{}:{:02}:{:02}", s / 3600, (s % 3600) / 60, s % 60)
    } else {
        format!("{}:{:02}", s / 60, s % 60)
    }
}
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Timer {
    pub id: u64,
    pub seconds: u64,
    pub due: Ms,
}
#[derive(Debug, Default)]
pub struct Timers {
    generation: u64,
    current: Option<Timer>,
}
impl Timers {
    pub fn active(&self) -> Option<Timer> {
        self.current
    }
    pub fn start(&mut self, seconds: u64, now: Ms) -> Option<Timer> {
        if !(1..=MAX_SECONDS).contains(&seconds) {
            return None;
        }
        self.arm(seconds, seconds * 1000, now)
    }
    pub fn arm(&mut self, original_seconds: u64, remaining: Ms, now: Ms) -> Option<Timer> {
        if self.current.is_some()
            || !(1..=MAX_SECONDS).contains(&original_seconds)
            || remaining == 0
            || remaining > original_seconds * 1000
        {
            return None;
        }
        let id = self.generation.checked_add(1)?;
        let due = now.checked_add(remaining)?;
        let v = Timer {
            id,
            seconds: original_seconds,
            due,
        };
        self.generation = id;
        self.current = Some(v);
        Some(v)
    }
    pub fn stop(&mut self, id: u64) -> Option<Timer> {
        if self.current?.id != id {
            return None;
        }
        self.current.take()
    }
    pub fn finish(&mut self, id: u64, now: Ms) -> Option<Timer> {
        if self.current?.due > now {
            return None;
        }
        self.stop(id)
    }
    pub fn rows(&self, words: &str, now: Ms) -> Vec<Row> {
        let Some(request) = request(words) else {
            return Vec::new();
        };
        if let Some(v) = self.current {
            let left = remaining_words(v.due.saturating_sub(now));
            let status = Row::new(
                format!("quick-timer:status:{}", v.id),
                format!("Timer · {left} remaining"),
            )
            .with_sub("Runs while Script Kit is open")
            .with_icon("timer")
            .with_enter_label("Show time left");
            let stop = Row::new(format!("quick-timer:stop:{}", v.id), "Stop timer")
                .with_sub(format!("Stop the timer with {left} remaining"))
                .with_icon("timer")
                .with_enter_label("Stop timer");
            return if request == Request::Stop {
                vec![stop, status]
            } else {
                vec![status, stop]
            };
        }
        let start = |seconds| {
            Row::new(
                format!("quick-timer:start:{seconds}"),
                format!("Start timer · {}", duration_words(seconds)),
            )
            .with_sub("Count down locally while Script Kit is open")
            .with_icon("timer")
            .with_enter_label("Start timer")
        };
        match request {
            Request::Start(s) => vec![start(s)],
            Request::List => vec![start(300), start(60), start(1500)],
            Request::Stop => vec![
                Row::new("quick-timer:idle", "No timer is running")
                    .with_enter_label("Check status"),
            ],
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn a_duration_offers_a_countdown_without_starting_it() {
        let t = Timers::default();
        for s in [
            "5 minute timer",
            "timer for 5 minutes",
            "set a timer for 5m",
            "start a 5-minute timer",
        ] {
            assert_eq!(request(s), Some(Request::Start(300)));
            assert_eq!(t.rows(s, 100)[0].key, "quick-timer:start:300");
        }
        assert!(t.active().is_none());
        for s in [
            "0 minute timer",
            "-5 minute timer",
            "timer -5m",
            "25 hour timer",
            "5 things timer",
            "notes about a timer",
            "99999999999999999999999h timer",
        ] {
            assert!(request(s).is_none());
        }
    }
    #[test]
    fn old_callbacks_cannot_finish_a_resumed_or_replaced_timer() {
        let mut t = Timers::default();
        let a = t.start(3, 20).unwrap();
        assert!(t.start(1, 30).is_none());
        let saved = t.stop(a.id).unwrap();
        let b = t.arm(saved.seconds, 2200, 400).unwrap();
        assert_ne!(a.id, b.id);
        assert!(t.finish(a.id, 9000).is_none());
        assert!(t.finish(b.id, 2599).is_none());
        assert_eq!(t.finish(b.id, 2600), Some(b));
        assert!(t.finish(b.id, 2601).is_none());
    }
}
