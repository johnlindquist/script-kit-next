use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use tracing::debug;
use crate::keystroke_logger::keystroke_logger;
/// Default maximum buffer size for typed characters
const DEFAULT_MAX_BUFFER_SIZE: usize = 50;
/// Characters that should clear the buffer when typed
const BUFFER_CLEAR_CHARS: &[char] = &[
    '\n',   // Enter/Return
    '\r',   // Carriage return
    '\x1b', // Escape
    '\t',   // Tab (optional, but common delimiter)
];
/// Result of a successful trigger match
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchResult {
    /// The trigger keyword that was matched (e.g., ":sig")
    pub trigger: String,
    /// Path to the scriptlet to execute
    pub scriptlet_path: PathBuf,
    /// Number of characters to delete (length of trigger)
    pub chars_to_delete: usize,
}
/// Trigger detection system for text expansion
///
/// Maintains a rolling buffer of typed characters and checks for matches
/// against registered triggers. Uses immediate matching - the trigger fires
/// as soon as the keyword is fully typed.
#[derive(Debug)]
pub struct KeywordMatcher {
    /// Map of trigger keywords to their scriptlet paths
    triggers: HashMap<String, PathBuf>,
    /// Fast suffix-matching index keyed by trigger last character
    triggers_by_last_char: HashMap<char, Vec<TriggerPattern>>,
    /// Rolling buffer of recent keystrokes
    buffer: VecDeque<char>,
    /// Maximum size of the buffer
    max_buffer_size: usize,
}
#[derive(Debug, Clone)]
struct TriggerPattern {
    keyword: String,
    scriptlet_path: PathBuf,
    char_count: usize,
    reversed_chars: Vec<char>,
}
impl TriggerPattern {
    fn new(keyword: String, scriptlet_path: PathBuf) -> Self {
        let reversed_chars: Vec<char> = keyword.chars().rev().collect();
        let char_count = reversed_chars.len();
        Self {
            keyword,
            scriptlet_path,
            char_count,
            reversed_chars,
        }
    }

    fn matches_buffer_suffix(&self, buffer: &VecDeque<char>) -> bool {
        if self.char_count > buffer.len() {
            return false;
        }

        self.reversed_chars
            .iter()
            .zip(buffer.iter().rev())
            .all(|(pattern_char, buffered_char)| pattern_char == buffered_char)
    }
}
impl Default for KeywordMatcher {
    fn default() -> Self {
        Self::new()
    }
}
impl KeywordMatcher {
    /// Create a new KeywordMatcher with default settings
    pub fn new() -> Self {
        Self {
            triggers: HashMap::new(),
            triggers_by_last_char: HashMap::new(),
            buffer: VecDeque::with_capacity(DEFAULT_MAX_BUFFER_SIZE),
            max_buffer_size: DEFAULT_MAX_BUFFER_SIZE,
        }
    }

    /// Create a new KeywordMatcher with a custom buffer size
    #[allow(dead_code)]
    pub fn with_buffer_size(max_size: usize) -> Self {
        Self {
            triggers: HashMap::new(),
            triggers_by_last_char: HashMap::new(),
            buffer: VecDeque::with_capacity(max_size),
            max_buffer_size: max_size,
        }
    }

    /// Register a trigger keyword with its associated scriptlet path
    ///
    /// If the trigger already exists, it will be replaced.
    ///
    /// # Arguments
    /// * `keyword` - The trigger text (e.g., ":sig", "!today", "/date")
    /// * `scriptlet_path` - Path to the scriptlet file to execute on match
    pub fn register_trigger(&mut self, keyword: &str, scriptlet_path: PathBuf) {
        if keyword.is_empty() {
            debug!("Attempted to register empty trigger, ignoring");
            return;
        }

        debug!(
            trigger = %keyword,
            path = %scriptlet_path.display(),
            "Registering keyword trigger"
        );

        self.triggers.insert(keyword.to_string(), scriptlet_path);
        self.rebuild_trigger_index();
    }

    /// Unregister a trigger keyword
    ///
    /// # Arguments
    /// * `keyword` - The trigger text to remove
    ///
    /// # Returns
    /// `true` if the trigger was removed, `false` if it didn't exist
    #[allow(dead_code)]
    pub fn unregister_trigger(&mut self, keyword: &str) -> bool {
        let removed = self.triggers.remove(keyword).is_some();

        if removed {
            debug!(trigger = %keyword, "Unregistered keyword trigger");
            self.rebuild_trigger_index();
        }

        removed
    }

    /// Process a keystroke and check for trigger matches
    ///
    /// # Arguments
    /// * `c` - The character that was typed
    ///
    /// # Returns
    /// `Some(MatchResult)` if a trigger was matched, `None` otherwise
    pub fn process_keystroke(&mut self, c: char) -> Option<MatchResult> {
        // Check for buffer-clearing characters
        if BUFFER_CLEAR_CHARS.contains(&c) {
            keystroke_logger().record_buffer_clear();
            self.clear_buffer();
            return None;
        }

        // Add character to buffer
        self.buffer.push_back(c);

        // Trim buffer if it exceeds max size (remove from front)
        while self.buffer.len() > self.max_buffer_size {
            self.buffer.pop_front();
        }

        // Update buffer state for debounced logging
        keystroke_logger().update_buffer_state(self.buffer.len(), self.triggers.len());

        // Check for matches - look for triggers at the end of the buffer
        self.check_for_match(c)
    }

    /// Check if the buffer ends with any registered trigger
    fn check_for_match(&self, last_char: char) -> Option<MatchResult> {
        let candidates = self.triggers_by_last_char.get(&last_char)?;

        for candidate in candidates {
            if candidate.matches_buffer_suffix(&self.buffer) {
                return Some(MatchResult {
                    trigger: candidate.keyword.clone(),
                    scriptlet_path: candidate.scriptlet_path.clone(),
                    chars_to_delete: candidate.char_count,
                });
            }
        }

        None
    }

    /// Clear the keystroke buffer
    pub fn clear_buffer(&mut self) {
        self.buffer.clear();
        debug!("Buffer cleared");
    }

    /// Get the current buffer contents (for debugging)
    #[allow(dead_code)]
    pub fn buffer(&self) -> String {
        self.buffer.iter().collect()
    }

    /// Get the number of registered triggers
    pub fn trigger_count(&self) -> usize {
        self.triggers.len()
    }

    /// Check if a specific trigger is registered
    #[allow(dead_code)]
    pub fn has_trigger(&self, keyword: &str) -> bool {
        self.triggers.contains_key(keyword)
    }

    /// Get all registered triggers
    #[allow(dead_code)]
    pub fn triggers(&self) -> impl Iterator<Item = (&String, &PathBuf)> {
        self.triggers.iter()
    }

    /// Clear all registered triggers
    #[allow(dead_code)]
    pub fn clear_triggers(&mut self) {
        self.triggers.clear();
        self.triggers_by_last_char.clear();
        debug!("All triggers cleared");
    }

    /// Bulk register triggers from an iterator
    #[allow(dead_code)]
    pub fn register_triggers<I>(&mut self, triggers: I)
    where
        I: IntoIterator<Item = (String, PathBuf)>,
    {
        let mut has_updates = false;
        for (keyword, path) in triggers {
            if keyword.is_empty() {
                continue;
            }
            self.triggers.insert(keyword, path);
            has_updates = true;
        }

        if has_updates {
            self.rebuild_trigger_index();
        }
    }

    fn rebuild_trigger_index(&mut self) {
        self.triggers_by_last_char.clear();

        let mut patterns: Vec<TriggerPattern> = self
            .triggers
            .iter()
            .map(|(keyword, path)| TriggerPattern::new(keyword.clone(), path.clone()))
            .collect();

        // Prefer longer suffix matches first when multiple triggers can match
        // the same tail (e.g. ":sig" vs "ig").
        patterns.sort_by(|a, b| {
            b.char_count
                .cmp(&a.char_count)
                .then_with(|| a.keyword.cmp(&b.keyword))
        });

        for pattern in patterns {
            if let Some(&last_char) = pattern.reversed_chars.first() {
                self.triggers_by_last_char
                    .entry(last_char)
                    .or_default()
                    .push(pattern);
            }
        }
    }
}
