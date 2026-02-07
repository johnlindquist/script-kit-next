use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::{debug, info, instrument, warn};
/// Maximum number of entries to store in history
const MAX_ENTRIES: usize = 100;
/// Input history with navigation state
///
/// NOTE: Clone is intentionally NOT derived to prevent accidental data loss
/// in multi-window contexts. If you need to share InputHistory across
/// multiple owners, use `Arc<Mutex<InputHistory>>` explicitly.
#[derive(Debug, Serialize, Deserialize)]
pub struct InputHistory {
    /// Stored entries (most recent first)
    entries: Vec<String>,
    /// Current navigation index (None = not navigating, Some(i) = at entries[i])
    /// This is ephemeral and not persisted
    #[serde(skip)]
    current_index: Option<usize>,
    /// Path to the history file (not persisted)
    #[serde(skip)]
    file_path: PathBuf,
}
impl Default for InputHistory {
    fn default() -> Self {
        Self::new()
    }
}
impl InputHistory {
    /// Create a new InputHistory with the default path (~/.scriptkit/input_history.json)
    pub fn new() -> Self {
        let file_path = Self::default_path();
        InputHistory {
            entries: Vec::new(),
            current_index: None,
            file_path,
        }
    }

    /// Create an InputHistory with a custom path (for testing)
    #[allow(dead_code)]
    pub fn with_path(path: PathBuf) -> Self {
        InputHistory {
            entries: Vec::new(),
            current_index: None,
            file_path: path,
        }
    }

    /// Get the default history file path
    fn default_path() -> PathBuf {
        PathBuf::from(shellexpand::tilde("~/.scriptkit/input_history.json").as_ref())
    }

    /// Load history from disk
    ///
    /// Creates an empty history if the file doesn't exist.
    #[instrument(name = "input_history_load", skip(self))]
    pub fn load(&mut self) -> Result<()> {
        if !self.file_path.exists() {
            info!(path = %self.file_path.display(), "Input history file not found, starting fresh");
            return Ok(());
        }

        let content = std::fs::read_to_string(&self.file_path).with_context(|| {
            format!(
                "Failed to read input history file: {}",
                self.file_path.display()
            )
        })?;

        // Deserialize just the entries array
        let data: InputHistoryData =
            serde_json::from_str(&content).with_context(|| "Failed to parse input history JSON")?;

        self.entries = data.entries;
        self.current_index = None; // Always reset navigation on load

        // Enforce max entries in case file was manually edited
        if self.entries.len() > MAX_ENTRIES {
            self.entries.truncate(MAX_ENTRIES);
        }

        info!(
            path = %self.file_path.display(),
            entry_count = self.entries.len(),
            "Loaded input history"
        );

        Ok(())
    }

    /// Save history to disk using atomic write (write temp + rename)
    #[instrument(name = "input_history_save", skip(self))]
    pub fn save(&self) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = self.file_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }

        // Serialize just the entries
        let data = InputHistoryData {
            entries: self.entries.clone(),
        };
        let json =
            serde_json::to_string_pretty(&data).context("Failed to serialize input history")?;

        // Atomic write: write to temp file, then rename
        let temp_path = self.file_path.with_extension("json.tmp");

        std::fs::write(&temp_path, &json).with_context(|| {
            format!(
                "Failed to write temp input history file: {}",
                temp_path.display()
            )
        })?;

        // Atomic rename
        std::fs::rename(&temp_path, &self.file_path).with_context(|| {
            format!("Failed to rename temp file to {}", self.file_path.display())
        })?;

        info!(
            path = %self.file_path.display(),
            entry_count = self.entries.len(),
            bytes = json.len(),
            "Saved input history (atomic)"
        );

        Ok(())
    }

    /// Add an entry to history
    ///
    /// - Prepends the entry to the front
    /// - Removes any existing duplicate
    /// - Caps at MAX_ENTRIES (100)
    /// - Resets navigation state
    #[instrument(name = "input_history_add", skip(self))]
    pub fn add_entry(&mut self, text: &str) {
        // Skip empty entries
        let text = text.trim();
        if text.is_empty() {
            debug!("Skipping empty input");
            return;
        }

        // Remove duplicate if exists
        self.entries.retain(|e| e != text);

        // Prepend new entry
        self.entries.insert(0, text.to_string());

        // Cap at max
        if self.entries.len() > MAX_ENTRIES {
            self.entries.truncate(MAX_ENTRIES);
        }

        // Reset navigation
        self.current_index = None;

        debug!(
            entry = text,
            total_entries = self.entries.len(),
            "Added entry to input history"
        );
    }

    /// Navigate up (to older entries)
    ///
    /// Returns the entry at the new position, or None if at the end.
    pub fn navigate_up(&mut self) -> Option<String> {
        if self.entries.is_empty() {
            return None;
        }

        let new_index = match self.current_index {
            None => 0, // Start at most recent
            Some(i) => {
                if i + 1 < self.entries.len() {
                    i + 1
                } else {
                    return None; // Already at oldest
                }
            }
        };

        self.current_index = Some(new_index);
        let entry = self.entries.get(new_index).cloned();

        debug!(
            index = new_index,
            entry = entry.as_deref().unwrap_or("<none>"),
            "Navigated up in history"
        );

        entry
    }

    /// Navigate down (to newer entries)
    ///
    /// Returns the entry at the new position, or None if past the newest entry
    /// (indicating the user should see their current typed input).
    pub fn navigate_down(&mut self) -> Option<String> {
        match self.current_index {
            None => None, // Not navigating
            Some(0) => {
                // At most recent entry, reset navigation
                self.current_index = None;
                debug!("Navigated past newest entry, returning to input");
                None
            }
            Some(i) => {
                let new_index = i - 1;
                self.current_index = Some(new_index);
                let entry = self.entries.get(new_index).cloned();

                debug!(
                    index = new_index,
                    entry = entry.as_deref().unwrap_or("<none>"),
                    "Navigated down in history"
                );

                entry
            }
        }
    }

    /// Reset navigation state
    ///
    /// Call this when the user starts typing or submits input.
    pub fn reset_navigation(&mut self) {
        if self.current_index.is_some() {
            debug!("Reset input history navigation");
            self.current_index = None;
        }
    }

    /// Get the number of entries
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if history is empty
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get current navigation index (for debugging/testing)
    #[allow(dead_code)]
    pub fn current_index(&self) -> Option<usize> {
        self.current_index
    }

    /// Get all entries (for debugging/testing)
    #[allow(dead_code)]
    pub fn entries(&self) -> &[String] {
        &self.entries
    }

    /// Clear all entries
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.entries.clear();
        self.current_index = None;
        debug!("Cleared input history");
    }
}
/// Raw data format for JSON serialization
#[derive(Debug, Serialize, Deserialize)]
struct InputHistoryData {
    entries: Vec<String>,
}
