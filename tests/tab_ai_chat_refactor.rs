//! Source-contract tests locking the Tab AI overlay-to-chat cutover.
//!
//! These tests ensure that:
//! 1. No production source references overlay-era symbols
//! 2. Tab routing targets `open_tab_ai_chat(cx)`
//! 3. The storybook has been renamed from overlay to chat
//! 4. The new TabAiChat entity owns proper ChatPrompt-style state

use std::fs;
use std::path::Path;

/// Collect all `.rs` files under `src/` (production code only).
fn collect_production_sources() -> String {
    let mut combined = String::new();
    collect_rs_files(Path::new("src"), &mut combined);
    combined
}

fn collect_rs_files(dir: &Path, buf: &mut String) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, buf);
        } else if path.extension().is_some_and(|e| e == "rs") {
            if let Ok(content) = fs::read_to_string(&path) {
                buf.push_str(&content);
                buf.push('\n');
            }
        }
    }
}

const BANNED_SYMBOLS: &[&str] = &[
    "TabAiOverlayState",
    "tab_ai_state",
    "render_tab_ai_overlay",
    "handle_tab_ai_key_down",
    "submit_tab_ai_overlay",
    "close_tab_ai_overlay",
    "open_tab_ai_overlay",
];

// ---------------------------------------------------------------------------
// No overlay-era symbols in production source
// ---------------------------------------------------------------------------

#[test]
fn no_overlay_symbols_in_production_source() {
    let src = collect_production_sources();
    for symbol in BANNED_SYMBOLS {
        assert!(
            !src.contains(symbol),
            "production source must not reference overlay-era symbol: {symbol}"
        );
    }
}

// ---------------------------------------------------------------------------
// Tab routing targets open_tab_ai_chat
// ---------------------------------------------------------------------------

#[test]
fn startup_routes_tab_into_full_view_chat() {
    let src = include_str!("../src/app_impl/startup_new_tab.rs");
    assert!(
        src.contains("this.open_tab_ai_chat(cx);"),
        "startup_new_tab.rs must call open_tab_ai_chat"
    );
    assert!(
        !src.contains("open_tab_ai_overlay"),
        "startup_new_tab.rs must not reference the removed overlay opener"
    );
}

// ---------------------------------------------------------------------------
// Storybook renamed from overlay to chat
// ---------------------------------------------------------------------------

#[test]
fn storybook_uses_chat_naming_not_overlay() {
    let stories_mod = include_str!("../src/stories/mod.rs");
    assert!(
        !stories_mod.contains("tab_ai_overlay_stories"),
        "stories/mod.rs must not reference the old overlay story module"
    );
    assert!(
        !stories_mod.contains("TabAiOverlayStory"),
        "stories/mod.rs must not reference the old TabAiOverlayStory type"
    );
    assert!(
        stories_mod.contains("tab_ai_chat_stories"),
        "stories/mod.rs must reference the new tab_ai_chat_stories module"
    );
    assert!(
        stories_mod.contains("TabAiChatStory"),
        "stories/mod.rs must reference the new TabAiChatStory type"
    );
}

#[test]
fn old_overlay_story_file_does_not_exist() {
    assert!(
        !Path::new("src/stories/tab_ai_overlay_stories.rs").exists(),
        "the old tab_ai_overlay_stories.rs file must be deleted"
    );
}

// ---------------------------------------------------------------------------
// TabAiChat entity has ChatPrompt-style state
// ---------------------------------------------------------------------------

#[test]
fn app_view_state_declares_tab_ai_chat_with_entity_state() {
    let src = include_str!("../src/main_sections/app_view_state.rs");
    assert!(
        src.contains("struct TabAiChat"),
        "app_view_state.rs must declare a TabAiChat struct"
    );
    assert!(
        src.contains("input: TextInputState"),
        "TabAiChat must own input via TextInputState"
    );
    assert!(
        src.contains("turns_list_state: ListState"),
        "TabAiChat must own a ListState for scrollable turns"
    );
    assert!(
        src.contains("focus_handle: FocusHandle"),
        "TabAiChat must own a FocusHandle"
    );
}

#[test]
fn tab_ai_chat_is_non_dismissable() {
    let src = include_str!("../src/app_impl/shortcuts_hud_grid.rs");
    assert!(
        src.contains("TabAiChat"),
        "is_dismissable_view() must include TabAiChat to prevent blur-close"
    );
}

#[test]
fn render_impl_dispatches_tab_ai_chat() {
    let src = include_str!("../src/main_sections/render_impl.rs");
    assert!(
        src.contains("AppView::TabAiChat"),
        "render_impl.rs must dispatch AppView::TabAiChat"
    );
}

// ---------------------------------------------------------------------------
// Origin-view context preservation (Task 1)
// ---------------------------------------------------------------------------

#[test]
fn submission_payload_returns_origin_view() {
    let src = include_str!("../src/main_sections/app_view_state.rs");
    // submission_payload must return AppView as part of its tuple
    assert!(
        src.contains("fn submission_payload("),
        "TabAiChat must define submission_payload"
    );
    assert!(
        src.contains("AppView,"),
        "submission_payload return type must include AppView (the origin view)"
    );
    assert!(
        src.contains("self.return_view.clone()"),
        "submission_payload must return the stored return_view, not current_view"
    );
}

#[test]
fn build_context_uses_source_view_for_targets() {
    let src = include_str!("../src/app_impl/tab_ai_mode.rs");
    assert!(
        src.contains("resolve_tab_ai_surface_targets_for_view("),
        "build_tab_ai_context_from must use the _for_view variant"
    );
    assert!(
        src.contains("resolve_tab_ai_clipboard_context_for_view("),
        "build_tab_ai_context_from must use the _for_view variant for clipboard"
    );
}

#[test]
fn submit_tab_ai_chat_destructures_source_view() {
    let src = include_str!("../src/app_impl/tab_ai_mode.rs");
    assert!(
        src.contains("let (intent, source_view, ui_snapshot, invocation_receipt)"),
        "submit_tab_ai_chat must destructure source_view from submission_payload"
    );
}

#[test]
fn surface_targets_for_view_accepts_explicit_view() {
    let src = include_str!("../src/app_impl/tab_ai_mode.rs");
    assert!(
        src.contains("fn resolve_tab_ai_surface_targets_for_view("),
        "must have a _for_view variant for surface target resolution"
    );
    assert!(
        src.contains("view: &AppView,"),
        "the _for_view variant must accept an explicit AppView parameter"
    );
}

#[test]
fn clipboard_context_for_view_accepts_explicit_view() {
    let src = include_str!("../src/app_impl/tab_ai_mode.rs");
    assert!(
        src.contains("fn resolve_tab_ai_clipboard_context_for_view("),
        "must have a _for_view variant for clipboard context resolution"
    );
}

// ---------------------------------------------------------------------------
// Origin-view clipboard: selected_index flows through source_view
// ---------------------------------------------------------------------------

#[test]
fn tab_ai_chat_submit_uses_origin_view_for_clipboard_context() {
    // The submit path must destructure `source_view` from `submission_payload()`
    // and pass it to `resolve_tab_ai_clipboard_context_for_view(&source_view)`.
    // This ensures that after the view has switched to `TabAiChat`, the clipboard
    // selected_index still comes from the *origin* surface, not current_view.
    let src = include_str!("../src/app_impl/tab_ai_mode.rs");

    // 1. submit_tab_ai_chat extracts source_view from submission_payload
    assert!(
        src.contains("let (intent, source_view, ui_snapshot, invocation_receipt)"),
        "submit must destructure source_view from submission_payload"
    );

    // 2. build_tab_ai_context_from receives source_view and forwards it
    assert!(
        src.contains("source_view: AppView,"),
        "build_tab_ai_context_from must accept an explicit source_view param"
    );

    // 3. clipboard resolution uses the source_view, not self.current_view
    assert!(
        src.contains("resolve_tab_ai_clipboard_context_for_view(&source_view)"),
        "clipboard context must resolve against source_view, not current_view"
    );

    // 4. The _for_view variant extracts selected_index from ClipboardHistoryView
    assert!(
        src.contains("AppView::ClipboardHistoryView { selected_index, .. }"),
        "clipboard_for_view must pattern-match ClipboardHistoryView to extract selected_index"
    );
}

// ---------------------------------------------------------------------------
// Origin-view file targets: FileSearchView metadata preserved
// ---------------------------------------------------------------------------

#[test]
fn tab_ai_chat_submit_preserves_file_target_metadata_from_file_search() {
    // When the user presses Tab from FileSearchView, the submit path must
    // resolve targets against the FileSearchView origin — producing a
    // focusedTarget with source="FileSearch", kind="file"/"directory",
    // and metadata containing "path" and "fileType".
    let src = include_str!("../src/app_impl/tab_ai_mode.rs");

    // 1. surface targets are resolved against source_view, not current_view
    assert!(
        src.contains("resolve_tab_ai_surface_targets_for_view(&source_view, &ui"),
        "surface targets at submit time must use source_view"
    );

    // 2. The _for_view variant has a FileSearchView branch
    assert!(
        src.contains("AppView::FileSearchView { selected_index, .. } =>"),
        "resolve_tab_ai_surface_targets_for_view must match FileSearchView"
    );

    // 3. FileSearch branch emits path and fileType metadata
    let file_search_section = src
        .find("AppView::FileSearchView { selected_index, .. } =>")
        .expect("must find FileSearchView branch in _for_view");
    let section = &src[file_search_section..file_search_section + 1200];
    assert!(
        section.contains("entry.path"),
        "FileSearchView target metadata must include file path"
    );
    assert!(
        section.contains("entry.file_type"),
        "FileSearchView target metadata must include file type"
    );
    assert!(
        section.contains("source: \"FileSearch\""),
        "FileSearchView targets must have source='FileSearch'"
    );
}

// ---------------------------------------------------------------------------
// ⌘K propagation (Task 2)
// ---------------------------------------------------------------------------

#[test]
fn cmd_k_propagates_in_tab_ai_chat() {
    let src = include_str!("../src/app_impl/tab_ai_mode.rs");
    assert!(
        src.contains("modifiers.platform && key.eq_ignore_ascii_case(\"k\")"),
        "handle_tab_ai_chat_key_down must check for ⌘K"
    );
    // The ⌘K handler must call propagate, not stop_propagation
    let cmd_k_section = src
        .find("eq_ignore_ascii_case(\"k\")")
        .expect("must find ⌘K check");
    let after_cmd_k = &src[cmd_k_section..cmd_k_section + 200];
    assert!(
        after_cmd_k.contains("cx.propagate()"),
        "⌘K handler must call cx.propagate() to let Actions dialog open"
    );
}

#[test]
fn cmd_k_is_not_routed_to_text_input() {
    // ⌘K must return *before* reaching the TextInputState::handle_key call.
    // This ensures the Actions dialog shortcut is never consumed by input.
    let src = include_str!("../src/app_impl/tab_ai_mode.rs");

    // Find positions of both the ⌘K early-return and the input handler
    let cmd_k_pos = src
        .find("eq_ignore_ascii_case(\"k\")")
        .expect("must find ⌘K check");
    let input_handle_pos = src[cmd_k_pos..]
        .find("chat.input.handle_key(")
        .expect("must find TextInputState::handle_key after ⌘K check");

    // Between ⌘K check and handle_key, there must be a `return` — proving
    // ⌘K exits the handler before reaching TextInputState
    let between = &src[cmd_k_pos..cmd_k_pos + input_handle_pos];
    assert!(
        between.contains("return;"),
        "⌘K handler must return before reaching TextInputState::handle_key"
    );
}

#[test]
fn unhandled_keys_propagate_not_swallow() {
    let src = include_str!("../src/app_impl/tab_ai_mode.rs");
    // After the input handler, unhandled keys must propagate
    assert!(
        src.contains("} else {\n            cx.propagate();\n        }"),
        "unhandled keys must call cx.propagate() instead of unconditional stop_propagation"
    );
}

// ---------------------------------------------------------------------------
// Conversational fallback (Task 4) — dual-mode script vs text
// ---------------------------------------------------------------------------

#[test]
fn tab_ai_worker_result_enum_exists() {
    let src = include_str!("../src/app_impl/tab_ai_mode.rs");
    assert!(
        src.contains("enum TabAiWorkerResult"),
        "tab_ai_mode.rs must declare TabAiWorkerResult enum for dual-mode responses"
    );
}

#[test]
fn tab_ai_worker_result_has_script_and_text_variants() {
    let src = include_str!("../src/app_impl/tab_ai_mode.rs");
    assert!(
        src.contains("Script { slug: String, source: String }"),
        "TabAiWorkerResult must have a Script variant with slug and source"
    );
    assert!(
        src.contains("Text(String)"),
        "TabAiWorkerResult must have a Text variant for conversational responses"
    );
    assert!(
        src.contains("Error(String)"),
        "TabAiWorkerResult must have an Error variant for hard failures"
    );
}

#[test]
fn worker_thread_returns_text_on_script_parse_failure() {
    let src = include_str!("../src/app_impl/tab_ai_mode.rs");
    // When prepare_script_from_ai_response fails, the worker must return
    // TabAiWorkerResult::Text(raw_response) instead of an error
    assert!(
        src.contains("TabAiWorkerResult::Text(raw_response)"),
        "worker must return Text(raw_response) when script parsing fails"
    );
    // The old error message about "no runnable script" must be gone
    assert!(
        !src.contains("AI returned no runnable script"),
        "the old 'no runnable script' error string must be removed — text responses are valid"
    );
}

#[test]
fn text_response_handler_appends_assistant_text_turn() {
    let src = include_str!("../src/app_impl/tab_ai_mode.rs");
    // The Text arm must append an assistant text turn and clear running state
    assert!(
        src.contains("TabAiWorkerResult::Text(text)"),
        "response handler must match TabAiWorkerResult::Text"
    );
    assert!(
        src.contains("chat.append_assistant_text_turn(text)"),
        "text response handler must append an assistant text turn"
    );
}

#[test]
fn text_response_does_not_execute_script() {
    let src = include_str!("../src/app_impl/tab_ai_mode.rs");
    // Find the Text arm and verify it does NOT call execute_script_by_path
    let text_arm_pos = src
        .find("TabAiWorkerResult::Text(text)")
        .expect("must find Text arm in response handler");
    // Find the next match arm (Error) to bound our search
    let error_arm_pos = src[text_arm_pos..]
        .find("TabAiWorkerResult::Error")
        .expect("must find Error arm after Text arm");
    let text_arm_body = &src[text_arm_pos..text_arm_pos + error_arm_pos];
    assert!(
        !text_arm_body.contains("execute_script_by_path"),
        "Text response handler must NOT call execute_script_by_path"
    );
    assert!(
        !text_arm_body.contains("create_interactive_temp_script"),
        "Text response handler must NOT create temp scripts"
    );
}

#[test]
fn text_response_clears_running_state() {
    let src = include_str!("../src/app_impl/tab_ai_mode.rs");
    let text_arm_pos = src
        .find("TabAiWorkerResult::Text(text)")
        .expect("must find Text arm");
    let error_arm_pos = src[text_arm_pos..]
        .find("TabAiWorkerResult::Error")
        .expect("must find Error arm after Text arm");
    let text_arm_body = &src[text_arm_pos..text_arm_pos + error_arm_pos];
    assert!(
        text_arm_body.contains("set_running(false)"),
        "Text response handler must clear running state so user can send again"
    );
    assert!(
        text_arm_body.contains("cx.notify()"),
        "Text response handler must notify after state change"
    );
}

#[test]
fn channel_uses_worker_result_not_plain_result() {
    let src = include_str!("../src/app_impl/tab_ai_mode.rs");
    assert!(
        src.contains("async_channel::bounded::<TabAiWorkerResult>"),
        "channel must carry TabAiWorkerResult, not Result<(String, String), String>"
    );
    assert!(
        !src.contains("async_channel::bounded::<Result<(String, String), String>>"),
        "old Result<(String, String), String> channel type must be removed"
    );
}

#[test]
fn empty_response_is_hard_error_not_text() {
    let src = include_str!("../src/app_impl/tab_ai_mode.rs");
    // Empty/whitespace-only AI responses should be a hard error, not a text turn
    assert!(
        src.contains("raw_response.trim().is_empty()"),
        "worker must check for empty responses before attempting script parse"
    );
}

// ---------------------------------------------------------------------------
// Full-view footer contract (regression lock)
// ---------------------------------------------------------------------------

#[test]
fn tab_ai_chat_uses_full_view_footer_contract() {
    const TAB_AI_SOURCE: &str = include_str!("../src/app_impl/tab_ai_mode.rs");
    assert!(
        TAB_AI_SOURCE.contains(r#""\u{21B5} Send"#),
        "tab ai chat footer must show the Send hint"
    );
    assert!(
        TAB_AI_SOURCE.contains(r#""\u{2318}K Actions"#),
        "tab ai chat footer must show the Actions hint"
    );
    assert!(
        TAB_AI_SOURCE.contains(r#""Esc Back"#),
        "tab ai chat footer must show the Esc Back hint"
    );
}

// ---------------------------------------------------------------------------
// Streaming helper path (regression lock)
// ---------------------------------------------------------------------------

#[test]
fn tab_ai_chat_uses_streaming_and_markdown_paths() {
    const TAB_AI_SOURCE: &str = include_str!("../src/app_impl/tab_ai_mode.rs");
    assert!(
        TAB_AI_SOURCE.contains("start_assistant_turn("),
        "tab ai must use start_assistant_turn for streaming"
    );
    assert!(
        TAB_AI_SOURCE.contains("append_turn_chunk("),
        "tab ai must use append_turn_chunk for progressive reveal"
    );
    assert!(
        TAB_AI_SOURCE.contains("complete_turn_stream("),
        "tab ai must use complete_turn_stream to finalize"
    );
    assert!(
        TAB_AI_SOURCE.contains("render_markdown("),
        "tab ai must use render_markdown for assistant output"
    );
    assert!(
        TAB_AI_SOURCE.contains(r#""Thinking\u{2026}""#)
            || TAB_AI_SOURCE.contains(r#""Thinking...""#),
        "tab ai must show a thinking indicator while streaming"
    );
}

// ---------------------------------------------------------------------------
// Overlay symbols are gone (regression lock)
// ---------------------------------------------------------------------------

#[test]
fn tab_ai_overlay_symbols_are_gone() {
    const TAB_AI_SOURCE: &str = include_str!("../src/app_impl/tab_ai_mode.rs");
    assert!(
        !TAB_AI_SOURCE.contains("render_tab_ai_overlay"),
        "render_tab_ai_overlay must not exist in tab_ai_mode.rs"
    );
    assert!(
        !TAB_AI_SOURCE.contains("submit_tab_ai_overlay"),
        "submit_tab_ai_overlay must not exist in tab_ai_mode.rs"
    );
}
