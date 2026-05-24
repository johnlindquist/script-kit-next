use gpui::{
    div, prelude::FluentBuilder, px, rgba, AnyElement, AnyWindowHandle, App, AppContext, Bounds,
    Context, DisplayId, InteractiveElement, IntoElement, MouseButton, MouseDownEvent,
    ParentElement, Pixels, Render, SharedString, StatefulInteractiveElement, Styled, Window,
    WindowBackgroundAppearance, WindowBounds, WindowHandle, WindowKind, WindowOptions,
};
use std::sync::{Mutex, OnceLock};

#[cfg(target_os = "macos")]
use cocoa::base::{id, nil, NO, YES};

#[cfg(target_os = "macos")]
const FOOTER_EFFECT_ID: &str = "script-kit-footer-effect";
#[cfg(target_os = "macos")]
const FOOTER_DIVIDER_ID: &str = "script-kit-footer-divider";
#[cfg(target_os = "macos")]
const FOOTER_HINTS_ID: &str = "script-kit-footer-hints";
#[cfg(target_os = "macos")]
const FOOTER_HINT_ITEM_GAP: f64 =
    crate::components::footer_chrome::FOOTER_ACTION_ITEM_GAP_PX as f64;
#[cfg(target_os = "macos")]
const FOOTER_HINT_KEY_LABEL_GAP: f64 =
    crate::components::footer_chrome::FOOTER_ACTION_CONTENT_GAP_PX as f64;
#[cfg(target_os = "macos")]
const FOOTER_HINT_SIDE_INSET: f64 = crate::window_resize::mini_layout::HINT_STRIP_PADDING_X as f64;
#[cfg(target_os = "macos")]
const FOOTER_HINT_PADDING_X: f64 =
    crate::components::footer_chrome::FOOTER_ACTION_CONTENT_PADDING_X_PX as f64;
#[cfg(target_os = "macos")]
const FOOTER_RUN_HINT_PADDING_X: f64 =
    crate::components::footer_chrome::FOOTER_KEY_ANCHORED_CONTENT_PADDING_X_PX as f64;
#[cfg(target_os = "macos")]
const FOOTER_HINT_RADIUS: f64 =
    crate::components::footer_chrome::FOOTER_ACTION_BUTTON_RADIUS_PX as f64;
#[cfg(target_os = "macos")]
const FOOTER_HINT_TEXT_ALIGN_LEFT: usize = 0;
#[cfg(target_os = "macos")]
const FOOTER_HINT_TEXT_ALIGN_RIGHT: usize = 2;
#[cfg(target_os = "macos")]
const FOOTER_HINT_BUTTON_ID_PREFIX: &str = "script-kit-footer-button-";
#[cfg(target_os = "macos")]
const FOOTER_LEFT_INFO_ID: &str = "script-kit-footer-left-info";
#[cfg(target_os = "macos")]
const FOOTER_STATUS_DOT_ID: &str = "script-kit-footer-status-dot";
#[cfg(target_os = "macos")]
const FOOTER_MODEL_LABEL_ID: &str = "script-kit-footer-model-label";
#[cfg(target_os = "macos")]
const FOOTER_STREAMING_DOT_SIZE: f64 = 6.0;
#[cfg(target_os = "macos")]
const FOOTER_LEFT_DOT_LABEL_GAP: f64 = 6.0;
#[cfg(target_os = "macos")]
const FOOTER_ACTIVE_DOT_MIN_OPACITY: f32 = 0.22;
#[cfg(target_os = "macos")]
const FOOTER_ACTIVE_DOT_HALF_CYCLE_SECONDS: f64 = 1.1;
#[cfg(target_os = "macos")]
const FOOTER_RUN_SLOT_MIN_WIDTH: f64 =
    crate::components::footer_chrome::FOOTER_RUN_SLOT_MIN_WIDTH_PX as f64;
#[cfg(target_os = "macos")]
const FOOTER_RUN_SLOT_MAX_WIDTH: f64 =
    crate::components::footer_chrome::FOOTER_RUN_SLOT_MAX_WIDTH_PX as f64;
const FOOTER_RUN_ANCHORED_PADDING_DELTA_PX: f32 =
    (crate::components::footer_chrome::FOOTER_KEY_ANCHORED_CONTENT_PADDING_X_PX
        - crate::components::footer_chrome::FOOTER_ACTION_CONTENT_PADDING_X_PX)
        * 2.0;
#[cfg(target_os = "macos")]
const FOOTER_ACTIONS_SLOT_WIDTH: f64 =
    crate::components::footer_chrome::FOOTER_ACTIONS_SLOT_WIDTH_PX as f64;
#[cfg(target_os = "macos")]
const FOOTER_AI_SLOT_WIDTH: f64 = crate::components::footer_chrome::FOOTER_AI_SLOT_WIDTH_PX as f64;
#[cfg(target_os = "macos")]
const FOOTER_APPLY_SLOT_WIDTH: f64 =
    crate::components::footer_chrome::FOOTER_APPLY_SLOT_WIDTH_PX as f64;
#[cfg(target_os = "macos")]
const FOOTER_CLOSE_SLOT_WIDTH: f64 =
    crate::components::footer_chrome::FOOTER_CLOSE_SLOT_WIDTH_PX as f64;
#[cfg(target_os = "macos")]
const FOOTER_STOP_SLOT_WIDTH: f64 =
    crate::components::footer_chrome::FOOTER_STOP_SLOT_WIDTH_PX as f64;
#[cfg(target_os = "macos")]
const FOOTER_PASTE_RESPONSE_SLOT_WIDTH: f64 =
    crate::components::footer_chrome::FOOTER_PASTE_RESPONSE_SLOT_WIDTH_PX as f64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FooterAction {
    Run,
    Actions,
    Ai,
    Apply,
    Close,
    Stop,
    PasteResponse,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FooterButtonConfig {
    pub action: FooterAction,
    pub key: SharedString,
    pub label: SharedString,
    pub selected: bool,
    pub enabled: bool,
    pub disabled_reason: Option<&'static str>,
}

impl FooterButtonConfig {
    pub(crate) fn new(
        action: FooterAction,
        key: impl Into<SharedString>,
        label: impl Into<SharedString>,
    ) -> Self {
        Self {
            action,
            key: key.into(),
            label: label.into(),
            selected: false,
            enabled: true,
            disabled_reason: None,
        }
    }

    pub(crate) fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub(crate) fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub(crate) fn disabled_reason(mut self, reason: &'static str) -> Self {
        self.disabled_reason = Some(reason);
        self.enabled = false;
        self
    }
}

impl FooterAction {
    pub(crate) fn is_actions(self) -> bool {
        matches!(self, Self::Actions)
    }
}

/// Status of the ACP thread, used to pick dot color and animation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum FooterDotStatus {
    /// No dot shown.
    #[default]
    Hidden,
    /// Streaming — pulsing, high-contrast theme-aligned dot.
    Streaming,
    /// Waiting for user permission — same pulsing active dot treatment.
    WaitingForPermission,
    /// Idle / done — subtle theme-matched dot.
    Idle,
    /// Error — solid theme error dot.
    Error,
}

/// Optional left-side info for the native footer (status dot + model name).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct FooterLeftInfo {
    /// Controls dot color and animation.
    pub dot_status: FooterDotStatus,
    /// Model display name (e.g. "Claude Sonnet 4"). Empty = hide label.
    pub model_name: String,
    /// When true, active ACP states should use the accent token instead of the
    /// generic high-contrast fallback so the footer clearly reads as AI-active.
    pub prefer_accent_for_active_states: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct MainWindowFooterConfig {
    pub surface: &'static str,
    pub buttons: Vec<FooterButtonConfig>,
    pub left_info: Option<FooterLeftInfo>,
}

impl MainWindowFooterConfig {
    pub(crate) fn new(surface: &'static str, buttons: Vec<FooterButtonConfig>) -> Self {
        Self {
            surface,
            buttons,
            left_info: None,
        }
    }
}

fn footer_active_dot_hex(theme: &crate::theme::Theme, prefer_accent: bool) -> u32 {
    let colors = &theme.colors;
    let accent = colors.accent.selected;

    if prefer_accent {
        return accent;
    }

    let background = colors.background.main;
    let primary_text = colors.text.primary;

    if crate::theme::contrast_ratio(accent, background)
        >= crate::theme::contrast_ratio(primary_text, background)
    {
        accent
    } else {
        primary_text
    }
}

fn footer_dot_hex(
    status: FooterDotStatus,
    theme: &crate::theme::Theme,
    prefer_accent_for_active_states: bool,
) -> u32 {
    let colors = &theme.colors;
    match status {
        FooterDotStatus::Streaming | FooterDotStatus::WaitingForPermission => {
            footer_active_dot_hex(theme, prefer_accent_for_active_states)
        }
        FooterDotStatus::Idle => colors.text.secondary,
        FooterDotStatus::Error => colors.ui.error,
        FooterDotStatus::Hidden => unreachable!(),
    }
}

static FOOTER_ACTION_CHANNEL: std::sync::LazyLock<(
    async_channel::Sender<FooterAction>,
    async_channel::Receiver<FooterAction>,
)> = std::sync::LazyLock::new(|| async_channel::bounded(32));

static DICTATION_FOOTER_ACTION_CHANNEL: std::sync::LazyLock<(
    async_channel::Sender<FooterAction>,
    async_channel::Receiver<FooterAction>,
)> = std::sync::LazyLock::new(|| async_channel::bounded(32));

static ACP_FOOTER_ACTION_CHANNEL: std::sync::LazyLock<(
    async_channel::Sender<FooterAction>,
    async_channel::Receiver<FooterAction>,
)> = std::sync::LazyLock::new(|| async_channel::bounded(32));

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct MainWindowFooterHostSnapshot {
    pub requested_surface: Option<&'static str>,
    pub installed_surface: Option<&'static str>,
    pub native_host_installed: bool,
}

static MAIN_WINDOW_FOOTER_HOST_STATE: std::sync::Mutex<MainWindowFooterHostSnapshot> =
    std::sync::Mutex::new(MainWindowFooterHostSnapshot {
        requested_surface: None,
        installed_surface: None,
        native_host_installed: false,
    });

#[derive(Clone, Debug, PartialEq, Eq)]
struct MainWindowFooterRefreshSignature {
    config: MainWindowFooterConfig,
    content_width_bits: u64,
    dark: bool,
    material: crate::theme::VibrancyMaterial,
    divider_rgba: u32,
    text_primary_hex: u32,
    background_hex: u32,
    accent_hex: u32,
    selection_rgba: u32,
    hover_rgba: u32,
    left_dot_hex: Option<u32>,
}

static MAIN_WINDOW_FOOTER_REFRESH_SIGNATURE: std::sync::Mutex<
    Option<MainWindowFooterRefreshSignature>,
> = std::sync::Mutex::new(None);

struct GpuiFooterOverlaySlot {
    handle: WindowHandle<GpuiFooterOverlay>,
    parent_window_handle: AnyWindowHandle,
}

static MAIN_WINDOW_GPUI_FOOTER_OVERLAY: OnceLock<Mutex<Option<GpuiFooterOverlaySlot>>> =
    OnceLock::new();

struct GpuiFooterOverlay {
    config: MainWindowFooterConfig,
    overlay_width_px: f32,
}

impl GpuiFooterOverlay {
    fn new(config: MainWindowFooterConfig, overlay_width_px: f32) -> Self {
        Self {
            config,
            overlay_width_px,
        }
    }

    fn set_config(&mut self, config: MainWindowFooterConfig, overlay_width_px: f32) {
        self.config = config;
        self.overlay_width_px = overlay_width_px;
    }

    fn content_width_px(&self) -> f32 {
        (self.overlay_width_px - crate::window_resize::mini_layout::HINT_STRIP_PADDING_X * 2.0)
            .max(0.0)
    }

    fn button_width_px(&self, button: &FooterButtonConfig) -> f32 {
        footer_overlay_button_width_px(button, self.content_width_px(), &self.config.buttons)
    }

    fn trailing_button_widths(&self, buttons: &[FooterButtonConfig]) -> Vec<f32> {
        if buttons.is_empty() {
            return Vec::new();
        }

        let fixed_tail_width = buttons
            .iter()
            .skip(1)
            .map(footer_overlay_button_full_width_px)
            .sum::<f32>();
        let gap_width = buttons.len().saturating_sub(1) as f32
            * crate::components::footer_chrome::FOOTER_ACTION_ITEM_GAP_PX;
        let first = &buttons[0];
        let first_base = footer_hint_slot_width(first.action) as f32;
        let first_full = footer_overlay_button_full_width_px(first);
        let first_max = if matches!(first.action, FooterAction::Run) {
            crate::components::footer_chrome::FOOTER_RUN_SLOT_MAX_WIDTH_PX
        } else {
            first_full
        };
        let first_available = self.content_width_px() - fixed_tail_width - gap_width;
        let first_width = first_available
            .clamp(first_base, first_max.max(first_base))
            .min(first_full.max(first_base));

        std::iter::once(first_width)
            .chain(
                buttons
                    .iter()
                    .skip(1)
                    .map(footer_overlay_button_full_width_px),
            )
            .collect()
    }

    fn render_left_info(
        &self,
        left_info: Option<&FooterLeftInfo>,
        theme: &crate::theme::Theme,
    ) -> AnyElement {
        let Some(info) = left_info else {
            return div().into_any_element();
        };

        let mut row = div()
            .flex()
            .flex_1()
            .items_center()
            .gap(px(FOOTER_LEFT_DOT_LABEL_GAP as f32))
            .min_w(px(0.0))
            .overflow_hidden();

        if !matches!(info.dot_status, FooterDotStatus::Hidden) {
            row = row.child(
                div()
                    .size(px(FOOTER_STREAMING_DOT_SIZE as f32))
                    .rounded(px((FOOTER_STREAMING_DOT_SIZE / 2.0) as f32))
                    .bg(rgba(
                        (footer_dot_hex(
                            info.dot_status,
                            theme,
                            info.prefer_accent_for_active_states,
                        ) << 8)
                            | 0xff,
                    )),
            );
        }

        if !info.model_name.trim().is_empty() {
            row = row.child(
                div()
                    .min_w(px(0.0))
                    .font_family(crate::list_item::FONT_SYSTEM_UI)
                    .font_weight(crate::components::footer_chrome::FOOTER_HINT_FONT_WEIGHT_GPUI)
                    .text_size(px(
                        crate::components::footer_chrome::FOOTER_HINT_FONT_SIZE_PX,
                    ))
                    .text_color(crate::components::footer_chrome::footer_hint_text_color(
                        theme,
                    ))
                    .overflow_hidden()
                    .text_ellipsis()
                    .whitespace_nowrap()
                    .child(info.model_name.clone()),
            );
        }

        row.into_any_element()
    }

    fn render_button(
        &self,
        button: FooterButtonConfig,
        slot_width: f32,
        theme: &crate::theme::Theme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let chrome = crate::theme::AppChromeColors::from_theme(theme);
        let action = button.action;
        let selected_bg = rgba(footer_selected_background_rgba(action, &chrome));
        let hover_bg = rgba(chrome.hover_rgba);
        let active_bg = rgba(chrome.selection_rgba);
        let item_height = crate::components::footer_chrome::footer_button_height(
            crate::window_resize::mini_layout::NATIVE_MAIN_WINDOW_FOOTER_HEIGHT,
        );
        let key_first = is_footer_left_pinned_mic_button(&button);
        let justify = if key_first {
            crate::components::footer_chrome::FooterHintContentJustify::Start
        } else if matches!(action, FooterAction::Run) {
            crate::components::footer_chrome::FooterHintContentJustify::KeyAnchored
        } else {
            crate::components::footer_chrome::FooterHintContentJustify::Center
        };

        let mut item = div()
            .id(format!(
                "gpui-footer-overlay-button-{}",
                footer_action_key(action)
            ))
            .w(px(slot_width))
            .min_w(px(slot_width))
            .max_w(px(slot_width))
            .h(px(item_height))
            .rounded(px(
                crate::components::footer_chrome::FOOTER_ACTION_BUTTON_RADIUS_PX,
            ))
            .overflow_hidden()
            .flex()
            .items_center()
            .justify_center()
            .group("footer-action-button")
            .when(button.selected, |style| style.bg(selected_bg))
            .child(
                crate::components::footer_chrome::render_footer_hint_content_constrained(
                    button.label.clone(),
                    button.key.clone(),
                    crate::components::footer_chrome::FooterHintKeyMode::Shortcut,
                    theme,
                    slot_width,
                    key_first,
                    justify,
                ),
            );

        if button.enabled {
            item = item
                .cursor_pointer()
                .hover(move |style| style.bg(hover_bg))
                .active(move |style| style.bg(active_bg))
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |_this, _event: &MouseDownEvent, _window, cx| {
                        cx.stop_propagation();
                        send_footer_action_to_channel(action, false);
                    }),
                );
        } else {
            item = item.opacity(0.45);
        }

        item.into_any_element()
    }
}

impl Render for GpuiFooterOverlay {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = crate::theme::get_cached_theme();
        let left_pinned_buttons: Vec<_> = self
            .config
            .buttons
            .iter()
            .filter(|button| is_footer_left_pinned_mic_button(button))
            .cloned()
            .collect();
        let trailing_buttons: Vec<_> = self
            .config
            .buttons
            .iter()
            .filter(|button| !is_footer_left_pinned_mic_button(button))
            .cloned()
            .collect();
        let trailing_button_widths = self.trailing_button_widths(&trailing_buttons);

        div()
            .id("gpui-footer-overlay-spike")
            .w_full()
            .h_full()
            .px(px(crate::window_resize::mini_layout::HINT_STRIP_PADDING_X))
            .py(px(
                crate::components::footer_chrome::FOOTER_BUTTON_VERTICAL_INSET_PX,
            ))
            .flex()
            .items_center()
            .gap(px(
                crate::components::footer_chrome::FOOTER_ACTION_ITEM_GAP_PX,
            ))
            .child(
                div()
                    .flex()
                    .flex_1()
                    .items_center()
                    .gap(px(
                        crate::components::footer_chrome::FOOTER_ACTION_ITEM_GAP_PX,
                    ))
                    .min_w(px(0.0))
                    .overflow_hidden()
                    .children(left_pinned_buttons.into_iter().map(|button| {
                        let slot_width = self.button_width_px(&button);
                        self.render_button(button, slot_width, &theme, cx)
                    }))
                    .child(self.render_left_info(self.config.left_info.as_ref(), &theme)),
            )
            .child(
                div()
                    .flex()
                    .flex_none()
                    .items_center()
                    .gap(px(
                        crate::components::footer_chrome::FOOTER_ACTION_ITEM_GAP_PX,
                    ))
                    .children(
                        trailing_buttons
                            .into_iter()
                            .zip(trailing_button_widths)
                            .map(|(button, slot_width)| {
                                self.render_button(button, slot_width, &theme, cx)
                            }),
                    ),
            )
    }
}

fn footer_overlay_button_width_px(
    button: &FooterButtonConfig,
    content_width_px: f32,
    buttons: &[FooterButtonConfig],
) -> f32 {
    footer_hint_max_item_width(button.action, content_width_px as f64, buttons)
        .unwrap_or_else(|| footer_hint_slot_width(button.action))
        .max(footer_hint_slot_width(button.action)) as f32
}

fn footer_overlay_button_full_width_px(button: &FooterButtonConfig) -> f32 {
    let width = crate::components::footer_chrome::footer_hint_content_estimated_width_px(
        button.label.as_ref(),
        button.key.as_ref(),
        crate::components::footer_chrome::FooterHintKeyMode::Shortcut,
    );
    let width = if matches!(button.action, FooterAction::Run) && !button.label.trim().is_empty() {
        width + FOOTER_RUN_ANCHORED_PADDING_DELTA_PX
    } else {
        width
    };

    width
        .max(footer_hint_slot_width(button.action) as f32)
        .ceil()
}

fn gpui_footer_overlay_spike_enabled() -> bool {
    std::env::var("SCRIPT_KIT_GPUI_FOOTER_OVERLAY_SPIKE")
        .map(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
        .unwrap_or(false)
}

fn gpui_footer_overlay_bounds(parent_bounds: Bounds<Pixels>) -> Bounds<Pixels> {
    let footer_height = crate::window_resize::mini_layout::NATIVE_MAIN_WINDOW_FOOTER_HEIGHT;
    Bounds {
        origin: gpui::point(
            parent_bounds.origin.x,
            parent_bounds.origin.y + parent_bounds.size.height - px(footer_height),
        ),
        size: gpui::size(parent_bounds.size.width, px(footer_height)),
    }
}

fn gpui_footer_overlay_window_options(
    bounds: Bounds<Pixels>,
    display_id: Option<DisplayId>,
) -> WindowOptions {
    WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        titlebar: None,
        window_background: WindowBackgroundAppearance::Transparent,
        focus: false,
        show: true,
        kind: WindowKind::PopUp,
        is_movable: false,
        is_resizable: false,
        is_minimizable: false,
        display_id,
        ..Default::default()
    }
}

fn clear_main_window_footer_refresh_signature() {
    *MAIN_WINDOW_FOOTER_REFRESH_SIGNATURE
        .lock()
        .unwrap_or_else(|poison| poison.into_inner()) = None;
}

fn close_gpui_footer_overlay(cx: &mut App) {
    let storage = MAIN_WINDOW_GPUI_FOOTER_OVERLAY.get_or_init(|| Mutex::new(None));
    let slot = storage.lock().ok().and_then(|mut guard| guard.take());
    if let Some(slot) = slot {
        let _ = slot.handle.update(cx, |_overlay, window, _cx| {
            window.remove_window();
        });
    }
}

fn sync_gpui_footer_overlay(
    cx: &mut App,
    parent_window_handle: AnyWindowHandle,
    parent_bounds: Bounds<Pixels>,
    display_id: Option<DisplayId>,
    config: MainWindowFooterConfig,
) {
    if !gpui_footer_overlay_spike_enabled() {
        close_gpui_footer_overlay(cx);
        return;
    }

    let bounds = gpui_footer_overlay_bounds(parent_bounds);
    let overlay_width_px: f32 = bounds.size.width.into();
    let storage = MAIN_WINDOW_GPUI_FOOTER_OVERLAY.get_or_init(|| Mutex::new(None));
    if let Ok(mut guard) = storage.lock() {
        if let Some(slot) = guard.as_ref() {
            if slot.parent_window_handle == parent_window_handle {
                let update_result = slot.handle.update(cx, |overlay, window, cx| {
                    overlay.set_config(config.clone(), overlay_width_px);
                    set_gpui_footer_overlay_window_bounds(window, bounds, cx);
                    cx.notify();
                });
                if update_result.is_ok() {
                    return;
                }
                *guard = None;
            } else {
                let _ = slot.handle.update(cx, |_overlay, window, _cx| {
                    window.remove_window();
                });
                *guard = None;
            }
        }
    }

    let options = gpui_footer_overlay_window_options(bounds, display_id);
    let Ok(handle) = cx.open_window(options, |_window, cx| {
        cx.new(|_| GpuiFooterOverlay::new(config.clone(), overlay_width_px))
    }) else {
        tracing::warn!(
            target: "script_kit::footer_popup",
            event = "gpui_footer_overlay_open_failed",
            "Failed to open experimental GPUI footer overlay"
        );
        return;
    };

    if configure_gpui_footer_overlay_window(&handle, cx, parent_window_handle).is_err() {
        let _ = handle.update(cx, |_overlay, window, _cx| {
            window.remove_window();
        });
        return;
    }

    if let Ok(mut guard) = storage.lock() {
        *guard = Some(GpuiFooterOverlaySlot {
            handle,
            parent_window_handle,
        });
    }
}

fn update_main_window_footer_host_state(
    requested_surface: Option<&'static str>,
    installed_surface: Option<&'static str>,
    native_host_installed: bool,
) {
    *MAIN_WINDOW_FOOTER_HOST_STATE
        .lock()
        .unwrap_or_else(|poison| poison.into_inner()) = MainWindowFooterHostSnapshot {
        requested_surface,
        installed_surface,
        native_host_installed,
    };
}

pub(crate) fn main_window_footer_host_snapshot() -> MainWindowFooterHostSnapshot {
    *MAIN_WINDOW_FOOTER_HOST_STATE
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
}

pub(crate) fn active_main_window_footer_surface() -> Option<&'static str> {
    main_window_footer_host_snapshot().installed_surface
}

pub(crate) fn footer_action_channel() -> &'static (
    async_channel::Sender<FooterAction>,
    async_channel::Receiver<FooterAction>,
) {
    &FOOTER_ACTION_CHANNEL
}

pub(crate) fn dictation_footer_action_channel() -> &'static (
    async_channel::Sender<FooterAction>,
    async_channel::Receiver<FooterAction>,
) {
    &DICTATION_FOOTER_ACTION_CHANNEL
}

pub(crate) fn acp_footer_action_channel() -> &'static (
    async_channel::Sender<FooterAction>,
    async_channel::Receiver<FooterAction>,
) {
    &ACP_FOOTER_ACTION_CHANNEL
}

pub(crate) fn sync_main_footer_popup(
    window: &mut Window,
    config: Option<&MainWindowFooterConfig>,
    cx: &mut App,
) {
    let requested_surface = config.map(|cfg| cfg.surface);
    update_main_window_footer_host_state(requested_surface, None, false);
    let parent_window_handle = window.window_handle();
    let parent_bounds = window.bounds();
    let display_id = window.display(cx).as_ref().map(|display| display.id());

    #[cfg(target_os = "macos")]
    {
        let Some(ns_window) = main_window_ns_window(window) else {
            tracing::warn!(
                target: "script_kit::footer_popup",
                event = "native_footer_missing_ns_window",
                "Unable to resolve NSWindow for native footer host"
            );
            return;
        };

        // SAFETY: `ns_window` comes from the live GPUI main window currently
        // being rendered/observed on the AppKit thread.
        unsafe {
            use objc::{msg_send, sel, sel_impl};
            if let Some(config) = config {
                let content_view: id = msg_send![ns_window, contentView];
                let existed = content_view != nil
                    && find_subview_by_identifier(content_view, FOOTER_EFFECT_ID) != nil;
                let installed_host = ensure_main_footer_host(ns_window);
                if installed_host && !existed {
                    clear_main_window_footer_refresh_signature();
                }
                let installed = installed_host && refresh_main_footer_host(ns_window, config);
                update_main_window_footer_host_state(
                    requested_surface,
                    installed.then_some(config.surface),
                    installed,
                );
            } else {
                clear_main_window_footer_refresh_signature();
                remove_main_footer_host(ns_window);
                update_main_window_footer_host_state(None, None, false);
            }
        }
    }

    if let Some(config) = config {
        sync_gpui_footer_overlay(
            cx,
            parent_window_handle,
            parent_bounds,
            display_id,
            config.clone(),
        );
    } else {
        close_gpui_footer_overlay(cx);
    }

    #[cfg(not(target_os = "macos"))]
    let _ = (window, config);
}

pub(crate) fn sync_window_footer_popup(window: &mut Window, config: &MainWindowFooterConfig) {
    #[cfg(target_os = "macos")]
    {
        let Some(ns_window) = main_window_ns_window(window) else {
            tracing::warn!(
                target: "script_kit::footer_popup",
                event = "native_footer_missing_ns_window",
                surface = config.surface,
                "Unable to resolve NSWindow for reusable native footer host"
            );
            return;
        };

        // SAFETY: `ns_window` comes from the live GPUI window currently being
        // rendered/observed on the AppKit thread.
        unsafe {
            let installed = ensure_main_footer_host(ns_window);
            if installed {
                let _ = refresh_main_footer_host(ns_window, config);
            }
        }
    }

    #[cfg(not(target_os = "macos"))]
    let _ = (window, config);
}

pub(crate) fn notify_main_footer_popup(
    window: &mut Window,
    config: Option<&MainWindowFooterConfig>,
    cx: &mut App,
) {
    let requested_surface = config.map(|cfg| cfg.surface);
    update_main_window_footer_host_state(requested_surface, None, false);
    let parent_window_handle = window.window_handle();
    let parent_bounds = window.bounds();
    let display_id = window.display(cx).as_ref().map(|display| display.id());

    #[cfg(target_os = "macos")]
    {
        let Some(ns_window) = main_window_ns_window(window) else {
            return;
        };

        // SAFETY: `ns_window` comes from the live GPUI main window currently
        // being rendered/observed on the AppKit thread.
        unsafe {
            if let Some(config) = config {
                let installed = refresh_main_footer_host(ns_window, config);
                update_main_window_footer_host_state(
                    requested_surface,
                    installed.then_some(config.surface),
                    installed,
                );
            } else {
                clear_main_window_footer_refresh_signature();
                remove_main_footer_host(ns_window);
                update_main_window_footer_host_state(None, None, false);
            }
        }
    }

    if let Some(config) = config {
        sync_gpui_footer_overlay(
            cx,
            parent_window_handle,
            parent_bounds,
            display_id,
            config.clone(),
        );
    } else {
        close_gpui_footer_overlay(cx);
    }

    #[cfg(not(target_os = "macos"))]
    let _ = (window, config);
}

pub(crate) fn close_main_footer_popup(cx: &mut App) {
    clear_main_window_footer_refresh_signature();
    update_main_window_footer_host_state(None, None, false);
    close_gpui_footer_overlay(cx);

    let Some(window_handle) = crate::get_main_window_handle() else {
        return;
    };

    let _ = window_handle.update(cx, move |_, window, _cx| {
        #[cfg(target_os = "macos")]
        {
            let Some(ns_window) = main_window_ns_window(window) else {
                return;
            };

            // SAFETY: `ns_window` comes from the live GPUI main window on the
            // AppKit main thread while `update_window` is executing.
            unsafe {
                remove_main_footer_host(ns_window);
            }
        }

        #[cfg(not(target_os = "macos"))]
        let _ = window;
    });
}

#[cfg(target_os = "macos")]
fn main_window_ns_window(window: &mut Window) -> Option<id> {
    if let Ok(window_handle) = raw_window_handle::HasWindowHandle::window_handle(window) {
        if let raw_window_handle::RawWindowHandle::AppKit(appkit) = window_handle.as_raw() {
            use objc::{msg_send, sel, sel_impl};

            let ns_view = appkit.ns_view.as_ptr() as id;
            // SAFETY: `ns_view` comes from a live GPUI window on the AppKit
            // main thread. `-[NSView window]` returns the owning NSWindow or nil.
            unsafe {
                let ns_window: id = msg_send![ns_view, window];
                if ns_window != nil {
                    return Some(ns_window);
                }
            }
        }
    }

    None
}

fn set_gpui_footer_overlay_window_bounds(
    window: &mut Window,
    bounds: Bounds<Pixels>,
    cx: &mut App,
) {
    crate::components::inline_popup_window::set_inline_popup_window_bounds(window, bounds, cx);
}

fn configure_gpui_footer_overlay_window<T: 'static>(
    handle: &WindowHandle<T>,
    cx: &mut App,
    parent_window_handle: AnyWindowHandle,
) -> anyhow::Result<()> {
    #[cfg(target_os = "macos")]
    {
        handle
            .update(cx, move |_overlay, window, cx| {
                window.defer(cx, move |window, cx| {
                    if let Some(ns_window) =
                        crate::components::inline_popup_window::inline_popup_ns_window(window)
                    {
                        // SAFETY: `ns_window` is the live GPUI overlay NSWindow.
                        // The spike overlay is visual-only; mouse and key focus
                        // must continue to belong to the main launcher window.
                        unsafe {
                            configure_gpui_footer_overlay_ns_window(ns_window);
                        }
                        crate::components::inline_popup_window::attach_inline_popup_to_parent_window(
                            cx,
                            parent_window_handle,
                            ns_window,
                        );
                    }
                });
            })
            .map_err(|_| anyhow::anyhow!("failed to configure GPUI footer overlay window"))?;
    }

    #[cfg(not(target_os = "macos"))]
    let _ = (handle, cx, parent_window_handle);

    Ok(())
}

#[cfg(target_os = "macos")]
unsafe fn configure_gpui_footer_overlay_ns_window(ns_window: id) {
    use cocoa::base::NO;
    use objc::{class, msg_send, sel, sel_impl};

    if ns_window == nil {
        return;
    }

    let clear_color: id = msg_send![class!(NSColor), clearColor];
    if clear_color != nil {
        let _: () = msg_send![ns_window, setBackgroundColor: clear_color];
    }
    let _: () = msg_send![ns_window, setOpaque: NO];
    let _: () = msg_send![ns_window, setHasShadow: NO];
    let _: () = msg_send![ns_window, setIgnoresMouseEvents: NO];
    let _: () = msg_send![ns_window, setBecomesKeyOnlyIfNeeded: YES];
    let _: () = msg_send![ns_window, setMovable: NO];
    let _: () = msg_send![ns_window, setMovableByWindowBackground: NO];
    let _: () = msg_send![ns_window, setAnimationBehavior: 2isize];
    let _: () = msg_send![ns_window, setRestorable: NO];

    let title = ns_string("Script Kit GPUI Footer Overlay Spike");
    if title != nil {
        let _: () = msg_send![ns_window, setTitle: title];
    }
}

#[cfg(target_os = "macos")]
unsafe fn ensure_main_footer_host(ns_window: id) -> bool {
    use cocoa::appkit::NSViewWidthSizable;
    use cocoa::foundation::{NSPoint, NSRect, NSSize};
    use objc::{class, msg_send, sel, sel_impl};

    if crate::platform::require_main_thread("ensure_main_footer_host") {
        return false;
    }

    let content_view: id = msg_send![ns_window, contentView];
    if content_view == nil {
        return false;
    }

    let existing = find_subview_by_identifier(content_view, FOOTER_EFFECT_ID);
    if existing != nil {
        return true;
    }

    let content_bounds: NSRect = msg_send![content_view, bounds];
    let footer_frame = NSRect::new(
        NSPoint::new(0.0, 0.0),
        NSSize::new(content_bounds.size.width, footer_height()),
    );

    let footer_cls = footer_effect_view_class();
    let footer_view: id = msg_send![footer_cls, alloc];
    let footer_view: id = msg_send![footer_view, initWithFrame: footer_frame];
    if footer_view == nil {
        return false;
    }

    let effect_identifier = ns_string(FOOTER_EFFECT_ID);
    if effect_identifier != nil {
        let _: () = msg_send![footer_view, setIdentifier: effect_identifier];
    }
    let _: () = msg_send![footer_view, setAutoresizingMask: NSViewWidthSizable];
    let _: () = msg_send![footer_view, setWantsLayer: YES];

    let divider_view: id = msg_send![class!(NSView), alloc];
    let divider_view: id = msg_send![
        divider_view,
        initWithFrame: NSRect::new(
            NSPoint::new(0.0, footer_height() - 1.0),
            NSSize::new(content_bounds.size.width, 1.0)
        )
    ];
    if divider_view != nil {
        let divider_identifier = ns_string(FOOTER_DIVIDER_ID);
        if divider_identifier != nil {
            let _: () = msg_send![divider_view, setIdentifier: divider_identifier];
        }
        let _: () = msg_send![divider_view, setAutoresizingMask: NSViewWidthSizable];
        let _: () = msg_send![divider_view, setWantsLayer: YES];
        let _: () = msg_send![footer_view, addSubview: divider_view];
    }

    let hints_view: id = msg_send![class!(NSView), alloc];
    let hints_view: id =
        msg_send![hints_view, initWithFrame: footer_hints_frame(content_bounds.size.width)];
    if hints_view != nil {
        let hints_identifier = ns_string(FOOTER_HINTS_ID);
        if hints_identifier != nil {
            let _: () = msg_send![hints_view, setIdentifier: hints_identifier];
        }
        let _: () = msg_send![hints_view, setAutoresizingMask: NSViewWidthSizable];
        let _: () = msg_send![footer_view, addSubview: hints_view];
    }

    // Left-info container (streaming dot + model label)
    let left_info_view: id = msg_send![footer_passthrough_view_class(), alloc];
    let left_info_view: id = msg_send![
        left_info_view,
        initWithFrame: footer_left_info_frame(content_bounds.size.width)
    ];
    if left_info_view != nil {
        let left_info_id = ns_string(FOOTER_LEFT_INFO_ID);
        if left_info_id != nil {
            let _: () = msg_send![left_info_view, setIdentifier: left_info_id];
        }
        let _: () = msg_send![left_info_view, setAutoresizingMask: NSViewWidthSizable];
        let _: () = msg_send![footer_view, addSubview: left_info_view];
    }

    let _: () = msg_send![
        content_view,
        addSubview: footer_view
        positioned: 1isize
        relativeTo: nil
    ];

    tracing::info!(
        target: "script_kit::footer_popup",
        event = "native_footer_host_installed",
        "Installed native footer host inside the main window contentView"
    );

    find_subview_by_identifier(content_view, FOOTER_EFFECT_ID) != nil
}

#[cfg(target_os = "macos")]
unsafe fn refresh_main_footer_host(ns_window: id, config: &MainWindowFooterConfig) -> bool {
    use cocoa::foundation::{NSPoint, NSRect, NSSize};
    use objc::{class, msg_send, sel, sel_impl};

    if crate::platform::require_main_thread("refresh_main_footer_host") {
        return false;
    }

    let content_view: id = msg_send![ns_window, contentView];
    if content_view == nil {
        return false;
    }

    let footer_view = find_subview_by_identifier(content_view, FOOTER_EFFECT_ID);
    if footer_view == nil {
        return false;
    }

    let theme = crate::theme::get_cached_theme();
    let chrome = crate::theme::AppChromeColors::from_theme(&theme);
    let is_dark = theme.should_use_dark_vibrancy();
    let material = match theme.get_vibrancy().material {
        crate::theme::VibrancyMaterial::Hud => {
            crate::platform::ns_visual_effect_material::HUD_WINDOW
        }
        crate::theme::VibrancyMaterial::Popover => {
            crate::platform::ns_visual_effect_material::POPOVER
        }
        crate::theme::VibrancyMaterial::Menu => crate::platform::ns_visual_effect_material::MENU,
        crate::theme::VibrancyMaterial::Sidebar => {
            crate::platform::ns_visual_effect_material::SIDEBAR
        }
        crate::theme::VibrancyMaterial::Content => {
            crate::platform::ns_visual_effect_material::CONTENT_BACKGROUND
        }
    };
    let content_bounds: NSRect = msg_send![content_view, bounds];
    let left_dot_hex = config.left_info.as_ref().and_then(|info| {
        if matches!(info.dot_status, FooterDotStatus::Hidden) {
            None
        } else {
            Some(footer_dot_hex(
                info.dot_status,
                &theme,
                info.prefer_accent_for_active_states,
            ))
        }
    });
    let signature = MainWindowFooterRefreshSignature {
        config: config.clone(),
        content_width_bits: content_bounds.size.width.to_bits(),
        dark: is_dark,
        material: theme.get_vibrancy().material,
        divider_rgba: chrome.divider_rgba,
        text_primary_hex: theme.colors.text.primary,
        background_hex: theme.colors.background.main,
        accent_hex: chrome.accent_hex,
        selection_rgba: chrome.selection_rgba,
        hover_rgba: chrome.hover_rgba,
        left_dot_hex,
    };
    {
        let mut guard = MAIN_WINDOW_FOOTER_REFRESH_SIGNATURE
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        if guard.as_ref() == Some(&signature) {
            update_main_window_footer_host_state(Some(config.surface), Some(config.surface), true);
            return true;
        }
        *guard = Some(signature);
    }

    let appearance_name = if is_dark {
        ns_string("NSAppearanceNameVibrantDark")
    } else {
        ns_string("NSAppearanceNameVibrantLight")
    };
    if appearance_name != nil {
        let appearance: id = msg_send![class!(NSAppearance), appearanceNamed: appearance_name];
        if appearance != nil {
            let _: () = msg_send![footer_view, setAppearance: appearance];
        }
    }

    let _: () = msg_send![footer_view, setMaterial: material];
    let _: () = msg_send![footer_view, setState: 1isize];
    let _: () = msg_send![footer_view, setBlendingMode: 1isize];
    let _: () = msg_send![footer_view, setEmphasized: is_dark];
    let _: () = msg_send![footer_view, setNeedsDisplay: YES];

    let footer_frame = NSRect::new(
        NSPoint::new(0.0, 0.0),
        NSSize::new(content_bounds.size.width, footer_height()),
    );
    let _: () = msg_send![footer_view, setFrame: footer_frame];

    let footer_layer: id = msg_send![footer_view, layer];
    if footer_layer != nil {
        let _: () = msg_send![footer_layer, setCornerRadius: 0.0_f64];
        let _: () = msg_send![footer_layer, setMasksToBounds: YES];
    }

    let divider_view = find_subview_by_identifier(footer_view, FOOTER_DIVIDER_ID);
    if divider_view != nil {
        let divider_frame = NSRect::new(
            NSPoint::new(0.0, footer_height() - 1.0),
            NSSize::new(content_bounds.size.width, 1.0),
        );
        let _: () = msg_send![divider_view, setFrame: divider_frame];
        let divider_layer: id = msg_send![divider_view, layer];
        if divider_layer != nil {
            let divider_color = ns_color_from_rgba(chrome.divider_rgba);
            if divider_color != nil {
                let cg_color: id = msg_send![divider_color, CGColor];
                if cg_color != nil {
                    let _: () = msg_send![divider_layer, setBackgroundColor: cg_color];
                }
            }
        }
    }

    let alpha = crate::window_resize::mini_layout::HINT_TEXT_OPACITY as f64;
    let text_color = ns_color_from_hex_with_alpha(theme.colors.text.primary, alpha);

    let hints_view = find_subview_by_identifier(footer_view, FOOTER_HINTS_ID);
    if hints_view != nil {
        let _: () = msg_send![hints_view, setFrame: footer_hints_frame(content_bounds.size.width)];
        if gpui_footer_overlay_spike_enabled() {
            // The spike proves the sandwich layering: AppKit keeps only the
            // material/divider while GPUI owns the footer glyphs in a child
            // overlay window above this footer host.
            layout_footer_hints(hints_view, text_color, &[], &theme);
        } else {
            layout_footer_hints(hints_view, text_color, &config.buttons, &theme);
        }
    }

    // Left info (streaming dot + model name)
    let left_info_view = find_subview_by_identifier(footer_view, FOOTER_LEFT_INFO_ID);
    if left_info_view != nil {
        let _: () = msg_send![
            left_info_view,
            setFrame: footer_left_info_frame(content_bounds.size.width)
        ];
        if gpui_footer_overlay_spike_enabled() {
            layout_footer_left_info(left_info_view, None, text_color);
        } else {
            layout_footer_left_info(left_info_view, config.left_info.as_ref(), text_color);
        }
    }

    tracing::info!(
        target: "script_kit::footer_popup",
        event = "native_footer_host_refreshed",
        surface = config.surface,
        button_count = config.buttons.len(),
        width = content_bounds.size.width,
        height = footer_height(),
        dark = is_dark,
        "Refreshed native footer host"
    );

    true
}

#[cfg(target_os = "macos")]
unsafe fn remove_main_footer_host(ns_window: id) {
    use objc::{msg_send, sel, sel_impl};

    if crate::platform::require_main_thread("remove_main_footer_host") {
        return;
    }

    clear_main_window_footer_refresh_signature();

    let content_view: id = msg_send![ns_window, contentView];
    if content_view == nil {
        return;
    }

    let footer_view = find_subview_by_identifier(content_view, FOOTER_EFFECT_ID);
    if footer_view == nil {
        return;
    }

    let _: () = msg_send![footer_view, removeFromSuperview];
}

#[cfg(target_os = "macos")]
unsafe fn find_subview_by_identifier(parent: id, identifier: &str) -> id {
    use objc::{msg_send, sel, sel_impl};

    let identifier = ns_string(identifier);
    if parent == nil || identifier == nil {
        return nil;
    }

    let subviews: id = msg_send![parent, subviews];
    if subviews == nil {
        return nil;
    }

    let count: usize = msg_send![subviews, count];
    for index in 0..count {
        let view: id = msg_send![subviews, objectAtIndex: index];
        if view == nil {
            continue;
        }
        let view_identifier: id = msg_send![view, identifier];
        if view_identifier != nil {
            let matches: cocoa::base::BOOL =
                msg_send![view_identifier, isEqualToString: identifier];
            if matches == YES {
                return view;
            }
        }
    }

    nil
}

#[cfg(target_os = "macos")]
fn footer_height() -> f64 {
    crate::window_resize::mini_layout::NATIVE_MAIN_WINDOW_FOOTER_HEIGHT as f64
}

#[cfg(target_os = "macos")]
fn footer_hints_frame(width: f64) -> cocoa::foundation::NSRect {
    cocoa::foundation::NSRect::new(
        cocoa::foundation::NSPoint::new(FOOTER_HINT_SIDE_INSET, 0.0),
        cocoa::foundation::NSSize::new(width - (FOOTER_HINT_SIDE_INSET * 2.0), footer_height()),
    )
}

#[cfg(target_os = "macos")]
fn footer_left_info_frame(width: f64) -> cocoa::foundation::NSRect {
    cocoa::foundation::NSRect::new(
        cocoa::foundation::NSPoint::new(FOOTER_HINT_SIDE_INSET, 0.0),
        cocoa::foundation::NSSize::new(width / 2.0, footer_height()),
    )
}

#[cfg(target_os = "macos")]
unsafe fn layout_footer_left_info(
    left_info_view: id,
    left_info: Option<&FooterLeftInfo>,
    text_color: id,
) {
    use cocoa::foundation::{NSPoint, NSRect, NSSize};
    use objc::{msg_send, sel, sel_impl};

    let Some(info) = left_info else {
        remove_identified_subview(left_info_view, FOOTER_STATUS_DOT_ID);
        remove_identified_subview(left_info_view, FOOTER_MODEL_LABEL_ID);
        return;
    };

    let bounds: NSRect = msg_send![left_info_view, bounds];
    let mut x = 0.0_f64;

    // ── Status dot (color + animation depends on thread status) ──
    let show_dot = !matches!(info.dot_status, FooterDotStatus::Hidden);
    if show_dot {
        let dot_y = ((bounds.size.height - FOOTER_STREAMING_DOT_SIZE) / 2.0).round();
        let dot_view = ensure_footer_status_dot_view(left_info_view);
        if dot_view != nil {
            let _: () = msg_send![
                dot_view,
                setFrame: NSRect::new(
                    NSPoint::new(x, dot_y),
                    NSSize::new(FOOTER_STREAMING_DOT_SIZE, FOOTER_STREAMING_DOT_SIZE),
                )
            ];
            let dot_layer: id = msg_send![dot_view, layer];
            if dot_layer != nil {
                update_footer_dot_layer(dot_layer, info);
            }
            x += FOOTER_STREAMING_DOT_SIZE + FOOTER_LEFT_DOT_LABEL_GAP;
        }
    } else {
        remove_identified_subview(left_info_view, FOOTER_STATUS_DOT_ID);
    }

    // ── Model name label ──
    if info.model_name.is_empty() {
        remove_identified_subview(left_info_view, FOOTER_MODEL_LABEL_ID);
    } else {
        let label = ensure_footer_model_label(left_info_view, &info.model_name, text_color);
        if label != nil {
            let label_size: NSSize = msg_send![label, fittingSize];
            let label_y = ((bounds.size.height - label_size.height) / 2.0).round();
            let _: () = msg_send![
                label,
                setFrame: NSRect::new(
                    NSPoint::new(x, label_y),
                    NSSize::new(label_size.width, label_size.height),
                )
            ];
        }
    }
}

#[cfg(target_os = "macos")]
unsafe fn remove_identified_subview(parent: id, identifier: &str) {
    use objc::{msg_send, sel, sel_impl};

    let view = find_subview_by_identifier(parent, identifier);
    if view == nil {
        return;
    }
    let layer: id = msg_send![view, layer];
    if layer != nil {
        remove_active_dot_pulse_animation(layer);
    }
    let _: () = msg_send![view, removeFromSuperview];
}

#[cfg(target_os = "macos")]
unsafe fn ensure_footer_status_dot_view(left_info_view: id) -> id {
    use cocoa::foundation::{NSPoint, NSRect, NSSize};
    use objc::{class, msg_send, sel, sel_impl};

    let existing = find_subview_by_identifier(left_info_view, FOOTER_STATUS_DOT_ID);
    if existing != nil {
        return existing;
    }

    let dot_view: id = msg_send![class!(NSView), alloc];
    let dot_view: id = msg_send![
        dot_view,
        initWithFrame: NSRect::new(
            NSPoint::new(0.0, 0.0),
            NSSize::new(FOOTER_STREAMING_DOT_SIZE, FOOTER_STREAMING_DOT_SIZE),
        )
    ];
    if dot_view == nil {
        return nil;
    }

    let identifier = ns_string(FOOTER_STATUS_DOT_ID);
    if identifier != nil {
        let _: () = msg_send![dot_view, setIdentifier: identifier];
    }

    let layer: id = msg_send![class!(CALayer), layer];
    if layer != nil {
        let _: () = msg_send![layer, setMasksToBounds: NO];
        let _: () = msg_send![layer, setCornerRadius: FOOTER_STREAMING_DOT_SIZE / 2.0_f64];
        let _: () = msg_send![dot_view, setLayer: layer];
    }
    let _: () = msg_send![dot_view, setWantsLayer: YES];
    let _: () = msg_send![left_info_view, addSubview: dot_view];
    dot_view
}

#[cfg(target_os = "macos")]
unsafe fn ensure_footer_model_label(left_info_view: id, text: &str, text_color: id) -> id {
    use objc::{class, msg_send, sel, sel_impl};

    let font: id = msg_send![
        class!(NSFont),
        systemFontOfSize: crate::components::footer_chrome::FOOTER_HINT_FONT_SIZE_PX as f64
        weight: crate::components::footer_chrome::FOOTER_HINT_FONT_WEIGHT_APPKIT
    ];
    let label = find_subview_by_identifier(left_info_view, FOOTER_MODEL_LABEL_ID);
    if label != nil {
        let string_value = ns_string(text);
        if string_value != nil {
            let _: () = msg_send![label, setStringValue: string_value];
        }
        if font != nil {
            let _: () = msg_send![label, setFont: font];
        }
        if text_color != nil {
            let _: () = msg_send![label, setTextColor: text_color];
        }
        let _: () = msg_send![label, setAlignment: FOOTER_HINT_TEXT_ALIGN_LEFT];
        let _: () = msg_send![label, sizeToFit];
        return label;
    }

    let label = make_footer_hint_text_field(text, font, text_color, FOOTER_HINT_TEXT_ALIGN_LEFT);
    if label != nil {
        let identifier = ns_string(FOOTER_MODEL_LABEL_ID);
        if identifier != nil {
            let _: () = msg_send![label, setIdentifier: identifier];
        }
        let _: () = msg_send![left_info_view, addSubview: label];
    }
    label
}

#[cfg(target_os = "macos")]
unsafe fn update_footer_dot_layer(layer: id, info: &FooterLeftInfo) {
    use objc::{msg_send, sel, sel_impl};

    let theme = crate::theme::get_cached_theme();
    let dot_hex = footer_dot_hex(
        info.dot_status,
        &theme,
        info.prefer_accent_for_active_states,
    );
    let _: () = msg_send![layer, setCornerRadius: FOOTER_STREAMING_DOT_SIZE / 2.0_f64];

    let dot_ns = ns_color_from_hex_with_alpha(dot_hex, 1.0);
    if dot_ns != nil {
        let cg: id = msg_send![dot_ns, CGColor];
        if cg != nil {
            let _: () = msg_send![layer, setBackgroundColor: cg];
        }
    }

    let should_pulse = matches!(
        info.dot_status,
        FooterDotStatus::Streaming | FooterDotStatus::WaitingForPermission
    );
    if should_pulse {
        ensure_active_dot_pulse_animation(layer);
    } else {
        remove_active_dot_pulse_animation(layer);
        let _: () = msg_send![layer, setOpacity: 1.0_f32];
    }
}

/// Attach a repeating CoreAnimation color/opacity pulse for active work.
#[cfg(target_os = "macos")]
unsafe fn add_active_dot_pulse_animation(layer: id) {
    use objc::{class, msg_send, sel, sel_impl};

    // Use ease-in-ease-out for a smooth sine-like curve.
    let timing_name = ns_string("easeInEaseOut");
    let timing: id = if timing_name != nil {
        msg_send![
            class!(CAMediaTimingFunction),
            functionWithName: timing_name
        ]
    } else {
        nil
    };

    let duration: f64 = FOOTER_ACTIVE_DOT_HALF_CYCLE_SECONDS;

    // SAFETY: `layer` is a live CALayer. Keep the pulse visual-only; do not
    // scale the dot because size motion is distracting in the compact footer.
    let opacity_key_path = ns_string("opacity");
    if opacity_key_path != nil {
        let opacity_anim: id =
            msg_send![class!(CABasicAnimation), animationWithKeyPath: opacity_key_path];
        if opacity_anim != nil {
            let from_value: id =
                msg_send![class!(NSNumber), numberWithFloat: FOOTER_ACTIVE_DOT_MIN_OPACITY];
            let to_value: id = msg_send![class!(NSNumber), numberWithFloat: 1.0_f32];

            let _: () = msg_send![opacity_anim, setFromValue: from_value];
            let _: () = msg_send![opacity_anim, setToValue: to_value];
            let _: () = msg_send![opacity_anim, setDuration: duration];
            let _: () = msg_send![opacity_anim, setAutoreverses: YES];
            let _: () = msg_send![opacity_anim, setRepeatCount: f32::INFINITY];
            let _: () = msg_send![opacity_anim, setRemovedOnCompletion: NO];
            if timing != nil {
                let _: () = msg_send![opacity_anim, setTimingFunction: timing];
            }

            let anim_key = ns_string("pulseOpacity");
            if anim_key != nil {
                let _: () = msg_send![layer, addAnimation: opacity_anim forKey: anim_key];
            }
        }
    }
}

#[cfg(target_os = "macos")]
unsafe fn layer_has_animation(layer: id, key: &str) -> bool {
    use objc::{msg_send, sel, sel_impl};

    let key = ns_string(key);
    if key == nil {
        return false;
    }
    let animation: id = msg_send![layer, animationForKey: key];
    animation != nil
}

#[cfg(target_os = "macos")]
unsafe fn ensure_active_dot_pulse_animation(layer: id) {
    if layer == nil {
        return;
    }
    let has_opacity = layer_has_animation(layer, "pulseOpacity");
    if has_opacity {
        remove_active_dot_scale_animation(layer);
        return;
    }
    remove_active_dot_pulse_animation(layer);
    add_active_dot_pulse_animation(layer);
}

#[cfg(target_os = "macos")]
unsafe fn remove_active_dot_pulse_animation(layer: id) {
    use objc::{msg_send, sel, sel_impl};

    let opacity_key = ns_string("pulseOpacity");
    if opacity_key != nil {
        let _: () = msg_send![layer, removeAnimationForKey: opacity_key];
    }
    remove_active_dot_scale_animation(layer);
}

#[cfg(target_os = "macos")]
unsafe fn remove_active_dot_scale_animation(layer: id) {
    use objc::{msg_send, sel, sel_impl};

    let scale_key = ns_string("pulseScale");
    if scale_key != nil {
        let _: () = msg_send![layer, removeAnimationForKey: scale_key];
    }
}

#[cfg(target_os = "macos")]
unsafe fn layout_footer_hints(
    hints_view: id,
    text_color: id,
    buttons: &[FooterButtonConfig],
    theme: &crate::theme::Theme,
) {
    use cocoa::foundation::{NSPoint, NSRect, NSSize};
    use objc::{msg_send, sel, sel_impl};

    // Remove tracking areas from all buttons BEFORE removing them from the
    // view hierarchy. This prevents use-after-free crashes when AppKit tries
    // to deliver mouseEntered/mouseExited to a deallocated button owner.
    let subviews: id = msg_send![hints_view, subviews];
    if subviews != nil {
        let count: usize = msg_send![subviews, count];
        for index in (0..count).rev() {
            let container: id = msg_send![subviews, objectAtIndex: index];
            if container != nil {
                // Find and clean up tracking areas on any NSButton inside this container.
                let container_subs: id = msg_send![container, subviews];
                if container_subs != nil {
                    let sub_count: usize = msg_send![container_subs, count];
                    for si in 0..sub_count {
                        let child: id = msg_send![container_subs, objectAtIndex: si];
                        if child != nil {
                            let is_button: cocoa::base::BOOL =
                                msg_send![child, isKindOfClass: objc::class!(NSButton)];
                            if is_button == YES {
                                let areas: id = msg_send![child, trackingAreas];
                                if areas != nil {
                                    let ac: usize = msg_send![areas, count];
                                    for ai in (0..ac).rev() {
                                        let area: id = msg_send![areas, objectAtIndex: ai];
                                        let _: () = msg_send![child, removeTrackingArea: area];
                                    }
                                }
                            }
                        }
                    }
                }
                let _: () = msg_send![container, removeFromSuperview];
            }
        }
    }

    let hints_bounds: NSRect = msg_send![hints_view, bounds];
    let font: id = msg_send![
        objc::class!(NSFont),
        systemFontOfSize: crate::components::footer_chrome::FOOTER_HINT_FONT_SIZE_PX as f64
        weight: crate::components::footer_chrome::FOOTER_HINT_FONT_WEIGHT_APPKIT
    ];

    let mut items = Vec::new();
    let mut trailing_item_width = 0.0_f64;
    for button_cfg in buttons {
        let max_item_width =
            footer_hint_max_item_width(button_cfg.action, hints_bounds.size.width, buttons);
        let item = make_footer_hint_item(button_cfg, font, text_color, max_item_width, theme);
        if item == nil {
            continue;
        }
        let item_frame: NSRect = msg_send![item, frame];
        let target_width = footer_hint_slot_width(button_cfg.action).max(item_frame.size.width);
        let left_pinned = is_footer_left_pinned_mic_button(button_cfg);
        if !left_pinned {
            if trailing_item_width > 0.0 {
                trailing_item_width += FOOTER_HINT_ITEM_GAP;
            }
            trailing_item_width += target_width;
        }
        items.push((
            item,
            target_width,
            button_cfg.action,
            button_cfg.enabled,
            left_pinned,
        ));
    }

    let left_pinned_width = items
        .iter()
        .filter(|(_, _, _, _, left_pinned)| *left_pinned)
        .map(|(_, target_width, _, _, _)| *target_width + FOOTER_HINT_ITEM_GAP)
        .sum::<f64>();
    let mut trailing_x = (hints_bounds.size.width - trailing_item_width)
        .max(left_pinned_width)
        .max(0.0);
    for (item, target_width, action, enabled, left_pinned) in items {
        let x = if left_pinned { 0.0 } else { trailing_x };
        let item_y = crate::components::footer_chrome::FOOTER_BUTTON_VERTICAL_INSET_PX as f64;
        let item_height =
            crate::components::footer_chrome::footer_button_height(hints_bounds.size.height as f32)
                as f64;
        let frame = NSRect::new(
            NSPoint::new(x, item_y),
            NSSize::new(target_width, item_height),
        );
        tracing::debug!(
            target: "script_kit::footer_popup",
            event = "native_footer_item_layout",
            action = footer_action_key(action),
            x,
            y = item_y,
            width = target_width,
            height = item_height,
            enabled,
            "Laid out native footer item slot"
        );
        let _: () = msg_send![item, setFrame: frame];
        let _: () = msg_send![hints_view, addSubview: item];
        if !left_pinned {
            trailing_x += target_width + FOOTER_HINT_ITEM_GAP;
        }
    }
}

#[cfg(target_os = "macos")]
fn is_footer_left_pinned_mic_button(button_cfg: &FooterButtonConfig) -> bool {
    matches!(button_cfg.action, FooterAction::Ai)
        && button_cfg.key.as_ref() == crate::components::footer_chrome::FOOTER_MIC_ICON_TOKEN
}

#[cfg(target_os = "macos")]
fn footer_hint_max_item_width(
    action: FooterAction,
    hints_width: f64,
    buttons: &[FooterButtonConfig],
) -> Option<f64> {
    let mic_button = buttons
        .iter()
        .find(|button| is_footer_left_pinned_mic_button(button));
    if let Some(mic_button) = mic_button {
        if matches!(action, FooterAction::Ai) && is_footer_left_pinned_mic_button(mic_button) {
            let trailing_reserved_width = buttons
                .iter()
                .filter(|button| !is_footer_left_pinned_mic_button(button))
                .map(|button| footer_hint_slot_width(button.action))
                .sum::<f64>()
                + buttons.len().saturating_sub(1) as f64 * FOOTER_HINT_ITEM_GAP;
            return Some(
                (hints_width - trailing_reserved_width)
                    .clamp(FOOTER_AI_SLOT_WIDTH, 220.0)
                    .round(),
            );
        }
    }

    if !matches!(action, FooterAction::Run) {
        return None;
    }

    let gap_width = buttons.len().saturating_sub(1) as f64 * FOOTER_HINT_ITEM_GAP;
    let reserved_width = buttons
        .iter()
        .filter(|button| !matches!(button.action, FooterAction::Run))
        .map(|button| footer_hint_slot_width(button.action))
        .sum::<f64>()
        + gap_width;

    Some(
        (hints_width - reserved_width)
            .clamp(FOOTER_RUN_SLOT_MIN_WIDTH, FOOTER_RUN_SLOT_MAX_WIDTH)
            .round(),
    )
}

#[cfg(target_os = "macos")]
fn footer_hint_slot_width(action: FooterAction) -> f64 {
    match action {
        FooterAction::Run => FOOTER_RUN_SLOT_MIN_WIDTH,
        FooterAction::Actions => FOOTER_ACTIONS_SLOT_WIDTH,
        FooterAction::Ai => FOOTER_AI_SLOT_WIDTH,
        FooterAction::Apply => FOOTER_APPLY_SLOT_WIDTH,
        FooterAction::Close => FOOTER_CLOSE_SLOT_WIDTH,
        FooterAction::Stop => FOOTER_STOP_SLOT_WIDTH,
        FooterAction::PasteResponse => FOOTER_PASTE_RESPONSE_SLOT_WIDTH,
    }
}

fn footer_hint_content_layout(
    action: FooterAction,
    item_width: f64,
    label_width: f64,
    key_width: f64,
) -> (f64, f64, f64) {
    let has_label = label_width > 0.0;
    let has_key = key_width > 0.0;
    let gap_width = if has_label && has_key {
        FOOTER_HINT_KEY_LABEL_GAP
    } else {
        0.0
    };
    let content_width = label_width + gap_width + key_width;

    if matches!(action, FooterAction::Run) {
        let key_x = (item_width - FOOTER_RUN_HINT_PADDING_X - key_width).round();
        let label_x = (key_x - gap_width - label_width).max(0.0).round();
        return (label_x, key_x, content_width);
    }

    let label_x = ((item_width - content_width) / 2.0).max(0.0).round();
    let key_x = (label_x + label_width + gap_width).round();
    (label_x, key_x, content_width)
}

fn footer_hint_content_layout_for_button(
    button_cfg: &FooterButtonConfig,
    item_width: f64,
    label_width: f64,
    key_width: f64,
) -> (f64, f64, f64) {
    if is_footer_left_pinned_mic_button(button_cfg) {
        let gap_width = if label_width > 0.0 && key_width > 0.0 {
            FOOTER_HINT_KEY_LABEL_GAP
        } else {
            0.0
        };
        let key_x = FOOTER_HINT_PADDING_X.round();
        let label_x = (key_x + key_width + gap_width).round();
        return (label_x, key_x, label_width + gap_width + key_width);
    }

    footer_hint_content_layout(button_cfg.action, item_width, label_width, key_width)
}

#[cfg(target_os = "macos")]
fn footer_selected_background_rgba(
    action: FooterAction,
    chrome: &crate::theme::AppChromeColors,
) -> u32 {
    if matches!(action, FooterAction::Actions) {
        chrome.hover_rgba
    } else {
        chrome.selection_rgba
    }
}

#[cfg(target_os = "macos")]
fn footer_selected_background_rgba_for_actions_button(
    is_actions_button: cocoa::base::BOOL,
    chrome: &crate::theme::AppChromeColors,
) -> u32 {
    if is_actions_button == YES {
        footer_selected_background_rgba(FooterAction::Actions, chrome)
    } else {
        chrome.selection_rgba
    }
}

#[cfg(target_os = "macos")]
fn footer_hint_label_widths(
    natural_label_width: f64,
    label_padding_x: f64,
    label_chip_height: f64,
    max_item_width: Option<f64>,
    keys_view_width: f64,
    edge_padding_x: f64,
) -> (f64, f64) {
    let max_label_chip_width = max_item_width.map(|max_width| {
        (max_width - (edge_padding_x * 2.0) - FOOTER_HINT_KEY_LABEL_GAP - keys_view_width)
            .max(label_chip_height)
    });
    let label_chip_width = (natural_label_width + label_padding_x * 2.0)
        .max(label_chip_height)
        .min(max_label_chip_width.unwrap_or(f64::MAX));
    let label_text_width = (label_chip_width - label_padding_x * 2.0).max(0.0);
    (label_chip_width, label_text_width)
}

#[cfg(target_os = "macos")]
const FOOTER_MIC_ICON_SVG: &str =
    include_str!("../vendor/gpui-component/crates/assets/assets/icons/mic.svg");

#[cfg(target_os = "macos")]
fn footer_mic_icon_png_data() -> Option<&'static [u8]> {
    static PNG_DATA: std::sync::OnceLock<Option<Vec<u8>>> = std::sync::OnceLock::new();
    PNG_DATA
        .get_or_init(|| {
            let svg = FOOTER_MIC_ICON_SVG.replace("currentColor", "white");
            let opts = usvg::Options::default();
            let tree = usvg::Tree::from_str(&svg, &opts).ok()?;
            let size = 32_u32;
            let mut pixmap = tiny_skia::Pixmap::new(size, size)?;
            let svg_size = tree.size();
            let scale = (size as f32 / svg_size.width()).min(size as f32 / svg_size.height());
            resvg::render(
                &tree,
                tiny_skia::Transform::from_scale(scale, scale),
                &mut pixmap.as_mut(),
            );
            let rgba = pixmap.take();
            if !rgba.chunks_exact(4).any(|pixel| pixel[3] != 0) {
                return None;
            }
            let image = image::RgbaImage::from_raw(size, size, rgba)?;
            let mut cursor = std::io::Cursor::new(Vec::new());
            image::DynamicImage::ImageRgba8(image)
                .write_to(&mut cursor, image::ImageFormat::Png)
                .ok()?;
            Some(cursor.into_inner())
        })
        .as_deref()
}

#[cfg(target_os = "macos")]
unsafe fn footer_mic_icon_image() -> id {
    use objc::{class, msg_send, sel, sel_impl};

    let Some(png_data) = footer_mic_icon_png_data() else {
        return nil;
    };
    let data: id = msg_send![
        class!(NSData),
        dataWithBytes: png_data.as_ptr()
        length: png_data.len()
    ];
    if data == nil {
        return nil;
    }
    let image: id = msg_send![class!(NSImage), alloc];
    let image: id = msg_send![image, initWithData: data];
    if image != nil {
        let _: () = msg_send![image, setTemplate: YES];
    }
    image
}

#[cfg(target_os = "macos")]
unsafe fn make_footer_hint_item(
    button_cfg: &FooterButtonConfig,
    font: id,
    text_color: id,
    max_item_width: Option<f64>,
    theme: &crate::theme::Theme,
) -> id {
    use cocoa::foundation::{NSPoint, NSRect, NSSize};
    use objc::{class, msg_send, sel, sel_impl};

    let item_height =
        crate::components::footer_chrome::footer_button_height(footer_height() as f32) as f64;

    let container: id = msg_send![class!(NSView), alloc];
    let container: id = msg_send![
        container,
        initWithFrame: NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(0.0, item_height))
    ];
    if container == nil {
        return nil;
    }

    let has_label = !button_cfg.label.as_ref().is_empty();
    let label_field = if has_label {
        make_footer_hint_text_field(
            button_cfg.label.as_ref(),
            font,
            text_color,
            FOOTER_HINT_TEXT_ALIGN_RIGHT,
        )
    } else {
        nil
    };
    if has_label && label_field == nil {
        return nil;
    }

    let chrome = crate::theme::AppChromeColors::from_theme(theme);
    let edge_padding_x = if matches!(button_cfg.action, FooterAction::Run) {
        FOOTER_RUN_HINT_PADDING_X
    } else {
        FOOTER_HINT_PADDING_X
    };
    let keycap_border_color = ns_color_from_hex_with_alpha(
        theme.colors.text.primary,
        crate::components::footer_chrome::footer_keycap_border_alpha(theme, button_cfg.selected)
            as f64,
    );
    let labelcap_border_color = ns_color_from_hex_with_alpha(
        theme.colors.text.primary,
        crate::components::footer_chrome::footer_keycap_border_alpha(theme, button_cfg.selected)
            as f64,
    );
    let key_font: id = font;

    let shortcut_keys =
        crate::components::footer_chrome::split_footer_shortcut(button_cfg.key.as_ref());

    let keys_view: id = msg_send![class!(NSView), alloc];
    let keys_view: id = msg_send![
        keys_view,
        initWithFrame: NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(0.0, item_height))
    ];
    if keys_view == nil {
        return nil;
    }
    let _: () = msg_send![keys_view, setWantsLayer: YES];

    let mut keys_view_width = 0.0_f64;
    let key_gap = 3.0_f64;

    for (i, key_str) in shortcut_keys.iter().enumerate() {
        let chip_view: id = msg_send![class!(NSView), alloc];
        let chip_view: id = msg_send![
            chip_view,
            initWithFrame: NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(0.0, 0.0))
        ];
        if chip_view == nil {
            continue;
        }

        let _: () = msg_send![chip_view, setWantsLayer: YES];
        let chip_layer: id = msg_send![chip_view, layer];
        if chip_layer != nil {
            let _: () = msg_send![
                chip_layer,
                setCornerRadius: crate::components::footer_chrome::FOOTER_KEYCAP_RADIUS_PX as f64
            ];
            let _: () = msg_send![chip_layer, setBorderWidth: 1.0_f64];
            if keycap_border_color != nil {
                let cg_border: id = msg_send![keycap_border_color, CGColor];
                if cg_border != nil {
                    let _: () = msg_send![chip_layer, setBorderColor: cg_border];
                }
            }
        }

        let is_mic_icon = key_str == crate::components::footer_chrome::FOOTER_MIC_ICON_TOKEN;
        let chip_padding_x = crate::components::footer_chrome::FOOTER_KEYCAP_PADDING_X_PX as f64;
        let chip_height = crate::components::footer_chrome::FOOTER_KEYCAP_HEIGHT_PX as f64;
        let (glyph_view, glyph_size) = if is_mic_icon {
            let image = footer_mic_icon_image();
            if image == nil {
                continue;
            }
            let image_view: id = msg_send![class!(NSImageView), alloc];
            let image_view: id = msg_send![
                image_view,
                initWithFrame: NSRect::new(
                    NSPoint::new(0.0, 0.0),
                    NSSize::new(13.0_f64, 13.0_f64)
                )
            ];
            if image_view == nil {
                continue;
            }
            let _: () = msg_send![image_view, setImage: image];
            let _: () = msg_send![image_view, setContentTintColor: text_color];
            let _: () = msg_send![image_view, setAlphaValue: 1.0_f64];
            let _: () = msg_send![image_view, setImageScaling: 0usize];
            (image_view, NSSize::new(13.0_f64, 13.0_f64))
        } else {
            let glyph_field = make_footer_hint_text_field(
                key_str,
                key_font,
                text_color,
                FOOTER_HINT_TEXT_ALIGN_LEFT,
            );
            if glyph_field == nil {
                continue;
            }
            let glyph_size: NSSize = msg_send![glyph_field, fittingSize];
            (glyph_field, glyph_size)
        };
        let chip_width = (glyph_size.width + chip_padding_x * 2.0).max(chip_height);

        let glyph_x = ((chip_width - glyph_size.width) / 2.0).round();
        let glyph_y = crate::components::footer_chrome::footer_appkit_glyph_y(
            key_str,
            chip_height,
            glyph_size.height,
        );

        let _: () = msg_send![
            glyph_view,
            setFrame: NSRect::new(
                NSPoint::new(glyph_x, glyph_y),
                NSSize::new(glyph_size.width, glyph_size.height)
            )
        ];
        let _: () = msg_send![chip_view, addSubview: glyph_view];

        let chip_y = ((item_height - chip_height) / 2.0).round();
        let chip_x = keys_view_width;

        let _: () = msg_send![
            chip_view,
            setFrame: NSRect::new(
                NSPoint::new(chip_x, chip_y),
                NSSize::new(chip_width, chip_height)
            )
        ];

        let _: () = msg_send![keys_view, addSubview: chip_view];

        keys_view_width += chip_width;
        if i < shortcut_keys.len() - 1 {
            keys_view_width += key_gap;
        }
    }

    let _: () = msg_send![
        keys_view,
        setFrame: NSRect::new(
            NSPoint::new(0.0, 0.0),
            NSSize::new(keys_view_width, item_height)
        )
    ];

    let label_padding_x = crate::components::footer_chrome::FOOTER_KEYCAP_PADDING_X_PX as f64;
    let label_chip_height = crate::components::footer_chrome::FOOTER_KEYCAP_HEIGHT_PX as f64;
    let (label_view, label_chip_width, _label_text_width) = if has_label {
        let label_size: NSSize = msg_send![label_field, fittingSize];
        let (label_chip_width, label_text_width) = footer_hint_label_widths(
            label_size.width,
            label_padding_x,
            label_chip_height,
            max_item_width,
            keys_view_width,
            edge_padding_x,
        );
        let label_view: id = msg_send![class!(NSView), alloc];
        let label_view: id = msg_send![
            label_view,
            initWithFrame: NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(label_chip_width, label_chip_height))
        ];
        if label_view == nil {
            return nil;
        }
        let _: () = msg_send![label_view, setWantsLayer: YES];
        let label_layer: id = msg_send![label_view, layer];
        if label_layer != nil {
            let _: () = msg_send![
                label_layer,
                setCornerRadius: crate::components::footer_chrome::FOOTER_KEYCAP_RADIUS_PX as f64
            ];
            let _: () = msg_send![label_layer, setBorderWidth: 1.0_f64];
            if labelcap_border_color != nil {
                let cg_border: id = msg_send![labelcap_border_color, CGColor];
                if cg_border != nil {
                    let _: () = msg_send![label_layer, setBorderColor: cg_border];
                }
            }
        }

        let label_field_x = ((label_chip_width - label_text_width) / 2.0).round();
        let label_field_y = ((label_chip_height - label_size.height) / 2.0).round();
        let _: () = msg_send![
            label_field,
            setFrame: NSRect::new(
                NSPoint::new(label_field_x, label_field_y),
                NSSize::new(label_text_width, label_size.height)
            )
        ];
        let _: () = msg_send![label_view, addSubview: label_field];
        (label_view, label_chip_width, label_text_width)
    } else {
        (nil, 0.0_f64, 0.0_f64)
    };

    let gap_width = if has_label && keys_view_width > 0.0 {
        FOOTER_HINT_KEY_LABEL_GAP
    } else {
        0.0
    };
    let legacy_extra_padding = if matches!(button_cfg.action, FooterAction::Run) {
        0.0
    } else {
        12.0
    };
    let min_content_width = keys_view_width
        + label_chip_width
        + gap_width
        + (edge_padding_x * 2.0)
        + legacy_extra_padding;
    let content_width = label_chip_width + gap_width + keys_view_width;
    let intrinsic_width = content_width + (edge_padding_x * 2.0);
    let mut item_width = footer_hint_slot_width(button_cfg.action)
        .max(min_content_width)
        .max(intrinsic_width);
    if let Some(max_item_width) = max_item_width {
        item_width = item_width.min(max_item_width.max(min_content_width));
    }
    let label_y = ((item_height - label_chip_height) / 2.0).round();
    let (label_x, key_x, _) = footer_hint_content_layout_for_button(
        button_cfg,
        item_width,
        label_chip_width,
        keys_view_width,
    );

    if has_label && label_view != nil {
        let _: () = msg_send![
            label_view,
            setFrame: NSRect::new(
                NSPoint::new(label_x, label_y),
                NSSize::new(label_chip_width, label_chip_height)
            )
        ];
    }
    let _: () = msg_send![
        keys_view,
        setFrame: NSRect::new(
            NSPoint::new(key_x, 0.0),
            NSSize::new(keys_view_width, item_height)
        )
    ];
    let _: () = msg_send![container, setWantsLayer: YES];
    let container_layer: id = msg_send![container, layer];
    if container_layer != nil {
        let _: () = msg_send![container_layer, setCornerRadius: FOOTER_HINT_RADIUS];
        if button_cfg.selected {
            let selected_ns: id =
                ns_color_from_rgba(footer_selected_background_rgba(button_cfg.action, &chrome));
            if selected_ns != nil {
                let cg: id = msg_send![selected_ns, CGColor];
                if cg != nil {
                    let _: () = msg_send![container_layer, setBackgroundColor: cg];
                }
            }
        }
    }

    let button: id = msg_send![footer_button_class(), alloc];
    let button: id = msg_send![
        button,
        initWithFrame: NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(item_width, item_height))
    ];
    if button != nil {
        let empty_title = ns_string("");
        if empty_title != nil {
            let _: () = msg_send![button, setTitle: empty_title];
        }
        let button_id = ns_string(&format!(
            "{}{}",
            FOOTER_HINT_BUTTON_ID_PREFIX,
            footer_action_key(button_cfg.action)
        ));
        if button_id != nil {
            let _: () = msg_send![button, setIdentifier: button_id];
        }
        let _: () = msg_send![button, setBordered: NO];
        let _: () = msg_send![button, setBezelStyle: 0usize];
        let _: () = msg_send![button, setButtonType: 0usize];
        let _: () = msg_send![button, setTransparent: YES];
        let _: () = msg_send![button, setEnabled: if button_cfg.enabled { YES } else { NO }];
        let _: () = msg_send![button, setTarget: footer_action_target()];
        let _: () = msg_send![button, setAction: footer_action_selector(button_cfg.action)];

        // Store button state for hover/cursor behavior and selected restoration.
        let is_actions = matches!(button_cfg.action, FooterAction::Actions);
        if let Some(obj) = button.as_mut() {
            obj.set_ivar::<cocoa::base::BOOL>(
                "_isActionsButton",
                if is_actions { YES } else { NO },
            );
            obj.set_ivar::<cocoa::base::BOOL>(
                "_selected",
                if button_cfg.selected { YES } else { NO },
            );
            obj.set_ivar::<cocoa::base::BOOL>(
                "_enabled",
                if button_cfg.enabled { YES } else { NO },
            );
        }
    }

    if has_label && label_view != nil {
        let _: () = msg_send![container, addSubview: label_view];
    }
    let _: () = msg_send![container, addSubview: keys_view];
    if button != nil {
        let _: () = msg_send![container, addSubview: button];
    }
    let _: () = msg_send![
        container,
        setFrame: NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(item_width, item_height))
    ];

    container
}

#[cfg(target_os = "macos")]
unsafe fn make_footer_hint_text_field(
    text: &str,
    font: id,
    text_color: id,
    alignment: usize,
) -> id {
    use objc::{class, msg_send, sel, sel_impl};

    let field: id = msg_send![class!(NSTextField), alloc];
    let field: id = msg_send![field, init];
    if field == nil {
        return nil;
    }

    let string_value = ns_string(text);
    if string_value == nil {
        return nil;
    }

    let _: () = msg_send![field, setStringValue: string_value];
    let _: () = msg_send![field, setBezeled: NO];
    let _: () = msg_send![field, setBordered: NO];
    let _: () = msg_send![field, setDrawsBackground: NO];
    let _: () = msg_send![field, setEditable: NO];
    let _: () = msg_send![field, setSelectable: NO];
    if font != nil {
        let _: () = msg_send![field, setFont: font];
    }
    if text_color != nil {
        let _: () = msg_send![field, setTextColor: text_color];
    }
    let _: () = msg_send![field, setAlignment: alignment];
    let _: () = msg_send![field, setLineBreakMode: 4usize];
    let _: () = msg_send![field, setUsesSingleLineMode: YES];
    let _: () = msg_send![field, sizeToFit];
    field
}

#[cfg(test)]
mod footer_layout_tests {
    use super::{
        footer_active_dot_hex, footer_dot_hex, footer_hint_content_layout,
        footer_hint_label_widths, footer_hint_max_item_width, footer_hint_slot_width,
        footer_overlay_button_full_width_px, footer_selected_background_rgba, FooterAction,
        FooterButtonConfig, FooterDotStatus, FOOTER_HINT_KEY_LABEL_GAP, FOOTER_HINT_PADDING_X,
        FOOTER_RUN_HINT_PADDING_X,
    };

    #[test]
    fn footer_hint_slot_widths_are_stable_per_action() {
        assert_eq!(footer_hint_slot_width(FooterAction::Run), 92.0);
        assert_eq!(footer_hint_slot_width(FooterAction::Actions), 92.0);
        assert_eq!(footer_hint_slot_width(FooterAction::Ai), 52.0);
        assert_eq!(footer_hint_slot_width(FooterAction::Stop), 76.0);
        assert_eq!(footer_hint_slot_width(FooterAction::PasteResponse), 140.0);
    }

    #[test]
    fn run_slot_remains_at_least_as_wide_as_actions_and_wider_than_ai() {
        assert!(
            footer_hint_slot_width(FooterAction::Run)
                >= footer_hint_slot_width(FooterAction::Actions)
        );
        assert!(
            footer_hint_slot_width(FooterAction::Run) > footer_hint_slot_width(FooterAction::Ai)
        );
    }

    #[test]
    fn footer_hint_content_group_is_centered_within_slot() {
        let item_width = 92.0;
        let label_width = 34.0;
        let key_width = 18.0;

        let (label_x, key_x, content_width) =
            footer_hint_content_layout(FooterAction::Actions, item_width, label_width, key_width);
        let left_padding = label_x;
        let right_padding = item_width - (key_x + key_width);

        assert_eq!(content_width, label_width + 2.0 + key_width);
        assert!((left_padding - right_padding).abs() <= 1.0);
    }

    #[test]
    fn run_hint_keeps_key_glyph_anchored_to_trailing_padding() {
        let short = footer_hint_content_layout(FooterAction::Run, 92.0, 20.0, 18.0);
        let long = footer_hint_content_layout(FooterAction::Run, 140.0, 64.0, 18.0);

        assert_eq!(short.1, 68.0);
        assert_eq!(long.1, 116.0);
        assert_eq!(92.0 - (short.1 + 18.0), 6.0);
        assert_eq!(140.0 - (long.1 + 18.0), 6.0);
    }

    #[test]
    fn run_hint_native_layout_can_balance_short_label_padding() {
        let label_width = 26.0;
        let key_width = 20.0;
        let item_width =
            label_width + FOOTER_HINT_KEY_LABEL_GAP + key_width + FOOTER_RUN_HINT_PADDING_X * 2.0;
        let (label_x, key_x, _) =
            footer_hint_content_layout(FooterAction::Run, item_width, label_width, key_width);

        assert_eq!(label_x, FOOTER_RUN_HINT_PADDING_X);
        assert_eq!(item_width - (key_x + key_width), FOOTER_RUN_HINT_PADDING_X);
    }

    #[test]
    fn actions_selected_background_uses_hover_opacity() {
        let theme = crate::theme::Theme::dark_default();
        let chrome = crate::theme::AppChromeColors::from_theme(&theme);

        assert_eq!(
            footer_selected_background_rgba(FooterAction::Actions, &chrome),
            chrome.hover_rgba
        );
        assert_eq!(
            footer_selected_background_rgba(FooterAction::Run, &chrome),
            chrome.selection_rgba
        );
        assert_ne!(chrome.hover_rgba, chrome.selection_rgba);
    }

    #[test]
    fn run_hint_width_is_capped_to_stable_slot() {
        let buttons = vec![
            FooterButtonConfig::new(
                FooterAction::Run,
                "↵",
                "Open Screen Recording Permission Assistant",
            ),
            FooterButtonConfig::new(FooterAction::Ai, "⌘↵", "Agent"),
            FooterButtonConfig::new(FooterAction::Actions, "⌘K", "Actions"),
        ];

        assert_eq!(
            footer_hint_max_item_width(FooterAction::Run, 480.0, &buttons),
            Some(172.0)
        );
        assert_eq!(
            footer_hint_max_item_width(FooterAction::Run, 640.0, &buttons),
            Some(172.0)
        );
        assert_eq!(
            footer_hint_max_item_width(FooterAction::Run, 120.0, &buttons),
            Some(92.0)
        );
        assert_eq!(
            footer_hint_max_item_width(FooterAction::Ai, 480.0, &buttons),
            None
        );
    }

    #[test]
    fn run_command_label_gets_natural_width_before_truncation() {
        let run_command = FooterButtonConfig::new(FooterAction::Run, "↵", "Run Command");

        assert!(footer_overlay_button_full_width_px(&run_command) > 112.0);
        assert!(
            footer_overlay_button_full_width_px(&run_command)
                < crate::components::footer_chrome::FOOTER_RUN_SLOT_MAX_WIDTH_PX
        );
    }

    #[test]
    fn run_hint_label_text_width_truncates_inside_remaining_slot() {
        let (chip_width, text_width) =
            footer_hint_label_widths(360.0, 5.0, 18.0, Some(180.0), 20.0, FOOTER_HINT_PADDING_X);

        assert_eq!(chip_width, 154.0);
        assert_eq!(text_width, 144.0);
        assert!(text_width < 360.0);
    }

    #[test]
    fn footer_buttons_keep_two_pixel_vertical_inset() {
        assert_eq!(
            crate::components::footer_chrome::FOOTER_BUTTON_VERTICAL_INSET_PX,
            2.0
        );
        assert_eq!(
            crate::components::footer_chrome::footer_button_height(32.0),
            28.0
        );
    }

    #[test]
    fn active_dot_prefers_the_most_contrasting_theme_color() {
        let mut theme = crate::theme::Theme::dark_default();
        theme.colors.background.main = 0x101114;
        theme.colors.accent.selected = 0x3a4250;
        theme.colors.text.primary = 0xf5f7fa;

        assert_eq!(
            footer_active_dot_hex(&theme, false),
            theme.colors.text.primary
        );

        theme.colors.accent.selected = 0xffc600;
        theme.colors.text.primary = 0x8892a0;
        assert_eq!(
            footer_active_dot_hex(&theme, false),
            theme.colors.accent.selected
        );
    }

    #[test]
    fn active_dot_can_force_accent_for_acp_states() {
        let mut theme = crate::theme::Theme::dark_default();
        theme.colors.background.main = 0x101114;
        theme.colors.accent.selected = 0x3a4250;
        theme.colors.text.primary = 0xf5f7fa;

        assert_eq!(
            footer_active_dot_hex(&theme, true),
            theme.colors.accent.selected
        );
    }

    #[test]
    fn footer_dot_colors_follow_theme_tokens() {
        let mut theme = crate::theme::Theme::dark_default();
        theme.colors.text.secondary = 0x778899;
        theme.colors.ui.error = 0xaa3344;

        assert_eq!(
            footer_dot_hex(FooterDotStatus::Idle, &theme, false),
            theme.colors.text.secondary
        );
        assert_eq!(
            footer_dot_hex(FooterDotStatus::Error, &theme, false),
            theme.colors.ui.error
        );
    }
}

#[cfg(target_os = "macos")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FooterWindowKind {
    Main,
    Dictation,
    AcpChat,
}

#[cfg(target_os = "macos")]
fn send_footer_action_from_sender(sender: id, action: FooterAction) {
    let title = unsafe { footer_sender_window_title(sender) };
    let window_kind = if let Some(ref t) = title {
        if t.contains("Script Kit Dictation") {
            FooterWindowKind::Dictation
        } else if t.contains("Script Kit Agent Chat") {
            FooterWindowKind::AcpChat
        } else {
            FooterWindowKind::Main
        }
    } else {
        FooterWindowKind::Main
    };
    send_footer_action_to_channel_v2(action, window_kind);
}

#[cfg(target_os = "macos")]
fn send_footer_action_to_channel_v2(action: FooterAction, window_kind: FooterWindowKind) {
    let action_name = footer_action_key(action);
    tracing::info!(
        target: "script_kit::footer_popup",
        event = "native_footer_action_enqueued",
        action = action_name,
        ?window_kind,
        "Enqueued native footer action"
    );
    let (tx, _) = match window_kind {
        FooterWindowKind::Dictation => dictation_footer_action_channel(),
        FooterWindowKind::AcpChat => acp_footer_action_channel(),
        FooterWindowKind::Main => footer_action_channel(),
    };
    if let Err(error) = tx.try_send(action) {
        tracing::warn!(
            target: "script_kit::footer_popup",
            event = "native_footer_action_enqueue_failed",
            action = action_name,
            %error,
            "Failed to enqueue footer action"
        );
    }
}

fn send_footer_action_to_channel(action: FooterAction, dictation_footer: bool) {
    #[cfg(target_os = "macos")]
    {
        let window_kind = if dictation_footer {
            FooterWindowKind::Dictation
        } else {
            FooterWindowKind::Main
        };
        send_footer_action_to_channel_v2(action, window_kind);
    }
    #[cfg(not(target_os = "macos"))]
    {
        let (tx, _) = if dictation_footer {
            dictation_footer_action_channel()
        } else {
            footer_action_channel()
        };
        if let Err(error) = tx.try_send(action) {
            tracing::warn!(
                target: "script_kit::footer_popup",
                event = "native_footer_action_enqueue_failed",
                action = footer_action_key(action),
                %error,
                "Failed to enqueue footer action"
            );
        }
    }
}

#[cfg(target_os = "macos")]
unsafe fn footer_sender_window_title(sender: id) -> Option<String> {
    use objc::{msg_send, sel, sel_impl};
    use std::ffi::CStr;

    if sender == nil {
        return None;
    }

    let ns_window: id = msg_send![sender, window];
    if ns_window == nil {
        return None;
    }

    let title: id = msg_send![ns_window, title];
    if title == nil {
        return None;
    }

    let utf8: *const std::os::raw::c_char = msg_send![title, UTF8String];
    if utf8.is_null() {
        return None;
    }

    Some(CStr::from_ptr(utf8).to_string_lossy().into_owned())
}

fn footer_action_key(action: FooterAction) -> &'static str {
    match action {
        FooterAction::Run => "run",
        FooterAction::Actions => "actions",
        FooterAction::Ai => "ai",
        FooterAction::Apply => "apply",
        FooterAction::Close => "close",
        FooterAction::Stop => "stop",
        FooterAction::PasteResponse => "pasteResponse",
    }
}

#[cfg(target_os = "macos")]
fn ns_string(text: &str) -> id {
    use objc::{class, msg_send, sel, sel_impl};

    let Ok(c_string) = std::ffi::CString::new(text) else {
        return nil;
    };

    // SAFETY: The CString is NUL-terminated and lives for the duration of the call.
    unsafe { msg_send![class!(NSString), stringWithUTF8String: c_string.as_ptr()] }
}

#[cfg(target_os = "macos")]
unsafe fn ns_color_from_rgba(rgba: u32) -> id {
    use objc::{class, msg_send, sel, sel_impl};

    let red = ((rgba >> 24) & 0xFF) as f64 / 255.0;
    let green = ((rgba >> 16) & 0xFF) as f64 / 255.0;
    let blue = ((rgba >> 8) & 0xFF) as f64 / 255.0;
    let alpha = (rgba & 0xFF) as f64 / 255.0;

    // SAFETY: Standard AppKit color construction on the main thread.
    msg_send![
        class!(NSColor),
        colorWithSRGBRed: red
        green: green
        blue: blue
        alpha: alpha
    ]
}

#[cfg(target_os = "macos")]
unsafe fn ns_color_from_hex_with_alpha(hex: u32, alpha: f64) -> id {
    use objc::{class, msg_send, sel, sel_impl};

    let red = ((hex >> 16) & 0xFF) as f64 / 255.0;
    let green = ((hex >> 8) & 0xFF) as f64 / 255.0;
    let blue = (hex & 0xFF) as f64 / 255.0;

    // SAFETY: Standard AppKit color construction on the main thread.
    msg_send![
        class!(NSColor),
        colorWithSRGBRed: red
        green: green
        blue: blue
        alpha: alpha
    ]
}

#[cfg(target_os = "macos")]
fn footer_passthrough_view_class() -> *const objc::runtime::Class {
    use std::sync::OnceLock;

    use objc::declare::ClassDecl;
    use objc::runtime::{Object, Sel};
    use objc::{class, sel, sel_impl};

    static CLASS: OnceLock<usize> = OnceLock::new();

    *CLASS.get_or_init(|| unsafe {
        let superclass = class!(NSView);
        let Some(mut decl) = ClassDecl::new("ScriptKitFooterPassthroughView", superclass) else {
            return class!(NSView) as *const _ as usize;
        };
        decl.add_method(
            sel!(hitTest:),
            footer_passthrough_hit_test
                as extern "C" fn(&Object, Sel, cocoa::foundation::NSPoint) -> id,
        );
        decl.register() as *const _ as usize
    }) as *const objc::runtime::Class
}

#[cfg(target_os = "macos")]
extern "C" fn footer_passthrough_hit_test(
    _this: &objc::runtime::Object,
    _: objc::runtime::Sel,
    _: cocoa::foundation::NSPoint,
) -> id {
    nil
}

#[cfg(target_os = "macos")]
fn footer_button_class() -> *const objc::runtime::Class {
    use std::sync::OnceLock;

    use objc::declare::ClassDecl;
    use objc::runtime::{Object, Sel};
    use objc::{class, sel, sel_impl};

    static CLASS: OnceLock<usize> = OnceLock::new();

    *CLASS.get_or_init(|| {
        // SAFETY: Registering an ObjC class from NSButton. ClassDecl::new returns
        // None only if the class name is already registered, in which case we
        // fall back to the plain NSButton class.
        unsafe {
            let superclass = class!(NSButton);
            let Some(mut decl) = ClassDecl::new("ScriptKitFooterButton", superclass) else {
                return class!(NSButton) as *const _ as usize;
            };
            decl.add_ivar::<usize>("_hoverCGColor");
            decl.add_ivar::<usize>("_selectedCGColor");
            decl.add_ivar::<cocoa::base::BOOL>("_isActionsButton");
            decl.add_ivar::<cocoa::base::BOOL>("_selected");
            decl.add_ivar::<cocoa::base::BOOL>("_enabled");
            decl.add_method(
                sel!(acceptsFirstMouse:),
                footer_button_accepts_first_mouse
                    as extern "C" fn(&Object, Sel, id) -> cocoa::base::BOOL,
            );
            decl.add_method(
                sel!(mouseDownCanMoveWindow),
                footer_button_mouse_down_can_move_window
                    as extern "C" fn(&Object, Sel) -> cocoa::base::BOOL,
            );
            decl.add_method(
                sel!(mouseDown:),
                footer_button_mouse_down as extern "C" fn(&Object, Sel, id),
            );
            decl.add_method(
                sel!(resetCursorRects),
                footer_button_reset_cursor_rects as extern "C" fn(&Object, Sel),
            );
            decl.add_method(
                sel!(updateTrackingAreas),
                footer_button_update_tracking_areas as extern "C" fn(&Object, Sel),
            );
            decl.add_method(
                sel!(mouseEntered:),
                footer_button_mouse_entered as extern "C" fn(&Object, Sel, id),
            );
            decl.add_method(
                sel!(mouseExited:),
                footer_button_mouse_exited as extern "C" fn(&Object, Sel, id),
            );
            decl.register() as *const _ as usize
        }
    }) as *const objc::runtime::Class
}

#[cfg(target_os = "macos")]
extern "C" fn footer_button_accepts_first_mouse(
    this: &objc::runtime::Object,
    _: objc::runtime::Sel,
    _: id,
) -> cocoa::base::BOOL {
    // SAFETY: `this` is a live instance of our registered NSButton subclass,
    // so reading the `_enabled` ivar is valid for the duration of this call.
    let enabled: cocoa::base::BOOL = unsafe { *this.get_ivar::<cocoa::base::BOOL>("_enabled") };
    if enabled != YES {
        return NO;
    }
    tracing::debug!(
        target: "script_kit::footer_popup",
        event = "native_footer_button_accepts_first_mouse",
        "Native footer button accepted first mouse"
    );
    YES
}

#[cfg(target_os = "macos")]
extern "C" fn footer_button_mouse_down_can_move_window(
    _this: &objc::runtime::Object,
    _: objc::runtime::Sel,
) -> cocoa::base::BOOL {
    NO
}

#[cfg(target_os = "macos")]
extern "C" fn footer_button_reset_cursor_rects(
    this: &objc::runtime::Object,
    _: objc::runtime::Sel,
) {
    use objc::{class, msg_send, sel, sel_impl};

    // SAFETY: `this` is a live NSButton subclass. We add a cursor rect covering
    // the full button bounds so the footer keeps the default arrow cursor.
    unsafe {
        let enabled: cocoa::base::BOOL = *this.get_ivar::<cocoa::base::BOOL>("_enabled");
        if enabled != YES {
            return;
        }
        let bounds: cocoa::foundation::NSRect = msg_send![this, bounds];
        let cursor: id = msg_send![class!(NSCursor), arrowCursor];
        let _: () = msg_send![this, addCursorRect:bounds cursor:cursor];
    }
}

#[cfg(target_os = "macos")]
extern "C" fn footer_button_update_tracking_areas(
    this: &objc::runtime::Object,
    _: objc::runtime::Sel,
) {
    use objc::{class, msg_send, sel, sel_impl};

    // SAFETY: Replace old tracking areas with a fresh one matching the button
    // bounds. This is the standard AppKit pattern for views that change size.
    unsafe {
        // Call super first.
        let this_id = this as *const _ as id;
        let _: () = msg_send![super(this_id, class!(NSButton)), updateTrackingAreas];

        // Remove existing tracking areas.
        let existing: id = msg_send![this, trackingAreas];
        if existing != nil {
            let count: usize = msg_send![existing, count];
            for i in (0..count).rev() {
                let area: id = msg_send![existing, objectAtIndex: i];
                let _: () = msg_send![this, removeTrackingArea: area];
            }
        }

        // Add a new tracking area for mouseEntered/mouseExited.
        let opts: usize = 0x01 /* MouseEnteredAndExited */ | 0x80 /* ActiveAlways */ | 0x20 /* InVisibleRect */;
        let bounds: cocoa::foundation::NSRect = msg_send![this, bounds];
        let area: id = msg_send![class!(NSTrackingArea), alloc];
        let area: id = msg_send![
            area,
            initWithRect: bounds
            options: opts
            owner: this_id
            userInfo: nil
        ];
        if area != nil {
            let _: () = msg_send![this, addTrackingArea: area];
        }
    }
}

#[cfg(target_os = "macos")]
unsafe fn set_footer_button_text_opacity(view: id, opacity: f64) {
    use objc::{class, msg_send, sel, sel_impl};

    if view == nil {
        return;
    }

    let theme = crate::theme::get_cached_theme();
    let color = ns_color_from_hex_with_alpha(theme.colors.text.primary, opacity);
    if color == nil {
        return;
    }

    let is_text_field: cocoa::base::BOOL = msg_send![view, isKindOfClass: class!(NSTextField)];
    if is_text_field == YES {
        let _: () = msg_send![view, setTextColor: color];
    }
    let is_image_view: cocoa::base::BOOL = msg_send![view, isKindOfClass: class!(NSImageView)];
    if is_image_view == YES {
        let _: () = msg_send![view, setContentTintColor: color];
    }

    let subviews: id = msg_send![view, subviews];
    if subviews == nil {
        return;
    }
    let count: usize = msg_send![subviews, count];
    for i in 0..count {
        let child: id = msg_send![subviews, objectAtIndex: i];
        set_footer_button_text_opacity(child, opacity);
    }
}

#[cfg(target_os = "macos")]
unsafe fn set_footer_button_border_alpha(view: id, alpha: f64) {
    use objc::{msg_send, sel, sel_impl};

    if view == nil {
        return;
    }

    let theme = crate::theme::get_cached_theme();
    let color = ns_color_from_hex_with_alpha(theme.colors.text.primary, alpha);
    if color == nil {
        return;
    }

    let layer: id = msg_send![view, layer];
    if layer != nil {
        let border_width: f64 = msg_send![layer, borderWidth];
        if border_width > 0.0 {
            let cg_border: id = msg_send![color, CGColor];
            if cg_border != nil {
                let _: () = msg_send![layer, setBorderColor: cg_border];
            }
        }
    }

    let subviews: id = msg_send![view, subviews];
    if subviews == nil {
        return;
    }
    let count: usize = msg_send![subviews, count];
    for i in 0..count {
        let child: id = msg_send![subviews, objectAtIndex: i];
        set_footer_button_border_alpha(child, alpha);
    }
}

#[cfg(target_os = "macos")]
unsafe fn apply_footer_button_background(button: id, rgba_value: Option<u32>) {
    use objc::{msg_send, sel, sel_impl};

    if button == nil {
        return;
    }

    let superview: id = msg_send![button, superview];
    if superview == nil {
        return;
    }

    let layer: id = msg_send![superview, layer];
    if layer == nil {
        return;
    }

    if let Some(rgba_value) = rgba_value {
        let ns_color: id = ns_color_from_rgba(rgba_value);
        if ns_color != nil {
            let cg: id = msg_send![ns_color, CGColor];
            if cg != nil {
                let _: () = msg_send![layer, setBackgroundColor: cg];
            }
        }
    } else {
        let null_color: id = std::ptr::null_mut();
        let _: () = msg_send![layer, setBackgroundColor: null_color];
    }
}

#[cfg(target_os = "macos")]
extern "C" fn footer_button_mouse_down(
    this: &objc::runtime::Object,
    _: objc::runtime::Sel,
    event: id,
) {
    use objc::{class, msg_send, sel, sel_impl};

    // SAFETY: `this` is our NSButton subclass. Actions opens a persistent popup,
    // so it owns selected visuals on mouse down instead of waiting for AppKit's
    // mouse-up action cycle to briefly clear and restore the state.
    unsafe {
        let enabled: cocoa::base::BOOL = *this.get_ivar::<cocoa::base::BOOL>("_enabled");
        if enabled != YES {
            let this_id = this as *const _ as id;
            let _: () = msg_send![super(this_id, class!(NSButton)), mouseDown: event];
            return;
        }

        let is_actions: cocoa::base::BOOL = *this.get_ivar::<cocoa::base::BOOL>("_isActionsButton");
        if is_actions != YES {
            let this_id = this as *const _ as id;
            let _: () = msg_send![super(this_id, class!(NSButton)), mouseDown: event];
            return;
        }

        let button_id = this as *const _ as id;
        if let Some(obj) = button_id.as_mut() {
            obj.set_ivar::<cocoa::base::BOOL>("_selected", YES);
        }
        let theme = crate::theme::get_cached_theme();
        let chrome = crate::theme::AppChromeColors::from_theme(&theme);
        apply_footer_button_background(
            button_id,
            Some(footer_selected_background_rgba(
                FooterAction::Actions,
                &chrome,
            )),
        );
        let superview: id = msg_send![button_id, superview];
        set_footer_button_text_opacity(superview, 1.0);
        set_footer_button_border_alpha(
            superview,
            crate::components::footer_chrome::footer_keycap_border_alpha(&theme, true) as f64,
        );

        tracing::debug!(
            target: "script_kit::footer_popup",
            event = "native_footer_actions_mouse_down_selected",
            "Selected native footer Actions on mouse down"
        );
        let this_id = this as *const _ as id;
        send_footer_action_from_sender(this_id, FooterAction::Actions);
    }
}

#[cfg(target_os = "macos")]
extern "C" fn footer_button_mouse_entered(
    this: &objc::runtime::Object,
    _: objc::runtime::Sel,
    _event: id,
) {
    use objc::{msg_send, sel, sel_impl};

    // SAFETY: Set hover background on the parent container's layer.
    // Recompute color from theme each time to avoid dangling CGColor pointers.
    unsafe {
        let enabled: cocoa::base::BOOL = *this.get_ivar::<cocoa::base::BOOL>("_enabled");
        if enabled != YES {
            return;
        }
        let is_actions: cocoa::base::BOOL = *this.get_ivar::<cocoa::base::BOOL>("_isActionsButton");
        tracing::debug!(
            target: "script_kit::footer_popup",
            event = "native_footer_button_hover_entered",
            is_actions_button = is_actions == YES,
            "Native footer button hover entered"
        );

        let superview: id = msg_send![this, superview];
        if superview == nil {
            return;
        }
        let layer: id = msg_send![superview, layer];
        if layer == nil {
            return;
        }
        let theme = crate::theme::get_cached_theme();
        let chrome = crate::theme::AppChromeColors::from_theme(&theme);
        apply_footer_button_background(this as *const _ as id, Some(chrome.hover_rgba));
        set_footer_button_text_opacity(superview, 1.0);
        set_footer_button_border_alpha(
            superview,
            crate::components::footer_chrome::footer_keycap_border_hover_alpha(&theme) as f64,
        );
    }
}

#[cfg(target_os = "macos")]
extern "C" fn footer_button_mouse_exited(
    this: &objc::runtime::Object,
    _: objc::runtime::Sel,
    _event: id,
) {
    use objc::{msg_send, sel, sel_impl};

    // SAFETY: Clear hover background on the parent container's layer.
    // If this button has _selected set, restore the selected color instead
    // of clearing.
    unsafe {
        let selected: cocoa::base::BOOL = *this.get_ivar::<cocoa::base::BOOL>("_selected");
        let is_actions: cocoa::base::BOOL = *this.get_ivar::<cocoa::base::BOOL>("_isActionsButton");
        let actions_window_open = crate::actions::is_actions_window_open();
        tracing::debug!(
            target: "script_kit::footer_popup",
            event = "native_footer_button_hover_exited",
            is_actions_button = is_actions == YES,
            selected = selected == YES,
            actions_window_open,
            "Native footer button hover exited"
        );

        let superview: id = msg_send![this, superview];
        if superview == nil {
            return;
        }
        let layer: id = msg_send![superview, layer];
        if layer == nil {
            return;
        }

        let theme = crate::theme::get_cached_theme();
        if selected == YES || (is_actions == YES && actions_window_open) {
            let chrome = crate::theme::AppChromeColors::from_theme(&theme);
            apply_footer_button_background(
                this as *const _ as id,
                Some(footer_selected_background_rgba_for_actions_button(
                    is_actions, &chrome,
                )),
            );
        } else {
            apply_footer_button_background(this as *const _ as id, None);
        }
        set_footer_button_text_opacity(superview, crate::theme::opacity::OPACITY_TEXT_MUTED as f64);
        set_footer_button_border_alpha(
            superview,
            crate::components::footer_chrome::footer_keycap_border_alpha(
                &theme,
                selected == YES || (is_actions == YES && actions_window_open),
            ) as f64,
        );
    }
}

#[cfg(target_os = "macos")]
fn footer_effect_view_class() -> *const objc::runtime::Class {
    use std::sync::OnceLock;

    use objc::declare::ClassDecl;
    use objc::runtime::{Object, Sel};
    use objc::{class, sel, sel_impl};

    static CLASS: OnceLock<usize> = OnceLock::new();

    *CLASS.get_or_init(|| unsafe {
        let superclass = class!(NSVisualEffectView);
        let Some(mut decl) = ClassDecl::new("ScriptKitFooterEffectView", superclass) else {
            return class!(NSVisualEffectView) as *const _ as usize;
        };
        decl.add_method(
            sel!(hitTest:),
            footer_hit_test as extern "C" fn(&Object, Sel, cocoa::foundation::NSPoint) -> id,
        );
        decl.add_method(
            sel!(mouseDown:),
            footer_mouse_down as extern "C" fn(&Object, Sel, id),
        );
        decl.add_method(
            sel!(mouseUp:),
            footer_mouse_up as extern "C" fn(&Object, Sel, id),
        );
        decl.add_method(
            sel!(mouseDragged:),
            footer_mouse_dragged as extern "C" fn(&Object, Sel, id),
        );
        decl.add_method(
            sel!(rightMouseDown:),
            footer_mouse_down as extern "C" fn(&Object, Sel, id),
        );
        decl.add_method(
            sel!(rightMouseUp:),
            footer_mouse_up as extern "C" fn(&Object, Sel, id),
        );
        decl.add_method(
            sel!(otherMouseDown:),
            footer_mouse_down as extern "C" fn(&Object, Sel, id),
        );
        decl.add_method(
            sel!(otherMouseUp:),
            footer_mouse_up as extern "C" fn(&Object, Sel, id),
        );
        decl.add_method(
            sel!(scrollWheel:),
            footer_scroll_wheel as extern "C" fn(&Object, Sel, id),
        );
        decl.add_method(
            sel!(acceptsFirstMouse:),
            footer_accepts_first_mouse as extern "C" fn(&Object, Sel, id) -> cocoa::base::BOOL,
        );
        decl.register() as *const _ as usize
    }) as *const objc::runtime::Class
}

#[cfg(target_os = "macos")]
/// Walk up the view hierarchy from `view` looking for the nearest NSButton.
/// Returns the button if found, nil otherwise.
///
/// SAFETY: Caller must ensure `view` is a valid, live AppKit view pointer on
/// the main thread.
unsafe fn nearest_footer_button(mut view: id) -> id {
    use objc::{class, msg_send, sel, sel_impl};

    while view != nil {
        let is_button: cocoa::base::BOOL = msg_send![view, isKindOfClass: class!(NSButton)];
        if is_button == YES {
            return view;
        }

        let superview: id = msg_send![view, superview];
        if superview == nil || superview == view {
            break;
        }
        view = superview;
    }

    nil
}

#[cfg(target_os = "macos")]
/// Return a footer button contained by `view`, if `view` is one of the native
/// footer item wrappers.
///
/// SAFETY: Caller must ensure `view` is a valid, live AppKit view pointer on
/// the main thread.
unsafe fn footer_button_in_subviews(view: id) -> id {
    use objc::{class, msg_send, sel, sel_impl};

    if view == nil {
        return nil;
    }

    let subviews: id = msg_send![view, subviews];
    if subviews == nil {
        return nil;
    }

    let count: usize = msg_send![subviews, count];
    for index in 0..count {
        let child: id = msg_send![subviews, objectAtIndex: index];
        if child == nil {
            continue;
        }

        let is_button: cocoa::base::BOOL = msg_send![child, isKindOfClass: class!(NSButton)];
        if is_button == YES {
            return child;
        }
    }

    nil
}

#[cfg(target_os = "macos")]
/// Resolve text-field or empty-area hits inside a footer item wrapper to the
/// sibling button that owns that whole visual slot.
///
/// SAFETY: Caller must ensure `view` is a valid, live AppKit view pointer on
/// the main thread.
unsafe fn nearest_footer_item_button(mut view: id) -> id {
    use objc::{msg_send, sel, sel_impl};

    while view != nil {
        let button = footer_button_in_subviews(view);
        if button != nil {
            return button;
        }

        let superview: id = msg_send![view, superview];
        if superview == nil || superview == view {
            break;
        }
        view = superview;
    }

    nil
}

#[cfg(target_os = "macos")]
fn ns_point_in_rect(point: cocoa::foundation::NSPoint, rect: cocoa::foundation::NSRect) -> bool {
    point.x >= rect.origin.x
        && point.y >= rect.origin.y
        && point.x < rect.origin.x + rect.size.width
        && point.y < rect.origin.y + rect.size.height
}

#[cfg(target_os = "macos")]
/// Resolve a footer point to the button inside the visible hint item frame,
/// before AppKit's normal hit test can return an unrelated overlay sibling.
///
/// SAFETY: Caller must ensure `footer_view` is a valid footer AppKit view
/// pointer on the main thread.
unsafe fn footer_item_button_at_point(
    footer_view: id,
    point_in_footer: cocoa::foundation::NSPoint,
) -> id {
    use objc::{msg_send, sel, sel_impl};

    let hints_view = find_subview_by_identifier(footer_view, FOOTER_HINTS_ID);
    if hints_view == nil {
        return nil;
    }

    let point_in_hints: cocoa::foundation::NSPoint =
        msg_send![hints_view, convertPoint: point_in_footer fromView: footer_view];
    let hints_bounds: cocoa::foundation::NSRect = msg_send![hints_view, bounds];
    if !ns_point_in_rect(point_in_hints, hints_bounds) {
        return nil;
    }

    let items: id = msg_send![hints_view, subviews];
    if items == nil {
        return nil;
    }

    let count: usize = msg_send![items, count];
    for index in (0..count).rev() {
        let item: id = msg_send![items, objectAtIndex: index];
        if item == nil {
            continue;
        }

        let point_in_item: cocoa::foundation::NSPoint =
            msg_send![item, convertPoint: point_in_hints fromView: hints_view];
        let item_bounds: cocoa::foundation::NSRect = msg_send![item, bounds];
        if !ns_point_in_rect(point_in_item, item_bounds) {
            continue;
        }

        let button = footer_button_in_subviews(item);
        if button != nil {
            tracing::debug!(
                target: "script_kit::footer_popup",
                event = "native_footer_hit_test_item_geometry",
                x = point_in_footer.x,
                y = point_in_footer.y,
                "Resolved native footer hit by item geometry"
            );
            return button;
        }
    }

    nil
}

#[cfg(target_os = "macos")]
extern "C" fn footer_hit_test(
    this: &objc::runtime::Object,
    _: objc::runtime::Sel,
    point: cocoa::foundation::NSPoint,
) -> id {
    use objc::{class, msg_send, sel, sel_impl};

    // SAFETY: `this` is a live NSVisualEffectView subclass instance. We delegate
    // Route clicks to buttons, let everything else (scroll, hover) fall
    // through to the GPUI Metal view behind us. Returning nil for non-button
    // areas is critical — returning self would intercept scroll events and
    // break list scrolling.
    unsafe {
        let this_id = this as *const _ as id;
        let item_button = footer_item_button_at_point(this_id, point);
        if item_button != nil {
            return item_button;
        }

        let hit: id = msg_send![super(this_id, class!(NSVisualEffectView)), hitTest: point];
        let button = nearest_footer_button(hit);
        if button != nil {
            return button;
        }
        let item_button = nearest_footer_item_button(hit);
        if item_button != nil {
            return item_button;
        }
        nil
    }
}

#[cfg(target_os = "macos")]
extern "C" fn footer_mouse_down(_this: &objc::runtime::Object, _: objc::runtime::Sel, _: id) {
    tracing::debug!(
        target: "script_kit::footer_popup",
        event = "native_footer_background_mouse_swallowed",
        "Swallowed background mouseDown in native footer"
    );
}

#[cfg(target_os = "macos")]
extern "C" fn footer_mouse_up(_this: &objc::runtime::Object, _: objc::runtime::Sel, _: id) {
    tracing::debug!(
        target: "script_kit::footer_popup",
        event = "native_footer_background_mouse_up_swallowed",
        "Swallowed background mouseUp in native footer"
    );
}

#[cfg(target_os = "macos")]
extern "C" fn footer_mouse_dragged(_this: &objc::runtime::Object, _: objc::runtime::Sel, _: id) {
    tracing::debug!(
        target: "script_kit::footer_popup",
        event = "native_footer_background_mouse_dragged_swallowed",
        "Swallowed background mouseDragged in native footer"
    );
}

#[cfg(target_os = "macos")]
extern "C" fn footer_scroll_wheel(this: &objc::runtime::Object, _: objc::runtime::Sel, event: id) {
    use objc::{msg_send, sel, sel_impl};

    // SAFETY: Forward scroll events to the next responder so the GPUI list
    // behind the footer can scroll.
    unsafe {
        let next: id = msg_send![this, nextResponder];
        if next != nil {
            let _: () = msg_send![next, scrollWheel: event];
        }
    }
}

#[cfg(target_os = "macos")]
extern "C" fn footer_accepts_first_mouse(
    _this: &objc::runtime::Object,
    _: objc::runtime::Sel,
    _: id,
) -> cocoa::base::BOOL {
    YES
}

#[cfg(target_os = "macos")]
fn footer_action_target() -> id {
    use std::sync::OnceLock;

    use objc::{msg_send, sel, sel_impl};

    static TARGET: OnceLock<usize> = OnceLock::new();

    *TARGET.get_or_init(|| unsafe {
        let target: id = msg_send![footer_action_target_class(), new];
        target as usize
    }) as id
}

#[cfg(target_os = "macos")]
fn footer_action_selector(action: FooterAction) -> objc::runtime::Sel {
    use objc::{sel, sel_impl};

    match action {
        FooterAction::Run => sel!(runFooterAction:),
        FooterAction::Actions => sel!(actionsFooterAction:),
        FooterAction::Ai => sel!(aiFooterAction:),
        FooterAction::Apply => sel!(applyFooterAction:),
        FooterAction::Close => sel!(closeFooterAction:),
        FooterAction::Stop => sel!(stopFooterAction:),
        FooterAction::PasteResponse => sel!(pasteResponseFooterAction:),
    }
}

#[cfg(target_os = "macos")]
fn footer_action_target_class() -> *const objc::runtime::Class {
    use std::sync::OnceLock;

    use objc::declare::ClassDecl;
    use objc::runtime::{Object, Sel};
    use objc::{class, sel, sel_impl};

    static CLASS: OnceLock<usize> = OnceLock::new();

    *CLASS.get_or_init(|| unsafe {
        let superclass = class!(NSObject);
        let Some(mut decl) = ClassDecl::new("ScriptKitFooterActionTarget", superclass) else {
            return class!(NSObject) as *const _ as usize;
        };
        decl.add_method(
            sel!(runFooterAction:),
            footer_run_action as extern "C" fn(&Object, Sel, id),
        );
        decl.add_method(
            sel!(actionsFooterAction:),
            footer_actions_action as extern "C" fn(&Object, Sel, id),
        );
        decl.add_method(
            sel!(aiFooterAction:),
            footer_ai_action as extern "C" fn(&Object, Sel, id),
        );
        decl.add_method(
            sel!(applyFooterAction:),
            footer_apply_action as extern "C" fn(&Object, Sel, id),
        );
        decl.add_method(
            sel!(closeFooterAction:),
            footer_close_action as extern "C" fn(&Object, Sel, id),
        );
        decl.add_method(
            sel!(stopFooterAction:),
            footer_stop_action as extern "C" fn(&Object, Sel, id),
        );
        decl.add_method(
            sel!(pasteResponseFooterAction:),
            footer_paste_response_action as extern "C" fn(&Object, Sel, id),
        );
        decl.register() as *const _ as usize
    }) as *const objc::runtime::Class
}

#[cfg(target_os = "macos")]
extern "C" fn footer_run_action(_this: &objc::runtime::Object, _: objc::runtime::Sel, sender: id) {
    send_footer_action_from_sender(sender, FooterAction::Run);
}

#[cfg(target_os = "macos")]
extern "C" fn footer_actions_action(
    _this: &objc::runtime::Object,
    _: objc::runtime::Sel,
    sender: id,
) {
    send_footer_action_from_sender(sender, FooterAction::Actions);
}

#[cfg(target_os = "macos")]
extern "C" fn footer_ai_action(_this: &objc::runtime::Object, _: objc::runtime::Sel, sender: id) {
    send_footer_action_from_sender(sender, FooterAction::Ai);
}

#[cfg(target_os = "macos")]
extern "C" fn footer_apply_action(
    _this: &objc::runtime::Object,
    _: objc::runtime::Sel,
    sender: id,
) {
    send_footer_action_from_sender(sender, FooterAction::Apply);
}

#[cfg(target_os = "macos")]
extern "C" fn footer_close_action(
    _this: &objc::runtime::Object,
    _: objc::runtime::Sel,
    sender: id,
) {
    send_footer_action_from_sender(sender, FooterAction::Close);
}

#[cfg(target_os = "macos")]
extern "C" fn footer_stop_action(_this: &objc::runtime::Object, _: objc::runtime::Sel, sender: id) {
    send_footer_action_from_sender(sender, FooterAction::Stop);
}

#[cfg(target_os = "macos")]
extern "C" fn footer_paste_response_action(
    _this: &objc::runtime::Object,
    _: objc::runtime::Sel,
    sender: id,
) {
    send_footer_action_from_sender(sender, FooterAction::PasteResponse);
}
