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

    fn webcam_execute_section(content: &str) -> &str {
        let start = content
            .find("fn execute_webcam_action")
            .expect("execute_webcam_action not found");
        let after_start = &content[start..];
        let end = after_start
            .find("// ========================================================================")
            .unwrap_or(after_start.len());
        &content[start..start + end]
    }

    fn webcam_open_section(content: &str) -> &str {
        let start = content
            .find("fn open_webcam")
            .expect("open_webcam not found");
        let after_start = &content[start..];
        let end = after_start
            .find("/// Handle builtin confirmation modal result.")
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

        assert!(
            section.contains("\"Capture Photo\""),
            "Webcam actions should label the primary action as 'Capture Photo'."
        );
    }

    #[test]
    fn webcam_capture_action_saves_photo_and_shows_feedback() {
        let content = fs::read_to_string("src/app_impl.rs").expect("Failed to read app_impl.rs");
        let section = webcam_execute_section(&content);

        assert!(
            section.contains("capture_webcam_photo"),
            "Webcam capture action should call a dedicated photo capture/save helper."
        );
        assert!(
            content.contains("Photo saved to"),
            "Webcam capture action should provide explicit save-path HUD feedback."
        );
    }

    #[test]
    fn render_webcam_footer_primary_uses_capture_flow() {
        let content =
            fs::read_to_string("src/render_prompts/other.rs").expect("Failed to read render file");

        let start = content
            .find("render_webcam_prompt(")
            .expect("render_webcam_prompt not found");
        let section = &content[start..];

        assert!(
            section.contains("capture_webcam_photo"),
            "Webcam footer primary action should use capture flow instead of only closing."
        );
        assert!(
            section.contains(".primary_label(\"Capture Photo\")"),
            "Webcam footer primary action label should be 'Capture Photo'."
        );
    }

    #[test]
    fn webcam_start_errors_are_surfaceable_in_open_flow() {
        let content = fs::read_to_string("src/app_execute.rs").expect("Failed to read app_execute");
        let section = webcam_open_section(&content);

        assert!(
            section.contains("Failed to start webcam"),
            "open_webcam should log startup failures with a clear message."
        );
        assert!(
            section.contains("prompt.set_error(err_msg, cx)"),
            "open_webcam should surface startup failures to the webcam prompt error state."
        );
    }

    #[test]
    fn webcam_camera_module_uses_typed_startup_error_taxonomy() {
        let content = fs::read_to_string("src/camera.rs").expect("Failed to read camera.rs");

        assert!(
            content.contains("pub enum WebcamStartError"),
            "camera.rs should define a typed WebcamStartError enum."
        );
        assert!(
            content.contains("PermissionDenied")
                && content.contains("DeviceBusy")
                && content.contains("InputInitFailed"),
            "camera.rs should classify permission, busy, and generic input startup failures."
        );
        assert!(
            content.contains("-> std::result::Result<(mpsc::Receiver<CVPixelBuffer>, CaptureHandle), WebcamStartError>"),
            "start_capture should return typed webcam startup errors."
        );
    }
}
