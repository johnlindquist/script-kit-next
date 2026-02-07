#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hotkey_config_to_shortcut_string_basic() {
        let config = HotkeyConfig {
            modifiers: vec!["meta".to_string()],
            key: "KeyK".to_string(),
        };
        assert_eq!(config.to_shortcut_string(), "cmd+k");
    }

    #[test]
    fn hotkey_config_to_shortcut_string_multiple_modifiers() {
        let config = HotkeyConfig {
            modifiers: vec!["meta".to_string(), "shift".to_string()],
            key: "KeyV".to_string(),
        };
        assert_eq!(config.to_shortcut_string(), "cmd+shift+v");
    }

    #[test]
    fn hotkey_config_to_shortcut_string_all_modifiers() {
        let config = HotkeyConfig {
            modifiers: vec![
                "alt".to_string(),
                "meta".to_string(),
                "ctrl".to_string(),
                "shift".to_string(),
            ],
            key: "KeyA".to_string(),
        };
        assert_eq!(config.to_shortcut_string(), "alt+cmd+ctrl+shift+a");
    }

    #[test]
    fn hotkey_config_to_shortcut_string_digit_key() {
        let config = HotkeyConfig {
            modifiers: vec!["meta".to_string()],
            key: "Digit0".to_string(),
        };
        assert_eq!(config.to_shortcut_string(), "cmd+0");
    }

    #[test]
    fn hotkey_config_to_shortcut_string_special_key() {
        let config = HotkeyConfig {
            modifiers: vec!["meta".to_string(), "shift".to_string()],
            key: "Space".to_string(),
        };
        assert_eq!(config.to_shortcut_string(), "cmd+shift+space");
    }

    #[test]
    fn hotkey_config_to_shortcut_string_semicolon() {
        let config = HotkeyConfig {
            modifiers: vec!["meta".to_string()],
            key: "Semicolon".to_string(),
        };
        assert_eq!(config.to_shortcut_string(), "cmd+semicolon");
    }

    #[test]
    fn hotkey_config_to_shortcut_string_ctrl_modifier() {
        let config = HotkeyConfig {
            modifiers: vec!["meta".to_string(), "ctrl".to_string()],
            key: "KeyI".to_string(),
        };
        assert_eq!(config.to_shortcut_string(), "cmd+ctrl+i");
    }

    #[test]
    fn hotkey_config_to_shortcut_string_option_alias() {
        // "option" should be treated as "alt"
        let config = HotkeyConfig {
            modifiers: vec!["option".to_string()],
            key: "KeyN".to_string(),
        };
        assert_eq!(config.to_shortcut_string(), "alt+n");
    }

    #[test]
    fn hotkey_config_to_shortcut_string_cmd_alias() {
        // "cmd" should work as well as "meta"
        let config = HotkeyConfig {
            modifiers: vec!["cmd".to_string()],
            key: "KeyJ".to_string(),
        };
        assert_eq!(config.to_shortcut_string(), "cmd+j");
    }
}
