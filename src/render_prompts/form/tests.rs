mod form_prompt_render_tests {
    use super::*;

    #[test]
    fn form_enter_behavior_submits_non_textarea_on_enter() {
        assert_eq!(
            form_enter_behavior("enter", false, false),
            FormEnterBehavior::Submit
        );
    }

    #[test]
    fn form_enter_behavior_forwards_textarea_enter_without_cmd() {
        assert_eq!(
            form_enter_behavior("enter", false, true),
            FormEnterBehavior::ForwardToField
        );
    }

    #[test]
    fn form_enter_behavior_submits_textarea_on_cmd_enter() {
        assert_eq!(
            form_enter_behavior("enter", true, true),
            FormEnterBehavior::Submit
        );
    }

    #[test]
    fn form_footer_status_text_mentions_cmd_enter_for_textarea() {
        assert_eq!(
            form_footer_status_text(true),
            running_status_text("press ⌘↵ to submit (Enter adds a new line)")
        );
    }

    #[test]
    fn form_footer_status_text_mentions_enter_for_non_textarea() {
        assert_eq!(
            form_footer_status_text(false),
            running_status_text("press Enter to submit")
        );
    }

    #[test]
    fn form_field_value_is_valid_for_submit_accepts_common_valid_inputs() {
        assert!(form_field_value_is_valid_for_submit(
            Some("email"),
            "user@example.com"
        ));
        assert!(form_field_value_is_valid_for_submit(Some("number"), "42.5"));
        assert!(form_field_value_is_valid_for_submit(Some("number"), ""));
    }

    #[test]
    fn form_field_value_is_valid_for_submit_rejects_invalid_email_and_number() {
        assert!(!form_field_value_is_valid_for_submit(
            Some("email"),
            "invalid-email"
        ));
        assert!(!form_field_value_is_valid_for_submit(
            Some("number"),
            "12abc"
        ));
    }
}
