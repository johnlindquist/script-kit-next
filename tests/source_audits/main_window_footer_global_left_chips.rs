//! Source audit for the global main-window footer left chips (Spine cwd +
//! Agent·Model).
//!
//! The reported bug was that the two bottom-left footer chips vanished / flashed
//! a gap when switching between the main menu (ScriptList) and Agent Chat
//! (AcpChatView), because each surface built its own footer button list and the
//! ACP path only re-added cwd as a *visual-only* `left_info.cwd_chip`.
//!
//! The fix makes Cwd + Agent·Model real `FooterButtonConfig` entries prepended
//! centrally for every surface that shows them, so they are a single source of
//! truth and persist with no gap. This audit freezes that structure so a future
//! edit cannot silently regress back to per-surface chip assembly or the
//! visual-only cwd rail.

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
fn main_window_footer_config_prepends_global_left_chips_once() {
    let src = read("src/app_impl/ui_window.rs");
    assert!(
        src.contains("fn global_main_window_left_chip_buttons("),
        "global left-chip builder helper must exist"
    );
    assert!(
        src.contains("fn prepend_global_main_window_left_chips("),
        "global left-chip prepend helper must exist"
    );
    let body = fn_body(&src, "fn main_window_footer_config_with_cx(");
    assert!(
        body.contains("self.prepend_global_main_window_left_chips("),
        "the central footer config must prepend the global left chips"
    );
    assert!(
        body.contains("current_view_shows_global_left_chips()"),
        "the prepend must be gated to surfaces that show the global chips"
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
fn acp_footer_buttons_remain_snapshot_owned() {
    // The frozen ACP footer snapshot contract stays the source of ACP's own
    // buttons; the global chips wrap it from outside, not inside.
    let src = read("src/app_impl/ui_window.rs");
    let body = fn_body(&src, "fn acp_footer_buttons(");
    assert!(
        body.contains("self.acp_footer_snapshot"),
        "ACP footer buttons must still come from the snapshot"
    );
    assert!(
        !body.contains("FooterAction::Cwd"),
        "ACP footer buttons must not embed the global Cwd chip"
    );
    assert!(
        !body.contains("FooterAction::AgentModel"),
        "ACP footer buttons must not embed the global Agent·Model chip"
    );
}

#[test]
fn acp_enrichment_suppresses_visual_cwd_when_real_chips_exist() {
    let src = read("src/app_impl/ui_window.rs");
    let body = fn_body(&src, "fn enrich_footer_config_with_acp_info(");
    assert!(
        body.contains("has_real_global_left_chips"),
        "enrichment must detect when real global chips are present"
    );
    assert!(
        body.contains("config.left_info = None"),
        "enrichment must drop the left-info rail when real chips exist"
    );
    // The visual-only cwd_chip injection that caused the flash/overlap is gone.
    assert!(
        !body.contains("cwd_chip: self.global_footer_cwd_chip()"),
        "enrichment must not inject a visual-only cwd_chip"
    );
    assert!(
        !body.contains("left_info.cwd_chip = self.global_footer_cwd_chip()"),
        "enrichment must not inject a visual-only cwd_chip"
    );
}

#[test]
fn acp_status_dot_threads_through_agent_model_chip_not_left_info() {
    // The ACP streaming/status dot must ride as a reserved leading dot INSIDE
    // the real Agent·Model footer button (only on Agent Chat), NOT via the old
    // visual-only left_info rail — that rail must stay suppressed when real
    // chips exist so the overlap/flash regression cannot return.
    let ui = read("src/app_impl/ui_window.rs");
    let footer = read("src/footer_popup.rs");

    let global = fn_body(&ui, "fn global_main_window_left_chip_buttons(");
    assert!(
        global.contains("AppView::AcpChatView"),
        "only Agent Chat receives a status dot lane (ScriptList stays dot-free)"
    );
    assert!(
        global.contains("acp_footer_dot_status"),
        "Agent·Model chip must use the host-cached ACP footer dot status"
    );
    assert!(
        global.contains(".leading_dot("),
        "ACP status must be threaded as FooterButtonConfig::leading_dot"
    );

    let enrich = fn_body(&ui, "fn enrich_footer_config_with_acp_info(");
    assert!(
        enrich.contains("config.left_info = None"),
        "real global chips must still suppress the old left-info rail"
    );

    assert!(
        footer.contains("leading_dot: Option<FooterDotStatus>"),
        "FooterButtonConfig must carry a leading_dot field"
    );
    assert!(
        footer.contains("fn make_footer_hint_leading_dot_view"),
        "native footer must render the dot inside the button via a dedicated view"
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
