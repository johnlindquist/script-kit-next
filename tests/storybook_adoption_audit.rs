#[test]
fn catalog_requires_representation_and_adoption_metadata() {
    let adoption = include_str!("../src/storybook/adoption.rs");
    let audit = include_str!("../src/storybook/audit_report.rs");
    let diagnostics = include_str!("../src/storybook/diagnostics.rs");
    let main_menu = include_str!("../src/storybook/main_menu_variations/mod.rs");

    for token in [
        "canonicalState",
        "adoptableVariation",
        "representation",
        "liveSurface",
        "presenterFixture",
        "adoptedSurfaceCoverage",
    ] {
        assert!(
            adoption.contains(token)
                || audit.contains(token)
                || diagnostics.contains(token)
                || main_menu.contains(token),
            "storybook catalog missing required metadata token: {token}"
        );
    }

    assert!(
        !main_menu.contains("runtimeFixture"),
        "runtimeFixture must not be registered in the primary main-menu catalog"
    );
}
