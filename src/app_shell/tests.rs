//! Tests for app_shell module

use super::*;

mod chrome_tests {
    use super::*;

    #[test]
    fn chrome_mode_full_frame_shows_divider() {
        assert!(ChromeMode::FullFrame.shows_divider());
        assert!(ChromeMode::FullFrame.has_background());
        assert!(ChromeMode::FullFrame.has_shadow());
    }

    #[test]
    fn chrome_mode_minimal_hides_divider() {
        assert!(!ChromeMode::MinimalFrame.shows_divider());
        assert!(ChromeMode::MinimalFrame.has_background());
        assert!(ChromeMode::MinimalFrame.has_shadow());
    }

    #[test]
    fn chrome_mode_content_only_has_no_styling() {
        assert!(!ChromeMode::ContentOnly.shows_divider());
        assert!(!ChromeMode::ContentOnly.has_background());
        assert!(!ChromeMode::ContentOnly.has_shadow());
    }

    #[test]
    fn chrome_spec_full_frame_defaults() {
        let spec = ChromeSpec::full_frame();
        assert_eq!(spec.mode, ChromeMode::FullFrame);
        assert_eq!(spec.border_radius, 12.0);
        assert_eq!(spec.divider, DividerSpec::Hairline);
        assert!(spec.should_show_divider());
    }

    #[test]
    fn chrome_spec_minimal_defaults() {
        let spec = ChromeSpec::minimal();
        assert_eq!(spec.mode, ChromeMode::MinimalFrame);
        assert_eq!(spec.border_radius, 8.0);
        assert_eq!(spec.divider, DividerSpec::None);
        assert!(!spec.should_show_divider());
    }

    #[test]
    fn chrome_spec_content_only_defaults() {
        let spec = ChromeSpec::content_only();
        assert_eq!(spec.mode, ChromeMode::ContentOnly);
        assert_eq!(spec.border_radius, 0.0);
        assert!(!spec.should_show_divider());
    }

    #[test]
    fn chrome_spec_builder_pattern() {
        let spec = ChromeSpec::full_frame()
            .radius(16.0)
            .opacity(0.75)
            .divider(DividerSpec::None);

        assert_eq!(spec.border_radius, 16.0);
        assert_eq!(spec.background_opacity, 0.75);
        assert_eq!(spec.divider, DividerSpec::None);
        assert!(!spec.should_show_divider());
    }

    #[test]
    fn divider_spec_height() {
        assert_eq!(DividerSpec::None.height(), gpui::px(0.0));
        assert_eq!(DividerSpec::Hairline.height(), gpui::px(1.0));
    }
}

mod focus_tests {
    use super::*;

    #[test]
    fn focus_policy_default_is_header_input() {
        assert_eq!(FocusPolicy::default(), FocusPolicy::HeaderInput);
    }
}

mod keymap_tests {
    use super::keymap::{default_bindings, route_key, Modifiers};
    use super::*;

    #[test]
    fn shell_action_none_is_not_handled() {
        assert!(ShellAction::None.is_none());
        assert!(!ShellAction::None.is_handled());
    }

    #[test]
    fn shell_action_cancel_is_handled() {
        assert!(!ShellAction::Cancel.is_none());
        assert!(ShellAction::Cancel.is_handled());
    }

    #[test]
    fn modifiers_command_only() {
        let mods = Modifiers::command();
        assert!(mods.command);
        assert!(!mods.shift);
        assert!(!mods.alt);
        assert!(!mods.control);
        assert!(!mods.is_empty());
    }

    #[test]
    fn modifiers_empty() {
        let mods = Modifiers::default();
        assert!(mods.is_empty());
    }

    #[test]
    fn keymap_spec_bind() {
        let keymap = KeymapSpec::new()
            .bind("j", ShellAction::Next)
            .bind("k", ShellAction::Prev);

        assert_eq!(keymap.bindings.len(), 2);
        assert_eq!(
            keymap.lookup("j", &Modifiers::default()),
            Some(ShellAction::Next)
        );
        assert_eq!(
            keymap.lookup("k", &Modifiers::default()),
            Some(ShellAction::Prev)
        );
    }

    #[test]
    fn keymap_spec_modal_blocks_defaults() {
        let keymap = KeymapSpec::modal();
        assert!(keymap.modal);

        // Modal keymap should return None for default bindings
        let action = route_key("escape", &Modifiers::default(), &keymap);
        assert_eq!(action, ShellAction::None);
    }

    #[test]
    fn route_key_escape_returns_cancel() {
        let keymap = KeymapSpec::new();
        let action = route_key("escape", &Modifiers::default(), &keymap);
        assert_eq!(action, ShellAction::Cancel);
    }

    #[test]
    fn route_key_cmd_k_returns_open_actions() {
        let keymap = KeymapSpec::new();
        let action = route_key("k", &Modifiers::command(), &keymap);
        assert_eq!(action, ShellAction::OpenActions);
    }

    #[test]
    fn route_key_view_override_takes_precedence() {
        let keymap = KeymapSpec::new().bind("escape", ShellAction::FocusSearch);

        let action = route_key("escape", &Modifiers::default(), &keymap);
        assert_eq!(action, ShellAction::FocusSearch);
    }

    #[test]
    fn test_route_key_returns_prev_for_arrowup_variants() {
        let keymap = KeymapSpec::new();
        assert_eq!(
            route_key("arrowup", &Modifiers::default(), &keymap),
            ShellAction::Prev
        );
        assert_eq!(
            route_key("ArrowUp", &Modifiers::default(), &keymap),
            ShellAction::Prev
        );
    }

    #[test]
    fn test_route_key_returns_next_for_arrowdown_variants() {
        let keymap = KeymapSpec::new();
        assert_eq!(
            route_key("arrowdown", &Modifiers::default(), &keymap),
            ShellAction::Next
        );
        assert_eq!(
            route_key("DownArrow", &Modifiers::default(), &keymap),
            ShellAction::Next
        );
    }

    #[test]
    fn test_route_key_returns_cancel_for_escape_variants() {
        let keymap = KeymapSpec::new();
        assert_eq!(
            route_key("Esc", &Modifiers::default(), &keymap),
            ShellAction::Cancel
        );
        assert_eq!(
            route_key("Escape", &Modifiers::default(), &keymap),
            ShellAction::Cancel
        );
    }

    #[test]
    fn test_route_key_returns_run_for_return_variant() {
        let keymap = KeymapSpec::new();
        assert_eq!(
            route_key("return", &Modifiers::default(), &keymap),
            ShellAction::Run
        );
    }

    #[test]
    fn test_keymap_spec_bind_normalizes_variant_keys_for_lookup() {
        let keymap = KeymapSpec::new().bind("ArrowUp", ShellAction::Prev);
        assert_eq!(
            keymap.lookup("up", &Modifiers::default()),
            Some(ShellAction::Prev)
        );
        assert_eq!(
            keymap.lookup("UpArrow", &Modifiers::default()),
            Some(ShellAction::Prev)
        );
    }

    #[test]
    fn default_bindings_include_expected_keys() {
        let defaults = default_bindings();

        // Check that expected bindings exist
        let escape_binding = defaults.iter().find(|b| b.key == "escape");
        assert!(escape_binding.is_some());
        assert_eq!(escape_binding.unwrap().action, ShellAction::Cancel);

        let enter_binding = defaults.iter().find(|b| b.key == "enter");
        assert!(enter_binding.is_some());
        assert_eq!(enter_binding.unwrap().action, ShellAction::Run);

        let cmd_k_binding = defaults
            .iter()
            .find(|b| b.key == "k" && b.modifiers.command);
        assert!(cmd_k_binding.is_some());
        assert_eq!(cmd_k_binding.unwrap().action, ShellAction::OpenActions);
    }
}

mod spec_tests {
    use super::spec::ButtonAction;
    use super::*;
    use crate::components::button::ButtonVariant;

    #[test]
    fn shell_spec_builder_pattern() {
        let spec = ShellSpec::new()
            .chrome(ChromeSpec::minimal())
            .focus_policy(FocusPolicy::Content);

        assert_eq!(spec.chrome.mode, ChromeMode::MinimalFrame);
        assert_eq!(spec.focus_policy, FocusPolicy::Content);
        assert!(!spec.has_header());
        assert!(!spec.has_footer());
    }

    #[test]
    fn header_spec_search_builder() {
        let header = HeaderSpec::search("Type to search...")
            .text("hello")
            .cursor_visible(true)
            .button("Run", "↵")
            .logo(true);

        assert!(header.input.is_some());
        let input = header.input.unwrap();
        assert_eq!(input.placeholder.as_ref(), "Type to search...");
        assert_eq!(input.text.as_ref(), "hello");
        assert!(input.cursor_visible);
        assert_eq!(header.buttons.len(), 1);
        assert!(header.show_logo);
    }

    #[test]
    fn footer_spec_builder() {
        let footer = FooterSpec::new()
            .primary("Run Script", "↵")
            .secondary("Actions", "⌘K")
            .helper("Tab 1 of 2")
            .info("typescript");

        assert_eq!(footer.primary_label.as_ref(), "Run Script");
        assert_eq!(footer.primary_shortcut.as_ref(), "↵");
        assert_eq!(
            footer.secondary_label.as_ref().map(|s| s.as_ref()),
            Some("Actions")
        );
        assert_eq!(
            footer.secondary_shortcut.as_ref().map(|s| s.as_ref()),
            Some("⌘K")
        );
        assert_eq!(
            footer.helper_text.as_ref().map(|s| s.as_ref()),
            Some("Tab 1 of 2")
        );
        assert_eq!(
            footer.info_label.as_ref().map(|s| s.as_ref()),
            Some("typescript")
        );
    }

    #[test]
    fn button_spec_primary_defaults() {
        let btn = ButtonSpec::primary("Run", "↵");
        assert_eq!(btn.label.as_ref(), "Run");
        assert_eq!(btn.shortcut.as_ref(), "↵");
        assert_eq!(btn.action, ButtonAction::Submit);
        assert_eq!(btn.variant, ButtonVariant::Primary);
    }

    #[test]
    fn button_spec_secondary_defaults() {
        let btn = ButtonSpec::secondary("Actions", "⌘K");
        assert_eq!(btn.label.as_ref(), "Actions");
        assert_eq!(btn.action, ButtonAction::OpenActions);
        assert_eq!(btn.variant, ButtonVariant::Ghost);
    }

    #[test]
    fn button_action_default_is_submit() {
        assert_eq!(ButtonAction::default(), ButtonAction::Submit);
    }
}
