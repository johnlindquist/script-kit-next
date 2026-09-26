use std::rc::Rc;

use gpui::{px, App, AppContext as _, Pixels, SharedString, WeakEntity, Window};

use super::text_field::SharedConfirmTextField;
use super::window::{
    open_confirm_popup_window, ConfirmPopupParentWindow, ConfirmWindowOptions, ParentDialogResult,
};
use crate::components::confirm_modal_shell::PARENT_MODAL_WIDTH_PX;

type ConfirmCallback = Rc<dyn Fn(&mut Window, &mut App)>;
type QueuedModal = Box<dyn FnOnce(&mut App)>;

thread_local! {
    /// Modals asked for while another is open. One modal shows at a time; the
    /// next opens once the current one has closed, instead of replacing it.
    static QUEUED_MODALS: std::cell::RefCell<std::collections::VecDeque<QueuedModal>> =
        const { std::cell::RefCell::new(std::collections::VecDeque::new()) };
}

/// Open the oldest waiting modal, if no modal is open. Waiting modals whose
/// window has closed, or whose question no longer matters, are dropped.
pub(crate) fn open_next_queued_modal(cx: &mut App) {
    while !super::window::is_confirm_window_open() {
        let Some(next) = QUEUED_MODALS.with_borrow_mut(|queue| queue.pop_front()) else {
            return;
        };
        next(cx);
    }
}

pub(crate) fn queued_modal_count() -> usize {
    QUEUED_MODALS.with_borrow(|queue| queue.len())
}

/// What a modal's primary action can cost the user. It decides how the
/// primary button looks and which button a reflexive Enter lands on.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum ConfirmRisk {
    /// Nothing is lost. The primary button reads normally and starts focused.
    #[default]
    Safe,
    /// Stops or discards something the user asked for by name (quit, shut
    /// down, stop processes). The primary label is red; it still starts
    /// focused, because Enter is what the user came to press.
    Destructive,
    /// Permanently destroys the user's data and cannot be undone (delete
    /// permanently, empty the Trash, delete history). The primary label is
    /// red and Cancel starts focused, so a reflexive Enter is harmless.
    /// Actions that can be undone use an Undo toast instead of a modal.
    Irreversible,
}

#[derive(Clone)]
pub(crate) struct ParentConfirmOptions {
    pub title: SharedString,
    pub body: SharedString,
    pub confirm_text: SharedString,
    pub cancel_text: SharedString,
    pub risk: ConfirmRisk,
    pub width: Pixels,
}

#[derive(Clone, Debug)]
pub(crate) struct ParentActionDialogOptions {
    pub title: SharedString,
    pub body: SharedString,
    pub primary_text: SharedString,
    pub secondary_text: Option<SharedString>,
    pub dismiss_text: SharedString,
    pub width: Pixels,
}

impl Default for ParentActionDialogOptions {
    fn default() -> Self {
        Self {
            title: "Action required".into(),
            body: "".into(),
            primary_text: "Continue".into(),
            secondary_text: None,
            dismiss_text: "Back".into(),
            width: px(PARENT_MODAL_WIDTH_PX),
        }
    }
}

impl std::fmt::Debug for ParentConfirmOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ParentConfirmOptions")
            .field("title", &self.title)
            .field("confirm_text", &self.confirm_text)
            .field("cancel_text", &self.cancel_text)
            .field("risk", &self.risk)
            .finish()
    }
}

impl Default for ParentConfirmOptions {
    fn default() -> Self {
        Self {
            title: "Confirm".into(),
            body: "".into(),
            confirm_text: "OK".into(),
            cancel_text: "Cancel".into(),
            risk: ConfirmRisk::Safe,
            width: px(PARENT_MODAL_WIDTH_PX),
        }
    }
}

// ── Design-contract values for the confirm mockups ─────────────────────────
// Exported through src/design_contract for the HTML design mockups. Every
// confirmation in the app is the native confirm popup (src/confirm/window.rs);
// the full-window confirm these values once described was retired on
// 2026-09-24 because a modal must never replace the window it belongs to.

/// Title font size for the confirm mockups.
pub(crate) const CONFIRM_PROMPT_TITLE_FONT_SIZE_PX: f32 = 20.0;
/// Body font size for the confirm mockups.
pub(crate) const CONFIRM_PROMPT_BODY_FONT_SIZE_PX: f32 = 14.0;
/// Reading-width cap for the body copy.
pub(crate) const CONFIRM_PROMPT_BODY_MAX_WIDTH_PX: f32 = 560.0;

/// The implicit GPUI line height: `TextStyle::default()` uses `phi()` and
/// `line_height_in_pixels` rounds (vendor/gpui/src/style.rs). The confirm
/// renderer never overrides it, so 14pt body → 23, 20pt title → 32.
/// Pixel-validated against a live capture on 2026-07-11 (body line spacing
/// measured at exactly 23.0 logical pt).
pub(crate) fn confirm_prompt_line_height_px(font_size: f32) -> f32 {
    const PHI: f32 = 1.618_034;
    (font_size * PHI).round()
}

/// Confirm mockup layout inputs exported as design-contract tokens.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ConfirmPromptMetrics {
    /// Padding on the centered content region (`design_spacing.padding_xl`).
    pub content_padding: f32,
    /// Gap between title and body (`design_spacing.padding_md`). The protocol
    /// layout model claims 16; the renderer truth is this field (12 under the
    /// Default design) — recorded as `confirmGap.rendererSpacingVsLayoutOracle`.
    pub stack_gap: f32,
    pub title_font_size: f32,
    pub title_line_height: f32,
    pub body_font_size: f32,
    pub body_line_height: f32,
    pub body_max_width: f32,
    /// Shared footer action-row geometry used by the native rail projection.
    pub action_button_height: f32,
    pub action_button_gap: f32,
    pub confirm_slot_width: f32,
    pub cancel_slot_width: f32,
    /// Height of the in-shell native-footer spacer (the shared footer rail
    /// height). The confirm shell is a fixed `STANDARD_HEIGHT` column, so the
    /// centered region is `STANDARD_HEIGHT - footer_spacer_height` tall
    /// starting below the main context header.
    pub footer_spacer_height: f32,
}

pub(crate) fn resolved_confirm_prompt_metrics(
    spacing: crate::designs::DesignSpacing,
    footer_rail_height: f32,
) -> ConfirmPromptMetrics {
    ConfirmPromptMetrics {
        content_padding: spacing.padding_xl,
        stack_gap: spacing.padding_md,
        title_font_size: CONFIRM_PROMPT_TITLE_FONT_SIZE_PX,
        title_line_height: confirm_prompt_line_height_px(CONFIRM_PROMPT_TITLE_FONT_SIZE_PX),
        body_font_size: CONFIRM_PROMPT_BODY_FONT_SIZE_PX,
        body_line_height: confirm_prompt_line_height_px(CONFIRM_PROMPT_BODY_FONT_SIZE_PX),
        body_max_width: CONFIRM_PROMPT_BODY_MAX_WIDTH_PX,
        action_button_height: crate::components::footer_chrome::footer_button_height(
            footer_rail_height,
        ),
        action_button_gap: crate::components::footer_chrome::current_main_menu_footer_metrics()
            .item_gap_px,
        confirm_slot_width: crate::components::footer_chrome::footer_action_slot_width(
            crate::components::footer_chrome::FooterActionSlot::Run,
        ),
        cancel_slot_width: crate::components::footer_chrome::footer_action_slot_width(
            crate::components::footer_chrome::FooterActionSlot::Close,
        ),
        footer_spacer_height: footer_rail_height,
    }
}

/// What the in-window confirm surface paints for its two text blocks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ResolvedConfirmPromptColors {
    pub title_rgba: u32,
    pub body_rgba: u32,
}

pub(crate) fn resolved_confirm_prompt_colors(
    theme: &crate::theme::Theme,
    is_danger: bool,
) -> ResolvedConfirmPromptColors {
    ResolvedConfirmPromptColors {
        title_rgba: ((if is_danger {
            theme.colors.ui.error
        } else {
            theme.colors.text.primary
        }) << 8)
            | 0xFF,
        body_rgba: (theme.colors.text.secondary << 8) | 0xFF,
    }
}

impl ParentConfirmOptions {
    /// A confirmation for something the user asked for by name that stops or
    /// discards work (see [`ConfirmRisk::Destructive`]).
    #[allow(dead_code)]
    pub(crate) fn destructive(
        title: impl Into<SharedString>,
        body: impl Into<SharedString>,
        confirm_text: impl Into<SharedString>,
    ) -> Self {
        Self::with_risk(ConfirmRisk::Destructive, title, body, confirm_text)
    }

    /// A confirmation for permanently deleting the user's data (see
    /// [`ConfirmRisk::Irreversible`]): Cancel starts focused.
    #[allow(dead_code)]
    pub(crate) fn irreversible(
        title: impl Into<SharedString>,
        body: impl Into<SharedString>,
        confirm_text: impl Into<SharedString>,
    ) -> Self {
        Self::with_risk(ConfirmRisk::Irreversible, title, body, confirm_text)
    }

    fn with_risk(
        risk: ConfirmRisk,
        title: impl Into<SharedString>,
        body: impl Into<SharedString>,
        confirm_text: impl Into<SharedString>,
    ) -> Self {
        Self {
            title: title.into(),
            body: body.into(),
            confirm_text: confirm_text.into(),
            risk,
            ..Self::default()
        }
    }
}

// Used by include!() code in app_actions/handle_action/scripts.rs — clippy
// cannot trace through include!() and reports a false-positive dead_code lint.
#[allow(dead_code)]
pub(crate) fn open_parent_confirm_dialog(
    window: &mut Window,
    cx: &mut App,
    options: ParentConfirmOptions,
    on_confirm: impl Fn(&mut Window, &mut App) + 'static,
    on_cancel: impl Fn(&mut Window, &mut App) + 'static,
) {
    open_parent_confirm_dialog_with_lifecycle(window, cx, options, || true, on_confirm, on_cancel);
}

/// Open a confirm popup whose parent window is identified explicitly by
/// automation id (e.g. `"notes"`). Use this for non-main parent windows so the
/// popup is pinned and AppKit-attached to the *intended* window even when
/// focus has drifted.
#[allow(dead_code)]
pub(crate) fn open_parent_confirm_dialog_for_automation_parent(
    window: &mut Window,
    cx: &mut App,
    parent_automation_id: impl Into<String>,
    options: ParentConfirmOptions,
    on_confirm: impl Fn(&mut Window, &mut App) + 'static,
    on_cancel: impl Fn(&mut Window, &mut App) + 'static,
) {
    open_parent_confirm_dialog_with_lifecycle_and_parent(
        window,
        cx,
        options,
        Some(parent_automation_id.into()),
        || true,
        Rc::new(on_confirm),
        DialogExtras::default(),
        Rc::new(on_cancel),
    );
}

#[allow(dead_code)]
pub(crate) fn open_parent_confirm_dialog_for_entity<T: 'static>(
    window: &mut Window,
    cx: &mut App,
    owner: WeakEntity<T>,
    options: ParentConfirmOptions,
    on_confirm: impl Fn(&mut Window, &mut App) + 'static,
    on_cancel: impl Fn(&mut Window, &mut App) + 'static,
) {
    let owner_for_lifecycle = owner.clone();
    open_parent_confirm_dialog_with_lifecycle(
        window,
        cx,
        options,
        move || owner_for_lifecycle.upgrade().is_some(),
        on_confirm,
        on_cancel,
    );
}

pub(crate) fn open_parent_confirm_dialog_with_lifecycle(
    window: &mut Window,
    cx: &mut App,
    options: ParentConfirmOptions,
    keep_open_while: impl Fn() -> bool + 'static,
    on_confirm: impl Fn(&mut Window, &mut App) + 'static,
    on_cancel: impl Fn(&mut Window, &mut App) + 'static,
) {
    open_parent_confirm_dialog_with_lifecycle_and_parent(
        window,
        cx,
        options,
        None,
        keep_open_while,
        Rc::new(on_confirm),
        DialogExtras::default(),
        Rc::new(on_cancel),
    );
}

#[allow(
    dead_code,
    reason = "the separately compiled application binary owns Agent Chat parent action dialogs"
)]
pub(crate) fn open_parent_action_dialog(
    window: &mut Window,
    cx: &mut App,
    options: ParentActionDialogOptions,
    on_primary: impl Fn(&mut Window, &mut App) + 'static,
    on_secondary: impl Fn(&mut Window, &mut App) + 'static,
    on_dismiss: impl Fn(&mut Window, &mut App) + 'static,
) {
    let secondary = options
        .secondary_text
        .clone()
        .map(|label| (label, Rc::new(on_secondary) as ConfirmCallback));
    open_parent_confirm_dialog_with_lifecycle_and_parent(
        window,
        cx,
        action_dialog_popup_options(&options),
        None,
        || true,
        Rc::new(on_primary),
        DialogExtras {
            secondary,
            input: None,
        },
        Rc::new(on_dismiss),
    );
}

/// Ask for a short value in the shared confirm modal: a single-line field
/// above the buttons, with `options.body` as the hint under it. The primary
/// button stays disabled while `field` reports a problem; `on_submit`
/// receives the field's text. The caller keeps `field` to read or set the
/// text while the modal is open. The modal closes without running any
/// callback once `keep_open_while` turns false.
#[allow(
    dead_code,
    reason = "the separately compiled application binary owns the alias editor"
)]
#[expect(
    clippy::too_many_arguments,
    reason = "one callback per modal outcome plus the owner's lifetime and field"
)]
pub(crate) fn open_parent_input_dialog(
    window: &mut Window,
    cx: &mut App,
    options: ParentActionDialogOptions,
    field: SharedConfirmTextField,
    keep_open_while: impl Fn() -> bool + 'static,
    on_submit: impl Fn(String, &mut Window, &mut App) + 'static,
    on_secondary: impl Fn(&mut Window, &mut App) + 'static,
    on_dismiss: impl Fn(&mut Window, &mut App) + 'static,
) {
    let secondary = options
        .secondary_text
        .clone()
        .map(|label| (label, Rc::new(on_secondary) as ConfirmCallback));
    let submitted = field.clone();
    open_parent_confirm_dialog_with_lifecycle_and_parent(
        window,
        cx,
        action_dialog_popup_options(&options),
        None,
        keep_open_while,
        Rc::new(move |window, cx| {
            let text = submitted.borrow().text().to_string();
            on_submit(text, window, cx);
        }),
        DialogExtras {
            secondary,
            input: Some(field),
        },
        Rc::new(on_dismiss),
    );
}

fn action_dialog_popup_options(options: &ParentActionDialogOptions) -> ParentConfirmOptions {
    ParentConfirmOptions {
        title: options.title.clone(),
        body: options.body.clone(),
        confirm_text: options.primary_text.clone(),
        cancel_text: options.dismiss_text.clone(),
        risk: ConfirmRisk::Safe,
        width: options.width,
    }
}

/// Optional parts of a modal beyond primary and dismiss.
#[derive(Default)]
struct DialogExtras {
    secondary: Option<(SharedString, ConfirmCallback)>,
    input: Option<SharedConfirmTextField>,
}

#[expect(
    clippy::too_many_arguments,
    reason = "the shared parent-dialog adapter preserves separately owned lifecycle, callbacks, and actions"
)]
fn open_parent_confirm_dialog_with_lifecycle_and_parent(
    window: &mut Window,
    cx: &mut App,
    options: ParentConfirmOptions,
    explicit_parent_automation_id: Option<String>,
    keep_open_while: impl Fn() -> bool + 'static,
    on_confirm: ConfirmCallback,
    extras: DialogExtras,
    on_cancel: ConfirmCallback,
) {
    if super::window::is_confirm_window_open() {
        tracing::info!(
            event = "parent_confirm_dialog_queued",
            title = %options.title,
            waiting = queued_modal_count() + 1,
            "parent_confirm_dialog_queued"
        );
        let parent = window.window_handle();
        QUEUED_MODALS.with_borrow_mut(|queue| {
            queue.push_back(Box::new(move |cx| {
                if !keep_open_while() {
                    return;
                }
                let _ = parent.update(cx, move |_, window, cx| {
                    open_parent_confirm_dialog_with_lifecycle_and_parent(
                        window,
                        cx,
                        options,
                        explicit_parent_automation_id,
                        keep_open_while,
                        on_confirm,
                        extras,
                        on_cancel,
                    );
                });
            }));
        });
        return;
    }

    let has_lifecycle_predicate = true;
    tracing::info!(
        event = "parent_confirm_dialog_opened",
        title = %options.title,
        has_lifecycle_predicate,
        "parent_confirm_dialog_opened"
    );

    let width_value: f32 = options.width.into();
    tracing::info!(
        event = "parent_confirm_dialog_building",
        title = %options.title,
        width = width_value,
        "parent_confirm_dialog_building"
    );

    let host_policy = if window.is_owned_hidden() {
        crate::runtime_policy::WindowHostPolicy::OwnedHidden
    } else {
        crate::runtime_policy::WindowHostPolicy::Interactive
    };
    if !host_policy.is_hidden() {
        window.activate_window();
    }

    let parent_bounds = window.bounds();
    let display_id = window.display(cx).map(|d| d.id());
    let parent_window_handle = window.window_handle();
    let parent_info = match &explicit_parent_automation_id {
        Some(id) => crate::windows::automation_window_by_id(id),
        None => crate::windows::list_automation_windows()
            .into_iter()
            .find(|info| {
                info.generation.is_some_and(|generation| {
                    crate::windows::get_runtime_window_handle_for_generation(&info.id, generation)
                        == Some(parent_window_handle)
                })
            }),
    };
    let Some(parent_info) = parent_info else {
        tracing::error!("confirm_parent_identity_missing");
        return;
    };
    let Some(parent_generation) = parent_info.generation else {
        tracing::error!("confirm_parent_generation_missing");
        return;
    };
    let parent_automation_id = parent_info.id;
    let parent_id_for_result = parent_automation_id.clone();
    // A modal never outlives its parent window: when the parent closes
    // (Notes toggled away, detached Agent Chat closed), close the modal
    // instead of leaving an orphan popup that still claims the keyboard.
    let keep_open_while: Rc<dyn Fn() -> bool> = {
        let parent_id = parent_automation_id.clone();
        Rc::new(move || {
            crate::windows::get_runtime_window_handle_for_generation(&parent_id, parent_generation)
                == Some(parent_window_handle)
                && keep_open_while()
        })
    };

    let (result_tx, result_rx) = async_channel::bounded::<ParentDialogResult>(1);

    let on_confirm_for_task = on_confirm.clone();
    let on_cancel_for_task = on_cancel.clone();
    let (secondary_text, on_secondary_for_task) = extras.secondary.unzip();

    tracing::info!(
        target: "script_kit::confirm",
        event = "parent_confirm_spawning_result_task",
        "Spawning async task to await confirm popup result"
    );

    cx.spawn(async move |cx: &mut gpui::AsyncApp| {
        tracing::info!(
            target: "script_kit::confirm",
            event = "parent_confirm_result_task_waiting",
            "Result task: waiting for confirm popup result..."
        );

        let result = match result_rx.recv().await {
            Ok(result) => result,
            Err(error) => {
                tracing::error!(%error, "confirm_result_channel_closed");
                return;
            }
        };

        tracing::info!(
            target: "script_kit::confirm",
            event = "parent_confirm_result_received",
            result = ?result,
            "Result task: received result from confirm popup"
        );

        if crate::windows::get_runtime_window_handle_for_generation(
            &parent_id_for_result,
            parent_generation,
        ) != Some(parent_window_handle)
        {
            return;
        }
        let update_result = cx.update_window(parent_window_handle, move |_, parent_window, cx| {
            tracing::info!(
                target: "script_kit::confirm",
                event = "parent_confirm_activating_parent",
                result = ?result,
                "Result task: re-activating parent window and calling callback"
            );

            if !host_policy.is_hidden() {
                parent_window.activate_window();
            }

            match result {
                ParentDialogResult::Primary => on_confirm_for_task(parent_window, cx),
                ParentDialogResult::Secondary => {
                    if let Some(callback) = on_secondary_for_task {
                        callback(parent_window, cx);
                    } else {
                        on_cancel_for_task(parent_window, cx);
                    }
                }
                ParentDialogResult::Dismiss => on_cancel_for_task(parent_window, cx),
                ParentDialogResult::ProgrammaticClose => {}
            }
        });

        if let Err(error) = update_result {
            tracing::error!(
                target: "script_kit::confirm",
                event = "parent_confirm_update_window_failed",
                result = ?result,
                error = ?error,
                "Result task: failed to update parent window"
            );
        }
    })
    .detach();

    let popup_options = ConfirmWindowOptions {
        title: options.title,
        body: options.body,
        confirm_text: options.confirm_text,
        cancel_text: options.cancel_text,
        secondary_text,
        risk: options.risk,
        width: options.width,
        input: extras.input,
    };

    match open_confirm_popup_window(
        cx,
        ConfirmPopupParentWindow {
            handle: parent_window_handle,
            bounds: parent_bounds,
            display_id,
            automation_id: Some(parent_automation_id),
        },
        popup_options,
        keep_open_while,
        result_tx,
        host_policy,
    ) {
        Ok(_handle) => {
            tracing::info!(
                target: "script_kit::confirm",
                event = "parent_confirm_popup_opened_ok",
                "Confirm popup window opened successfully"
            );
        }
        Err(error) => {
            tracing::error!(
                target: "script_kit::confirm",
                event = "parent_confirm_dialog_open_failed",
                error = ?error,
                "parent_confirm_dialog_open_failed"
            );
        }
    }
}

/// Open a native confirmation popup centered over the caller window and return
/// whether the user confirmed. Uses the global main window handle from async
/// contexts that only have `&mut AsyncApp`.
///
/// Returns `Ok(true)` if the user confirms, `Ok(false)` if they cancel, close,
/// or the dialog cannot be opened.
#[allow(dead_code)]
pub(crate) async fn confirm_with_parent_dialog(
    cx: &mut gpui::AsyncApp,
    options: ParentConfirmOptions,
    trace_id: &str,
) -> anyhow::Result<bool> {
    tracing::info!(
        category = "UI",
        trace_id = %trace_id,
        event = "confirm_modal_open",
        title = %options.title,
        "Opening confirmation modal"
    );

    let (confirm_tx, confirm_rx) = async_channel::bounded::<bool>(1);

    let window_handle = crate::get_main_window_handle()
        .ok_or_else(|| anyhow::anyhow!("Main window handle not available"))?;

    let sender_ok = confirm_tx.clone();
    let sender_cancel = confirm_tx;

    cx.update_window(window_handle, move |_any_view, window, cx| {
        tracing::info!(
            target: "script_kit::confirm",
            event = "confirm_route_decision",
            "Opening native parent-attached confirm popup"
        );

        open_parent_confirm_dialog(
            window,
            cx,
            options,
            move |_window, _cx| {
                let _ = sender_ok.try_send(true);
            },
            move |_window, _cx| {
                let _ = sender_cancel.try_send(false);
            },
        );
    })?;

    let confirmed = confirm_rx.recv().await.unwrap_or(false);
    tracing::info!(
        category = "UI",
        trace_id = %trace_id,
        event = "confirm_modal_result",
        confirmed,
        "Confirmation modal resolved"
    );
    Ok(confirmed)
}

#[cfg(test)]
mod tests {
    use std::fs;

    fn normalize_ws(source: &str) -> String {
        source.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    #[test]
    fn parent_confirm_dialog_activates_window_before_opening_native_popup() {
        let source = fs::read_to_string("src/confirm/parent_dialog.rs")
            .expect("Failed to read src/confirm/parent_dialog.rs");
        let normalized = normalize_ws(&source);

        let activate_idx = normalized
            .find("window.activate_window();")
            .expect("parent confirm dialog should activate the parent window");
        let open_idx = normalized
            .find("open_confirm_popup_window(")
            .expect("parent confirm dialog should open the native confirm popup window");

        assert!(
            activate_idx < open_idx,
            "parent confirm dialog should activate the window before opening the native popup"
        );
    }

    #[test]
    fn parent_confirm_dialog_no_longer_uses_window_open_dialog() {
        let source = fs::read_to_string("src/confirm/parent_dialog.rs")
            .expect("Failed to read src/confirm/parent_dialog.rs");
        // Only scan production code — strip everything after #[cfg(test)]
        let production_source = source.split("#[cfg(test)]").next().unwrap_or(&source);
        let normalized = normalize_ws(production_source);

        let legacy_marker = ["window", ".open_dialog("].concat();
        assert!(
            !normalized.contains(&legacy_marker),
            "parent confirm dialog should no longer use in-window open_dialog"
        );
    }

    #[test]
    fn parent_confirm_dialog_passes_parent_bounds_to_popup_window() {
        let source = fs::read_to_string("src/confirm/parent_dialog.rs")
            .expect("Failed to read src/confirm/parent_dialog.rs");
        let normalized = normalize_ws(&source);

        assert!(
            normalized.contains("let parent_bounds = window.bounds();"),
            "parent confirm dialog should capture parent bounds for centering"
        );
        assert!(
            normalized.contains("open_confirm_popup_window("),
            "parent confirm dialog should open the native popup window"
        );
    }

    #[test]
    fn parent_confirm_dialog_logs_build_width_before_render() {
        let source = fs::read_to_string("src/confirm/parent_dialog.rs")
            .expect("Failed to read src/confirm/parent_dialog.rs");
        let normalized = normalize_ws(&source);

        assert!(
            normalized.contains("event = \"parent_confirm_dialog_building\"")
                && normalized.contains("width = width_value,"),
            "parent confirm dialog should log the concrete width used to build the dialog"
        );
    }

    #[test]
    fn quit_action_and_builtin_quit_share_shutdown_cleanup() {
        let scripts_source = fs::read_to_string("src/app_actions/handle_action/scripts.rs")
            .expect("Failed to read scripts.rs");
        let builtin_source = fs::read_to_string("src/app_execute/builtin_execution.rs")
            .expect("Failed to read builtin_execution.rs");

        let scripts = normalize_ws(&scripts_source);
        let builtins = normalize_ws(&builtin_source);

        assert!(
            scripts.contains("Self::quit_script_kit_confirm_options()"),
            "direct quit action should keep using the shared quit confirm copy"
        );

        let helper_idx = builtins
            .find("fn prepare_script_kit_shutdown()")
            .expect("Expected shared quit shutdown helper");
        let kill_idx = builtins
            .find("PROCESS_MANAGER.kill_all_processes();")
            .expect("Expected helper to stop running processes");
        let remove_pid_idx = builtins
            .find("PROCESS_MANAGER.remove_main_pid();")
            .expect("Expected helper to clear the main pid");

        assert!(
            helper_idx < kill_idx && kill_idx < remove_pid_idx,
            "shared quit shutdown helper should stop processes before clearing the main pid"
        );

        let action_helper_idx = scripts
            .find("Self::prepare_script_kit_shutdown();")
            .expect("Expected direct quit action to use the shared shutdown helper");
        let action_quit_idx = scripts[action_helper_idx..]
            .find("cx.quit();")
            .map(|idx| action_helper_idx + idx)
            .expect("Expected direct quit action to quit after shutdown cleanup");

        assert!(
            action_helper_idx < action_quit_idx,
            "direct quit action should run shutdown cleanup before quitting"
        );

        let builtin_branch_idx = builtins
            .find("SystemActionType::QuitScriptKit => {")
            .expect("Expected builtin quit system action branch");
        let builtin_helper_idx = builtins[builtin_branch_idx..]
            .find("Self::prepare_script_kit_shutdown();")
            .map(|idx| builtin_branch_idx + idx)
            .expect("Expected builtin quit to use the shared shutdown helper");
        let builtin_quit_idx = builtins[builtin_helper_idx..]
            .find("cx.quit();")
            .map(|idx| builtin_helper_idx + idx)
            .expect("Expected builtin quit to call cx.quit() after cleanup");

        assert!(
            builtin_helper_idx < builtin_quit_idx,
            "builtin quit should run shutdown cleanup before quitting"
        );
    }
}
