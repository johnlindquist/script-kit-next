#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InlineOverlayAttachment {
    Standalone,
    AttachedToParent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InlineAgentExecutorMode {
    AgentChatPi,
    MockFixture,
}

use std::sync::{Mutex, OnceLock};

use crate::ai::inline_agent::agent_chat_adapter::{
    prepare_default_agent_chat_inline_agent_warm_session,
    spawn_default_agent_chat_inline_agent_executor,
};
use crate::ai::inline_agent::executor::InlineAgentExecutor;
use crate::ai::inline_agent::{
    InlineAgentEditSemantics, InlineAgentPhase, InlineAgentProviderEvent, InlineAgentSession,
};
use gpui::{
    div, prelude::FluentBuilder, px, rgb, App, AppContext, Bounds, Context, DisplayId, FocusHandle,
    InteractiveElement, IntoElement, ParentElement, Pixels, Render, StatefulInteractiveElement,
    Styled, Task, Window, WindowBackgroundAppearance, WindowBounds, WindowHandle, WindowKind,
    WindowOptions,
};

use crate::platform::accessibility::geometry::{preferred_anchor_geometry, RectPx};
use crate::platform::accessibility::{
    capture_focused_text_field, CaptureFocusedTextOptions, FocusedTextSnapshot,
};
use crate::protocol::{AutomationWindowBounds, AutomationWindowInfo, AutomationWindowKind};

use super::automation::INLINE_AGENT_COMPACT_ID;
use super::layout::{place_compact_overlay, place_expanded_overlay, InlineAgentLayoutDefaults};
use super::render_actions::apply_latest_output_action;
use super::render_compact::{compact_view_model, InlineAgentCompactViewModel};
use super::render_expanded::{expanded_view_model, InlineAgentExpandedViewModel};
use super::state::{InlineAgentMode, InlineAgentRunState};
use super::theme::InlineAgentColors;
use super::types::{InlineAgentMutationReceipt, InlineAgentOutputAction, InlineAgentSnapshot};
use super::SystemInlineAgentPlatformBridge;

pub const INLINE_AGENT_WINDOW_AUTOMATION_ID: &str = "inline-agent";
pub const INLINE_AGENT_WINDOW_TITLE: &str = "Inline Agent";
pub const INLINE_AGENT_SEMANTIC_SURFACE: &str = "inlineAgent";
const INLINE_AGENT_REAL_PI_FIXTURE_ENV: &str = "SCRIPT_KIT_INLINE_AGENT_REAL_PI_FIXTURE";

static INLINE_AGENT_OVERLAY_WINDOW: OnceLock<
    Mutex<Option<WindowHandle<InlineAgentOverlayWindow>>>,
> = OnceLock::new();
static INLINE_AGENT_WINDOW_SNAPSHOT: OnceLock<Mutex<Option<InlineAgentWindowSnapshot>>> =
    OnceLock::new();

#[derive(Debug, Clone, PartialEq)]
pub struct InlineAgentWindowSnapshot {
    pub attachment: InlineOverlayAttachment,
    pub mode: InlineAgentMode,
    pub run_state: InlineAgentRunState,
    pub session_id: String,
    pub app_name: String,
    pub can_replace: bool,
    pub can_append: bool,
    pub can_copy: bool,
    pub bounds: RectPx,
    pub focus_prompt: bool,
}

pub fn plan_open_inline_agent_overlay(
    snapshot: &InlineAgentSnapshot,
    attachment: InlineOverlayAttachment,
) -> InlineAgentWindowSnapshot {
    let run_state = InlineAgentRunState::Idle;
    let bounds = compact_bounds_for_run_state(snapshot, &run_state);

    InlineAgentWindowSnapshot {
        attachment,
        mode: InlineAgentMode::Compact,
        run_state,
        session_id: snapshot.session_id.to_string(),
        app_name: snapshot.app.name.clone(),
        can_replace: snapshot.capabilities.can_replace,
        can_append: snapshot.capabilities.can_append,
        can_copy: snapshot.capabilities.can_copy,
        bounds,
        focus_prompt: true,
    }
}

pub fn plan_expanded_inline_agent_overlay(
    snapshot: &InlineAgentSnapshot,
    current: &InlineAgentWindowSnapshot,
) -> InlineAgentWindowSnapshot {
    let anchor = preferred_anchor_geometry(&snapshot.anchor.geometry);

    InlineAgentWindowSnapshot {
        attachment: current.attachment,
        mode: InlineAgentMode::Expanded,
        run_state: current.run_state.clone(),
        session_id: current.session_id.clone(),
        app_name: current.app_name.clone(),
        can_replace: current.can_replace,
        can_append: current.can_append,
        can_copy: current.can_copy,
        bounds: place_expanded_overlay(anchor, snapshot.anchor.geometry.display_bounds),
        focus_prompt: true,
    }
}

pub fn plan_compact_inline_agent_overlay(
    snapshot: &InlineAgentSnapshot,
    current: &InlineAgentWindowSnapshot,
) -> InlineAgentWindowSnapshot {
    InlineAgentWindowSnapshot {
        attachment: current.attachment,
        mode: InlineAgentMode::Compact,
        run_state: current.run_state.clone(),
        session_id: current.session_id.clone(),
        app_name: current.app_name.clone(),
        can_replace: current.can_replace,
        can_append: current.can_append,
        can_copy: current.can_copy,
        bounds: compact_bounds_for_run_state(snapshot, &current.run_state),
        focus_prompt: true,
    }
}

pub fn inline_agent_window_options(
    plan: &InlineAgentWindowSnapshot,
    display_id: Option<DisplayId>,
) -> WindowOptions {
    let theme = crate::theme::get_cached_theme();
    let window_background = if theme.is_vibrancy_enabled() {
        WindowBackgroundAppearance::Blurred
    } else {
        WindowBackgroundAppearance::Opaque
    };

    WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(rect_to_gpui_bounds(plan.bounds))),
        titlebar: None,
        window_background,
        focus: plan.focus_prompt,
        show: true,
        kind: WindowKind::PopUp,
        is_movable: false,
        is_resizable: false,
        display_id,
        ..Default::default()
    }
}

pub fn inline_agent_automation_info(plan: &InlineAgentWindowSnapshot) -> AutomationWindowInfo {
    AutomationWindowInfo {
        id: INLINE_AGENT_WINDOW_AUTOMATION_ID.to_string(),
        kind: AutomationWindowKind::MiniAi,
        title: Some(INLINE_AGENT_WINDOW_TITLE.to_string()),
        focused: plan.focus_prompt,
        visible: true,
        semantic_surface: Some(INLINE_AGENT_SEMANTIC_SURFACE.to_string()),
        bounds: Some(rect_to_automation_bounds(plan.bounds)),
        parent_window_id: None,
        parent_kind: None,
        pid: None,
    }
}

pub fn register_inline_agent_automation_window(plan: &InlineAgentWindowSnapshot) {
    update_inline_agent_window_snapshot(plan);
    crate::windows::upsert_automation_window(inline_agent_automation_info(plan));
}

pub fn update_inline_agent_automation_bounds(plan: &InlineAgentWindowSnapshot) {
    update_inline_agent_window_snapshot(plan);
    crate::windows::set_automation_bounds(
        INLINE_AGENT_WINDOW_AUTOMATION_ID,
        Some(rect_to_automation_bounds(plan.bounds)),
    );
}

pub fn remove_inline_agent_automation_window() {
    clear_inline_agent_window_snapshot();
    crate::windows::remove_automation_window(INLINE_AGENT_WINDOW_AUTOMATION_ID);
}

pub fn inline_agent_current_window_snapshot() -> Option<InlineAgentWindowSnapshot> {
    INLINE_AGENT_WINDOW_SNAPSHOT
        .get()
        .and_then(|storage| storage.lock().ok())
        .and_then(|guard| guard.clone())
}

pub fn inline_agent_automation_state() -> Option<serde_json::Value> {
    let snapshot = inline_agent_current_window_snapshot()?;
    Some(serde_json::json!({
        "schemaVersion": 1,
        "surface": INLINE_AGENT_SEMANTIC_SURFACE,
        "sessionId": snapshot.session_id,
        "appName": snapshot.app_name,
        "attachment": match snapshot.attachment {
            InlineOverlayAttachment::Standalone => "standalone",
            InlineOverlayAttachment::AttachedToParent => "attachedToParent",
        },
        "mode": match snapshot.mode {
            InlineAgentMode::Compact => "compact",
            InlineAgentMode::Expanded => "expanded",
        },
        "phase": inline_agent_run_state_phase(&snapshot.run_state),
        "focusedPrompt": snapshot.focus_prompt,
        "output": inline_agent_run_state_output_summary(&snapshot.run_state),
        "actions": inline_agent_run_state_action_summary(&snapshot),
        "lastMutation": inline_agent_run_state_mutation_summary(&snapshot.run_state),
    }))
}

fn update_inline_agent_window_snapshot(plan: &InlineAgentWindowSnapshot) {
    let storage = INLINE_AGENT_WINDOW_SNAPSHOT.get_or_init(|| Mutex::new(None));
    if let Ok(mut guard) = storage.lock() {
        *guard = Some(plan.clone());
    }
}

fn clear_inline_agent_window_snapshot() {
    if let Some(storage) = INLINE_AGENT_WINDOW_SNAPSHOT.get() {
        if let Ok(mut guard) = storage.lock() {
            *guard = None;
        }
    }
}

fn clear_inline_agent_overlay_storage() {
    if let Some(storage) = INLINE_AGENT_OVERLAY_WINDOW.get() {
        if let Ok(mut guard) = storage.lock() {
            *guard = None;
        }
    }
}

pub fn compact_root_automation_id() -> &'static str {
    INLINE_AGENT_COMPACT_ID
}

pub fn sync_inline_agent_overlay_window(
    cx: &mut App,
    focused_snapshot: FocusedTextSnapshot,
    plan: InlineAgentWindowSnapshot,
    display_id: Option<DisplayId>,
) -> anyhow::Result<()> {
    sync_inline_agent_overlay_window_with_executor_mode(
        cx,
        focused_snapshot,
        plan,
        display_id,
        InlineAgentExecutorMode::AgentChatPi,
    )
}

fn sync_inline_agent_overlay_window_with_executor_mode(
    cx: &mut App,
    focused_snapshot: FocusedTextSnapshot,
    plan: InlineAgentWindowSnapshot,
    display_id: Option<DisplayId>,
    executor_mode: InlineAgentExecutorMode,
) -> anyhow::Result<()> {
    let storage = INLINE_AGENT_OVERLAY_WINDOW.get_or_init(|| Mutex::new(None));
    if let Ok(mut guard) = storage.lock() {
        if let Some(handle) = guard.as_ref() {
            let update_result = handle.update(cx, |overlay, window, cx| {
                overlay.set_snapshot(focused_snapshot.clone(), plan.clone(), executor_mode);
                let bounds = rect_to_gpui_bounds(plan.bounds);
                crate::components::inline_popup_window::set_inline_popup_window_bounds(
                    window, bounds, cx,
                );
                update_inline_agent_automation_bounds(&plan);
                cx.notify();
            });

            if update_result.is_ok() {
                prewarm_inline_agent_executor_mode(executor_mode);
                return Ok(());
            }

            remove_inline_agent_automation_window();
            *guard = None;
        }
    }

    let options = inline_agent_window_options(&plan, display_id);
    let handle = cx.open_window(options, |_window, cx| {
        cx.new(|cx| {
            InlineAgentOverlayWindow::new(focused_snapshot.clone(), plan.clone(), executor_mode, cx)
        })
    })?;

    configure_inline_agent_overlay_window(&handle, cx)?;
    register_inline_agent_automation_window(&plan);

    if let Ok(mut guard) = storage.lock() {
        *guard = Some(handle);
    }

    prewarm_inline_agent_executor_mode(executor_mode);

    Ok(())
}

fn prewarm_inline_agent_executor_mode(executor_mode: InlineAgentExecutorMode) {
    if executor_mode != InlineAgentExecutorMode::AgentChatPi {
        return;
    }

    std::thread::spawn(|| {
        if let Err(error) = prepare_default_agent_chat_inline_agent_warm_session() {
            tracing::warn!(
                target: "script_kit::inline_agent",
                event = "inline_agent_pi_prewarm_failed",
                error = %error,
            );
        }
    });
}

pub fn open_inline_agent_mock_fixture(
    cx: &mut App,
    text: Option<String>,
    instruction: Option<String>,
) -> anyhow::Result<()> {
    open_inline_agent_fixture_with_executor_mode(
        cx,
        text,
        instruction,
        InlineAgentExecutorMode::MockFixture,
    )
}

pub fn open_inline_agent_pi_fixture(
    cx: &mut App,
    text: Option<String>,
    instruction: Option<String>,
) -> anyhow::Result<()> {
    if std::env::var(INLINE_AGENT_REAL_PI_FIXTURE_ENV)
        .ok()
        .as_deref()
        != Some("1")
    {
        anyhow::bail!("Inline Agent real Pi fixture requires {INLINE_AGENT_REAL_PI_FIXTURE_ENV}=1");
    }
    open_inline_agent_fixture_with_executor_mode(
        cx,
        text,
        instruction,
        InlineAgentExecutorMode::AgentChatPi,
    )
}

fn open_inline_agent_fixture_with_executor_mode(
    cx: &mut App,
    text: Option<String>,
    instruction: Option<String>,
    executor_mode: InlineAgentExecutorMode,
) -> anyhow::Result<()> {
    close_inline_agent_overlay_window(cx);
    let focused_snapshot =
        crate::platform::accessibility::focused_text::focused_text_snapshot_for_tests(
            text.unwrap_or_else(|| "Hello world".to_string()),
        );
    let snapshot: InlineAgentSnapshot = focused_snapshot.clone().into();
    let plan = plan_open_inline_agent_overlay(&snapshot, InlineOverlayAttachment::Standalone);
    sync_inline_agent_overlay_window_with_executor_mode(
        cx,
        focused_snapshot,
        plan,
        None,
        executor_mode,
    )?;

    let Some(instruction) = instruction
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    else {
        return Ok(());
    };
    if let Some(storage) = INLINE_AGENT_OVERLAY_WINDOW.get() {
        if let Ok(guard) = storage.lock() {
            if let Some(handle) = guard.as_ref() {
                let _ = handle.update(cx, |overlay, window, cx| {
                    overlay.submit_inline_agent_turn(
                        instruction,
                        InlineAgentEditSemantics::Replace,
                        true,
                        window,
                        cx,
                    );
                });
            }
        }
    }

    Ok(())
}

pub fn close_inline_agent_overlay_window(cx: &mut App) {
    remove_inline_agent_automation_window();

    if let Some(storage) = INLINE_AGENT_OVERLAY_WINDOW.get() {
        if let Ok(mut guard) = storage.lock() {
            if let Some(handle) = guard.take() {
                let _ = handle.update(cx, |overlay, window, _cx| {
                    overlay.cancel_active_turn_for_lifecycle("close");
                    window.remove_window();
                });
            }
        }
    }
}

pub fn is_inline_agent_overlay_window_open() -> bool {
    INLINE_AGENT_OVERLAY_WINDOW
        .get()
        .and_then(|storage| storage.lock().ok())
        .is_some_and(|guard| guard.is_some())
}

pub fn launch_inline_agent_from_focused_text(cx: &mut App) -> anyhow::Result<()> {
    if is_inline_agent_overlay_window_open() {
        tracing::info!(
            target: "script_kit::inline_agent",
            event = "inline_agent_launch_reset_existing_overlay",
            "Inline agent launch is resetting the existing overlay before recapture"
        );
        close_inline_agent_overlay_window(cx);
    }

    tracing::info!(
        target: "script_kit::inline_agent",
        event = "inline_agent_capture_start",
        "Inline agent focused-text capture starting"
    );

    let focused_snapshot = capture_focused_text_field(CaptureFocusedTextOptions::default())?;
    let snapshot: InlineAgentSnapshot = focused_snapshot.clone().into();

    tracing::info!(
        target: "script_kit::inline_agent",
        event = "inline_agent_capture_complete_before_overlay",
        session_id = %snapshot.session_id,
        app_name = %snapshot.app.name,
        app_bundle_id = ?snapshot.app.bundle_id,
        app_process_id = ?snapshot.app.process_id,
        chars = snapshot.metrics.chars,
        utf16_units = snapshot.metrics.utf16_units,
        "Inline agent focused-text capture completed before overlay sync"
    );

    let plan = plan_open_inline_agent_overlay(&snapshot, InlineOverlayAttachment::Standalone);
    tracing::info!(
        target: "script_kit::inline_agent",
        event = "inline_agent_overlay_sync_start",
        session_id = %snapshot.session_id,
        "Inline agent overlay sync starting"
    );
    sync_inline_agent_overlay_window(cx, focused_snapshot, plan, None)
}

pub struct InlineAgentOverlayWindow {
    snapshot: InlineAgentSnapshot,
    ai_session: InlineAgentSession,
    plan: InlineAgentWindowSnapshot,
    instruction_text: String,
    submit_counter: u64,
    stream_generation: u64,
    stream_task: Option<Task<()>>,
    active_executor: Option<Box<dyn InlineAgentExecutor>>,
    executor_mode: InlineAgentExecutorMode,
    focus_handle: FocusHandle,
}

impl InlineAgentOverlayWindow {
    fn new(
        focused_snapshot: FocusedTextSnapshot,
        plan: InlineAgentWindowSnapshot,
        executor_mode: InlineAgentExecutorMode,
        cx: &mut Context<Self>,
    ) -> Self {
        let snapshot: InlineAgentSnapshot = focused_snapshot.clone().into();
        Self {
            snapshot,
            ai_session: InlineAgentSession::new(focused_snapshot),
            plan,
            instruction_text: String::new(),
            submit_counter: 0,
            stream_generation: 0,
            stream_task: None,
            active_executor: None,
            executor_mode,
            focus_handle: cx.focus_handle(),
        }
    }

    fn set_snapshot(
        &mut self,
        focused_snapshot: FocusedTextSnapshot,
        plan: InlineAgentWindowSnapshot,
        executor_mode: InlineAgentExecutorMode,
    ) {
        self.cancel_active_turn_for_lifecycle("reset_snapshot");
        self.snapshot = focused_snapshot.clone().into();
        self.ai_session = InlineAgentSession::new(focused_snapshot);
        self.plan = plan;
        self.executor_mode = executor_mode;
        self.instruction_text.clear();
    }

    fn render_compact(
        &self,
        colors: InlineAgentColors,
        cx: &mut Context<Self>,
    ) -> gpui::AnyElement {
        let mut view = compact_view_model(&self.snapshot, &self.plan.run_state);
        view.instruction_text = self.instruction_text.clone();
        render_compact_view_model(view, colors, cx)
    }

    fn render_expanded(
        &self,
        colors: InlineAgentColors,
        cx: &mut Context<Self>,
    ) -> gpui::AnyElement {
        let mut view = expanded_view_model(
            &self.snapshot,
            &self.ai_session.history,
            &self.plan.run_state,
        );
        view.instruction_text = self.instruction_text.clone();
        render_expanded_view_model(view, colors, cx)
    }

    fn handle_output_action(
        &mut self,
        action: InlineAgentOutputAction,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if action == InlineAgentOutputAction::Chat {
            self.plan = plan_expanded_inline_agent_overlay(&self.snapshot, &self.plan);
            self.apply_current_plan_bounds(window, cx);
            cx.notify();
            return;
        }

        let previous_run_state = self.plan.run_state.clone();
        let latest_output = previous_run_state.latest_output_owned().or_else(|| {
            self.ai_session
                .latest_complete_output()
                .map(ToOwned::to_owned)
        });
        let action_run_state =
            action_run_state_with_latest_output(&previous_run_state, latest_output.as_deref());
        self.plan.run_state = InlineAgentRunState::Applying {
            action,
            latest_output: latest_output.clone(),
        };
        cx.notify();

        match apply_latest_output_action(
            &SystemInlineAgentPlatformBridge,
            action,
            &action_run_state,
            &self.snapshot,
        ) {
            Ok(Some(receipt)) if receipt.success => {
                if let Some(output) = latest_output {
                    self.plan.run_state = InlineAgentRunState::Applied {
                        action: receipt.action,
                        output,
                        receipt,
                    };
                } else {
                    self.plan.run_state = previous_run_state;
                }
            }
            Ok(Some(receipt)) => {
                self.plan.run_state = InlineAgentRunState::Error {
                    message: receipt
                        .message
                        .unwrap_or_else(|| "Inline agent action failed".to_string()),
                    retryable: true,
                    latest_output,
                };
            }
            Ok(None) => {
                self.plan.run_state = previous_run_state;
            }
            Err(error) => {
                self.plan.run_state = InlineAgentRunState::Error {
                    message: error.to_string(),
                    retryable: true,
                    latest_output,
                };
            }
        }

        self.apply_current_plan_bounds(window, cx);
        cx.notify();
    }

    fn collapse_expanded(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.plan = plan_compact_inline_agent_overlay(&self.snapshot, &self.plan);
        self.apply_current_plan_bounds(window, cx);
        cx.notify();
    }

    fn apply_current_plan_bounds(&self, window: &mut Window, cx: &mut Context<Self>) {
        let bounds = rect_to_gpui_bounds(self.plan.bounds);
        crate::components::inline_popup_window::set_inline_popup_window_bounds(window, bounds, cx);
        update_inline_agent_automation_bounds(&self.plan);
    }

    fn handle_key_down(
        &mut self,
        event: &gpui::KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let key = event.keystroke.key.as_str();
        let has_cmd = event.keystroke.modifiers.platform;

        if key.eq_ignore_ascii_case("escape") && !has_cmd {
            if self.plan.mode == InlineAgentMode::Expanded {
                self.collapse_expanded(window, cx);
            } else {
                self.close_from_overlay_window(window);
            }
            cx.notify();
            cx.stop_propagation();
            return;
        }

        if crate::ui_foundation::is_key_enter(key) && !has_cmd {
            self.submit_instruction(window, cx);
            cx.stop_propagation();
            return;
        }

        if key.eq_ignore_ascii_case("backspace") && !has_cmd {
            self.instruction_text.pop();
            cx.notify();
            cx.stop_propagation();
            return;
        }

        if has_cmd {
            cx.propagate();
            return;
        }

        if let Some(key_char) = &event.keystroke.key_char {
            if let Some(ch) = key_char.chars().next() {
                if !ch.is_control() {
                    self.instruction_text.push(ch);
                    cx.notify();
                    cx.stop_propagation();
                    return;
                }
            }
        }

        cx.propagate();
    }

    fn close_from_overlay_window(&mut self, window: &mut Window) {
        self.cancel_active_turn_for_lifecycle("escape_close");
        remove_inline_agent_automation_window();
        clear_inline_agent_overlay_storage();
        window.remove_window();
    }

    fn cancel_active_turn_for_lifecycle(&mut self, reason: &'static str) {
        self.stream_generation = self.stream_generation.wrapping_add(1);
        self.stream_task = None;
        let Some(executor) = self.active_executor.take() else {
            return;
        };

        match self.ai_session.cancel_active_turn(executor.as_ref()) {
            Ok(()) => {
                tracing::info!(
                    target: "script_kit::inline_agent",
                    event = "inline_agent_turn_cancelled_for_lifecycle",
                    reason,
                    session_id = %self.snapshot.session_id,
                );
            }
            Err(error) => {
                tracing::warn!(
                    target: "script_kit::inline_agent",
                    event = "inline_agent_turn_cancel_lifecycle_failed",
                    reason,
                    session_id = %self.snapshot.session_id,
                    error = %error,
                );
            }
        }
    }

    fn submit_instruction(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let instruction = self.instruction_text.trim().to_string();
        let semantics = self.submit_semantics_for_mode();
        self.submit_inline_agent_turn(instruction, semantics, true, window, cx);
    }

    fn retry_last_turn(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(request) = self.ai_session.last_retry_request().cloned() else {
            return;
        };
        self.submit_inline_agent_turn(request.instruction, request.semantics, false, window, cx);
    }

    fn submit_inline_agent_turn(
        &mut self,
        instruction: String,
        semantics: InlineAgentEditSemantics,
        clear_instruction_text: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if instruction.is_empty() || self.is_turn_active() {
            return;
        }

        self.submit_counter = self.submit_counter.saturating_add(1);
        self.stream_generation = self.stream_generation.wrapping_add(1);
        let request_id = format!("inline-agent-submit-{}", self.submit_counter);
        self.plan.run_state = InlineAgentRunState::Thinking {
            request_id: request_id.clone(),
            started_at_ms: current_time_ms(),
        };
        cx.notify();

        let executor: Box<dyn InlineAgentExecutor> = match self.spawn_executor_for_turn() {
            Ok(executor) => executor,
            Err(error) => {
                self.plan.run_state = InlineAgentRunState::Error {
                    message: error,
                    retryable: true,
                    latest_output: self
                        .ai_session
                        .latest_complete_output()
                        .map(ToOwned::to_owned),
                };
                if clear_instruction_text {
                    self.instruction_text.clear();
                }
                self.apply_current_plan_bounds(window, cx);
                cx.notify();
                return;
            }
        };

        self.active_executor = Some(executor);
        let Some(executor) = self.active_executor.as_deref() else {
            self.plan.run_state = InlineAgentRunState::Error {
                message: "Inline agent executor was not installed".to_string(),
                retryable: true,
                latest_output: self
                    .ai_session
                    .latest_complete_output()
                    .map(ToOwned::to_owned),
            };
            if clear_instruction_text {
                self.instruction_text.clear();
            }
            self.apply_current_plan_bounds(window, cx);
            cx.notify();
            return;
        };
        match self.ai_session.begin_turn(instruction, semantics, executor) {
            Ok((events, _audit)) => {
                if clear_instruction_text {
                    self.instruction_text.clear();
                }
                self.bind_provider_stream(events, request_id, window, cx);
            }
            Err(error) => {
                self.plan.run_state = InlineAgentRunState::Error {
                    message: error.to_string(),
                    retryable: true,
                    latest_output: self
                        .ai_session
                        .latest_complete_output()
                        .map(ToOwned::to_owned),
                };
                self.active_executor = None;
                if clear_instruction_text {
                    self.instruction_text.clear();
                }
            }
        }
        self.apply_current_plan_bounds(window, cx);
        cx.notify();
    }

    fn spawn_executor_for_turn(&self) -> Result<Box<dyn InlineAgentExecutor>, String> {
        match self.executor_mode {
            InlineAgentExecutorMode::AgentChatPi => {
                spawn_default_agent_chat_inline_agent_executor()
                    .map(|executor| Box::new(executor) as Box<dyn InlineAgentExecutor>)
            }
            InlineAgentExecutorMode::MockFixture => Ok(Box::new(
                crate::ai::inline_agent::mock::MockInlineAgentExecutor,
            )),
        }
    }

    fn stop_active_turn_from_user(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.cancel_active_turn_for_lifecycle("stop_button");
        self.sync_run_state_from_ai_session("inline-agent-stopped".to_string());
        self.apply_current_plan_bounds(window, cx);
        cx.notify();
    }

    fn is_turn_active(&self) -> bool {
        matches!(
            self.plan.run_state,
            InlineAgentRunState::Thinking { .. } | InlineAgentRunState::Streaming { .. }
        )
    }

    fn submit_semantics_for_mode(&self) -> InlineAgentEditSemantics {
        match self.plan.mode {
            InlineAgentMode::Compact => InlineAgentEditSemantics::Replace,
            InlineAgentMode::Expanded => InlineAgentEditSemantics::Chat,
        }
    }

    fn bind_provider_stream(
        &mut self,
        rx: async_channel::Receiver<InlineAgentProviderEvent>,
        request_id: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let generation = self.stream_generation;
        self.stream_task = Some(cx.spawn_in(window, async move |this, cx| {
            while let Ok(event) = rx.recv().await {
                let terminal = matches!(
                    event,
                    InlineAgentProviderEvent::TurnFinished
                        | InlineAgentProviderEvent::Failed { .. }
                );
                let request_id = request_id.clone();
                if this
                    .update_in(cx, move |overlay, window, cx| {
                        if overlay.stream_generation != generation {
                            tracing::debug!(
                                target: "script_kit::inline_agent",
                                event = "inline_agent_stream_event_discarded_stale_generation",
                                expected_generation = generation,
                                actual_generation = overlay.stream_generation,
                            );
                            return;
                        }
                        overlay.ai_session.apply_provider_event(event);
                        overlay.sync_run_state_from_ai_session(request_id);
                        overlay.apply_current_plan_bounds(window, cx);
                        if terminal {
                            overlay.active_executor = None;
                        }
                        cx.notify();
                    })
                    .is_err()
                {
                    break;
                }

                if terminal {
                    break;
                }
            }
        }));
    }

    fn sync_run_state_from_ai_session(&mut self, request_id: String) {
        self.plan.run_state = match self.ai_session.stream.phase {
            InlineAgentPhase::Ready | InlineAgentPhase::Capturing => InlineAgentRunState::Idle,
            InlineAgentPhase::Thinking => InlineAgentRunState::Thinking {
                request_id,
                started_at_ms: current_time_ms(),
            },
            InlineAgentPhase::Streaming => InlineAgentRunState::Streaming {
                request_id,
                partial_output: self.ai_session.stream.visible_output.clone(),
            },
            InlineAgentPhase::Cancelling => InlineAgentRunState::Idle,
            InlineAgentPhase::Complete => InlineAgentRunState::Completed {
                output: self
                    .ai_session
                    .latest_complete_output()
                    .unwrap_or_default()
                    .to_string(),
            },
            InlineAgentPhase::Error => InlineAgentRunState::Error {
                message: self
                    .ai_session
                    .stream
                    .error
                    .clone()
                    .unwrap_or_else(|| "Inline agent turn failed".to_string()),
                retryable: true,
                latest_output: self
                    .ai_session
                    .latest_complete_output()
                    .map(ToOwned::to_owned),
            },
        };
    }
}

impl Render for InlineAgentOverlayWindow {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.plan.focus_prompt && !self.focus_handle.is_focused(window) {
            window.focus(&self.focus_handle, cx);
        }

        let theme = crate::theme::get_cached_theme();
        let colors = InlineAgentColors::from_theme(&theme);

        div()
            .id(match self.plan.mode {
                InlineAgentMode::Compact => super::automation::INLINE_AGENT_COMPACT_ID,
                InlineAgentMode::Expanded => super::automation::INLINE_AGENT_EXPANDED_ID,
            })
            .track_focus(&self.focus_handle)
            .w_full()
            .h_full()
            .p(px(12.0))
            .flex()
            .flex_col()
            .gap(px(8.0))
            .bg(rgb(colors.surface))
            .border_1()
            .border_color(rgb(colors.border))
            .text_color(rgb(colors.text_primary))
            .on_key_down(cx.listener(|this, event: &gpui::KeyDownEvent, window, cx| {
                this.handle_key_down(event, window, cx);
            }))
            .child(match self.plan.mode {
                InlineAgentMode::Compact => self.render_compact(colors, cx),
                InlineAgentMode::Expanded => self.render_expanded(colors, cx),
            })
    }
}

fn render_compact_view_model(
    view: InlineAgentCompactViewModel,
    colors: InlineAgentColors,
    cx: &mut Context<InlineAgentOverlayWindow>,
) -> gpui::AnyElement {
    let mut action_strip = div().flex().flex_row().gap(px(6.0));
    for action in view.actions {
        let id = match action.action {
            super::types::InlineAgentOutputAction::Replace => {
                super::automation::INLINE_AGENT_ACTION_REPLACE_ID
            }
            super::types::InlineAgentOutputAction::Append => {
                super::automation::INLINE_AGENT_ACTION_APPEND_ID
            }
            super::types::InlineAgentOutputAction::Copy => {
                super::automation::INLINE_AGENT_ACTION_COPY_ID
            }
            super::types::InlineAgentOutputAction::Chat => {
                super::automation::INLINE_AGENT_ACTION_CHAT_ID
            }
        };
        let mut action_button = div()
            .id(id)
            .px(px(8.0))
            .py(px(4.0))
            .rounded(px(4.0))
            .bg(rgb(colors.accent))
            .text_color(rgb(if action.enabled {
                colors.accent_text
            } else {
                colors.text_disabled
            }))
            .child(format!("{:?}", action.action));
        if action.enabled {
            let output_action = action.action;
            action_button = action_button.cursor_pointer().on_click(cx.listener(
                move |this, _event, window, cx| {
                    this.handle_output_action(output_action, window, cx);
                },
            ));
        }
        action_strip = action_strip.child(action_button);
    }

    div()
        .flex()
        .flex_col()
        .gap(px(8.0))
        .child(
            div()
                .id(super::automation::INLINE_AGENT_HEADER_ID)
                .flex()
                .justify_between()
                .child(
                    div()
                        .id(super::automation::INLINE_AGENT_APP_BADGE_ID)
                        .child(view.app_badge),
                )
                .child(
                    div()
                        .id(super::automation::INLINE_AGENT_METRICS_ID)
                        .text_color(rgb(colors.text_secondary))
                        .child(view.metrics_label),
                ),
        )
        .child(
            div()
                .id(super::automation::INLINE_AGENT_INPUT_ID)
                .text_color(rgb(if view.instruction_text.is_empty() {
                    colors.text_secondary
                } else {
                    colors.text_primary
                }))
                .child(if view.instruction_text.is_empty() {
                    view.input_placeholder.to_string()
                } else {
                    view.instruction_text
                }),
        )
        .when(view.thinking_visible, |node| {
            node.child(
                div()
                    .id(super::automation::INLINE_AGENT_THINKING_BAR_ID)
                    .text_color(rgb(colors.thinking))
                    .child(
                        div()
                            .id(super::automation::INLINE_AGENT_THINKING_LABEL_ID)
                            .child(view.thinking_label.unwrap_or_default()),
                    ),
            )
        })
        .when_some(view.output_preview, |node, output| {
            node.child(
                div()
                    .id(super::automation::INLINE_AGENT_OUTPUT_PREVIEW_ID)
                    .text_color(rgb(colors.text_primary))
                    .child(output),
            )
        })
        .when(view.stop_enabled, |node| {
            node.child(
                div()
                    .id(super::automation::INLINE_AGENT_ACTION_STOP_ID)
                    .px(px(8.0))
                    .py(px(4.0))
                    .rounded(px(4.0))
                    .bg(rgb(colors.border))
                    .text_color(rgb(colors.text_primary))
                    .cursor_pointer()
                    .on_click(cx.listener(|this, _event, window, cx| {
                        this.stop_active_turn_from_user(window, cx);
                    }))
                    .child("Stop"),
            )
        })
        .when(view.retry_enabled, |node| {
            node.child(
                div()
                    .id(super::automation::INLINE_AGENT_ACTION_RETRY_ID)
                    .px(px(8.0))
                    .py(px(4.0))
                    .rounded(px(4.0))
                    .bg(rgb(colors.accent))
                    .text_color(rgb(colors.accent_text))
                    .cursor_pointer()
                    .on_click(cx.listener(|this, _event, window, cx| {
                        this.retry_last_turn(window, cx);
                    }))
                    .child("Retry"),
            )
        })
        .child(action_strip)
        .into_any_element()
}

fn render_expanded_view_model(
    view: InlineAgentExpandedViewModel,
    colors: InlineAgentColors,
    cx: &mut Context<InlineAgentOverlayWindow>,
) -> gpui::AnyElement {
    let mut action_strip = div().flex().flex_row().gap(px(6.0));
    for action in view.actions {
        let id = match action.action {
            super::types::InlineAgentOutputAction::Replace => {
                super::automation::INLINE_AGENT_ACTION_REPLACE_ID
            }
            super::types::InlineAgentOutputAction::Append => {
                super::automation::INLINE_AGENT_ACTION_APPEND_ID
            }
            super::types::InlineAgentOutputAction::Copy => {
                super::automation::INLINE_AGENT_ACTION_COPY_ID
            }
            super::types::InlineAgentOutputAction::Chat => {
                super::automation::INLINE_AGENT_ACTION_CHAT_ID
            }
        };
        let mut action_button = div()
            .id(id)
            .px(px(8.0))
            .py(px(4.0))
            .rounded(px(4.0))
            .bg(rgb(colors.accent))
            .text_color(rgb(if action.enabled {
                colors.accent_text
            } else {
                colors.text_disabled
            }))
            .child(format!("{:?}", action.action));
        if action.enabled {
            let output_action = action.action;
            action_button = action_button.cursor_pointer().on_click(cx.listener(
                move |this, _event, window, cx| {
                    this.handle_output_action(output_action, window, cx);
                },
            ));
        }
        action_strip = action_strip.child(action_button);
    }

    div()
        .flex()
        .flex_col()
        .gap(px(8.0))
        .child(
            div()
                .id(super::automation::INLINE_AGENT_HEADER_ID)
                .flex()
                .justify_between()
                .child(view.header_label)
                .child(
                    div()
                        .id(super::automation::INLINE_AGENT_COLLAPSE_ID)
                        .text_color(rgb(colors.accent))
                        .cursor_pointer()
                        .on_click(cx.listener(|this, _event, window, cx| {
                            this.collapse_expanded(window, cx);
                        }))
                        .child("Collapse"),
                ),
        )
        .child(
            div()
                .id(super::automation::INLINE_AGENT_TURN_LIST_ID)
                .flex()
                .flex_col()
                .gap(px(6.0))
                .children(view.turns.into_iter().map(|turn| {
                    div()
                        .text_color(rgb(colors.text_primary))
                        .child(turn.user_instruction)
                        .when_some(turn.assistant_output, |node, output| {
                            node.child(div().text_color(rgb(colors.text_secondary)).child(output))
                        })
                })),
        )
        .child(action_strip)
        .when(view.stop_enabled, |node| {
            node.child(
                div()
                    .id(super::automation::INLINE_AGENT_ACTION_STOP_ID)
                    .px(px(8.0))
                    .py(px(4.0))
                    .rounded(px(4.0))
                    .bg(rgb(colors.border))
                    .text_color(rgb(colors.text_primary))
                    .cursor_pointer()
                    .on_click(cx.listener(|this, _event, window, cx| {
                        this.stop_active_turn_from_user(window, cx);
                    }))
                    .child("Stop"),
            )
        })
        .when(view.retry_enabled, |node| {
            node.child(
                div()
                    .id(super::automation::INLINE_AGENT_ACTION_RETRY_ID)
                    .px(px(8.0))
                    .py(px(4.0))
                    .rounded(px(4.0))
                    .bg(rgb(colors.accent))
                    .text_color(rgb(colors.accent_text))
                    .cursor_pointer()
                    .on_click(cx.listener(|this, _event, window, cx| {
                        this.retry_last_turn(window, cx);
                    }))
                    .child("Retry"),
            )
        })
        .child(
            div()
                .id(super::automation::INLINE_AGENT_EXPANDED_COMPOSER_ID)
                .text_color(rgb(if view.instruction_text.is_empty() {
                    colors.text_secondary
                } else {
                    colors.text_primary
                }))
                .child(if view.instruction_text.is_empty() {
                    view.input_placeholder.to_string()
                } else {
                    view.instruction_text
                }),
        )
        .into_any_element()
}

fn configure_inline_agent_overlay_window(
    handle: &WindowHandle<InlineAgentOverlayWindow>,
    cx: &mut App,
) -> anyhow::Result<()> {
    #[cfg(target_os = "macos")]
    {
        let is_dark_vibrancy = crate::theme::get_cached_theme().should_use_dark_vibrancy();
        handle
            .update(cx, move |_overlay, window, cx| {
                window.defer(cx, move |window, _cx| {
                    if let Some(ns_window) =
                        crate::components::inline_popup_window::inline_popup_ns_window(window)
                    {
                        unsafe {
                            crate::platform::configure_actions_popup_window(
                                ns_window,
                                is_dark_vibrancy,
                            );
                        }
                    }
                });
            })
            .map_err(|_| anyhow::anyhow!("failed to configure inline agent overlay window"))?;
    }

    #[cfg(not(target_os = "macos"))]
    let _ = (handle, cx);

    Ok(())
}

fn compact_bounds_for_run_state(
    snapshot: &InlineAgentSnapshot,
    run_state: &InlineAgentRunState,
) -> RectPx {
    let defaults = InlineAgentLayoutDefaults::default();
    let height = compact_height_for_run_state(run_state, defaults);
    let anchor = preferred_anchor_geometry(&snapshot.anchor.geometry);
    place_compact_overlay(anchor, snapshot.anchor.geometry.display_bounds, height)
}

fn compact_height_for_run_state(
    run_state: &InlineAgentRunState,
    defaults: InlineAgentLayoutDefaults,
) -> f64 {
    match run_state {
        InlineAgentRunState::Thinking { .. } | InlineAgentRunState::Streaming { .. } => {
            defaults.compact_thinking_height
        }
        InlineAgentRunState::Completed { .. }
        | InlineAgentRunState::Applied { .. }
        | InlineAgentRunState::Applying {
            latest_output: Some(_),
            ..
        }
        | InlineAgentRunState::Error {
            latest_output: Some(_),
            ..
        } => defaults.compact_completed_height,
        InlineAgentRunState::Error { .. }
        | InlineAgentRunState::Idle
        | InlineAgentRunState::Applying { .. } => defaults.compact_idle_height,
    }
}

fn current_time_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or_default()
}

fn action_run_state_with_latest_output(
    state: &InlineAgentRunState,
    latest_output: Option<&str>,
) -> InlineAgentRunState {
    if state.latest_complete_output().is_some() {
        return state.clone();
    }

    latest_output
        .map(|output| InlineAgentRunState::Completed {
            output: output.to_string(),
        })
        .unwrap_or_else(|| state.clone())
}

fn inline_agent_run_state_phase(state: &InlineAgentRunState) -> &'static str {
    match state {
        InlineAgentRunState::Idle => "idle",
        InlineAgentRunState::Thinking { .. } => "thinking",
        InlineAgentRunState::Streaming { .. } => "streaming",
        InlineAgentRunState::Completed { .. } => "completed",
        InlineAgentRunState::Error { .. } => "error",
        InlineAgentRunState::Applying { .. } => "applying",
        InlineAgentRunState::Applied { .. } => "applied",
    }
}

fn inline_agent_run_state_output_summary(state: &InlineAgentRunState) -> serde_json::Value {
    let latest_complete_chars = state
        .latest_complete_output()
        .map(|output| output.chars().count())
        .unwrap_or(0);
    let streaming_partial_chars = match state {
        InlineAgentRunState::Streaming { partial_output, .. } => partial_output.chars().count(),
        _ => 0,
    };

    serde_json::json!({
        "hasLatestComplete": latest_complete_chars > 0,
        "latestCompleteChars": latest_complete_chars,
        "streamingPartialChars": streaming_partial_chars,
    })
}

fn inline_agent_run_state_action_summary(
    snapshot: &InlineAgentWindowSnapshot,
) -> serde_json::Value {
    let state = &snapshot.run_state;
    let is_active = matches!(
        state,
        InlineAgentRunState::Thinking { .. } | InlineAgentRunState::Streaming { .. }
    );
    let has_output = state.latest_complete_output().is_some();
    let retryable = matches!(
        state,
        InlineAgentRunState::Error {
            retryable: true,
            ..
        }
    );
    serde_json::json!({
        "replaceEnabled": has_output && !is_active && snapshot.can_replace,
        "appendEnabled": has_output && !is_active && snapshot.can_append,
        "copyEnabled": has_output && !is_active && snapshot.can_copy,
        "chatEnabled": snapshot.mode == InlineAgentMode::Compact && !is_active,
        "stopEnabled": is_active,
        "retryEnabled": retryable,
        "collapseEnabled": snapshot.mode == InlineAgentMode::Expanded,
    })
}

fn inline_agent_run_state_mutation_summary(state: &InlineAgentRunState) -> serde_json::Value {
    let Some(receipt) = inline_agent_run_state_mutation_receipt(state) else {
        return serde_json::Value::Null;
    };

    serde_json::json!({
        "schemaVersion": 1,
        "action": inline_agent_output_action_name(receipt.action),
        "success": receipt.success,
        "changedText": receipt.changed_text,
        "copiedToClipboard": receipt.copied_to_clipboard,
        "safeLog": true,
    })
}

fn inline_agent_run_state_mutation_receipt(
    state: &InlineAgentRunState,
) -> Option<&InlineAgentMutationReceipt> {
    match state {
        InlineAgentRunState::Applied { receipt, .. } => Some(receipt),
        _ => None,
    }
}

fn inline_agent_output_action_name(action: InlineAgentOutputAction) -> &'static str {
    match action {
        InlineAgentOutputAction::Replace => "replace",
        InlineAgentOutputAction::Append => "append",
        InlineAgentOutputAction::Copy => "copy",
        InlineAgentOutputAction::Chat => "chat",
    }
}

fn rect_to_gpui_bounds(rect: RectPx) -> Bounds<Pixels> {
    Bounds {
        origin: gpui::point(px(rect.x as f32), px(rect.y as f32)),
        size: gpui::size(px(rect.width as f32), px(rect.height as f32)),
    }
}

fn rect_to_automation_bounds(rect: RectPx) -> AutomationWindowBounds {
    AutomationWindowBounds {
        x: rect.x,
        y: rect.y,
        width: rect.width,
        height: rect.height,
    }
}
