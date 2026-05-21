//! Source contracts for Clipboard History `type:` filters and bottom preview info.
//!
//! Runtime DevTools receipts prove these paths against the live app. These
//! contracts pin the implementation shape so the renderer does not drift back
//! to duplicated text-only filtering or top-flow preview metadata.

const CLIPBOARD: &str = include_str!("../src/render_builtins/clipboard.rs");
const CLIPBOARD_PREVIEW: &str = include_str!("../src/render_builtins/clipboard_preview.rs");

#[test]
fn clipboard_history_type_filters_have_local_content_type_parser() {
    let production = CLIPBOARD
        .split("#[cfg(test)]")
        .next()
        .expect("clipboard source should contain production section");

    for literal in [
        "\"type:\"",
        "\"kind:\"",
        "\"texts\"",
        "\"images\"",
        "\"urls\"",
        "\"files\"",
        "\"colors\"",
        "clipboard_history_visible_rows_for_entries",
        "ClipboardHistoryFilterQuery::parse(filter)",
    ] {
        assert!(
            production.contains(literal),
            "clipboard history filter source must contain {literal}"
        );
    }
}

#[test]
fn clipboard_history_filtering_stays_centralized() {
    let production = CLIPBOARD
        .split("#[cfg(test)]")
        .next()
        .expect("clipboard source should contain production section");

    assert!(
        !production.contains("let entry_matches_filter"),
        "wheel scrolling must not reintroduce a local filter closure"
    );
    assert_eq!(
        production.matches("entry_text_matches(entry,").count(),
        1,
        "text/OCR substring matching should live only in the central query matcher"
    );
    assert!(
        production.contains("let filtered_entries = self.clipboard_history_visible_rows(&filter);"),
        "render path must use the shared visible-row helper"
    );
    assert!(
        production.contains(
            "let filtered_entries = this.clipboard_history_visible_rows(&current_filter);"
        ),
        "keyboard and wheel paths must use the shared visible-row helper"
    );
}

#[test]
fn clipboard_preview_has_flexible_content_then_bottom_information() {
    let production = CLIPBOARD_PREVIEW
        .split("#[cfg(test)]")
        .next()
        .expect("clipboard preview source should contain production section");

    let info_pos = production
        .find(".id(\"clipboard-preview-information\")")
        .expect("preview must define bottom information block");
    let content_pos = production
        .find(".id(\"clipboard-preview-content-area\")")
        .expect("preview must define flexible content area");
    let append_pos = production
        .find("panel = panel.child(content_area).child(information);")
        .expect("preview must append information after content area");

    assert!(
        info_pos < content_pos,
        "information is assembled independently before content branches"
    );
    assert!(
        content_pos < append_pos,
        "content area must be constructed before final bottom-info append"
    );
    assert!(
        production.contains(".flex_1()") && production.contains(".flex_none()"),
        "preview must keep content flexible and information natural-height"
    );
    assert!(
        production.contains(".border_t_1()"),
        "bottom information block must be visually separated from content"
    );
}
