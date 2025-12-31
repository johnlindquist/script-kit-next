# Input Lag Performance Expert Bundle

## Problem Statement

There is significant lag when typing in the main search input field of the Script Kit GPUI launcher. Users report noticeable delay between keystrokes and UI updates. We already implemented caching for `get_grouped_results()` (commit `0a5dc7b`) but lag persists.

The app is a Rust/GPUI launcher with fuzzy search across scripts, scriptlets, apps, and built-ins. Key events trigger `update_filter()` which recomputes search results and triggers UI updates.

## Files Included

| File | Role |
|------|------|
| `src/main.rs` | Main app: `ScriptListApp` struct, `update_filter()`, `NavCoalescer`, key handlers, `render_script_list()`, caching |
| `src/scripts.rs` | Fuzzy search: `fuzzy_search_*` functions, `get_grouped_results()`, scoring algorithms |
| `src/perf.rs` | Performance instrumentation: `KeyEventPerfGuard`, `TimingGuard`, thresholds |
| `src/window_resize.rs` | Window resize logic called on filter updates |
| `src/list_item.rs` | List item rendering, `ListItemColors`, `GroupedListItem` |

---

## src/main.rs (focused extractions)

### ScriptListApp struct and state (lines ~1580-1694)

```rust
struct ScriptListApp {
    scripts: Vec<scripts::Script>,
    scriptlets: Vec<scripts::Scriptlet>,
    builtin_entries: Vec<builtins::BuiltInEntry>,
    /// Cached list of installed applications for main search
    apps: Vec<app_launcher::AppInfo>,
    selected_index: usize,
    filter_text: String,
    last_output: Option<SharedString>,
    focus_handle: FocusHandle,
    show_logs: bool,
    theme: theme::Theme,
    #[allow(dead_code)]
    config: config::Config,
    // Scroll activity tracking for scrollbar fade
    /// Whether scroll activity is happening (scrollbar should be visible)
    is_scrolling: bool,
    /// Timestamp of last scroll activity (for fade-out timer)
    last_scroll_time: Option<std::time::Instant>,
    // Interactive script state
    current_view: AppView,
    script_session: SharedSession,
    // Prompt-specific state (used when view is ArgPrompt or DivPrompt)
    arg_input_text: String,
    arg_selected_index: usize,
    // Channel for receiving prompt messages from script thread (async_channel for event-driven)
    prompt_receiver: Option<async_channel::Receiver<PromptMessage>>,
    // Channel for sending responses back to script
    response_sender: Option<mpsc::Sender<Message>>,
    // List state for variable-height list (supports section headers at 24px + items at 48px)
    main_list_state: ListState,
    // Scroll handle for uniform_list (still used for backward compat in some views)
    list_scroll_handle: UniformListScrollHandle,
    // P0: Scroll handle for virtualized arg prompt choices
    arg_list_scroll_handle: UniformListScrollHandle,
    // Scroll handle for clipboard history list
    clipboard_list_scroll_handle: UniformListScrollHandle,
    // Scroll handle for window switcher list
    window_list_scroll_handle: UniformListScrollHandle,
    // Scroll handle for design gallery list
    design_gallery_scroll_handle: UniformListScrollHandle,
    // ... additional fields truncated
    last_scrolled_window: Option<usize>,
    #[allow(dead_code)]
    last_scrolled_design_gallery: Option<usize>,
}
```

### NavCoalescer - 20ms batching for arrow keys (lines ~1704-1796)

```rust
/// Direction of navigation (up/down arrow keys)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NavDirection {
    Up,
    Down,
}

/// Result of recording a navigation event
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NavRecord {
    /// First event - apply immediately (move by 1)
    ApplyImmediate,
    /// Same direction - coalesce (buffer additional movement)
    Coalesced,
    /// Direction changed - flush old delta, then apply new direction
    FlushOld { dir: NavDirection, delta: i32 },
}

/// Coalesces rapid arrow key events to prevent UI lag during fast keyboard repeat.
///
/// The coalescing window is 20ms - events within this window are batched together
/// and applied as a single larger movement at the next flush.
#[derive(Debug)]
struct NavCoalescer {
    /// Current pending direction (None if no pending movement)
    pending_dir: Option<NavDirection>,
    /// Accumulated delta for pending direction (# of additional moves beyond first)
    pending_delta: i32,
    /// Timestamp of last navigation event (for determining flush eligibility)
    last_event: std::time::Instant,
    /// Whether the background flush task is currently running
    flush_task_running: bool,
}

impl NavCoalescer {
    /// Coalescing window: 20ms between events triggers batching
    const WINDOW: std::time::Duration = std::time::Duration::from_millis(20);

    fn new() -> Self {
        Self {
            pending_dir: None,
            pending_delta: 0,
            last_event: std::time::Instant::now(),
            flush_task_running: false,
        }
    }

    /// Record a navigation event. Returns how to handle it:
    /// - ApplyImmediate: First event, move by 1 now
    /// - Coalesced: Same direction, buffered for later flush
    /// - FlushOld: Direction changed, flush old delta then move by 1
    fn record(&mut self, dir: NavDirection) -> NavRecord {
        self.last_event = std::time::Instant::now();
        match self.pending_dir {
            None => {
                // First event - start tracking this direction
                self.pending_dir = Some(dir);
                self.pending_delta = 0;
                NavRecord::ApplyImmediate
            }
            Some(existing) if existing == dir => {
                // Same direction - coalesce
                self.pending_delta += 1;
                NavRecord::Coalesced
            }
            // ... continues
        }
    }
}
```

### get_grouped_results_cached - caching layer (lines ~2226-2278)

```rust
/// P1: Get grouped results with caching - avoids recomputing 9+ times per keystroke
/// 
/// This is the ONLY place that should call scripts::get_grouped_results().
/// The cache is invalidated when filter_text changes.
/// 
/// Returns references to cached (grouped_items, flat_results).
fn get_grouped_results_cached(&mut self) -> (&Vec<GroupedListItem>, &Vec<scripts::SearchResult>) {
    // Check if cache is valid
    if self.filter_text == self.grouped_cache_key {
        logging::log_debug(
            "CACHE",
            &format!("Grouped cache HIT for '{}'", self.filter_text),
        );
        return (&self.cached_grouped_items, &self.cached_grouped_flat_results);
    }

    // Cache miss - need to recompute
    logging::log_debug(
        "CACHE",
        &format!("Grouped cache MISS - recomputing for '{}'", self.filter_text),
    );

    let start = std::time::Instant::now();
    let (grouped_items, flat_results) = get_grouped_results(
        &self.scripts,
        &self.scriptlets,
        &self.builtin_entries,
        &self.apps,
        &self.frecency_store,
        &self.filter_text,
    );
    let elapsed = start.elapsed();

    // Update cache
    self.cached_grouped_items = grouped_items;
    self.cached_grouped_flat_results = flat_results;
    self.grouped_cache_key = self.filter_text.clone();

    if !self.filter_text.is_empty() {
        logging::log_debug(
            "CACHE",
            &format!(
                "Grouped results computed in {:.2}ms for '{}' ({} items)",
                elapsed.as_secs_f64() * 1000.0,
                self.filter_text,
                self.cached_grouped_items.len()
            ),
        );
    }

    (&self.cached_grouped_items, &self.cached_grouped_flat_results)
}
```

### update_filter - called on every keystroke (lines ~2684-2718)

```rust
fn update_filter(
    &mut self,
    new_char: Option<char>,
    backspace: bool,
    clear: bool,
    cx: &mut Context<Self>,
) {
    if clear {
        self.filter_text.clear();
        self.selected_index = 0;
        self.last_scrolled_index = None;
        // Use main_list_state for variable-height list (not the legacy list_scroll_handle)
        self.main_list_state.scroll_to_reveal_item(0);
        self.last_scrolled_index = Some(0);
    } else if backspace && !self.filter_text.is_empty() {
        self.filter_text.pop();
        self.selected_index = 0;
        self.last_scrolled_index = None;
        // Use main_list_state for variable-height list (not the legacy list_scroll_handle)
        self.main_list_state.scroll_to_reveal_item(0);
        self.last_scrolled_index = Some(0);
    } else if let Some(ch) = new_char {
        self.filter_text.push(ch);
        self.selected_index = 0;
        self.last_scrolled_index = None;
        // Use main_list_state for variable-height list (not the legacy list_scroll_handle)
        self.main_list_state.scroll_to_reveal_item(0);
        self.last_scrolled_index = Some(0);
    }

    // Trigger window resize based on new filter results
    self.update_window_size();

    cx.notify();
}
```

### update_window_size - called after filter update (lines ~2725-2770)

```rust
/// Update window size based on current view and item count.
/// This implements dynamic window resizing:
/// - Script list: resize based on filtered results (including section headers)
/// - Arg prompt: resize based on filtered choices
/// - Div/Editor/Term: use full height
fn update_window_size(&mut self) {
    let (view_type, item_count) = match &self.current_view {
        AppView::ScriptList => {
            // Get grouped results which includes section headers (cached)
            let (grouped_items, _) = self.get_grouped_results_cached();
            let count = grouped_items.len();
            (ViewType::ScriptList, count)
        }
        AppView::ArgPrompt { choices, .. } => {
            let filtered = self.get_filtered_arg_choices(choices);
            if filtered.is_empty() && choices.is_empty() {
                (ViewType::ArgPromptNoChoices, 0)
            } else {
                (ViewType::ArgPromptWithChoices, filtered.len())
            }
        }
        AppView::DivPrompt { .. } => (ViewType::DivPrompt, 0),
        AppView::FormPrompt { .. } => (ViewType::DivPrompt, 0),
        AppView::EditorPrompt { .. } => (ViewType::EditorPrompt, 0),
        AppView::SelectPrompt { .. } => (ViewType::ArgPromptWithChoices, 0),
        AppView::PathPrompt { .. } => (ViewType::DivPrompt, 0),
        AppView::EnvPrompt { .. } => (ViewType::ArgPromptNoChoices, 0),
        AppView::DropPrompt { .. } => (ViewType::DivPrompt, 0),
        AppView::TemplatePrompt { .. } => (ViewType::DivPrompt, 0),
        AppView::TermPrompt { .. } => (ViewType::TermPrompt, 0),
        AppView::ActionsDialog => {
            // Actions dialog is an overlay, don't resize
            return;
        }
        // Clipboard history and app launcher use standard height (same as script list)
        AppView::ClipboardHistoryView { entries, filter, .. } => {
            let filtered_count = if filter.is_empty() {
                entries.len()
            } else {
                let filter_lower = filter.to_lowercase();
                entries
                    .iter()
                    .filter(|e| e.content.to_lowercase().contains(&filter_lower))
                    .count()
            };
            // ... continues
        }
        // ... more cases
    };
    // ... window resize logic
}
```

### render_script_list - main render function (lines ~7627-7908)

```rust
fn render_script_list(&mut self, cx: &mut Context<Self>) -> AnyElement {
    // Get grouped or flat results based on filter state (cached) - MUST come first
    // to avoid borrow conflicts with theme access below
    // When filter is empty, use frecency-grouped results with RECENT/MAIN sections
    // When filtering, use flat fuzzy search results
    let (grouped_items, flat_results) = self.get_grouped_results_cached();
    // Clone for use in closures and to avoid borrow issues
    let grouped_items = grouped_items.clone();
    let flat_results = flat_results.clone();

    // Get design tokens for current design variant
    let tokens = get_tokens(self.current_design);
    let design_colors = tokens.colors();
    let design_spacing = tokens.spacing();
    let design_visual = tokens.visual();
    let design_typography = tokens.typography();
    let theme = &self.theme;

    // For Default design, use theme.colors for backward compatibility
    // For other designs, use design tokens
    let is_default_design = self.current_design == DesignVariant::Default;

    // P4: Pre-compute theme values using ListItemColors
    let _list_colors = ListItemColors::from_theme(theme);
    logging::log_debug("PERF", "P4: Using ListItemColors for render closure");

    let item_count = grouped_items.len();
    let _total_len = self.scripts.len() + self.scriptlets.len();

    // Handle edge cases - keep selected_index in valid bounds
    // Also skip section headers when adjusting bounds
    if item_count > 0 {
        if self.selected_index >= item_count {
            self.selected_index = item_count.saturating_sub(1);
        }
        // If we land on a section header, move to first valid item
        if let Some(GroupedListItem::SectionHeader(_)) = grouped_items.get(self.selected_index) {
            // Move down to find first Item
            for (i, item) in grouped_items.iter().enumerate().skip(self.selected_index) {
                if matches!(item, GroupedListItem::Item(_)) {
                    // ... continues
                }
            }
        }
    }
    
    // ... extensive render logic continues (variable_height_list, scrollbar, log_panel, etc.)
}
```

### Key event handler (lines ~7910-8000+)

```rust
let handle_key = cx.listener(
    move |this: &mut Self,
          event: &gpui::KeyDownEvent,
          window: &mut Window,
          cx: &mut Context<Self>| {
        let key_str = event.keystroke.key.to_lowercase();
        let has_cmd = event.keystroke.modifiers.platform;

        // Check SDK action shortcuts FIRST (before built-in shortcuts)
        // This allows scripts to override default shortcuts via setActions()
        if !this.action_shortcuts.is_empty() {
            let key_combo =
                Self::keystroke_to_shortcut(&key_str, &event.keystroke.modifiers);
            if let Some(action_name) = this.action_shortcuts.get(&key_combo).cloned() {
                logging::log(
                    "ACTIONS",
                    &format!(
                        "SDK action shortcut matched: '{}' -> '{}'",
                        key_combo, action_name
                    ),
                );
                if this.trigger_action_by_name(&action_name, cx) {
                    return;
                }
            }
        }

        if has_cmd {
            let has_shift = event.keystroke.modifiers.shift;

            match key_str.as_str() {
                "l" => {
                    this.toggle_logs(cx);
                    return;
                }
                "k" => {
                    this.toggle_actions(cx, window);
                    return;
                }
                // Cmd+1 cycles through all designs
                "1" => {
                    // ... design cycling
                }
                // ... more cases
            }
        }
        
        // ... character input handling eventually calls update_filter()
    },
);
```

### move_selection_up/down (lines ~2407-2506)

```rust
fn move_selection_up(&mut self, cx: &mut Context<Self>) {
    // Get grouped results to check for section headers (cached)
    let (grouped_items, _) = self.get_grouped_results_cached();
    // Clone to avoid borrow issues with self mutation below
    let grouped_items = grouped_items.clone();

    // Find the first selectable (non-header) item index
    let first_selectable = grouped_items
        .iter()
        .position(|item| matches!(item, GroupedListItem::Item(_)));

    // If already at or before first selectable, can't go further up
    if let Some(first) = first_selectable {
        if self.selected_index <= first {
            // Already at the first selectable item, stay here
            return;
        }
    }

    if self.selected_index > 0 {
        let mut new_index = self.selected_index - 1;

        // Skip section headers when moving up
        while new_index > 0 {
            if let Some(GroupedListItem::SectionHeader(_)) = grouped_items.get(new_index) {
                new_index -= 1;
            } else {
                break;
            }
        }

        // Make sure we didn't land on a section header at index 0
        if let Some(GroupedListItem::SectionHeader(_)) = grouped_items.get(new_index) {
            // Stay at current position if we can't find a valid item
            return;
        }

        self.selected_index = new_index;
        self.scroll_to_selected_if_needed("keyboard_up");
        self.trigger_scroll_activity(cx);
        cx.notify();
    }
}

fn move_selection_down(&mut self, cx: &mut Context<Self>) {
    // Get grouped results to check for section headers (cached)
    let (grouped_items, _) = self.get_grouped_results_cached();
    // Clone to avoid borrow issues with self mutation below
    let grouped_items = grouped_items.clone();

    let item_count = grouped_items.len();

    // Find the last selectable (non-header) item index
    let last_selectable = grouped_items
        .iter()
        .rposition(|item| matches!(item, GroupedListItem::Item(_)));

    // If already at or after last selectable, can't go further down
    if let Some(last) = last_selectable {
        if self.selected_index >= last {
            // Already at the last selectable item, stay here
            return;
        }
    }

    if self.selected_index < item_count.saturating_sub(1) {
        let mut new_index = self.selected_index + 1;

        // Skip section headers when moving down
        while new_index < item_count.saturating_sub(1) {
            if let Some(GroupedListItem::SectionHeader(_)) = grouped_items.get(new_index) {
                new_index += 1;
            } else {
                break;
            }
        }

        // Make sure we didn't land on a section header at the end
        if let Some(GroupedListItem::SectionHeader(_)) = grouped_items.get(new_index) {
            // Stay at current position if we can't find a valid item
            return;
        }

        self.selected_index = new_index;
        self.scroll_to_selected_if_needed("keyboard_down");
        // ... continues
    }
}
```

---

## src/scripts.rs (fuzzy search functions)

### is_fuzzy_match and fuzzy_match_with_indices (lines ~821-851)

```rust
/// Check if a pattern is a fuzzy match for haystack (characters appear in order)
fn is_fuzzy_match(haystack: &str, pattern: &str) -> bool {
    let mut pattern_chars = pattern.chars().peekable();
    for ch in haystack.chars() {
        if let Some(&p) = pattern_chars.peek() {
            if ch.eq_ignore_ascii_case(&p) {
                pattern_chars.next();
            }
        }
    }
    pattern_chars.peek().is_none()
}

/// Perform fuzzy matching and return the indices of matched characters
/// Returns (matched, indices) where matched is true if all pattern chars found in order
fn fuzzy_match_with_indices(haystack: &str, pattern: &str) -> (bool, Vec<usize>) {
    let mut indices = Vec::new();
    let mut pattern_chars = pattern.chars().peekable();

    for (idx, ch) in haystack.chars().enumerate() {
        if let Some(&p) = pattern_chars.peek() {
            if ch.eq_ignore_ascii_case(&p) {
                indices.push(idx);
                pattern_chars.next();
            }
        }
    }

    let matched = pattern_chars.peek().is_none();
    (matched, if matched { indices } else { Vec::new() })
}
```

### fuzzy_search_scripts (lines ~884-977)

```rust
/// Fuzzy search scripts by query string
/// Searches across name, filename (e.g., "my-script.ts"), description, and path
/// Returns results sorted by relevance score (highest first)
/// Match indices are provided to enable UI highlighting of matched characters
pub fn fuzzy_search_scripts(scripts: &[Script], query: &str) -> Vec<ScriptMatch> {
    if query.is_empty() {
        // If no query, return all scripts with equal score, sorted by name
        return scripts
            .iter()
            .map(|s| {
                let filename = extract_filename(&s.path);
                ScriptMatch {
                    script: s.clone(),
                    score: 0,
                    filename,
                    match_indices: MatchIndices::default(),
                }
            })
            .collect();
    }

    let query_lower = query.to_lowercase();
    let mut matches = Vec::new();

    for script in scripts {
        let mut score = 0i32;
        let mut match_indices = MatchIndices::default();

        let name_lower = script.name.to_lowercase();
        let filename = extract_filename(&script.path);
        let filename_lower = filename.to_lowercase();

        // Score by name match - highest priority
        if let Some(pos) = name_lower.find(&query_lower) {
            // Bonus for exact substring match at start of name
            score += if pos == 0 { 100 } else { 75 };
        }

        // Fuzzy character matching in name (characters in order)
        let (name_fuzzy_matched, name_indices) =
            fuzzy_match_with_indices(&name_lower, &query_lower);
        if name_fuzzy_matched {
            score += 50;
            match_indices.name_indices = name_indices;
        }

        // ... filename matching, description matching, path matching ...

        if score > 0 {
            matches.push(ScriptMatch {
                script: script.clone(),
                score,
                filename,
                match_indices,
            });
        }
    }

    // Sort by score (highest first), then by name for ties
    matches.sort_by(|a, b| match b.score.cmp(&a.score) {
        Ordering::Equal => a.script.name.cmp(&b.script.name),
        other => other,
    });

    matches
}
```

### fuzzy_search_scriptlets (lines ~979-1081)

```rust
/// Fuzzy search scriptlets by query string
/// Searches across name, file_path with anchor (e.g., "url.md#open-github"), description, and code
/// Returns results sorted by relevance score (highest first)
pub fn fuzzy_search_scriptlets(scriptlets: &[Scriptlet], query: &str) -> Vec<ScriptletMatch> {
    if query.is_empty() {
        return scriptlets
            .iter()
            .map(|s| {
                let display_file_path = extract_scriptlet_display_path(&s.file_path);
                ScriptletMatch {
                    scriptlet: s.clone(),
                    score: 0,
                    display_file_path,
                    match_indices: MatchIndices::default(),
                }
            })
            .collect();
    }

    let query_lower = query.to_lowercase();
    let mut matches = Vec::new();

    for scriptlet in scriptlets {
        let mut score = 0i32;
        let mut match_indices = MatchIndices::default();

        let name_lower = scriptlet.name.to_lowercase();
        // ... similar scoring logic ...

        // Score by description match - medium priority
        if let Some(ref desc) = scriptlet.description {
            if desc.to_lowercase().contains(&query_lower) {
                score += 25;
            }
        }

        // Score by code content match - lower priority
        if scriptlet.code.to_lowercase().contains(&query_lower) {
            score += 5;
        }

        // Bonus for tool type match
        if scriptlet.tool.to_lowercase().contains(&query_lower) {
            score += 10;
        }

        if score > 0 {
            matches.push(ScriptletMatch { /* ... */ });
        }
    }

    matches.sort_by(|a, b| match b.score.cmp(&a.score) {
        Ordering::Equal => a.scriptlet.name.cmp(&b.scriptlet.name),
        other => other,
    });

    matches
}
```

### fuzzy_search_builtins and fuzzy_search_apps (lines ~1083-1215)

```rust
/// Fuzzy search built-in entries by query string
pub fn fuzzy_search_builtins(entries: &[BuiltInEntry], query: &str) -> Vec<BuiltInMatch> {
    if query.is_empty() {
        return entries
            .iter()
            .map(|e| BuiltInMatch { entry: e.clone(), score: 0 })
            .collect();
    }

    let query_lower = query.to_lowercase();
    let mut matches = Vec::new();

    for entry in entries {
        let mut score = 0i32;
        let name_lower = entry.name.to_lowercase();

        // Score by name match - highest priority
        if let Some(pos) = name_lower.find(&query_lower) {
            score += if pos == 0 { 100 } else { 75 };
        }

        // Fuzzy character matching in name
        if is_fuzzy_match(&name_lower, &query_lower) {
            score += 50;
        }

        // Score by description match
        if entry.description.to_lowercase().contains(&query_lower) {
            score += 25;
        }

        // Score by keyword match - high priority
        for keyword in &entry.keywords {
            if keyword.to_lowercase().contains(&query_lower) {
                score += 75;
                break;
            }
        }

        // Fuzzy match on keywords
        for keyword in &entry.keywords {
            if is_fuzzy_match(&keyword.to_lowercase(), &query_lower) {
                score += 30;
                break;
            }
        }

        if score > 0 {
            matches.push(BuiltInMatch { entry: entry.clone(), score });
        }
    }

    matches.sort_by(|a, b| match b.score.cmp(&a.score) {
        Ordering::Equal => a.entry.name.cmp(&b.entry.name),
        other => other,
    });

    matches
}

/// Fuzzy search applications by query string
pub fn fuzzy_search_apps(apps: &[crate::app_launcher::AppInfo], query: &str) -> Vec<AppMatch> {
    // Similar pattern - iterate, score, sort
    // ...
}
```

### fuzzy_search_unified_all (lines ~1326-1385)

```rust
/// Perform unified fuzzy search across scripts, scriptlets, built-ins, and apps
/// Returns combined and ranked results sorted by relevance
pub fn fuzzy_search_unified_all(
    scripts: &[Script],
    scriptlets: &[Scriptlet],
    builtins: &[BuiltInEntry],
    apps: &[crate::app_launcher::AppInfo],
    query: &str,
) -> Vec<SearchResult> {
    let mut results = Vec::new();

    // Search built-ins first (they should appear at top when scores are equal)
    let builtin_matches = fuzzy_search_builtins(builtins, query);
    for bm in builtin_matches {
        results.push(SearchResult::BuiltIn(bm));
    }

    // Search apps (appear after built-ins but before scripts)
    let app_matches = fuzzy_search_apps(apps, query);
    for am in app_matches {
        results.push(SearchResult::App(am));
    }

    // Search scripts
    let script_matches = fuzzy_search_scripts(scripts, query);
    for sm in script_matches {
        results.push(SearchResult::Script(sm));
    }

    // Search scriptlets
    let scriptlet_matches = fuzzy_search_scriptlets(scriptlets, query);
    for sm in scriptlet_matches {
        results.push(SearchResult::Scriptlet(sm));
    }

    // Sort by score (highest first), then by type, then by name
    results.sort_by(|a, b| {
        match b.score().cmp(&a.score()) {
            Ordering::Equal => {
                // Prefer builtins over apps over windows over scripts over scriptlets
                let type_order = |r: &SearchResult| -> i32 {
                    match r {
                        SearchResult::BuiltIn(_) => 0,
                        SearchResult::App(_) => 1,
                        SearchResult::Window(_) => 2,
                        SearchResult::Script(_) => 3,
                        SearchResult::Scriptlet(_) => 4,
                    }
                };
                let type_order_a = type_order(a);
                let type_order_b = type_order(b);
                match type_order_a.cmp(&type_order_b) {
                    Ordering::Equal => a.name().cmp(b.name()),
                    other => other,
                }
            }
            other => other,
        }
    });

    results
}
```

### get_grouped_results (lines ~1458-1538)

```rust
/// Maximum number of items to show in the RECENT section
const MAX_RECENT_ITEMS: usize = 10;

/// Get grouped results with RECENT/MAIN sections based on frecency
#[instrument(level = "debug", skip_all, fields(filter_len = filter_text.len()))]
pub fn get_grouped_results(
    scripts: &[Script],
    scriptlets: &[Scriptlet],
    builtins: &[BuiltInEntry],
    apps: &[AppInfo],
    frecency_store: &FrecencyStore,
    filter_text: &str,
) -> (Vec<GroupedListItem>, Vec<SearchResult>) {
    // Get all unified search results
    let results = fuzzy_search_unified_all(scripts, scriptlets, builtins, apps, filter_text);

    // Search mode: return flat list with no headers
    if !filter_text.is_empty() {
        let grouped: Vec<GroupedListItem> = (0..results.len()).map(GroupedListItem::Item).collect();
        debug!(
            result_count = results.len(),
            "Search mode: returning flat list"
        );
        return (grouped, results);
    }

    // Grouped view mode: create RECENT and MAIN sections
    let mut grouped = Vec::new();

    // Get recent items from frecency store
    let recent_items = frecency_store.get_recent_items(MAX_RECENT_ITEMS);

    // Build a set of paths that are "recent" (have frecency score > 0)
    let recent_paths: std::collections::HashSet<String> = recent_items
        .iter()
        .filter(|(_, score): &&(String, f64)| *score > 0.0)
        .map(|(path, _): &(String, f64)| path.clone())
        .collect();

    // Map each result to its frecency score (if any)
    let get_result_path = |result: &SearchResult| -> Option<String> {
        match result {
            SearchResult::Script(sm) => Some(sm.script.path.to_string_lossy().to_string()),
            SearchResult::App(am) => Some(am.app.path.to_string_lossy().to_string()),
            SearchResult::BuiltIn(bm) => Some(format!("builtin:{}", bm.entry.name)),
            // ... more cases
        }
    };
    
    // ... continues with grouping logic
}
```

---

## src/perf.rs (full file)

```rust
#![allow(dead_code)]
//! Performance instrumentation and benchmarking utilities

use std::collections::VecDeque;
use std::sync::Mutex;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use tracing::{debug, warn};

// =============================================================================
// CONFIGURATION
// =============================================================================

/// Maximum number of samples to keep for rolling averages
const MAX_SAMPLES: usize = 100;

/// Threshold for "slow" key event processing (microseconds)
const SLOW_KEY_THRESHOLD_US: u128 = 16_666; // ~16ms (60fps frame budget)

/// Threshold for "slow" scroll operation (microseconds)
const SLOW_SCROLL_THRESHOLD_US: u128 = 8_000; // 8ms

// =============================================================================
// KEY EVENT TRACKING
// =============================================================================

/// Tracks key event timing and rate
pub struct KeyEventTracker {
    event_times: VecDeque<Instant>,
    processing_durations: VecDeque<Duration>,
    last_event: Option<Instant>,
    slow_event_count: usize,
    total_events: usize,
}

impl KeyEventTracker {
    pub fn new() -> Self {
        Self {
            event_times: VecDeque::with_capacity(MAX_SAMPLES),
            processing_durations: VecDeque::with_capacity(MAX_SAMPLES),
            last_event: None,
            slow_event_count: 0,
            total_events: 0,
        }
    }

    pub fn start_event(&mut self) -> Instant {
        let now = Instant::now();
        if self.event_times.len() >= MAX_SAMPLES {
            self.event_times.pop_front();
        }
        self.event_times.push_back(now);
        self.total_events += 1;
        now
    }

    pub fn end_event(&mut self, start: Instant) {
        let duration = start.elapsed();
        if self.processing_durations.len() >= MAX_SAMPLES {
            self.processing_durations.pop_front();
        }
        self.processing_durations.push_back(duration);
        if duration.as_micros() > SLOW_KEY_THRESHOLD_US {
            self.slow_event_count += 1;
        }
        self.last_event = Some(start);
    }

    pub fn events_per_second(&self) -> f64 {
        if self.event_times.len() < 2 { return 0.0; }
        let first = self.event_times.front().unwrap();
        let last = self.event_times.back().unwrap();
        let elapsed = last.duration_since(*first);
        if elapsed.as_secs_f64() < 0.001 { return 0.0; }
        (self.event_times.len() - 1) as f64 / elapsed.as_secs_f64()
    }

    pub fn avg_processing_time_us(&self) -> u128 {
        if self.processing_durations.is_empty() { return 0; }
        let total: Duration = self.processing_durations.iter().sum();
        total.as_micros() / self.processing_durations.len() as u128
    }

    pub fn slow_event_percentage(&self) -> f64 {
        if self.total_events == 0 { return 0.0; }
        (self.slow_event_count as f64 / self.total_events as f64) * 100.0
    }

    pub fn log_stats(&self) {
        debug!(
            category = "KEY_PERF",
            rate_per_sec = self.events_per_second(),
            avg_ms = self.avg_processing_time_us() as f64 / 1000.0,
            slow_percent = self.slow_event_percentage(),
            total_events = self.total_events,
            "Key event statistics"
        );
    }
}

// =============================================================================
// RAII PERF GUARDS
// =============================================================================

/// RAII guard that records key-event timing into KeyEventTracker
pub struct KeyEventPerfGuard {
    start: Instant,
    _timing: TimingGuard,
}

impl Default for KeyEventPerfGuard {
    fn default() -> Self { Self::new() }
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
        log_key_rate();
    }
}

/// RAII guard for timing operations - logs when dropped
pub struct TimingGuard {
    operation: &'static str,
    start: Instant,
    threshold_us: u128,
}

impl TimingGuard {
    pub fn new(operation: &'static str, threshold_us: u128) -> Self {
        Self { operation, start: Instant::now(), threshold_us }
    }

    pub fn key_event() -> Self {
        Self::new("key_event", SLOW_KEY_THRESHOLD_US)
    }

    pub fn scroll() -> Self {
        Self::new("scroll", SLOW_SCROLL_THRESHOLD_US)
    }
}

impl Drop for TimingGuard {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        let duration_us = duration.as_micros();
        if duration_us > self.threshold_us {
            warn!(
                category = "PERF_SLOW",
                operation = self.operation,
                duration_ms = duration_us as f64 / 1000.0,
                threshold_ms = self.threshold_us as f64 / 1000.0,
                "Slow operation detected"
            );
        }
    }
}

// Global tracker and convenience functions...
static PERF_TRACKER: OnceLock<Mutex<PerfTracker>> = OnceLock::new();

pub fn get_perf_tracker() -> &'static Mutex<PerfTracker> {
    PERF_TRACKER.get_or_init(|| Mutex::new(PerfTracker::new()))
}

pub fn start_key_event() -> Instant {
    if let Ok(mut tracker) = get_perf_tracker().lock() {
        tracker.key_events.start_event()
    } else {
        Instant::now()
    }
}

pub fn end_key_event(start: Instant) {
    if let Ok(mut tracker) = get_perf_tracker().lock() {
        tracker.key_events.end_event(start);
    }
}

pub fn log_key_rate() {
    if let Ok(tracker) = get_perf_tracker().lock() {
        let rate = tracker.key_events.events_per_second();
        if rate > 20.0 {
            warn!(
                category = "KEY_PERF",
                rate_per_sec = rate,
                "High key event rate detected"
            );
        }
    }
}
```

---

## src/window_resize.rs (full file)

```rust
//! Dynamic Window Resizing Module

#[cfg(target_os = "macos")]
use cocoa::foundation::{NSPoint, NSRect, NSSize};
#[cfg(target_os = "macos")]
use objc::{msg_send, sel, sel_impl};

use gpui::{px, Context, Pixels, Render, Timer};
use std::time::Duration;
use tracing::{info, warn};

use crate::logging;
use crate::window_manager;

/// Layout constants for height calculations
pub mod layout {
    use gpui::{px, Pixels};

    pub const MIN_HEIGHT: Pixels = px(120.0);
    pub const STANDARD_HEIGHT: Pixels = px(500.0);
    pub const MAX_HEIGHT: Pixels = px(700.0);
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewType {
    ScriptList,
    ArgPromptWithChoices,
    ArgPromptNoChoices,
    DivPrompt,
    EditorPrompt,
    TermPrompt,
}

pub fn height_for_view(view_type: ViewType, _item_count: usize) -> Pixels {
    use layout::*;

    let height = match view_type {
        ViewType::ScriptList | ViewType::ArgPromptWithChoices | ViewType::DivPrompt => {
            STANDARD_HEIGHT
        }
        ViewType::ArgPromptNoChoices => MIN_HEIGHT,
        ViewType::EditorPrompt | ViewType::TermPrompt => MAX_HEIGHT,
    };

    let height_px = f32::from(height);
    info!(
        view_type = ?view_type,
        height_px = height_px,
        "height_for_view calculated"
    );
    logging::log(
        "RESIZE",
        &format!("height_for_view({:?}) = {:.0}", view_type, height_px),
    );

    height
}

pub fn defer_resize_to_view<T: Render>(
    view_type: ViewType,
    item_count: usize,
    cx: &mut Context<T>,
) {
    let target_height = height_for_view(view_type, item_count);

    cx.spawn(async move |_this, _cx: &mut gpui::AsyncApp| {
        Timer::after(Duration::from_millis(16)).await;

        if window_manager::get_main_window().is_some() {
            resize_first_window_to_height(target_height);
        } else {
            warn!("defer_resize_to_view: window no longer exists, skipping resize");
        }
    })
    .detach();
}

#[cfg(target_os = "macos")]
pub fn resize_first_window_to_height(target_height: Pixels) {
    let height_f64: f64 = f32::from(target_height) as f64;

    let window = match window_manager::get_main_window() {
        Some(w) => w,
        None => {
            warn!("Main window not registered in WindowManager, cannot resize");
            return;
        }
    };

    unsafe {
        let current_frame: NSRect = msg_send![window, frame];
        let current_height = current_frame.size.height;
        
        if (current_height - height_f64).abs() < 1.0 {
            info!("Skip resize - already at target height");
            return;
        }

        info!(from_height = current_height, to_height = height_f64, "Resizing window");

        let height_delta = height_f64 - current_height;
        let new_origin_y = current_frame.origin.y - height_delta;

        let new_frame = NSRect::new(
            NSPoint::new(current_frame.origin.x, new_origin_y),
            NSSize::new(current_frame.size.width, height_f64),
        );

        let _: () = msg_send![window, setFrame:new_frame display:true animate:false];
    }
}
```

---

## src/list_item.rs (key excerpts)

```rust
/// Fixed height for list items used in uniform-height virtualized lists.
pub const LIST_ITEM_HEIGHT: f32 = 48.0;

/// Fixed height for section headers (RECENT, MAIN, etc.)
pub const SECTION_HEADER_HEIGHT: f32 = 24.0;

/// Enum for grouped list items
#[derive(Clone, Debug)]
pub enum GroupedListItem {
    SectionHeader(String),
    Item(usize),
}

/// Pre-computed colors for ListItem rendering
#[derive(Clone, Copy)]
pub struct ListItemColors {
    pub text_primary: u32,
    pub text_secondary: u32,
    pub text_muted: u32,
    pub text_dimmed: u32,
    pub accent_selected: u32,
    pub accent_selected_subtle: u32,
    pub background: u32,
    pub background_selected: u32,
}

impl ListItemColors {
    pub fn from_theme(theme: &crate::theme::Theme) -> Self {
        Self {
            text_primary: theme.colors.text.primary,
            text_secondary: theme.colors.text.secondary,
            text_muted: theme.colors.text.muted,
            text_dimmed: theme.colors.text.dimmed,
            accent_selected: theme.colors.accent.selected,
            accent_selected_subtle: theme.colors.accent.selected_subtle,
            background: theme.colors.background.main,
            background_selected: theme.colors.accent.selected_subtle,
        }
    }
}

/// A reusable list item component
#[derive(IntoElement)]
pub struct ListItem {
    name: SharedString,
    description: Option<String>,
    shortcut: Option<String>,
    icon: Option<IconKind>,
    selected: bool,
    hovered: bool,
    colors: ListItemColors,
    index: Option<usize>,
    on_hover: Option<OnHoverCallback>,
    semantic_id: Option<String>,
    show_accent_bar: bool,
}

impl RenderOnce for ListItem {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let colors = self.colors;
        // ... builds flex row with icon, name, description, shortcut
        // Uses h(px(LIST_ITEM_HEIGHT)) for fixed height
    }
}
```

---

## Instructions For The Next AI Agent

You are receiving this bundle to analyze input lag in a Rust/GPUI application. The user reports significant lag when typing in the main search input.

**Your task:**
1. Analyze the code paths from keystroke to UI update
2. Identify performance bottlenecks
3. Propose specific fixes with code examples

**Key observations to investigate:**
- What happens on each keystroke? Trace through `handle_key` -> `update_filter` -> `update_window_size` -> `cx.notify()`
- How does the caching in `get_grouped_results_cached()` actually help? When is it hit vs miss?
- Are there unnecessary clones? The code clones `grouped_items` and `flat_results` in multiple places
- Is `update_window_size()` doing expensive work on every keystroke?
- What does `cx.notify()` trigger? Is the entire view re-rendered?
- Arrow key navigation has 20ms coalescing (`NavCoalescer`) - why doesn't character input have similar batching?
- Are the fuzzy search functions (`fuzzy_search_scripts`, etc.) efficient? They iterate and allocate on every query
- How many items are being searched? (scripts + scriptlets + builtins + apps)

**Performance thresholds from `src/perf.rs`:**
- Target: < 16.67ms per key event (60fps frame budget)
- Slow scroll: > 8ms
- The `KeyEventPerfGuard` and `TimingGuard` can help measure actual performance

**Do NOT:**
- Ask the user to test - propose concrete fixes
- Make assumptions without citing specific code
- Skip any of the files in this bundle

**Output format:**
Provide a prioritized list of issues with:
1. Root cause (cite line numbers/function names)
2. Performance impact (estimated ms or %)
3. Proposed fix (actual code changes)
4. Risk assessment (breaking changes, side effects)

OUTPUT_FILE_PATH: expert-bundles/input-lag-performance.md
