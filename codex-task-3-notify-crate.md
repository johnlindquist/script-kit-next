# Research Task: Rust notify crate for App Watching

Research best practices for using the `notify` crate to watch application folders:

1. Check Cargo.toml for current notify crate version
2. Research notify crate patterns for:
   - Watching multiple directories
   - Recursive vs non-recursive watching
   - Debouncing events (apps install with many file operations)
   - Filtering for .app bundles only
   
3. What's the recommended approach for:
   - Handling high-volume directories like /Applications
   - Avoiding watching inside .app bundles (they're directories)
   - Detecting when an app bundle is "complete" (fully copied)

4. Provide code examples for efficient app folder watching
