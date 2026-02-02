# Terminal Search Functionality Research

## Files Investigated

1. **src/term_prompt.rs** - TermPrompt struct with:
   - Key handling via `handle_key` listener
   - `render_content()` method that batches cells for rendering
   - Selection highlighting via `selected_cells` HashSet
   - `suppress_keys` flag for when actions panel is open
   - Refresh timer that calls `cx.notify()` at ~60fps

2. **src/terminal/alacritty.rs** - TerminalHandle and TerminalContent:
   - `content()` method returns TerminalContent snapshot
   - TerminalContent contains: lines, styled_lines, cursor position, selected_cells
   - Selection APIs: start_selection, update_selection, clear_selection
   - Scroll APIs: scroll_page_up, scroll_to_bottom, etc.

3. **alacritty_terminal crate** (0.25.1):
   - `RegexSearch::new(pattern)` - creates regex search
   - `Term::search_next(regex, origin, direction)` - finds next match
   - Returns `Option<RangeInclusive<Point>>` for match range

## Current Behavior

- TermPrompt handles keyboard input and forwards to PTY
- Special keys: Escape (cancel), Shift+PageUp/Down (scrollback), Cmd+C (copy/interrupt), Cmd+V (paste)
- No search functionality exists yet
- Rendering batches cells by style for performance
- Selection is highlighted via selected_cells lookup

## Root Cause Analysis

Terminal lacks find/search functionality. The alacritty_terminal crate provides:
- `RegexSearch` for pattern matching
- `Term::search_next` for finding matches
- `Term::scroll_to_point` for scrolling to matches

However, these APIs are not currently exposed via TerminalHandle.

## Proposed Solution Approach

### Phase 1: Add Search State to TermPrompt

Add to TermPrompt struct:
- `search_mode: bool` - whether search is active
- `search_query: String` - current search text
- `search_matches: Vec<(usize, usize)>` - (line, col) of match starts
- `current_match_index: Option<usize>` - index into search_matches

### Phase 2: Implement Search Logic

Add method to search terminal content (simple string search, not regex):
```rust
fn search_content(&mut self, query: &str) {
    self.search_matches.clear();
    if query.is_empty() { return; }
    
    let content = self.terminal.content();
    let query_lower = query.to_lowercase();
    
    for (line_idx, line) in content.lines.iter().enumerate() {
        let line_lower = line.to_lowercase();
        let mut start = 0;
        while let Some(pos) = line_lower[start..].find(&query_lower) {
            self.search_matches.push((line_idx, start + pos));
            start += pos + 1;
        }
    }
    
    // Select first match
    if !self.search_matches.is_empty() {
        self.current_match_index = Some(0);
    }
}
```

### Phase 3: Add Cmd+F Handler

In key handling, intercept Cmd+F to toggle search mode.
When search_mode is true:
- Consume typing to update search_query
- Enter/Cmd+G: next match
- Shift+Cmd+G: previous match
- Escape: exit search mode

### Phase 4: Render Search UI

When search_mode is true:
- Show search input overlay at top of terminal
- Highlight all matches in render_content()
- Different color for current/active match

### Phase 5: Integrate with Command Bar

Add "Find" action to command bar that activates search mode.

## Key Files to Modify

1. `src/term_prompt.rs`:
   - Add search state fields to TermPrompt
   - Add search_content() method
   - Modify handle_key to intercept Cmd+F and handle search mode keys
   - Modify render_content() to highlight search matches
   - Add render for search overlay UI

2. `src/terminal/alacritty.rs`:
   - Consider adding search wrapper methods (optional - can do simple string search first)

## Verification

### What Was Changed

1. **src/term_prompt.rs** - Added terminal search functionality:
   - Added search state fields: `search_mode`, `search_query`, `search_matches`, `current_match_index`
   - Added search methods: `activate_search()`, `deactivate_search()`, `is_search_active()`, `update_search()`, `next_match()`, `prev_match()`, `match_info()`, `build_match_cells()`, `build_current_match_cells()`
   - Modified `render_content()` to highlight search matches (yellow for all matches, orange for current match)
   - Added Cmd+F key handler to toggle search mode
   - When in search mode: typing updates search, Enter/Cmd+G navigates next, Shift+Cmd+G navigates previous, Escape exits search
   - Added search UI overlay at top-right showing query, match count, and "Esc to close" hint

2. **Other files** - Fixed pre-existing issues that blocked compilation:
   - src/actions/types.rs - Fixed clippy empty-line-after-outer-attr, added dead_code allow for Terminal variant
   - src/terminal/mod.rs - Added allow(unused_imports) for terminal command exports
   - src/app_impl.rs - Added dead_code allow for toggle_terminal_commands

### Test Results

- All tests pass (20 passed, 58 ignored)
- cargo check passes
- cargo clippy --all-targets -- -D warnings passes

### Before/After Comparison

**Before:**
- No find/search functionality in terminal
- Users could not search through terminal scrollback

**After:**
- Press Cmd+F to activate search mode
- Type to search (case-insensitive)
- All matches highlighted in yellow
- Current match highlighted in orange
- Navigate with Enter/Cmd+G (next) and Shift+Cmd+G (previous)
- Press Escape to exit search mode
- Search UI overlay shows query text and match count

### Deviations from Proposed Solution

1. Used simple string search instead of alacritty's RegexSearch API to keep implementation simple and focused
2. Search matches are stored as (line, col) pairs directly instead of using alacritty's Point type
3. Search UI is rendered as absolute-positioned overlay rather than modifying terminal layout
4. Did not add Find action to command bar (can be added later if needed)
