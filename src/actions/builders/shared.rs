use itertools::Itertools;

/// Convert a script name to a deeplink-safe format (lowercase, hyphenated)
///
/// Examples:
/// - "My Script" -> "my-script"
/// - "Clipboard History" -> "clipboard-history"
/// - "hello_world" -> "hello-world"
/// - "Café Script" -> "caf%C3%A9-script"
pub fn to_deeplink_name(name: &str) -> String {
    let normalized = name
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .join("-");

    if normalized.is_empty() {
        return "_unnamed".to_string();
    }

    percent_encode_non_ascii(&normalized)
}

fn percent_encode_non_ascii(input: &str) -> String {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let mut encoded = String::with_capacity(input.len());

    for c in input.chars() {
        if c.is_ascii() {
            encoded.push(c);
            continue;
        }

        let mut utf8_buf = [0u8; 4];
        for byte in c.encode_utf8(&mut utf8_buf).as_bytes() {
            encoded.push('%');
            encoded.push(HEX[(byte >> 4) as usize] as char);
            encoded.push(HEX[(byte & 0x0F) as usize] as char);
        }
    }

    encoded
}

/// Format a shortcut string for display in the UI
/// Converts "cmd+shift+c" to "⌘⇧C"
///
/// Delegates to the shared hint_strip normalizer to prevent mapping drift.
pub(crate) fn format_shortcut_hint(shortcut: &str) -> String {
    let display = crate::components::hint_strip::compact_shortcut_display_string(shortcut);
    crate::components::hint_strip::emit_shortcut_normalization_audit(
        "actions_builders_shared",
        shortcut,
        &display,
    );
    display
}

#[cfg(test)]
mod tests {
    use super::to_deeplink_name;

    #[test]
    fn test_to_deeplink_name_percent_encodes_non_ascii_when_present() {
        assert_eq!(to_deeplink_name("Café Script"), "caf%C3%A9-script");
        assert_eq!(to_deeplink_name("日本語"), "%E6%97%A5%E6%9C%AC%E8%AA%9E");
    }

    #[test]
    fn test_to_deeplink_name_returns_unnamed_when_input_is_empty_or_symbols() {
        assert_eq!(to_deeplink_name(""), "_unnamed");
        assert_eq!(to_deeplink_name("   "), "_unnamed");
        assert_eq!(to_deeplink_name("!@#$%^&*()"), "_unnamed");
    }
}
