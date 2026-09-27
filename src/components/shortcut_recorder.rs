//! The keys recorded for a shortcut, shown with macOS symbols (⌘⇧K). The shortcut view under the
//! main input records them for a command's shortcut and for a script's `hotkey()`.

/// Represents a recorded keyboard shortcut
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RecordedShortcut {
    /// Command key (macOS) / Super key
    pub cmd: bool,
    /// Control key
    pub ctrl: bool,
    /// Option/Alt key
    pub alt: bool,
    /// Shift key
    pub shift: bool,
    /// The actual key pressed (single character or key name)
    pub key: Option<String>,
}

impl RecordedShortcut {
    /// Create a new empty shortcut
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if the shortcut is complete (has modifiers + key)
    pub fn is_complete(&self) -> bool {
        (self.cmd || self.ctrl || self.alt || self.shift) && self.key.is_some()
    }

    /// Format as a display string using macOS symbols
    pub fn to_display_string(&self) -> String {
        let mut parts = Vec::new();

        if self.ctrl {
            parts.push("⌃");
        }
        if self.alt {
            parts.push("⌥");
        }
        if self.cmd {
            parts.push("⌘");
        }
        if self.shift {
            parts.push("⇧");
        }

        if let Some(ref key) = self.key {
            parts.push(key.as_str());
        }

        parts.join("")
    }

    /// Format as a config string (e.g., "cmd+shift+k")
    pub fn to_config_string(&self) -> String {
        let mut parts = Vec::new();

        if self.ctrl {
            parts.push("ctrl".to_string());
        }
        if self.alt {
            parts.push("alt".to_string());
        }
        if self.cmd {
            parts.push("cmd".to_string());
        }
        if self.shift {
            parts.push("shift".to_string());
        }

        if let Some(ref key) = self.key {
            parts.push(key.to_lowercase());
        }

        parts.join("+")
    }

    /// Get individual keycaps for display through the shared shortcut parser.
    pub fn to_keycaps(&self) -> Vec<String> {
        crate::components::hint_strip::shortcut_tokens_from_hint(&self.to_config_string())
    }

    /// Format as the transient SDK `HotkeyInfo` payload. Scripts have always received the key
    /// upper-cased, in `key`, `keyCode` and `shortcut` alike.
    pub fn to_hotkey_info_json(&self) -> String {
        let captured = Self {
            key: self.key.as_deref().map(str::to_uppercase),
            ..self.clone()
        };
        let key = captured.key.clone().unwrap_or_default();
        serde_json::json!({
            "key": key,
            "command": captured.cmd,
            "shift": captured.shift,
            "option": captured.alt,
            "control": captured.ctrl,
            "shortcut": captured.to_display_string(),
            "keyCode": key,
        })
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::RecordedShortcut;

    fn cmd_shift(key: &str) -> RecordedShortcut {
        RecordedShortcut {
            cmd: true,
            shift: true,
            key: Some(key.to_string()),
            ..RecordedShortcut::default()
        }
    }

    #[test]
    fn test_recorded_shortcut_to_display_string() {
        assert_eq!(cmd_shift("K").to_display_string(), "⌘⇧K");
    }

    #[test]
    fn test_recorded_shortcut_to_config_string() {
        assert_eq!(cmd_shift("K").to_config_string(), "cmd+shift+k");
    }

    #[test]
    fn test_recorded_shortcut_is_complete() {
        let mut shortcut = RecordedShortcut {
            cmd: true,
            ..RecordedShortcut::default()
        };
        assert!(!shortcut.is_complete());
        shortcut.key = Some("K".to_string());
        assert!(shortcut.is_complete());
    }

    #[test]
    fn test_recorded_shortcut_to_keycaps() {
        let shortcut = RecordedShortcut {
            ctrl: true,
            alt: true,
            ..cmd_shift("K")
        };
        assert_eq!(shortcut.to_keycaps(), vec!["⌃", "⌥", "⌘", "⇧", "K"]);
    }

    #[test]
    fn hotkey_info_is_the_same_payload_whatever_case_the_key_arrived_in() {
        let expected = r#"{"key":"K","command":true,"shift":true,"option":false,"control":false,"shortcut":"⌘⇧K","keyCode":"K"}"#;
        assert_eq!(cmd_shift("k").to_hotkey_info_json(), expected);
        assert_eq!(cmd_shift("K").to_hotkey_info_json(), expected);
    }
}
