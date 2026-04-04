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
    fn env_prompt_uses_whisper_chrome_field_surface() {
        let source = include_str!("render.rs");
        let render_fn_end = source.find("#[cfg(test)]").unwrap_or(source.len());
        let render_code = &source[..render_fn_end];
        assert!(
            !render_code.contains("InlinePromptInput::new("),
            "env render should not use InlinePromptInput (migrated to whisper chrome field)"
        );
        assert!(
            !render_code.contains(".size(px(64.))"),
            "env render should not have a 64px hero icon tile"
        );
        assert!(
            render_code.contains("field_bg"),
            "env render should use a whisper-chrome field background"
        );
        assert!(
            render_code.contains("field_border"),
            "env render should use a whisper-chrome field border"
        );
    }

    #[test]
    fn env_prompt_render_delegates_footer_to_wrapper() {
        let source = include_str!("render.rs");
        // Scope to render code only (exclude test assertions that mention the same strings)
        let render_fn_end = source.find("#[cfg(test)]").unwrap_or(source.len());
        let render_code = &source[..render_fn_end];
        // Footer is owned by the outer wrapper shell (render_prompts::other.rs),
        // so the inner prompt must NOT render its own footer.
        assert!(
            !render_code.contains("render_simple_hint_strip("),
            "env render should not render its own hint strip (wrapper owns the footer)"
        );
        assert!(
            !render_code.contains("PromptFooter::new("),
            "env render should not use PromptFooter"
        );
        assert!(
            !render_code.contains("PromptFooterColors"),
            "env render should not reference PromptFooterColors"
        );
        // Storage hint text still appears in the prompt body (inline context).
        assert!(
            render_code.contains("env_storage_hint_text("),
            "env render should still show storage hint text in the body"
        );
    }
}
