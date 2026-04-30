//! Shared quote helper for filter-expression value formatting.
//!
//! Two places in the codebase emit `key:"phrase"` syntax into a filter
//! expression: [[crate::menu_syntax::action_effects::apply_safe_effect]]
//! (`DefaultTime` arm — Pass 29) and
//! [[crate::menu_syntax_ai_apply::apply_proposal]] (`AddDate` arm — Pass 35).
//! Both originally used `format!("…\"{phrase}\"")` with no escaping, so a
//! phrase containing `"` produced unbalanced output that downstream
//! parsers mis-tokenized (Pass 32 `[?]` and Pass 36 `[?]`).
//!
//! Pass 41 closes both `[?]`s by routing both arms through the same
//! escaping helper. The function returns the value WITH surrounding
//! quotes so callers don't accidentally drop them when refactoring.

/// Wrap `s` in double quotes, backslash-escaping any `\` and `"` inside.
/// Backslashes are escaped FIRST so a literal `\` doesn't get turned into
/// the start of an escape sequence by the subsequent `"` pass.
///
/// Examples:
/// - `today 9am` → `"today 9am"`
/// - `today "9am"` → `"today \"9am\""`
/// - `path\to\file` → `"path\\to\\file"`
/// - `mix \" and "x` → `"mix \\\" and \"x"`
pub fn quote_for_filter_value(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            other => out.push(other),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quotes_a_clean_phrase_with_surrounding_double_quotes() {
        assert_eq!(quote_for_filter_value("today 9am"), "\"today 9am\"");
    }

    #[test]
    fn escapes_internal_double_quotes() {
        assert_eq!(
            quote_for_filter_value(r#"today "9am""#),
            r#""today \"9am\"""#
        );
    }

    #[test]
    fn escapes_backslashes_before_quotes() {
        // Backslash escaping must run FIRST so a literal `\` doesn't
        // become the start of `\"` after the quote-escape pass.
        assert_eq!(
            quote_for_filter_value(r"path\to\file"),
            r#""path\\to\\file""#
        );
    }

    #[test]
    fn handles_mixed_backslash_and_quote() {
        // `mix \" and "x` should round-trip to `"mix \\\" and \"x"`.
        // Verifies the order: backslash first, then quote.
        assert_eq!(
            quote_for_filter_value(r#"mix \" and "x"#),
            r#""mix \\\" and \"x""#
        );
    }

    #[test]
    fn empty_phrase_yields_empty_quoted_string() {
        assert_eq!(quote_for_filter_value(""), "\"\"");
    }

    #[test]
    fn preserves_unicode_passthrough() {
        assert_eq!(
            quote_for_filter_value("café \u{1F680}"),
            "\"café \u{1F680}\""
        );
    }

    #[test]
    fn output_has_balanced_unescaped_quotes_falsifier() {
        // Falsifier for a future regression that drops the escape: count
        // the UNESCAPED `"` chars (i.e. not preceded by `\`) — must be
        // exactly 2 (the opening and closing wrappers).
        let input = r#"today "9am""#;
        let out = quote_for_filter_value(input);
        let mut unescaped = 0;
        let mut prev_backslash = false;
        for c in out.chars() {
            if c == '"' && !prev_backslash {
                unescaped += 1;
            }
            prev_backslash = c == '\\' && !prev_backslash;
        }
        assert_eq!(
            unescaped, 2,
            "output {out:?} should have exactly 2 unescaped quotes"
        );
    }
}
