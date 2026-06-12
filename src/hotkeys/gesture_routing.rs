//! Pure helpers for main-hotkey gesture routing (carry-over text, surface toggles).

/// Merge launcher query text into day-page editor content as the start of a capture.
pub fn merge_launcher_query_into_day_page_content(existing: &str, query: &str) -> String {
    let query = query.trim();
    if query.is_empty() {
        return existing.to_string();
    }
    if existing.trim().is_empty() {
        query.to_string()
    } else if existing.ends_with('\n') {
        format!("{existing}{query}")
    } else {
        format!("{existing}\n{query}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn carry_over_into_empty_day_page() {
        assert_eq!(
            merge_launcher_query_into_day_page_content("", "buy milk"),
            "buy milk"
        );
    }

    #[test]
    fn carry_over_appends_after_existing_content() {
        assert_eq!(
            merge_launcher_query_into_day_page_content("09:00 — note", "buy milk"),
            "09:00 — note\nbuy milk"
        );
    }

    #[test]
    fn carry_over_uses_existing_empty_end_line() {
        assert_eq!(
            merge_launcher_query_into_day_page_content("09:00 — note\n\n", "buy milk"),
            "09:00 — note\n\nbuy milk"
        );
    }

    #[test]
    fn empty_query_leaves_content_unchanged() {
        assert_eq!(
            merge_launcher_query_into_day_page_content("keep me", "   "),
            "keep me"
        );
    }
}
