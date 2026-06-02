use gpui::{
    div, prelude::*, px, rgb, rgba, Context, Entity, FocusHandle, IntoElement, KeyDownEvent,
    ParentElement, Render, Window,
};

use super::super::catalog::{
    AcpAgentAuthState, AcpAgentCatalogEntry, AcpAgentConfigState, AcpAgentInstallState,
};
use super::super::preflight::AcpLaunchRequirements;
use super::super::setup_state::{AcpInlineSetupState, AcpSetupAction};
use crate::theme;
use crate::ui_foundation;

/// State for the setup-mode agent selection picker.
#[derive(Debug, Clone)]
pub struct AcpSetupAgentPickerState {
    pub items: Vec<AcpAgentCatalogEntry>,
    pub selected_index: usize,
    pub visible_start: usize,
}

#[allow(clippy::large_enum_variant)]
pub enum AcpSetupCardEvent {
    ConfirmAgent(AcpAgentCatalogEntry),
    CancelPicker,
    OpenPicker,
    Retry,
}

impl gpui::EventEmitter<AcpSetupCardEvent> for AcpSetupCard {}

pub struct AcpSetupCard {
    state: AcpInlineSetupState,
    pub(crate) agent_picker: Option<AcpSetupAgentPickerState>,
    focus_handle: FocusHandle,
}

impl AcpSetupCard {
    pub fn new(
        state: AcpInlineSetupState,
        agent_picker: Option<AcpSetupAgentPickerState>,
        cx: &mut Context<Self>,
    ) -> Self {
        Self {
            state,
            agent_picker,
            focus_handle: cx.focus_handle(),
        }
    }

    pub fn set_state(&mut self, state: AcpInlineSetupState, cx: &mut Context<Self>) {
        self.state = state;
        cx.notify();
    }

    pub fn set_agent_picker(
        &mut self,
        picker: Option<AcpSetupAgentPickerState>,
        cx: &mut Context<Self>,
    ) {
        self.agent_picker = picker;
        cx.notify();
    }

    pub fn select_agent_by_id(&mut self, agent_id: &str, cx: &mut Context<Self>) -> bool {
        if let Some(ref mut picker) = self.agent_picker {
            if let Some(idx) = picker.items.iter().position(|item| item.id == agent_id) {
                picker.selected_index = idx;
                cx.notify();
                return true;
            }
        }
        false
    }

    pub fn handle_key_down(&mut self, event: &KeyDownEvent, cx: &mut Context<Self>) -> bool {
        let key = event.keystroke.key.as_str();

        if let Some(ref mut picker) = self.agent_picker {
            if ui_foundation::is_key_up(key) {
                if picker.selected_index > 0 {
                    picker.selected_index -= 1;
                }
                cx.notify();
                return true;
            }
            if ui_foundation::is_key_down(key) {
                if picker.selected_index + 1 < picker.items.len() {
                    picker.selected_index += 1;
                }
                cx.notify();
                return true;
            }
            if ui_foundation::is_key_enter(key) {
                if let Some(agent) = picker.items.get(picker.selected_index).cloned() {
                    cx.emit(AcpSetupCardEvent::ConfirmAgent(agent));
                }
                return true;
            }
            if ui_foundation::is_key_escape(key) {
                self.agent_picker = None;
                cx.emit(AcpSetupCardEvent::CancelPicker);
                cx.notify();
                return true;
            }
            return false;
        }

        if ui_foundation::is_key_tab(key) {
            cx.emit(AcpSetupCardEvent::Retry);
            return true;
        }

        if ui_foundation::is_key_enter(key) {
            cx.emit(AcpSetupCardEvent::OpenPicker);
            return true;
        }

        false
    }

    fn render_agent_picker(
        &self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<gpui::AnyElement> {
        let picker = self.agent_picker.as_ref()?;
        let theme = theme::get_cached_theme();

        let rows: Vec<gpui::AnyElement> = picker
            .items
            .iter()
            .enumerate()
            .map(|(ix, item)| {
                let is_selected = ix == picker.selected_index;
                let status_text =
                    format_setup_agent_picker_status(item, self.state.launch_requirements);
                div()
                    .id(gpui::ElementId::Name(
                        format!("acp-setup-agent-{ix}").into(),
                    ))
                    .w_full()
                    .px(px(10.0))
                    .py(px(5.0))
                    .when(is_selected, |d| {
                        d.bg(rgba((theme.colors.accent.selected << 8) | 0x14))
                            .border_l_2()
                            .border_color(rgb(theme.colors.accent.selected))
                    })
                    .when(!is_selected, |d| {
                        d.border_l_2().border_color(gpui::transparent_black())
                    })
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(1.0))
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(theme.colors.text.primary))
                                    .child(item.display_name.clone()),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(theme.colors.text.muted))
                                    .child(status_text),
                            ),
                    )
                    .into_any_element()
            })
            .collect();

        Some(
            div()
                .w_full()
                .max_w(px(400.0))
                .max_h(px(300.0))
                .overflow_y_hidden()
                .bg(rgb(theme.colors.background.main))
                .border_1()
                .border_color(rgb(theme.colors.ui.border))
                .rounded_md()
                .children(rows)
                .into_any_element(),
        )
    }
}

impl Render for AcpSetupCard {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = theme::get_cached_theme();
        let state = &self.state;

        let action_hint: String = match state.primary_action {
            AcpSetupAction::Retry => "Press Enter to retry".to_string(),
            AcpSetupAction::Install => "Install the agent, then press Enter to retry".to_string(),
            AcpSetupAction::Authenticate => "Authenticate, then press Enter to retry".to_string(),
            AcpSetupAction::OpenCatalog => {
                "Add or edit an Agent Chat profile in config.ts, then press Enter to retry"
                    .to_string()
            }
            AcpSetupAction::SelectAgent => "Press Enter to select a different agent".to_string(),
        };

        let secondary_hint: Option<String> = state.secondary_action.map(|action| match action {
            AcpSetupAction::SelectAgent => "Enter: select agent".to_string(),
            AcpSetupAction::Retry => "Enter: retry".to_string(),
            AcpSetupAction::OpenCatalog => "Add agent".to_string(),
            _ => String::new(),
        });

        let agent_name: Option<String> = state
            .selected_agent
            .as_ref()
            .map(|a| format!("Selected: {}", a.display_name));

        div()
            .id("acp-inline-setup")
            .size_full()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap(px(16.0))
            .track_focus(&self.focus_handle)
            .child(
                div()
                    .text_xl()
                    .text_color(rgb(theme.colors.text.primary))
                    .child(state.title.clone()),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(theme.colors.text.muted))
                    .max_w(px(400.0))
                    .text_center()
                    .child(state.body.clone()),
            )
            .when_some(agent_name, |d, name| {
                d.child(
                    div()
                        .text_xs()
                        .text_color(rgb(theme.colors.text.muted))
                        .child(name),
                )
            })
            .when_some(self.render_agent_picker(window, cx), |d, picker| {
                d.child(picker)
            })
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(theme.colors.text.muted))
                    .child(action_hint),
            )
            .when_some(secondary_hint.filter(|s| !s.is_empty()), |d, hint| {
                d.child(
                    div()
                        .text_xs()
                        .text_color(rgb(theme.colors.text.muted))
                        .opacity(0.6)
                        .child(hint),
                )
            })
    }
}

fn setup_agent_install_label(state: AcpAgentInstallState) -> &'static str {
    match state {
        AcpAgentInstallState::Ready => "ready",
        AcpAgentInstallState::NeedsInstall => "install",
        AcpAgentInstallState::Unsupported => "unsupported",
    }
}

fn setup_agent_auth_label(state: AcpAgentAuthState) -> &'static str {
    match state {
        AcpAgentAuthState::Unknown => "auth?",
        AcpAgentAuthState::Authenticated => "authed",
        AcpAgentAuthState::NeedsAuthentication => "login",
    }
}

fn setup_agent_config_label(state: AcpAgentConfigState) -> &'static str {
    match state {
        AcpAgentConfigState::Valid => "config-ok",
        AcpAgentConfigState::Missing => "config-missing",
        AcpAgentConfigState::Invalid => "config-invalid",
    }
}

fn setup_agent_capability_label(
    entry: &AcpAgentCatalogEntry,
    requirements: AcpLaunchRequirements,
) -> Option<&'static str> {
    if !requirements.needs_embedded_context && !requirements.needs_image {
        return None;
    }
    if entry.satisfies_requirements(requirements) {
        Some("compatible")
    } else if requirements.needs_image {
        Some("image-mismatch")
    } else {
        Some("context-mismatch")
    }
}

fn format_setup_agent_picker_status(
    entry: &AcpAgentCatalogEntry,
    requirements: AcpLaunchRequirements,
) -> String {
    let mut parts = vec![
        format!("{:?}", entry.source),
        setup_agent_install_label(entry.install_state).to_string(),
        setup_agent_auth_label(entry.auth_state).to_string(),
        setup_agent_config_label(entry.config_state).to_string(),
    ];
    if let Some(capability) = setup_agent_capability_label(entry, requirements) {
        parts.push(capability.to_string());
    }
    if entry.last_session_ok {
        parts.push("last-ok".to_string());
    }
    parts.join(" \u{00b7} ")
}
