#[cfg(test)]
mod tests {
    use super::*;
    use gpui::px;

    fn default_layout() -> LayoutConfig {
        LayoutConfig::default()
    }

    #[test]
    fn test_script_list_fixed_height() {
        let layout = default_layout();

        // Script list should always be STANDARD_HEIGHT regardless of item count
        assert_eq!(
            height_for_view_with_layout(ViewType::ScriptList, 0, &layout),
            layout::STANDARD_HEIGHT
        );
        assert_eq!(
            height_for_view_with_layout(ViewType::ScriptList, 5, &layout),
            layout::STANDARD_HEIGHT
        );
        assert_eq!(
            height_for_view_with_layout(ViewType::ScriptList, 100, &layout),
            layout::STANDARD_HEIGHT
        );
    }

    #[test]
    fn test_arg_with_choices_dynamic_height() {
        let layout = default_layout();

        // Arg with choices should size to items, clamped to STANDARD_HEIGHT
        let base_height =
            layout::ARG_HEADER_HEIGHT + layout::ARG_DIVIDER_HEIGHT + layout::ARG_LIST_PADDING_Y;
        assert_eq!(
            height_for_view_with_layout(ViewType::ArgPromptWithChoices, 1, &layout),
            px(base_height + LIST_ITEM_HEIGHT)
        );
        assert_eq!(
            height_for_view_with_layout(ViewType::ArgPromptWithChoices, 2, &layout),
            px(base_height + (2.0 * LIST_ITEM_HEIGHT))
        );
        assert_eq!(
            height_for_view_with_layout(ViewType::ArgPromptWithChoices, 100, &layout),
            layout::STANDARD_HEIGHT
        );
    }

    #[test]
    fn test_arg_no_choices_compact() {
        let layout = default_layout();

        // Arg without choices should be MIN_HEIGHT
        assert_eq!(
            height_for_view_with_layout(ViewType::ArgPromptNoChoices, 0, &layout),
            layout::MIN_HEIGHT
        );
    }

    #[test]
    fn test_full_height_views() {
        let layout = default_layout();

        // Editor and Terminal use MAX_HEIGHT (700px)
        assert_eq!(
            height_for_view_with_layout(ViewType::EditorPrompt, 0, &layout),
            layout::MAX_HEIGHT
        );
        assert_eq!(
            height_for_view_with_layout(ViewType::TermPrompt, 0, &layout),
            layout::MAX_HEIGHT
        );
    }

    #[test]
    fn test_div_prompt_standard_height() {
        let layout = default_layout();

        // DivPrompt uses STANDARD_HEIGHT (500px) to match main window
        assert_eq!(
            height_for_view_with_layout(ViewType::DivPrompt, 0, &layout),
            layout::STANDARD_HEIGHT
        );
    }

    #[test]
    fn test_initial_window_height() {
        let layout = default_layout();
        assert_eq!(
            initial_window_height_with_layout(&layout),
            layout::STANDARD_HEIGHT
        );
        assert_eq!(
            initial_window_height(),
            height_for_view(ViewType::ScriptList, 0)
        );
    }

    #[test]
    fn test_height_constants() {
        assert_eq!(layout::MIN_HEIGHT, px(layout::ARG_HEADER_HEIGHT));
        assert_eq!(layout::STANDARD_HEIGHT, px(500.0));
        assert_eq!(layout::MAX_HEIGHT, px(700.0));
    }

    #[test]
    fn test_layout_uses_configured_standard_and_max_height() {
        let custom_layout = LayoutConfig {
            standard_height: 540.0,
            max_height: 860.0,
        };

        assert_eq!(
            height_for_view_with_layout(ViewType::ScriptList, 0, &custom_layout),
            px(540.0)
        );
        assert_eq!(
            height_for_view_with_layout(ViewType::EditorPrompt, 0, &custom_layout),
            px(860.0)
        );
        assert_eq!(initial_window_height_with_layout(&custom_layout), px(540.0));
    }

    #[test]
    fn test_sanitize_layout_config_enforces_bounds() {
        let sanitized = sanitize_layout_config(LayoutConfig {
            standard_height: 10.0,
            max_height: 5.0,
        });

        assert_eq!(sanitized.standard_height, f32::from(layout::MIN_HEIGHT));
        assert_eq!(sanitized.max_height, f32::from(layout::MIN_HEIGHT));
    }

    #[test]
    fn test_calculate_resized_frame_keeps_top_edge_fixed() {
        let current = FrameGeometry::new(100.0, 200.0, 750.0, 500.0);
        let resized = calculate_resized_frame(current, 700.0, None, None);

        assert!((resized.y - 0.0).abs() < 0.001);
        assert!((resized.height - 700.0).abs() < 0.001);
    }

    #[test]
    fn test_calculate_resized_frame_clamps_bottom_to_visible_bounds() {
        let current = FrameGeometry::new(100.0, 200.0, 750.0, 500.0);
        let visible = FrameGeometry::new(0.0, 50.0, 1920.0, 800.0);
        let resized = calculate_resized_frame(current, 700.0, Some(visible), None);

        assert!((resized.y - 54.0).abs() < 0.001);
    }

    #[test]
    fn test_calculate_resized_frame_caps_height_to_visible_bounds() {
        let current = FrameGeometry::new(100.0, 300.0, 750.0, 400.0);
        let visible = FrameGeometry::new(0.0, 0.0, 1920.0, 700.0);
        let resized = calculate_resized_frame(current, 900.0, Some(visible), None);

        assert!((resized.height - 692.0).abs() < 0.001);
        assert!((resized.y - 4.0).abs() < 0.001);
    }

    #[test]
    fn test_calculate_resized_frame_rounds_to_backing_scale() {
        let current = FrameGeometry::new(10.1, 20.2, 749.7, 500.3);
        let resized = calculate_resized_frame(current, 700.7, None, Some(2.0));

        assert!((resized.x - 10.0).abs() < 0.001);
        assert!((resized.y - -180.0).abs() < 0.001);
        assert!((resized.width - 749.5).abs() < 0.001);
        assert!((resized.height - 700.5).abs() < 0.001);
    }

    #[test]
    fn test_should_apply_resize_true_when_height_changes() {
        assert!(should_apply_resize(500.0, 700.0));
    }

    #[test]
    fn test_should_apply_resize_false_when_height_is_effectively_unchanged() {
        assert!(!should_apply_resize(500.0, 500.4));
    }

    #[test]
    fn test_window_resize_animation_flag_is_disabled() {
        assert!(
            !WINDOW_RESIZE_ANIMATE,
            "Window resize must stay instant with animation disabled"
        );
    }
}
