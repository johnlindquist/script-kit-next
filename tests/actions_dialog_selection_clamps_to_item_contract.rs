//! Source-level + behavioral contract: the shared `ActionsDialog`'s
//! `selected_index` MUST ALWAYS point at a `GroupedActionItem::Item(_)`
//! row — not a `SectionHeader` — at every entry point that assigns it:
//! construction, reset, and refilter.
//!
//! # Why this matters (UX)
//!
//! Pass #13 (`tests/actions_dialog_arrow_nav_skips_section_headers_contract.rs`)
//! pinned the arrow-key paths — `move_up` and `move_down` MUST skip
//! `SectionHeader` rows. But the arrow-key contract only guards the
//! delta-from-current case. There are at least three other entry
//! points that assign `selected_index` directly and each one must
//! also land on a selectable row:
//!
//!   (a) Construction: `initial_selection_index(&grouped_items)` at
//!       `src/actions/dialog.rs:647` (and every reset / restore path).
//!       Clipboard-history's first row is a `SectionHeader` —
//!       without a clamp, the cursor opens on a header and Enter
//!       silently no-ops via `ActionsDialogActivation::NoSelection`.
//!   (b) Refilter: `refilter(&mut self)` at `src/actions/dialog.rs:1710`
//!       rebuilds `grouped_items` after a filter change. If the
//!       previously-selected action is gone, the fallback MUST
//!       clamp to the first selectable row — not blindly set to 0,
//!       which may be a `SectionHeader` in the newly-filtered list.
//!   (c) Config-driven rebuild: `refresh_*` / `update_config` paths
//!       call `initial_selection_index(&self.grouped_items)` after
//!       a section-style change — same hazard as (a).
//!
//! All three delegate to one shared helper:
//! `coerce_action_selection(rows, ix)` at `src/actions/dialog.rs:179`.
//! The helper is the single source of truth for "given an index,
//! return the nearest selectable `Item(_)` row (search down first,
//! then up, `None` if all rows are headers)". `initial_selection_index`
//! at line 212 is a thin wrapper that calls
//! `coerce_action_selection(rows, 0)`.
//!
//! The regression this contract blocks: a "simplification" of
//! `coerce_action_selection` to `Some(ix.min(rows.len().saturating_sub(1)))`
//! compiles, passes every behavioral test that doesn't construct a
//! grouped list where index 0 is a header, and silently lands the
//! cursor on a `SectionHeader` on every Cmd+K host whose first row
//! is a header (clipboard-history is the canonical case — its
//! `Clipboard Actions` header is always at index 0). The user would
//! see a dialog open with the selection ring on a non-activatable
//! row and Enter would do nothing. A refactor that inlines
//! `selected_index = 0` anywhere in `refilter` (bypassing the
//! helper) has the same shape.
//!
//! # Anchors
//!
//!   (1) `coerce_action_selection` body MUST match on
//!       `GroupedActionItem::Item(` — the selectable-row predicate.
//!   (2) `coerce_action_selection` body MUST contain BOTH a
//!       forward search (iterator `skip(ix + 1)`) AND a backward
//!       search (iterator `take(ix)` with `.rev()`) so the helper
//!       handles trailing-header as well as leading-header cases.
//!   (3) `initial_selection_index` body MUST call
//!       `coerce_action_selection(` — must not inline a raw `0`
//!       or re-implement the skip loop.
//!   (4) `refilter` body MUST contain `coerce_action_selection(`
//!       (not a raw `self.selected_index = 0` fallback).
//!
//! Together with Pass #13's arrow-key pin, these anchors close the
//! "cursor can never land on a `SectionHeader`" invariant at every
//! assignment site inside `ActionsDialog`.

const DIALOG_SOURCE: &str = include_str!("../src/actions/dialog.rs");

fn extract_fn_body<'a>(src: &'a str, header: &str) -> &'a str {
    let header_pos = src.find(header).unwrap_or_else(|| {
        panic!(
            "source: `{header}` not found — the method may have been renamed. \
             Update the contract anchors in \
             tests/actions_dialog_selection_clamps_to_item_contract.rs."
        )
    });
    let open_rel = src[header_pos..]
        .find('{')
        .unwrap_or_else(|| panic!("no `{{` after `{header}` header"));
    let open_abs = header_pos + open_rel;

    let mut depth: i32 = 0;
    let mut close_abs: Option<usize> = None;
    for (offset, &b) in src.as_bytes()[open_abs..].iter().enumerate() {
        match b {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    close_abs = Some(open_abs + offset);
                    break;
                }
            }
            _ => {}
        }
    }
    let close_abs = close_abs.unwrap_or_else(|| panic!("no matching `}}` for `{header}` body"));
    &src[open_abs..=close_abs]
}

#[test]
fn coerce_action_selection_filters_on_item_variant() {
    // The single source of truth for "skip to the nearest selectable
    // row". If this body loses the `GroupedActionItem::Item(` match,
    // every construction / reset / refilter path that delegates here
    // regresses in one commit — the cursor could open on a header
    // on every Cmd+K host whose first row is a `SectionHeader`
    // (clipboard-history being the canonical case).
    let body = extract_fn_body(DIALOG_SOURCE, "fn coerce_action_selection(");
    assert!(
        body.contains("GroupedActionItem::Item("),
        "`coerce_action_selection` MUST match on `GroupedActionItem::Item(` \
         to identify selectable rows. Without this predicate the helper \
         becomes a no-op bounds clamp and every caller (construction, \
         reset, refilter fallback) would land the cursor on a \
         `SectionHeader` whenever index 0 is one. Body was:\n{body}"
    );
}

#[test]
fn coerce_action_selection_searches_both_directions() {
    // The helper MUST search BOTH directions: forward (past `ix`) for
    // the common leading-header case (e.g. clipboard-history), AND
    // backward (before `ix`) for the trailing-header case (e.g. a
    // filter that leaves only pre-header items selectable). A
    // unidirectional simplification is a silent regression: on a
    // filtered list whose surviving rows are all BEFORE `ix`, a
    // forward-only search returns `None` and the caller falls back
    // to 0, which may itself be a header.
    let body = extract_fn_body(DIALOG_SOURCE, "fn coerce_action_selection(");
    assert!(
        body.contains(".skip(ix + 1)"),
        "`coerce_action_selection` MUST contain a forward search \
         (`iter().enumerate().skip(ix + 1)`) — otherwise leading-header \
         cases land on the header. Body was:\n{body}"
    );
    // The backward search uses `.take(ix).rev()` in the current
    // implementation. Accept either `take(ix)` or `.rev()` as the
    // anchor — if both disappear the helper is single-direction.
    assert!(
        body.contains(".take(ix)") && body.contains(".rev()"),
        "`coerce_action_selection` MUST contain a backward search \
         (`iter().enumerate().take(ix).rev()`) — otherwise trailing-header \
         cases land on the header. A unidirectional helper silently \
         regresses on filtered lists whose surviving items all live \
         before `ix`. Body was:\n{body}"
    );
}

#[test]
fn initial_selection_index_delegates_to_coerce() {
    // `initial_selection_index` is called by construction and every
    // reset / restore / config-rebuild path. Inlining `0` here is
    // the shape the regression takes when a contributor decides the
    // helper is "overkill" for the initial case — it looks harmless
    // until the first row is a header.
    let body = extract_fn_body(DIALOG_SOURCE, "fn initial_selection_index(");
    assert!(
        body.contains("coerce_action_selection("),
        "`initial_selection_index` MUST delegate to \
         `coerce_action_selection(rows, 0)` rather than inlining a raw \
         `0`. Every construction / reset path assigns \
         `self.selected_index = initial_selection_index(&grouped_items)` \
         expecting a selectable row back — an inline `0` would land \
         the cursor on a header on clipboard-history and any other \
         host whose first row is a `SectionHeader`. Body was:\n{body}"
    );
}

#[test]
fn refilter_clamps_selection_via_coerce() {
    // Refilter is the highest-traffic selection-assignment path:
    // every keystroke that changes the filter text rebuilds
    // `grouped_items`. The previous selection may disappear, in
    // which case the fallback MUST clamp to a selectable row — a
    // raw `self.selected_index = 0` fallback would land on a header
    // on every host whose filtered list starts with one (clipboard
    // history's section-headers survive because sections are
    // rebuilt around the filtered items).
    let body = extract_fn_body(DIALOG_SOURCE, "fn refilter(");
    assert!(
        body.contains("coerce_action_selection("),
        "`refilter` MUST use `coerce_action_selection(` to clamp \
         `selected_index` after a filter-driven rebuild of \
         `grouped_items`. A raw `self.selected_index = 0` fallback \
         would land the cursor on a `SectionHeader` any time the \
         newly-filtered list starts with one. Body was:\n{body}"
    );
}

#[test]
fn refilter_does_not_inline_raw_zero_selection_fallback() {
    // The symmetric negative pin: even if `coerce_action_selection`
    // is *also* called somewhere in `refilter`, an accidental extra
    // `self.selected_index = 0;` (no `coerce_action_selection` wrapper)
    // bypasses the clamp in that branch. This test makes sure the
    // specific anti-pattern string is absent.
    let body = extract_fn_body(DIALOG_SOURCE, "fn refilter(");
    assert!(
        !body.contains("self.selected_index = 0;"),
        "`refilter` MUST NOT assign `self.selected_index = 0;` directly — \
         even as a fallback. Every selection assignment inside refilter \
         must route through `coerce_action_selection(` so headers are \
         skipped. A raw `= 0;` would land on a leading section-header \
         on every filter keystroke that clears the previous selection. \
         Body was:\n{body}"
    );
}
