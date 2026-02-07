impl FrecencyStore {
    /// Create a new FrecencyStore with the default path (~/.scriptkit/frecency.json)
    pub fn new() -> Self {
        let file_path = Self::default_path();
        FrecencyStore {
            entries: HashMap::new(),
            file_path,
            dirty: false,
            half_life_days: DEFAULT_SUGGESTED_HALF_LIFE_DAYS,
            revision: 0,
        }
    }

    /// Create a FrecencyStore with config settings
    pub fn with_config(config: &SuggestedConfig) -> Self {
        let file_path = Self::default_path();
        FrecencyStore {
            entries: HashMap::new(),
            file_path,
            dirty: false,
            half_life_days: config.half_life_days,
            revision: 0,
        }
    }

    /// Create a FrecencyStore with a custom path (for testing)
    #[allow(dead_code)]
    pub fn with_path(path: PathBuf) -> Self {
        FrecencyStore {
            entries: HashMap::new(),
            file_path: path,
            dirty: false,
            half_life_days: DEFAULT_SUGGESTED_HALF_LIFE_DAYS,
            revision: 0,
        }
    }

    /// Update the half-life setting (e.g., after config reload)
    #[allow(dead_code)]
    pub fn set_half_life_days(&mut self, half_life_days: f64) {
        if (self.half_life_days - half_life_days).abs() > 0.001 {
            self.half_life_days = half_life_days;
            // Recalculate all scores with new half-life
            for entry in self.entries.values_mut() {
                entry.recalculate_score_with_half_life(half_life_days);
            }
            self.revision = self.revision.wrapping_add(1);
        }
    }

    /// Get the current half-life setting
    #[allow(dead_code)]
    pub fn half_life_days(&self) -> f64 {
        self.half_life_days
    }

    /// Get the current revision counter for cache invalidation
    ///
    /// This value increments on any change that affects ranking:
    /// record_use, remove, clear, set_half_life_days
    #[allow(dead_code)]
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// Get the default frecency file path
    fn default_path() -> PathBuf {
        PathBuf::from(shellexpand::tilde("~/.scriptkit/frecency.json").as_ref())
    }

    /// Load frecency data from disk
    ///
    /// Creates an empty store if the file doesn't exist.
    /// Recalculates all scores on load to account for time passed.
    #[instrument(name = "frecency_load", skip(self))]
    pub fn load(&mut self) -> Result<()> {
        if !self.file_path.exists() {
            info!(path = %self.file_path.display(), "Frecency file not found, starting fresh");
            return Ok(());
        }

        let content = std::fs::read_to_string(&self.file_path).with_context(|| {
            format!("Failed to read frecency file: {}", self.file_path.display())
        })?;

        let data: FrecencyData =
            serde_json::from_str(&content).with_context(|| "Failed to parse frecency JSON")?;

        self.entries = data.entries;

        // Recalculate all scores to account for time passed since last save
        let half_life = self.half_life_days;
        for entry in self.entries.values_mut() {
            entry.recalculate_score_with_half_life(half_life);
        }

        info!(
            path = %self.file_path.display(),
            entry_count = self.entries.len(),
            "Loaded frecency data"
        );

        self.dirty = false;
        Ok(())
    }

    /// Save frecency data to disk using atomic write (write temp + rename)
    ///
    /// Uses compact JSON for performance and atomic rename for crash safety.
    #[instrument(name = "frecency_save", skip(self))]
    pub fn save(&mut self) -> Result<()> {
        if !self.dirty {
            debug!("No changes to save");
            return Ok(());
        }

        // Ensure parent directory exists
        if let Some(parent) = self.file_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }

        // Serialize directly from reference to avoid cloning the entire map
        let json = serde_json::to_string(&FrecencyDataRef {
            entries: &self.entries,
        })
        .context("Failed to serialize frecency data")?;

        // Atomic write: write to temp file, then rename
        let temp_path = self.file_path.with_extension("json.tmp");

        // Write to temp file
        std::fs::write(&temp_path, &json).with_context(|| {
            format!(
                "Failed to write temp frecency file: {}",
                temp_path.display()
            )
        })?;

        // Atomic rename (on Unix, this is atomic; on Windows, it's best-effort)
        std::fs::rename(&temp_path, &self.file_path).with_context(|| {
            format!("Failed to rename temp file to {}", self.file_path.display())
        })?;

        info!(
            path = %self.file_path.display(),
            entry_count = self.entries.len(),
            bytes = json.len(),
            "Saved frecency data (atomic)"
        );

        self.dirty = false;
        Ok(())
    }

    /// Record a use of a script
    ///
    /// Uses the incremental frecency model: score = decayed_score + 1
    /// Creates a new entry if the script hasn't been used before.
    /// Uses the store's configured half-life for score calculation.
    #[instrument(name = "frecency_record_use", skip(self))]
    pub fn record_use(&mut self, path: &str) {
        let half_life = self.half_life_days;
        let now = current_timestamp();

        if let Some(entry) = self.entries.get_mut(path) {
            // Use incremental model: decay existing score, then add 1
            entry.record_use_with_timestamp(now, half_life);
            debug!(
                path = path,
                count = entry.count,
                score = entry.score,
                half_life_days = half_life,
                "Updated frecency entry (incremental model)"
            );
        } else {
            // New entry starts with score 1.0
            let entry = FrecencyEntry {
                count: 1,
                last_used: now,
                score: 1.0,
            };
            debug!(
                path = path,
                half_life_days = half_life,
                "Created new frecency entry"
            );
            self.entries.insert(path.to_string(), entry);
        }
        self.dirty = true;
        self.revision = self.revision.wrapping_add(1);
    }

    /// Record a use of a script at a specific timestamp (for testing)
    ///
    /// Same as `record_use()` but allows injecting a specific timestamp
    /// for deterministic testing.
    #[allow(dead_code)]
    pub fn record_use_at(&mut self, path: &str, timestamp: u64) {
        let half_life = self.half_life_days;

        if let Some(entry) = self.entries.get_mut(path) {
            entry.record_use_with_timestamp(timestamp, half_life);
            debug!(
                path = path,
                count = entry.count,
                score = entry.score,
                half_life_days = half_life,
                timestamp = timestamp,
                "Updated frecency entry at timestamp"
            );
        } else {
            let entry = FrecencyEntry {
                count: 1,
                last_used: timestamp,
                score: 1.0,
            };
            debug!(
                path = path,
                half_life_days = half_life,
                timestamp = timestamp,
                "Created new frecency entry at timestamp"
            );
            self.entries.insert(path.to_string(), entry);
        }
        self.dirty = true;
        self.revision = self.revision.wrapping_add(1);
    }

    /// Get the frecency score for a script
    ///
    /// Returns 0.0 if the script has never been used.
    pub fn get_score(&self, path: &str) -> f64 {
        self.entries.get(path).map(|e| e.score).unwrap_or(0.0)
    }

    /// Get the top N items by frecency score
    ///
    /// Computes live scores (with decay) for accurate ranking.
    /// Returns a vector of (path, score) tuples sorted by:
    /// 1. Score descending
    /// 2. Last used descending (tie-breaker)
    /// 3. Path ascending (final tie-breaker for determinism)
    pub fn get_recent_items(&self, limit: usize) -> Vec<(String, f64)> {
        let now = current_timestamp();
        let hl = self.half_life_days;

        // Compute live scores with decay for accurate ranking
        let mut items: Vec<_> = self
            .entries
            .iter()
            .map(|(path, entry)| {
                let live_score = entry.score_at(now, hl);
                (path.clone(), live_score, entry.last_used)
            })
            .collect();

        // Sort by score descending, then last_used descending, then path ascending
        items.sort_by(|a, b| {
            // Primary: score descending
            match b.1.partial_cmp(&a.1) {
                Some(std::cmp::Ordering::Equal) | None => {}
                Some(ord) => return ord,
            }
            // Secondary: last_used descending (more recent first)
            match b.2.cmp(&a.2) {
                std::cmp::Ordering::Equal => {}
                ord => return ord,
            }
            // Tertiary: path ascending (alphabetical)
            a.0.cmp(&b.0)
        });

        // Take top N and drop last_used from result
        items
            .into_iter()
            .take(limit)
            .map(|(path, score, _)| (path, score))
            .collect()
    }

    /// Get the top N items by frecency score at a specific timestamp (for testing)
    ///
    /// Same as `get_recent_items()` but allows querying at a specific timestamp
    /// for deterministic testing.
    #[allow(dead_code)]
    pub fn get_recent_items_at(&self, limit: usize, at_timestamp: u64) -> Vec<(String, f64)> {
        let hl = self.half_life_days;

        let mut items: Vec<_> = self
            .entries
            .iter()
            .map(|(path, entry)| {
                let live_score = entry.score_at(at_timestamp, hl);
                (path.clone(), live_score, entry.last_used)
            })
            .collect();

        items.sort_by(|a, b| {
            match b.1.partial_cmp(&a.1) {
                Some(std::cmp::Ordering::Equal) | None => {}
                Some(ord) => return ord,
            }
            match b.2.cmp(&a.2) {
                std::cmp::Ordering::Equal => {}
                ord => return ord,
            }
            a.0.cmp(&b.0)
        });

        items
            .into_iter()
            .take(limit)
            .map(|(path, score, _)| (path, score))
            .collect()
    }

    /// Get the number of tracked scripts
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the store is empty
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Check if there are unsaved changes
    #[allow(dead_code)]
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Remove an entry by path
    #[allow(dead_code)]
    pub fn remove(&mut self, path: &str) -> Option<FrecencyEntry> {
        let entry = self.entries.remove(path);
        if entry.is_some() {
            self.dirty = true;
            self.revision = self.revision.wrapping_add(1);
        }
        entry
    }

    /// Clear all entries
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        if !self.entries.is_empty() {
            self.entries.clear();
            self.dirty = true;
            self.revision = self.revision.wrapping_add(1);
        }
    }

    /// Prune stale entries to keep the frecency file bounded
    ///
    /// Removes entries where:
    /// - Live score (with decay) is below `score_threshold`
    /// - AND last_used is older than `min_age_days` days
    ///
    /// Returns the number of entries pruned.
    #[allow(dead_code)]
    pub fn prune_stale_entries(&mut self, score_threshold: f64, min_age_days: u64) -> usize {
        let now = current_timestamp();
        let hl = self.half_life_days;
        let min_age_seconds = min_age_days * SECONDS_PER_DAY as u64;

        let entries_before = self.entries.len();

        self.entries.retain(|_path, entry| {
            let live_score = entry.score_at(now, hl);
            let age_seconds = now.saturating_sub(entry.last_used);

            // Keep if: score is above threshold OR entry is recent enough
            live_score >= score_threshold || age_seconds < min_age_seconds
        });

        let pruned_count = entries_before - self.entries.len();

        if pruned_count > 0 {
            self.dirty = true;
            self.revision = self.revision.wrapping_add(1);
            debug!(
                pruned_count = pruned_count,
                remaining = self.entries.len(),
                score_threshold = score_threshold,
                min_age_days = min_age_days,
                "Pruned stale frecency entries"
            );
        }

        pruned_count
    }
}
impl Default for FrecencyStore {
    fn default() -> Self {
        Self::new()
    }
}
