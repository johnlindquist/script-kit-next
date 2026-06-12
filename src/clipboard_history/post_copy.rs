//! Post-copy modifier-tap quick menu + HUD whisper (T12).
//!
//! Wires the tap-window state machine to a CGEventTap, bridges UI events onto
//! the GPUI main loop, and hosts the quick-menu popup near the cursor.

use std::sync::{Arc, LazyLock, Mutex, OnceLock};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::Result;
use gpui::{
    div, px, App, AppContext, Bounds, Context, DisplayId, Entity, FocusHandle, Focusable,
    InteractiveElement, IntoElement, KeyDownEvent, ParentElement, Pixels, Render, SharedString,
    StatefulInteractiveElement, Styled, Window, WindowHandle,
};
use gpui_component::input::{Input, InputEvent, InputState};
use parking_lot::Mutex as ParkingMutex;
use tracing::{debug, info, warn};

use crate::components::inline_dropdown::{
    inline_dropdown_visible_range_from_start, render_soft_compact_picker_row, InlineDropdown,
    InlineDropdownColors, SOFT_COMPACT_PICKER_ROW_HEIGHT,
};
use crate::components::inline_popup_window::{
    inline_popup_height_for_row_height, inline_popup_window_options, INLINE_POPUP_DEFAULT_WIDTH,
    INLINE_POPUP_EDGE_GUTTER, INLINE_POPUP_MAX_VISIBLE_ROWS, INLINE_POPUP_VERTICAL_PADDING,
};
use crate::platform::{
    clamp_to_visible, display_for_point, get_global_mouse_position, get_macos_visible_displays,
};
use crate::theme::get_cached_theme;

use super::sediment::{annotate_clipboard_entry, reject_clipboard_entry};
use super::tap_window::{TapWindowInput, TapWindowMachine, TapWindowOutput};

pub const POST_COPY_MENU_AUTOMATION_ID: &str = "clipboard-post-copy-menu";

/// User-facing post-copy menu configuration (mapped from `clipboardHistoryPostCopyMenu`).
#[derive(Debug, Clone)]
pub struct PostCopyMenuConfig {
    pub enabled: bool,
    pub tap_window_ms: u64,
    pub trigger_modifiers: Vec<String>,
}

impl Default for PostCopyMenuConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            tap_window_ms: 2500,
            trigger_modifiers: vec!["meta".to_string()],
        }
    }
}

/// Runtime configuration for the post-copy menu lane.
#[derive(Debug, Clone)]
struct PostCopyRuntimeConfig {
    enabled: bool,
    tap_window_ms: u64,
    watch_command_modifier: bool,
}

impl From<PostCopyMenuConfig> for PostCopyRuntimeConfig {
    fn from(config: PostCopyMenuConfig) -> Self {
        let watch_command_modifier = config
            .trigger_modifiers
            .iter()
            .any(|modifier: &String| modifier.eq_ignore_ascii_case("meta") || modifier == "cmd");
        Self {
            enabled: config.enabled,
            tap_window_ms: config.tap_window_ms.max(250),
            watch_command_modifier,
        }
    }
}

impl Default for PostCopyRuntimeConfig {
    fn default() -> Self {
        PostCopyRuntimeConfig::from(PostCopyMenuConfig::default())
    }
}

/// Optional HUD whisper hook (registered from the binary crate where `hud_manager` lives).
pub type KeptHudWhisperFn = fn(&mut App);

static KEPT_HUD_WHISPER: OnceLock<KeptHudWhisperFn> = OnceLock::new();

/// Register the quiet "Kept" HUD handler (call from app startup before install).
pub fn register_kept_hud_whisper(handler: KeptHudWhisperFn) {
    let _ = KEPT_HUD_WHISPER.set(handler);
}

static RUNTIME_CONFIG: OnceLock<PostCopyRuntimeConfig> = OnceLock::new();
static DEFAULT_RUNTIME: LazyLock<PostCopyRuntimeConfig> =
    LazyLock::new(PostCopyRuntimeConfig::default);
static PENDING_ENTRY_ID: LazyLock<Mutex<Option<String>>> = LazyLock::new(|| Mutex::new(None));
static TAP_MACHINE: LazyLock<ParkingMutex<TapWindowMachine>> =
    LazyLock::new(|| ParkingMutex::new(TapWindowMachine::new(2500)));
static INSTALLED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

#[derive(Debug, Clone)]
enum PostCopyUiEvent {
    OpenQuickMenu { entry_id: String },
    ShowKeptHud,
}

static POST_COPY_UI_CHANNEL: LazyLock<(
    async_channel::Sender<PostCopyUiEvent>,
    async_channel::Receiver<PostCopyUiEvent>,
)> = LazyLock::new(|| async_channel::bounded(32));

/// Load post-copy menu settings (call before clipboard monitor starts).
pub fn configure_post_copy_menu(config: PostCopyMenuConfig) {
    let runtime = PostCopyRuntimeConfig::from(config);
    *TAP_MACHINE.lock() = TapWindowMachine::new(runtime.tap_window_ms);
    let _ = RUNTIME_CONFIG.set(runtime);
}

fn runtime_config() -> &'static PostCopyRuntimeConfig {
    RUNTIME_CONFIG.get().unwrap_or(&DEFAULT_RUNTIME)
}

/// Notify the tap window that a keepable text entry was stored.
pub fn notify_text_copy_stored(entry_id: &str) {
    let config = runtime_config();
    if !config.enabled || !config.watch_command_modifier {
        return;
    }

    if let Ok(mut pending) = PENDING_ENTRY_ID.lock() {
        *pending = Some(entry_id.to_string());
    }

    let at = Instant::now();
    let mut outputs = Vec::new();
    {
        let mut machine = TAP_MACHINE.lock();
        outputs.extend(machine.apply(TapWindowInput::CopyStored { at }));
        if command_modifier_is_down() {
            outputs.extend(machine.apply(TapWindowInput::ModifierDown { at }));
        }
    }
    dispatch_tap_outputs(outputs);
}

/// Queue a quiet HUD whisper for an auto-keep (ADR 0004).
pub fn request_kept_hud_whisper() {
    let _ = POST_COPY_UI_CHANNEL
        .0
        .try_send(PostCopyUiEvent::ShowKeptHud);
}

fn dispatch_tap_outputs(outputs: Vec<TapWindowOutput>) {
    for output in outputs {
        match output {
            TapWindowOutput::OpenMenu => {
                if let Ok(pending) = PENDING_ENTRY_ID.lock() {
                    if let Some(entry_id) = pending.clone() {
                        let _ = POST_COPY_UI_CHANNEL
                            .0
                            .try_send(PostCopyUiEvent::OpenQuickMenu { entry_id });
                    }
                }
            }
            TapWindowOutput::Cancelled => {
                if let Ok(mut pending) = PENDING_ENTRY_ID.lock() {
                    *pending = None;
                }
                TAP_MACHINE.lock().reset();
            }
        }
    }
}

fn command_modifier_is_down() -> bool {
    #[cfg(target_os = "macos")]
    {
        use objc::{class, msg_send, sel, sel_impl};

        const NS_COMMAND_KEY_MASK: u64 = 1 << 20;
        // SAFETY: NSEvent modifierFlags is a class property safe on any thread.
        unsafe {
            let flags: u64 = msg_send![class!(NSEvent), modifierFlags];
            flags & NS_COMMAND_KEY_MASK != 0
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        false
    }
}

/// Install the CGEventTap bridge and GPUI quick-menu host. Idempotent.
pub fn install_post_copy_quick_menu(cx: &mut App) -> Result<()> {
    if INSTALLED.swap(true, std::sync::atomic::Ordering::SeqCst) {
        return Ok(());
    }

    install_event_tap_thread();
    start_timeout_ticker();

    let rx = POST_COPY_UI_CHANNEL.1.clone();
    cx.spawn(async move |cx: &mut gpui::AsyncApp| {
        while let Ok(event) = rx.recv().await {
            cx.update(|cx| match event {
                PostCopyUiEvent::OpenQuickMenu { entry_id } => {
                    if let Err(error) = open_post_copy_quick_menu(&entry_id, cx) {
                        warn!(entry_id = %entry_id, error = %error, "post-copy quick menu failed");
                    }
                    TAP_MACHINE.lock().reset();
                    if let Ok(mut pending) = PENDING_ENTRY_ID.lock() {
                        *pending = None;
                    }
                }
                PostCopyUiEvent::ShowKeptHud => {
                    if let Some(show) = KEPT_HUD_WHISPER.get() {
                        show(cx);
                    }
                }
            });
        }
    })
    .detach();

    info!("post-copy quick menu installed");
    Ok(())
}

fn start_timeout_ticker() {
    thread::Builder::new()
        .name("clipboard-post-copy-ticker".to_string())
        .spawn(|| loop {
            thread::sleep(Duration::from_millis(200));
            let outputs = TAP_MACHINE
                .lock()
                .apply(TapWindowInput::Tick { at: Instant::now() });
            dispatch_tap_outputs(outputs);
        })
        .ok();
}

#[cfg(target_os = "macos")]
struct SendableMachPortRef(Option<core_foundation::mach_port::CFMachPortRef>);

#[cfg(target_os = "macos")]
unsafe impl Send for SendableMachPortRef {}

#[cfg(target_os = "macos")]
unsafe impl Sync for SendableMachPortRef {}

#[cfg(target_os = "macos")]
fn install_event_tap_thread() {
    use core_foundation::base::TCFType;
    use core_foundation::runloop::{kCFRunLoopCommonModes, CFRunLoop};
    use core_graphics::event::{
        CGEvent, CGEventFlags, CGEventTap, CGEventTapLocation, CGEventTapOptions,
        CGEventTapPlacement, CGEventType,
    };

    thread::Builder::new()
        .name("clipboard-post-copy-tap".to_string())
        .spawn(|| {
            let current_run_loop = CFRunLoop::get_current();
            let mach_port_ref = Arc::new(std::sync::Mutex::new(SendableMachPortRef(None)));
            let mach_port_for_callback = Arc::clone(&mach_port_ref);

            let event_tap = match CGEventTap::new(
                CGEventTapLocation::HID,
                CGEventTapPlacement::HeadInsertEventTap,
                CGEventTapOptions::ListenOnly,
                vec![CGEventType::FlagsChanged, CGEventType::KeyDown],
                move |_proxy, event_type, event: &CGEvent| {
                    match event_type {
                        CGEventType::TapDisabledByTimeout | CGEventType::TapDisabledByUserInput => {
                            reenable_tap(&mach_port_for_callback);
                            return None;
                        }
                        _ => {}
                    }

                    let config = runtime_config();
                    if !config.enabled || !config.watch_command_modifier {
                        return None;
                    }

                    let at = Instant::now();
                    let flags = event.get_flags();
                    let command_down = flags.contains(CGEventFlags::CGEventFlagCommand);

                    let input = match event_type {
                        CGEventType::FlagsChanged if command_down => {
                            Some(TapWindowInput::ModifierDown { at })
                        }
                        CGEventType::FlagsChanged => Some(TapWindowInput::ModifierUp { at }),
                        CGEventType::KeyDown => Some(TapWindowInput::KeyDown { at }),
                        _ => None,
                    };

                    if let Some(input) = input {
                        let outputs = TAP_MACHINE.lock().apply(input);
                        dispatch_tap_outputs(outputs);
                    }

                    None
                },
            ) {
                Ok(tap) => tap,
                Err(()) => {
                    warn!("post-copy event tap creation failed (accessibility?)");
                    return;
                }
            };

            if let Ok(mut guard) = mach_port_ref.lock() {
                guard.0 = Some(event_tap.mach_port.as_concrete_TypeRef());
            }

            let run_loop_source = match event_tap.mach_port.create_runloop_source(0) {
                Ok(source) => source,
                Err(()) => {
                    warn!("post-copy event tap run-loop source failed");
                    return;
                }
            };

            unsafe {
                current_run_loop.add_source(&run_loop_source, kCFRunLoopCommonModes);
            }
            event_tap.enable();

            loop {
                unsafe {
                    core_foundation::runloop::CFRunLoop::run_in_mode(
                        core_foundation::runloop::kCFRunLoopDefaultMode,
                        Duration::from_millis(250),
                        true,
                    );
                }
            }
        })
        .ok();
}

#[cfg(not(target_os = "macos"))]
fn install_event_tap_thread() {}

#[cfg(target_os = "macos")]
fn reenable_tap(mach_port_ref: &Arc<std::sync::Mutex<SendableMachPortRef>>) {
    extern "C" {
        fn CGEventTapEnable(tap: core_foundation::mach_port::CFMachPortRef, enable: bool);
    }

    if let Ok(guard) = mach_port_ref.lock() {
        if let Some(port) = guard.0 {
            unsafe {
                CGEventTapEnable(port, true);
            }
        }
    }
}

// =============================================================================
// Quick menu popup (shared inline-popup + Actions row language)
// =============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum QuickMenuAction {
    Annotate,
    Reject,
    Dismiss,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct QuickMenuRow {
    row_id: String,
    semantic_id: String,
    title: String,
    subtitle: String,
    action: QuickMenuAction,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum QuickMenuMode {
    Actions,
    AnnotateInput,
}

struct PostCopyQuickMenuWindow {
    entry_id: String,
    mode: QuickMenuMode,
    rows: Vec<QuickMenuRow>,
    selected_row_id: Option<String>,
    visible_start: usize,
    why_input: Entity<InputState>,
    focus_handle: FocusHandle,
}

impl PostCopyQuickMenuWindow {
    fn new(entry_id: String, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let rows = vec![
            QuickMenuRow {
                row_id: "annotate".to_string(),
                semantic_id: "clipboard-post-copy:annotate".to_string(),
                title: "Annotate".to_string(),
                subtitle: "Say why you copied this".to_string(),
                action: QuickMenuAction::Annotate,
            },
            QuickMenuRow {
                row_id: "reject".to_string(),
                semantic_id: "clipboard-post-copy:reject".to_string(),
                title: "Reject".to_string(),
                subtitle: "Delete and undo any auto-keep".to_string(),
                action: QuickMenuAction::Reject,
            },
            QuickMenuRow {
                row_id: "dismiss".to_string(),
                semantic_id: "clipboard-post-copy:dismiss".to_string(),
                title: "Dismiss".to_string(),
                subtitle: "Keep without changes".to_string(),
                action: QuickMenuAction::Dismiss,
            },
        ];
        let why_input =
            cx.new(|cx| InputState::new(window, cx).placeholder("Why did you copy this?"));
        cx.subscribe_in(
            &why_input,
            window,
            |this, _input, event: &InputEvent, window, cx| {
                if matches!(event, InputEvent::PressEnter { .. }) {
                    this.submit_annotation(window, cx);
                }
            },
        )
        .detach();

        Self {
            entry_id,
            mode: QuickMenuMode::Actions,
            selected_row_id: Some("annotate".to_string()),
            visible_start: 0,
            rows,
            why_input,
            focus_handle: cx.focus_handle(),
        }
    }

    fn selected_index(&self) -> Option<usize> {
        let selected = self.selected_row_id.as_deref()?;
        self.rows.iter().position(|row| row.row_id == selected)
    }

    fn visible_range(&self) -> std::ops::Range<usize> {
        let row_count = self.rows.len();
        if row_count == 0 {
            return 0..0;
        }
        let selected = self
            .selected_index()
            .unwrap_or_else(|| self.visible_start.min(row_count.saturating_sub(1)));
        inline_dropdown_visible_range_from_start(
            self.visible_start,
            selected,
            row_count,
            row_count.min(INLINE_POPUP_MAX_VISIBLE_ROWS),
        )
    }

    fn select_row(&mut self, index: usize, cx: &mut Context<Self>) {
        if let Some(row) = self.rows.get(index) {
            self.selected_row_id = Some(row.row_id.clone());
            cx.notify();
        }
    }

    fn accept_row(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(row) = self.rows.get(index) else {
            return;
        };
        match row.action {
            QuickMenuAction::Annotate => {
                self.mode = QuickMenuMode::AnnotateInput;
                self.focus_handle.focus(window, cx);
                cx.notify();
            }
            QuickMenuAction::Reject => {
                let entry_id = self.entry_id.clone();
                if let Err(error) = reject_clipboard_entry(&entry_id) {
                    warn!(entry_id = %entry_id, error = %error, "post-copy reject failed");
                }
                close_post_copy_quick_menu(window);
            }
            QuickMenuAction::Dismiss => {
                close_post_copy_quick_menu(window);
            }
        }
    }

    fn submit_annotation(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let why = self.why_input.read(cx).value().to_string();
        let entry_id = self.entry_id.clone();
        if let Err(error) = annotate_clipboard_entry(&entry_id, &why, chrono::Utc::now()) {
            warn!(entry_id = %entry_id, error = %error, "post-copy annotate failed");
        }
        close_post_copy_quick_menu(window);
    }

    fn handle_key_down(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.mode == QuickMenuMode::AnnotateInput {
            if crate::ui_foundation::is_key_escape(event.keystroke.key.as_str()) {
                close_post_copy_quick_menu(window);
                cx.stop_propagation();
            }
            return;
        }

        let row_count = self.rows.len();
        if row_count == 0 {
            cx.propagate();
            return;
        }
        let key = event.keystroke.key.as_str();
        let current = self.selected_index().unwrap_or(0);
        if crate::ui_foundation::is_key_down(key) {
            self.select_row((current + 1) % row_count, cx);
            cx.stop_propagation();
            return;
        }
        if crate::ui_foundation::is_key_up(key) {
            let next = if current == 0 {
                row_count - 1
            } else {
                current - 1
            };
            self.select_row(next, cx);
            cx.stop_propagation();
            return;
        }
        if crate::ui_foundation::is_key_enter(key) {
            self.accept_row(current, window, cx);
            cx.stop_propagation();
            return;
        }
        if crate::ui_foundation::is_key_escape(key) {
            close_post_copy_quick_menu(window);
            cx.stop_propagation();
            return;
        }
        cx.propagate();
    }

    fn render_actions(&self, cx: &mut Context<Self>) -> gpui::AnyElement {
        let theme = get_cached_theme();
        let colors = InlineDropdownColors::popup_from_theme(&theme);
        let visible = self.visible_range();
        let selected_index = self.selected_index();
        let rows: Vec<_> = self
            .rows
            .iter()
            .enumerate()
            .skip(visible.start)
            .take(visible.len())
            .collect();

        let body = div()
            .size_full()
            .flex()
            .flex_col()
            .children(rows.into_iter().map(|(idx, row)| {
                let is_selected = selected_index == Some(idx);
                render_soft_compact_picker_row(
                    SharedString::from(format!("clipboard-post-copy-row-{idx}")),
                    row.title.clone().into(),
                    Some(row.subtitle.clone().into()),
                    &[],
                    &[],
                    is_selected,
                    colors,
                )
                .id(SharedString::from(row.semantic_id.clone()))
                .cursor_pointer()
                .on_click(cx.listener(move |this, _event, window, cx| {
                    this.accept_row(idx, window, cx);
                }))
                .into_any_element()
            }))
            .into_any_element();

        InlineDropdown::new(
            SharedString::from(POST_COPY_MENU_AUTOMATION_ID),
            body,
            colors,
        )
        .vertical_padding(INLINE_POPUP_VERTICAL_PADDING / 2.0)
        .into_any_element()
    }

    fn render_annotate(&self, _cx: &mut Context<Self>) -> gpui::AnyElement {
        let theme = get_cached_theme();
        let colors = InlineDropdownColors::popup_from_theme(&theme);
        let body = div()
            .w_full()
            .px(px(8.0))
            .py(px(6.0))
            .child(
                div()
                    .text_sm()
                    .text_color(colors.muted_foreground)
                    .child("Why did you copy this?"),
            )
            .child(Input::new(&self.why_input).w_full().h(px(28.0)))
            .into_any_element();

        InlineDropdown::new(
            SharedString::from("clipboard-post-copy-annotate"),
            body,
            colors,
        )
        .vertical_padding(INLINE_POPUP_VERTICAL_PADDING / 2.0)
        .into_any_element()
    }
}

impl Focusable for PostCopyQuickMenuWindow {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for PostCopyQuickMenuWindow {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let child = match self.mode {
            QuickMenuMode::Actions => self.render_actions(cx),
            QuickMenuMode::AnnotateInput => self.render_annotate(cx),
        };
        div()
            .size_full()
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(Self::handle_key_down))
            .child(child)
    }
}

static QUICK_MENU_SLOT: OnceLock<Mutex<Option<WindowHandle<PostCopyQuickMenuWindow>>>> =
    OnceLock::new();

fn quick_menu_slot() -> &'static Mutex<Option<WindowHandle<PostCopyQuickMenuWindow>>> {
    QUICK_MENU_SLOT.get_or_init(|| Mutex::new(None))
}

fn cursor_anchored_bounds(row_count: usize) -> (Bounds<Pixels>, Option<DisplayId>) {
    let row_height = SOFT_COMPACT_PICKER_ROW_HEIGHT;
    let height = inline_popup_height_for_row_height(row_count, row_height);
    let width = INLINE_POPUP_DEFAULT_WIDTH;

    let displays: Vec<_> = get_macos_visible_displays();
    let mouse = get_global_mouse_position().unwrap_or((400.0, 400.0));
    let display = display_for_point(mouse, &displays);
    let visible =
        display
            .as_ref()
            .map(|d| d.visible_area.clone())
            .unwrap_or(crate::windows::DisplayBounds {
                origin_x: 0.0,
                origin_y: 0.0,
                width: 1920.0,
                height: 1080.0,
            });

    let left = (mouse.0 - f64::from(width) * 0.5)
        .max(visible.origin_x + f64::from(INLINE_POPUP_EDGE_GUTTER))
        .min(
            visible.origin_x + visible.width
                - f64::from(width)
                - f64::from(INLINE_POPUP_EDGE_GUTTER),
        );
    let top = (mouse.1 + 12.0)
        .max(visible.origin_y + f64::from(INLINE_POPUP_EDGE_GUTTER))
        .min(
            visible.origin_y + visible.height
                - f64::from(height)
                - f64::from(INLINE_POPUP_EDGE_GUTTER),
        );

    let bounds = Bounds::new(
        gpui::point(px(left as f32), px(top as f32)),
        gpui::size(px(width), px(height)),
    );
    let clamped = clamp_to_visible(bounds, &visible);
    (clamped, None)
}

fn open_post_copy_quick_menu(entry_id: &str, cx: &mut App) -> Result<()> {
    close_post_copy_quick_menu_entity(cx);

    let (bounds, display_id) = cursor_anchored_bounds(3);
    let entry_id_owned = entry_id.to_string();
    let window_options = inline_popup_window_options(bounds, display_id);
    let handle = cx.open_window(window_options, |window, cx| {
        cx.new(|cx| PostCopyQuickMenuWindow::new(entry_id_owned, window, cx))
    })?;

    crate::windows::register_attached_popup(
        POST_COPY_MENU_AUTOMATION_ID.to_string(),
        crate::protocol::AutomationWindowKind::PromptPopup,
        Some("Clipboard Post-Copy Menu".to_string()),
        Some("clipboardPostCopyMenu".to_string()),
        Some(crate::protocol::AutomationWindowBounds {
            x: f32::from(bounds.origin.x) as f64,
            y: f32::from(bounds.origin.y) as f64,
            width: f32::from(bounds.size.width) as f64,
            height: f32::from(bounds.size.height) as f64,
        }),
        None,
    )?;

    if let Ok(mut slot) = quick_menu_slot().lock() {
        *slot = Some(handle);
    }
    debug!(entry_id = %entry_id, "post-copy quick menu opened");
    Ok(())
}

fn close_post_copy_quick_menu_entity(cx: &mut App) {
    if let Ok(mut slot) = quick_menu_slot().lock() {
        if let Some(handle) = slot.take() {
            let _ = handle.update(cx, |_view, window, _cx| {
                window.remove_window();
            });
        }
    }
    crate::windows::remove_automation_window(POST_COPY_MENU_AUTOMATION_ID);
}

fn close_post_copy_quick_menu(window: &mut Window) {
    if let Ok(mut slot) = quick_menu_slot().lock() {
        slot.take();
    }
    crate::windows::remove_automation_window(POST_COPY_MENU_AUTOMATION_ID);
    window.remove_window();
}
