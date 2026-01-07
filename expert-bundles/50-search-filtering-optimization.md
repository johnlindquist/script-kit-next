# Script Kit GPUI - Expert Bundle 50: Search/Filtering Optimization

## Project Context

Script Kit GPUI is a **Rust desktop app** built with GPUI (Zed's UI framework) that serves as a command launcher and script runner.

**Search Requirements:**
- Instant results as you type (~16ms latency target)
- Fuzzy matching across name, description, keywords
- Frecency-weighted results
- Cross-source search (scripts, scriptlets, built-ins, apps)

---

## Goal

Optimize the **search and filtering system** for:
1. Sub-16ms filter latency on 1000+ items
2. Better fuzzy matching quality
3. Smarter result ranking
4. Search history and suggestions
5. Advanced query syntax (optional)

---

## Current State

### Search Implementation

```rust
// scripts.rs - fuzzy matching
pub fn fuzzy_score(query: &str, text: &str) -> Option<(i64, Vec<usize>)> {
    let query_lower = query.to_lowercase();
    let text_lower = text.to_lowercase();
    
    // Use fuzzy_matcher crate
    let matcher = SkimMatcherV2::default();
    matcher.fuzzy_indices(&text_lower, &query_lower)
}

// get_filtered_results - called per keystroke
pub fn get_filtered_results(&self) -> Vec<SearchResult> {
    if self.filter_text.is_empty() {
        return self.get_all_results();
    }
    
    let mut results = Vec::new();
    
    // Search scripts
    for script in &self.scripts {
        if let Some((score, indices)) = fuzzy_score(&self.filter_text, &script.name) {
            results.push(SearchResult::Script(ScriptMatch {
                script: script.clone(),
                score,
                indices,
            }));
        }
    }
    
    // Search scriptlets, built-ins, apps...
    // ... similar loops
    
    results.sort_by(|a, b| b.score().cmp(&a.score()));
    results
}
```

### Performance Characteristics

| Items | Filter Time | Notes |
|-------|-------------|-------|
| 100 | ~2ms | Fast |
| 500 | ~8ms | Acceptable |
| 1000 | ~20ms | Noticeable lag |
| 2000 | ~45ms | Janky typing |

### Problems

1. **No Caching** - Full recompute on every keystroke
2. **Sequential Search** - Searches each source serially
3. **Substring Redundancy** - "gi" then "git" restarts from scratch
4. **No Debouncing** - Every keystroke triggers search
5. **Over-Cloning** - Scripts cloned for every match
6. **No Index** - Linear scan through all items

---

## Proposed Architecture

### 1. Search Index

```rust
/// Pre-built search index for fast lookups
pub struct SearchIndex {
    /// All searchable items
    items: Vec<IndexedItem>,
    /// Trigram index for fast prefix matching
    trigrams: HashMap<[u8; 3], Vec<usize>>,
    /// Word index for exact word matching
    words: HashMap<String, Vec<usize>>,
    /// Frecency scores (updated separately)
    frecency: HashMap<String, f64>,
}

pub struct IndexedItem {
    /// Unique identifier
    pub id: String,
    /// Primary searchable text (name)
    pub primary: String,
    /// Secondary searchable text (description)
    pub secondary: Option<String>,
    /// Additional keywords
    pub keywords: Vec<String>,
    /// Pre-lowercased primary for fast matching
    pub primary_lower: String,
    /// Item type for result construction
    pub item_type: IndexItemType,
    /// Original index into source array
    pub source_index: usize,
}

#[derive(Clone, Copy)]
pub enum IndexItemType {
    Script,
    Scriptlet,
    BuiltIn,
    App,
    Window,
}

impl SearchIndex {
    /// Build index from all sources
    pub fn build(
        scripts: &[Arc<Script>],
        scriptlets: &[Arc<Scriptlet>],
        builtins: &[BuiltInEntry],
        apps: &[AppInfo],
    ) -> Self {
        let mut index = Self::default();
        
        // Index scripts
        for (i, script) in scripts.iter().enumerate() {
            let item = IndexedItem {
                id: script.path.to_string_lossy().to_string(),
                primary: script.name.clone(),
                secondary: script.description.clone(),
                keywords: script.keywords.clone().unwrap_or_default(),
                primary_lower: script.name.to_lowercase(),
                item_type: IndexItemType::Script,
                source_index: i,
            };
            index.add_item(item);
        }
        
        // Index other sources similarly...
        
        index.build_trigrams();
        index.build_word_index();
        
        index
    }
    
    fn add_item(&mut self, item: IndexedItem) {
        self.items.push(item);
    }
    
    fn build_trigrams(&mut self) {
        for (idx, item) in self.items.iter().enumerate() {
            let bytes = item.primary_lower.as_bytes();
            for window in bytes.windows(3) {
                let trigram: [u8; 3] = window.try_into().unwrap();
                self.trigrams.entry(trigram).or_default().push(idx);
            }
        }
    }
    
    fn build_word_index(&mut self) {
        for (idx, item) in self.items.iter().enumerate() {
            // Index primary name words
            for word in item.primary_lower.split_whitespace() {
                self.words.entry(word.to_string()).or_default().push(idx);
            }
            // Index keywords
            for keyword in &item.keywords {
                self.words.entry(keyword.to_lowercase()).or_default().push(idx);
            }
        }
    }
}
```

### 2. Incremental Search

```rust
/// Search state that supports incremental refinement
pub struct SearchState {
    /// Current query
    query: String,
    /// Previous query (for incremental refinement)
    prev_query: Option<String>,
    /// Candidate item indices (refined incrementally)
    candidates: Vec<usize>,
    /// Scored results (computed from candidates)
    results: Vec<ScoredResult>,
    /// Whether results are fresh
    dirty: bool,
}

pub struct ScoredResult {
    pub item_index: usize,
    pub score: i64,
    pub match_indices: Vec<usize>,
}

impl SearchState {
    /// Update search with new query
    pub fn update(&mut self, query: &str, index: &SearchIndex) {
        if query == self.query {
            return; // No change
        }
        
        let query_lower = query.to_lowercase();
        
        // Determine if we can refine incrementally
        let can_refine = self.prev_query.as_ref()
            .map(|pq| query_lower.starts_with(pq))
            .unwrap_or(false);
        
        if can_refine && !self.candidates.is_empty() {
            // Incremental: Filter existing candidates
            self.candidates = self.candidates.iter()
                .copied()
                .filter(|&idx| {
                    index.items[idx].primary_lower.contains(&query_lower)
                })
                .collect();
        } else {
            // Full search: Use trigram index to find candidates
            self.candidates = self.find_candidates(&query_lower, index);
        }
        
        self.prev_query = Some(query_lower);
        self.query = query.to_string();
        self.dirty = true;
    }
    
    fn find_candidates(&self, query: &str, index: &SearchIndex) -> Vec<usize> {
        if query.len() < 3 {
            // Short query: linear scan with prefix match
            return index.items.iter()
                .enumerate()
                .filter(|(_, item)| item.primary_lower.contains(query))
                .map(|(i, _)| i)
                .collect();
        }
        
        // Use trigrams for candidate set
        let query_bytes = query.as_bytes();
        let mut candidate_counts: HashMap<usize, usize> = HashMap::new();
        
        for window in query_bytes.windows(3) {
            let trigram: [u8; 3] = window.try_into().unwrap();
            if let Some(indices) = index.trigrams.get(&trigram) {
                for &idx in indices {
                    *candidate_counts.entry(idx).or_default() += 1;
                }
            }
        }
        
        // Keep candidates with enough trigram matches
        let min_matches = (query.len() - 2).saturating_sub(1);
        candidate_counts.into_iter()
            .filter(|(_, count)| *count >= min_matches)
            .map(|(idx, _)| idx)
            .collect()
    }
    
    /// Get final results (computed lazily)
    pub fn results(&mut self, index: &SearchIndex) -> &[ScoredResult] {
        if self.dirty {
            self.compute_results(index);
            self.dirty = false;
        }
        &self.results
    }
    
    fn compute_results(&mut self, index: &SearchIndex) {
        let matcher = SkimMatcherV2::default();
        let query_lower = self.query.to_lowercase();
        
        self.results = self.candidates.iter()
            .filter_map(|&idx| {
                let item = &index.items[idx];
                
                // Score against primary text
                let (score, indices) = matcher.fuzzy_indices(&item.primary_lower, &query_lower)?;
                
                // Boost for frecency
                let frecency_boost = index.frecency.get(&item.id)
                    .map(|f| (f * 100.0) as i64)
                    .unwrap_or(0);
                
                Some(ScoredResult {
                    item_index: idx,
                    score: score + frecency_boost,
                    match_indices: indices,
                })
            })
            .collect();
        
        // Sort by score descending
        self.results.sort_by(|a, b| b.score.cmp(&a.score));
    }
}
```

### 3. Filter Coalescing

```rust
/// Coalesces rapid filter updates to reduce computation
pub struct FilterCoalescer {
    /// Pending query (latest)
    pending: Option<String>,
    /// Last applied query
    applied: String,
    /// Time of last update
    last_update: Instant,
    /// Minimum delay between applications
    debounce: Duration,
}

impl FilterCoalescer {
    pub fn new(debounce_ms: u64) -> Self {
        Self {
            pending: None,
            applied: String::new(),
            last_update: Instant::now(),
            debounce: Duration::from_millis(debounce_ms),
        }
    }
    
    /// Record a new filter value
    pub fn update(&mut self, query: String) {
        self.pending = Some(query);
        self.last_update = Instant::now();
    }
    
    /// Check if we should apply pending filter
    pub fn should_apply(&self) -> bool {
        self.pending.is_some() && 
        self.last_update.elapsed() >= self.debounce
    }
    
    /// Apply pending filter (returns new query if changed)
    pub fn apply(&mut self) -> Option<String> {
        if let Some(pending) = self.pending.take() {
            if pending != self.applied {
                self.applied = pending.clone();
                return Some(pending);
            }
        }
        None
    }
}
```

### 4. Parallel Search

```rust
/// Search multiple sources in parallel using rayon
pub fn parallel_search(
    query: &str,
    index: &SearchIndex,
    num_threads: usize,
) -> Vec<ScoredResult> {
    use rayon::prelude::*;
    
    let matcher = SkimMatcherV2::default();
    let query_lower = query.to_lowercase();
    
    // Chunk items for parallel processing
    let chunk_size = (index.items.len() / num_threads).max(100);
    
    index.items
        .par_chunks(chunk_size)
        .enumerate()
        .flat_map(|(chunk_idx, chunk)| {
            chunk.iter()
                .enumerate()
                .filter_map(|(i, item)| {
                    let global_idx = chunk_idx * chunk_size + i;
                    matcher.fuzzy_indices(&item.primary_lower, &query_lower)
                        .map(|(score, indices)| ScoredResult {
                            item_index: global_idx,
                            score,
                            match_indices: indices,
                        })
                })
                .collect::<Vec<_>>()
        })
        .collect()
}
```

### 5. Search History

```rust
/// Search history for suggestions and recall
pub struct SearchHistory {
    /// Recent queries (most recent first)
    queries: VecDeque<HistoryEntry>,
    /// Max entries to keep
    max_entries: usize,
    /// File path for persistence
    path: PathBuf,
}

pub struct HistoryEntry {
    pub query: String,
    pub timestamp: DateTime<Utc>,
    pub result_count: usize,
    pub selected_id: Option<String>,
}

impl SearchHistory {
    /// Add query to history
    pub fn record(&mut self, query: String, result_count: usize, selected_id: Option<String>) {
        // Don't record empty or very short queries
        if query.len() < 2 {
            return;
        }
        
        // Remove duplicate
        self.queries.retain(|e| e.query != query);
        
        // Add new entry
        self.queries.push_front(HistoryEntry {
            query,
            timestamp: Utc::now(),
            result_count,
            selected_id,
        });
        
        // Trim to max
        while self.queries.len() > self.max_entries {
            self.queries.pop_back();
        }
        
        // Persist
        self.save();
    }
    
    /// Get suggestions for current input
    pub fn suggest(&self, prefix: &str, limit: usize) -> Vec<&str> {
        let prefix_lower = prefix.to_lowercase();
        self.queries.iter()
            .filter(|e| e.query.to_lowercase().starts_with(&prefix_lower))
            .take(limit)
            .map(|e| e.query.as_str())
            .collect()
    }
    
    /// Get recent queries
    pub fn recent(&self, limit: usize) -> Vec<&str> {
        self.queries.iter()
            .take(limit)
            .map(|e| e.query.as_str())
            .collect()
    }
}
```

---

## Performance Targets

| Metric | Target | Current | Notes |
|--------|--------|---------|-------|
| Filter latency (100 items) | <2ms | ~2ms | Good |
| Filter latency (1000 items) | <10ms | ~20ms | Needs work |
| Filter latency (5000 items) | <20ms | N/A | Not tested |
| Index build time | <100ms | N/A | New |
| Memory overhead | <10MB | N/A | New |

---

## Implementation Checklist

### Phase 1: Index
- [ ] Create `SearchIndex` struct
- [ ] Build trigram index
- [ ] Build word index
- [ ] Update index on script changes

### Phase 2: Incremental Search
- [ ] Implement `SearchState`
- [ ] Add incremental refinement
- [ ] Cache results between renders

### Phase 3: Optimization
- [ ] Add `FilterCoalescer` (debouncing)
- [ ] Implement parallel search
- [ ] Profile and benchmark

### Phase 4: History
- [ ] Implement `SearchHistory`
- [ ] Persist to SQLite
- [ ] Add suggestions UI

### Phase 5: Advanced
- [ ] Support query operators (`-exclude`, `type:script`)
- [ ] Add typo tolerance
- [ ] Improve ranking algorithm

---

## Key Questions

1. Is 10ms latency noticeable to users? What's acceptable?
2. Should the index be rebuilt on every script change, or batched?
3. How much memory is acceptable for the index?
4. Should we use a dedicated search library (tantivy, meilisearch)?
5. How to balance fuzzy matching quality vs. speed?

---

## Related Bundles

- Bundle 34: Main Menu Patterns - displays search results
- Bundle 18: Frecency Bundle - ranking integration
- Bundle 43: Shared UI Components - list rendering
