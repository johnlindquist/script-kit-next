//! WindowBounds type and size constraints

/// Window bounds in AX coordinate system (top-left origin, Y grows downward).
///
/// This is the canonical coordinate system used internally by this module.
/// All bounds returned from queries and accepted by setters use AX coords.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WindowBounds {
    /// X coordinate (pixels from left edge of primary display)
    pub x: f64,
    /// Y coordinate (pixels from top edge of primary display, grows downward)
    pub y: f64,
    /// Width in pixels
    pub width: f64,
    /// Height in pixels
    pub height: f64,
}

impl WindowBounds {
    /// Create new bounds
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Create bounds from integer values (convenience for legacy Bounds compatibility)
    pub fn from_ints(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            x: x as f64,
            y: y as f64,
            width: width as f64,
            height: height as f64,
        }
    }

    /// Get the center point of these bounds
    pub fn center(&self) -> (f64, f64) {
        (self.x + self.width / 2.0, self.y + self.height / 2.0)
    }

    /// Get the right edge X coordinate
    pub fn right(&self) -> f64 {
        self.x + self.width
    }

    /// Get the bottom edge Y coordinate
    pub fn bottom(&self) -> f64 {
        self.y + self.height
    }

    /// Check if a point is within these bounds
    pub fn contains_point(&self, x: f64, y: f64) -> bool {
        x >= self.x && x < self.right() && y >= self.y && y < self.bottom()
    }

    /// Calculate the intersection area with another bounds
    pub fn intersection_area(&self, other: &WindowBounds) -> f64 {
        let left = self.x.max(other.x);
        let top = self.y.max(other.y);
        let right = self.right().min(other.right());
        let bottom = self.bottom().min(other.bottom());

        if right > left && bottom > top {
            (right - left) * (bottom - top)
        } else {
            0.0
        }
    }

    /// Total area of these bounds
    pub fn area(&self) -> f64 {
        self.width * self.height
    }

    /// Clamp these bounds to fit within container bounds
    pub fn clamp_to(&self, container: &WindowBounds) -> WindowBounds {
        let mut result = *self;

        // Clamp width/height to container
        result.width = result.width.min(container.width);
        result.height = result.height.min(container.height);

        // Clamp position so window stays within container
        result.x = result.x.max(container.x);
        result.y = result.y.max(container.y);

        if result.right() > container.right() {
            result.x = container.right() - result.width;
        }
        if result.bottom() > container.bottom() {
            result.y = container.bottom() - result.height;
        }

        result
    }
}

impl Default for WindowBounds {
    fn default() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0)
    }
}

/// Optional size constraints for a window
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct SizeConstraints {
    /// Minimum width (if available)
    pub min_width: Option<f64>,
    /// Minimum height (if available)
    pub min_height: Option<f64>,
    /// Maximum width (if available)
    pub max_width: Option<f64>,
    /// Maximum height (if available)
    pub max_height: Option<f64>,
}

impl SizeConstraints {
    /// Clamp a size to these constraints
    pub fn clamp_size(&self, width: f64, height: f64) -> (f64, f64) {
        let w = width
            .max(self.min_width.unwrap_or(0.0))
            .min(self.max_width.unwrap_or(f64::MAX));
        let h = height
            .max(self.min_height.unwrap_or(0.0))
            .min(self.max_height.unwrap_or(f64::MAX));
        (w, h)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_and_default() {
        let bounds = WindowBounds::new(100.0, 200.0, 800.0, 600.0);
        assert_eq!(bounds.x, 100.0);
        assert_eq!(bounds.y, 200.0);
        assert_eq!(bounds.width, 800.0);
        assert_eq!(bounds.height, 600.0);

        let default = WindowBounds::default();
        assert_eq!(default.x, 0.0);
        assert_eq!(default.y, 0.0);
    }

    #[test]
    fn test_from_ints() {
        let bounds = WindowBounds::from_ints(10, 20, 100, 200);
        assert_eq!(bounds.x, 10.0);
        assert_eq!(bounds.y, 20.0);
        assert_eq!(bounds.width, 100.0);
        assert_eq!(bounds.height, 200.0);
    }

    #[test]
    fn test_center() {
        let bounds = WindowBounds::new(100.0, 200.0, 800.0, 600.0);
        let (cx, cy) = bounds.center();
        assert_eq!(cx, 500.0);
        assert_eq!(cy, 500.0);
    }

    #[test]
    fn test_right_and_bottom() {
        let bounds = WindowBounds::new(100.0, 200.0, 800.0, 600.0);
        assert_eq!(bounds.right(), 900.0);
        assert_eq!(bounds.bottom(), 800.0);
    }

    #[test]
    fn test_contains_point() {
        let bounds = WindowBounds::new(100.0, 100.0, 200.0, 200.0);
        assert!(bounds.contains_point(150.0, 150.0));
        assert!(bounds.contains_point(100.0, 100.0));
        assert!(!bounds.contains_point(99.0, 150.0));
        assert!(!bounds.contains_point(300.0, 150.0));
    }

    #[test]
    fn test_intersection_area() {
        let a = WindowBounds::new(0.0, 0.0, 100.0, 100.0);
        let b = WindowBounds::new(50.0, 50.0, 100.0, 100.0);
        assert_eq!(a.intersection_area(&b), 2500.0);

        let c = WindowBounds::new(200.0, 200.0, 50.0, 50.0);
        assert_eq!(a.intersection_area(&c), 0.0);
    }

    #[test]
    fn test_clamp_to() {
        let container = WindowBounds::new(0.0, 0.0, 1920.0, 1080.0);
        let window = WindowBounds::new(1800.0, 100.0, 200.0, 200.0);
        let clamped = window.clamp_to(&container);
        assert_eq!(clamped.x, 1720.0);
        assert_eq!(clamped.y, 100.0);
    }

    #[test]
    fn test_size_constraints_clamp() {
        let constraints = SizeConstraints {
            min_width: Some(100.0),
            min_height: Some(100.0),
            max_width: Some(1000.0),
            max_height: Some(800.0),
        };

        let (w, h) = constraints.clamp_size(500.0, 400.0);
        assert_eq!(w, 500.0);
        assert_eq!(h, 400.0);

        let (w, h) = constraints.clamp_size(50.0, 50.0);
        assert_eq!(w, 100.0);
        assert_eq!(h, 100.0);

        let (w, h) = constraints.clamp_size(2000.0, 1500.0);
        assert_eq!(w, 1000.0);
        assert_eq!(h, 800.0);
    }
}
