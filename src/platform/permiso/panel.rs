#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum PermisoPanel {
    Accessibility,
    ScreenRecording,
}

impl PermisoPanel {
    pub fn settings_url(self) -> &'static str {
        match self {
            Self::Accessibility => {
                "x-apple.systempreferences:com.apple.settings.PrivacySecurity.extension?Privacy_Accessibility"
            }
            Self::ScreenRecording => {
                "x-apple.systempreferences:com.apple.settings.PrivacySecurity.extension?Privacy_ScreenCapture"
            }
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::Accessibility => "Accessibility",
            Self::ScreenRecording => "Screen Recording",
        }
    }

    pub fn receipt_name(self) -> &'static str {
        match self {
            Self::Accessibility => "accessibility",
            Self::ScreenRecording => "screenRecording",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PermisoPanel;

    #[test]
    fn settings_url_targets_accessibility_privacy_pane() {
        assert_eq!(
            PermisoPanel::Accessibility.settings_url(),
            "x-apple.systempreferences:com.apple.settings.PrivacySecurity.extension?Privacy_Accessibility"
        );
    }

    #[test]
    fn settings_url_targets_screen_recording_privacy_pane() {
        assert_eq!(
            PermisoPanel::ScreenRecording.settings_url(),
            "x-apple.systempreferences:com.apple.settings.PrivacySecurity.extension?Privacy_ScreenCapture"
        );
    }

    #[test]
    fn display_names_match_system_settings_labels() {
        assert_eq!(PermisoPanel::Accessibility.display_name(), "Accessibility");
        assert_eq!(
            PermisoPanel::ScreenRecording.display_name(),
            "Screen Recording"
        );
    }
}
