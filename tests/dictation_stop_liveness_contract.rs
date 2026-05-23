const BUILTIN_EXECUTION: &str = include_str!("../src/app_execute/builtin_execution.rs");
const DICTATION_CAPTURE: &str = include_str!("../src/dictation/capture.rs");
const DICTATION_RUNTIME: &str = include_str!("../src/dictation/runtime.rs");
const DICTATION_WINDOW: &str = include_str!("../src/dictation/window.rs");

fn section_between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start_index = source
        .find(start)
        .unwrap_or_else(|| panic!("missing start marker: {start}"));
    let tail = &source[start_index..];
    let end_index = tail
        .find(end)
        .unwrap_or_else(|| panic!("missing end marker: {end}"));
    &tail[..end_index]
}

#[test]
fn dictation_stop_paths_route_through_nonblocking_coordinator() {
    assert!(
        DICTATION_RUNTIME.contains("STOP_IN_FLIGHT")
            && DICTATION_RUNTIME.contains("pub fn is_dictation_busy(")
            && DICTATION_RUNTIME.contains("pub fn begin_stop_capture(")
            && DICTATION_RUNTIME.contains("BeginStopCapture::AlreadyStopping"),
        "runtime must expose a stop-in-flight coordinator before stop collection leaves the UI path"
    );

    let builtin_stop = section_between(
        BUILTIN_EXECUTION,
        "fn request_active_dictation_stop(",
        "fn begin_dictation_transcription(",
    );
    assert!(
        builtin_stop.contains("background_executor()")
            && builtin_stop.contains("collect_with_deadline")
            && builtin_stop.contains("finish_stop_capture"),
        "app stop helper must collect capture on a background executor and finish through the coordinator"
    );

    let submit = section_between(
        BUILTIN_EXECUTION,
        "fn submit_active_dictation_from_overlay(",
        "fn request_active_dictation_stop(",
    );
    assert!(
        !submit.contains("toggle_dictation(") && submit.contains("request_active_dictation_stop("),
        "overlay submit must request async stop instead of blocking toggle"
    );
}

#[test]
fn capture_processor_and_drop_cannot_join_before_event_drain() {
    let drop_impl = section_between(
        DICTATION_CAPTURE,
        "impl Drop for DictationCaptureHandle",
        "#[cfg(not(target_os = \"macos\"))]",
    );
    assert!(
        !drop_impl.contains("processor_thread.join()"),
        "capture drop must not unconditionally join the processor thread before runtime drains events"
    );
    assert!(
        DICTATION_CAPTURE.contains("try_send(DictationCaptureEvent::Bars")
            && DICTATION_CAPTURE.contains("try_send(DictationCaptureEvent::Chunk")
            && DICTATION_CAPTURE.contains("try_send(DictationCaptureEvent::EndOfStream"),
        "processor sends must be nonblocking so a full UI event channel cannot deadlock shutdown"
    );
}

#[test]
fn overlay_global_key_actions_skip_stale_pump_state() {
    assert!(
        DICTATION_WINDOW.contains("enum GlobalKeyProcessResult")
            && DICTATION_WINDOW.contains("match view.process_global_keys_if_requested")
            && DICTATION_WINDOW.contains("Skipped stale overlay pump state after global key action"),
        "global Escape/Enter processing must prevent stale set_state from overwriting consumed actions"
    );

    let open = section_between(
        DICTATION_WINDOW,
        "pub fn open_dictation_overlay(",
        "pub fn update_dictation_overlay(",
    );
    let close = section_between(
        DICTATION_WINDOW,
        "pub fn close_dictation_overlay(",
        "pub fn is_dictation_overlay_open(",
    );
    assert!(open.contains("ENTER_REQUESTED.store(false"));
    assert!(close.contains("ENTER_REQUESTED.store(false"));
}

#[test]
fn dictation_start_preflight_never_requests_permission_from_hotkey_path() {
    let preflight = section_between(
        BUILTIN_EXECUTION,
        "fn open_dictation_setup_if_not_ready(",
        "fn ensure_dictation_delivery_target_available(",
    );
    assert!(preflight.contains("microphone_permission_status()"));
    assert!(preflight.contains("list_input_devices()"));
    assert!(preflight.contains("Ok(Vec::new())"));
    assert!(
        !preflight.contains("request_microphone_permission"),
        "hotkey/start preflight must remain passive and never wait for a TCC permission prompt"
    );
}
