# xorloop Report — 20260212-210644

**Project:** script-kit-gpui
**Branch:** main
**Started:** Thu Feb 12 21:06:44 MST 2026

---

## Iteration 1 — performance (21:23)

**Feature:** FEATURE: Quicklinks is a built-in feature that manages custom URL shortcuts with optional `{query}` placeholder expansion, allowing users to select a named link from a modal picker, optionally enter a search term, and open the expanded URL in the default browser.

- **Blocking I/O on UI thread is the critical issue**: synchronous disk reads, JSON parsing, and `osascript` process spawns in the Quicklinks flow freeze the app; must move to background executor via `cx.spawn()`.

- **Redundant work dominates the major findings**: double `osascript` spawns, double string escaping, chained `.replace()` allocations, full-buffer JSON I/O, and unconditional writes on no-op deletes/updates all create unnecessary CPU and memory pressure.

- **Minor wins center on avoiding repeated allocations**: short-circuit `expand_url` for non-placeholder links, return indices instead of labels from AppleScript, cache `quicklinks_path()` with `OnceLock`, and memoize `get_builtin_entries()` to stop per-keystroke rebuilds.


---

## Summary

**Completed:** Thu Feb 12 21:35:09 MST 2026
**Iterations:** 1
**Status:** signal
