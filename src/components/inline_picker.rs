//! Neutral inline-picker row shape and enabled-aware selection helpers.
//!
//! Context for this module (Oracle iter 015 + iter 018):
//!
//! The shared **renderer** for dense inline pickers already lives in
//! [`crate::components::inline_dropdown`] (see `InlineDropdown`,
//! `render_soft_compact_picker_row`, `inline_dropdown_visible_range_from_start`,
//! `InlineDropdownColors`, `InlineDropdownEmptyState`, `InlineDropdownSynopsis`).
//! Agent Chat's slash/profile popups consume that renderer today.
//!
//! What this module adds is the neutral **data shape** that callers like
//! the menu-syntax `:`, `;`, and `!` trigger pickers and cross-surface automation tooling
//! that inspects picker state — can hold independent of Agent Chat's
//! `ContextSelectorRow` or menu-syntax's `TriggerPickerRow`. Both owners map
//! their domain row into `InlinePickerRow` via a small adapter function kept in
//! the owner's module (`adapt_context_selector_row` inside Agent Chat code,
//! `adapt_trigger_picker_row` inside the menu-syntax picker owner). The
//! adapters live with the domain types so this shared file never imports Agent Chat
//! or menu-syntax types.
//!
//! On top of the shape, this module also provides enabled-row-aware selection
//! helpers that the existing `inline_dropdown` selection helpers do not
//! offer. Menu-syntax picker rows include non-selectable items (section
//! markers, "coming soon" footers) that keyboard navigation must skip,
//! which is a category the generic `inline_dropdown_select_next` helpers do
//! not know about.
//!
//! The shape carries **no closures and no domain actions**. Owners map a
//! selection back to their domain behavior using the row `id`. Keeping the
//! shape behavior-free is what makes it usable across Agent Chat and menu-syntax
//! without either one leaking into the other.

use gpui::SharedString;
use std::ops::Range;

/// Stable identity for a row inside one owner snapshot.
///
/// Owners should preserve selection by `id`, not by index, when the row list
/// is rebuilt after filtering. `SharedString` is cheap to clone and already
/// the preferred GPUI string type across this codebase.
pub type InlinePickerRowId = SharedString;

/// Visual classification only. The shared layer may use this for icon
/// treatment or subtle styling; it must not infer behavior from it.
#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum InlinePickerRowKind {
    /// Agent Chat `@` mention rows (files, dictation, screenshots, etc.).
    Context,
    /// Agent Chat `/` slash-command rows.
    SlashCommand,
    /// Menu-syntax `:`, `;`, and `!` trigger rows.
    TextTrigger,
    /// Non-trigger action rows (footer "Create capture handler…",
    /// "Open Menu Syntax help").
    Action,
    /// Escape hatch for owner-specific kinds that do not fit the four
    /// above categories. Shared layer must not branch on the payload.
    Custom(SharedString),
}

/// Optional leading visual. Kept generic so Agent Chat and menu-syntax do not leak
/// their icon systems into the shared row type.
#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum InlinePickerLeadingVisual {
    /// Single-character glyph (emoji, symbol).
    Glyph(SharedString),
    /// Named icon the renderer resolves against its own registry.
    IconName(SharedString),
}

/// Tone applied to a row badge chip.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum InlinePickerBadgeTone {
    Neutral,
    Accent,
    Warning,
    Disabled,
}

/// Small chip rendered near the row text (e.g. `default`, `shipped`,
/// `coming soon`).
#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub struct InlinePickerBadge {
    pub label: SharedString,
    pub tone: InlinePickerBadgeTone,
}

/// Trailing affordance rendered on the right edge of the row.
#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum InlinePickerAccessory {
    Text(SharedString),
    Shortcut(SharedString),
    Token(SharedString),
}

/// Precomputed highlight ranges for each visible text slot. Ranges are
/// byte offsets into the corresponding UTF-8 string and must land on
/// character boundaries. [`validate_highlight_ranges`] enforces this.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[allow(dead_code)]
pub struct InlinePickerHighlights {
    pub title: Vec<Range<usize>>,
    pub token: Vec<Range<usize>>,
    pub subtitle: Vec<Range<usize>>,
    pub detail: Vec<Range<usize>>,
}

/// Neutral row consumed by menu-syntax's popup and by cross-surface
/// automation / inspection tooling.
///
/// This struct is intentionally behavior-free. It does not carry callbacks,
/// domain actions, Agent Chat context objects, or menu-syntax actions. Owners map
/// `id` back to their domain row when Enter/Tab accepts a selection.
#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub struct InlinePickerRow {
    pub id: InlinePickerRowId,
    pub kind: InlinePickerRowKind,
    pub title: SharedString,
    pub token: Option<SharedString>,
    pub subtitle: Option<SharedString>,
    pub detail: Option<SharedString>,
    pub example: Option<SharedString>,
    pub leading: Option<InlinePickerLeadingVisual>,
    pub badges: Vec<InlinePickerBadge>,
    pub accessory: Option<InlinePickerAccessory>,
    pub highlights: InlinePickerHighlights,
    pub enabled: bool,
    pub disabled_reason: Option<SharedString>,
}

/// Return a reference to the currently selected row, skipping the bounds
/// check callers would otherwise duplicate.
#[allow(dead_code)]
pub fn inline_picker_selected_row(
    rows: &[InlinePickerRow],
    selected_index: Option<usize>,
) -> Option<&InlinePickerRow> {
    selected_index.and_then(|idx| rows.get(idx))
}

/// Clamp `selected_index` to a valid, enabled row. If the current index is
/// out of range or points at a disabled row, fall forward to the next
/// enabled row, then backward. Returns `None` only when there are no
/// enabled rows at all.
#[allow(dead_code)]
pub fn inline_picker_normalize_selected_index(
    rows: &[InlinePickerRow],
    selected_index: Option<usize>,
) -> Option<usize> {
    if rows.is_empty() {
        return None;
    }

    if let Some(idx) = selected_index {
        if rows.get(idx).is_some_and(|row| row.enabled) {
            return Some(idx);
        }
        if let Some(next) = rows
            .iter()
            .enumerate()
            .skip(idx.saturating_add(1))
            .find(|(_, row)| row.enabled)
            .map(|(i, _)| i)
        {
            return Some(next);
        }
    }

    rows.iter()
        .enumerate()
        .find(|(_, row)| row.enabled)
        .map(|(i, _)| i)
}

/// Advance selection to the next enabled row after `selected_index`, wrapping
/// around at the end. Returns `None` when no rows are enabled.
#[allow(dead_code)]
pub fn inline_picker_next_enabled_index(
    rows: &[InlinePickerRow],
    selected_index: Option<usize>,
) -> Option<usize> {
    if rows.iter().all(|row| !row.enabled) {
        return None;
    }

    let len = rows.len();
    let start = match selected_index {
        Some(idx) => (idx + 1) % len,
        None => 0,
    };

    for offset in 0..len {
        let idx = (start + offset) % len;
        if rows[idx].enabled {
            return Some(idx);
        }
    }

    None
}

/// Advance selection to the previous enabled row before `selected_index`,
/// wrapping around at the start. Returns `None` when no rows are enabled.
#[allow(dead_code)]
pub fn inline_picker_previous_enabled_index(
    rows: &[InlinePickerRow],
    selected_index: Option<usize>,
) -> Option<usize> {
    if rows.iter().all(|row| !row.enabled) {
        return None;
    }

    let len = rows.len();
    let start = match selected_index {
        Some(0) | None => len - 1,
        Some(idx) => idx - 1,
    };

    for offset in 0..len {
        let idx = (start + len - offset) % len;
        if rows[idx].enabled {
            return Some(idx);
        }
    }

    None
}

/// Visible-window helper that delegates to the existing
/// [`crate::components::inline_dropdown::inline_dropdown_visible_range_from_start`]
/// renderer contract, so every inline popup uses the same scrolling rules.
#[allow(dead_code)]
pub fn inline_picker_visible_range(
    visible_start: usize,
    selected_index: usize,
    rows_len: usize,
    max_visible_rows: usize,
) -> Range<usize> {
    crate::components::inline_dropdown::inline_dropdown_visible_range_from_start(
        visible_start,
        selected_index,
        rows_len,
        max_visible_rows,
    )
}

/// Validate that every byte-offset range in the row's highlight set lands
/// on a UTF-8 character boundary of the corresponding text field. Callers
/// should run this before rendering to avoid runtime panics on non-ASCII
/// content. Returns `true` when all ranges are valid.
#[allow(dead_code)]
pub fn validate_highlight_ranges(row: &InlinePickerRow) -> bool {
    let title = row.title.as_ref();
    let token = row.token.as_ref().map_or("", |s| s.as_ref());
    let subtitle = row.subtitle.as_ref().map_or("", |s| s.as_ref());
    let detail = row.detail.as_ref().map_or("", |s| s.as_ref());

    ranges_valid(title, &row.highlights.title)
        && ranges_valid(token, &row.highlights.token)
        && ranges_valid(subtitle, &row.highlights.subtitle)
        && ranges_valid(detail, &row.highlights.detail)
}

fn ranges_valid(text: &str, ranges: &[Range<usize>]) -> bool {
    ranges.iter().all(|r| {
        r.start <= r.end
            && r.end <= text.len()
            && text.is_char_boundary(r.start)
            && text.is_char_boundary(r.end)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(id: &str, enabled: bool) -> InlinePickerRow {
        InlinePickerRow {
            id: SharedString::from(id.to_string()),
            kind: InlinePickerRowKind::TextTrigger,
            title: SharedString::from(id.to_string()),
            token: None,
            subtitle: None,
            detail: None,
            example: None,
            leading: None,
            badges: Vec::new(),
            accessory: None,
            highlights: InlinePickerHighlights::default(),
            enabled,
            disabled_reason: None,
        }
    }

    #[test]
    fn selected_row_returns_none_when_index_missing() {
        let rows = vec![row("a", true), row("b", true)];
        assert!(inline_picker_selected_row(&rows, None).is_none());
    }

    #[test]
    fn selected_row_returns_reference_when_index_valid() {
        let rows = vec![row("a", true), row("b", true)];
        let picked = inline_picker_selected_row(&rows, Some(1)).unwrap();
        assert_eq!(picked.id.as_ref(), "b");
    }

    #[test]
    fn selected_row_out_of_range_is_none() {
        let rows = vec![row("a", true)];
        assert!(inline_picker_selected_row(&rows, Some(99)).is_none());
    }

    #[test]
    fn normalize_returns_none_for_empty_rows() {
        assert!(inline_picker_normalize_selected_index(&[], Some(0)).is_none());
    }

    #[test]
    fn normalize_snaps_disabled_to_next_enabled() {
        let rows = vec![row("a", false), row("b", false), row("c", true)];
        assert_eq!(
            inline_picker_normalize_selected_index(&rows, Some(0)),
            Some(2)
        );
    }

    #[test]
    fn normalize_falls_back_to_first_enabled_when_past_end() {
        let rows = vec![row("a", false), row("b", true), row("c", false)];
        assert_eq!(
            inline_picker_normalize_selected_index(&rows, Some(2)),
            Some(1)
        );
    }

    #[test]
    fn normalize_returns_none_when_every_row_disabled() {
        let rows = vec![row("a", false), row("b", false)];
        assert!(inline_picker_normalize_selected_index(&rows, Some(0)).is_none());
    }

    #[test]
    fn next_enabled_skips_disabled_rows() {
        let rows = vec![row("a", true), row("b", false), row("c", true)];
        assert_eq!(inline_picker_next_enabled_index(&rows, Some(0)), Some(2));
    }

    #[test]
    fn next_enabled_wraps_at_end() {
        let rows = vec![row("a", true), row("b", false)];
        assert_eq!(inline_picker_next_enabled_index(&rows, Some(0)), Some(0));
    }

    #[test]
    fn next_enabled_from_none_picks_first_enabled() {
        let rows = vec![row("a", false), row("b", true)];
        assert_eq!(inline_picker_next_enabled_index(&rows, None), Some(1));
    }

    #[test]
    fn next_enabled_returns_none_when_none_enabled() {
        let rows = vec![row("a", false), row("b", false)];
        assert!(inline_picker_next_enabled_index(&rows, Some(0)).is_none());
    }

    #[test]
    fn previous_enabled_skips_disabled_rows() {
        let rows = vec![row("a", true), row("b", false), row("c", true)];
        assert_eq!(
            inline_picker_previous_enabled_index(&rows, Some(2)),
            Some(0)
        );
    }

    #[test]
    fn previous_enabled_wraps_at_start() {
        let rows = vec![row("a", false), row("b", true)];
        assert_eq!(
            inline_picker_previous_enabled_index(&rows, Some(1)),
            Some(1)
        );
    }

    #[test]
    fn previous_enabled_from_none_picks_last_enabled() {
        let rows = vec![row("a", true), row("b", false)];
        assert_eq!(inline_picker_previous_enabled_index(&rows, None), Some(0));
    }

    #[test]
    fn previous_enabled_returns_none_when_none_enabled() {
        let rows = vec![row("a", false), row("b", false)];
        assert!(inline_picker_previous_enabled_index(&rows, Some(0)).is_none());
    }

    #[test]
    fn validate_highlight_ranges_passes_for_ascii() {
        let mut r = row("typ:script", true);
        r.title = SharedString::from("type:script");
        r.highlights.title = vec![0..4];
        assert!(validate_highlight_ranges(&r));
    }

    #[test]
    fn validate_highlight_ranges_rejects_mid_char_boundary() {
        let mut r = row("weather", true);
        // "☕️" is 6 bytes in UTF-8 — slicing mid-code-point must fail.
        r.title = SharedString::from("☕️coffee");
        r.highlights.title = vec![2..5];
        assert!(!validate_highlight_ranges(&r));
    }

    #[test]
    fn validate_highlight_ranges_rejects_end_past_string() {
        let mut r = row("short", true);
        r.title = SharedString::from("abc");
        r.highlights.title = vec![0..99];
        assert!(!validate_highlight_ranges(&r));
    }

    #[test]
    fn validate_highlight_ranges_accepts_empty_ranges() {
        let r = row("no-highlights", true);
        assert!(validate_highlight_ranges(&r));
    }

    #[test]
    fn visible_range_delegates_to_inline_dropdown() {
        // With 20 rows, max 8 visible, selected at 15, the window should
        // include row 15 — same contract as inline_dropdown.
        let range = inline_picker_visible_range(10, 15, 20, 8);
        assert!(range.contains(&15));
        assert!(range.len() <= 8);
    }
}
