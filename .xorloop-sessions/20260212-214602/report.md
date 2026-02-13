# xorloop Report — 20260212-214602

**Project:** script-kit-gpui
**Branch:** main
**Started:** Thu Feb 12 21:46:02 MST 2026

---

## Iteration 1 — security audit (22:55)

**Feature:** FEATURE: Frecency scoring system that ranks scripts by combining usage frequency with recency via exponential half-life decay, persisted to JSON with atomic writes and cache-invalidation revision tracking.

- **Unbounded deserialization & unsafe file writes**: The frecency store lacks size limits on JSON loading (DoS risk), uses predictable temp paths without fsync (data corruption/symlink attacks), and creates files world-readable while logging sensitive filter text verbatim.
- **NaN propagation & ranking manipulation**: Half-life calculations don't guard against NaN, and loaded JSON is fully trusted — future timestamps prevent decay (permanent top rank), oversized keys bloat memory, and the filtering cache ignores frecency revision changes.
- **Information leakage via logs**: Both filter text (which may contain pasted tokens/passwords) and full script paths (which may reveal project/client names) are logged verbatim; frecency file permissions default to 0644 instead of 0600.


---

## Summary

**Completed:** Thu Feb 12 22:55:55 MST 2026
**Iterations:** 1
**Status:** signal
