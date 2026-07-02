//! Structured Day Page document lifecycle telemetry.
//!
//! Every event shares the stable `script_kit::day_page` target so agents can
//! reconstruct the open→edit→save arc of a Day Page / fragment / note binding
//! with `getLogs {target: "script_kit::day_page"}`. Callers pass a static
//! `source` site string.
//!
//! These helpers log the binding kind, the bound date, and content *lengths*
//! only — never the document text itself.

/// A document was bound into the Day Page editor (day, fragment, or note).
/// `date` is the ISO day for day/fragment bindings, `None` for notes.
pub(crate) fn log_document_loaded(
    source: &'static str,
    kind: &'static str,
    date: Option<&str>,
    content_bytes: usize,
) {
    tracing::info!(
        target: "script_kit::day_page",
        category = "DAY_PAGE",
        event = "document_loaded",
        source,
        kind,
        date = ?date,
        content_bytes,
        "day_page_lifecycle"
    );
}

/// A dirty document buffer was flushed to disk (or the notes store),
/// transitioning the session from dirty→clean. `merged` is true when an
/// external append landed during the save and was merged in.
pub(crate) fn log_document_saved(
    source: &'static str,
    kind: &'static str,
    content_bytes: usize,
    merged: bool,
) {
    tracing::info!(
        target: "script_kit::day_page",
        category = "DAY_PAGE",
        event = "document_saved",
        source,
        kind,
        content_bytes,
        merged,
        "day_page_lifecycle"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn day_page_telemetry_helpers_do_not_panic() {
        log_document_loaded("test", "day", Some("2026-07-01"), 128);
        log_document_loaded("test", "note", None, 0);
        log_document_saved("test", "day", 256, false);
        log_document_saved("test", "fragment", 256, true);
    }
}
