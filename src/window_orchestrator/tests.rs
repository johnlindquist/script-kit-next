use super::*;

#[test]
fn default_state_is_correct() {
    let state = OrchestratorState::default();
    assert_eq!(state.main.visibility, MainVisibility::Visible);
    assert_eq!(state.main.content, MainContentKind::ScriptList);
    assert_eq!(state.main.return_focus, FocusToken::MainFilter);
    assert!(!state.main.actions_open);
    assert!(!state.detached_ai.visible);
    assert_eq!(state.key_surface, Some(SurfaceId::Main));
}

#[test]
fn show_main_reveals_and_focuses() {
    let mut state = OrchestratorState::default();
    state.main.visibility = MainVisibility::Hidden(HiddenReason::Dismissed);
    state.key_surface = None;

    let t = reduce(&state, WindowEvent::ShowMain { activate_app: true });
    assert_eq!(t.next.main.visibility, MainVisibility::Visible);
    assert_eq!(t.next.key_surface, Some(SurfaceId::Main));
    assert!(t.commands.contains(&WindowCommand::RevealMain {
        activate_app: true,
        make_key: true,
    }));
}

#[test]
fn dismiss_main_resets_content() {
    let mut state = OrchestratorState::default();
    state.main.content = MainContentKind::Prompt;
    state.main.return_focus = FocusToken::PromptInput;

    let t = reduce(&state, WindowEvent::DismissMain);
    assert_eq!(
        t.next.main.visibility,
        MainVisibility::Hidden(HiddenReason::Dismissed)
    );
    assert_eq!(t.next.main.content, MainContentKind::ScriptList);
    assert_eq!(t.next.main.return_focus, FocusToken::MainFilter);
    assert!(t.commands.contains(&WindowCommand::SaveMainBounds));
    assert!(t.commands.contains(&WindowCommand::DismissMain));
}

#[test]
fn conceal_main_preserves_content() {
    let mut state = OrchestratorState::default();
    state.main.content = MainContentKind::Prompt;
    state.main.return_focus = FocusToken::PromptInput;

    let t = reduce(&state, WindowEvent::ConcealMain);
    assert_eq!(
        t.next.main.visibility,
        MainVisibility::Hidden(HiddenReason::Concealed)
    );
    // Content and focus preserved.
    assert_eq!(t.next.main.content, MainContentKind::Prompt);
    assert_eq!(t.next.main.return_focus, FocusToken::PromptInput);
    assert!(t.commands.contains(&WindowCommand::SaveMainBounds));
    assert!(t.commands.contains(&WindowCommand::ConcealMain));
}

#[test]
fn dismiss_then_show_round_trip() {
    let mut state = OrchestratorState::default();
    let cmds = state.dispatch(WindowEvent::DismissMain);
    assert!(cmds.contains(&WindowCommand::DismissMain));
    assert_eq!(
        state.main.visibility,
        MainVisibility::Hidden(HiddenReason::Dismissed)
    );

    let cmds = state.dispatch(WindowEvent::ShowMain {
        activate_app: false,
    });
    assert!(cmds.contains(&WindowCommand::RevealMain {
        activate_app: false,
        make_key: true,
    }));
    assert_eq!(state.main.visibility, MainVisibility::Visible);
    assert_eq!(state.key_surface, Some(SurfaceId::Main));
}

#[test]
fn open_close_actions() {
    let mut state = OrchestratorState::default();

    let cmds = state.dispatch(WindowEvent::OpenActions);
    assert!(state.main.actions_open);
    assert!(cmds.contains(&WindowCommand::OpenActions));

    // Idempotent open — no duplicate command.
    let cmds = state.dispatch(WindowEvent::OpenActions);
    assert!(cmds.is_empty());

    let cmds = state.dispatch(WindowEvent::CloseActions);
    assert!(!state.main.actions_open);
    assert!(cmds.contains(&WindowCommand::CloseActions));

    // Idempotent close.
    let cmds = state.dispatch(WindowEvent::CloseActions);
    assert!(cmds.is_empty());
}

#[test]
fn dismiss_main_closes_actions() {
    let mut state = OrchestratorState::default();
    state.main.actions_open = true;

    let cmds = state.dispatch(WindowEvent::DismissMain);
    assert!(!state.main.actions_open);
    assert!(cmds.contains(&WindowCommand::CloseActions));
    assert!(cmds.contains(&WindowCommand::DismissMain));
}

#[test]
fn open_close_detached_ai() {
    let mut state = OrchestratorState::default();

    let cmds = state.dispatch(WindowEvent::OpenDetachedAiChat);
    assert!(state.detached_ai.visible);
    assert_eq!(state.key_surface, Some(SurfaceId::DetachedAiChat));
    assert!(cmds.contains(&WindowCommand::OpenDetachedAiChatWindow));

    let cmds = state.dispatch(WindowEvent::CloseDetachedAiChat);
    assert!(!state.detached_ai.visible);
    assert_eq!(state.key_surface, Some(SurfaceId::Main));
    assert!(cmds.contains(&WindowCommand::CloseDetachedAiChatWindow));
}

#[test]
fn surface_closed_by_system_resyncs_main() {
    let mut state = OrchestratorState::default();
    state.main.content = MainContentKind::AgentChat;
    state.main.return_focus = FocusToken::ChatComposer;
    state.key_surface = Some(SurfaceId::Main);

    let cmds = state.dispatch(WindowEvent::SurfaceClosedBySystem(SurfaceId::Main));

    assert_eq!(
        state.main.visibility,
        MainVisibility::Hidden(HiddenReason::Dismissed)
    );
    assert_eq!(state.main.content, MainContentKind::ScriptList);
    assert_eq!(state.main.return_focus, FocusToken::MainFilter);
    assert_eq!(state.key_surface, None);
    assert!(cmds.is_empty()); // system close doesn't emit commands — it syncs state only
}

#[test]
fn surface_closed_by_system_resyncs_detached_ai() {
    let mut state = OrchestratorState::default();
    state.detached_ai.visible = true;
    state.key_surface = Some(SurfaceId::DetachedAiChat);

    state.dispatch(WindowEvent::SurfaceClosedBySystem(
        SurfaceId::DetachedAiChat,
    ));

    assert!(!state.detached_ai.visible);
    assert_eq!(state.key_surface, Some(SurfaceId::Main));
}

#[test]
fn main_content_changed_updates_content_and_focus() {
    let mut state = OrchestratorState::default();

    state.dispatch(WindowEvent::MainContentChanged {
        content: MainContentKind::AgentChat,
        focus: FocusToken::ChatComposer,
    });

    assert_eq!(state.main.content, MainContentKind::AgentChat);
    assert_eq!(state.main.return_focus, FocusToken::ChatComposer);
}

#[test]
fn current_return_focus_reflects_key_surface() {
    let mut state = OrchestratorState::default();
    state.main.return_focus = FocusToken::PromptInput;
    assert_eq!(current_return_focus(&state), FocusToken::PromptInput);

    state.key_surface = Some(SurfaceId::DetachedAiChat);
    assert_eq!(current_return_focus(&state), FocusToken::DetachedAiComposer);

    state.key_surface = None;
    assert_eq!(current_return_focus(&state), FocusToken::None);
}

#[test]
fn conceal_main_closes_open_actions() {
    let mut state = OrchestratorState::default();
    state.main.actions_open = true;

    let cmds = state.dispatch(WindowEvent::ConcealMain);

    assert!(!state.main.actions_open);
    assert!(cmds.contains(&WindowCommand::CloseActions));
}

#[test]
fn dispatch_convenience_mutates_in_place() {
    let mut state = OrchestratorState::default();
    let cmds = state.dispatch(WindowEvent::DismissMain);

    assert!(!cmds.is_empty());
    assert_eq!(
        state.main.visibility,
        MainVisibility::Hidden(HiddenReason::Dismissed)
    );
}
