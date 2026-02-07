use super::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct InlineStyle {
    pub(super) bold: bool,
    pub(super) italic: bool,
    pub(super) code: bool,
    pub(super) link: bool,
    pub(super) strikethrough: bool,
}

#[derive(Clone, Debug)]
pub(super) struct InlineSpan {
    pub(super) text: String,
    pub(super) style: InlineStyle,
    /// URL for link spans (None for non-link text)
    pub(super) link_url: Option<String>,
}

#[derive(Clone, Debug)]
pub(super) struct ListState {
    pub(super) ordered: bool,
    pub(super) start: usize,
    pub(super) items: Vec<ListItem>,
}

/// Table parsing state
#[derive(Debug)]
pub(super) struct TableState {
    pub(super) headers: Vec<Vec<InlineSpan>>,
    pub(super) rows: Vec<Vec<Vec<InlineSpan>>>,
    pub(super) current_row: Vec<Vec<InlineSpan>>,
    pub(super) in_head: bool,
}

#[derive(Debug)]
pub(super) struct CodeBlockState {
    pub(super) language: Option<String>,
    pub(super) code: String,
}

#[derive(Debug)]
pub(super) struct ImageState {
    pub(super) url: String,
    pub(super) alt_text: String,
}

// ---------------------------------------------------------------------------
// Cached intermediate representation
// ---------------------------------------------------------------------------

/// A single list item with inline spans and optional task-list checkbox state.
#[derive(Clone, Debug)]
pub(super) struct ListItem {
    pub(super) spans: Vec<InlineSpan>,
    /// `Some(true)` = checked `[x]`, `Some(false)` = unchecked `[ ]`, `None` = regular item
    pub(super) checked: Option<bool>,
    pub(super) nested_lists: Vec<ListState>,
}

/// Cached intermediate representation of a parsed markdown block.
/// Stored in a global cache to avoid re-parsing on every render frame.
#[derive(Clone, Debug)]
pub(super) enum ParsedBlock {
    Paragraph {
        spans: Vec<InlineSpan>,
        quote_depth: usize,
    },
    Heading {
        level: u32,
        spans: Vec<InlineSpan>,
        quote_depth: usize,
    },
    ListBlock {
        ordered: bool,
        start: usize,
        items: Vec<ListItem>,
        quote_depth: usize,
    },
    CodeBlock {
        lang_label: String,
        lines: Vec<CodeLine>,
        /// Raw code text for copy-to-clipboard functionality
        raw_code: Arc<str>,
        quote_depth: usize,
    },
    Table {
        headers: Vec<Vec<InlineSpan>>,
        rows: Vec<Vec<Vec<InlineSpan>>>,
        quote_depth: usize,
    },
    HorizontalRule {
        quote_depth: usize,
    },
}
