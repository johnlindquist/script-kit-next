use std::collections::BTreeSet;

use super::read_source;

fn function_body<'a>(source: &'a str, signature: &str) -> &'a str {
    let start = source
        .find(signature)
        .unwrap_or_else(|| panic!("missing function signature `{signature}`"));
    let open = source[start..]
        .find('{')
        .map(|offset| start + offset)
        .unwrap_or_else(|| panic!("missing opening brace for `{signature}`"));
    let mut depth = 0usize;
    for (offset, byte) in source.as_bytes()[open..].iter().enumerate() {
        match byte {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return &source[open..=open + offset];
                }
            }
            _ => {}
        }
    }
    panic!("missing closing brace for `{signature}`");
}

fn app_view_variants() -> BTreeSet<String> {
    let source = read_source("src/main_sections/app_view_state.rs");
    let enum_start = source
        .find("enum AppView {")
        .expect("AppView enum should exist");
    let enum_body = source[enum_start..]
        .split("/// Exhaustive host-level header/input ownership")
        .next()
        .expect("AppView enum should precede the header/input policy docs");

    enum_body
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim_start();
            if !line.starts_with("    ")
                || trimmed.starts_with("//")
                || trimmed.starts_with("#[")
                || trimmed.starts_with("///")
                || trimmed.is_empty()
            {
                return None;
            }

            let name = trimmed
                .split(|ch: char| ch == '{' || ch == ',' || ch.is_whitespace())
                .next()
                .unwrap_or("");
            if name.chars().next().is_some_and(char::is_uppercase) {
                Some(name.to_string())
            } else {
                None
            }
        })
        .collect()
}

#[test]
fn every_app_view_is_classified_for_main_window_header_input_policy() {
    let actual = app_view_variants();
    let source = read_source("src/main_sections/app_view_state.rs");
    let policy = function_body(
        &source,
        "pub(crate) fn main_view_header_input_policy(&self)",
    );

    for variant in actual {
        assert!(
            policy.contains(&format!("AppView::{variant}")),
            "new AppView variant {variant} must be classified by main_view_header_input_policy"
        );
    }
    assert!(
        !policy.contains("_ =>"),
        "main_view_header_input_policy must stay exhaustive so rustc catches new views"
    );
}

#[test]
fn searchable_main_window_surfaces_have_shared_runtime_layout_context_zone() {
    // build_layout_info is the single owner of main-view header geometry;
    // component bounds only adds per-surface detail below it.
    let layout_info = read_source("src/app_layout/build_layout_info.rs");

    for (name, model) in [("layout info", layout_info)] {
        assert!(model.contains("main_view_header_input_policy()"));
        assert!(model.contains("main_view_header_metrics(menu_def, input_height)"));
        for component in [
            "MainViewHeader",
            "MainViewContextZone",
            "MainViewInput",
            "MainViewMain",
        ] {
            assert!(model.contains(component), "{name} must emit {component}");
        }
        assert!(
            !model.contains("let main_view_has_context_zone = matches!"),
            "{name} must not duplicate the exhaustive AppView policy"
        );
    }
}

#[test]
fn prompt_and_child_content_surfaces_use_root_shared_context_header_fallback() {
    let render_impl = read_source("src/main_sections/render_impl.rs");
    let app_view_state = read_source("src/main_sections/app_view_state.rs");
    let ownership_body = function_body(
        &app_view_state,
        "pub(crate) fn main_view_header_input_policy(&self)",
    );

    assert!(
        render_impl.contains(".uses_view_owned_main_window_shell()"),
        "root renderer must ask the AppView ownership policy before wrapping content"
    );
    assert!(
        render_impl.contains("render_main_view_slot_line_header("),
        "root renderer must provide the line above the input for prompt/script child surfaces"
    );

    for variant in [
        "AppView::ArgPrompt { .. }",
        "AppView::DivPrompt { .. }",
        "AppView::FormPrompt { .. }",
        "AppView::TermPrompt { .. }",
        "AppView::EditorPrompt { .. }",
        "AppView::SelectPrompt { .. }",
        "AppView::PathPrompt { .. }",
        "AppView::EnvPrompt { .. }",
        "AppView::DropPrompt { .. }",
        "AppView::TemplatePrompt { .. }",
        "AppView::ChatPrompt { .. }",
        "AppView::MiniPrompt { .. }",
        "AppView::MicroPrompt { .. }",
        "AppView::NamingPrompt { .. }",
        "AppView::CreateAiPresetView { .. }",
        "AppView::QuickTerminalView { .. }",
        "AppView::ScratchPadView { .. }",
    ] {
        assert!(
            ownership_body.contains(variant),
            "{variant} must be explicitly classified by the root shared context-header fallback"
        );
    }
    assert!(ownership_body.contains("MainViewHeaderInputPolicy::RootContextOnly"));
}

#[test]
fn split_preview_builtins_do_not_use_legacy_expanded_header_scaffolds() {
    for path in [
        "src/render_builtins/browser_history.rs",
        "src/render_builtins/notes_browse.rs",
        "src/render_builtins/sdk_reference.rs",
        "src/render_builtins/script_templates.rs",
    ] {
        let source = read_source(path);
        assert!(
            source.contains("render_builtin_split_main_content("),
            "{path} should share split main-slot content geometry"
        );
        assert!(
            !source.contains("render_expanded_view_scaffold_with_footer(")
                && !source.contains("render_expanded_view_scaffold_with_hints("),
            "{path} must not rebuild local header padding through expanded-view scaffolds"
        );
    }
}
