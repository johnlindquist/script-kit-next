const FILTER_INPUT_CORE: &str = include_str!("../src/app_impl/filter_input_core.rs");
const STARTUP: &str = include_str!("../src/app_impl/startup.rs");
const GHOST: &str = include_str!("../src/scripts/search/ghost.rs");

fn fn_body(source: &str, signature: &str) -> String {
    let start = source.find(signature).expect("signature must exist");
    let rest = &source[start..];
    let mut depth = 0usize;
    let mut seen_open = false;
    for (idx, ch) in rest.char_indices() {
        match ch {
            '{' => {
                seen_open = true;
                depth += 1;
            }
            '}' if seen_open => {
                depth -= 1;
                if depth == 0 {
                    return rest[..=idx].to_string();
                }
            }
            _ => {}
        }
    }
    panic!("function body must close");
}

#[test]
fn tab_accepts_only_first_eligible_ghost_word() {
    let body = fn_body(FILTER_INPUT_CORE, "pub(crate) fn accept_ghost_prediction(");
    assert!(body.contains("prediction.accepts_tab()"));
    assert!(body.contains("first_word_acceptance_suffix"));
    assert!(body.contains("state.insert(accepted_suffix.clone()"));
    assert!(!body.contains("accept_inline_completion"));
    assert!(
        body.find("prediction.accepts_tab()") < body.find("state.insert("),
        "hint-only predictions must be rejected before Tab inserts text"
    );
}

#[test]
fn backquote_accepts_full_eligible_ghost_suggestion() {
    let body = fn_body(
        FILTER_INPUT_CORE,
        "pub(crate) fn accept_full_ghost_prediction(",
    );
    assert!(body.contains("prediction.accepts_tab()"));
    assert!(body.contains("accept_inline_completion"));
    assert!(
        body.find("prediction.accepts_tab()") < body.find("accept_inline_completion"),
        "hint-only predictions must be rejected before Backquote accepts the full suffix"
    );

    assert!(STARTUP.contains("is_backquote_key"));
    assert!(STARTUP.contains("accept_full_ghost_prediction(window, cx)"));
    assert!(STARTUP.contains("matches!(this.current_view, AppView::ScriptList)"));
}

#[test]
fn first_word_suffix_helper_preserves_leading_space_until_first_word_end() {
    assert!(GHOST.contains("pub fn first_word_acceptance_suffix("));
    assert!(GHOST.contains("if saw_non_whitespace"));
    assert!(GHOST.contains("return &suffix[..idx];"));
}
