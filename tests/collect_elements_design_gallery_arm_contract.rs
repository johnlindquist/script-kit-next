//! Source-level contract for the Run 9 Pass #15 Extend of
//! `actions-cmdk-builtin-design-gallery` — adds a
//! `AppView::DesignGalleryView` match arm to `collect_visible_elements`
//! in `src/app_layout/collect_elements.rs` so `getElements` against a
//! live DesignGallery-hosted main window returns full
//! `input + list + row*` semantics instead of the
//! `panel:current-view` + `collector_used_current_view_fallback`
//! shape that Pass #10 documented for sibling BuiltinList views.
//!
//! Follows the Pass #11 BrowserTabs / Pass #14 BrowserHistory
//! template: same `input-filter` + `list` naming shape, same predicate
//! mirror against the renderer. The row strings come from
//! `design_gallery_item_label` (new shared helper in
//! `src/render_builtins/design_gallery.rs` alongside the Pass #9
//! `build_gallery_items` + `gallery_item_matches` helpers) so the
//! collect arm, the state arm (Pass #9), and the renderer all consume
//! the same source-of-truth function.

const COLLECT_ELEMENTS: &str = include_str!("../src/app_layout/collect_elements.rs");
const DESIGN_GALLERY: &str = include_str!("../src/render_builtins/design_gallery.rs");

// doc-anchor-removed: [[removed-docs and introspection]]
#[test]
fn collect_visible_elements_has_design_gallery_view_arm() {
    assert!(
        COLLECT_ELEMENTS.contains("AppView::DesignGalleryView {"),
        "src/app_layout/collect_elements.rs must contain an \
         `AppView::DesignGalleryView {{` match arm in \
         `collect_visible_elements`. Without it, `getElements` with \
         no target (or `target=Main`) under a DesignGallery host falls \
         through to the `_ =>` catch-all and returns only \
         `panel:current-view` with `collector_used_current_view_fallback` \
         — the Pass #10 BuiltinList tool-gap shape."
    );
}

// doc-anchor-removed: [[removed-docs and introspection]]
#[test]
fn design_gallery_arm_calls_collect_named_rows_with_design_gallery_list_name() {
    assert!(
        COLLECT_ELEMENTS.contains("\"design-gallery-filter\","),
        "DesignGalleryView arm must pass `\"design-gallery-filter\"` \
         as the input name to `collect_named_rows`. This keeps the \
         input semanticId stable across future edits so agentic callers \
         reading `focusedSemanticId` don't drift."
    );
    assert!(
        COLLECT_ELEMENTS.contains("\"design-gallery\","),
        "DesignGalleryView arm must pass `\"design-gallery\"` as the \
         list name to `collect_named_rows`. This keeps the list \
         semanticId stable for `focusedSemanticId` / \
         `selectedSemanticId` receipts."
    );
}

// doc-anchor-removed: [[removed-docs and introspection]]
#[test]
fn design_gallery_arm_uses_shared_helpers_not_raw_contains() {
    // DesignGalleryView's filter predicate is `gallery_item_matches`
    // (shared helper at `src/render_builtins/design_gallery.rs:38`).
    // The collect arm MUST use that exact function, NOT reimplement
    // the match-arms or fall back to `.to_lowercase().contains()` on
    // a single field — that would drift the collector's row count
    // away from the renderer (which at line 107 also uses
    // `gallery_item_matches`) and the state arm (which at
    // `src/prompt_handler/mod.rs:2531` calls
    // `design_gallery_filtered_len` that itself wraps the same
    // predicate). Three-site lock-step is the Pass #9 invariant.
    let start = COLLECT_ELEMENTS
        .find("AppView::DesignGalleryView {")
        .expect("DesignGalleryView arm must exist (see sibling contract)");
    let end_rel = COLLECT_ELEMENTS[start..]
        .find("\n            AppView::")
        .or_else(|| COLLECT_ELEMENTS[start..].find("\n            _ =>"))
        .expect(
            "DesignGalleryView arm must be followed by a sibling `AppView::` \
             variant or the `_ =>` catch-all — anchor end.",
        );
    let arm = &COLLECT_ELEMENTS[start..start + end_rel];
    assert!(
        arm.contains("gallery_item_matches"),
        "DesignGalleryView arm must call `gallery_item_matches` for \
         its non-empty-filter branch. Any other predicate would skew \
         row count vs renderer + state arm. Arm body was:\n{}",
        arm
    );
    assert!(
        arm.contains("build_gallery_items"),
        "DesignGalleryView arm must call `build_gallery_items` to \
         enumerate the dataset, mirroring the renderer at \
         `src/render_builtins/design_gallery.rs:93`. Any other \
         enumeration would drift. Arm body was:\n{}",
        arm
    );
    assert!(
        arm.contains("design_gallery_item_label"),
        "DesignGalleryView arm must use the shared \
         `design_gallery_item_label` helper for row strings so the \
         collect arm and renderer agree on what text to show per \
         GalleryItem variant. Arm body was:\n{}",
        arm
    );
}

// doc-anchor-removed: [[removed-docs and introspection]]
#[test]
fn design_gallery_item_label_helper_exists_with_all_variants() {
    // Contract on the shared helper itself — if a future edit removes
    // it or only handles a subset of `GalleryItem` variants, the collect
    // arm would silently drop rows. Catching that at the helper level
    // is cheaper than discovering the drift in a live `getElements`
    // receipt.
    assert!(
        DESIGN_GALLERY.contains("fn design_gallery_item_label("),
        "src/render_builtins/design_gallery.rs must declare a \
         `design_gallery_item_label` helper used by both the collect \
         arm and any future renderer refactor. Removing it orphans \
         the collect arm's row string source-of-truth."
    );
    for variant in [
        "GalleryItem::GroupHeaderCategory",
        "GalleryItem::GroupHeader",
        "GalleryItem::IconCategoryHeader",
        "GalleryItem::Icon",
    ] {
        assert!(
            DESIGN_GALLERY.contains(variant),
            "design_gallery_item_label must handle `{}` — a new \
             GalleryItem variant without a label branch would leave \
             the collect arm panicking or dropping rows silently.",
            variant
        );
    }
}
