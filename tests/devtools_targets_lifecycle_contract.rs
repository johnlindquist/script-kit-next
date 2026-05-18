const TARGETS_TS: &str = include_str!("../scripts/devtools/targets.ts");

#[test]
fn targets_preserve_parsed_session_lifecycle_errors() {
    for needle in [
        "parsedError",
        "sessionLifecycle",
        "lifecycleDetails",
        "app_process_dead_before_rpc",
        "forwarder_dead_before_rpc",
    ] {
        assert!(
            TARGETS_TS.contains(needle),
            "targets.ts must preserve lifecycle error detail {needle}"
        );
    }
}
#[test]
fn targets_classify_lifecycle_separately_from_timeouts() {
    for needle in [
        "blocked-by-session-lifecycle",
        "hasSessionLifecycleError(errors)",
        "lifecycleCodes",
    ] {
        assert!(
            TARGETS_TS.contains(needle),
            "targets.ts must classify session lifecycle explicitly: {needle}"
        );
    }
}
