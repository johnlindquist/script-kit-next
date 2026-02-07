//! Window State Persistence
//!
//! This module handles saving and restoring window positions for the main launcher,
//! Notes window, and AI window. Positions are stored in `~/.sk/kit/window-state.json`.
//!
//! # Architecture (Following Expert Review Recommendations)
//!
//! 1. **Canonical coordinate space**: "Global top-left origin (CoreGraphics-style), y increases downward"
//! 2. **Persistence via WindowBounds**: Aligns with GPUI's `WindowBounds` (Windowed/Maximized/Fullscreen)
//! 3. **Restore via WindowOptions.window_bounds**: No "jump after open"
//! 4. **Validation via geometry intersection**: Not display IDs (which can change)
//! 5. **Save on close/hide**: Main window saves on hide (since it's often hidden not closed)

include!("part_000.rs");
include!("part_001.rs");
