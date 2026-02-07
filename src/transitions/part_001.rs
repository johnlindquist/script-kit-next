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
    // TransitionColor Lerp Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_transition_color_lerp() {
        let from = TransitionColor::transparent();
        let to = TransitionColor::new(1.0, 1.0, 1.0, 1.0);

        // At delta=0
        let result = from.lerp(&to, 0.0);
        assert!((result.a - 0.0).abs() < f32::EPSILON);

        // At delta=1
        let result = from.lerp(&to, 1.0);
        assert!((result.r - 1.0).abs() < f32::EPSILON);
        assert!((result.g - 1.0).abs() < f32::EPSILON);
        assert!((result.b - 1.0).abs() < f32::EPSILON);
        assert!((result.a - 1.0).abs() < f32::EPSILON);

        // At delta=0.5
        let result = from.lerp(&to, 0.5);
        assert!((result.r - 0.5).abs() < f32::EPSILON);
        assert!((result.a - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_transition_color_from_hex() {
        let color = TransitionColor::from_hex_alpha(0xFF8800, 0.5);
        assert!((color.r - 1.0).abs() < 0.01); // FF = 255 = 1.0
        assert!((color.g - 0.533).abs() < 0.01); // 88 = 136 = 0.533
        assert!((color.b - 0.0).abs() < f32::EPSILON); // 00 = 0 = 0.0
        assert!((color.a - 0.5).abs() < f32::EPSILON);
    }

    // -------------------------------------------------------------------------
    // SlideOffset Lerp Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_slide_offset_lerp() {
        let from = SlideOffset::from_bottom(20.0);
        let to = SlideOffset::ZERO;

        // At delta=0
        let result = from.lerp(&to, 0.0);
        assert!((result.y - 20.0).abs() < f32::EPSILON);

        // At delta=1
        let result = from.lerp(&to, 1.0);
        assert!((result.y - 0.0).abs() < f32::EPSILON);

        // At delta=0.5
        let result = from.lerp(&to, 0.5);
        assert!((result.y - 10.0).abs() < f32::EPSILON);
    }

    // -------------------------------------------------------------------------
    // AppearTransition Lerp Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_appear_transition_hidden_to_visible() {
        let from = AppearTransition::hidden();
        let to = AppearTransition::visible();

        // Check hidden state values
        assert!((from.opacity.0 - 0.0).abs() < f32::EPSILON);
        assert!((from.offset.y - 20.0).abs() < f32::EPSILON);

        // Check visible state values
        assert!((to.opacity.0 - 1.0).abs() < f32::EPSILON);
        assert!((to.offset.y - 0.0).abs() < f32::EPSILON);

        // At delta=0.5
        let result = from.lerp(&to, 0.5);
        assert!((result.opacity.0 - 0.5).abs() < f32::EPSILON);
        assert!((result.offset.y - 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_appear_transition_visible_to_dismissed() {
        let from = AppearTransition::visible();
        let to = AppearTransition::dismissed();

        // At delta=1
        let result = from.lerp(&to, 1.0);
        assert!((result.opacity.0 - 0.0).abs() < f32::EPSILON);
        assert!((result.offset.y - (-10.0)).abs() < f32::EPSILON);
    }

    // -------------------------------------------------------------------------
    // HoverState Lerp Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_hover_state_lerp() {
        let from = HoverState::normal();
        let to = HoverState::with_background(TransitionColor::from_hex_alpha(0xFFFFFF, 0.2));

        // At delta=0, should be transparent
        let result = from.lerp(&to, 0.0);
        assert!((result.background.a - 0.0).abs() < f32::EPSILON);

        // At delta=1, should be target color
        let result = from.lerp(&to, 1.0);
        assert!((result.background.a - 0.2).abs() < f32::EPSILON);
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
