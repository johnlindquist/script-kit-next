//! Source audit for the shared main-window context chips (Spine cwd +
//! Agent·Model).
//!
//! The reported bug was that the two context chips vanished / flashed a gap
//! when switching between the main menu (ScriptList) and Agent Chat
//! (AgentChatView), because footer paths tried to own the same context state.
//!
//! The current fix keeps Cwd + Agent·Model in the shared main-view header
//! context zone and keeps the native footer scoped to surface actions. This
//! audit freezes that ownership so a future edit cannot silently regress back
//! to per-surface footer chip assembly or the visual-only cwd rail.

use std::fs;
use std::path::Path;

fn read(rel: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
}

/// Returns the body of a `fn <header...` declaration: everything from the `{`
/// that opens the function up to its matching `}` (brace-balanced).
fn fn_body<'a>(src: &'a str, header: &str) -> &'a str {
    let start = src
        .find(header)
        .unwrap_or_else(|| panic!("function header not found: {header}"));
    let open = src[start..]
        .find('{')
        .map(|offset| start + offset)
        .unwrap_or_else(|| panic!("no opening brace after {header}"));
    let bytes = src.as_bytes();
    let mut depth = 0usize;
    let mut idx = open;
    while idx < bytes.len() {
        match bytes[idx] {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return &src[open..=idx];
                }
            }
            _ => {}
        }
        idx += 1;
    }
    panic!("unbalanced braces in {header}");
}

#[test]
fn native_footer_config_does_not_own_context_chips() {
    let src = read("src/app_impl/ui_window.rs");
    assert!(
        src.contains("pub(crate) fn global_footer_cwd_chip("),
        "global cwd context helper must exist"
    );
    assert!(
        src.contains("pub(crate) fn agent_model_footer_label("),
        "global Agent·Model context helper must exist"
    );
    let body = fn_body(&src, "fn main_window_footer_config_with_cx(");
    assert!(
        !body.contains("FooterAction::Cwd") && !body.contains("FooterAction::AgentModel"),
        "native footer config must not own the shared context chips"
    );
}

#[test]
fn standard_footer_no_longer_owns_global_left_chips() {
    let src = read("src/app_impl/ui_window.rs");
    let body = fn_body(&src, "fn standard_main_window_footer_buttons(");
    assert!(
        !body.contains("FooterAction::Cwd"),
        "standard footer must not build the global Cwd chip (now central)"
    );
    assert!(
        !body.contains("FooterAction::AgentModel"),
        "standard footer must not build the global Agent·Model chip (now central)"
    );
}

#[test]
fn native_footer_left_pins_cwd_and_agent_model() {
    // The native AppKit footer must keep recognizing Cwd + AgentModel as
    // left-pinned, otherwise prepended chips would render on the right.
    let src = read("src/footer_popup.rs");
    assert!(
        src.contains("fn is_footer_left_pinned_button("),
        "footer_popup must use the generic left-pinned predicate name"
    );
    assert!(
        src.contains("FooterAction::Cwd | FooterAction::AgentModel"),
        "footer_popup must left-pin Cwd + AgentModel"
    );
}
