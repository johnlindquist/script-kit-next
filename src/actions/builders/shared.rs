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
        .collect::<Vec<_>>()
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
pub(crate) fn format_shortcut_hint(shortcut: &str) -> String {
    let mut result = String::new();
    let parts: Vec<&str> = shortcut.split('+').collect();

    for (i, part) in parts.iter().enumerate() {
        let part_lower = part.trim().to_lowercase();
        let formatted = match part_lower.as_str() {
            "cmd" | "command" | "meta" | "super" => "⌘",
            "ctrl" | "control" => "⌃",
            "alt" | "opt" | "option" => "⌥",
            "shift" => "⇧",
            "enter" | "return" => "↵",
            "escape" | "esc" => "⎋",
            "tab" => "⇥",
            "backspace" | "delete" => "⌫",
            "space" => "␣",
            "up" | "arrowup" => "↑",
            "down" | "arrowdown" => "↓",
            "left" | "arrowleft" => "←",
            "right" | "arrowright" => "→",
            _ => {
                if i == parts.len() - 1 {
                    result.push_str(&part.trim().to_uppercase());
                    continue;
                }
                part.trim()
            }
        };
        result.push_str(formatted);
    }

    result
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
