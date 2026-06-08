//! Source-level + behavioral contract: arrow navigation inside
//! `ActionsDialog` MUST skip `GroupedActionItem::SectionHeader` rows.
//!
//! # Why this matters (UX)
//!
//! Every Cmd+K actions menu in the app (main-menu, clipboard history
//! with its 6 section headers, emoji picker, file search, Agent Chat chat,
//! notes, app launcher, …) groups actions under section headers via
//! `GroupedActionItem::SectionHeader(String)`. Headers render as a
//! distinct 22px row and are NOT activatable — `get_selected_action_id`
//! returns `None` when the cursor lands on one (see
//! `src/actions/dialog.rs:1716` match arm). If arrow navigation were
//! allowed to select a header:
//!
//!   * The visible selection ring would land on a header.
//!   * Enter would silently no-op via `ActionsDialogActivation::NoSelection`.
//!   * Users would see "skipped" rows in their up/down rhythm
//!     (e.g. clipboard history's 6-header / 12-action menu becomes
//!     "down down (no visible change) down down …").
//!
//! That's a classic user-visible regression a "DRY cleanup" refactor
//! could easily ship. The `move_up` / `move_down` implementations
//! at `src/actions/dialog.rs` delegate to the shared directional
//! `selectable_index_*` helpers owned beside `GroupedActionItem` in
//! `src/actions/dialog.rs`, matching the MoveHome / MoveEnd /
//! MovePageUp / MovePageDown paths. A contributor touching that
//! shared selection path might
//! "simplify" both to:
//!
//! ```ignore
//! pub fn move_down(&mut self, cx: &mut Context<Self>) {
//!     if self.selected_index < self.grouped_items.len().saturating_sub(1) {
//!         self.selected_index += 1;
//!         cx.notify();
//!     }
//! }
//! ```
//!
//! which compiles, passes every behavioral test that doesn't
//! construct a grouped list with a header, and looks like a
//! reasonable cleanup. That's the regression this contract exists
//! to make impossible to ship silently.
//!
//! # Anchors
//!
//! This file pins the invariant two ways so the contract is robust
//! against either a body-level refactor or an enum-level refactor:
//!
//!   (1) `move_up` body delegates to
//!       `selectable_index_at_or_before`, so it searches in the
//!       same direction as the key movement while filtering for
//!       selectable rows. A `selected_index -= 1; return;`
//!       reduction does not satisfy this anchor.
//!
//!   (2) `move_down` body delegates to
//!       `selectable_index_at_or_after`, the downward companion.
//!
//!   (3) `GroupedActionItem` enum declares both `SectionHeader(...)`
//!       and `Item(...)` variants. Flattening the enum to a single
//!       variant would be the shape this regression takes at the
//!       type level; the test catches it even if body anchors are
//!       preserved via some trivial wrapper.
//!
//!   (4) The shared helpers in `src/actions/dialog.rs`
//!       (`first_selectable_index`, `last_selectable_index`,
//!       `selectable_index_at_or_before`, `selectable_index_at_or_after`)
//!       still filter via `is_selectable_row` which checks
//!       `matches!(row, GroupedActionItem::Item(_))`. MoveHome /
//!       MoveEnd / MovePageUp / MovePageDown all delegate to these
//!       helpers — if the helpers lose the filter, all four page
//!       intents would regress in one commit.
//!
//!   (5) The `GroupedActionItem::SectionHeader` variant is actually
//!       USED in the grouped-items build path — we include the
//!       `grouped.push(GroupedActionItem::SectionHeader(...))` call
//!       so any refactor that collapses section rendering flips a
//!       test red before the regression ships.

const DIALOG_SOURCE: &str = include_str!("../src/actions/dialog.rs");
const WINDOW_SOURCE: &str = include_str!("../src/actions/window.rs");

fn extract_fn_body<'a>(src: &'a str, header: &str) -> &'a str {
    let header_pos = src.find(header).unwrap_or_else(|| {
        panic!(
            "source: `{header}` not found — the method may have been renamed. \
             Update the contract anchors in \
             tests/actions_dialog_arrow_nav_skips_section_headers_contract.rs."
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

fn body_filters_for_selectable_rows(body: &str) -> bool {
    body.contains("GroupedActionItem::Item(")
        || body.contains("is_selectable_row(")
        || body.contains("first_selectable_index(")
        || body.contains("last_selectable_index(")
        || body.contains("selectable_index_at_or_before(")
        || body.contains("selectable_index_at_or_after(")
}

#[test]
fn move_up_skips_section_headers() {
    let body = extract_fn_body(DIALOG_SOURCE, "pub fn move_up(");
    assert!(
        body_filters_for_selectable_rows(body),
        "ActionsDialog::move_up MUST filter for selectable rows via \
         `GroupedActionItem::Item(` match OR a call to one of the \
         `selectable_index_*` / `first_selectable_index` / \
         `last_selectable_index` / `is_selectable_row` helpers. \
         A naïve `self.selected_index -= 1` is the shape this \
         contract blocks — it would let users land on \
         `SectionHeader` rows during up/down navigation, producing \
         visibly-skipped rhythm and a silent no-op on Enter. \
         Body was:\n{body}"
    );
    assert!(
        body.contains("selectable_index_at_or_before("),
        "ActionsDialog::move_up should delegate to the shared directional \
         selectable-row helper instead of carrying a bespoke skip loop. \
         Body was:\n{body}"
    );
}

#[test]
fn move_down_skips_section_headers() {
    let body = extract_fn_body(DIALOG_SOURCE, "pub fn move_down(");
    assert!(
        body_filters_for_selectable_rows(body),
        "ActionsDialog::move_down MUST filter for selectable rows via \
         `GroupedActionItem::Item(` match OR a call to one of the \
         `selectable_index_*` helpers. Same regression class as \
         `move_up` — a `self.selected_index += 1` reduction would \
         land the cursor on section headers. Body was:\n{body}"
    );
    assert!(
        body.contains("selectable_index_at_or_after("),
        "ActionsDialog::move_down should delegate to the shared directional \
         selectable-row helper instead of carrying a bespoke skip loop. \
         Body was:\n{body}"
    );
}

#[test]
fn grouped_action_item_enum_declares_both_variants() {
    let marker = "pub enum GroupedActionItem {";
    let start = DIALOG_SOURCE.find(marker).expect(
        "pub enum GroupedActionItem not found in src/actions/dialog.rs — \
         the enum may have been renamed. Update the contract anchors.",
    );
    let after_brace = start + marker.len();
    let end_rel = DIALOG_SOURCE[after_brace..]
        .find('}')
        .expect("no closing `}` for GroupedActionItem enum");
    let enum_body = &DIALOG_SOURCE[after_brace..after_brace + end_rel];

    assert!(
        enum_body.contains("SectionHeader("),
        "GroupedActionItem must keep the `SectionHeader(...)` variant. \
         Dropping it collapses the grouped rendering that Cmd+K \
         dialogs across hosts rely on. Body was:\n{enum_body}"
    );
    assert!(
        enum_body.contains("Item("),
        "GroupedActionItem must keep the `Item(...)` variant — it's \
         the discriminator the skip-loop match relies on. \
         Body was:\n{enum_body}"
    );
}

#[test]
fn shared_selectable_helpers_filter_on_item_variant() {
    // The `is_selectable_row` helper is the single source of truth
    // for "is this row selectable?" across MoveHome, MoveEnd,
    // MovePageUp, MovePageDown, and Up/Down. If it stops checking for
    // `GroupedActionItem::Item(_)`, the whole helper family regresses
    // in one commit.
    let body = extract_fn_body(DIALOG_SOURCE, "pub(super) fn is_selectable_row(");
    assert!(
        body.contains("GroupedActionItem::Item(_)"),
        "is_selectable_row in src/actions/dialog.rs MUST match on \
         `GroupedActionItem::Item(_)`. Relaxing the pattern would \
         let MoveHome / MoveEnd / MovePageUp / MovePageDown / Up / Down land \
         on section headers in one commit. Body was:\n{body}"
    );
    assert!(
        !WINDOW_SOURCE.contains("fn selectable_index_at_or_before(")
            && !WINDOW_SOURCE.contains("fn selectable_index_at_or_after("),
        "selectable-row helpers should be owned beside GroupedActionItem in \
         src/actions/dialog.rs, not redeclared in src/actions/window.rs"
    );

    // Confirm the page-intent paths actually delegate to the filtered
    // helpers (not a raw index). This is the symmetric half of the
    // move_up/move_down anchors above.
    // Anchor on the dispatch arm (`=>`), NOT the key-intent producer
    // that returns `Some(ActionsWindowKeyIntent::MoveHome)` from the
    // parser above.
    let move_home_block = "Some(ActionsWindowKeyIntent::MoveHome) =>";
    let mh_start = WINDOW_SOURCE
        .find(move_home_block)
        .expect("MoveHome dispatch arm not found in src/actions/window.rs — update anchors");
    // Scope to the next 600 bytes so we don't false-match on other arms.
    let mh_window = &WINDOW_SOURCE[mh_start..mh_start + 600.min(WINDOW_SOURCE.len() - mh_start)];
    assert!(
        mh_window.contains("first_selectable_index("),
        "MoveHome arm MUST delegate to `first_selectable_index`. \
         Inlining a raw `selected_index = 0` would skip the \
         section-header filter. Window was:\n{mh_window}"
    );
}

#[test]
fn grouped_items_build_path_still_pushes_section_headers() {
    // The skip logic only matters if section headers are actually
    // produced upstream. If a future refactor stopped pushing
    // `GroupedActionItem::SectionHeader(...)` into `grouped_items`,
    // Up/Down's skip loop would still be correct but the UX shape
    // the skip exists to preserve would be gone. This test guards
    // the upstream producer so we notice a "flatten the list"
    // refactor alongside any body-level change.
    assert!(
        DIALOG_SOURCE.contains("grouped.push(GroupedActionItem::SectionHeader("),
        "src/actions/dialog.rs MUST continue to push \
         `GroupedActionItem::SectionHeader(...)` into the grouped \
         items list. If the grouped-items build path stops emitting \
         headers, the grouped Cmd+K catalog collapses to a flat \
         list across hosts (clipboard history loses its 6 section \
         groupings, emoji picker loses its category headers, etc.)."
    );
}
