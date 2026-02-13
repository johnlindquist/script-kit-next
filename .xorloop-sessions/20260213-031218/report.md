# xorloop Report — 20260213-031218

**Project:** script-kit-gpui
**Branch:** main
**Started:** Fri Feb 13 03:12:18 MST 2026

---

## Iteration 1 — performance (03:32)

**Feature:** FEATURE: The menu_bar module uses macOS Accessibility APIs (AXUIElement FFI) to recursively scan, parse, and cache application menu bar hierarchies — exposing menu items with titles, enabled state, keyboard shortcuts, and element paths for programmatic menu execution.

- **Critical: Excessive FFI allocations in hot paths** — Every AX attribute read creates throwaway CString/CFString objects; constant attribute names are never cached; and multiple separate cross-process AX calls per menu item should be batched with `AXUIElementCopyMultipleAttributeValues`.

- **Major: Cloning, locking, and concurrency waste** — Full menu tree deep-cloned on every read (use `Arc`); SQLite mutex held during serde; rapid app-switching spawns unbounded OS threads for AX scans (need debounce + single worker); linear string-allocating scans for menu item lookup.

- **Minor: Small allocation and timing inefficiencies** — Hard-coded 50ms sleeps instead of bounded polling; missing `Vec::with_capacity` hints; unconditional logging and redundant attribute fetches in separator detection that compound after the critical fixes land.


---

## Iteration 2 — documentation gaps (04:04)

**Feature:** FEATURE: The emoji picker implements keyboard-navigable grid selection across multi-category emoji groups, using a computed row mapper that accounts for category header offsets to keep the active cell scrolled into view.

- The emoji picker's core navigation/scroll logic relies on undocumented invariants (category contiguity, flat-index model, row mapping) shared between `compute_scroll_row` and the grid renderer — the highest-priority gap.
- Public types (`Emoji`, `EmojiCategory`, `ALL_CATEGORIES`, `EMOJIS`) and helpers (`search_emojis`, `grouped_emojis`) lack rustdoc, module-level docs, and ordering/search-semantics documentation.
- The render function (`render_emoji_picker`) has no section comments or module docs despite being ~500 lines, and there's no standalone architecture doc for the emoji subsystem.


---

## Iteration 3 — performance (04:33)

**Feature:** FEATURE: The `app_navigation` module handles keyboard and scroll-based navigation of the script list, including up/down movement, page navigation, scroll-to-reveal, and smart section-header skipping.

- **Unbounded timer spawning:** Every arrow key press spawns a new detached 1s timer task with no dedup guard, creating a "thundering herd" of tasks during rapid input — fix with a single debounced fade task pattern.

- **Redundant work per keystroke:** Double `cx.notify()` calls per navigation event cause extra render invalidations, and repeated linear scans over `grouped_items` (`.position()` + `.rposition()`) compound to O(n²) during sustained key holds.

- **Missing fast-path guards:** Cursor-hide platform calls fire unconditionally every keystroke even when already in keyboard mode, and `validate_selection_bounds()` does multiple full passes plus eager cache clearing regardless of whether selection actually changed.


---

## Iteration 4 — documentation gaps (05:13)

**Feature:** FEATURE: Text expansion trigger system that buffers typed characters in a rolling window and fires scriptlet expansions when registered keyword patterns (like `:sig`, `!today`, `addr,,`) are completed

- **Critical gaps**: Architecture docs in `keyword_manager` describe outdated stop/restart behavior; keystroke logger captures passwords with no privacy warning; keyword expansion silently falls back on unknown tool types without documentation.

- **Major gaps**: Timing constants reference nonexistent logic, keystroke filtering/variable substitution/matching semantics are undocumented, `scriptlet_path` field is misleading (accepts synthetic IDs), and no authoring guide exists for keyword triggers.

- **Minor gaps**: Stale inline comments in scriptlet parsing don't match actual checks, and non-macOS stub functions lack rustdoc explaining their no-op behavior.


---

## Iteration 5 — performance (05:51)

**Feature:** FEATURE: Window tiling and management system using macOS Accessibility APIs (AXUIElement) to list, move, resize, tile (halves/quadrants/thirds/sixths), minimize, maximize, close, and focus windows across applications and displays.

- **Full AX crawl on every cache miss**: Each window operation triggers a complete enumeration of all apps and windows on miss, with cache-clearing causing thrash — the single biggest bottleneck at O(apps × windows) per operation.
- **Redundant per-call allocations**: CFString attribute names are re-created for every AX call, window titles allocate fresh buffers in hot loops, and the cache locks/unlocks per insertion instead of batching — all adding unnecessary overhead.
- **Synchronous AX on the UI thread**: All accessibility calls are cross-process and blocking; if invoked from GPUI action handlers they stall rendering, and display bounds are also re-queried on every operation instead of being cached.


---

## Iteration 6 — security audit (06:48)

**Feature:** FEATURE: Emoji picker built-in with categorized grid navigation, keyword search filtering, horizontal/vertical arrow-key movement, category tabs, and clipboard insertion of selected emoji.

- **Unbounded search input (MAJOR):** Pasting multi-MB text into the emoji filter triggers expensive string clones on the UI thread, causing a DoS. Fix by clamping input to 512 bytes (UTF-8-safe) before processing.
- **Minor defensive gaps (3 findings):** No bidi/control char validation on clipboard output, emoji picker unconditionally registered (can't be disabled via config), and a fragile slice index that could panic if invariants break.
- **Clean baseline:** No `unsafe` blocks, no deserialization or file handling, and no secret leaks found in the emoji picker path — the codebase is structurally sound here.


---

## Iteration 7 — documentation gaps (09:20)

**Feature:** FEATURE: The emoji picker is a built-in command that renders a searchable, category-grouped 8-column grid of emojis with arrow-key navigation, mouse hover selection, and Enter-to-copy-to-clipboard functionality.

- GPT-5.2 Pro found 16 documentation gaps in the emoji picker and related modules: 7 major (missing rustdoc on public structs/enums/functions, undocumented invariants in `compute_scroll_row`, misleading API names) and 7 minor (undocumented constants, search semantics, placeholder stubs).
- The most impactful gaps are in `emoji/mod.rs` (no module docs, undocumented `Emoji` struct and ordering invariant), `emoji_picker.rs` (no top-level doc for the core render function), and `prompt_footer.rs` (misleading `from_design()` that ignores its parameter, hidden magic strings suppressing rendering).
- Public API surface in `builtins/mod.rs` (`BuiltInFeature`, `BuiltInEntry`, `get_builtin_entries()`) and `app_view_state.rs` (`InputMode` enum) lack any rustdoc, making the built-in command lifecycle and input mode contract opaque to contributors.


---

## Summary

**Completed:** Fri Feb 13 09:20:22 MST 2026
**Iterations:** 7
**Status:** signal
