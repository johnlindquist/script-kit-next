use std::fs;

#[test]
fn active_preview_panel_uses_match_aware_cache_and_emphasis_rendering() {
    let source = fs::read_to_string("src/app_render/preview_panel.rs")
        .expect("should read src/app_render/preview_panel.rs");
    assert!(
        source.contains("get_or_update_preview_cache_with_match("),
        "active preview panel must use match-aware preview cache"
    );
    assert!(
        source.contains("script_match.content_match.as_ref()"),
        "active preview panel must pass content_match into preview cache"
    );
    assert!(
        source.contains("if span.is_match_emphasis"),
        "active preview panel must render match emphasis spans"
    );
    assert!(
        source.contains("[PREVIEW_CONTEXT]"),
        "active preview panel should log preview content-match context for verification"
    );
}

#[test]
fn result_script_preview_path_still_uses_match_aware_cache() {
    let source = fs::read_to_string("src/app_render/preview_panel/result_script.rs")
        .expect("should read src/app_render/preview_panel/result_script.rs");
    assert!(
        source.contains("get_or_update_preview_cache_with_match("),
        "result_script preview path must remain match-aware"
    );
    assert!(
        source.contains("script_match.content_match.as_ref()"),
        "result_script preview path must pass content_match"
    );
    assert!(
        source.contains("if span.is_match_emphasis"),
        "result_script preview path must render emphasized spans"
    );
}
