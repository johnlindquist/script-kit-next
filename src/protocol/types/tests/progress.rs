use super::*;

#[test]
fn test_progress_eta_display_label_for_estimating_and_unknown() {
    assert_eq!(ProgressEta::Estimating.display_label(), "estimating");
    assert_eq!(ProgressEta::Unknown.display_label(), "unknown");
}

#[test]
fn test_progress_eta_display_label_trims_known_value() {
    assert_eq!(
        ProgressEta::Known(" 25s ".to_string()).display_label(),
        "25s"
    );
}

#[test]
fn test_progress_eta_display_label_falls_back_to_unknown_for_blank_known_value() {
    assert_eq!(
        ProgressEta::Known("   ".to_string()).display_label(),
        "unknown"
    );
}

#[test]
fn test_progress_operation_serializes_as_camel_case() {
    let value = serde_json::to_value(ProgressOperation::DownloadInstall)
        .expect("progress operation should serialize");
    assert_eq!(value, serde_json::json!("downloadInstall"));
}

#[test]
fn test_progress_eta_serializes_with_kind_tag() {
    let value = serde_json::to_value(ProgressEta::Known("2m".to_string()))
        .expect("progress eta should serialize");
    assert_eq!(
        value,
        serde_json::json!({"kind":"known","value":"2m"})
    );
}
