use script_kit_gpui::menu_syntax::create_capture_handler_scaffold;
use script_kit_gpui::menu_syntax::trigger_picker::OsCaptureHandlerScaffoldEffects;

#[test]
fn create_handler_writes_template_to_expected_path() {
    let temp = tempfile::tempdir().expect("tempdir");
    let scripts_dir = temp.path().join("plugins/main/scripts");
    let effects = OsCaptureHandlerScaffoldEffects;

    let created = create_capture_handler_scaffold(&effects, &scripts_dir, "gcal", false)
        .expect("create scaffold");

    assert_eq!(created.filename, "capture-gcal-handler.ts");
    assert_eq!(created.path, scripts_dir.join("capture-gcal-handler.ts"));
    let contents = std::fs::read_to_string(&created.path).expect("read scaffold");
    assert!(contents.contains("capture.v1"));
    assert!(contents.contains(r#"targets: ["gcal"]"#));
}

#[test]
fn create_handler_does_not_overwrite_existing_file() {
    let temp = tempfile::tempdir().expect("tempdir");
    let scripts_dir = temp.path().join("plugins/main/scripts");
    std::fs::create_dir_all(&scripts_dir).expect("create scripts dir");
    let existing_path = scripts_dir.join("capture-gcal-handler.ts");
    std::fs::write(&existing_path, "existing user code").expect("write existing");
    let effects = OsCaptureHandlerScaffoldEffects;

    let created = create_capture_handler_scaffold(&effects, &scripts_dir, "gcal", false)
        .expect("create scaffold");

    assert_eq!(created.filename, "capture-gcal-handler-2.ts");
    assert_eq!(created.path, scripts_dir.join("capture-gcal-handler-2.ts"));
    assert_eq!(
        std::fs::read_to_string(&existing_path).expect("read existing"),
        "existing user code"
    );
    let contents = std::fs::read_to_string(&created.path).expect("read scaffold");
    assert!(contents.contains("capture.v1"));
}
