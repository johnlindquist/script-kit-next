pub const EXCLUDED_BUNDLE_IDS: &[&str] = &[
    "com.1password.1password",
    "com.agilebits.onepassword7",
    "com.bitwarden.desktop",
    "com.lastpass.LastPass",
    "org.keepassxc.keepassxc",
    "com.dashlane.Dashlane",
];

pub fn should_exclude_clipboard(source_bundle_id: &str) -> bool {
    EXCLUDED_BUNDLE_IDS
        .iter()
        .any(|excluded| source_bundle_id.starts_with(excluded))
}

#[cfg(test)]
mod tests {
    use super::{should_exclude_clipboard, EXCLUDED_BUNDLE_IDS};

    #[test]
    fn test_should_exclude_clipboard_does_match_when_bundle_id_is_exact() {
        for excluded in EXCLUDED_BUNDLE_IDS {
            assert!(
                should_exclude_clipboard(excluded),
                "{excluded} should match"
            );
        }
    }

    #[test]
    fn test_should_exclude_clipboard_does_match_when_bundle_id_has_excluded_prefix() {
        assert!(should_exclude_clipboard(
            "com.1password.1password.browser-helper"
        ));
        assert!(should_exclude_clipboard("com.bitwarden.desktop.autofill"));
    }

    #[test]
    fn test_should_exclude_clipboard_does_not_match_when_bundle_id_is_not_excluded() {
        assert!(!should_exclude_clipboard("com.apple.TextEdit"));
        assert!(!should_exclude_clipboard("io.github.clipboard-tool"));
        assert!(!should_exclude_clipboard(""));
    }
}
