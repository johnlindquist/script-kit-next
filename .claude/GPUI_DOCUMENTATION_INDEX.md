# GPUI Framework Documentation Index

**Research Completed**: March 1, 2026
**GPUI Target Revision**: `03416097` (Zed editor)
**Total Lines**: 1,972 lines across 4 documents

---

## Quick Navigation

### I Need to Know ASAP (5 min read)
→ **[GPUI_QUICK_REFERENCE.md](GPUI_QUICK_REFERENCE.md)**
- Top 10 gotchas
- Copy-paste code patterns
- "How do I fix X?" answers

### I'm Implementing Code (30 min reference)
→ **[GPUI_GOTCHAS_AND_PATTERNS.md](GPUI_GOTCHAS_AND_PATTERNS.md)**
- Complete pattern reference
- Why each pattern exists
- All context types explained
- Lifecycle details
- Event system
- Focus system
- Memory management
- 12 comprehensive sections

### I Want to Understand GPUI (20 min)
→ **[GPUI_RESEARCH_SUMMARY.md](GPUI_RESEARCH_SUMMARY.md)**
- Key findings overview
- Patterns that work well
- AI agent checklist
- Unique GPUI characteristics
- Performance considerations
- Common anti-patterns

### I Want to Verify & Dig Deeper (reference)
→ **[GPUI_RESEARCH_SOURCES.md](GPUI_RESEARCH_SOURCES.md)**
- All sources listed with URLs
- Knowledge coverage map
- Gaps and limitations
- Research methodology
- Recommended reading order

---

## Document Overview

| Document | Size | Lines | Purpose |
|----------|------|-------|---------|
| [GPUI_QUICK_REFERENCE.md](GPUI_QUICK_REFERENCE.md) | 6.2 KB | 264 | 10 critical gotchas + bonus |
| [GPUI_GOTCHAS_AND_PATTERNS.md](GPUI_GOTCHAS_AND_PATTERNS.md) | 30 KB | 1,090 | Comprehensive reference (12 topics) |
| [GPUI_RESEARCH_SUMMARY.md](GPUI_RESEARCH_SUMMARY.md) | 11 KB | 331 | Overview, findings, checklists |
| [GPUI_RESEARCH_SOURCES.md](GPUI_RESEARCH_SOURCES.md) | 9.2 KB | 287 | Sources, methodology, gaps |

---

## Topics Covered

### 1. Entity Lifecycle & Memory Management
**Files**: GOTCHAS_AND_PATTERNS (section 1), RESEARCH_SUMMARY (section: Inverted Ownership Model)

- App owns all entities (inverted model)
- Entity<T> (strong references)
- WeakEntity<T> (weak references)
- Reference counting and cleanup
- Subscription lifetime management
- Effect queuing and reentrancy prevention

### 2. Render Trait Contract
**Files**: GOTCHAS_AND_PATTERNS (section 2), QUICK_REFERENCE (gotcha #4)

- What can/cannot be done in render()
- Three-phase render cycle (Prepaint, Paint, GPU)
- Synchronous-only constraint
- Only self mutations allowed
- No blocking operations

### 3. Context Types
**Files**: GOTCHAS_AND_PATTERNS (section 3), QUICK_REFERENCE (gotcha #5)

- App context
- Context<T>
- AsyncApp
- AsyncWindowContext
- Fallible async operations
- Borrow rules

### 4. Asynchronous Execution (cx.spawn)
**Files**: GOTCHAS_AND_PATTERNS (section 4), QUICK_REFERENCE (gotcha #3)

- Modern spawn signature (03416097)
- WeakEntity<T> closure parameter
- AsyncWindowContext access
- Task lifecycle (.detach() vs storage)
- Fallible operations in async context
- Nested cx usage

### 5. Focus System & Keyboard Events
**Files**: GOTCHAS_AND_PATTERNS (section 5), QUICK_REFERENCE (gotcha #6)

- Complete focus chain (4 steps)
- FocusHandle creation and lifecycle
- Focusable trait implementation
- track_focus() connection
- on_action() registration
- Event dispatch phases (capture + bubble)
- Action definitions (semantic vs raw keys)

### 6. Subscriptions & Observers
**Files**: GOTCHAS_AND_PATTERNS (section 6), QUICK_REFERENCE (gotcha #1)

- cx.observe() pattern
- cx.subscribe() + EventEmitter pattern
- Subscription storage requirement
- Subscription::drop() behavior
- Reentrancy safety via effect queuing

### 7. Element Traits: RenderOnce vs Render
**Files**: GOTCHAS_AND_PATTERNS (section 7), QUICK_REFERENCE (gotcha #7)

- RenderOnce: stateless, ownership (consumes self)
- Render: stateful, borrowing (&mut self)
- When to use each
- Component reusability patterns

### 8. The Critical cx.notify() Requirement
**Files**: GOTCHAS_AND_PATTERNS (section 8), QUICK_REFERENCE (gotcha #2), RESEARCH_SUMMARY (No Automatic Reactivity)

- No automatic change detection
- Performance rationale
- When to call cx.notify()
- Symptoms of missing cx.notify()
- Not callable during render

### 9. Overlays & Positioning
**Files**: GOTCHAS_AND_PATTERNS (section 9), RESEARCH_SUMMARY (Patterns That Work Well)

- Anchored overlays using absolute positioning
- No built-in popup primitive
- Deferred opening via cx.spawn()
- Click handlers for dismissal
- Safe window management

### 10. Scroll Management
**Files**: GOTCHAS_AND_PATTERNS (section 10)

- ListState for variable-height lists
- UniformListScrollHandle for fixed-height lists
- Scroll activity patterns
- Fade animations

### 11. Common Gotchas & Traps
**Files**: GOTCHAS_AND_PATTERNS (section 11), QUICK_REFERENCE (all 10 gotchas + bonus)

- Entity "already being updated" panic
- Weak reference upgrade() safety
- Subscription not stored (callback never fires)
- Task spawn without .detach()
- render() calling cx.spawn()
- Multiple borrow conflicts
- State mutation during render
- Observing dropped entities

### 12. Event Dispatch System
**Files**: GOTCHAS_AND_PATTERNS (section 12), QUICK_REFERENCE (bonus: Action Binding)

- Keyboard-first design
- Actions vs raw key events
- Action registration and dispatch
- Context scoping
- Complex action serialization
- Keymaps (JSON binding)

---

## Use Cases

### Case 1: "I'm writing a button component"
1. Read: QUICK_REFERENCE gotcha #7 (RenderOnce vs Render)
2. Read: GOTCHAS_AND_PATTERNS section 7
3. Choose: RenderOnce (if stateless) or Render (if stateful)
4. Implement: Using example patterns

### Case 2: "My callback never fires"
1. Read: QUICK_REFERENCE gotcha #1 (Subscriptions must be stored)
2. Read: GOTCHAS_AND_PATTERNS section 6 (Subscriptions & Observers)
3. Check: Is subscription stored as struct field?
4. Verify: Subscription is pushed to Vec during initialization

### Case 3: "Keyboard shortcuts don't work"
1. Read: QUICK_REFERENCE gotcha #6 (Focus chain required)
2. Read: GOTCHAS_AND_PATTERNS section 5 (Focus System)
3. Check: All 4 steps of focus chain implemented
4. Verify: Action bound in keymap.json

### Case 4: "UI doesn't update after state change"
1. Read: QUICK_REFERENCE gotcha #2 (cx.notify() required)
2. Read: GOTCHAS_AND_PATTERNS section 8 (cx.notify())
3. Check: cx.notify() called in event handler
4. Debug: Verify event handler is actually being called

### Case 5: "Async task is cancelled before completing"
1. Read: QUICK_REFERENCE gotcha #3 (Tasks must detach or store)
2. Read: GOTCHAS_AND_PATTERNS section 4 (cx.spawn)
3. Fix: Add .detach() or store task in struct field

### Case 6: "Borrow conflict in listener"
1. Read: QUICK_REFERENCE gotcha #5 (Multiple borrow conflicts)
2. Read: GOTCHAS_AND_PATTERNS section 4 (Nested cx)
3. Fix: Use listener's cx parameter, not outer cx

### Case 7: "I want a reusable component"
1. Read: QUICK_REFERENCE gotcha #7 (RenderOnce vs Render)
2. Read: GOTCHAS_AND_PATTERNS section 7 (Element Traits)
3. Choose: RenderOnce for lightweight, stateless
4. Pattern: Use `#[derive(IntoElement)]` + RenderOnce

---

## Key Takeaways

### The Five Most Critical Patterns

1. **Subscriptions are resources** (gotcha #1)
   - Storage: `subscriptions: Vec<Subscription>`
   - Dropped subscription = unregistered callback
   - Pattern: Push to vec immediately after creation

2. **Explicit reactivity** (gotcha #2)
   - No automatic change detection
   - Must call `cx.notify()` after mutations
   - Event handler → mutation → `cx.notify()` → UI re-renders

3. **Task lifecycle** (gotcha #3)
   - Tasks are cancelled when dropped
   - Must either `.detach()` or store
   - Pattern: `cx.spawn(async { ... }).detach()`

4. **Render is synchronous** (gotcha #4)
   - Cannot call `cx.spawn()` in render()
   - Cannot do blocking operations
   - Cannot mutate other entities
   - Move async work to event handlers

5. **Complete focus chain** (gotcha #6)
   - Missing one step = no keyboard
   - Four steps: Handle → Trait → track_focus() → on_action()
   - All or nothing (no partial credit)

### The Three Biggest Surprises

1. **App owns entities** (not you)
   - `Entity<T>` is a handle, not ownership
   - Enables safe observation and async
   - WeakEntity<T> for cycles

2. **No automatic reactivity**
   - Unlike React, Compose, Flutter
   - Performance tradeoff (no dirty tracking)
   - Explicit `cx.notify()` requirement

3. **Actions replace raw key events**
   - Semantic intent, not keycodes
   - Requires keymap binding
   - Enables easy rebinding

---

## Recommended Reading Sequence

### For Quick Problem Solving (5 min)
1. QUICK_REFERENCE.md
2. Use Ctrl+F to find your problem
3. Read the example and fix code

### For Learning GPUI (60 min)
1. QUICK_REFERENCE.md (10 min) - Get overview
2. RESEARCH_SUMMARY.md (15 min) - Understand why
3. GOTCHAS_AND_PATTERNS.md (30 min) - Deep dive
4. RESEARCH_SOURCES.md (5 min) - Verify sources

### For Reference During Implementation (ongoing)
1. Keep QUICK_REFERENCE.md visible
2. Use GOTCHAS_AND_PATTERNS.md Ctrl+F for details
3. Reference RESEARCH_SOURCES.md for original sources

### For Architecture/Code Review (30 min)
1. RESEARCH_SUMMARY.md - Big picture
2. GOTCHAS_AND_PATTERNS.md - Verify patterns
3. QUICK_REFERENCE.md - Check gotchas

---

## File Locations

All documents are in the `.claude` directory of the Script Kit GPUI repository:

```
/Users/johnlindquist/dev/script-kit-gpui/.claude/
├── GPUI_DOCUMENTATION_INDEX.md          (this file)
├── GPUI_QUICK_REFERENCE.md              (5 min read)
├── GPUI_GOTCHAS_AND_PATTERNS.md         (30 min reference)
├── GPUI_RESEARCH_SUMMARY.md             (20 min overview)
└── GPUI_RESEARCH_SOURCES.md             (sources + methodology)
```

---

## Version Information

| Component | Version |
|-----------|---------|
| GPUI Revision | `03416097` |
| Zed Branch | main |
| Research Date | March 1, 2026 |
| Documentation Version | 1.0 |

---

## Feedback & Updates

If you find:
- **Inaccuracies**: Check against source, update with citation
- **Missing patterns**: Add to GOTCHAS_AND_PATTERNS with example
- **New gotchas**: Add to QUICK_REFERENCE with solution
- **Source changes**: Update RESEARCH_SOURCES with new URL

---

## License & Attribution

Research based on:
- Official Zed GPUI documentation (Apache 2.0)
- Zed engineering blog posts
- Community GPUI guides
- Script Kit codebase (your project)

Use freely within your organization.

---

**Happy GPUI coding!** 🚀

