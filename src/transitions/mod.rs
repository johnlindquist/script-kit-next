//! UI Transitions Module
//!
//! Provides transition helpers for smooth UI animations.

// Allow dead code as this module provides utility functions that may not all be used yet
#![allow(dead_code)]
//!
//! # Key Components
//!
//! - `TransitionColor`: Color value supporting Lerp for smooth transitions
//! - `Opacity`: Opacity value (0.0-1.0) for fade transitions
//! - `SlideOffset`: X/Y offset for slide animations
//! - `AppearTransition`: Combined opacity + slide for toast/notification animations
//! - `HoverState`: Background color transition for list item hover effects
//!
//!
//! # Easing Functions
//!
//! - `linear`: No easing (constant velocity)
//! - `ease_out_quad`: Fast start, slow end (good for enter animations)
//! - `ease_in_quad`: Slow start, fast end (good for exit animations)
//! - `ease_in_out_quad`: Slow start and end (good for continuous loops)

include!("part_000.rs");
include!("part_001.rs");
