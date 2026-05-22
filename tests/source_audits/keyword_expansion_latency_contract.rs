//! Source-audit contracts for keyword-triggered snippet expansion latency.

use super::read_source;

const KEYWORD_MANAGER_PATH: &str = "src/keyword_manager/mod.rs";

fn keyword_expansion_region(source: &str) -> &str {
    source
        .split("// Delete trigger characters")
        .nth(1)
        .and_then(|rest| rest.split("// Paste replacement text").next())
        .expect("keyword expansion should delete trigger chars before paste_text")
}

fn keyword_timing_region(source: &str) -> &str {
    source
        .split("event = \"keyword_expansion_timing\"")
        .nth(1)
        .and_then(|rest| rest.split("\"Keyword expansion timing\"").next())
        .expect("keyword expansion should emit content-light timing logs")
}

#[test]
fn keyword_expansion_does_not_have_hardcoded_post_delete_50ms_sleep() {
    let source = read_source(KEYWORD_MANAGER_PATH);
    let region = keyword_expansion_region(&source);

    assert!(
        !region.contains("Duration::from_millis(50)")
            && !region.contains("sleep_ms(50)")
            && !region.contains("sleep(Duration::from_millis(50))"),
        "keyword expansion must not keep the redundant fixed 50 ms post-delete delay"
    );

    assert!(
        source.contains("SCRIPT_KIT_KEYWORD_POST_DELETE_DELAY_MS"),
        "post-delete delay should be env-configurable for app compatibility fallback"
    );
    assert!(
        source.contains("DEFAULT_KEYWORD_POST_DELETE_DELAY_MS: u64 = 0"),
        "keyword post-delete delay should default to zero"
    );
    assert!(
        source.contains("MAX_KEYWORD_POST_DELETE_DELAY_MS: u64 = 250"),
        "keyword post-delete delay fallback should be bounded"
    );
}

#[test]
fn keyword_expansion_logs_content_light_phase_timings() {
    let source = read_source(KEYWORD_MANAGER_PATH);
    let region = keyword_timing_region(&source);

    for required in [
        "keyword_expansion_timing",
        "chars_to_delete",
        "trigger_len",
        "replacement_len",
        "stop_delay_ms",
        "delete_ms",
        "post_delete_delay_ms",
        "paste_ms",
        "total_ms",
        "success",
    ] {
        assert!(
            source.contains(required),
            "keyword timing log should include content-light field: {required}"
        );
    }

    for forbidden in [
        "trigger = %",
        "trigger = ?",
        "replacement = %",
        "replacement = ?",
        "text = %",
        "text = ?",
    ] {
        assert!(
            !region.contains(forbidden),
            "keyword timing logs must not expose trigger or replacement content: {forbidden}"
        );
    }
}
