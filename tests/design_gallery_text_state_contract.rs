const DESIGN_GALLERY: &str = include_str!("../src/render_builtins/design_gallery.rs");

#[test]
fn design_gallery_empty_copy_is_modeled_by_filter_state() {
    assert!(
        DESIGN_GALLERY.contains("enum DesignGalleryEmptyState"),
        "Design Gallery empty copy should be modeled as named states"
    );
    assert!(
        DESIGN_GALLERY.contains("DesignGalleryEmptyState::from_filter(&filter)"),
        "Design Gallery renderer should derive empty copy from filter state"
    );
    assert!(
        DESIGN_GALLERY.contains("Self::EmptyCatalog => \"No design variations available\""),
        "empty catalog copy should not imply a filter mismatch"
    );
    assert!(
        DESIGN_GALLERY.contains("Self::NoFilterMatches => \"No designs match your filter\""),
        "non-empty filter copy should stay filter-specific"
    );
}

#[test]
fn design_gallery_input_and_count_copy_are_modeled() {
    assert!(
        DESIGN_GALLERY.contains("fn design_gallery_input_display("),
        "Design Gallery placeholder/input copy should live in a named helper"
    );
    assert!(
        DESIGN_GALLERY.contains("Self::design_gallery_input_display(&filter)"),
        "renderer should use the input display helper"
    );
    assert!(
        DESIGN_GALLERY.contains("fn design_gallery_count_label("),
        "Design Gallery count copy should live in a named helper"
    );
    assert!(
        DESIGN_GALLERY.contains("let suffix = if filtered_len == 1 { \"\" } else { \"s\" };")
            && DESIGN_GALLERY.contains("format!(\"{} item{}\", filtered_len, suffix)"),
        "Design Gallery count helper should avoid '1 items'"
    );
    assert!(
        DESIGN_GALLERY.contains("Self::design_gallery_count_label(filtered_len)"),
        "renderer should use the count label helper"
    );
}
