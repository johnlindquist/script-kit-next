const WARM_SESSION_SOURCE: &str = include_str!("../src/ai/agent_chat/warm_session.rs");

#[test]
fn warm_session_lifecycle_symbols_are_explicit() {
    for symbol in [
        "AgentChatWarmSessionManager",
        "prepare_warm",
        "acquire_warm",
        "dismiss_reset",
        "AgentChatWarmSessionLease",
        "AgentChatWarmSessionSpec",
        "AgentChatWarmRuntimeFactory",
    ] {
        assert!(WARM_SESSION_SOURCE.contains(symbol), "missing {}", symbol);
    }
}

#[test]
fn dismiss_replacement_semantics_are_source_guarded() {
    for symbol in [
        "cancel_turn",
        "generation",
        "ui_thread_id_source",
        "prepare_session",
    ] {
        assert!(WARM_SESSION_SOURCE.contains(symbol), "missing {}", symbol);
    }
}

#[test]
fn warm_ready_requires_bounded_prepare_readiness_event() {
    for symbol in [
        "DEFAULT_PREPARE_READY_TIMEOUT",
        "wait_for_prepare_ready",
        "AgentChatEvent::ModelsAvailable",
        "AgentChatEvent::SetupRequired",
        "AgentChatEvent::Failed",
        "agent_chat_warm_prepare_ready_timeout",
    ] {
        assert!(WARM_SESSION_SOURCE.contains(symbol), "missing {}", symbol);
    }

    let prepare_slot = WARM_SESSION_SOURCE
        .split("fn prepare_new_slot")
        .nth(1)
        .expect("prepare_new_slot must exist");
    assert!(prepare_slot.contains("connection.prepare_session"));
    assert!(prepare_slot.contains("wait_for_prepare_ready"));
    assert!(
        !prepare_slot.contains(".prepare_session(ui_thread_id.clone(), spec.cwd.clone())\n                    .is_ok()"),
        "warm sessions must not become Ready merely because prepare_session enqueued"
    );
}

#[test]
fn warm_prepare_reserves_preparing_slot_before_spawn() {
    let prepare_warm = WARM_SESSION_SOURCE
        .split("pub(crate) fn prepare_warm")
        .nth(1)
        .expect("prepare_warm must exist");
    let prepare_warm = prepare_warm
        .split("pub(crate) fn acquire_warm")
        .next()
        .expect("prepare_warm body must precede acquire_warm");

    assert!(prepare_warm.contains("inner.slots.insert"));
    assert!(prepare_warm.contains("state: AgentChatWarmSessionState::Preparing"));
    assert!(prepare_warm.contains("prepare_slot_with_generation"));
    assert!(prepare_warm.contains("current.generation != generation"));
    assert!(
        prepare_warm.find("state: AgentChatWarmSessionState::Preparing")
            < prepare_warm.find("prepare_slot_with_generation"),
        "prepare_warm must reserve a Preparing slot before spawning/preparing the runtime"
    );
}
