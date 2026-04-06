use super::*;

#[test]
fn default_state_is_correct() {
    let state = OrchestratorState::default();
    assert_eq!(state.main.visibility, MainVisibility::Visible);
    assert_eq!(state.main.content, MainContentKind::ScriptList);
    assert_eq!(state.main.return_focus, FocusToken::MainFilter);
    assert!(!state.main.actions_open);
    assert!(!state.notes.visible);
    assert!(!state.detached_ai.visible);
    assert_eq!(state.dictation, DictationSurfaceState::Hidden);
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
fn open_close_notes_focus_transitions() {
    let mut state = OrchestratorState::default();

    let cmds = state.dispatch(WindowEvent::OpenNotes);
    assert!(state.notes.visible);
    assert_eq!(state.key_surface, Some(SurfaceId::Notes));
    assert!(cmds.contains(&WindowCommand::OpenNotesWindow));
    assert!(cmds.contains(&WindowCommand::FocusSurface(SurfaceId::Notes)));

    // Close notes — focus should return to main (visible).
    let cmds = state.dispatch(WindowEvent::CloseNotes);
    assert!(!state.notes.visible);
    assert_eq!(state.key_surface, Some(SurfaceId::Main));
    assert!(cmds.contains(&WindowCommand::CloseNotesWindow));
    assert!(cmds.contains(&WindowCommand::FocusSurface(SurfaceId::Main)));
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
fn start_dictation_from_main_conceals_and_opens_overlay() {
    let state = OrchestratorState::default();

    let t = reduce(
        &state,
        WindowEvent::StartDictation {
            target: DictationTarget::MainWindowPrompt,
        },
    );

    // Main concealed.
    assert_eq!(
        t.next.main.visibility,
        MainVisibility::Hidden(HiddenReason::Concealed)
    );
    // Dictation visible and recording.
    assert!(matches!(
        t.next.dictation,
        DictationSurfaceState::Visible {
            phase: DictationPhase::Recording,
            target: DictationTarget::MainWindowPrompt,
            restore_main_visibility: true,
            ..
        }
    ));
    assert_eq!(t.next.key_surface, Some(SurfaceId::DictationOverlay));

    // Commands.
    assert!(t.commands.contains(&WindowCommand::SaveMainBounds));
    assert!(t.commands.contains(&WindowCommand::ConcealMain));
    assert!(t.commands.contains(&WindowCommand::OpenDictationOverlay {
        phase: DictationPhase::Recording,
    }));
}

#[test]
fn finish_dictation_reveals_main_and_restores_focus() {
    let mut state = OrchestratorState::default();
    state.main.return_focus = FocusToken::PromptInput;
    state.dispatch(WindowEvent::StartDictation {
        target: DictationTarget::MainWindowPrompt,
    });

    let cmds = state.dispatch(WindowEvent::FinishDictation);

    assert_eq!(state.dictation, DictationSurfaceState::Hidden);
    assert_eq!(state.main.visibility, MainVisibility::Visible);
    assert_eq!(state.key_surface, Some(SurfaceId::Main));
    assert!(cmds.contains(&WindowCommand::CloseDictationOverlay));
    assert!(cmds.contains(&WindowCommand::RevealMain {
        activate_app: false,
        make_key: true,
    }));
    assert!(cmds.contains(&WindowCommand::FocusMain(FocusToken::PromptInput)));
}

#[test]
fn abort_dictation_same_as_finish() {
    let mut state = OrchestratorState::default();
    state.dispatch(WindowEvent::StartDictation {
        target: DictationTarget::MainWindowPrompt,
    });

    let cmds = state.dispatch(WindowEvent::AbortDictation);

    assert_eq!(state.dictation, DictationSurfaceState::Hidden);
    assert_eq!(state.main.visibility, MainVisibility::Visible);
    assert!(cmds.contains(&WindowCommand::CloseDictationOverlay));
    assert!(cmds.contains(&WindowCommand::RevealMain {
        activate_app: false,
        make_key: true,
    }));
}

#[test]
fn start_dictation_while_main_already_hidden() {
    let mut state = OrchestratorState::default();
    state.main.visibility = MainVisibility::Hidden(HiddenReason::Dismissed);
    state.notes.visible = true;
    state.key_surface = Some(SurfaceId::Notes);

    let cmds = state.dispatch(WindowEvent::StartDictation {
        target: DictationTarget::NotesEditor,
    });

    // Main was already hidden — should NOT conceal again.
    assert!(!cmds.contains(&WindowCommand::ConcealMain));
    assert!(!cmds.contains(&WindowCommand::SaveMainBounds));

    // Dictation remembers not to restore main.
    assert!(matches!(
        state.dictation,
        DictationSurfaceState::Visible {
            restore_main_visibility: false,
            return_to: FocusToken::NotesEditor,
            ..
        }
    ));
}

#[test]
fn finish_dictation_returns_to_notes() {
    let mut state = OrchestratorState::default();
    state.main.visibility = MainVisibility::Hidden(HiddenReason::Dismissed);
    state.notes.visible = true;
    state.key_surface = Some(SurfaceId::Notes);

    state.dispatch(WindowEvent::StartDictation {
        target: DictationTarget::NotesEditor,
    });
    let cmds = state.dispatch(WindowEvent::FinishDictation);

    assert_eq!(state.key_surface, Some(SurfaceId::Notes));
    assert!(cmds.contains(&WindowCommand::FocusSurface(SurfaceId::Notes)));
    // Main should NOT be revealed.
    assert!(!cmds
        .iter()
        .any(|c| matches!(c, WindowCommand::RevealMain { .. })));
}

#[test]
fn double_start_dictation_is_idempotent() {
    let mut state = OrchestratorState::default();
    state.dispatch(WindowEvent::StartDictation {
        target: DictationTarget::MainWindowPrompt,
    });

    let cmds = state.dispatch(WindowEvent::StartDictation {
        target: DictationTarget::MainWindowPrompt,
    });

    // No new commands.
    assert!(cmds.is_empty());
}

#[test]
fn dictation_phase_changed_updates_state() {
    let mut state = OrchestratorState::default();
    state.dispatch(WindowEvent::StartDictation {
        target: DictationTarget::MainWindowPrompt,
    });

    let cmds = state.dispatch(WindowEvent::DictationPhaseChanged(
        DictationPhase::Transcribing,
    ));

    assert!(matches!(
        state.dictation,
        DictationSurfaceState::Visible {
            phase: DictationPhase::Transcribing,
            ..
        }
    ));
    assert!(cmds.contains(&WindowCommand::UpdateDictationOverlay {
        phase: DictationPhase::Transcribing,
    }));
}

#[test]
fn surface_closed_by_system_resyncs_main() {
    let mut state = OrchestratorState::default();

    let cmds = state.dispatch(WindowEvent::SurfaceClosedBySystem(SurfaceId::Main));

    assert_eq!(
        state.main.visibility,
        MainVisibility::Hidden(HiddenReason::Dismissed)
    );
    assert_eq!(state.main.content, MainContentKind::ScriptList);
    assert_eq!(state.key_surface, None);
    assert!(cmds.is_empty()); // system close doesn't emit commands — it syncs state only
}

#[test]
fn surface_closed_by_system_resyncs_notes() {
    let mut state = OrchestratorState::default();
    state.notes.visible = true;
    state.key_surface = Some(SurfaceId::Notes);

    state.dispatch(WindowEvent::SurfaceClosedBySystem(SurfaceId::Notes));

    assert!(!state.notes.visible);
    assert_eq!(state.key_surface, Some(SurfaceId::Main));
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
        content: MainContentKind::AcpChat,
        focus: FocusToken::ChatComposer,
    });

    assert_eq!(state.main.content, MainContentKind::AcpChat);
    assert_eq!(state.main.return_focus, FocusToken::ChatComposer);
}

#[test]
fn current_return_focus_reflects_key_surface() {
    let mut state = OrchestratorState::default();
    state.main.return_focus = FocusToken::PromptInput;
    assert_eq!(current_return_focus(&state), FocusToken::PromptInput);

    state.key_surface = Some(SurfaceId::Notes);
    assert_eq!(current_return_focus(&state), FocusToken::NotesEditor);

    state.key_surface = Some(SurfaceId::DetachedAiChat);
    assert_eq!(current_return_focus(&state), FocusToken::DetachedAiComposer);

    state.key_surface = None;
    assert_eq!(current_return_focus(&state), FocusToken::None);
}

#[test]
fn close_notes_when_main_hidden_key_goes_to_none() {
    let mut state = OrchestratorState::default();
    state.main.visibility = MainVisibility::Hidden(HiddenReason::Dismissed);
    state.notes.visible = true;
    state.key_surface = Some(SurfaceId::Notes);

    state.dispatch(WindowEvent::CloseNotes);

    // No visible surface left.
    assert_eq!(state.key_surface, None);
}

#[test]
fn start_dictation_closes_open_actions() {
    let mut state = OrchestratorState::default();
    state.main.actions_open = true;

    let cmds = state.dispatch(WindowEvent::StartDictation {
        target: DictationTarget::MainWindowPrompt,
    });

    assert!(!state.main.actions_open);
    assert!(cmds.contains(&WindowCommand::CloseActions));
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

#[test]
fn finish_dictation_from_main_filter_reveals_and_restores_main_filter_focus() {
    let mut state = OrchestratorState::default();
    // Default state: ScriptList with MainFilter focus.
    assert_eq!(state.main.return_focus, FocusToken::MainFilter);

    state.dispatch(WindowEvent::StartDictation {
        target: DictationTarget::MainWindowFilter,
    });

    // Main should be concealed during dictation.
    assert_eq!(
        state.main.visibility,
        MainVisibility::Hidden(HiddenReason::Concealed)
    );

    let cmds = state.dispatch(WindowEvent::FinishDictation);

    // Main should be revealed and focus restored to MainFilter.
    assert_eq!(state.dictation, DictationSurfaceState::Hidden);
    assert_eq!(state.main.visibility, MainVisibility::Visible);
    assert_eq!(state.key_surface, Some(SurfaceId::Main));
    assert!(cmds.contains(&WindowCommand::CloseDictationOverlay));
    assert!(cmds.contains(&WindowCommand::RevealMain {
        activate_app: false,
        make_key: true,
    }));
    assert!(cmds.contains(&WindowCommand::FocusMain(FocusToken::MainFilter)));
}

#[test]
fn start_dictation_from_script_list_filter_conceals_and_opens_overlay() {
    let state = OrchestratorState::default();

    let t = reduce(
        &state,
        WindowEvent::StartDictation {
            target: DictationTarget::MainWindowFilter,
        },
    );

    assert!(matches!(
        t.next.dictation,
        DictationSurfaceState::Visible {
            phase: DictationPhase::Recording,
            target: DictationTarget::MainWindowFilter,
            restore_main_visibility: true,
            ..
        }
    ));
    assert!(t.commands.contains(&WindowCommand::ConcealMain));
    assert!(t.commands.contains(&WindowCommand::OpenDictationOverlay {
        phase: DictationPhase::Recording,
    }));
}
