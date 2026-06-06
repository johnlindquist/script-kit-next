const ACP_LAUNCH_SOURCE: &str = include_str!("../src/app_impl/tab_ai_mode/acp_launch.rs");
const ACP_THREAD_SOURCE: &str = include_str!("../src/ai/acp/thread.rs");
const STDIN_SOURCE: &str = include_str!("../src/stdin_commands/mod.rs");
const RUNTIME_STDIN_SOURCE: &str = include_str!("../src/main_entry/runtime_stdin.rs");
const RUNTIME_TAIL_SOURCE: &str = include_str!("../src/main_entry/runtime_stdin_match_tail.rs");
const APP_RUN_SETUP_SOURCE: &str = include_str!("../src/main_entry/app_run_setup.rs");
const ACP_DEVTOOLS_SOURCE: &str = include_str!("../scripts/devtools/acp.ts");

fn source_between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start_index = source
        .find(start)
        .unwrap_or_else(|| panic!("missing start marker: {start}"));
    let after_start = &source[start_index..];
    let end_index = after_start
        .find(end)
        .unwrap_or_else(|| panic!("missing end marker after {start}: {end}"));
    &after_start[..end_index]
}

#[test]
fn kitchen_sink_thread_loader_installs_all_fixture_messages() {
    let loader = source_between(
        ACP_THREAD_SOURCE,
        "pub(crate) fn load_kitchen_sink_fixture",
        "/// Clear composer-attached context state",
    );

    assert!(loader.contains("agent_chat_kitchen_sink_fixture()"));
    assert!(loader.contains("self.messages.clear();"));
    assert!(loader.contains("self.clear_all_pending_context(\"load_kitchen_sink_fixture\")"));
    assert!(loader.contains("for message in fixture.messages"));
    assert!(loader.contains("AcpKitchenSinkFixtureRole::User => AcpThreadMessageRole::User"));
    assert!(
        loader.contains("AcpKitchenSinkFixtureRole::Assistant => AcpThreadMessageRole::Assistant")
    );
    assert!(loader.contains("AcpKitchenSinkFixtureRole::Thought => AcpThreadMessageRole::Thought"));
    assert!(loader.contains("AcpKitchenSinkFixtureRole::Tool => AcpThreadMessageRole::Tool"));
    assert!(loader.contains("AcpKitchenSinkFixtureRole::System => AcpThreadMessageRole::System"));
    assert!(loader.contains("AcpKitchenSinkFixtureRole::Error => AcpThreadMessageRole::Error"));
    assert!(loader.contains("AcpThreadMessage::with_tool_call_id"));
    assert!(loader.contains("self.set_status(AcpThreadStatus::Idle);"));
    assert!(loader.contains("event = \"acp_kitchen_sink_fixture_loaded\""));
}

#[test]
fn kitchen_sink_launcher_uses_standard_embedded_agent_chat_without_provider_warmup() {
    let launcher = source_between(
        ACP_LAUNCH_SOURCE,
        "pub(crate) fn open_agent_chat_kitchen_sink_fixture",
        "/// **Contract:** `AppView::AcpChatView`",
    );

    assert!(launcher.contains("agent_chat_kitchen_sink_fixture()"));
    assert!(launcher.contains("StandardAgentChatMockFixtureConnection"));
    assert!(launcher.contains("thread.load_kitchen_sink_fixture(cx);"));
    assert!(launcher.contains("AcpChatUiVariant::Standard"));
    assert!(launcher.contains("self.enter_embedded_acp_chat_surface(view_entity, cx);"));
    assert!(launcher.contains("script_kit_gpui::set_main_window_visible(true);"));
    assert!(launcher.contains("script_kit_gpui::mark_window_shown();"));
    assert!(!launcher.contains("request_show_main_window"));
    assert!(!launcher.contains("prepare_session("));
    assert!(!launcher.contains("open_tab_ai_acp_with_entry_intent"));
}

#[test]
fn kitchen_sink_stdin_command_is_registered_and_dispatched() {
    assert!(STDIN_SOURCE.contains("OpenAgentChatKitchenSinkFixture"));
    assert!(STDIN_SOURCE.contains("\"openAgentChatKitchenSinkFixture\""));
    assert!(STDIN_SOURCE
        .contains("test_external_command_open_agent_chat_kitchen_sink_fixture_deserialization"));

    for (path, source) in [
        ("src/main_entry/runtime_stdin.rs", RUNTIME_STDIN_SOURCE),
        (
            "src/main_entry/runtime_stdin_match_tail.rs",
            RUNTIME_TAIL_SOURCE,
        ),
        ("src/main_entry/app_run_setup.rs", APP_RUN_SETUP_SOURCE),
    ] {
        assert!(
            source.contains("ExternalCommand::OpenAgentChatKitchenSinkFixture"),
            "{path} must dispatch the kitchen sink fixture command"
        );
        assert!(
            source.contains("view.open_agent_chat_kitchen_sink_fixture(ctx);"),
            "{path} must call the provider-free kitchen sink launcher"
        );
        assert!(
            source.contains("agent_chat_kitchen_sink_fixture_opened"),
            "{path} must emit a traceable fixture-open receipt"
        );
        assert!(
            source.contains("Message::external_command_result("),
            "{path} must acknowledge the fixture command over the protocol response bus"
        );
    }
}

#[test]
fn acp_devtools_can_open_kitchen_sink_fixture() {
    assert!(ACP_DEVTOOLS_SOURCE.contains("\"open-kitchen-sink\""));
    assert!(ACP_DEVTOOLS_SOURCE.contains("openAgentChatKitchenSinkFixture"));
    assert!(ACP_DEVTOOLS_SOURCE.contains("\"rpc\""));
    assert!(ACP_DEVTOOLS_SOURCE.contains("acp.openKitchenSink"));
    assert!(ACP_DEVTOOLS_SOURCE.contains("providerRequired: false"));
    assert!(ACP_DEVTOOLS_SOURCE.contains("fixtureOnly: true"));
}
