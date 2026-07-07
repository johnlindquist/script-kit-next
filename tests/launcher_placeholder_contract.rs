use std::fs;

fn read_source(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|err| panic!("failed to read {path}: {err}"))
}

/// Extract the string literal assigned to `ROOT_LAUNCHER_PLACEHOLDER` in the
/// given source text. The constant is defined twice by design — once in
/// `src/lib.rs` (lib crate root) and once in `src/main_sections/app_state.rs`
/// (include!-merged into the bin crate root) — so the load-bearing invariant
/// is that the two copies never drift apart.
fn extract_placeholder(source: &str, path: &str) -> String {
    let marker = "ROOT_LAUNCHER_PLACEHOLDER: &str =";
    let start = source
        .find(marker)
        .unwrap_or_else(|| panic!("{path} should define ROOT_LAUNCHER_PLACEHOLDER"));
    let rest = &source[start + marker.len()..];
    let open = rest
        .find('"')
        .unwrap_or_else(|| panic!("{path}: placeholder should be a string literal"));
    let rest = &rest[open + 1..];
    let close = rest
        .find('"')
        .unwrap_or_else(|| panic!("{path}: unterminated placeholder literal"));
    rest[..close].to_string()
}

/// WHY: the root launcher placeholder is the only always-visible teacher of
/// the main-input sigil grammar. It exists as two constants (lib crate root +
/// bin crate root via include!), and they historically drifted into three
/// different sigil sets. The user-facing contract is: one copy, everywhere,
/// and it must advertise the core sigil families that actually work.
#[test]
fn root_launcher_placeholder_is_identical_in_lib_and_bin() {
    let lib = extract_placeholder(&read_source("src/lib.rs"), "src/lib.rs");
    let bin = extract_placeholder(
        &read_source("src/main_sections/app_state.rs"),
        "src/main_sections/app_state.rs",
    );
    assert_eq!(
        lib, bin,
        "lib.rs and app_state.rs root launcher placeholders must not drift"
    );
}

#[test]
fn root_launcher_placeholder_teaches_core_sigils() {
    let placeholder = extract_placeholder(&read_source("src/lib.rs"), "src/lib.rs");
    for sigil in ["@", "/", ";", ":"] {
        assert!(
            placeholder.contains(sigil),
            "placeholder should advertise the `{sigil}` sigil family; got {placeholder:?}"
        );
    }
}

#[test]
fn root_launcher_placeholder_replaces_theme_identifier_resets() {
    let startup = read_source("src/app_impl/startup.rs");
    let startup_new = read_source("src/app_impl/startup_new_prelude.rs");
    let lifecycle_reset = read_source("src/app_impl/lifecycle_reset.rs");
    let registries_state = read_source("src/app_impl/registries_state.rs");
    let theme_focus = read_source("src/app_impl/theme_focus.rs");

    for (path, source) in [
        ("src/app_impl/startup.rs", startup.as_str()),
        ("src/app_impl/startup_new_prelude.rs", startup_new.as_str()),
        ("src/app_impl/lifecycle_reset.rs", lifecycle_reset.as_str()),
        (
            "src/app_impl/registries_state.rs",
            registries_state.as_str(),
        ),
        ("src/app_impl/theme_focus.rs", theme_focus.as_str()),
    ] {
        assert!(
            source.contains("effective_main_input_placeholder()"),
            "{path} should reset the root launcher placeholder through the live copy override helper"
        );
    }

    assert!(
        !startup.contains("MainMenuThemeVariant::default().placeholder()"),
        "startup should not expose the theme identifier as the root placeholder"
    );
    assert!(
        !startup_new.contains("AccentVariation::default().placeholder()"),
        "new startup path should not expose the accent identifier as the root placeholder"
    );
    assert!(
        !lifecycle_reset.contains("current_main_menu_theme.placeholder()"),
        "script-exit reset should restore the selected launcher guidance copy"
    );
    assert!(
        !registries_state.contains("current_main_menu_theme.placeholder()"),
        "focus reset should restore the selected launcher guidance copy"
    );
    assert!(
        !theme_focus.contains("new_theme.placeholder()"),
        "theme cycling should not replace the root launcher guidance copy"
    );
}
