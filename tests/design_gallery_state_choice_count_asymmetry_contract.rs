//! Source-level contract for the Run 9 Pass #9 Fix of
//! `fix-designgallery-state-choicecount-collapses-with-filter`.
//!
//! Pre-fix, the `AppView::DesignGalleryView` arm of `collect_state`
//! (`src/prompt_handler/mod.rs`) computed a single `total_items`
//! binding from `SeparatorStyle::count() + total_icon_count() + 8 + 6`
//! (a static sum unrelated to what the renderer actually builds) and
//! returned it in BOTH the `choice_count` and `visible_choice_count`
//! slots of the state tuple. Live receipts on session pid 14780
//! confirmed the collapse in the opposite direction from the
//! EmojiPicker pre-fix shape: `triggerBuiltin design-gallery` +
//! `getState` → `choiceCount:85, visibleChoiceCount:85` (correct,
//! empty filter), but `setFilter "icon"` + `getState` →
//! `choiceCount:85, visibleChoiceCount:85` — `visibleChoiceCount`
//! did NOT narrow, despite the renderer filtering the visible list.
//! This violated the `removed-docs` §"Query and introspection"
//! invariant:
//!
//!   > `stateResult` carries both `choiceCount` (total dataset) and
//!   > `visibleChoiceCount` (filter-aware). The two slots MUST NOT
//!   > both bind to a single filter-agnostic expression — doing so
//!   > either collapses both counts (pre-Pass-#6 EmojiPicker) or
//!   > pins `visibleChoiceCount` to the dataset ceiling regardless
//!   > of filter (pre-Pass-#9 DesignGallery).
//!
//! The fix extracts shared helpers at the top of
//! `src/render_builtins/design_gallery.rs`:
//!   - `build_gallery_items() -> Vec<GalleryItem>` — canonical
//!     construction of the rendered list, iterating
//!     `GroupHeaderCategory::all()` and `IconCategory::all()`.
//!   - `gallery_item_matches(&GalleryItem, &str) -> bool` — the
//!     filter predicate, lowercased `.contains()` on the name and
//!     description fields.
//!   - `design_gallery_total_items() -> usize` — the dataset
//!     ceiling (filter-agnostic).
//!   - `design_gallery_filtered_len(&str) -> usize` — the
//!     filter-narrowed count.
//!
//! The renderer now calls the shared helpers instead of inlining
//! the enum + build + filter logic. The `collect_state` arm calls
//! `design_gallery_total_items()` into the `choice_count` slot and
//! `design_gallery_filtered_len(filter)` into the
//! `visible_choice_count` slot — distinct bindings, filter-aware
//! narrowing.
//!
//! Live post-fix receipts (session pid <rebuilt>, after
//! `cargo build` hot-swap):
//!   - setFilter ""     → choiceCount:N, visibleChoiceCount:N
//!   - setFilter "icon" → choiceCount:N, visibleChoiceCount:M (M<N)
//!
//! This contract pins both sites so a future "helpful" consolidation
//! refactor cannot silently re-collapse the counts. The refactor
//! threat is concrete: a contributor extracting a single
//! `design_gallery_count(filter: &str) -> usize` helper used in both
//! slots would flip the invariant; a contributor renaming
//! `design_gallery_filtered_len` without updating the arm would
//! silently fall back to the dataset ceiling.

const PROMPT_HANDLER: &str = include_str!("../src/prompt_handler/mod.rs");
const DESIGN_GALLERY: &str = include_str!("../src/render_builtins/design_gallery.rs");

/// Returns the byte range of the `AppView::DesignGalleryView` arm
/// inside `collect_state`. End is the next sibling `AppView` arm.
fn design_gallery_state_arm() -> &'static str {
    let start = PROMPT_HANDLER
        .find("AppView::DesignGalleryView {\n                        filter,")
        .expect(
            "src/prompt_handler/mod.rs must contain an \
             `AppView::DesignGalleryView` arm inside `collect_state` \
             destructuring `filter,` on the line after the opening \
             brace. Any other shape (binding via `..` or reading \
             `self.filter_text`) would break the Run 9 Pass #9 Fix \
             that pins `visibleChoiceCount` to the renderer's \
             filter-narrowed count.",
        );
    let end_rel = PROMPT_HANDLER[start..]
        .find("\n                    #[cfg(feature = \"storybook\")]")
        .or_else(|| PROMPT_HANDLER[start..].find("\n                    AppView::ScratchPadView"))
        .expect(
            "`AppView::DesignGalleryView` state arm must be followed \
             by `#[cfg(feature = \"storybook\")]` or \
             `AppView::ScratchPadView` — sibling-variant reorder must \
             update this contract's end anchor.",
        );
    &PROMPT_HANDLER[start..start + end_rel]
}

#[test]
fn design_gallery_state_arm_derives_dataset_count_from_shared_helper() {
    // The `choice_count` slot MUST be populated from a `dataset_count`
    // binding that calls `design_gallery_total_items()`. A static sum
    // (`SeparatorStyle::count() + total_icon_count() + 8 + 6`) would
    // drift against the renderer and reintroduce the pre-fix shape.
    let body = design_gallery_state_arm();
    assert!(
        body.contains("let dataset_count = crate::design_gallery_total_items();"),
        "DesignGalleryView state arm must bind `dataset_count` from \
         `crate::design_gallery_total_items()`. Any \
         static sum (`SeparatorStyle::count() + total_icon_count() \
         + ...`) would drift with renderer edits — the pre-Pass-#9 \
         bug shape. Arm body was:\n{}",
        body
    );
    assert!(
        !body.contains("SeparatorStyle::count()"),
        "DesignGalleryView state arm must NOT reference \
         `SeparatorStyle::count()` — that was the pre-Pass-#9 \
         static-sum shape. Use the shared \
         `design_gallery_total_items()` helper instead."
    );
}

#[test]
fn design_gallery_state_arm_derives_visible_count_from_filter_helper() {
    // The `visible_choice_count` slot MUST be populated from a
    // `visible_count` binding that calls
    // `design_gallery_filtered_len(filter)`. Losing this call would
    // silently make `visibleChoiceCount` match the dataset size,
    // exactly the pre-Pass-#9 bug shape.
    let body = design_gallery_state_arm();
    assert!(
        body.contains("let visible_count = crate::design_gallery_filtered_len(filter);"),
        "DesignGalleryView state arm must bind `visible_count` via \
         `crate::design_gallery_filtered_len(filter)`. \
         This is the single filter-aware accessor and matches the \
         shape of the renderer's `build_gallery_items()` + \
         `gallery_item_matches()` pipeline. Arm body was:\n{}",
        body
    );
}

#[test]
fn design_gallery_state_arm_tuple_slots_are_dataset_then_visible() {
    // The state tuple slots for DesignGallery are, in order:
    //   ("designGallery".to_string(), None, None, filter.clone(),
    //    <choice_count>, <visible_choice_count>,
    //    *selected_index as i32, None)
    // The Pass #9 Fix pins `dataset_count` into slot 5 (choice_count)
    // and `visible_count` into slot 6 (visible_choice_count). A
    // refactor that swaps them would flip the Pass #2 invariant.
    let body = design_gallery_state_arm();
    assert!(
        body.contains(
            "filter.clone(),\n                            dataset_count,\n                            visible_count,"
        ),
        "DesignGalleryView state arm must construct the tuple with \
         `filter.clone(), dataset_count, visible_count,` contiguous \
         in that order. Any other slot order would silently break \
         the Pass #2 invariant `visibleChoiceCount <= choiceCount` \
         or collapse it back to the pre-fix single-count shape."
    );
}

#[test]
fn design_gallery_state_arm_forbids_single_count_tuple_shape() {
    // The pre-fix shape was a single `total_items` binding appearing
    // TWICE in the tuple (slots 5 and 6 both reading `total_items`).
    // Pin that specific regression: the arm must not contain
    // `total_items,\n...total_items,`.
    let body = design_gallery_state_arm();
    assert!(
        !body.contains("total_items,\n                            total_items,"),
        "DesignGalleryView state arm must not re-use a single \
         `total_items` binding in both tuple slots — that is the \
         pre-Pass-#9 bug shape where `visibleChoiceCount` did not \
         narrow with the filter. Use distinct `dataset_count` and \
         `visible_count` bindings."
    );
}

#[test]
fn design_gallery_module_exposes_shared_helpers() {
    // The shared helpers at the top of
    // `src/render_builtins/design_gallery.rs` MUST be present with
    // their documented signatures. Renaming or removing any of them
    // would break both the renderer and the collect_state arm.
    assert!(
        DESIGN_GALLERY.contains("pub(crate) fn build_gallery_items() -> Vec<GalleryItem>"),
        "design_gallery.rs must expose `pub(crate) fn \
         build_gallery_items() -> Vec<GalleryItem>` — this is the \
         single canonical construction of the rendered list, used \
         by both the renderer and the count helpers."
    );
    assert!(
        DESIGN_GALLERY.contains(
            "pub(crate) fn gallery_item_matches(item: &GalleryItem, filter_lower: &str) -> bool"
        ),
        "design_gallery.rs must expose `pub(crate) fn \
         gallery_item_matches(item: &GalleryItem, filter_lower: \
         &str) -> bool` — the single filter predicate used by both \
         the renderer and `design_gallery_filtered_len`."
    );
    assert!(
        DESIGN_GALLERY.contains("pub(crate) fn design_gallery_total_items() -> usize"),
        "design_gallery.rs must expose `pub(crate) fn \
         design_gallery_total_items() -> usize` — the dataset \
         ceiling used in the `choice_count` slot of stateResult."
    );
    assert!(
        DESIGN_GALLERY.contains("pub(crate) fn design_gallery_filtered_len(filter: &str) -> usize"),
        "design_gallery.rs must expose `pub(crate) fn \
         design_gallery_filtered_len(filter: &str) -> usize` — the \
         filter-narrowed count used in the `visible_choice_count` \
         slot of stateResult."
    );
}

#[test]
fn design_gallery_renderer_uses_shared_builder() {
    // The renderer MUST call `build_gallery_items()` and
    // `gallery_item_matches(item, &filter_lower)` instead of
    // inlining the enum + build + filter logic. Re-inlining the
    // logic would silently drift against the `collect_state` arm
    // — the exact failure mode this pass closes.
    assert!(
        DESIGN_GALLERY.contains("let gallery_items = build_gallery_items();"),
        "render_design_gallery must call the shared \
         `build_gallery_items()` helper. Inlining the build loop \
         would allow renderer-vs-state drift on every future edit."
    );
    assert!(
        DESIGN_GALLERY.contains(".filter(|(_, item)| gallery_item_matches(item, &filter_lower))"),
        "render_design_gallery must use `gallery_item_matches` in \
         its filter closure. Inlining a divergent match expression \
         would silently skew `filtered_len` away from \
         `design_gallery_filtered_len`."
    );
}
