# Research Task: Integration with App Launcher

Research how to integrate app folder watching with the existing app launcher:

1. Find the current app launcher implementation in the codebase
2. How are apps currently loaded/cached?
3. What data structures hold the app list?
4. How should the watcher notify the app list to refresh?

Look for:
- App launcher built-in functionality
- How apps are enumerated
- Cache invalidation patterns
- UI refresh triggers (cx.notify patterns)

Propose an architecture for: watcher detects new app → updates app cache → UI refreshes
