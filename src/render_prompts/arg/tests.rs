mod arg_prompt_render_tests {
    use super::*;

    use crate::designs::{get_tokens, DesignColors, DesignVariant};
    use crate::theme::Theme;
    use crate::protocol::Choice;

    fn choice(name: &str, value: &str) -> Choice {
        Choice::new(name.to_string(), value.to_string())
    }

    #[test]
    fn prompt_actions_dialog_offsets_match_legacy_defaults() {
        let tokens = get_tokens(DesignVariant::Default);
        let spacing = tokens.spacing();
        let visual = tokens.visual();

        let (top, right) = prompt_actions_dialog_offsets(spacing.padding_sm, visual.border_thin);
        assert_eq!(top, 52.0);
        assert_eq!(right, 8.0);
    }

    #[test]
    fn test_prompt_render_context_new_extracts_design_tokens_and_offsets() {
        let theme = Theme::default();
        let variant = DesignVariant::Default;
        let tokens = get_tokens(variant);

        let context = PromptRenderContext::new(&theme, variant);
        let (expected_top, expected_right) = prompt_actions_dialog_offsets(
            context.design_spacing.padding_sm,
            context.design_visual.border_thin,
        );

        assert!(std::ptr::eq(context.theme, &theme));
        assert_eq!(context.design_colors, tokens.colors());
        assert_eq!(context.design_spacing, tokens.spacing());
        assert_eq!(context.design_typography, tokens.typography());
        assert_eq!(context.design_visual, tokens.visual());
        assert_eq!(context.actions_dialog_top, expected_top);
        assert_eq!(context.actions_dialog_right, expected_right);
    }

    #[test]
    fn prompt_footer_config_has_consistent_actions_defaults() {
        let config =
            prompt_footer_config_with_status("Continue", true, Some("Running".into()), None);
        assert_eq!(config.primary_label, "Continue");
        assert_eq!(config.primary_shortcut, "↵");
        assert_eq!(config.secondary_label, "Actions");
        assert_eq!(config.secondary_shortcut, "⌘K");
        assert!(config.show_secondary);
        assert_eq!(config.helper_text.as_deref(), Some("Running"));
    }

    #[test]
    fn prompt_footer_colors_use_selected_background_for_surface() {
        let design_colors = DesignColors {
            background_secondary: 0x123456,
            background_selected: 0xabcdef,
            ..DesignColors::default()
        };

        let footer_colors = prompt_footer_colors_for_prompt(&design_colors, true);

        assert_eq!(footer_colors.background, 0xabcdef);
        assert!(footer_colors.is_light_mode);
    }

    #[test]
    fn test_footer_surface_color_uses_surface_token_in_light_mode() {
        let footer = crate::components::prompt_footer::PromptFooterColors {
            accent: 0,
            text_muted: 0,
            border: 0,
            surface: 0x123456,
            background: 0x000000,
            is_light_mode: true,
        };

        assert_eq!(
            crate::components::prompt_footer::footer_surface_rgba(footer),
            0x123456ff
        );
    }

    #[test]
    fn running_status_text_is_contextual() {
        assert_eq!(
            running_status_text("awaiting input"),
            "Script running · awaiting input"
        );
    }

    #[test]
    fn test_resolve_arg_submit_outcome_returns_invalid_when_input_is_empty() {
        let outcome = resolve_arg_submit_outcome(None, "");
        assert_eq!(outcome, ArgSubmitOutcome::InvalidEmpty);
    }

    #[test]
    fn test_resolve_arg_submit_outcome_returns_selected_choice_value_when_available() {
        let outcome = resolve_arg_submit_outcome(Some("selected-choice"), "typed value");
        assert_eq!(
            outcome,
            ArgSubmitOutcome::SubmitChoice("selected-choice".to_string())
        );
    }

    #[test]
    fn test_resolve_arg_submit_outcome_returns_raw_text_when_no_selection_and_non_empty_input() {
        let outcome = resolve_arg_submit_outcome(None, "typed value");
        assert_eq!(
            outcome,
            ArgSubmitOutcome::SubmitText("typed value".to_string())
        );
    }

    #[test]
    fn test_resolve_arg_helper_status_returns_no_match_hint_when_choices_filtered_out() {
        let status = resolve_arg_helper_status(true, 0, false);
        assert_eq!(status, ArgHelperStatus::NoMatchesSubmitTypedValue);
        assert_eq!(
            arg_helper_status_text(status),
            "Script running · no matches · Enter submits typed value"
        );
    }

    #[test]
    fn test_resolve_arg_tab_completion_returns_single_choice_when_single_match() {
        let choices = [choice("Alpha", "alpha")];
        let filtered: Vec<(usize, &Choice)> = choices.iter().enumerate().collect();
        assert_eq!(
            resolve_arg_tab_completion(&filtered, 0),
            Some("Alpha".to_string())
        );
    }

    #[test]
    fn test_resolve_arg_tab_completion_uses_selected_choice_when_multiple_matches() {
        let choices = [choice("Alpha", "alpha"), choice("Bravo", "bravo")];
        let filtered: Vec<(usize, &Choice)> = choices.iter().enumerate().collect();
        assert_eq!(
            resolve_arg_tab_completion(&filtered, 1),
            Some("Bravo".to_string())
        );
    }

    #[test]
    fn test_resolve_arg_tab_completion_falls_back_to_first_choice_when_selection_is_oob() {
        let choices = [choice("Alpha", "alpha"), choice("Bravo", "bravo")];
        let filtered: Vec<(usize, &Choice)> = choices.iter().enumerate().collect();
        assert_eq!(
            resolve_arg_tab_completion(&filtered, 99),
            Some("Alpha".to_string())
        );
    }

    #[test]
    fn test_arg_prompt_input_text_uses_theme_tokens_when_rendering() {
        let render_source = include_str!("render.rs");

        assert!(
            render_source.contains("let text_primary = self.theme.colors.text.primary;"),
            "arg prompt text should use theme.colors.text.primary"
        );
        assert!(
            render_source.contains("let text_muted = self.theme.colors.text.muted;"),
            "arg prompt placeholder text should use theme.colors.text.muted"
        );
        assert!(
            !render_source.contains("let text_primary = design_colors.text_primary;"),
            "arg prompt text should not use design_colors.text_primary"
        );
        assert!(
            !render_source.contains("let text_muted = design_colors.text_muted;"),
            "arg prompt placeholder text should not use design_colors.text_muted"
        );
    }

    #[test]
    fn test_prompt_render_context_constructor_is_used_across_prompt_renderers() {
        assert!(
            include_str!("render.rs").contains("PromptRenderContext::new("),
            "arg prompt render should construct PromptRenderContext"
        );
        assert!(
            include_str!("../div.rs").contains("PromptRenderContext::new("),
            "div prompt render should construct PromptRenderContext"
        );
        assert!(
            include_str!("../editor.rs").contains("PromptRenderContext::new("),
            "editor prompt render should construct PromptRenderContext"
        );
        assert!(
            include_str!("../form/render.rs").contains("PromptRenderContext::new("),
            "form prompt render should construct PromptRenderContext"
        );
        assert!(
            include_str!("../term.rs").contains("PromptRenderContext::new("),
            "term prompt render should construct PromptRenderContext"
        );
        assert!(
            include_str!("../other.rs").contains("PromptRenderContext::new("),
            "other prompt render helpers should construct PromptRenderContext"
        );
    }

    #[test]
    fn test_key_preamble_helper_is_used_across_prompt_renderers() {
        assert!(
            include_str!("helpers.rs").contains("fn key_preamble("),
            "arg helpers should define key_preamble"
        );
        assert!(
            include_str!("render.rs").contains("key_preamble(this, event, true, false, cx)"),
            "arg prompt key handler should use key_preamble"
        );
        assert!(
            include_str!("../div.rs").contains("key_preamble(this, event, true, true, cx)"),
            "div prompt key handler should use key_preamble with propagation stop"
        );
        assert!(
            include_str!("../editor.rs").contains("key_preamble(this, event, false, false, cx)"),
            "editor prompt key handler should use key_preamble"
        );
        assert!(
            include_str!("../form/render.rs").contains("key_preamble(this, event, true, false, cx)"),
            "form prompt key handler should use key_preamble"
        );
        assert!(
            include_str!("../term.rs").contains("key_preamble(this, event, false, false, cx)"),
            "term prompt key handler should use key_preamble"
        );
    }
}
