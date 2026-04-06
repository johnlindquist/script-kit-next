//! Pure state machine for window visibility and focus management.
//!
//! `State + Event → (State, Vec<Command>)` — no side effects, no AppKit, no GPUI.

// Wired up but not yet called from production code — Task #3 (dictation
// migration) will connect these. Allow dead_code until then.
#[allow(dead_code)]
pub mod executor;

#[cfg(test)]
mod tests;

// ---------------------------------------------------------------------------
// Surface identity
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SurfaceId {
    Main,
    Notes,
    DetachedAiChat,
    DictationOverlay,
}

// ---------------------------------------------------------------------------
// Main surface
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MainVisibility {
    Visible,
    Hidden(HiddenReason),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HiddenReason {
    /// Temporary hide — preserve current AppView/prompt.
    Concealed,
    /// Full launcher dismissal — reset to ScriptList.
    Dismissed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MainContentKind {
    ScriptList,
    Prompt,
    AcpChat,
    QuickTerminal,
    FileSearch,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusToken {
    MainFilter,
    PromptInput,
    ChatComposer,
    TermInput,
    NotesEditor,
    DetachedAiComposer,
    None,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MainSurfaceState {
    pub visibility: MainVisibility,
    pub content: MainContentKind,
    pub return_focus: FocusToken,
    pub actions_open: bool,
}

// ---------------------------------------------------------------------------
// Auxiliary surfaces
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AuxSurfaceState {
    pub visible: bool,
}

// ---------------------------------------------------------------------------
// Dictation surface
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DictationPhase {
    Recording,
    Confirming,
    Transcribing,
    Delivering,
    Finished,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DictationTarget {
    MainWindowFilter,
    MainWindowPrompt,
    NotesEditor,
    AiChatComposer,
    TabAiHarness,
    ExternalApp,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DictationSurfaceState {
    Hidden,
    Visible {
        phase: DictationPhase,
        target: DictationTarget,
        return_to: FocusToken,
        restore_main_visibility: bool,
    },
}

// ---------------------------------------------------------------------------
// Top-level orchestrator state
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrchestratorState {
    pub main: MainSurfaceState,
    pub notes: AuxSurfaceState,
    pub detached_ai: AuxSurfaceState,
    pub dictation: DictationSurfaceState,
    pub key_surface: Option<SurfaceId>,
}

impl Default for OrchestratorState {
    fn default() -> Self {
        Self {
            main: MainSurfaceState {
                visibility: MainVisibility::Visible,
                content: MainContentKind::ScriptList,
                return_focus: FocusToken::MainFilter,
                actions_open: false,
            },
            notes: AuxSurfaceState::default(),
            detached_ai: AuxSurfaceState::default(),
            dictation: DictationSurfaceState::Hidden,
            key_surface: Some(SurfaceId::Main),
        }
    }
}

impl OrchestratorState {
    /// Convenience: mutate in-place and return the commands.
    pub fn dispatch(&mut self, event: WindowEvent) -> Vec<WindowCommand> {
        let transition = reduce(self, event);
        *self = transition.next;
        transition.commands
    }
}

/// Derive the focus token that matches the current key surface.
pub fn current_return_focus(state: &OrchestratorState) -> FocusToken {
    match state.key_surface {
        Some(SurfaceId::Main) => state.main.return_focus,
        Some(SurfaceId::Notes) => FocusToken::NotesEditor,
        Some(SurfaceId::DetachedAiChat) => FocusToken::DetachedAiComposer,
        Some(SurfaceId::DictationOverlay) => {
            if let DictationSurfaceState::Visible { return_to, .. } = &state.dictation {
                *return_to
            } else {
                FocusToken::None
            }
        }
        None => FocusToken::None,
    }
}

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WindowEvent {
    ShowMain {
        activate_app: bool,
    },
    DismissMain,
    ConcealMain,

    OpenActions,
    CloseActions,

    OpenNotes,
    CloseNotes,

    OpenDetachedAiChat,
    CloseDetachedAiChat,

    StartDictation {
        target: DictationTarget,
    },
    DictationPhaseChanged(DictationPhase),
    FinishDictation,
    AbortDictation,

    SurfaceClosedBySystem(SurfaceId),

    MainContentChanged {
        content: MainContentKind,
        focus: FocusToken,
    },
}

// ---------------------------------------------------------------------------
// Commands (side-effects the caller must execute)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WindowCommand {
    // Main window
    SaveMainBounds,
    ConcealMain,
    DismissMain,
    RevealMain { activate_app: bool, make_key: bool },
    FocusMain(FocusToken),

    // Actions dialog
    OpenActions,
    CloseActions,

    // Notes
    OpenNotesWindow,
    CloseNotesWindow,
    FocusSurface(SurfaceId),

    // Detached AI
    OpenDetachedAiChatWindow,
    CloseDetachedAiChatWindow,

    // Dictation
    OpenDictationOverlay { phase: DictationPhase },
    UpdateDictationOverlay { phase: DictationPhase },
    CloseDictationOverlay,
}

// ---------------------------------------------------------------------------
// Transition
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transition {
    pub next: OrchestratorState,
    pub commands: Vec<WindowCommand>,
}

// ---------------------------------------------------------------------------
// Reducer
// ---------------------------------------------------------------------------

/// Pure reducer: `State + Event → (State, Vec<Command>)`.
pub fn reduce(state: &OrchestratorState, event: WindowEvent) -> Transition {
    let mut next = state.clone();
    let mut cmds: Vec<WindowCommand> = Vec::new();

    match event {
        // ---------------------------------------------------------------
        // Main window
        // ---------------------------------------------------------------
        WindowEvent::ShowMain { activate_app } => {
            next.main.visibility = MainVisibility::Visible;
            next.key_surface = Some(SurfaceId::Main);
            cmds.push(WindowCommand::RevealMain {
                activate_app,
                make_key: true,
            });
            cmds.push(WindowCommand::FocusMain(next.main.return_focus));
        }

        WindowEvent::DismissMain => {
            cmds.push(WindowCommand::SaveMainBounds);
            if next.main.actions_open {
                next.main.actions_open = false;
                cmds.push(WindowCommand::CloseActions);
            }
            next.main.visibility = MainVisibility::Hidden(HiddenReason::Dismissed);
            next.main.content = MainContentKind::ScriptList;
            next.main.return_focus = FocusToken::MainFilter;
            cmds.push(WindowCommand::DismissMain);
            if next.key_surface == Some(SurfaceId::Main) {
                next.key_surface = next_visible_surface(&next);
            }
        }

        WindowEvent::ConcealMain => {
            cmds.push(WindowCommand::SaveMainBounds);
            if next.main.actions_open {
                next.main.actions_open = false;
                cmds.push(WindowCommand::CloseActions);
            }
            next.main.visibility = MainVisibility::Hidden(HiddenReason::Concealed);
            cmds.push(WindowCommand::ConcealMain);
            if next.key_surface == Some(SurfaceId::Main) {
                next.key_surface = next_visible_surface(&next);
            }
        }

        // ---------------------------------------------------------------
        // Actions dialog (child of main, not a top-level surface)
        // ---------------------------------------------------------------
        WindowEvent::OpenActions => {
            if !next.main.actions_open {
                next.main.actions_open = true;
                cmds.push(WindowCommand::OpenActions);
            }
        }

        WindowEvent::CloseActions => {
            if next.main.actions_open {
                next.main.actions_open = false;
                cmds.push(WindowCommand::CloseActions);
            }
        }

        // ---------------------------------------------------------------
        // Notes
        // ---------------------------------------------------------------
        WindowEvent::OpenNotes => {
            next.notes.visible = true;
            next.key_surface = Some(SurfaceId::Notes);
            cmds.push(WindowCommand::OpenNotesWindow);
            cmds.push(WindowCommand::FocusSurface(SurfaceId::Notes));
        }

        WindowEvent::CloseNotes => {
            next.notes.visible = false;
            cmds.push(WindowCommand::CloseNotesWindow);
            if next.key_surface == Some(SurfaceId::Notes) {
                next.key_surface = next_visible_surface(&next);
                if let Some(sid) = next.key_surface {
                    cmds.push(WindowCommand::FocusSurface(sid));
                }
            }
        }

        // ---------------------------------------------------------------
        // Detached AI Chat
        // ---------------------------------------------------------------
        WindowEvent::OpenDetachedAiChat => {
            next.detached_ai.visible = true;
            next.key_surface = Some(SurfaceId::DetachedAiChat);
            cmds.push(WindowCommand::OpenDetachedAiChatWindow);
            cmds.push(WindowCommand::FocusSurface(SurfaceId::DetachedAiChat));
        }

        WindowEvent::CloseDetachedAiChat => {
            next.detached_ai.visible = false;
            cmds.push(WindowCommand::CloseDetachedAiChatWindow);
            if next.key_surface == Some(SurfaceId::DetachedAiChat) {
                next.key_surface = next_visible_surface(&next);
                if let Some(sid) = next.key_surface {
                    cmds.push(WindowCommand::FocusSurface(sid));
                }
            }
        }

        // ---------------------------------------------------------------
        // Dictation
        // ---------------------------------------------------------------
        WindowEvent::StartDictation { target } => {
            // Idempotent — if already visible, no-op.
            if matches!(next.dictation, DictationSurfaceState::Visible { .. }) {
                return Transition {
                    next,
                    commands: cmds,
                };
            }

            // Close actions if open.
            if next.main.actions_open {
                next.main.actions_open = false;
                cmds.push(WindowCommand::CloseActions);
            }

            let return_to = current_return_focus(state);
            let restore_main = next.main.visibility == MainVisibility::Visible;

            // Conceal main if visible.
            if restore_main {
                cmds.push(WindowCommand::SaveMainBounds);
                next.main.visibility = MainVisibility::Hidden(HiddenReason::Concealed);
                cmds.push(WindowCommand::ConcealMain);
            }

            next.dictation = DictationSurfaceState::Visible {
                phase: DictationPhase::Recording,
                target,
                return_to,
                restore_main_visibility: restore_main,
            };
            next.key_surface = Some(SurfaceId::DictationOverlay);
            cmds.push(WindowCommand::OpenDictationOverlay {
                phase: DictationPhase::Recording,
            });
        }

        WindowEvent::DictationPhaseChanged(phase) => {
            if let DictationSurfaceState::Visible {
                phase: ref mut p, ..
            } = next.dictation
            {
                *p = phase;
                cmds.push(WindowCommand::UpdateDictationOverlay { phase });
            }
        }

        WindowEvent::FinishDictation | WindowEvent::AbortDictation => {
            if let DictationSurfaceState::Visible {
                return_to,
                restore_main_visibility,
                ..
            } = next.dictation
            {
                cmds.push(WindowCommand::CloseDictationOverlay);
                next.dictation = DictationSurfaceState::Hidden;

                if restore_main_visibility {
                    next.main.visibility = MainVisibility::Visible;
                    let focus_main = matches!(
                        return_to,
                        FocusToken::MainFilter
                            | FocusToken::PromptInput
                            | FocusToken::ChatComposer
                            | FocusToken::TermInput
                    );
                    cmds.push(WindowCommand::RevealMain {
                        activate_app: false,
                        make_key: focus_main,
                    });
                    if focus_main {
                        next.key_surface = Some(SurfaceId::Main);
                        cmds.push(WindowCommand::FocusMain(return_to));
                    }
                }

                // If return focus is to an aux surface, focus it.
                match return_to {
                    FocusToken::NotesEditor => {
                        next.key_surface = Some(SurfaceId::Notes);
                        cmds.push(WindowCommand::FocusSurface(SurfaceId::Notes));
                    }
                    FocusToken::DetachedAiComposer => {
                        next.key_surface = Some(SurfaceId::DetachedAiChat);
                        cmds.push(WindowCommand::FocusSurface(SurfaceId::DetachedAiChat));
                    }
                    _ => {
                        // Already handled above for main-bound tokens,
                        // or FocusToken::None — pick next visible.
                        if !restore_main_visibility
                            && !matches!(
                                return_to,
                                FocusToken::MainFilter
                                    | FocusToken::PromptInput
                                    | FocusToken::ChatComposer
                                    | FocusToken::TermInput
                            )
                        {
                            next.key_surface = next_visible_surface(&next);
                        }
                    }
                }
            }
        }

        // ---------------------------------------------------------------
        // System close (user hit native close button, etc.)
        // ---------------------------------------------------------------
        WindowEvent::SurfaceClosedBySystem(surface) => match surface {
            SurfaceId::Main => {
                next.main.visibility = MainVisibility::Hidden(HiddenReason::Dismissed);
                next.main.content = MainContentKind::ScriptList;
                next.main.return_focus = FocusToken::MainFilter;
                if next.main.actions_open {
                    next.main.actions_open = false;
                }
                if next.key_surface == Some(SurfaceId::Main) {
                    next.key_surface = next_visible_surface(&next);
                }
            }
            SurfaceId::Notes => {
                next.notes.visible = false;
                if next.key_surface == Some(SurfaceId::Notes) {
                    next.key_surface = next_visible_surface(&next);
                }
            }
            SurfaceId::DetachedAiChat => {
                next.detached_ai.visible = false;
                if next.key_surface == Some(SurfaceId::DetachedAiChat) {
                    next.key_surface = next_visible_surface(&next);
                }
            }
            SurfaceId::DictationOverlay => {
                next.dictation = DictationSurfaceState::Hidden;
                if next.key_surface == Some(SurfaceId::DictationOverlay) {
                    next.key_surface = next_visible_surface(&next);
                }
            }
        },

        // ---------------------------------------------------------------
        // Content tracking
        // ---------------------------------------------------------------
        WindowEvent::MainContentChanged { content, focus } => {
            next.main.content = content;
            next.main.return_focus = focus;
        }
    }

    Transition {
        next,
        commands: cmds,
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Pick the next visible surface for key focus (priority: Main > Notes > DetachedAi).
fn next_visible_surface(state: &OrchestratorState) -> Option<SurfaceId> {
    if state.main.visibility == MainVisibility::Visible {
        return Some(SurfaceId::Main);
    }
    if state.notes.visible {
        return Some(SurfaceId::Notes);
    }
    if state.detached_ai.visible {
        return Some(SurfaceId::DetachedAiChat);
    }
    None
}
