//! UI Transitions Module
//!
//! Provides transition helpers for smooth UI animations.
//!
//! # Key Components
//!
//! - `Lerp`: Generic linear interpolation trait
//! - `Opacity`: Opacity value (0.0-1.0) for fade transitions
//!
//! # Easing Functions
//!
//! - `linear`: No easing (constant velocity)
//! - `ease_out_quad`: Fast start, slow end (good for enter animations)
//! - `ease_in_quad`: Slow start, fast end (good for exit animations)
//! - `ease_in_out_quad`: Slow start and end (good for continuous loops)

use std::time::Duration;

// ============================================================================
// Lerp Trait
// ============================================================================

/// A value which can be linearly interpolated with another value of the same type.
///
/// The `delta` parameter is a value from 0.0 to 1.0 where:
/// - 0.0 returns `self`
/// - 1.0 returns `to`
/// - Values in between return a linear interpolation
pub trait Lerp {
    fn lerp(&self, to: &Self, delta: f32) -> Self;
}

// ============================================================================
// Standard Durations
// ============================================================================

/// Fast transition (100ms) - for hover effects, micro-interactions
pub const DURATION_FAST: Duration = Duration::from_millis(100);
/// Standard transition (150ms) - for selection changes, hover feedback
pub const DURATION_STANDARD: Duration = Duration::from_millis(150);
/// Medium transition (200ms) - for panel reveals, focus changes
pub const DURATION_MEDIUM: Duration = Duration::from_millis(200);
/// Slow transition (300ms) - for large UI changes, modal appearances
pub const DURATION_SLOW: Duration = Duration::from_millis(300);

// ============================================================================
// Easing Functions
// ============================================================================

/// Linear easing - constant velocity
#[inline]
pub fn linear(t: f32) -> f32 {
    t
}

/// Quadratic ease out - fast start, slow end
/// Good for elements entering the screen
#[inline]
pub fn ease_out_quad(t: f32) -> f32 {
    1.0 - (1.0 - t) * (1.0 - t)
}

/// Quadratic ease in - slow start, fast end
/// Good for elements leaving the screen
#[inline]
pub fn ease_in_quad(t: f32) -> f32 {
    t * t
}

/// Quadratic ease in-out - slow start and end
/// Good for continuous looping animations
#[inline]
pub fn ease_in_out_quad(t: f32) -> f32 {
    if t < 0.5 {
        2.0 * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
    }
}

/// Cubic ease out - faster deceleration than quadratic
#[inline]
pub fn ease_out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

/// Cubic ease in - slower acceleration than quadratic
#[inline]
pub fn ease_in_cubic(t: f32) -> f32 {
    t * t * t
}

// ============================================================================
// Lerp Implementations for Primitives
// ============================================================================

impl Lerp for f32 {
    #[inline]
    fn lerp(&self, to: &Self, delta: f32) -> Self {
        self + (to - self) * delta
    }
}

impl Lerp for f64 {
    #[inline]
    fn lerp(&self, to: &Self, delta: f32) -> Self {
        self + (to - self) * (delta as f64)
    }
}

// ============================================================================
// Opacity Transition Helper
// ============================================================================

/// Opacity value for fade transitions (0.0 = invisible, 1.0 = fully visible)
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Opacity(pub f32);

impl Opacity {
    pub const INVISIBLE: Self = Self(0.0);
    pub const VISIBLE: Self = Self(1.0);
    pub const HALF: Self = Self(0.5);

    pub fn new(value: f32) -> Self {
        Self(value.clamp(0.0, 1.0))
    }

    pub fn value(&self) -> f32 {
        self.0
    }
}

impl Lerp for Opacity {
    fn lerp(&self, to: &Self, delta: f32) -> Self {
        Self(self.0 + (to.0 - self.0) * delta)
    }
}

impl Default for Opacity {
    fn default() -> Self {
        Self::VISIBLE
    }
}

// Keep these exported transition primitives live in binary targets where some
// symbols are currently not referenced directly.
const _: () = {
    let _ = DURATION_FAST;
    let _ = DURATION_STANDARD;
    let _ = DURATION_MEDIUM;
    let _ = DURATION_SLOW;
    let _ = linear as fn(f32) -> f32;
    let _ = ease_out_quad as fn(f32) -> f32;
    let _ = ease_in_quad as fn(f32) -> f32;
    let _ = ease_in_out_quad as fn(f32) -> f32;
    let _ = ease_out_cubic as fn(f32) -> f32;
    let _ = ease_in_cubic as fn(f32) -> f32;
    let _ = Opacity::INVISIBLE;
    let _ = Opacity::VISIBLE;
    let _ = Opacity::HALF;
    let _ = Opacity::new as fn(f32) -> Opacity;
    let _ = Opacity::value as fn(&Opacity) -> f32;
};

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // Easing Function Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_linear_easing() {
        assert!((linear(0.0) - 0.0).abs() < f32::EPSILON);
        assert!((linear(0.5) - 0.5).abs() < f32::EPSILON);
        assert!((linear(1.0) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_ease_out_quad() {
        // t=0 should give 0
        assert!((ease_out_quad(0.0) - 0.0).abs() < f32::EPSILON);
        // t=1 should give 1
        assert!((ease_out_quad(1.0) - 1.0).abs() < f32::EPSILON);
        // t=0.5 should be > 0.5 (fast start)
        assert!(ease_out_quad(0.5) > 0.5);
    }

    #[test]
    fn test_ease_in_quad() {
        assert!((ease_in_quad(0.0) - 0.0).abs() < f32::EPSILON);
        assert!((ease_in_quad(1.0) - 1.0).abs() < f32::EPSILON);
        // t=0.5 should be < 0.5 (slow start)
        assert!(ease_in_quad(0.5) < 0.5);
    }

    #[test]
    fn test_ease_in_out_quad() {
        assert!((ease_in_out_quad(0.0) - 0.0).abs() < f32::EPSILON);
        assert!((ease_in_out_quad(1.0) - 1.0).abs() < f32::EPSILON);
        // t=0.5 should be exactly 0.5 (symmetric)
        assert!((ease_in_out_quad(0.5) - 0.5).abs() < f32::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Opacity Lerp Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_opacity_lerp_start_end() {
        let from = Opacity::INVISIBLE;
        let to = Opacity::VISIBLE;

        // At delta=0, should be start value
        let result = from.lerp(&to, 0.0);
        assert!((result.0 - 0.0).abs() < f32::EPSILON);

        // At delta=1, should be end value
        let result = from.lerp(&to, 1.0);
        assert!((result.0 - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_opacity_lerp_midpoint() {
        let from = Opacity::INVISIBLE;
        let to = Opacity::VISIBLE;

        // At delta=0.5, should be halfway
        let result = from.lerp(&to, 0.5);
        assert!((result.0 - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_opacity_clamp() {
        let clamped = Opacity::new(1.5);
        assert!((clamped.0 - 1.0).abs() < f32::EPSILON);

        let clamped = Opacity::new(-0.5);
        assert!((clamped.0 - 0.0).abs() < f32::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Duration Constants Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_duration_ordering() {
        assert!(DURATION_FAST < DURATION_STANDARD);
        assert!(DURATION_STANDARD < DURATION_MEDIUM);
        assert!(DURATION_MEDIUM < DURATION_SLOW);
    }

    #[test]
    fn test_duration_values() {
        assert_eq!(DURATION_FAST.as_millis(), 100);
        assert_eq!(DURATION_STANDARD.as_millis(), 150);
        assert_eq!(DURATION_MEDIUM.as_millis(), 200);
        assert_eq!(DURATION_SLOW.as_millis(), 300);
    }

    // -------------------------------------------------------------------------
    // Primitive Lerp Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_f32_lerp() {
        assert!((0.0_f32.lerp(&1.0, 0.0) - 0.0).abs() < f32::EPSILON);
        assert!((0.0_f32.lerp(&1.0, 0.5) - 0.5).abs() < f32::EPSILON);
        assert!((0.0_f32.lerp(&1.0, 1.0) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_f64_lerp() {
        assert!((0.0_f64.lerp(&1.0, 0.0) - 0.0).abs() < f64::EPSILON);
        assert!((0.0_f64.lerp(&1.0, 0.5) - 0.5).abs() < f64::EPSILON);
        assert!((0.0_f64.lerp(&1.0, 1.0) - 1.0).abs() < f64::EPSILON);
    }
}
