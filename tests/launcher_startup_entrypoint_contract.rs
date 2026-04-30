//! Source-level contract for AURP-02 in `lat.md/agent-understanding-regression-plan.md`.
//!
//! The live launcher startup path is `src/app_impl/startup.rs`, imported by
//! `src/app_impl/mod.rs`. The `startup_new_*` files remain as legacy
//! source-audit parity fragments and must not be mistaken for production wiring.

const APP_IMPL_MOD: &str = include_str!("../src/app_impl/mod.rs");
const STARTUP_RS: &str = include_str!("../src/app_impl/startup.rs");
const ARCHITECTURE: &str = include_str!("../lat.md/architecture.md");

// @lat: [[lat.md/architecture#Architecture#Launcher Startup Source Of Truth]]
#[test]
fn app_impl_wires_startup_rs_as_the_live_startup_module() {
    assert!(
        APP_IMPL_MOD.contains("#[path = \"startup.rs\"]\nmod startup;"),
        "src/app_impl/mod.rs must wire the live launcher startup module to startup.rs"
    );
    assert!(
        !APP_IMPL_MOD.contains("startup_new_"),
        "src/app_impl/mod.rs must not wire startup_new_* fragments as production modules"
    );
}

#[test]
fn live_startup_rs_owns_script_list_app_new() {
    assert!(
        STARTUP_RS.contains("impl ScriptListApp")
            && STARTUP_RS.contains("pub(crate) fn new(")
            && STARTUP_RS.contains("scripts::read_scripts_report()"),
        "src/app_impl/startup.rs must own ScriptListApp::new and use the current script validation report loading path"
    );
}

#[test]
fn architecture_quarantines_startup_new_fragments_as_source_audits() {
    assert!(
        ARCHITECTURE
            .contains("The live launcher startup implementation is `src/app_impl/startup.rs`"),
        "lat.md/architecture.md must name startup.rs as the live launcher startup implementation"
    );
    assert!(
        ARCHITECTURE.contains("`src/app_impl/startup_new_*.rs` files are legacy source-audit parity fragments"),
        "lat.md/architecture.md must quarantine startup_new_* as legacy source-audit parity fragments"
    );
}
