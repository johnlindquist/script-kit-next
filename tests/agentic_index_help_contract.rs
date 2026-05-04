//! Source-level contract for agentic index command discovery.

const INDEX: &str = include_str!("../scripts/agentic/index.ts");

#[test]
fn index_routes_surface_navigate_to_standalone_script() {
    assert!(
        INDEX.contains("case \"surface-navigate\""),
        "index.ts must expose the surface-navigate recipe"
    );
    assert!(
        INDEX.contains("\"scripts/agentic/surface-navigator.ts\""),
        "surface-navigate must delegate to the standalone navigator"
    );
}

#[test]
fn index_json_help_lists_surface_navigate() {
    assert!(
        INDEX.contains("{ name: \"surface-navigate\"")
            && INDEX.contains("\"--case\"")
            && INDEX.contains("\"--group\"")
            && INDEX.contains("\"--interact\"")
            && INDEX.contains("\"--capture\"")
            && INDEX.contains("\"--out-dir\"")
            && INDEX.contains("\"--manifest\"")
            && INDEX.contains("\"--fresh-per-case\"")
            && INDEX.contains("\"--keep-session\""),
        "help --json must advertise surface-navigate and its agent-facing flags"
    );
}

#[test]
fn index_text_help_lists_surface_navigate_example() {
    assert!(
        INDEX.contains("surface-navigate       Warm-session navigation"),
        "human help must describe surface-navigate"
    );
    assert!(
        INDEX.contains("surface-navigate --session default --group filterable-main --case all")
            && INDEX.contains("--fresh-per-case")
            && INDEX.contains("--manifest .notes/image-library/manifest.json"),
        "human help must include a usable surface-navigate example"
    );
}
