//! Regression tests for webcam actions dialog consistency.
//!
//! Ensures webcam uses the same shared ActionsDialog behavior path as the
//! main actions window: native Action rows with shared filtering/navigation
//! instead of ad-hoc SDK action injection.

#[cfg(test)]
mod tests {
    use std::fs;

    fn webcam_toggle_section(content: &str) -> &str {
        let start = content
            .find("pub fn toggle_webcam_actions")
            .expect("toggle_webcam_actions not found");
        let after_start = &content[start..];
        let end = after_start
            .find("/// Toggle terminal command bar")
            .unwrap_or(after_start.len());
        &content[start..start + end]
    }

    fn webcam_actions_builder_section(content: &str) -> &str {
        let start = content
            .find("fn webcam_actions_for_dialog")
            .expect("webcam_actions_for_dialog not found");
        let after_start = &content[start..];
        let end = after_start
            .find("fn execute_webcam_action")
            .unwrap_or(after_start.len());
        &content[start..start + end]
    }

    #[test]
    fn webcam_actions_use_shared_native_actions_dialog_path() {
        let content = fs::read_to_string("src/app_impl.rs").expect("Failed to read app_impl.rs");
        let section = webcam_toggle_section(&content);

        assert!(
            section.contains("ActionsDialog::with_config"),
            "Webcam actions should use shared native ActionsDialog rows via with_config()."
        );

        assert!(
            !section.contains("set_sdk_actions"),
            "Webcam actions should not inject SDK actions; this diverges from native main dialog behavior."
        );
    }

    #[test]
    fn webcam_actions_keep_stable_capture_and_close_ids() {
        let content = fs::read_to_string("src/app_impl.rs").expect("Failed to read app_impl.rs");
        let section = webcam_actions_builder_section(&content);

        assert!(
            section.contains("\"capture\""),
            "Webcam actions should define a 'capture' action id."
        );

        assert!(
            section.contains("\"close\""),
            "Webcam actions should define a 'close' action id."
        );
    }
}
