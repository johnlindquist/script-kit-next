const COMMON: &str = include_str!("../src/render_builtins/common.rs");
const FAVORITES: &str = include_str!("../src/render_builtins/favorites.rs");
const AI_PRESETS: &str = include_str!("../src/render_builtins/ai_presets.rs");

fn function_section<'a>(source: &'a str, name: &str, next_marker: &str) -> &'a str {
    let start = source
        .find(name)
        .unwrap_or_else(|| panic!("missing function marker `{name}`"));
    let after = &source[start..];
    let end = after.find(next_marker).unwrap_or(after.len());
    &after[..end]
}

#[test]
fn generic_filterable_surfaces_share_search_shell() {
    let helper = function_section(
        COMMON,
        "render_generic_filterable_search_surface(",
        "/// Emit a structured scroll log line",
    );

    assert!(
        helper.contains("render_search_input()"),
        "generic filterable helper should use the shared main search input"
    );
    assert!(
        helper.contains("render_minimal_list_prompt_shell_with_footer("),
        "generic filterable helper should use the shared minimal-list shell"
    );
    assert!(
        helper.contains("count_label"),
        "generic filterable helper should own the count-label chrome"
    );
}

#[test]
fn favorites_and_ai_presets_delegate_to_generic_filterable_shell() {
    for (name, source, function, key_context) in [
        (
            "favorites",
            FAVORITES,
            "fn render_favorites_browse(",
            "\"favorites\"",
        ),
        (
            "search_ai_presets",
            AI_PRESETS,
            "fn render_search_ai_presets(",
            "\"search_ai_presets\"",
        ),
    ] {
        let render = function_section(source, function, "\n    ///");
        assert!(
            render.contains("render_generic_filterable_search_surface("),
            "{name} should delegate shared search chrome to the generic filterable helper"
        );
        assert!(
            render.contains(key_context),
            "{name} should preserve its key context"
        );
        assert!(
            render.contains("main_window_footer_slot("),
            "{name} should preserve native-footer ownership routing"
        );
        assert!(
            !render.contains("Input::new(&self.gpui_input_state)"),
            "{name} should not rebuild bespoke search input chrome"
        );
    }
}
