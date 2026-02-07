    use super::*;
    // ========================================
    // ExtensionManifest Tests
    // ========================================

    #[test]
    fn test_extension_manifest_default() {
        let manifest = ExtensionManifest::default();
        assert_eq!(manifest.name, "");
        // Note: Default::default() doesn't trigger serde defaults
        // The "MIT" default only applies during deserialization
        assert!(manifest.categories.is_empty());
        assert!(manifest.platforms.is_empty());
    }
    #[test]
    fn test_extension_manifest_license_default_on_deserialize() {
        // When deserializing without a license field, it defaults to MIT
        let yaml = "name: test";
        let manifest: ExtensionManifest = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(manifest.license, "MIT");
    }
    #[test]
    fn test_extension_manifest_parse_yaml() {
        let yaml = r#"
name: cleanshot
title: CleanShot X
description: Capture screenshots
icon: camera
author: scriptkit
license: MIT
categories:
  - Productivity
  - Media
platforms:
  - macOS
keywords:
  - screenshot
  - capture
"#;
        let manifest: ExtensionManifest = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(manifest.name, "cleanshot");
        assert_eq!(manifest.title, "CleanShot X");
        assert_eq!(manifest.description, "Capture screenshots");
        assert_eq!(manifest.icon, "camera");
        assert_eq!(manifest.author, "scriptkit");
        assert_eq!(manifest.license, "MIT");
        assert_eq!(manifest.categories, vec!["Productivity", "Media"]);
        assert_eq!(manifest.platforms, vec!["macOS"]);
        assert_eq!(manifest.keywords, vec!["screenshot", "capture"]);
    }
    #[test]
    fn test_extension_manifest_with_preferences() {
        let yaml = r#"
name: chrome
title: Google Chrome
preferences:
  - name: profile
    title: Chrome Profile
    description: Which profile to use
    type: dropdown
    required: false
    data:
      - title: Default
        value: Default
      - title: Work
        value: "Profile 1"
"#;
        let manifest: ExtensionManifest = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(manifest.preferences.len(), 1);
        let pref = &manifest.preferences[0];
        assert_eq!(pref.name, "profile");
        assert_eq!(pref.pref_type, PreferenceType::Dropdown);
        assert!(!pref.required);
        assert_eq!(pref.data.len(), 2);
        assert_eq!(pref.data[0].title, "Default");
        assert_eq!(pref.data[1].value, "Profile 1");
    }
    #[test]
    fn test_extension_manifest_supports_macos() {
        let empty_platforms = ExtensionManifest::default();
        assert!(empty_platforms.supports_macos());

        let macos_only: ExtensionManifest = serde_yaml::from_str("platforms: [macOS]").unwrap();
        assert!(macos_only.supports_macos());

        let windows_only: ExtensionManifest = serde_yaml::from_str("platforms: [Windows]").unwrap();
        assert!(!windows_only.supports_macos());

        let both: ExtensionManifest = serde_yaml::from_str("platforms: [macOS, Windows]").unwrap();
        assert!(both.supports_macos());
    }
    #[test]
    fn test_extension_manifest_validate_categories() {
        let valid: ExtensionManifest =
            serde_yaml::from_str("categories: [Productivity, Media]").unwrap();
        assert!(valid.validate_categories().is_ok());

        let invalid: ExtensionManifest =
            serde_yaml::from_str("categories: [Productivity, InvalidCategory]").unwrap();
        let err = invalid.validate_categories().unwrap_err();
        assert_eq!(err, vec!["InvalidCategory"]);
    }
    #[test]
    fn test_extension_manifest_min_version() {
        let yaml = r#"
name: test
minVersion: "2.0.0"
"#;
        let manifest: ExtensionManifest = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(manifest.min_version, Some("2.0.0".to_string()));
    }
    #[test]
    fn test_extension_manifest_min_version_alias() {
        let yaml = r#"
name: test
min_version: "2.0.0"
"#;
        let manifest: ExtensionManifest = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(manifest.min_version, Some("2.0.0".to_string()));
    }
    #[test]
    fn test_extension_manifest_preserves_extra_fields() {
        let yaml = r#"
name: test
unknownField: some value
anotherField: 123
"#;
        let manifest: ExtensionManifest = serde_yaml::from_str(yaml).unwrap();
        assert!(manifest.extra.contains_key("unknownField"));
        assert!(manifest.extra.contains_key("anotherField"));
    }
    // ========================================
    // CommandMetadata Tests
    // ========================================

    #[test]
    fn test_command_metadata_default() {
        let meta = CommandMetadata::default();
        assert_eq!(meta.mode, CommandMode::View);
        assert!(meta.keywords.is_empty());
        assert!(!meta.disabled_by_default);
        assert!(!meta.hidden);
    }
    #[test]
    fn test_command_metadata_parse_json() {
        let json = r#"{
            "description": "Capture a selected area",
            "mode": "no-view",
            "keywords": ["screenshot", "area"],
            "shortcut": "cmd shift 4"
        }"#;
        let meta: CommandMetadata = serde_json::from_str(json).unwrap();
        assert_eq!(
            meta.description,
            Some("Capture a selected area".to_string())
        );
        assert_eq!(meta.mode, CommandMode::NoView);
        assert_eq!(meta.keywords, vec!["screenshot", "area"]);
        assert_eq!(meta.shortcut, Some("cmd shift 4".to_string()));
    }
    #[test]
    fn test_command_metadata_with_arguments() {
        let json = r#"{
            "description": "Search for text",
            "arguments": [
                {
                    "name": "query",
                    "type": "text",
                    "placeholder": "Enter search term",
                    "required": true
                }
            ]
        }"#;
        let meta: CommandMetadata = serde_json::from_str(json).unwrap();
        assert_eq!(meta.arguments.len(), 1);
        let arg = &meta.arguments[0];
        assert_eq!(arg.name, "query");
        assert_eq!(arg.arg_type, ArgumentType::Text);
        assert_eq!(arg.placeholder, "Enter search term");
        assert!(arg.required);
    }
    #[test]
    fn test_command_metadata_with_interval() {
        let json = r#"{
            "description": "Background task",
            "mode": "no-view",
            "interval": "1h"
        }"#;
        let meta: CommandMetadata = serde_json::from_str(json).unwrap();
        assert_eq!(meta.interval, Some("1h".to_string()));
        assert_eq!(meta.mode, CommandMode::NoView);
    }
    #[test]
    fn test_command_mode_serialization() {
        assert_eq!(
            serde_json::to_string(&CommandMode::View).unwrap(),
            "\"view\""
        );
        assert_eq!(
            serde_json::to_string(&CommandMode::NoView).unwrap(),
            "\"no-view\""
        );
        assert_eq!(
            serde_json::to_string(&CommandMode::MenuBar).unwrap(),
            "\"menu-bar\""
        );
    }
    #[test]
    fn test_command_mode_deserialization() {
        assert_eq!(
            serde_json::from_str::<CommandMode>("\"view\"").unwrap(),
            CommandMode::View
        );
        assert_eq!(
            serde_json::from_str::<CommandMode>("\"no-view\"").unwrap(),
            CommandMode::NoView
        );
        assert_eq!(
            serde_json::from_str::<CommandMode>("\"menu-bar\"").unwrap(),
            CommandMode::MenuBar
        );
    }
    // ========================================
    // Preference Tests
    // ========================================

    #[test]
    fn test_preference_type_serialization() {
        assert_eq!(
            serde_json::to_string(&PreferenceType::Textfield).unwrap(),
            "\"textfield\""
        );
        assert_eq!(
            serde_json::to_string(&PreferenceType::Password).unwrap(),
            "\"password\""
        );
        assert_eq!(
            serde_json::to_string(&PreferenceType::AppPicker).unwrap(),
            "\"appPicker\""
        );
    }
    #[test]
    fn test_preference_parsing() {
        let json = r#"{
            "name": "apiKey",
            "title": "API Key",
            "description": "Your API key",
            "type": "password",
            "required": true
        }"#;
        let pref: Preference = serde_json::from_str(json).unwrap();
        assert_eq!(pref.name, "apiKey");
        assert_eq!(pref.pref_type, PreferenceType::Password);
        assert!(pref.required);
    }
    #[test]
    fn test_preference_with_default() {
        let json = r#"{
            "name": "maxResults",
            "title": "Max Results",
            "description": "Maximum results to show",
            "type": "textfield",
            "required": false,
            "default": 10
        }"#;
        let pref: Preference = serde_json::from_str(json).unwrap();
        assert_eq!(pref.default, Some(serde_json::json!(10)));
    }
    // ========================================
    // Argument Tests
    // ========================================

    #[test]
    fn test_argument_type_serialization() {
        assert_eq!(
            serde_json::to_string(&ArgumentType::Text).unwrap(),
            "\"text\""
        );
        assert_eq!(
            serde_json::to_string(&ArgumentType::Password).unwrap(),
            "\"password\""
        );
        assert_eq!(
            serde_json::to_string(&ArgumentType::Dropdown).unwrap(),
            "\"dropdown\""
        );
    }
    #[test]
    fn test_argument_with_dropdown() {
        let json = r#"{
            "name": "priority",
            "type": "dropdown",
            "placeholder": "Select priority",
            "required": true,
            "data": [
                {"title": "High", "value": "high"},
                {"title": "Medium", "value": "medium"},
                {"title": "Low", "value": "low"}
            ]
        }"#;
        let arg: Argument = serde_json::from_str(json).unwrap();
        assert_eq!(arg.name, "priority");
        assert_eq!(arg.arg_type, ArgumentType::Dropdown);
        assert_eq!(arg.data.len(), 3);
        assert_eq!(arg.data[0].value, "high");
    }
    // ========================================
    // Icon Resolution Tests
    // ========================================

    #[test]
    fn test_resolve_icon_named() {
        assert_eq!(
            resolve_icon("camera"),
            IconSource::Named("camera".to_string())
        );
        assert_eq!(resolve_icon("star"), IconSource::Named("star".to_string()));
        assert_eq!(
            resolve_icon("file-code"),
            IconSource::Named("file-code".to_string())
        );
    }
    #[test]
    fn test_resolve_icon_path() {
        assert_eq!(
            resolve_icon("./icon.png"),
            IconSource::Path("./icon.png".to_string())
        );
        assert_eq!(
            resolve_icon("/path/to/icon.png"),
            IconSource::Path("/path/to/icon.png".to_string())
        );
        assert_eq!(
            resolve_icon("../assets/icon.svg"),
            IconSource::Path("../assets/icon.svg".to_string())
        );
        assert_eq!(
            resolve_icon("assets/icon.png"),
            IconSource::Path("assets/icon.png".to_string())
        );
        assert_eq!(
            resolve_icon("icon.png"),
            IconSource::Path("icon.png".to_string())
        );
        assert_eq!(
            resolve_icon("icon.icns"),
            IconSource::Path("icon.icns".to_string())
        );
    }
    // ========================================
    // Version Checking Tests
    // ========================================

    #[test]
    fn test_check_min_version_satisfied() {
        assert!(check_min_version("1.0.0", "1.0.0").is_ok());
        assert!(check_min_version("1.0.0", "1.0.1").is_ok());
        assert!(check_min_version("1.0.0", "1.1.0").is_ok());
        assert!(check_min_version("1.0.0", "2.0.0").is_ok());
        assert!(check_min_version("1.5", "1.6.0").is_ok());
    }
    #[test]
    fn test_check_min_version_not_satisfied() {
        let result = check_min_version("2.0.0", "1.9.9");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("requires Script Kit 2.0.0"));
    }
    #[test]
    fn test_check_min_version_with_v_prefix() {
        assert!(check_min_version("v1.0.0", "v1.0.0").is_ok());
        assert!(check_min_version("1.0.0", "v1.0.1").is_ok());
        assert!(check_min_version("v1.0.0", "1.0.1").is_ok());
    }
    #[test]
    fn test_check_min_version_invalid() {
        assert!(check_min_version("invalid", "1.0.0").is_err());
        assert!(check_min_version("1.0.0", "invalid").is_err());
    }
    // ========================================
    // Valid Categories Tests
    // ========================================

    #[test]
    fn test_valid_categories_contains_all() {
        assert!(VALID_CATEGORIES.contains(&"Applications"));
        assert!(VALID_CATEGORIES.contains(&"Communication"));
        assert!(VALID_CATEGORIES.contains(&"Data"));
        assert!(VALID_CATEGORIES.contains(&"Design Tools"));
        assert!(VALID_CATEGORIES.contains(&"Developer Tools"));
        assert!(VALID_CATEGORIES.contains(&"Documentation"));
        assert!(VALID_CATEGORIES.contains(&"Finance"));
        assert!(VALID_CATEGORIES.contains(&"Fun"));
        assert!(VALID_CATEGORIES.contains(&"Media"));
        assert!(VALID_CATEGORIES.contains(&"News"));
        assert!(VALID_CATEGORIES.contains(&"Productivity"));
        assert!(VALID_CATEGORIES.contains(&"Security"));
        assert!(VALID_CATEGORIES.contains(&"System"));
        assert!(VALID_CATEGORIES.contains(&"Web"));
        assert!(VALID_CATEGORIES.contains(&"Other"));
        assert_eq!(VALID_CATEGORIES.len(), 15);
    }
    // ========================================
    // Command Tests
    // ========================================

    #[test]
    fn test_command_new() {
        let cmd = Command::new(
            "Hello World".to_string(),
            "bash".to_string(),
            "echo 'hello'".to_string(),
        );
        assert_eq!(cmd.name, "Hello World");
        assert_eq!(cmd.command, "hello-world");
        assert_eq!(cmd.tool, "bash");
        assert_eq!(cmd.content, "echo 'hello'");
        assert!(cmd.inputs.is_empty());
    }
