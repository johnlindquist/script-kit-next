/// Convert a script name to a deeplink-safe format (lowercase, hyphenated)
///
/// Examples:
/// - "My Script" -> "my-script"
/// - "Clipboard History" -> "clipboard-history"
/// - "hello_world" -> "hello-world"
pub fn to_deeplink_name(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
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
