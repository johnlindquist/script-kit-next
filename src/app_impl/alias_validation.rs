//! What the launcher accepts as a command alias.
//!
//! Pure rule shared by the alias editor's field (which disables Save and
//! explains the problem) and the save path. The lib crate carries this file
//! through `#[path]` so its tests run under `cargo test --lib`.

/// Longest alias the launcher accepts.
pub const ALIAS_MAX_LENGTH: usize = 32;

/// Why `text` cannot be saved as an alias, or `None` when it can. Leading
/// and trailing whitespace is ignored; the alias is the trimmed text.
pub fn alias_problem(text: &str) -> Option<String> {
    let alias = text.trim();
    if alias.is_empty() {
        return Some("Alias is required".into());
    }
    if alias.chars().any(char::is_whitespace) {
        return Some("Alias cannot contain spaces".into());
    }
    if !alias
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return Some("Use only letters, numbers, hyphens, or underscores".into());
    }
    if alias.chars().count() > ALIAS_MAX_LENGTH {
        return Some(format!(
            "Alias must be {ALIAS_MAX_LENGTH} characters or fewer"
        ));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{alias_problem, ALIAS_MAX_LENGTH};

    #[test]
    fn accepts_trimmed_words_with_hyphens_and_underscores() {
        for alias in ["clip", "  ch  ", "todo-list", "my_alias2", "é"] {
            assert_eq!(alias_problem(alias), None, "{alias:?}");
        }
    }

    #[test]
    fn rejects_empty_spaced_symbolic_and_overlong_aliases() {
        let too_long = "x".repeat(ALIAS_MAX_LENGTH + 1);
        for alias in [
            "",
            "   ",
            "two words",
            "tab\there",
            "a.b",
            "clip!",
            too_long.as_str(),
        ] {
            assert!(alias_problem(alias).is_some(), "{alias:?}");
        }
        assert_eq!(
            alias_problem(&"x".repeat(ALIAS_MAX_LENGTH)),
            None,
            "the limit itself is allowed"
        );
    }
}
