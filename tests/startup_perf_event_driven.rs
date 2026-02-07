use std::fs;

fn read_source(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|err| panic!("failed to read {path}: {err}"))
}

fn read_script_list_startup_source() -> String {
    read_source("src/app_impl/startup.rs")
}

fn read_prompt_ai_source() -> String {
    read_source("src/app_impl/prompt_ai.rs")
}

fn read_main_source() -> String {
    read_source("src/main.rs")
}

fn read_main_startup_bootstrap_source() -> String {
    read_source("src/main_entry/app_run_setup.rs")
}

fn function_body<'a>(source: &'a str, signature: &str) -> &'a str {
    let start = source
        .find(signature)
        .unwrap_or_else(|| panic!("missing function signature: {signature}"));
    let tail = &source[start..];

    let next_fn = tail
        .match_indices("\n    fn ")
        .find_map(|(idx, _)| (idx > 0).then_some(idx));
    let next_pub_fn = tail
        .match_indices("\n    pub fn ")
        .find_map(|(idx, _)| (idx > 0).then_some(idx));

    let end = match (next_fn, next_pub_fn) {
        (Some(a), Some(b)) => a.min(b),
        (Some(a), None) => a,
        (None, Some(b)) => b,
        (None, None) => tail.len(),
    };

    &tail[..end]
}

#[test]
fn test_script_list_app_new_uses_event_driven_receive_when_loading_startup_data() {
    let source = read_script_list_startup_source();
    let body = function_body(&source, "pub(crate) fn new(");

    assert!(
        body.contains("rx.recv().await"),
        "ScriptListApp::new should await channel receive instead of polling"
    );
    assert!(
        !body.contains("try_recv("),
        "ScriptListApp::new should not poll with try_recv()"
    );
    assert!(
        !body.contains("Duration::from_millis(50)"),
        "ScriptListApp::new should not use 50ms polling timers for startup bridges"
    );
}

#[test]
fn test_rebuild_provider_registry_async_uses_event_driven_receive_when_refreshing_registry() {
    let source = read_prompt_ai_source();
    let body = function_body(&source, "pub fn rebuild_provider_registry_async");

    assert!(
        body.contains("rx.recv().await"),
        "rebuild_provider_registry_async should await channel receive"
    );
    assert!(
        !body.contains("try_recv"),
        "rebuild_provider_registry_async should not poll with try_recv()"
    );
    assert!(
        !body.contains("Duration::from_millis(50)"),
        "rebuild_provider_registry_async should not use 50ms polling timers"
    );
}

#[test]
fn test_main_defers_tray_initialization_until_after_window_creation_for_first_render() {
    let main_source = read_main_source();
    assert!(
        main_source.contains("include!(\"main_entry/app_run_setup.rs\");"),
        "main.rs should include main_entry/app_run_setup.rs for startup wiring"
    );

    let source = read_main_startup_bootstrap_source();

    let window_open_log_idx = source
        .find("Window opened, creating ScriptListApp wrapped in Root")
        .expect("missing window-open log in app_run_setup.rs");
    let tray_init_idx = source
        .find("Tray icon initialized successfully (deferred)")
        .expect("missing deferred tray init log in app_run_setup.rs");
    let deferred_timer_idx = source
        .find("Duration::from_millis(1)")
        .expect("missing deferred tray init timer in app_run_setup.rs");

    assert!(
        tray_init_idx > window_open_log_idx,
        "tray init should be deferred until after window creation"
    );
    assert!(
        deferred_timer_idx < tray_init_idx,
        "tray init path should include a deferred timer before initialization"
    );
}

#[test]
fn test_main_tray_handler_uses_event_driven_receive_when_processing_menu_events() {
    let main_source = read_main_source();
    assert!(
        main_source.contains("include!(\"main_entry/app_run_setup.rs\");"),
        "main.rs should include main_entry/app_run_setup.rs for startup wiring"
    );

    let source = read_main_startup_bootstrap_source();

    let tray_block_start = source
        .find("Tray menu event handler started (event-driven)")
        .expect("missing tray event-driven handler block");
    let tray_block = &source[tray_block_start..];

    assert!(
        tray_block.contains("while let Ok(event) = tray_event_rx.recv().await"),
        "tray handler should await async-channel receive"
    );
    assert!(
        !tray_block.contains("menu_event_receiver().try_recv()"),
        "tray handler should not poll menu events with try_recv()"
    );
    assert!(
        !tray_block.contains("Duration::from_millis(250)"),
        "tray handler should not use 250ms polling timers"
    );
}
