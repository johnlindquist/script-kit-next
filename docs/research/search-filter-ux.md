# Search and Filtering UX Research for Launchers

This document summarizes best practices for search and filtering UX in launcher applications, with recommendations for Script Kit.

---

## Table of Contents

1. [Fuzzy Search Algorithms and UX](#fuzzy-search-algorithms-and-ux)
2. [Search Relevance and Ranking](#search-relevance-and-ranking)
3. [Instant Search vs Debounced Search](#instant-search-vs-debounced-search)
4. [Highlighting Matched Characters](#highlighting-matched-characters)
5. [Launcher-Specific Patterns](#launcher-specific-patterns)
6. [Accessibility Considerations](#accessibility-considerations)
7. [Recommendations for Script Kit](#recommendations-for-script-kit)

---

## Fuzzy Search Algorithms and UX

### What is Fuzzy Search?

Fuzzy search (approximate string matching) finds strings that match a pattern approximately rather than exactly. This accounts for typos, misspellings, and partial queries.

### Key Algorithms

| Algorithm | Description | Use Case |
|-----------|-------------|----------|
| **Levenshtein Distance** | Counts minimum single-character edits (insertions, deletions, substitutions) | Typo tolerance |
| **Damerau-Levenshtein** | Adds transposition (swapping adjacent characters) | Common typing errors |
| **Jaro-Winkler** | Weights matching characters and transpositions, with prefix bonus | Name matching |
| **Smith-Waterman** | Local sequence alignment with gap penalties | Subsequence matching |
| **Sublime Text-style** | Sequential character matching with position-based scoring | Command palettes, file finders |

### Scoring Factors (fzf-style)

Based on [fzf's algorithm](https://deepwiki.com/junegunn/fzf/2.2-fuzzy-matching-algorithm):

```
Score = base_match_score + position_bonuses - gap_penalties
```

**Bonus Points:**
- **Word boundary match**: Character after `/`, `.`, `_`, `-`, or whitespace
- **CamelCase transition**: Lowercase followed by uppercase (`fileName` -> match on `N`)
- **Consecutive characters**: Each consecutive match adds bonus
- **First character match**: Matching the first character of the string
- **Exact prefix match**: Query matches the beginning of the string

**Penalties:**
- **Gap distance**: Penalty proportional to characters skipped between matches
- **Non-contiguous matches**: Spread-out matches score lower

### Example Scoring

Query: `gp` against candidates:
- `gpui` -> High score (prefix match, consecutive)
- `get_path` -> Medium score (word boundaries: `g`, `p`)
- `group_items` -> Lower score (gap between `g` and `p`)

### UX Best Practices

1. **Start matching after 1-3 characters** to avoid overwhelming results
2. **Show 5-8 autocomplete suggestions** - enough to be helpful without overwhelming
3. **Handle typos gracefully** - over 30% of searches may include typos
4. **Provide "Did you mean..." suggestions** for close matches with no exact results

**Sources:**
- [Meilisearch: Fuzzy Search Guide](https://www.meilisearch.com/blog/fuzzy-search)
- [fzf Fuzzy Matching Algorithm](https://deepwiki.com/junegunn/fzf/2.2-fuzzy-matching-algorithm)
- [Design Monks: Search UX Best Practices](https://www.designmonks.co/blog/search-ux-best-practices)

---

## Search Relevance and Ranking

### Default Ordering

Results should be ordered by relevance to the query by default. Studies show that 95% of search traffic goes to the top 10 results, with the first result getting nearly a third of all traffic.

### Relevance Factors

| Factor | Weight | Description |
|--------|--------|-------------|
| **Match quality** | High | How well the query matches (exact > fuzzy) |
| **Match position** | High | Matches at start of string/words score higher |
| **Recency/Frequency** | Medium | Recently or frequently used items |
| **String length** | Low | Shorter matching strings often more relevant |

### Displaying Relevance

**Visual Cues:**
- **Bold/highlight matched terms** in results
- **Show match context** with surrounding text
- **Indicate result type** with icons or labels
- **Group by category** for easier scanning

**Match Count:**
Display result count for context: "Showing 15 results for 'config'"

### Sorting Options

Allow users to re-sort by:
- Relevance (default)
- Alphabetical
- Most recently used
- Most frequently used

### Handling Zero Results

When no results match:
1. Suggest alternative queries
2. Show related/popular items
3. Offer to create new item if applicable
4. Explain why no results (e.g., "No scripts match 'xyz'")

**Sources:**
- [Algolia: Search UI Design Patterns](https://www.algolia.com/blog/ux/best-practices-for-site-search-ui-design-patterns)
- [UX Magazine: Designing Search Results](https://uxmag.com/articles/designing-search-results-pages)
- [Smashing Magazine: Search Results Design](https://www.smashingmagazine.com/2009/09/search-results-design-best-practices-and-design-patterns/)

---

## Instant Search vs Debounced Search

### The Tradeoff

| Approach | Pros | Cons |
|----------|------|------|
| **Instant (every keystroke)** | Immediate feedback, feels responsive | High CPU/API load, potential lag |
| **Debounced** | Reduced load, smoother UI | Slight delay before results |
| **Hybrid** | Best of both worlds | More complex to implement |

### Debouncing Explained

Debouncing delays execution until the user stops typing for a specified duration:

```
User types: s-c-r-i-p-t
Without debounce: 6 searches
With 200ms debounce: 1 search (after pause)
```

### Optimal Debounce Times

| Context | Recommended | Notes |
|---------|-------------|-------|
| **Desktop** | 150-200ms | Faster typing speed (~40 WPM) |
| **Mobile** | 250-350ms | Slower typing, tap delays |
| **API calls** | 300-500ms | Network latency tolerance |
| **Local filter** | 50-100ms | Minimal overhead |

**Key insight:** Delays over 300ms degrade perceived performance. Users expect "instant" even with small delays.

### Hybrid Approach (Recommended)

1. **Instant local filtering** for cached/in-memory data
2. **Debounced remote calls** for API/disk operations
3. **Immediate on Enter** - always search instantly when user presses Enter
4. **Cancel in-flight requests** when query changes

### Implementation Pattern

```rust
// Pseudocode for hybrid search
fn on_input_change(query: &str) {
    // Instant: filter visible items from cache
    filter_cached_results(query);

    // Debounced: fetch more results
    debounce(200ms, || {
        fetch_additional_results(query);
    });
}

fn on_enter(query: &str) {
    // Immediate: bypass debounce
    cancel_pending_debounce();
    execute_search(query);
}
```

**Sources:**
- [BytePlus: What is a Good Debounce Time?](https://www.byteplus.com/en/topic/498848)
- [Algolia: Improve Performance for InstantSearch](https://www.algolia.com/doc/guides/building-search-ui/going-further/improve-performance/vue)
- [Dev.to: Debounced Search Optimization](https://dev.to/goswamitushar/debounced-search-with-client-side-filtering-a-lightweight-optimization-for-large-lists-2mn2)

---

## Highlighting Matched Characters

### Why Highlight?

Highlighting matched characters:
- Shows users WHY a result matched
- Helps verify the result is what they wanted
- Provides visual feedback during typing
- Reduces cognitive load when scanning results

### Highlighting Strategies

#### 1. Contiguous Highlighting

Highlight the matched substring as a single block:

```
Query: "config"
Result: "Script [Config]uration"
```

#### 2. Character-by-Character Highlighting

Highlight individual matched characters (for fuzzy matching):

```
Query: "sc"
Result: "[S]cript [C]onfiguration"
```

#### 3. Word-Boundary Highlighting

Highlight matched words or word starts:

```
Query: "get path"
Result: "[Get] File [Path]"
```

### Visual Styles

| Style | CSS/Rendering | Best For |
|-------|---------------|----------|
| **Bold** | `font-weight: bold` | High contrast, accessible |
| **Background** | `background: yellow` | Very visible, traditional |
| **Underline** | `text-decoration: underline` | Subtle, doesn't change layout |
| **Color** | `color: accent` | Theme-aware, modern |
| **Combined** | Bold + subtle background | Maximum visibility |

### Implementation with Indices

Libraries like Fuse.js return match indices for highlighting:

```javascript
// Fuse.js match result
{
  item: "ScriptConfiguration",
  matches: [{
    indices: [[0, 0], [6, 6]], // 'S' and 'C'
    value: "ScriptConfiguration"
  }]
}
```

### Rendering Highlighted Text

```rust
// Pseudocode for rendering with highlights
fn render_highlighted(text: &str, indices: &[(usize, usize)]) {
    let mut pos = 0;
    for (start, end) in indices {
        // Render normal text before match
        render_normal(&text[pos..*start]);
        // Render highlighted match
        render_highlighted(&text[*start..=*end]);
        pos = end + 1;
    }
    // Render remaining text
    render_normal(&text[pos..]);
}
```

### Performance Considerations

- Pre-compute highlight indices during scoring (not in render loop)
- Cache rendered highlighted spans
- Limit highlight computation to visible items
- Use efficient string slicing (avoid allocations)

**Sources:**
- [Fuse.js Options: includeMatches](https://www.fusejs.io/api/options.html)
- [Mixpanel fuzzbunny](https://github.com/mixpanel/fuzzbunny)
- [fuzzysearch-highlight](https://github.com/uiur/fuzzysearch-highlight)
- [Algolia: Highlighting in InstantSearch](https://www.algolia.com/doc/guides/building-search-ui/ui-and-ux-patterns/highlighting-snippeting/js)

---

## Launcher-Specific Patterns

### Lessons from Raycast, Alfred, and Spotlight

These macOS launchers have refined search UX patterns over years:

#### Keyboard-First Design

- **Single hotkey activation** (Cmd+Space or custom)
- **Type immediately** - input focused on launch
- **Arrow keys for navigation** - up/down through results
- **Enter to execute** - selected action
- **Escape to dismiss** - clear and close
- **Tab for actions** - secondary action menu

#### Result List Design

| Element | Purpose |
|---------|---------|
| **Icon** | Visual identification of result type |
| **Title** | Primary text, highlighted matches |
| **Subtitle** | Secondary info (path, description) |
| **Shortcut hint** | Direct access key if available |
| **Category header** | Group similar results |

#### Action Menu Pattern (Cmd+K)

Secondary actions available via keyboard:
- Copy to clipboard
- Open in different app
- Reveal in Finder
- Edit/Configure
- Remove from history

#### Search Scope Prefixes

```
>  Command/action mode
:  Emoji picker
@  Mention/user search
#  Tag search
/  Path navigation
```

### VS Code Command Palette Insights

From [Microsoft's approach](https://github.com/Microsoft/vscode/issues/1964):

- **Stable ordering by default** - users learn positions
- **Fuzzy matching with shortcuts** - type "ssmd" for "Set Syntax: Markdown"
- **Recently used boost** - frequent commands float up
- **Prefix filtering** - `>` for commands, `@` for symbols

### Performance Expectations

| Metric | Target |
|--------|--------|
| **Time to first result** | < 50ms |
| **Full result list** | < 100ms |
| **Keystroke response** | < 16ms (60fps) |
| **Result selection** | Instant |

**Sources:**
- [Raycast vs Alfred Comparison](https://www.raycast.com/raycast-vs-alfred)
- [Reverse Engineering Sublime Text's Fuzzy Match](https://www.forrestthewoods.com/blog/reverse_engineering_sublime_texts_fuzzy_match/)
- [Designing a Command Palette](https://destiner.io/blog/post/designing-a-command-palette/)

---

## Accessibility Considerations

### WCAG Keyboard Requirements

All search functionality must be keyboard accessible (WCAG 2.1.1):

- **Tab** to reach search input
- **Arrow keys** to navigate results
- **Enter** to select/activate
- **Escape** to dismiss suggestions
- **No keyboard traps** - always able to navigate away

### ARIA for Autocomplete

Required ARIA attributes for accessible autocomplete:

```html
<input
  type="text"
  role="combobox"
  aria-autocomplete="list"
  aria-expanded="true|false"
  aria-controls="results-listbox"
  aria-activedescendant="result-3"
/>

<ul id="results-listbox" role="listbox">
  <li id="result-1" role="option">First result</li>
  <li id="result-2" role="option">Second result</li>
  <li id="result-3" role="option" aria-selected="true">Third result</li>
</ul>
```

### Key Attributes

| Attribute | Purpose |
|-----------|---------|
| `role="combobox"` | Identifies as combo box to screen readers |
| `aria-autocomplete="list"` | Announces that suggestions will appear |
| `aria-expanded` | Indicates if suggestion list is open |
| `aria-controls` | Links input to suggestion list |
| `aria-activedescendant` | Tracks keyboard-focused suggestion |
| `role="listbox"` | Container for suggestions |
| `role="option"` | Individual suggestions |
| `aria-selected` | Currently selected option |

### Live Region Announcements

Announce suggestion count for screen reader users:

```html
<div aria-live="polite" aria-atomic="true">
  5 suggestions available
</div>
```

### Focus Management

- **Keep DOM focus on input** while navigating suggestions
- **Use aria-activedescendant** to indicate visual focus
- **Return focus to input** after selection
- **Visible focus indicator** on suggestions (2px+ outline)

### Screen Reader Testing

Test with:
- VoiceOver (macOS)
- NVDA (Windows)
- JAWS (Windows)

Known issues:
- NVDA may not announce character deletions in combobox
- Clear `aria-activedescendant` on text changes for consistent announcements

**Sources:**
- [W3C WAI: Combobox Autocomplete Example](https://www.w3.org/WAI/ARIA/apg/patterns/combobox/examples/combobox-autocomplete-list/)
- [Harvard: Autocomplete Accessibility](https://accessibility.huit.harvard.edu/technique-aria-autocomplete)
- [MDN: aria-autocomplete](https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Attributes/aria-autocomplete)
- [React Aria: Building a Combobox](https://react-spectrum.adobe.com/blog/building-a-combobox.html)

---

## Recommendations for Script Kit

Based on this research, here are specific recommendations for Script Kit's search implementation:

### 1. Fuzzy Matching Algorithm

**Recommendation:** Implement Sublime Text-style sequential matching with scoring bonuses.

```rust
struct FuzzyMatch {
    score: i32,
    indices: Vec<usize>,  // For highlighting
}

fn fuzzy_match(query: &str, target: &str) -> Option<FuzzyMatch> {
    // Score bonuses (example values)
    const BONUS_CONSECUTIVE: i32 = 15;
    const BONUS_WORD_BOUNDARY: i32 = 25;
    const BONUS_CAMEL_CASE: i32 = 25;
    const BONUS_FIRST_CHAR: i32 = 15;
    const PENALTY_GAP: i32 = 3;

    // Implementation...
}
```

### 2. Search Timing

**Recommendation:** Hybrid instant + debounced approach.

| Data Source | Strategy |
|-------------|----------|
| In-memory scripts | Instant (every keystroke) |
| File system | Debounced (150ms) |
| Remote/API | Debounced (300ms) |

### 3. Result Display

**Recommendation:** Rich result items with highlighting.

```
[Icon] Script Name (highlighted matches)
       /path/to/script.ts | Tag1, Tag2
       [Shortcut: Cmd+1]
```

### 4. Keyboard Navigation

**Recommendation:** Full keyboard support with these bindings:

| Key | Action |
|-----|--------|
| `Up/Down` | Navigate results |
| `Enter` | Execute selected |
| `Tab` | Open actions menu |
| `Cmd+1-9` | Quick select by position |
| `Escape` | Clear search / Close |
| `Cmd+Backspace` | Clear search input |

### 5. Scoring Priorities

For Script Kit's use case, prioritize:

1. **Exact name matches** - highest priority
2. **Recent usage** - boost recently run scripts
3. **Frequency** - boost frequently used scripts
4. **Word boundary matches** - reward matching word starts
5. **Consecutive matches** - reward sequential character matches
6. **Description matches** - lower priority than name

### 6. Highlighting Implementation

**Recommendation:** Character-by-character highlighting with theming.

```rust
// Return match indices from fuzzy match
fn render_script_name(name: &str, match_indices: &[usize], theme: &Theme) {
    for (i, char) in name.chars().enumerate() {
        if match_indices.contains(&i) {
            // Use theme accent color, bold
            render_highlighted_char(char, theme.colors.accent);
        } else {
            render_normal_char(char, theme.colors.text);
        }
    }
}
```

### 7. Empty State

**Recommendation:** Helpful empty states.

When no results:
- "No scripts match '[query]'"
- "Create a new script?" (with action)
- Show recent scripts as fallback

### 8. Performance Targets

| Metric | Target |
|--------|--------|
| First result visible | < 16ms (one frame) |
| Full list filtered | < 50ms |
| Highlight rendering | < 5ms |

### 9. Future Enhancements

Consider for later iterations:
- **Search history** - recent searches
- **Saved searches** - bookmark common queries
- **Search operators** - `tag:utility`, `recent:7d`
- **Fuzzy path matching** - navigate by path fragments

---

## Summary

Effective search UX in launchers combines:

1. **Fast fuzzy matching** with intelligent scoring
2. **Instant feedback** for local data, debounced for remote
3. **Clear highlighting** of matched characters
4. **Full keyboard navigation** with logical shortcuts
5. **Accessible implementation** with proper ARIA
6. **Graceful handling** of empty/error states

Script Kit should leverage these patterns to provide a search experience that feels as fast and intuitive as Raycast or VS Code's command palette, while maintaining the flexibility Script Kit users expect.
