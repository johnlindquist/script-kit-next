use super::*;

#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use super::*;

    #[test]
    fn env_placeholder_copy_is_contextual() {
        assert_eq!(
            env_input_placeholder("OPENAI_API_KEY", false),
            "Paste value for OPENAI_API_KEY"
        );
        assert_eq!(
            env_input_placeholder("OPENAI_API_KEY", true),
            "Paste a replacement value for OPENAI_API_KEY"
        );
    }

    #[test]
    fn env_run_status_message_mentions_waiting_state() {
        assert_eq!(
            env_running_status("OPENAI_API_KEY"),
            "Script is running and waiting for OPENAI_API_KEY"
        );
    }

    #[test]
    fn env_description_mentions_existing_secret_when_present() {
        assert_eq!(
            env_default_description("OPENAI_API_KEY", true),
            "Update the saved value for OPENAI_API_KEY"
        );
        assert_eq!(
            env_default_description("OPENAI_API_KEY", false),
            "Enter the value for OPENAI_API_KEY"
        );
    }

    #[test]
    fn test_env_secret_mask_uses_char_count_when_input_contains_unicode() {
        assert_eq!(masked_secret_value_for_display("abc"), "•••");
        assert_eq!(masked_secret_value_for_display("🔐é"), "••");
    }

    #[test]
    fn test_env_storage_hint_describes_encrypted_store_when_secret() {
        assert_eq!(
            env_storage_hint_text(true),
            "Stored securely in ~/.scriptkit/secrets.age"
        );
    }

    #[test]
    fn test_env_storage_hint_describes_ephemeral_mode_when_not_secret() {
        assert_eq!(
            env_storage_hint_text(false),
            "Value is provided to the script for this run only"
        );
    }

    #[test]
    fn test_env_validation_returns_error_when_submit_value_is_empty() {
        assert_eq!(
            env_submit_validation_error(""),
            Some("Value cannot be empty"),
        );
        assert_eq!(
            env_submit_validation_error("   "),
            Some("Value cannot be empty"),
        );
        assert_eq!(env_submit_validation_error("abc"), None);
    }

    #[test]
    fn test_env_key_action_handles_enter_and_escape_aliases_case_insensitively() {
        assert_eq!(env_key_action("return"), Some(EnvKeyAction::Submit));
        assert_eq!(env_key_action("Enter"), Some(EnvKeyAction::Submit));
        assert_eq!(env_key_action("escape"), Some(EnvKeyAction::Cancel));
        assert_eq!(env_key_action("esc"), Some(EnvKeyAction::Cancel));
        assert_eq!(env_key_action("ESC"), Some(EnvKeyAction::Cancel));
        assert_eq!(env_key_action("tab"), None);
    }

    #[test]
    fn env_prompt_uses_borderless_input_shell() {
        let source = include_str!("render.rs");
        assert!(
            source.contains("InlinePromptInput::new("),
            "env render should use InlinePromptInput for borderless input"
        );
        assert!(
            !source.contains(".border_1()"),
            "env render should not use .border_1() on input card"
        );
        assert!(
            !source.contains(".rounded(px(12.))"),
            "env render should not use .rounded(px(12.)) on input card"
        );
    }

    #[test]
    fn env_prompt_render_uses_hint_strip_not_prompt_footer() {
        let source = include_str!("render.rs");
        assert!(
            source.contains("render_simple_hint_strip("),
            "env render should use render_simple_hint_strip for minimal chrome footer"
        );
        assert!(
            source.contains("env_storage_hint_text("),
            "env render should preserve storage hint text in the hint strip leading slot"
        );
        assert!(
            !source.contains("PromptFooter::new("),
            "env render should no longer use PromptFooter"
        );
        assert!(
            !source.contains("PromptFooterColors"),
            "env render should not reference PromptFooterColors"
        );
    }
}
