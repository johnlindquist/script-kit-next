use std::collections::VecDeque;
use std::sync::{Mutex, OnceLock};

const MAX_TERMINAL_HISTORY_ENTRIES: usize = 25;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalHistoryEntry {
    pub label: String,
    pub source: String,
    pub text: String,
    pub line_count: usize,
    pub truncated: bool,
    pub captured_at: String,
}

static TERMINAL_HISTORY: OnceLock<Mutex<VecDeque<TerminalHistoryEntry>>> = OnceLock::new();

fn history() -> &'static Mutex<VecDeque<TerminalHistoryEntry>> {
    TERMINAL_HISTORY.get_or_init(|| Mutex::new(VecDeque::new()))
}

pub fn record(entry: TerminalHistoryEntry) {
    if entry.text.trim().is_empty() {
        return;
    }
    let mut guard = history()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    guard.retain(|existing| existing.source != entry.source);
    guard.push_front(entry);
    while guard.len() > MAX_TERMINAL_HISTORY_ENTRIES {
        guard.pop_back();
    }
}

pub fn recent(limit: usize) -> Vec<TerminalHistoryEntry> {
    history()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .iter()
        .take(limit)
        .cloned()
        .collect()
}

#[cfg(test)]
pub fn clear_for_tests() {
    history()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clear();
}
