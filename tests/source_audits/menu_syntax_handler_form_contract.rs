use std::fs;

fn read(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|err| panic!("failed to read {path}: {err}"))
}

#[test]
fn menu_syntax_handler_form_snapshot_is_state_first_and_autocomplete_ready() {
    let form = read("src/menu_syntax/form.rs");
    assert!(
        form.contains("pub struct MenuSyntaxFormSnapshot")
            && form.contains("pub struct MenuSyntaxFormFieldSnapshot")
            && form.contains("pub struct MenuSyntaxFormSuggestion"),
        "menu syntax handler mode must expose a structured form snapshot for getState/devtools"
    );
    assert!(
        form.contains("tab_ai_disabled: true"),
        "handler form snapshots must explicitly record that Tab AI is disabled while the form owns Tab"
    );
    assert!(
        form.contains("tagHistory") && form.contains("schema"),
        "handler form fields must expose suggestion sources for autocomplete"
    );
    assert!(
        form.contains("apply_capture_form_field_edit")
            && form.contains("serialize_capture_invocation"),
        "handler form must own a parser-derived form-field edit path back to canonical capture text"
    );
    assert!(
        form.contains("empty_capture_invocation"),
        "bare committed handlers like ;todo and ;note must synthesize an empty invocation so fields render before a body exists"
    );
    let app = read("src/app_impl/menu_syntax_main_hint.rs");
    assert!(
        app.contains("menu_syntax_capture_form_invocation")
            && app.contains("IncompleteKind::MissingCaptureBody")
            && app.contains("empty_capture_invocation"),
        "app form snapshots and form-field edits must share the same incomplete-handler invocation path"
    );
    let state = read("src/main_sections/app_state.rs");
    assert!(
        state.contains("menu_syntax_form_input_active")
            && state.contains("menu_syntax_form_draft_field_id")
            && state.contains("menu_syntax_form_draft_value"),
        "handler forms need separate visible-form, active-field, and draft-value state so main input can keep focus until Tab arms the first field"
    );
}

#[test]
fn rendered_capture_composer_uses_form_before_instruction_rows() {
    let render = read("src/render_script_list/mod.rs");
    let form_render = render
        .find("fn render_menu_syntax_form(")
        .expect("render_menu_syntax_form must exist");
    let form_child = render
        .find("hint.form.as_ref()")
        .expect("capture hint render must include hint.form");
    let row_child = render
        .find("hint.rows.is_empty()")
        .expect("capture hint render must still include legacy rows");
    assert!(
        form_render < form_child && form_child < row_child,
        "handler form should render as the primary composer surface before legacy instructional rows"
    );
}

#[test]
fn form_mode_hides_ask_ai_hint_and_uses_quiet_focus_style() {
    let render = read("src/render_script_list/mod.rs");
    let ask_hint = render
        .find("render_launcher_ask_ai_hint")
        .expect("ScriptList header should still render the Ask AI hint outside form mode");
    let form_guard = render
        .find("handler_form_owns_input_for_render")
        .expect("render should compute handler form ownership for header chrome");
    let show_hint = render
        .find("show_launcher_ask_ai_hint")
        .expect("render should name the Ask AI hint visibility gate");
    let guarded_child = render
        .find(".when(show_launcher_ask_ai_hint")
        .expect("Ask AI hint must render through the handler-form visibility gate");
    assert!(
        form_guard < show_hint && show_hint < guarded_child && guarded_child < ask_hint,
        "handler form mode must hide the top-right Ask/Tab affordance"
    );

    let field_start = render
        .find("fn render_menu_syntax_form_field(")
        .expect("handler form field renderer must exist");
    let form_start = render
        .find("fn render_menu_syntax_form(")
        .expect("handler form renderer must exist");
    let field_renderer = &render[field_start..form_start];
    assert!(
        !field_renderer.contains("accent.selected")
            && !field_renderer.contains("rgb(accent)")
            && !field_renderer.contains("theme.colors.accent")
            && !field_renderer.contains("accent << 8"),
        "tabbing through handler fields should not use loud accent focus styling"
    );
    assert!(
        field_renderer.contains("theme.colors.ui.border")
            && field_renderer.contains("theme.colors.background.search_box")
            && field_renderer.contains("field.required && !field.satisfied"),
        "handler form focus should use quiet border/background tokens and only show missing required state"
    );
    assert!(
        field_renderer.contains("gpui_component::input::Input::new(&input)")
            && field_renderer.contains(".focus_bordered(false)")
            && field_renderer.contains("placeholder_color"),
        "handler fields should render real inputs with quiet focus chrome and dim placeholder fallback text"
    );
    assert!(
        !field_renderer.contains(".when(field.focused")
            && !field_renderer.contains(".h(px(18.0))\n                            .bg("),
        "handler fields must not draw fake focused-field cursors"
    );
    assert!(
        !render[field_start..].contains("Tab moves fields"),
        "handler form should avoid visible keyboard-instruction copy"
    );
}

#[test]
fn stdin_can_edit_menu_syntax_form_fields_for_runtime_proof() {
    let commands = read("src/stdin_commands/mod.rs");
    assert!(
        commands.contains("SetMenuSyntaxFormField")
            && commands.contains("\"setMenuSyntaxFormField\""),
        "stdin automation needs a direct form-field edit command for state-first runtime proof"
    );

    for path in [
        "src/main_entry/runtime_stdin.rs",
        "src/main_entry/runtime_stdin_match_core.rs",
        "src/main_entry/app_run_setup.rs",
    ] {
        let source = read(path);
        assert!(
            source.contains("ExternalCommand::SetMenuSyntaxFormField")
                && source.contains("update_menu_syntax_form_field"),
            "{path}: setMenuSyntaxFormField must route through the app's form sync method"
        );
    }
}

#[test]
fn tab_routes_to_handler_form_before_tab_ai_paths() {
    let form_owner = read("src/app_impl/menu_syntax_main_hint.rs");
    assert!(
        form_owner.contains("gpui_component::input::InputState::new(window, cx)")
            && form_owner.contains(".tab_navigation(true)")
            && form_owner.contains("state.focus(window, cx)")
            && form_owner.contains("actual_menu_syntax_form_focused_index"),
        "Tab routing must create real focusable handler field inputs and move actual focus handles"
    );
    assert!(
        form_owner.contains("focus_menu_syntax_main_input")
            && form_owner.contains("self.gpui_input_state")
            && form_owner.contains("state.focus(window, cx)"),
        "handler form traversal must be able to return actual focus to the main input"
    );

    let preflight = read("src/main_window_preflight/build.rs");
    let form_disabled = preflight
        .find("menu_syntax_capture_form_owns_input()")
        .expect("main-window preflight must disable Tab AI while handler form owns Tab");
    let ask_ai = preflight
        .find("MainWindowPreflightActionKind::AskAi")
        .expect("main-window preflight must still define Ask AI for normal launcher text");
    assert!(
        form_disabled < ask_ai,
        "preflight tabAction must be None for handler forms before emitting Ask AI"
    );

    let startup = read("src/app_impl/startup.rs");
    let physical_form = startup
        .find("menu_syntax_capture_form_owns_input()")
        .expect("physical Tab path must check handler form ownership");
    let physical_ai = startup
        .find("try_route_plain_tab_to_acp_context_capture")
        .expect("physical Tab path must still contain Tab AI route");
    assert!(
        physical_form < physical_ai,
        "physical Tab must move handler form focus before Tab AI can claim the key"
    );

    for path in [
        "src/main_entry/runtime_stdin.rs",
        "src/main_entry/runtime_stdin_match_simulate_key.rs",
        "src/main_entry/app_run_setup.rs",
    ] {
        let source = read(path);
        let form = source
            .find("menu_syntax_capture_form_owns_input()")
            .unwrap_or_else(|| panic!("{path} must check handler form ownership"));
        let ai = source
            .find("try_route_plain_tab_to_acp_context_capture")
            .unwrap_or_else(|| panic!("{path} must still contain Tab AI route"));
        assert!(
            form < ai,
            "{path}: simulateKey Tab must move handler form focus before Tab AI can claim the key"
        );
    }
}
