// ============================================================================
// Screenshot Capture
// ============================================================================

use image::codecs::png::PngEncoder;
use image::ImageEncoder;
use xcap::Window;

fn capture_and_encode_png(
    window: &xcap::Window,
    hi_dpi: bool,
) -> Result<(Vec<u8>, u32, u32), Box<dyn std::error::Error + Send + Sync>> {
    const DOWNSCALE_DIVISOR: u32 = 2;

    let image = window.capture_image()?;
    let original_width = image.width();
    let original_height = image.height();

    let (final_image, width, height) = if hi_dpi {
        (image, original_width, original_height)
    } else {
        let new_width = (original_width / DOWNSCALE_DIVISOR).max(1);
        let new_height = (original_height / DOWNSCALE_DIVISOR).max(1);
        let resized = image::imageops::resize(
            &image,
            new_width,
            new_height,
            image::imageops::FilterType::Lanczos3,
        );
        tracing::debug!(
            original_width = original_width,
            original_height = original_height,
            new_width = new_width,
            new_height = new_height,
            downscale_divisor = DOWNSCALE_DIVISOR,
            "Scaled screenshot to 1x resolution"
        );
        (resized, new_width, new_height)
    };

    let mut png_data = Vec::new();
    let encoder = PngEncoder::new(&mut png_data);
    encoder.write_image(&final_image, width, height, image::ExtendedColorType::Rgba8)?;

    Ok((png_data, width, height))
}

// ── Shared candidate-selection infrastructure ───────────────────────────

#[derive(Clone)]
struct Candidate {
    window: Window,
    title: String,
    app_name: String,
    focused: bool,
    width: i32,
    height: i32,
}

/// Enumerate all visible Script Kit OS windows that are large enough to be
/// meaningful screenshot targets.
fn list_script_kit_candidates() -> Result<Vec<Candidate>, Box<dyn std::error::Error + Send + Sync>> {
    let mut candidates = Vec::new();
    for window in Window::all()? {
        let title = window.title().unwrap_or_else(|_| String::new());
        let app_name = window.app_name().unwrap_or_else(|_| String::new());
        let focused = window.is_focused().unwrap_or(false);
        let is_minimized = window.is_minimized().unwrap_or(true);
        let width = window.width().unwrap_or(0);
        let height = window.height().unwrap_or(0);

        let is_our_window = app_name.contains("script-kit-gpui")
            || app_name == "Script Kit"
            || title.contains("Script Kit");

        // Width >= 200 filters out small UI elements
        // Height >= 50 allows compact prompts (arg prompt without choices is ~76px)
        let is_reasonable_size = width >= 200 && height >= 50;

        if is_our_window && !is_minimized && is_reasonable_size {
            candidates.push(Candidate {
                window,
                title,
                app_name,
                focused,
                width: width as i32,
                height: height as i32,
            });
        }
    }
    Ok(candidates)
}

/// Score how well a candidate's dimensions match the resolved target's bounds.
///
/// Returns a bonus/penalty based on dimension proximity. Exact match gets the
/// highest bonus; large deviations get a penalty to push mismatched windows
/// below better candidates.
fn candidate_size_score(
    resolved: &crate::protocol::AutomationWindowInfo,
    candidate: &Candidate,
) -> i32 {
    let Some(bounds) = resolved.bounds.as_ref() else {
        return 0;
    };

    let target_w = bounds.width.round() as i32;
    let target_h = bounds.height.round() as i32;

    let dw = (candidate.width - target_w).abs();
    let dh = (candidate.height - target_h).abs();

    match (dw, dh) {
        (0, 0) => 5_000,
        (dw, dh) if dw <= 4 && dh <= 4 => 2_500,
        (dw, dh) if dw <= 16 && dh <= 16 => 500,
        _ => -1_500,
    }
}

/// Score an OS window candidate against a resolved automation target.
///
/// Higher scores mean a better match. Uses bounds as a first-class signal
/// when available, plus title and focus agreement. The resolved target's
/// metadata drives selection.
fn score_candidate(
    resolved: &crate::protocol::AutomationWindowInfo,
    candidate: &Candidate,
) -> i32 {
    use crate::protocol::AutomationWindowKind;

    let mut score: i32 = candidate_size_score(resolved, candidate);

    // Exact title match is a strong signal
    if let Some(title) = resolved.title.as_deref() {
        if !title.is_empty() && candidate.title == title {
            score += 1_000;
        } else if !title.is_empty() && candidate.title.contains(title) {
            score += 500;
        }
    }

    // Focus agreement
    if resolved.focused == candidate.focused {
        score += 100;
    }

    // For main-window targets, penalize candidates that are clearly secondary
    // windows (Notes, AI) so we never accidentally prefer them.
    if resolved.kind == AutomationWindowKind::Main
        && (candidate.title.contains("Notes") || candidate.title.contains("AI"))
    {
        score -= 200;
    }

    score
}

/// Select the best-matching OS window candidate for a resolved automation target.
///
/// Returns a hard error when no OS window matches or when the top two
/// candidates tie (ambiguous match). Emits structured logs on every
/// successful selection and on ambiguous rejection so agents can audit
/// which OS window was actually selected.
///
/// This is the single candidate-ranking path reused by PNG capture,
/// RGBA capture, and native `os_window_id` resolution.
fn select_best_candidate<'a>(
    resolved: &crate::protocol::AutomationWindowInfo,
    candidates: &'a [Candidate],
    caller: &str,
) -> Result<&'a Candidate, Box<dyn std::error::Error + Send + Sync>> {
    let mut ranked: Vec<(i32, &Candidate)> = candidates
        .iter()
        .map(|candidate| (score_candidate(resolved, candidate), candidate))
        .collect();

    ranked.sort_by(|(left_score, left), (right_score, right)| {
        right_score
            .cmp(left_score)
            .then_with(|| right.focused.cmp(&left.focused))
            .then_with(|| right.title.cmp(&left.title))
    });

    let Some((best_score, best)) = ranked.first().copied() else {
        return Err(
            "No visible Script Kit windows available for screenshot capture"
                .to_string()
                .into(),
        );
    };

    if best_score <= 0 {
        return Err(format!(
            "No OS window matched automation target {} ({:?}) \
             strongly enough for deterministic capture",
            resolved.id, resolved.kind
        )
        .into());
    }

    // Reject tied top candidates instead of guessing
    if let Some((second_score, second)) = ranked.get(1).copied() {
        if second_score == best_score {
            tracing::warn!(
                target: "script_kit::automation",
                window_id = %resolved.id,
                kind = ?resolved.kind,
                caller = %caller,
                first_title = %best.title,
                first_size = %format!("{}x{}", best.width, best.height),
                second_title = %second.title,
                second_size = %format!("{}x{}", second.width, second.height),
                score = best_score,
                "automation.capture_screenshot.ambiguous_candidate"
            );

            return Err(format!(
                "Ambiguous OS window match for automation target {} ({:?}); \
                 '{}' and '{}' tied at score {}",
                resolved.id, resolved.kind, best.title, second.title, best_score
            )
            .into());
        }
    }

    tracing::info!(
        target: "script_kit::automation",
        window_id = %resolved.id,
        kind = ?resolved.kind,
        caller = %caller,
        requested_title = ?resolved.title,
        requested_bounds = ?resolved.bounds,
        candidate_count = ranked.len(),
        selected_title = %best.title,
        selected_app = %best.app_name,
        selected_focused = best.focused,
        selected_width = best.width,
        selected_height = best.height,
        selected_score = best_score,
        "automation.capture_screenshot.candidate_selected"
    );

    Ok(best)
}

/// Capture the OS window that best matches the resolved automation target.
///
/// Delegates to `select_best_candidate` for the single shared ranking path.
fn capture_resolved_window(
    resolved: &crate::protocol::AutomationWindowInfo,
    hi_dpi: bool,
) -> Result<(Vec<u8>, u32, u32), Box<dyn std::error::Error + Send + Sync>> {
    let candidates = list_script_kit_candidates()?;
    let best = select_best_candidate(resolved, &candidates, "capture_png")?;
    capture_and_encode_png(&best.window, hi_dpi)
}

// ── Popup capture receipt ──────────────────────────────────────────────

/// Strategy used to capture a popup window for verification.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PopupCaptureStrategy {
    /// Captured the parent window and crop to popup bounds.
    ParentCaptureWithCrop,
    /// Captured the detached window directly.
    DirectWindowCapture,
    /// Not a popup target (main window or unknown kind).
    NotApplicable,
}

/// Deterministic receipt for popup screenshot capture.
///
/// Always included in targeted capture results so agents can distinguish
/// how a popup was captured and whether the crop is trustworthy.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PopupCaptureReceipt {
    /// The capture strategy used.
    pub strategy: PopupCaptureStrategy,
    /// The automation window kind that determined the strategy.
    pub window_kind: String,
    /// Crop bounds within the screenshot (null for detached or when not applicable).
    pub target_bounds: Option<crate::protocol::InspectBoundsInScreenshot>,
    /// Whether semantic receipts (not screenshots) are the primary verification oracle.
    pub semantic_receipts_are_primary: bool,
}

/// Returns `true` for window kinds that are attached popups (rendered
/// inside the parent window's coordinate space).
fn is_attached_popup(kind: crate::protocol::AutomationWindowKind) -> bool {
    matches!(
        kind,
        crate::protocol::AutomationWindowKind::ActionsDialog
            | crate::protocol::AutomationWindowKind::PromptPopup
    )
}

/// Returns `true` for window kinds that are detached (own OS window).
fn is_detached_window(kind: crate::protocol::AutomationWindowKind) -> bool {
    matches!(
        kind,
        crate::protocol::AutomationWindowKind::AcpDetached
            | crate::protocol::AutomationWindowKind::Notes
    )
}

/// Capture a screenshot of a targeted window and produce a popup capture receipt.
///
/// For **attached popups** (ActionsDialog, PromptPopup):
/// - Resolves the parent window identity and popup bounds.
/// - Captures the parent window and computes crop bounds.
/// - Emits `automation.capture_screenshot.parent_crop_selected` on success.
/// - **Fails loudly** with `automation.capture_screenshot.parent_crop_failed`
///   if parent identity or popup bounds are missing — never returns an
///   unscoped whole-window screenshot.
///
/// For **detached windows** (AcpDetached, Notes):
/// - Captures the window directly.
///
/// For **other targets** (Main, etc.):
/// - Standard capture with `not_applicable` strategy.
#[allow(clippy::type_complexity)]
pub fn capture_targeted_screenshot_with_popup_receipt(
    target: Option<&crate::protocol::AutomationWindowTarget>,
    hi_dpi: bool,
) -> Result<(Vec<u8>, u32, u32, PopupCaptureReceipt), Box<dyn std::error::Error + Send + Sync>> {
    let resolved = match crate::windows::resolve_automation_window(target) {
        Ok(info) => info,
        Err(err) => {
            tracing::warn!(
                target: "script_kit::automation",
                error = %err,
                target = ?target,
                "automation.capture_screenshot.target_failed"
            );
            return Err(err.to_string().into());
        }
    };

    let kind = resolved.kind;
    let kind_str = format!("{kind:?}");

    if is_attached_popup(kind) {
        // Attached popup: require parent identity and bounds.
        let parent_id = match resolved.parent_window_id.as_ref() {
            Some(pid) => pid.clone(),
            None => {
                tracing::error!(
                    target: "script_kit::automation",
                    window_id = %resolved.id,
                    kind = %kind_str,
                    "automation.capture_screenshot.parent_crop_failed: \
                     attached popup has no parent_window_id in automation registry"
                );
                return Err(format!(
                    "Attached popup '{}' ({}) cannot be captured: \
                     no parent window identity registered. \
                     Refusing to fall back to unscoped whole-window screenshot.",
                    resolved.id, kind_str
                )
                .into());
            }
        };

        // Resolve parent window metadata
        let parent_target = crate::protocol::AutomationWindowTarget::Id { id: parent_id.clone() };
        let parent_resolved = match crate::windows::resolve_automation_window(Some(&parent_target)) {
            Ok(p) => p,
            Err(err) => {
                tracing::error!(
                    target: "script_kit::automation",
                    window_id = %resolved.id,
                    kind = %kind_str,
                    parent_id = %parent_id,
                    error = %err,
                    "automation.capture_screenshot.parent_crop_failed: \
                     parent window not found in automation registry"
                );
                return Err(format!(
                    "Attached popup '{}' ({}) cannot be captured: \
                     parent '{}' not found in registry. \
                     Refusing to fall back to unscoped whole-window screenshot.",
                    resolved.id, kind_str, parent_id
                )
                .into());
            }
        };

        // Compute crop bounds
        let target_bounds = crate::protocol::target_bounds_in_screenshot_with_main(
            &resolved,
            parent_resolved.bounds.as_ref(),
        );

        if target_bounds.is_none() {
            tracing::error!(
                target: "script_kit::automation",
                window_id = %resolved.id,
                kind = %kind_str,
                parent_id = %parent_id,
                popup_bounds = ?resolved.bounds,
                parent_bounds = ?parent_resolved.bounds,
                "automation.capture_screenshot.parent_crop_failed: \
                 popup or parent bounds missing, cannot compute crop region"
            );
            return Err(format!(
                "Attached popup '{}' ({}) cannot be captured: \
                 bounds geometry unavailable (popup bounds: {}, parent bounds: {}). \
                 Refusing to produce unscoped whole-window screenshot.",
                resolved.id,
                kind_str,
                if resolved.bounds.is_some() { "present" } else { "missing" },
                if parent_resolved.bounds.is_some() { "present" } else { "missing" },
            )
            .into());
        }

        // Capture the parent window
        tracing::info!(
            target: "script_kit::automation",
            window_id = %resolved.id,
            kind = %kind_str,
            parent_id = %parent_id,
            hi_dpi = hi_dpi,
            "automation.capture_screenshot.targeted"
        );

        let (png_data, width, height) = capture_resolved_window(&parent_resolved, hi_dpi)?;

        tracing::info!(
            target: "script_kit::automation",
            window_id = %resolved.id,
            kind = %kind_str,
            parent_id = %parent_id,
            parent_width = width,
            parent_height = height,
            crop_bounds = ?target_bounds,
            "automation.capture_screenshot.parent_crop_selected"
        );

        let receipt = PopupCaptureReceipt {
            strategy: PopupCaptureStrategy::ParentCaptureWithCrop,
            window_kind: kind_str,
            target_bounds,
            semantic_receipts_are_primary: true,
        };

        Ok((png_data, width, height, receipt))
    } else if is_detached_window(kind) {
        // Detached window: direct capture
        tracing::info!(
            target: "script_kit::automation",
            window_id = %resolved.id,
            kind = %kind_str,
            hi_dpi = hi_dpi,
            "automation.capture_screenshot.targeted"
        );

        let (png_data, width, height) = capture_resolved_window(&resolved, hi_dpi)?;

        let receipt = PopupCaptureReceipt {
            strategy: PopupCaptureStrategy::DirectWindowCapture,
            window_kind: kind_str,
            target_bounds: None,
            semantic_receipts_are_primary: true,
        };

        Ok((png_data, width, height, receipt))
    } else {
        // Main or other: standard capture
        tracing::info!(
            target: "script_kit::automation",
            window_id = %resolved.id,
            kind = %kind_str,
            hi_dpi = hi_dpi,
            "automation.capture_screenshot.targeted"
        );

        let (png_data, width, height) = capture_resolved_window(&resolved, hi_dpi)?;

        let receipt = PopupCaptureReceipt {
            strategy: PopupCaptureStrategy::NotApplicable,
            window_kind: kind_str,
            target_bounds: None,
            semantic_receipts_are_primary: false,
        };

        Ok((png_data, width, height, receipt))
    }
}

// ── Public API ──────────────────────────────────────────────────────────

/// Capture a screenshot of the main app window.
///
/// Resolves the main automation target first, then uses the shared
/// candidate-selection path. Does **not** preferentially capture Notes or
/// AI windows by heuristic — the resolved target drives selection.
///
/// # Arguments
/// * `hi_dpi` - If true, return full retina resolution (2x). If false, scale down to 1x.
pub fn capture_app_screenshot(
    hi_dpi: bool,
) -> Result<(Vec<u8>, u32, u32), Box<dyn std::error::Error + Send + Sync>> {
    let target = crate::protocol::AutomationWindowTarget::Main;
    let resolved = crate::windows::resolve_automation_window(Some(&target))
        .map_err(|err| err.to_string())?;
    capture_resolved_window(&resolved, hi_dpi)
}

/// Capture a screenshot of a window by its title pattern.
///
/// Similar to `capture_app_screenshot` but allows specifying which window to capture
/// by matching the title. This is useful for secondary windows like the AI Chat window.
///
/// # Arguments
/// * `title_pattern` - A string that the window title must contain
/// * `hi_dpi` - If true, return full retina resolution (2x). If false, scale down to 1x.
///
/// # Returns
/// A tuple of (png_data, width, height) on success.
pub fn capture_window_by_title(
    title_pattern: &str,
    hi_dpi: bool,
) -> Result<(Vec<u8>, u32, u32), Box<dyn std::error::Error + Send + Sync>> {
    let windows = Window::all()?;

    for window in windows {
        let title = window.title().unwrap_or_else(|_| String::new());
        let app_name = window.app_name().unwrap_or_else(|_| String::new());

        // Match window by title pattern (must also be our app)
        let is_our_app = app_name.contains("script-kit-gpui") || app_name == "Script Kit";
        let title_matches = title.contains(title_pattern);
        let is_minimized = window.is_minimized().unwrap_or(true);
        // Skip tiny windows (e.g. tray icon) when using empty title pattern
        let win_width = window.width().unwrap_or(0);
        let win_height = window.height().unwrap_or(0);
        let is_too_small = win_width < 100 || win_height < 100;

        if is_our_app && title_matches && !is_minimized && !is_too_small {
            tracing::debug!(
                app_name = %app_name,
                title = %title,
                title_pattern = %title_pattern,
                hi_dpi = hi_dpi,
                "Found window matching title pattern for screenshot"
            );

            let (png_data, width, height) = capture_and_encode_png(&window, hi_dpi)?;

            tracing::debug!(
                width = width,
                height = height,
                hi_dpi = hi_dpi,
                file_size = png_data.len(),
                title_pattern = %title_pattern,
                "Screenshot captured for window by title"
            );

            return Ok((png_data, width, height));
        }
    }

    Err(format!("Window with title containing '{}' not found", title_pattern).into())
}

/// Capture a screenshot routed through the automation window target resolver.
///
/// Always captures through the resolved `AutomationWindowInfo` metadata
/// using the shared candidate-selection path. Returns a hard error when
/// no OS window matches the resolved target — never silently falls back
/// to the main window.
///
/// # Arguments
/// * `target` - The automation window target. `None` defaults to `Focused`.
/// * `hi_dpi` - If true, return full retina resolution (2x). If false, scale down to 1x.
pub fn capture_targeted_screenshot(
    target: Option<&crate::protocol::AutomationWindowTarget>,
    hi_dpi: bool,
) -> Result<(Vec<u8>, u32, u32), Box<dyn std::error::Error + Send + Sync>> {
    let resolved = match crate::windows::resolve_automation_window(target) {
        Ok(info) => info,
        Err(err) => {
            tracing::warn!(
                target: "script_kit::automation",
                error = %err,
                target = ?target,
                "automation.capture_screenshot.target_failed"
            );
            return Err(err.to_string().into());
        }
    };

    tracing::info!(
        target: "script_kit::automation",
        window_id = %resolved.id,
        kind = ?resolved.kind,
        hi_dpi = hi_dpi,
        "automation.capture_screenshot.targeted"
    );

    capture_resolved_window(&resolved, hi_dpi)
}

/// Capture a window screenshot using the resolver-driven path, translating a title
/// pattern into an `AutomationWindowTarget` for deterministic capture.
///
/// This is the replacement for direct `capture_window_by_title` calls from stdin/runtime
/// paths. Empty titles resolve to `AutomationWindowTarget::Main`; non-empty titles
/// resolve to `AutomationWindowTarget::TitleContains`.
///
/// Emits an explicit compatibility log (`automation.capture_screenshot.title_compatibility`)
/// before delegating to `capture_targeted_screenshot`.
pub fn capture_window_by_title_via_resolver(
    title_pattern: &str,
    hi_dpi: bool,
) -> Result<(Vec<u8>, u32, u32), Box<dyn std::error::Error + Send + Sync>> {
    let target = if title_pattern.trim().is_empty() {
        crate::protocol::AutomationWindowTarget::Main
    } else {
        crate::protocol::AutomationWindowTarget::TitleContains {
            text: title_pattern.to_string(),
        }
    };

    tracing::info!(
        target: "script_kit::automation",
        title_pattern = %title_pattern,
        resolved_target = ?target,
        "automation.capture_screenshot.title_compatibility"
    );

    capture_targeted_screenshot(Some(&target), hi_dpi)
}

/// Capture a raw RGBA image of the OS window matching the resolved automation target.
///
/// Returns the raw `image::RgbaImage` (not PNG-encoded) so callers can
/// extract dimensions and sample individual pixels without a decode step.
/// Used by `inspectAutomationWindow` for lightweight pixel probes.
///
/// Delegates to `select_best_candidate` for the single shared ranking path,
/// emitting the same ambiguity/selection logs as PNG capture and OS window ID
/// resolution.
pub fn capture_targeted_rgba_image(
    target: Option<&crate::protocol::AutomationWindowTarget>,
    hi_dpi: bool,
) -> Result<image::RgbaImage, Box<dyn std::error::Error + Send + Sync>> {
    const DOWNSCALE_DIVISOR: u32 = 2;

    let resolved = crate::windows::resolve_automation_window(target)
        .map_err(|err| err.to_string())?;

    let candidates = list_script_kit_candidates()?;
    let best = select_best_candidate(&resolved, &candidates, "capture_rgba")?;

    let rgba_image = best.window.capture_image()?;

    if hi_dpi {
        Ok(rgba_image)
    } else {
        let new_width = (rgba_image.width() / DOWNSCALE_DIVISOR).max(1);
        let new_height = (rgba_image.height() / DOWNSCALE_DIVISOR).max(1);
        Ok(image::imageops::resize(
            &rgba_image,
            new_width,
            new_height,
            image::imageops::FilterType::Lanczos3,
        ))
    }
}

/// Resolve the best-matching OS window for an automation target and return its
/// native CGWindowID.
///
/// Delegates to `select_best_candidate` for the single shared ranking path,
/// emitting the same ambiguity/selection logs as PNG and RGBA capture.
/// Returns `None` when no candidate matches strongly enough.
pub fn resolve_targeted_os_window_id(
    target: Option<&crate::protocol::AutomationWindowTarget>,
) -> Option<u32> {
    let resolved = crate::windows::resolve_automation_window(target).ok()?;
    let candidates = list_script_kit_candidates().ok()?;
    let best = select_best_candidate(&resolved, &candidates, "resolve_os_window_id").ok()?;
    best.window.id().ok()
}
