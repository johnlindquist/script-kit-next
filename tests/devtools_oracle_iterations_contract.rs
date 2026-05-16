const ORACLE_INDEX: &str = include_str!(
    "../.agents/skills/script-kit-devtools/references/oracle-devtools-scenario-index.md"
);
const ORACLE_RAW: &str = include_str!(
    "../.agents/skills/script-kit-devtools/references/oracle-devtools-scenario-iterations.md"
);

#[test]
fn oracle_iteration_index_covers_all_50_corrected_iterations() {
    for id in 1..=50 {
        let needle = format!("| {:02} |", id);
        assert!(
            ORACLE_INDEX.contains(&needle),
            "missing Oracle scenario iteration {id:02}"
        );
    }
}

#[test]
fn oracle_iteration_index_records_batch_provenance() {
    for session in [
        "devtools-oracle-iterations-actions-popups",
        "devtools-oracle-iterations-prompts-resize",
        "devtools-oracle-iterations-notes-acp",
        "devtools-oracle-iterations-filterable-data",
        "devtools-oracle-iterations-native-a11y",
    ] {
        assert!(
            ORACLE_INDEX.contains(session),
            "missing Oracle session provenance for {session}"
        );
        assert!(
            ORACLE_RAW.contains(session),
            "missing raw Oracle output section for {session}"
        );
    }
}

#[test]
fn oracle_iteration_index_preserves_devtools_boundary() {
    for primitive in [
        "devtools.inspect",
        "devtools.measure",
        "devtools.compare",
        "devtools.act",
        "devtools.investigate",
    ] {
        assert!(
            ORACLE_INDEX.contains(primitive),
            "Oracle index must keep the DevTools primitive boundary visible: {primitive}"
        );
    }
}
