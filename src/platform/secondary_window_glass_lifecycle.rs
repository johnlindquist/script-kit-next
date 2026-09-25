#[cfg(target_os = "macos")]
static GLASS_EXIT_GENERATIONS: std::sync::Mutex<Vec<(usize, u64)>> =
    std::sync::Mutex::new(Vec::new());

#[cfg(target_os = "macos")]
#[derive(Clone, Debug)]
struct GlassExitLifecycle {
    window_key: usize,
    window_name: String,
    native_window_number: i64,
    generation: u64,
    mode: &'static str,
    original_frame: [f64; 4],
    /// Whether a native glass host (capsule container or footer panel) was
    /// attached when the exit was REQUESTED. The "host must not detach before
    /// the exit resolves" invariant is only meaningful when this is true —
    /// Notes mode legitimately owns no capsules at all.
    host_attached_at_request: bool,
    request_host_time_ns: u64,
    fade_duration_ns: u64,
    expected_removal_deadline_ns: u64,
    cancelled_at_host_time_ns: Option<u64>,
    committed_at_host_time_ns: Option<u64>,
    removed_at_host_time_ns: Option<u64>,
    host_teardown_at_host_time_ns: Option<u64>,
    events: Vec<(&'static str, u64)>,
}

#[cfg(target_os = "macos")]
static GLASS_EXIT_LIFECYCLES: std::sync::Mutex<Vec<GlassExitLifecycle>> =
    std::sync::Mutex::new(Vec::new());

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GlassExitTicket {
    window_key: usize,
    generation: u64,
}

impl GlassExitTicket {
    pub(crate) fn generation(self) -> u64 {
        self.generation
    }
}

#[cfg(target_os = "macos")]
fn record_glass_exit_begin(window: id, ticket: GlassExitTicket, window_name: &str) {
    use cocoa::foundation::NSRect;
    let now = crate::platform::host_clock::host_time_ns();
    let frame: NSRect = unsafe { msg_send![window, frame] };
    let native_window_number: i64 = unsafe { msg_send![window, windowNumber] };
    let fade_duration_ns = (GLASS_EXIT_DURATION * 1_000_000_000.0).round() as u64;
    let deadline_ns = now.saturating_add(
        glass_exit_remove_delay()
            .as_nanos()
            .min(u128::from(u64::MAX)) as u64,
    );
    let mut records = GLASS_EXIT_LIFECYCLES
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    records.push(GlassExitLifecycle {
        window_key: ticket.window_key,
        window_name: window_name.to_string(),
        native_window_number,
        generation: ticket.generation,
        mode: "FadeOnly",
        original_frame: [
            frame.origin.x,
            frame.origin.y,
            frame.size.width,
            frame.size.height,
        ],
        host_attached_at_request: crate::platform::glass_button_host::host_attached(
            ticket.window_key,
        ) || unsafe {
            crate::footer_popup::native_footer_host_attached(window)
        },
        request_host_time_ns: now,
        fade_duration_ns,
        expected_removal_deadline_ns: deadline_ns,
        cancelled_at_host_time_ns: None,
        committed_at_host_time_ns: None,
        removed_at_host_time_ns: None,
        host_teardown_at_host_time_ns: None,
        events: vec![("ticketBegin", now)],
    });
    if records.len() > 64 {
        let drain = records.len() - 64;
        records.drain(0..drain);
    }
}

#[cfg(target_os = "macos")]
fn mutate_glass_exit_lifecycle(
    ticket: GlassExitTicket,
    event: &'static str,
    mutate: impl FnOnce(&mut GlassExitLifecycle, u64),
) {
    let now = crate::platform::host_clock::host_time_ns();
    let mut records = GLASS_EXIT_LIFECYCLES
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    if let Some(record) = records.iter_mut().rev().find(|record| {
        record.window_key == ticket.window_key && record.generation == ticket.generation
    }) {
        mutate(record, now);
        record.events.push((event, now));
        if record.events.len() > 24 {
            record.events.remove(0);
        }
    }
}

#[cfg(target_os = "macos")]
pub(crate) fn record_glass_exit_commit(ticket: GlassExitTicket) {
    mutate_glass_exit_lifecycle(ticket, "ticketCommit", |record, now| {
        record.committed_at_host_time_ns = Some(now);
        // This receipt mutation runs on the main thread immediately before
        // `remove_window`. Treat that commit point as the native removal and
        // host-teardown boundary too: after it returns, `window_key` may be a
        // dangling NSWindow pointer and must never be messaged again.
        record.removed_at_host_time_ns = Some(now);
        record.host_teardown_at_host_time_ns = Some(now);
    });
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn record_glass_exit_commit(_ticket: GlassExitTicket) {}

#[cfg(target_os = "macos")]
pub(crate) fn glass_exit_lifecycle_receipt(window_name: &str) -> serde_json::Value {
    use cocoa::foundation::NSRect;
    let record = GLASS_EXIT_LIFECYCLES
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
        .iter()
        .rev()
        .find(|record| record.window_name == window_name)
        .cloned();
    let Some(record) = record else {
        return serde_json::json!({
            "schemaVersion": 2,
            "windowName": window_name,
            "phase": "Open",
            "history": [],
        });
    };
    let (current_frame, current_alpha, filter_count, glass_host_attached) =
        if record.removed_at_host_time_ns.is_some() {
            // Do not dereference the retained raw pointer after the commit
            // boundary. The last live invariants are already represented by
            // `originalFrame`; removal guarantees alpha zero, no attached
            // host, and no surviving content-view filters.
            (record.original_frame, 0.0, 0, false)
        } else {
            let window = record.window_key as id;
            unsafe {
                let frame: NSRect = msg_send![window, frame];
                let alpha: f64 = msg_send![window, alphaValue];
                let content_view: id = msg_send![window, contentView];
                let filters: id = if content_view == nil {
                    nil
                } else {
                    let layer: id = msg_send![content_view, layer];
                    if layer == nil {
                        nil
                    } else {
                        msg_send![layer, filters]
                    }
                };
                let count: u64 = if filters == nil {
                    0
                } else {
                    msg_send![filters, count]
                };
                let attached = crate::platform::glass_button_host::host_attached(record.window_key)
                    || crate::footer_popup::native_footer_host_attached(window);
                (
                    [
                        frame.origin.x,
                        frame.origin.y,
                        frame.size.width,
                        frame.size.height,
                    ],
                    alpha,
                    count,
                    attached,
                )
            }
        };
    serde_json::json!({
        "schemaVersion": 2,
        "windowName": record.window_name,
        "nativeWindowNumber": record.native_window_number,
        "exitGeneration": record.generation,
        "exitMode": record.mode,
        "originalFrame": record.original_frame,
        "requestHostTimeNs": record.request_host_time_ns,
        "fadeDurationNs": record.fade_duration_ns,
        "expectedRemovalDeadlineNs": record.expected_removal_deadline_ns,
        "cancelledAtHostTimeNs": record.cancelled_at_host_time_ns,
        "committedAtHostTimeNs": record.committed_at_host_time_ns,
        "removedAtHostTimeNs": record.removed_at_host_time_ns,
        "hostTeardownAtHostTimeNs": record.host_teardown_at_host_time_ns,
        "currentFrame": current_frame,
        "currentAlpha": current_alpha,
        "commonContentViewFilterCount": filter_count,
        "glassHostAttached": glass_host_attached,
        "hostAttachedAtRequest": record.host_attached_at_request,
        "history": record.events.iter().map(|(event, host_time_ns)| {
            serde_json::json!({"event": event, "hostTimeNs": host_time_ns})
        }).collect::<Vec<_>>(),
    })
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn glass_exit_lifecycle_receipt(window_name: &str) -> serde_json::Value {
    serde_json::json!({"schemaVersion": 2, "windowName": window_name, "history": []})
}

#[cfg(target_os = "macos")]
fn advance_glass_exit_generation(window_key: usize) -> GlassExitTicket {
    let mut generations = GLASS_EXIT_GENERATIONS
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let generation =
        if let Some((_, generation)) = generations.iter_mut().find(|(key, _)| *key == window_key) {
            *generation = generation.wrapping_add(1).max(1);
            *generation
        } else {
            generations.push((window_key, 1));
            1
        };
    GlassExitTicket {
        window_key,
        generation,
    }
}

#[cfg(target_os = "macos")]
pub(crate) fn glass_exit_ticket_is_current(ticket: GlassExitTicket) -> bool {
    GLASS_EXIT_GENERATIONS
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
        .iter()
        .any(|(key, generation)| *key == ticket.window_key && *generation == ticket.generation)
}

#[cfg(target_os = "macos")]
unsafe fn ns_window_from_gpui_window(window: &gpui::Window) -> Option<id> {
    let handle = raw_window_handle::HasWindowHandle::window_handle(window).ok()?;
    let raw_window_handle::RawWindowHandle::AppKit(appkit) = handle.as_raw() else {
        return None;
    };
    let ns_view = appkit.ns_view.as_ptr() as id;
    let ns_window: id = msg_send![ns_view, window];
    (ns_window != nil).then_some(ns_window)
}

#[cfg(target_os = "macos")]
pub(crate) fn cancel_gpui_window_exit_dematerialize(window: &gpui::Window) -> bool {
    if require_main_thread("cancel_gpui_window_exit_dematerialize") {
        return false;
    }
    unsafe {
        let Some(ns_window) = ns_window_from_gpui_window(window) else {
            return false;
        };
        cancel_ns_window_exit_dematerialize(ns_window)
    }
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn cancel_gpui_window_exit_dematerialize(_window: &gpui::Window) -> bool {
    false
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn glass_exit_ticket_is_current(_ticket: GlassExitTicket) -> bool {
    false
}

#[cfg(target_os = "macos")]
unsafe fn cancel_pending_glass_window_selectors(window: id) {
    let coherent_active = GLASS_COHERENT_ENTRY_STATES
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .contains_key(&(window as usize));
    let visible: bool = msg_send![window, isVisible];
    if !coherent_active || !visible {
        finish_coherent_glass_entry(window, "interrupted");
    } else {
        // A fade exit must not briefly expose a full, sharp stage while the
        // window is still visible. Invalidate
        // selectors now; the retained state is restored after orderOut.
        let mut receipt = MAIN_GLASS_ENTRY_RUNTIME_RECEIPT
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(receipt) = receipt.as_mut() {
            if receipt.window_key == window as usize && receipt.outcome == "in_flight" {
                receipt.outcome = "pending_hidden_restore";
                receipt.cancelled_at_host_time_ns =
                    Some(crate::platform::host_clock::host_time_ns());
            }
        }
    }
    let content_view: id = msg_send![window, contentView];
    if content_view == nil {
        return;
    }
    let glass_view: id = msg_send![content_view, viewWithTag: TAHOE_GLASS_BACKDROP_TAG];
    if glass_view == nil {
        return;
    }
    let _: () = msg_send![
        class!(NSObject),
        cancelPreviousPerformRequestsWithTarget: glass_view
        selector: sel!(finishOwnWindowEntry)
        object: nil
    ];
}

#[cfg(target_os = "macos")]
unsafe fn cancel_ns_window_exit_dematerialize(window: id) -> bool {
    cancel_ns_window_exit_dematerialize_impl(window, true)
}

/// Cancel a pending exit. `restore_alpha = true` is the public
/// cancellation/recovery contract (the window must end up presentable at
/// full alpha). Entry animation passes `restore_alpha = false`: restoring
/// 1.0 first would flash a full-alpha frame before the entry installs its
/// start alpha.
#[cfg(target_os = "macos")]
unsafe fn cancel_ns_window_exit_dematerialize_impl(window: id, restore_alpha: bool) -> bool {
    if window == nil {
        return false;
    }
    let window_key = window as usize;
    let ticket = {
        let records = GLASS_EXIT_LIFECYCLES
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        records
            .iter()
            .rev()
            .find(|record| record.window_key == window_key)
            .map(|record| GlassExitTicket {
                window_key,
                generation: record.generation,
            })
    };
    if let Some(ticket) = ticket {
        mutate_glass_exit_lifecycle(ticket, "ticketCancel", |record, now| {
            record.cancelled_at_host_time_ns = Some(now);
        });
    }
    advance_glass_exit_generation(window_key);
    cancel_pending_glass_window_selectors(window);
    clear_exit_dematerialize_blur(window);
    let content_view: id = msg_send![window, contentView];
    if content_view != nil {
        let layer: id = msg_send![content_view, layer];
        if layer != nil {
            let _: () = msg_send![layer, removeAllAnimations];
        }
    }
    if restore_alpha {
        let _: () = msg_send![window, setAlphaValue: 1.0f64];
    }
    true
}

/// A coherent entry owns every native object it temporarily changes. Object
/// pointers in the maps are only keys; retained objects are released exactly
/// once by the finish callback or the synchronous interruption path.
#[cfg(target_os = "macos")]
struct GlassEntryLayerSnapshot {
    layer: usize,
    filters: usize,
    mask: usize,
    masks_to_bounds: bool,
    prior_blur_animation: usize,
    original_animation_keys: usize,
    owned_filters: usize,
    owned_mask: usize,
    owned_clip: Option<bool>,
    owned_animation_keys: Vec<usize>,
}

#[cfg(target_os = "macos")]
struct GlassEntryStyleSnapshot {
    view: usize,
    tint: usize,
}

#[cfg(target_os = "macos")]
struct GlassCoherentEntryState {
    backdrop_key: usize,
    window_number: i64,
    alpha_views: Vec<(usize, f64)>,
    layers: Vec<GlassEntryLayerSnapshot>,
    styles: Vec<GlassEntryStyleSnapshot>,
}

#[cfg(target_os = "macos")]
static GLASS_COHERENT_ENTRY_STATES: std::sync::LazyLock<
    std::sync::Mutex<std::collections::HashMap<usize, GlassCoherentEntryState>>,
> = std::sync::LazyLock::new(|| std::sync::Mutex::new(std::collections::HashMap::new()));

#[cfg(target_os = "macos")]
unsafe fn retain_entry_object(object: id) -> usize {
    if object == nil {
        0
    } else {
        let retained: id = msg_send![object, retain];
        retained as usize
    }
}

#[cfg(target_os = "macos")]
unsafe fn release_entry_object(object: usize) {
    if object != 0 {
        let _: () = msg_send![object as id, release];
    }
}

#[cfg(target_os = "macos")]
impl GlassCoherentEntryState {
    fn new(backdrop: id, window_number: i64) -> Self {
        Self {
            backdrop_key: backdrop as usize,
            window_number,
            alpha_views: Vec::new(),
            layers: Vec::new(),
            styles: Vec::new(),
        }
    }

    unsafe fn capture_alpha(&mut self, view: id) -> Option<f64> {
        if view == nil {
            return None;
        }
        if let Some((_, alpha)) = self
            .alpha_views
            .iter()
            .find(|(pointer, _)| *pointer == view as usize)
        {
            return Some(*alpha);
        }
        let alpha: f64 = msg_send![view, alphaValue];
        self.alpha_views.push((retain_entry_object(view), alpha));
        Some(alpha)
    }

    unsafe fn capture_layer(&mut self, view: id) -> bool {
        if view == nil {
            return false;
        }
        let layer: id = msg_send![view, layer];
        if layer == nil {
            return false;
        }
        if self
            .layers
            .iter()
            .any(|snapshot| snapshot.layer == layer as usize)
        {
            return true;
        }
        let filters: id = msg_send![layer, filters];
        let mask: id = msg_send![layer, mask];
        let masks_to_bounds: bool = msg_send![layer, masksToBounds];
        let key = tahoe_ns_string("entryBlurRamp");
        let prior_animation: id = msg_send![layer, animationForKey: key];
        let animation_keys: id = msg_send![layer, animationKeys];
        self.layers.push(GlassEntryLayerSnapshot {
            layer: retain_entry_object(layer),
            filters: retain_entry_object(filters),
            mask: retain_entry_object(mask),
            masks_to_bounds,
            prior_blur_animation: retain_entry_object(prior_animation),
            original_animation_keys: retain_entry_object(animation_keys),
            owned_filters: 0,
            owned_mask: 0,
            owned_clip: None,
            owned_animation_keys: Vec::new(),
        });
        true
    }

    unsafe fn mark_owned_filter(&mut self, view: id) {
        let layer: id = msg_send![view, layer];
        if let Some(snapshot) = self
            .layers
            .iter_mut()
            .find(|snapshot| snapshot.layer == layer as usize)
        {
            release_entry_object(snapshot.owned_filters);
            let filters: id = msg_send![layer, filters];
            snapshot.owned_filters = retain_entry_object(filters);
        }
    }

    unsafe fn mark_owned_mask(&mut self, view: id) {
        let layer: id = msg_send![view, layer];
        if let Some(snapshot) = self
            .layers
            .iter_mut()
            .find(|snapshot| snapshot.layer == layer as usize)
        {
            release_entry_object(snapshot.owned_mask);
            let mask: id = msg_send![layer, mask];
            snapshot.owned_mask = retain_entry_object(mask);
        }
    }

    unsafe fn mark_owned_clip(&mut self, view: id) {
        let layer: id = msg_send![view, layer];
        if let Some(snapshot) = self
            .layers
            .iter_mut()
            .find(|snapshot| snapshot.layer == layer as usize)
        {
            snapshot.owned_clip = Some(msg_send![layer, masksToBounds]);
        }
    }

    unsafe fn mark_added_animations(&mut self) {
        for snapshot in &mut self.layers {
            let layer = snapshot.layer as id;
            let keys: id = msg_send![layer, animationKeys];
            if keys == nil {
                continue;
            }
            let count: usize = msg_send![keys, count];
            for index in 0..count {
                let key: id = msg_send![keys, objectAtIndex: index];
                let was_present: bool = snapshot.original_animation_keys != 0
                    && msg_send![snapshot.original_animation_keys as id, containsObject: key];
                if !was_present {
                    snapshot.owned_animation_keys.push(retain_entry_object(key));
                }
            }
        }
    }

    unsafe fn capture_style(&mut self, view: id) {
        if view == nil || self.styles.iter().any(|style| style.view == view as usize) {
            return;
        }
        let tint: id = msg_send![view, tintColor];
        self.styles.push(GlassEntryStyleSnapshot {
            view: retain_entry_object(view),
            tint: retain_entry_object(tint),
        });
    }

    /// A physically destroyed NSWindow has no model state to restore; the
    /// pending selector still owns the tagged backdrop until dispatch, so
    /// release all entry retains without messaging an invalid window.
    unsafe fn discard(self) {
        for (view, _) in self.alpha_views {
            release_entry_object(view);
        }
        for style in self.styles {
            release_entry_object(style.tint);
            release_entry_object(style.view);
        }
        for layer in self.layers {
            for key in layer.owned_animation_keys {
                release_entry_object(key);
            }
            release_entry_object(layer.owned_filters);
            release_entry_object(layer.owned_mask);
            release_entry_object(layer.original_animation_keys);
            release_entry_object(layer.prior_blur_animation);
            release_entry_object(layer.filters);
            release_entry_object(layer.mask);
            release_entry_object(layer.layer);
        }
    }

    unsafe fn restore(self, window: id) -> ([f64; 4], f64, bool) {
        use cocoa::foundation::NSRect;
        let Self {
            alpha_views,
            layers,
            styles,
            ..
        } = self;
        let _: () = msg_send![class!(NSAnimationContext), beginGrouping];
        let context: id = msg_send![class!(NSAnimationContext), currentContext];
        let _: () = msg_send![context, setDuration: 0.0f64];
        let _: () = msg_send![context, setAllowsImplicitAnimation: false];
        let mut effects_restored = true;
        for snapshot in layers {
            let layer = snapshot.layer as id;
            let current_filters: id = msg_send![layer, filters];
            let current_mask: id = msg_send![layer, mask];
            let owns_filters = snapshot.owned_filters != 0
                && (current_filters == snapshot.owned_filters as id
                    || (current_filters != nil
                        && msg_send![current_filters, isEqual: snapshot.owned_filters as id]));
            let owns_mask = snapshot.owned_mask != 0 && current_mask == snapshot.owned_mask as id;
            let current_clip: bool = msg_send![layer, masksToBounds];
            let owns_clip = snapshot
                .owned_clip
                .is_some_and(|clip| clip == current_clip && clip != snapshot.masks_to_bounds);
            for key in snapshot.owned_animation_keys {
                let _: () = msg_send![layer, removeAnimationForKey: key as id];
                release_entry_object(key);
            }
            if owns_filters {
                let _: () = msg_send![layer, setFilters: snapshot.filters as id];
                if snapshot.prior_blur_animation != 0 {
                    let _: () = msg_send![layer,
                        addAnimation: snapshot.prior_blur_animation as id
                        forKey: tahoe_ns_string("entryBlurRamp")];
                }
            }
            if owns_mask {
                let _: () = msg_send![layer, setMask: snapshot.mask as id];
            }
            if (owns_filters || owns_mask || owns_clip)
                && (snapshot.owned_filters == 0 || owns_filters)
                && (snapshot.owned_mask == 0 || owns_mask)
            {
                let _: () = msg_send![layer, setMasksToBounds: snapshot.masks_to_bounds];
            }
            // A refresh that replaced an owned filter or mask wins. Only
            // compare restored entry-owned channels, not newer theme state.
            if owns_filters {
                let actual_filters: id = msg_send![layer, filters];
                effects_restored &= actual_filters == snapshot.filters as id
                    || (actual_filters != nil
                        && snapshot.filters != 0
                        && msg_send![actual_filters, isEqual: snapshot.filters as id]);
            }
            if owns_mask {
                let actual_mask: id = msg_send![layer, mask];
                effects_restored &= actual_mask == snapshot.mask as id;
            }
            if owns_clip {
                let actual_clip: bool = msg_send![layer, masksToBounds];
                effects_restored &= actual_clip == snapshot.masks_to_bounds;
            }
            release_entry_object(snapshot.owned_filters);
            release_entry_object(snapshot.owned_mask);
            release_entry_object(snapshot.original_animation_keys);
            release_entry_object(snapshot.prior_blur_animation);
            release_entry_object(snapshot.filters);
            release_entry_object(snapshot.mask);
            release_entry_object(snapshot.layer);
        }
        for (view, alpha) in alpha_views {
            let _: () = msg_send![view as id, setAlphaValue: alpha];
            let actual: f64 = msg_send![view as id, alphaValue];
            effects_restored &= (actual - alpha).abs() < 0.000_001;
            release_entry_object(view);
        }
        for snapshot in styles {
            // The T=0 animator already returned the model to its natural
            // style/tint. Replaying the old snapshot here would overwrite a
            // newer authoritative theme refresh during the entry.
            release_entry_object(snapshot.tint);
            release_entry_object(snapshot.view);
        }
        let _: () = msg_send![window, setAlphaValue: 1.0f64];
        let _: () = msg_send![class!(NSAnimationContext), endGrouping];
        let actual_frame: NSRect = msg_send![window, frame];
        let actual_alpha: f64 = msg_send![window, alphaValue];
        let actual = [
            actual_frame.origin.x,
            actual_frame.origin.y,
            actual_frame.size.width,
            actual_frame.size.height,
        ];
        effects_restored &= (actual_alpha - 1.0).abs() < 0.000_001;
        (actual, actual_alpha, effects_restored)
    }
}

#[cfg(target_os = "macos")]
unsafe fn finish_coherent_glass_entry(window: id, outcome: &'static str) {
    if window == nil {
        return;
    }
    let state = GLASS_COHERENT_ENTRY_STATES
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .remove(&(window as usize));
    let Some(state) = state else {
        return;
    };
    let (actual_frame, actual_alpha, effects_restored) = state.restore(window);
    let now = crate::platform::host_clock::host_time_ns();
    let mut receipt = MAIN_GLASS_ENTRY_RUNTIME_RECEIPT
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some(receipt) = receipt.as_mut() {
        if receipt.window_key == window as usize
            && matches!(receipt.outcome, "in_flight" | "pending_hidden_restore")
        {
            receipt.outcome = outcome;
            receipt.actual_frame = Some(actual_frame);
            receipt.actual_alpha = Some(actual_alpha);
            receipt.effects_restored = Some(effects_restored);
            receipt.frame_unchanged = Some(actual_frame == receipt.expected_frame);
            if outcome == "interrupted" && receipt.cancelled_at_host_time_ns.is_none() {
                receipt.cancelled_at_host_time_ns = Some(now);
            }
            receipt.actual_end_host_time_ns = Some(now);
            logging::log("PANEL", &format!(
                "event=main_glass_entry_end preset={} outcome={} actual_end_host_time_ns={} actual_frame={:.3},{:.3},{:.3},{:.3} actual_alpha={:.6} effects_restored={}",
                receipt.preset_id, outcome, now, actual_frame[0], actual_frame[1],
                actual_frame[2], actual_frame[3], actual_alpha, effects_restored,
            ));
        }
    }
}

#[cfg(target_os = "macos")]
unsafe fn discard_closed_coherent_glass_entry(backdrop: id) {
    let window_key = GLASS_COHERENT_ENTRY_STATES
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .iter()
        .find(|(_, state)| state.backdrop_key == backdrop as usize)
        .map(|(window_key, _)| *window_key);
    if let Some(window_key) = window_key {
        discard_coherent_glass_entry_for_window_key(window_key);
    }
}

/// A visible exit defers its coherent entry restore to hidden parking, which
/// only the main window has. Other windows are destroyed instead, so their
/// committed exit releases the deferred state here — never
/// leaking the retained views or leaving a pointer-reusable stale key.
#[cfg(target_os = "macos")]
pub(crate) fn release_glass_entry_for_removed_window(ticket: GlassExitTicket) {
    // SAFETY: main thread (GPUI foreground update); discard never messages
    // the window, only releases objects the entry itself retained.
    unsafe { discard_coherent_glass_entry_for_window_key(ticket.window_key) }
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn release_glass_entry_for_removed_window(_ticket: GlassExitTicket) {}

#[cfg(target_os = "macos")]
unsafe fn discard_coherent_glass_entry_for_window_key(window_key: usize) {
    let state = GLASS_COHERENT_ENTRY_STATES
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .remove(&window_key);
    let Some(state) = state else {
        return;
    };
    let window_number = state.window_number;
    state.discard();
    let now = crate::platform::host_clock::host_time_ns();
    let mut receipt = MAIN_GLASS_ENTRY_RUNTIME_RECEIPT
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some(receipt) = receipt.as_mut() {
        if receipt.window_key == window_key
            && receipt.window_number == window_number
            && matches!(receipt.outcome, "in_flight" | "pending_hidden_restore")
        {
            receipt.outcome = "closed";
            receipt.actual_end_host_time_ns = Some(now);
        }
    }
}

#[cfg(target_os = "macos")]
extern "C" fn tahoe_glass_backdrop_finish_coherent_entry(
    this: &objc::runtime::Object,
    _: objc::runtime::Sel,
) {
    unsafe {
        let view = this as *const objc::runtime::Object as id;
        let window: id = msg_send![view, window];
        if window == nil {
            discard_closed_coherent_glass_entry(view);
        } else {
            finish_coherent_glass_entry(window, "completed");
        }
    }
}

/// Native bounds are authoritative for the foreground clip. Convert the
/// already bounded backdrop into the root's own coordinates and mask the root
/// to that rounded shape; never mask the common contentView or the
/// gap-spanning footer host. The frame never moves during entry, so the mask
/// is static.
#[cfg(target_os = "macos")]
unsafe fn clip_entry_defocus_to_backdrop(root: id, backdrop: id) -> bool {
    use cocoa::foundation::{NSPoint, NSRect, NSSize};
    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        fn CGPathCreateWithRoundedRect(
            rect: NSRect,
            corner_width: f64,
            corner_height: f64,
            transform: *const std::ffi::c_void,
        ) -> *mut std::ffi::c_void;
        fn CGPathRelease(path: *mut std::ffi::c_void);
    }
    let layer: id = msg_send![root, layer];
    let glass_bounds: NSRect = msg_send![backdrop, bounds];
    let backdrop_clip: NSRect = msg_send![root, convertRect: glass_bounds fromView: backdrop];
    let root_bounds: NSRect = msg_send![root, bounds];
    let left = backdrop_clip.origin.x.max(root_bounds.origin.x);
    let bottom = backdrop_clip.origin.y.max(root_bounds.origin.y);
    let right = (backdrop_clip.origin.x + backdrop_clip.size.width)
        .min(root_bounds.origin.x + root_bounds.size.width);
    let top = (backdrop_clip.origin.y + backdrop_clip.size.height)
        .min(root_bounds.origin.y + root_bounds.size.height);
    if layer == nil || right <= left || top <= bottom {
        return false;
    }
    let clip = NSRect::new(
        NSPoint::new(left, bottom),
        NSSize::new(right - left, top - bottom),
    );
    let Some(mask_class) = objc::runtime::Class::get("CAShapeLayer") else {
        return false;
    };
    let mask: id = msg_send![mask_class, layer];
    if mask == nil {
        return false;
    }
    let radius_supported: bool = msg_send![backdrop, respondsToSelector: sel!(cornerRadius)];
    let corner_radius: f64 = if radius_supported {
        msg_send![backdrop, cornerRadius]
    } else {
        0.0
    };
    let radius = corner_radius
        .max(0.0)
        .min(clip.size.height * 0.5)
        .min(clip.size.width * 0.5);
    let path = CGPathCreateWithRoundedRect(clip, radius, radius, std::ptr::null());
    if path.is_null() {
        return false;
    }
    let _: () = msg_send![mask, setFrame: root_bounds];
    let _: () = msg_send![mask, setPath: path as id];
    let _: () = msg_send![layer, setMasksToBounds: true];
    let _: () = msg_send![layer, setMask: mask];
    CGPathRelease(path);
    true
}

/// Ramp an entry defocus on the window's content view layer: the window
/// resolves FROM `GLASS_ENTRY_BLUR_RADIUS` INTO sharp focus over the entry.
///
/// The exact inverse of the exit blur, with one deliberate difference: the
/// layer's model value is left at radius 0 and the animation is
/// `removedOnCompletion`, so a finished entry leaves NO residual filter to
/// clean up. (The exit ramp must persist at 8pt, so it keeps
/// `fillMode=forwards` + `removedOnCompletion=false` and relies on
/// `clear_exit_dematerialize_blur`.)
///
/// Must be installed AFTER `cancel_ns_window_exit_dematerialize_impl`, which
/// clears ALL layer filters and animations.
///
/// Returns the radius actually installed (0.0 when the runtime lacks the
/// private `CAFilter` class, so the receipt can distinguish "no blur ran"
/// from "blur ran at 0").
#[cfg(target_os = "macos")]
unsafe fn ramp_entry_defocus(target_view: id, duration: f64, radius: f64, log_target: &str) -> f64 {
    // Every bail-out names itself: a silent 0.0 was indistinguishable from
    // "blur ran and did nothing".
    let bail = |reason: &str| {
        logging::log(
            log_target,
            &format!("event=glass_entry_defocus_skip reason={}", reason),
        );
        0.0
    };
    if radius <= 0.0 || duration <= 0.0 {
        return bail("zero_radius_or_duration");
    }
    // CHOOSE THE TARGET CAREFULLY. Runtime-proven 2026-07-27: on the MAIN
    // window, filtering the contentView layer blurs across the 8pt transparent
    // footer gutter and fills it, which the exit metrics catch as "transparent
    // footer gutter was not preserved". Main therefore passes its glass
    // backdrop, which is already laid out to EXCLUDE that gutter. Main-window
    // footer onset never passes its gap-spanning container here: footer_popup
    // returns only rounded NSGlassEffectView capsule layers. Detached reusable
    // windows may still pass their own bounded content view.
    if target_view == nil {
        return bail("no_target_view");
    }
    // `filters` is a CALayer property and the target is not guaranteed
    // layer-backed at entry time; without this the ramp silently no-ops.
    let wants_layer: bool = msg_send![target_view, wantsLayer];
    if !wants_layer {
        let _: () = msg_send![target_view, setWantsLayer: true];
    }
    let layer: id = msg_send![target_view, layer];
    if layer == nil {
        return bail("no_layer");
    }
    // A layer filter can sample beyond its bounds. Clip the defocus to the
    // owning surface so the main backdrop and detached footer cannot blur
    // across the calibrated transparent gutter between them.
    let _: () = msg_send![layer, setMasksToBounds: cocoa::base::YES];
    let (Some(filter_class), Some(anim_class)) = (
        objc::runtime::Class::get("CAFilter"),
        objc::runtime::Class::get("CABasicAnimation"),
    ) else {
        return bail("no_cafilter_class");
    };
    let blur_type = tahoe_ns_string("gaussianBlur");
    let filter: id = msg_send![filter_class, filterWithType: blur_type];
    if filter == nil {
        return bail("filter_alloc_failed");
    }
    let entry_name = tahoe_ns_string("entryBlur");
    let _: () = msg_send![filter, setName: entry_name];
    let radius_key = tahoe_ns_string("inputRadius");
    // Model value 0: once the animation is removed the window is sharp.
    let zero: id = msg_send![class!(NSNumber), numberWithDouble: 0.0f64];
    let _: () = msg_send![filter, setValue: zero forKey: radius_key];
    let filters: id = msg_send![class!(NSArray), arrayWithObject: filter];
    let _: () = msg_send![layer, setFilters: filters];

    let key_path = tahoe_ns_string("filters.entryBlur.inputRadius");
    let anim: id = msg_send![anim_class, animationWithKeyPath: key_path];
    if anim == nil {
        return bail("animation_alloc_failed");
    }
    let from: id = msg_send![class!(NSNumber), numberWithDouble: radius];
    let _: () = msg_send![anim, setFromValue: from];
    let _: () = msg_send![anim, setToValue: zero];
    let _: () = msg_send![anim, setDuration: duration];
    // Ease-out: most of the defocus resolves early, matching the material
    // onset curve rather than crawling to sharp at the very end.
    if let Some(timing_class) = objc::runtime::Class::get("CAMediaTimingFunction") {
        let name = tahoe_ns_string("easeOut");
        if name != nil {
            let timing: id = msg_send![timing_class, functionWithName: name];
            if timing != nil {
                let _: () = msg_send![anim, setTimingFunction: timing];
            }
        }
    }
    let _: () = msg_send![anim, setRemovedOnCompletion: true];
    let anim_key = tahoe_ns_string("entryBlurRamp");
    let _: () = msg_send![layer, addAnimation: anim forKey: anim_key];
    radius
}

#[cfg(target_os = "macos")]
static MAIN_COHERENT_ENTRY_PREPARING: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

#[cfg(target_os = "macos")]
unsafe fn main_has_coherent_entry_receipt(window: id) -> bool {
    if window == nil || crate::window_manager::get_main_window() != Some(window) {
        return false;
    }
    let window_number: i64 = msg_send![window, windowNumber];
    let receipt = MAIN_GLASS_ENTRY_RUNTIME_RECEIPT
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let matching = receipt.as_ref().filter(|receipt| {
        receipt.window_key == window as usize && receipt.window_number == window_number
    });
    if matching.is_some() {
        return true;
    }
    // Before the first coherent receipt exists, preserve unrelated native
    // filters already installed on rounded capsules. The arm site sets this
    // flag using its captured spec, not a new live preference read.
    MAIN_COHERENT_ENTRY_PREPARING.load(std::sync::atomic::Ordering::Relaxed)
}

#[cfg(target_os = "macos")]
unsafe fn record_main_coherent_hidden_alpha(window: id) {
    if !main_has_coherent_entry_receipt(window) {
        return;
    }
    let alpha: f64 = msg_send![window, alphaValue];
    let mut receipt = MAIN_GLASS_ENTRY_RUNTIME_RECEIPT
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some(receipt) = receipt.as_mut() {
        if receipt.window_key == window as usize {
            receipt.post_hide_alpha = Some(alpha);
        }
    }
}

/// Remove any exit-dematerialize blur left on the window's content view
/// layer (a superseded exit, or post-hide cleanup before the next show).
#[cfg(target_os = "macos")]
unsafe fn clear_exit_dematerialize_blur(window: id) {
    let content_view: id = msg_send![window, contentView];
    if content_view != nil {
        let layer: id = msg_send![content_view, layer];
        if layer != nil {
            let nil_id: id = nil;
            let _: () = msg_send![layer, setFilters: nil_id];
            let _: () = msg_send![layer, removeAllAnimations];
        }
    }
    // The coherent finish restored each capsule's original filters/mask.
    // A blanket clear would erase unrelated pre-entry CA filters on a
    // hide/reopen, so it only runs before the first coherent entry.
    if !main_has_coherent_entry_receipt(window) {
        crate::footer_popup::clear_main_window_footer_entry_capsule_effects(window);
    }
}

/// Exit for the main window: a fixed-frame ease-in-out alpha fade over
/// `GLASS_EXIT_DURATION`. Returns false when native glass is unavailable so
/// the caller hides immediately.
///
/// This runs ABOVE the synchronous hide layer: the caller plays this,
/// waits ~135ms, then runs the NORMAL hide flow. Never defer orderOut:
/// itself — that livelocked the hotkey gesture listener.
#[cfg(target_os = "macos")]
pub fn begin_main_window_exit_dematerialize() -> bool {
    if require_main_thread("begin_main_window_exit_dematerialize") {
        return false;
    }
    let Some(window) = window_manager::get_main_window() else {
        return false;
    };
    // SAFETY: main thread verified; window valid from the manager.
    unsafe { begin_ns_window_exit_dematerialize(window, "PANEL", "Main window") }
}

/// The same fixed-frame exit fade for any live GPUI window. Returns false
/// when native glass is unavailable so callers close instantly instead.
#[cfg(target_os = "macos")]
pub fn begin_gpui_window_exit_dematerialize(
    window: &gpui::Window,
    log_target: &str,
    window_name: &str,
) -> bool {
    begin_gpui_window_exit_with_ticket(window, log_target, window_name).is_some()
}

#[cfg(target_os = "macos")]
pub(crate) fn begin_gpui_window_exit_with_ticket(
    window: &gpui::Window,
    log_target: &str,
    window_name: &str,
) -> Option<GlassExitTicket> {
    if require_main_thread("begin_gpui_window_exit_dematerialize") {
        return None;
    }
    // SAFETY: ns_view belongs to a live GPUI window on the main thread.
    unsafe {
        let ns_window = ns_window_from_gpui_window(window)?;
        if !begin_ns_window_exit_dematerialize(ns_window, log_target, window_name) {
            return None;
        }
        let ticket = advance_glass_exit_generation(ns_window as usize);
        record_glass_exit_begin(ns_window, ticket, window_name);
        Some(ticket)
    }
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn begin_gpui_window_exit_with_ticket(
    _window: &gpui::Window,
    _log_target: &str,
    _window_name: &str,
) -> Option<GlassExitTicket> {
    None
}

/// Start the shared exit and remove the GPUI window after its short visual
/// tail. Registry cleanup and parent-focus handoff should happen before this
/// call; only destruction is delayed, so dismissal remains input-instant.
pub fn dematerialize_then_remove_gpui_window<V: 'static>(
    window: &mut gpui::Window,
    cx: &mut gpui::Context<V>,
    log_target: &'static str,
    window_name: &'static str,
) {
    if let Some(ticket) = begin_gpui_window_exit_with_ticket(window, log_target, window_name) {
        let any_handle = window.window_handle();
        cx.spawn(async move |_this, cx: &mut gpui::AsyncApp| {
            cx.background_executor()
                .timer(glass_exit_remove_delay())
                .await;
            cx.update(|cx| {
                if !glass_exit_ticket_is_current(ticket) {
                    return;
                }
                record_glass_exit_commit(ticket);
                release_glass_entry_for_removed_window(ticket);
                let _ = any_handle.update(cx, |_view, window, _cx| {
                    window.remove_window();
                });
            });
        })
        .detach();
    } else {
        window.remove_window();
    }
}

/// App-context counterpart used by global popup/HUD registries.
pub fn dematerialize_then_remove_gpui_window_from_app(
    window: &mut gpui::Window,
    cx: &mut gpui::App,
    log_target: &'static str,
    window_name: &'static str,
) {
    if let Some(ticket) = begin_gpui_window_exit_with_ticket(window, log_target, window_name) {
        remove_gpui_window_after_glass_exit_from_app_with_ticket(window, cx, ticket);
    } else {
        window.remove_window();
    }
}

/// Schedule only the destruction tail after a caller has already started the
/// exit. This is used by `on_window_should_close` handlers that must return
/// `false` while the visual tail completes.
pub fn remove_gpui_window_after_glass_exit_from_app(window: &mut gpui::Window, cx: &mut gpui::App) {
    let ticket = unsafe {
        ns_window_from_gpui_window(window)
            .map(|window| advance_glass_exit_generation(window as usize))
    };
    if let Some(ticket) = ticket {
        remove_gpui_window_after_glass_exit_from_app_with_ticket(window, cx, ticket);
    } else {
        window.remove_window();
    }
}

#[cfg(target_os = "macos")]
fn remove_gpui_window_after_glass_exit_from_app_with_ticket(
    window: &mut gpui::Window,
    cx: &mut gpui::App,
    ticket: GlassExitTicket,
) {
    let any_handle = window.window_handle();
    cx.spawn(async move |cx: &mut gpui::AsyncApp| {
        cx.background_executor()
            .timer(glass_exit_remove_delay())
            .await;
        cx.update(|cx| {
            if !glass_exit_ticket_is_current(ticket) {
                return;
            }
            record_glass_exit_commit(ticket);
            release_glass_entry_for_removed_window(ticket);
            let _ = any_handle.update(cx, |_view, window, _cx| {
                window.remove_window();
            });
        });
    })
    .detach();
}

#[cfg(not(target_os = "macos"))]
pub fn begin_gpui_window_exit_dematerialize(
    _window: &gpui::Window,
    _log_target: &str,
    _window_name: &str,
) -> bool {
    false
}

/// Core of the exit dematerialize, shared by every window kind.
///
/// # Safety
/// `window` must be a valid NSWindow on the main thread.
#[cfg(target_os = "macos")]
unsafe fn begin_ns_window_exit_dematerialize(
    window: id,
    log_target: &str,
    window_name: &str,
) -> bool {
    use cocoa::foundation::NSRect;

    if !(tahoe_native_glass_composition_available()
        && crate::theme::get_cached_theme().is_vibrancy_enabled())
    {
        return false;
    }
    let visible: bool = msg_send![window, isVisible];
    if !visible {
        return false;
    }
    let frame: NSRect = msg_send![window, frame];
    if frame.size.width < 40.0 || frame.size.height < 40.0 {
        return false;
    }
    // Freeze any in-flight entry so the fade starts from what is on screen.
    cancel_pending_glass_window_selectors(window);
    clear_exit_dematerialize_blur(window);
    let _: () = msg_send![class!(NSAnimationContext), beginGrouping];
    let ctx: id = msg_send![class!(NSAnimationContext), currentContext];
    let _: () = msg_send![ctx, setDuration: GLASS_EXIT_DURATION];
    let _: () = msg_send![ctx, setAllowsImplicitAnimation: true];
    if let Some(timing_class) = objc::runtime::Class::get("CAMediaTimingFunction") {
        let name = tahoe_ns_string("easeInEaseOut");
        if name != nil {
            let timing: id = msg_send![timing_class, functionWithName: name];
            if timing != nil {
                let _: () = msg_send![ctx, setTimingFunction: timing];
            }
        }
    }
    let animator: id = msg_send![window, animator];
    let _: () = msg_send![animator, setAlphaValue: 0.0f64];
    let _: () = msg_send![class!(NSAnimationContext), endGrouping];
    logging::log(
        log_target,
        &format!(
            "event=glass_morph window={} variant=fade_only phase=exit duration={:.2}s direction=fixed_frame",
            window_name, GLASS_EXIT_DURATION
        ),
    );
    true
}

#[cfg(not(target_os = "macos"))]
pub fn begin_main_window_exit_dematerialize() -> bool {
    false
}

/// Park a just-hidden window at alpha 0 so that whichever code path orders
/// it front next cannot flash a full-alpha frame before the entry fade
/// starts (multiple show paths exist: the platform show helpers and GPUI's
/// `activate_window`). The entry restores alpha — including on every early
/// exit — so the window can never stay invisible.
#[cfg(target_os = "macos")]
unsafe fn park_hidden_window_for_glass_morph(window: id) {
    if window.is_null() {
        return;
    }
    // Instant/fallback hides may not have begun a native exit. Cancel the
    // pending entry finish and restore its state while hidden.
    let hidden: bool = !msg_send![window, isVisible];
    if hidden
        && GLASS_COHERENT_ENTRY_STATES
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .contains_key(&(window as usize))
    {
        cancel_pending_glass_window_selectors(window);
    }
    // Post-hide cleanup: drop any exit blur so the next show starts crisp.
    clear_exit_dematerialize_blur(window);
    if hidden {
        record_main_coherent_hidden_alpha(window);
    }
    if !(tahoe_native_glass_composition_available()
        && crate::theme::get_cached_theme().is_vibrancy_enabled())
    {
        return;
    }
    // Zero-alpha parking is legal ONLY for a window that is not presented.
    // A visible window at alpha zero is wallpaper where UI should be — the
    // exact defect the visible-entry alpha policy forbids.
    let is_visible: bool = msg_send![window, isVisible];
    if is_visible {
        tracing::error!(
            target: "script_kit::native_glass",
            event = "glass_hidden_park_on_visible_window",
            "glass_hidden_park_on_visible_window"
        );
        return;
    }
    let _: () = msg_send![window, setAlphaValue: GLASS_HIDDEN_PARK_ALPHA];
    record_main_coherent_hidden_alpha(window);
}

/// True when a main entry started within the last 700ms. A rapid re-show
/// presents at full alpha immediately instead of restarting the fade.
#[cfg(target_os = "macos")]
fn glass_morph_recently_started() -> bool {
    use std::sync::atomic::Ordering;
    let now_ms = glass_morph_now_ms();
    let last = GLASS_MORPH_LAST_START_MS.load(Ordering::Relaxed);
    if last != u64::MAX && now_ms.saturating_sub(last) < 700 {
        return true;
    }
    GLASS_MORPH_LAST_START_MS.store(now_ms, Ordering::Relaxed);
    false
}

#[cfg(target_os = "macos")]
static GLASS_MORPH_EPOCH: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
#[cfg(target_os = "macos")]
static GLASS_MORPH_LAST_START_MS: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(u64::MAX);
#[cfg(target_os = "macos")]
static GLASS_MORPH_LAST_DURATION_MS: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(0);

/// Called after a menu selection has been persisted and before its guarded
/// hide/show replay. It bypasses only the main entry's recent-start gate,
/// and invalidates old callbacks. An in-flight visible exit retains its
/// model until orderOut; hidden parking restores the exact settled state.
#[cfg(target_os = "macos")]
pub fn reset_main_glass_entry_replay() {
    if require_main_thread("reset_main_glass_entry_replay") {
        return;
    }
    let Some(window) = crate::window_manager::get_main_window() else {
        return;
    };
    unsafe {
        cancel_ns_window_exit_dematerialize_impl(window, false);
    }
    GLASS_MORPH_LAST_START_MS.store(u64::MAX, std::sync::atomic::Ordering::Relaxed);
    GLASS_MORPH_LAST_DURATION_MS.store(0, std::sync::atomic::Ordering::Relaxed);
    logging::log("PANEL", "event=glass_morph_replay_reset window=Main window");
}

#[cfg(not(target_os = "macos"))]
pub fn reset_main_glass_entry_replay() {}

#[cfg(target_os = "macos")]
fn glass_morph_now_ms() -> u64 {
    let epoch = *GLASS_MORPH_EPOCH.get_or_init(std::time::Instant::now);
    epoch.elapsed().as_millis() as u64
}

/// Time left (plus a small settle tail) in the morph that most recently
/// started, if it is still in flight. Lets sibling windows (the footer
/// overlay) hide while the main window animates and fade in afterwards.
#[cfg(target_os = "macos")]
pub fn glass_morph_remaining() -> Option<std::time::Duration> {
    use std::sync::atomic::Ordering;
    let start = GLASS_MORPH_LAST_START_MS.load(Ordering::Relaxed);
    if start == u64::MAX {
        return None;
    }
    let duration_ms = GLASS_MORPH_LAST_DURATION_MS.load(Ordering::Relaxed) + 60;
    let end = start.saturating_add(duration_ms);
    let now = glass_morph_now_ms();
    if now >= end {
        None
    } else {
        Some(std::time::Duration::from_millis(end - now))
    }
}

/// How long the floating footer stays ordered out before it fades back in.
///
/// The footer is a SEPARATE NSWindow tracking the main window's bounds, so it
/// does not inherit the main entry's fade, blur, or geometry. It is parked
/// (ordered fully out) and re-presented on its own schedule.
///
/// LEGACY (`SCRIPT_KIT_GLASS_FOOTER_SYNC=0`): park for the whole remaining
/// morph plus a 60ms tail, then fade over 120ms — the footer is absent for the
/// entire 149ms entry and finishes arriving ~180ms AFTER the main window has
/// settled. That reads as a second, unrelated arrival (user report
/// 2026-07-27: "why aren't the floating buttons fading/blurring too?").
///
/// The park exists because the footer would otherwise chase the animating
/// frame — which mattered at the old `0.03` inset (106% -> 98%, tens of points
/// of travel). The current calibration moves 4.5pt per side, so the chase it
/// prevents is now far less visible than the late arrival it causes.
///
/// SYNCED (default): hold only to the main window's CONTENT-REVEAL anchor, so
/// the footer fades up on the same clock as the main content and the whole
/// surface materializes as one object.
#[cfg(target_os = "macos")]
fn glass_sibling_reveal_delay() -> Option<std::time::Duration> {
    use std::sync::atomic::Ordering;
    let synced = std::env::var("SCRIPT_KIT_GLASS_FOOTER_SYNC")
        .ok()
        .map(|raw| raw.trim() != "0")
        .unwrap_or(true);
    if !synced {
        return glass_morph_remaining();
    }
    // Still in flight?
    glass_morph_remaining()?;
    let start = GLASS_MORPH_LAST_START_MS.load(Ordering::Relaxed);
    if start == u64::MAX {
        return None;
    }
    let anchor_ms = (GLASS_ENTRY_CONTENT_HOLD_DURATION * 1000.0).round() as u64;
    let reveal_at = start.saturating_add(anchor_ms);
    let now = glass_morph_now_ms();
    Some(std::time::Duration::from_millis(
        reveal_at.saturating_sub(now),
    ))
}

#[cfg(not(target_os = "macos"))]
pub fn glass_morph_remaining() -> Option<std::time::Duration> {
    None
}

/// Park a sibling GPUI window (footer overlay) while a glass morph is in
/// flight, returning how long until it should fade back in. The overlay is a
/// separate NSWindow that tracks the main window's frame; without this it
/// appears instantly at full alpha and visibly chases the animating frame.
///
/// Parking means NOT PRESENTED: the window is ordered out for the interval,
/// and only then parked at the hidden alpha. Leaving a visible window on
/// screen at alpha zero is prohibited (wallpaper where UI should be), so the
/// old visible zero-alpha park was replaced by this orderOut-based park.
/// [`restore_gpui_window_alpha_animated`] re-presents it and keeps the
/// locked 0.12s fade.
#[cfg(target_os = "macos")]
pub fn park_gpui_window_alpha_if_morphing(window: &gpui::Window) -> Option<std::time::Duration> {
    let remaining = glass_sibling_reveal_delay()?;
    let handle = raw_window_handle::HasWindowHandle::window_handle(window).ok()?;
    let raw_window_handle::RawWindowHandle::AppKit(appkit) = handle.as_raw() else {
        return None;
    };
    let ns_view = appkit.ns_view.as_ptr() as id;
    // SAFETY: main thread; standard NSWindow accessors, nil-checked.
    unsafe {
        let ns_window: id = msg_send![ns_view, window];
        if ns_window.is_null() {
            return None;
        }
        let _: () = msg_send![ns_window, orderOut: nil];
        let is_visible: bool = msg_send![ns_window, isVisible];
        if is_visible {
            // Could not take the window off screen (e.g. still attached as a
            // child). Zero-alpha visible parking is prohibited — leave alpha
            // alone and let the entry fade own it.
            tracing::error!(
                target: "script_kit::native_glass",
                event = "glass_hidden_park_on_visible_window",
                "glass_hidden_park_on_visible_window"
            );
            return Some(remaining);
        }
        let _: () = msg_send![ns_window, setAlphaValue: GLASS_HIDDEN_PARK_ALPHA];
    }
    Some(remaining)
}

#[cfg(not(target_os = "macos"))]
pub fn park_gpui_window_alpha_if_morphing(_window: &gpui::Window) -> Option<std::time::Duration> {
    None
}

/// Re-present a previously parked sibling window and fade it back in.
///
/// The fade duration MATCHES the main window's content fade, so the floating
/// footer materializes on the same clock as the content it belongs to. It was
/// previously a hardcoded `0.12s` chosen independently of the entry, which —
/// combined with the full-morph park delay — made the footer a visibly
/// separate second arrival.
///
/// The park orders the window out, so restore orders it back in (without
/// activating) before the fade.
#[cfg(target_os = "macos")]
pub fn restore_gpui_window_alpha_animated(window: &gpui::Window) {
    let Ok(handle) = raw_window_handle::HasWindowHandle::window_handle(window) else {
        return;
    };
    let raw_window_handle::RawWindowHandle::AppKit(appkit) = handle.as_raw() else {
        return;
    };
    let ns_view = appkit.ns_view.as_ptr() as id;
    // SAFETY: main thread; standard NSWindow/NSAnimationContext usage.
    unsafe {
        let ns_window: id = msg_send![ns_view, window];
        if ns_window.is_null() {
            return;
        }
        let _: () = msg_send![ns_window, orderFrontRegardless];
        // Same defocus the main window resolves out of, so the buttons
        // materialize rather than simply appearing.
        // The footer window IS the buttons, so its own content view is the
        // right target — the main window's gutter is a different window and is
        // unaffected.
        let footer_content: id = msg_send![ns_window, contentView];
        let footer_blur = ramp_entry_defocus(
            footer_content,
            glass_entry_content_fade_duration(),
            glass_entry_blur_radius(),
            "FOOTER",
        );
        let _: () = msg_send![class!(NSAnimationContext), beginGrouping];
        let ctx: id = msg_send![class!(NSAnimationContext), currentContext];
        let _: () = msg_send![ctx, setDuration: glass_entry_content_fade_duration()];
        if let Some(timing_class) = objc::runtime::Class::get("CAMediaTimingFunction") {
            let name = tahoe_ns_string("easeOut");
            if name != nil {
                let timing: id = msg_send![timing_class, functionWithName: name];
                if timing != nil {
                    let _: () = msg_send![ctx, setTimingFunction: timing];
                }
            }
        }
        let animator: id = msg_send![ns_window, animator];
        let _: () = msg_send![animator, setAlphaValue: 1.0f64];
        let _: () = msg_send![class!(NSAnimationContext), endGrouping];
        logging::log(
            "FOOTER",
            &format!(
                "event=glass_sibling_reveal fade_ns={} blur_radius={:.2}",
                (glass_entry_content_fade_duration() * 1_000_000_000.0).round() as u64,
                footer_blur
            ),
        );
    }
}

#[cfg(not(target_os = "macos"))]
pub fn restore_gpui_window_alpha_animated(_window: &gpui::Window) {}
