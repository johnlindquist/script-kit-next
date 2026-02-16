use std::fs;
use std::path::PathBuf;

fn read_source(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|err| panic!("failed to read {path}: {err}"))
}

fn read_app_action_handler_source() -> String {
    let mut files: Vec<PathBuf> = fs::read_dir("src/app_actions/handle_action")
        .unwrap_or_else(|err| panic!("failed to read src/app_actions/handle_action: {err}"))
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.extension().is_some_and(|ext| ext == "rs"))
        .collect();
    files.sort();

    let mut content = read_source("src/app_actions/handle_action.rs");
    for file in files {
        content.push('\n');
        content.push_str(
            &fs::read_to_string(&file)
                .unwrap_or_else(|err| panic!("failed to read {}: {err}", file.display())),
        );
    }

    content
}

fn slice_from<'a>(source: &'a str, needle: &str) -> &'a str {
    let idx = source
        .find(needle)
        .unwrap_or_else(|| panic!("expected to find '{needle}'"));
    &source[idx..]
}

#[test]
fn test_add_to_ai_flow_defers_ai_window_open_when_file_attach_action_runs() {
    let handler = read_app_action_handler_source();
    let attach_branch = slice_from(&handler, "\"attach_to_ai\" => {");
    let helper_body = slice_from(&handler, "fn open_ai_window_after_main_hide(");

    assert!(
        attach_branch.contains("self.open_ai_window_after_main_hide("),
        "attach_to_ai should call the deferred helper instead of opening AI inline"
    );
    assert!(
        helper_body.contains("cx.spawn(async move |this, cx| {"),
        "deferred helper should spawn async work so action handling does not block"
    );
    assert!(
        helper_body.contains(".timer(std::time::Duration::from_millis(1))"),
        "deferred helper should wait briefly before opening AI to avoid same-tick contention"
    );
    assert!(
        helper_body.contains("ai::open_ai_window(cx)"),
        "deferred helper should still open the AI window on the spawned task"
    );
    assert!(
        helper_body.contains(".detach();"),
        "deferred helper task should be detached so action handling can return immediately"
    );
}

#[test]
fn test_add_to_ai_flow_hides_main_window_before_opening_ai_window_when_file_attach_action_runs() {
    let handler = read_app_action_handler_source();
    let helper_body = slice_from(&handler, "fn open_ai_window_after_main_hide(");

    let hide_idx = helper_body
        .find("self.hide_main_and_reset(cx);")
        .expect("expected helper to hide the main window");
    let spawn_idx = helper_body
        .find("cx.spawn(async move |this, cx| {")
        .expect("expected helper to defer work with cx.spawn");
    let open_idx = helper_body
        .find("ai::open_ai_window(cx)")
        .expect("expected helper to open AI window");

    assert!(
        hide_idx < spawn_idx && spawn_idx < open_idx,
        "helper should hide main first, then spawn deferred work, then open AI"
    );
}

#[test]
fn test_add_to_ai_flow_forwards_file_reference_to_ai_chat_window_when_file_attach_action_runs() {
    let handler = read_app_action_handler_source();
    let attach_branch = slice_from(&handler, "\"attach_to_ai\" => {");
    let ai_window_api = read_source("src/ai/window/window_api.rs");
    let ai_render_root = read_source("src/ai/window/render_root.rs");

    assert!(
        attach_branch.contains("DeferredAiWindowAction::AddAttachment { path: path.clone() }"),
        "attach_to_ai should pass the selected file path into deferred AI file-reference action"
    );
    assert!(
        handler.contains("Self::AddAttachment { path } => ai::add_ai_attachment(cx, &path),"),
        "deferred helper should map file-reference actions to ai::add_ai_attachment"
    );
    assert!(
        ai_window_api.contains("AiCommand::AddAttachment {")
            && ai_window_api.contains("path: path.to_string()"),
        "ai::add_ai_attachment should queue an AddAttachment command carrying the file path"
    );
    assert!(
        ai_render_root.contains("AiCommand::AddAttachment { path } => {")
            && ai_render_root.contains("self.add_attachment(path.clone(), cx);"),
        "AI window should consume AddAttachment commands so the file reference appears in chat UI"
    );
}
