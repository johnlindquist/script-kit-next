//! Pure state machine for window visibility and focus management.
//!
//! `State + Event → (State, Vec<Command>)` — no side effects, no AppKit, no GPUI.

// Production calls dispatch through `app_impl/window_orchestrator_bridge.rs`;
// keeping the reducer pure lets every transition be proved without a window.
pub mod executor;
pub mod interaction;

#[cfg(test)]
mod tests;

// ---------------------------------------------------------------------------
// Surface identity
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SurfaceId {
    Main,
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
    AgentChat,
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
// Top-level orchestrator state
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrchestratorState {
    pub main: MainSurfaceState,
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

    FocusSurface(SurfaceId),
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

/// Pick the next visible surface for key focus (only Main).
fn next_visible_surface(state: &OrchestratorState) -> Option<SurfaceId> {
    if state.main.visibility == MainVisibility::Visible {
        return Some(SurfaceId::Main);
    }
    None
}
