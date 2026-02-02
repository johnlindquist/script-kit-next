# Performance Perception and Perceived Speed in Launcher Apps

Research findings on techniques for optimizing perceived performance in keyboard-driven launcher applications like Script Kit, Raycast, Alfred, and Spotlight.

---

## Table of Contents

1. [Response Time Thresholds](#response-time-thresholds)
2. [Instant Feedback Techniques](#instant-feedback-techniques)
3. [Progressive Loading Patterns](#progressive-loading-patterns)
4. [Optimistic UI Updates](#optimistic-ui-updates)
5. [When to Show Spinners vs. Alternatives](#when-to-show-spinners-vs-alternatives)
6. [Launcher-Specific Patterns](#launcher-specific-patterns)
7. [Recommendations for Script Kit](#recommendations-for-script-kit)

---

## Response Time Thresholds

Foundational research from Jakob Nielsen (building on Robert B. Miller's 1968 work) established three critical response time thresholds that remain relevant today:

### The Three Classic Thresholds

| Threshold | User Perception | Design Implication |
|-----------|-----------------|-------------------|
| **0.1 seconds (100ms)** | Instantaneous / direct manipulation | User feels their action directly caused the result |
| **1 second** | Noticeable delay but focused | User senses delay but maintains flow |
| **10 seconds** | Flow breaks | Frustration sets in; users may abandon task |

### The Doherty Threshold

In 1982, IBM research established the **400ms threshold** as the target for computer response time (down from the previous 2-second standard). Keeping responses under 400ms creates a sense of system responsiveness that keeps users engaged.

### Finer-Grained Perception

- **50ms**: Users can distinguish visual appeal differences
- **16ms**: Frame budget for 60 FPS smooth animation (dropped frames create perceived jank)
- **50ms**: Haptic feedback threshold - beyond this, feedback feels like an "echo" rather than immediate

### Business Impact

- 100ms delay in load time can reduce conversion rates by 7%
- 2-second delay can increase bounce rates by 103%

**Sources:**
- [Response Time Limits - Nielsen Norman Group](https://www.nngroup.com/articles/response-times-3-important-limits/)
- [Doherty Threshold - Laws of UX](https://lawsofux.com/doherty-threshold/)
- [Time Scales of UX - Jakob Nielsen](https://jakobnielsenphd.substack.com/p/time-scale-ux)

---

## Instant Feedback Techniques

### Micro-interactions

Small, purposeful animations that provide immediate visual feedback on user actions:

- **Ripple effects** on button clicks
- **Sliding panels** for navigation
- **Celebratory animations** for task completion (Duolingo style - shown to boost retention by 15%)
- **Haptic vibrations** for confirmation (Apple Pay style)

### Keyboard-First Design Patterns

For launcher apps specifically:

- **Instant text echo**: Characters appear immediately as typed
- **Real-time filtering**: Results update with each keystroke
- **Highlighted matches**: Show which characters matched the query
- **Selection state changes**: Immediate visual feedback on arrow key navigation

### Animation Best Practices

- Well-timed animations create a sense of speed
- Subtle animations (ripple effects, sliding panels) make interfaces feel more responsive
- **Avoid over-animation**: Some users report that excessive animations introduce perceived delay
- Purpose over polish: Prioritize animations that support task completion over decorative ones

**Sources:**
- [The Role of Micro-interactions in Modern UX - IxDF](https://www.interaction-design.org/literature/article/micro-interactions-ux)
- [UI/UX Design Trends 2025 - Medium](https://medium.com/cygnis-media/top-ui-ux-trends-of-2025-reshaping-user-experiences-in-custom-web-apps-9746cd33aae7)

---

## Progressive Loading Patterns

### Skeleton Screens

Placeholder UI that mimics the final layout structure while content loads:

**Benefits:**
- Users perceive sites with skeleton screens as **30% faster** than identical sites with spinners
- Reduces cognitive load during wait times
- Progressive rendering keeps user attention
- Left-to-right loading motions feel faster

**Best Practices:**
- Skeleton must accurately represent final UI layout
- Add shimmer/wave animation to draw attention away from wait
- Use slow, steady loading motion (perceived as faster than abrupt changes)

**Who Uses Them:**
- LinkedIn, YouTube, Facebook, Amazon, Medium

### Progressive Image Loading

- Show predominant color first
- Display blurred/low-res version
- Fade in full resolution

### When NOT to Use Skeleton Screens

- Server-side rendered pages (content already in HTML)
- Waits under 200ms (no indicator needed)

**Sources:**
- [Skeleton Loading Screen Design - LogRocket](https://blog.logrocket.com/ux-design/skeleton-loading-screen-design/)
- [Skeleton Screens vs. Spinners - UI Deploy](https://ui-deploy.com/blog/skeleton-screens-vs-spinners-optimizing-perceived-performance)
- [How to Speed Up Your UX with Skeleton Screens - SitePoint](https://www.sitepoint.com/how-to-speed-up-your-ux-with-skeleton-screens/)

---

## Optimistic UI Updates

### What Is Optimistic UI?

Update the UI immediately assuming the operation will succeed, then roll back if it fails. This creates perceived instant response while the actual operation happens in the background.

### When to Use Optimistic Updates

**Ideal scenarios:**
- Binary actions: Like/Unlike, Star/Unstar, Save/Unsave
- High success rate operations (near 100%)
- Actions not tied to other parts of the interface
- Fast API response times
- Low-consequence failures (easy to undo)

**Examples:**
- Toggling favorites
- Marking items as read/unread
- Adding items to lists
- Preference updates

### When NOT to Use Optimistic Updates

- Complex server-side validations required
- Multiple users editing same data
- Irreversible consequences
- High failure rate operations
- Financial transactions

### Best Practices

1. **Update UI immediately** on user action
2. **Always prepare for rollback**: Maintain snapshot of previous state
3. **Keep APIs idempotent**: Multiple identical requests should have same effect
4. **Re-sync state after mutations**: Use query invalidation, polling, or fresh GETs
5. **Implement retry logic**: Backend must tolerate duplicate requests

### Common Pitfalls

- Never omit rollback logic
- Trigger API requests immediately with UI update (don't defer)
- Don't use optimistic updates to hide bad UX
- Be careful with multi-user scenarios where state can diverge

**Sources:**
- [Optimistic UI in Frontend Architecture - Medium](https://javascript.plainenglish.io/optimistic-ui-in-frontend-architecture-do-it-right-avoid-pitfalls-7507d713c19c)
- [When to Use Optimistic Updates - Dimitrios Lytras](https://dnlytras.com/blog/optimistic-updates)
- [Understanding Optimistic UI - LogRocket](https://blog.logrocket.com/understanding-optimistic-ui-react-useoptimistic-hook/)

---

## When to Show Spinners vs. Alternatives

### Time-Based Guidelines

| Wait Time | Recommended Approach |
|-----------|---------------------|
| **< 200-300ms** | No indicator needed |
| **200ms - 1s** | Subtle transition/animation |
| **1-4 seconds** | Spinner acceptable |
| **4-10 seconds** | Progress bar or skeleton screen |
| **10+ seconds** | Determinate progress with time estimate |

### Why Spinners Can Be Problematic

- For waits under 1 second: Spinners flash distractingly
- For waits over 4 seconds: Users grow impatient without progress indication
- For waits over 10 seconds: Indefinite spinners make users unsure if system is working

### The Delayed Spinner Pattern

Show spinners only after a threshold (typically 300ms) has elapsed:

```
if (wait_time > 300ms) {
    show_spinner()
}
```

This prevents spinner "flash" for quick operations while still indicating longer waits.

### Alternatives to Spinners

1. **Skeleton screens**: Show placeholder UI structure
2. **Progress bars**: For determinate operations
3. **Inline loading states**: Button changes to "Loading..." or shows small spinner
4. **Optimistic updates**: Show result immediately, sync in background
5. **Background processing**: Let user continue while operation completes

### Spinner Stall Threshold

If using debounced search with spinners, set stall threshold to:
```
debounce_delay + 300ms
```

This prevents spinners from appearing immediately after final keystroke.

**Sources:**
- [Progress Indicators Make a Slow System Less Insufferable - NN/G](https://www.nngroup.com/articles/progress-indicators/)
- [Progress Bars vs. Spinners - UX Movement](https://uxmovement.com/navigation/progress-bars-vs-spinners-when-to-use-which/)
- [Your Loading Spinner Is a UX Killer - Boldist](https://boldist.co/usability/loading-spinner-ux-killer/)

---

## Launcher-Specific Patterns

### Performance Benchmarks from Leading Launchers

| Launcher | Startup Time | CPU (Standby) | Notes |
|----------|--------------|---------------|-------|
| **Monarch** | ~0.3s | Low | Minimal features |
| **Raycast** | ~0.5s | 1-2% | May increase with extensions |
| **Alfred** | Fastest | 1-2% | "Speed is Alfred's identity" |

### What Makes Launchers Feel Fast

1. **Instant hotkey response**: No perceptible delay between keystroke and window appearance
2. **Fuzzy search intelligence**: Type "ps" and get "Photoshop"
3. **Learning user habits**: Predict most-used items
4. **Real-time filtering**: Results update as you type
5. **Keyboard-first navigation**: Arrow keys, Enter, Escape all instant

### Common Speed Complaints

- Added animations can introduce perceptible delay
- Extension loading can slow startup
- File search slower than in-memory search
- Noticeable lag between typing and results appearing

### Command Palette Best Practices

From Superhuman's engineering team:

- **Single entry point**: Command palette is THE place for all commands
- **Instant response**: Suggestions offered on every keypress (~100ms)
- **Keyboard shortcuts discoverable**: Users unlock extreme speed
- **Simple mental model**: Everything in one place

### Debounce Timing for Search

**Optimal debounce: 150-200ms**

- Matches typical typing speed (3-5 characters/second)
- Delays over 300ms degrade perceived responsiveness
- Faster typists may need shorter debounce

**Cache frequently used queries** to eliminate network latency for repeat searches.

**Sources:**
- [Alfred vs Raycast - Josh Collinsworth](https://joshcollinsworth.com/blog/alfred-raycast)
- [How to Build a Remarkable Command Palette - Superhuman](https://blog.superhuman.com/how-to-build-a-remarkable-command-palette/)
- [Command Palette UX Patterns - Medium](https://medium.com/design-bootcamp/command-palette-ux-patterns-1-d6b6e68f30c1)

---

## Recommendations for Script Kit

Based on this research, here are specific recommendations for Script Kit's perceived performance:

### 1. Window Appearance (< 100ms target)

- **Pre-warm the window** in background on system startup
- **Keep window hidden but initialized** for instant show
- **Minimize cold-start work**: Defer extension loading until after window appears
- **AOT compilation** where possible for faster initialization

### 2. Input Responsiveness (< 50ms target)

- **Immediate text echo**: Characters appear instantly as typed
- **No debounce on display**: Always show what user typed immediately
- **Debounce only the search operation**: 150-200ms for remote queries
- **Local search should be instant**: Filter in-memory data synchronously

### 3. Results Display

- **Use skeleton screens** for items loading from disk/network
- **Progressive loading**: Show items as they become available
- **Prioritize visible items**: Render viewport first, lazy-load rest
- **Cache recent results**: Instant recall for repeated queries

### 4. Feedback Patterns

| Action | Feedback | Timing |
|--------|----------|--------|
| Keystroke | Character appears | < 16ms |
| Arrow navigation | Selection highlight moves | < 50ms |
| Enter pressed | Item highlight / ripple | < 100ms |
| Action executing | Inline state change | < 100ms |
| Long operation | Delayed spinner (300ms) | Only if > 300ms |

### 5. Optimistic UI Opportunities

Apply optimistic updates for:
- Toggling script favorites
- Recent items list updates
- Theme/preference changes
- Script enable/disable

### 6. Avoid These Anti-Patterns

- **Don't show spinners for sub-300ms operations**
- **Don't use animations that delay content appearance**
- **Don't debounce local/cached searches**
- **Don't block UI thread during search**
- **Don't flash empty states** before results load

### 7. Perceived Speed Techniques

1. **Pre-fetch likely next actions**: If user hovers on item, pre-load its details
2. **Predict common queries**: Pre-cache results for frequent searches
3. **Show partial results immediately**: Don't wait for complete result set
4. **Animate in a way that feels fast**: Left-to-right, steady motion
5. **Use progress indication wisely**: Only for operations > 1 second

### 8. Performance Monitoring

Track these metrics:
- Time from hotkey to window visible
- Time from keystroke to first result
- Time from Enter to action completion
- Frame drops during scrolling
- Extension load times

### Implementation Priority

1. **High Impact, Low Effort**
   - Add delayed spinner pattern (300ms threshold)
   - Debounce search queries (150-200ms)
   - Cache recent search results

2. **High Impact, Medium Effort**
   - Pre-warm window on startup
   - Skeleton screens for async items
   - Progressive result loading

3. **High Impact, Higher Effort**
   - Predictive caching
   - Learning user patterns
   - Optimistic UI for all toggles

---

## Summary

The key insight from this research is that **perceived performance is often more important than actual performance**. Users judge speed based on:

1. **Immediate feedback**: Does the UI respond to my input instantly?
2. **Progress indication**: Do I know something is happening?
3. **Predictable timing**: Does the system meet my expectations?

For a launcher app like Script Kit, the critical path is:

```
Hotkey press -> Window appears -> User types -> Results appear -> User selects -> Action executes
```

Every step in this path should feel instantaneous (< 100ms) or show appropriate feedback for longer operations. The goal is to create a feeling of **direct manipulation** where the user feels in complete control.

---

## References

### Foundational UX Research
- [Response Time Limits - Nielsen Norman Group](https://www.nngroup.com/articles/response-times-3-important-limits/)
- [Powers of 10: Time Scales in UX - NN/G](https://www.nngroup.com/articles/powers-of-10-time-scales-in-ux/)
- [Doherty Threshold - Laws of UX](https://lawsofux.com/doherty-threshold/)

### Loading Patterns
- [Skeleton Loading Screen Design - LogRocket](https://blog.logrocket.com/ux-design/skeleton-loading-screen-design/)
- [UX Design Patterns for Loading - Pencil & Paper](https://www.pencilandpaper.io/articles/ux-pattern-analysis-loading-feedback)
- [Progress Indicators - NN/G](https://www.nngroup.com/articles/progress-indicators/)

### Optimistic UI
- [Optimistic UI in Frontend Architecture - Medium](https://javascript.plainenglish.io/optimistic-ui-in-frontend-architecture-do-it-right-avoid-pitfalls-7507d713c19c)
- [When to Use Optimistic Updates - Dimitrios Lytras](https://dnlytras.com/blog/optimistic-updates)

### Launcher Apps
- [Alfred vs Raycast - Josh Collinsworth](https://joshcollinsworth.com/blog/alfred-raycast)
- [How to Build a Remarkable Command Palette - Superhuman](https://blog.superhuman.com/how-to-build-a-remarkable-command-palette/)
- [Command Palette UX Patterns - Medium](https://medium.com/design-bootcamp/command-palette-ux-patterns-1-d6b6e68f30c1)

### Search UX
- [How to Optimize Typeahead Search - Medium](https://medium.com/geekculture/how-to-optimize-typeahead-search-in-your-web-application-8246cac5b05f)
- [What is a Good Debounce Time - BytePlus](https://www.byteplus.com/en/topic/498848)
