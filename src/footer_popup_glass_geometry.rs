/// Canonical partition of the expanded main-window host used by the detached
/// footer composition. The same physical regions are expressed in either
/// GPUI's top-left coordinate space or AppKit's bottom-left coordinate space.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct MainWindowDetachedFooterRegions {
    pub host: crate::protocol::LayoutBounds,
    pub main_content: crate::protocol::LayoutBounds,
    pub transparent_gap: crate::protocol::LayoutBounds,
    pub footer: crate::protocol::LayoutBounds,
}

fn round_footer_region_value(value: f32, backing_scale: f32) -> f32 {
    let scale = if backing_scale.is_finite() && backing_scale > 0.0 {
        backing_scale
    } else {
        1.0
    };
    (value * scale).round() / scale
}

fn main_window_detached_footer_region_dimensions(
    width: f32,
    host_height: f32,
    footer_height: f32,
    gap_height: f32,
    backing_scale: f32,
) -> (f32, f32, f32, f32) {
    let width = round_footer_region_value(width.max(0.0), backing_scale);
    let host_height = round_footer_region_value(host_height.max(0.0), backing_scale);
    let footer_height =
        round_footer_region_value(footer_height.max(0.0).min(host_height), backing_scale)
            .min(host_height);
    let gap_height = round_footer_region_value(
        gap_height.max(0.0).min(host_height - footer_height),
        backing_scale,
    )
    .min(host_height - footer_height);
    let main_height = host_height - footer_height - gap_height;
    (width, host_height, main_height, gap_height)
}

/// Partition an expanded host in GPUI's top-left, y-down coordinate space.
pub(crate) fn main_window_detached_footer_regions_gpui(
    width: f32,
    host_height: f32,
    footer_height: f32,
    gap_height: f32,
    backing_scale: f32,
) -> MainWindowDetachedFooterRegions {
    let (width, host_height, main_height, gap_height) =
        main_window_detached_footer_region_dimensions(
            width,
            host_height,
            footer_height,
            gap_height,
            backing_scale,
        );
    let footer_height = host_height - main_height - gap_height;
    MainWindowDetachedFooterRegions {
        host: crate::protocol::LayoutBounds {
            x: 0.0,
            y: 0.0,
            width,
            height: host_height,
        },
        main_content: crate::protocol::LayoutBounds {
            x: 0.0,
            y: 0.0,
            width,
            height: main_height,
        },
        transparent_gap: crate::protocol::LayoutBounds {
            x: 0.0,
            y: main_height,
            width,
            height: gap_height,
        },
        footer: crate::protocol::LayoutBounds {
            x: 0.0,
            y: main_height + gap_height,
            width,
            height: footer_height,
        },
    }
}

/// Partition an expanded host in AppKit's bottom-left, y-up coordinate space.
pub(crate) fn main_window_detached_footer_regions_appkit(
    width: f32,
    host_height: f32,
    footer_height: f32,
    gap_height: f32,
    backing_scale: f32,
) -> MainWindowDetachedFooterRegions {
    let (width, host_height, main_height, gap_height) =
        main_window_detached_footer_region_dimensions(
            width,
            host_height,
            footer_height,
            gap_height,
            backing_scale,
        );
    let footer_height = host_height - main_height - gap_height;
    MainWindowDetachedFooterRegions {
        host: crate::protocol::LayoutBounds {
            x: 0.0,
            y: 0.0,
            width,
            height: host_height,
        },
        main_content: crate::protocol::LayoutBounds {
            x: 0.0,
            y: footer_height + gap_height,
            width,
            height: main_height,
        },
        transparent_gap: crate::protocol::LayoutBounds {
            x: 0.0,
            y: footer_height,
            width,
            height: gap_height,
        },
        footer: crate::protocol::LayoutBounds {
            x: 0.0,
            y: 0.0,
            width,
            height: footer_height,
        },
    }
}

/// Tahoe floating chrome for the main window: the glass stage fills the whole
/// window and the footer capsules float INSIDE it, in one fixed zone at the
/// bottom (one gap above, below, and at each side), on every main-window
/// surface (2026-09-23 owner request: the capsules sat below the glass on
/// non-menu views). Views keep their content above the zone; only the main
/// menu list scrolls under it (faded and non-interactive).
pub(crate) fn main_window_footer_in_stage() -> bool {
    glass_scroll_bands_active()
}

/// Height of the in-stage footer zone: capsule height plus one gap above and
/// one below. 0 when the footer is not in the stage (glass off).
pub(crate) fn main_window_footer_zone_height() -> f32 {
    if main_window_footer_in_stage() {
        crate::components::footer_chrome::current_main_menu_footer_height()
            + FLOAT_FOOTER_CONTAINER_GAP_PX * 2.0
    } else {
        0.0
    }
}

/// Bottom space every main-window view keeps free of content: the in-stage
/// footer zone, or the in-flow GPUI footer when glass is off.
pub(crate) fn main_window_footer_reservation_height() -> f32 {
    if main_window_footer_in_stage() {
        main_window_footer_zone_height()
    } else {
        crate::components::footer_chrome::current_main_menu_footer_height()
    }
}

const MAIN_FOOTER_SURFACE_NONE: u8 = 0;
const MAIN_FOOTER_SURFACE_OTHER: u8 = 1;
const MAIN_FOOTER_SURFACE_MAIN_MENU: u8 = 2;

/// Which native footer surface the main window synced this frame: none, the
/// main menu (`script_list`, whose list scrolls under the zone), or another
/// view (which ends its content above the zone). Set by the footer sync
/// before the root lays out, so both read the same frame's answer.
static MAIN_FOOTER_SURFACE: std::sync::atomic::AtomicU8 =
    std::sync::atomic::AtomicU8::new(MAIN_FOOTER_SURFACE_NONE);

pub(crate) fn set_main_footer_surface(surface: Option<&str>) {
    let state = match surface {
        None => MAIN_FOOTER_SURFACE_NONE,
        Some("script_list") => MAIN_FOOTER_SURFACE_MAIN_MENU,
        Some(_) => MAIN_FOOTER_SURFACE_OTHER,
    };
    MAIN_FOOTER_SURFACE.store(state, std::sync::atomic::Ordering::Relaxed);
}

fn main_footer_surface_state() -> u8 {
    MAIN_FOOTER_SURFACE.load(std::sync::atomic::Ordering::Relaxed)
}

/// The main-menu list extends under the in-stage footer zone right now
/// (fade, input blocker, and reveal clearance apply). Every other surface
/// ends its content above the zone.
pub(crate) fn main_list_scrolls_under_footer() -> bool {
    main_window_footer_in_stage() && main_footer_surface_state() == MAIN_FOOTER_SURFACE_MAIN_MENU
}

/// Bottom space the root stage keeps free for the current view: the whole
/// in-stage footer zone while a native footer is present on a non-menu view,
/// otherwise nothing. The root is the single owner of this reservation.
pub(crate) fn main_window_view_footer_zone_reserve() -> f32 {
    if main_window_footer_in_stage() && main_footer_surface_state() == MAIN_FOOTER_SURFACE_OTHER {
        main_window_footer_zone_height()
    } else {
        0.0
    }
}

/// Footer space a view reserves inside its own layout (native-footer
/// spacers, focused-text budgets). Zero in glass mode, where the root owns
/// the in-stage zone; reserving here too would stack the two.
pub(crate) fn view_owned_native_footer_height() -> f32 {
    if main_window_footer_in_stage() {
        0.0
    } else {
        crate::components::footer_chrome::current_main_menu_footer_height()
    }
}

/// In-stage footer capsule container (GPUI top-left space): one `gap` inset
/// from the window's sides and bottom, identical on every surface.
pub(crate) fn main_window_footer_bounds_gpui(
    width: f32,
    host_height: f32,
    footer_height: f32,
    gap: f32,
    backing_scale: f32,
) -> crate::protocol::LayoutBounds {
    let width = round_footer_region_value(width.max(0.0), backing_scale);
    let host_height = round_footer_region_value(host_height.max(0.0), backing_scale);
    let footer_height =
        round_footer_region_value(footer_height.max(0.0), backing_scale).min(host_height);
    let inset = round_footer_region_value(gap.max(0.0), backing_scale).min(width / 2.0);
    crate::protocol::LayoutBounds {
        x: inset,
        y: (host_height - inset - footer_height).max(0.0),
        width: (width - inset * 2.0).max(0.0),
        height: footer_height,
    }
}

/// Height of the fully transparent strip the main window reserves below its
/// glass container so the footer capsules float over the bare desktop.
/// Both the GPUI root (bottom padding) and the native NSGlassEffectView
/// backdrop (bottom frame inset) subtract this same value; 0 when float
/// chrome is off.
pub(crate) fn main_window_float_footer_strip_height() -> f32 {
    if glass_scroll_bands_active() {
        crate::components::footer_chrome::current_main_menu_footer_height()
            + FLOAT_FOOTER_CONTAINER_GAP_PX
    } else {
        0.0
    }
}

/// Reconcile the platform-managed footer band and header edge strip with the
/// main window's current glass mode and frame. Called beside Tahoe backdrop
/// recreation and again when the footer host is created/refreshed.
#[cfg(target_os = "macos")]
pub(crate) unsafe fn sync_main_window_glass_scroll_bands(ns_window: id) {
    use cocoa::appkit::NSViewWidthSizable;
    use cocoa::base::YES;
    use cocoa::foundation::NSRect;
    use objc::{msg_send, sel, sel_impl};

    if crate::platform::require_main_thread("sync_main_window_glass_scroll_bands") {
        return;
    }
    let content_view: id = msg_send![ns_window, contentView];
    if content_view == nil {
        return;
    }

    let active = glass_scroll_bands_active();
    let search_root = main_window_footer_search_root(ns_window);
    let mut footer_view = find_subview_by_identifier(search_root, FOOTER_EFFECT_ID);
    if footer_view != nil {
        // Mode changed (same-window Tahoe container <-> blur-era in-window
        // VEV): recreate the host in the correct native parent.
        let float_ok = float_footer_host_view_class()
            .map(|cls| {
                let is_float: cocoa::base::BOOL = msg_send![footer_view, isKindOfClass: cls];
                is_float == YES
            })
            .unwrap_or(false);
        if float_ok != active {
            let _: () = msg_send![footer_view, removeFromSuperview];
            if !active {
                remove_main_window_footer_glass_container(ns_window);
            }
            clear_main_window_footer_refresh_signature();
            footer_view = nil;
        }
    }

    let content_bounds: NSRect = msg_send![content_view, bounds];
    if footer_view != nil {
        let footer_frame = cocoa::foundation::NSRect::new(
            cocoa::foundation::NSPoint::new(0.0, 0.0),
            cocoa::foundation::NSSize::new(content_bounds.size.width, footer_height()),
        );
        let _: () = msg_send![footer_view, setFrame: footer_frame];
    }
    if !active && main_window_footer_glass_root(ns_window) != nil {
        remove_main_window_footer_glass_container(ns_window);
    }
    log_strip_views_debug(ns_window);
    let _ = NSViewWidthSizable;
}

/// Identifier for the content view of the floating-footer child window.
#[cfg(target_os = "macos")]
const FLOAT_FOOTER_LAYER_ID: &str = "script-kit-float-footer-layer";

/// Shared styling for every floating footer capsule. The window backdrop and
/// every capsule resolve through the same appearance/RGB/effective-tint
/// policy; only the capsule role may add the shared adaptive separation rim.
#[cfg(target_os = "macos")]
unsafe fn style_float_footer_capsule(capsule: id, theme: &crate::theme::Theme) {
    let style = crate::platform::resolve_native_glass_style(
        theme,
        crate::platform::NativeGlassSurfaceRole::FloatingCapsule,
    );
    let _ = crate::platform::apply_native_glass_style(capsule, style);
}

#[cfg(target_os = "macos")]
thread_local! {
    /// Tahoe main-window footer containers. These are native siblings of the
    /// GPUI Metal view inside the same NSWindow, so WindowServer translates
    /// the complete composition atomically during a live drag.
    static MAIN_WINDOW_FOOTER_GLASS_HOSTS: std::cell::RefCell<
        std::collections::HashMap<usize, crate::platform::glass_button_host::NativeGlassContainerHost>
    > = std::cell::RefCell::new(std::collections::HashMap::new());
}

#[cfg(target_os = "macos")]
unsafe fn main_window_footer_glass_root(ns_window: id) -> id {
    MAIN_WINDOW_FOOTER_GLASS_HOSTS.with(|hosts| {
        let mut hosts = hosts.borrow_mut();
        hosts.retain(|_, host| host.window_is_alive());
        hosts
            .get(&(ns_window as usize))
            .map(|host| host.inner())
            .unwrap_or(nil)
    })
}

#[cfg(target_os = "macos")]
unsafe fn ensure_main_window_footer_glass_container(gpui_view: id, ns_window: id) -> id {
    use cocoa::appkit::NSViewWidthSizable;
    use cocoa::foundation::{NSPoint, NSRect, NSSize};
    use objc::{msg_send, sel, sel_impl};

    let content_view: id = msg_send![ns_window, contentView];
    if content_view == nil {
        return nil;
    }
    let content_bounds: NSRect = msg_send![content_view, bounds];
    let backing_scale: f64 = msg_send![ns_window, backingScaleFactor];
    let footer_bounds = main_window_footer_bounds_gpui(
        content_bounds.size.width as f32,
        content_bounds.size.height as f32,
        footer_height() as f32,
        FLOAT_FOOTER_CONTAINER_GAP_PX,
        backing_scale as f32,
    );
    let footer_frame = NSRect::new(
        NSPoint::new(
            footer_bounds.x as f64,
            content_bounds.size.height - (footer_bounds.y + footer_bounds.height) as f64,
        ),
        NSSize::new(footer_bounds.width as f64, footer_bounds.height as f64),
    );

    let existing = MAIN_WINDOW_FOOTER_GLASS_HOSTS.with(|hosts| {
        let hosts = hosts.borrow();
        hosts.get(&(ns_window as usize)).map(|host| {
            let container = host.container();
            let inner = host.inner();
            let _: () = msg_send![container, setFrame: footer_frame];
            let _: () = msg_send![
                inner,
                setFrame: NSRect::new(
                    NSPoint::new(0.0, 0.0),
                    NSSize::new(footer_frame.size.width, footer_frame.size.height)
                )
            ];
            inner
        })
    });
    if let Some(root) = existing {
        return root;
    }

    let Some(host) = crate::platform::glass_button_host::install_native_glass_container(
        ns_window,
        gpui_view,
        footer_frame,
        crate::platform::glass_button_host::NativeViewOrdering::AboveGpui,
        crate::platform::glass_button_host::shared_glass_spacing(),
        FOOTER_GLASS_CONTAINER_ID,
    ) else {
        return nil;
    };
    let _: () = msg_send![host.container(), setAutoresizingMask: NSViewWidthSizable];
    let root = host.inner();
    MAIN_WINDOW_FOOTER_GLASS_HOSTS.with(|hosts| {
        hosts.borrow_mut().insert(ns_window as usize, host);
    });
    root
}

#[cfg(target_os = "macos")]
unsafe fn remove_main_window_footer_glass_container(ns_window: id) {
    MAIN_WINDOW_FOOTER_GLASS_HOSTS.with(|hosts| {
        hosts.borrow_mut().remove(&(ns_window as usize));
    });
}

/// `CALayer.name` of the bottom-edge fade mask on the main GPUI layer. The
/// coherent entry may temporarily install its own foreground clip mask on
/// the same layer (and restores ours afterwards), so this owner only ever
/// updates or removes a mask carrying this name.
#[cfg(target_os = "macos")]
const MAIN_LIST_BOTTOM_EDGE_FADE_MASK_NAME: &str = "script-kit-main-list-bottom-edge-fade";

/// Unit-space gradient stops (y up, 0 = layer bottom) for the floating-footer
/// scroll edge: full opacity above `band_top`, ramping down to the floor over
/// `ramp_fraction` of the occluded band, then holding the floor to the bottom.
/// `None` when the geometry cannot express a band.
fn bottom_edge_fade_stops(
    layer_height: f64,
    band_top: f64,
    stage_bottom: f64,
    ramp_fraction: f64,
) -> Option<[f64; 2]> {
    if layer_height <= 1.0 || band_top <= stage_bottom || band_top > layer_height {
        return None;
    }
    let ramp_bottom = band_top - (band_top - stage_bottom) * ramp_fraction.clamp(0.0, 1.0);
    Some([
        (ramp_bottom / layer_height).clamp(0.0, 1.0),
        (band_top / layer_height).clamp(0.0, 1.0),
    ])
}

/// Tahoe floating-footer scroll edge: on the main menu the list scrolls under
/// the floating capsules, so fade the GPUI content there into the glass
/// (a true alpha mask, not a painted band), matching Apple's soft scroll
/// edge. Rows in the band stay visible but read as behind the chrome; GPUI
/// separately blocks hover/clicks there. Removed on every other surface.
///
/// # Safety
/// Main thread; `gpui_view` and `ns_window` belong to the live main window.
#[cfg(target_os = "macos")]
pub(crate) unsafe fn sync_main_list_bottom_edge_fade(gpui_view: id, ns_window: id) {
    use cocoa::base::YES;
    use cocoa::foundation::{NSPoint, NSRect, NSSize};
    use objc::{class, msg_send, sel, sel_impl};

    if gpui_view == nil {
        return;
    }
    let layer: id = msg_send![gpui_view, layer];
    if layer == nil {
        return;
    }
    let mask_name = ns_string(MAIN_LIST_BOTTOM_EDGE_FADE_MASK_NAME);
    let current: id = msg_send![layer, mask];
    let ours = current != nil && {
        let name: id = msg_send![current, name];
        let same: cocoa::base::BOOL = if name != nil && mask_name != nil {
            msg_send![name, isEqualToString: mask_name]
        } else {
            cocoa::base::NO
        };
        same == YES
    };
    if current != nil && !ours {
        // Another owner (entry foreground clip) holds the mask right now.
        log_main_list_bottom_edge_fade(BottomEdgeFadeState::DeferredForeignMask);
        return;
    }

    let container = MAIN_WINDOW_FOOTER_GLASS_HOSTS.with(|hosts| {
        hosts
            .borrow()
            .get(&(ns_window as usize))
            .map(|host| host.container())
            .unwrap_or(nil)
    });
    let layer_bounds: NSRect = msg_send![layer, bounds];
    let stops = if main_list_scrolls_under_footer() && container != nil {
        let container_frame: NSRect = msg_send![container, frame];
        let superview: id = msg_send![container, superview];
        // GPUI's view is not flipped and backs its layer, so view space is
        // the layer's y-up space.
        let footer: NSRect = msg_send![gpui_view, convertRect: container_frame fromView: superview];
        let tokens = crate::designs::current_main_menu_theme().def().list;
        // The band is the whole footer zone: from one gap above the capsules
        // (where keyboard reveal stops) down to the window bottom.
        let gap = f64::from(FLOAT_FOOTER_CONTAINER_GAP_PX);
        let band_top = footer.origin.y
            + footer.size.height
            + gap
            + f64::from(crate::list_item::effective_footer_reveal_clearance_height());
        let stage_bottom = footer.origin.y - gap;
        bottom_edge_fade_stops(
            layer_bounds.size.height,
            band_top,
            stage_bottom,
            f64::from(tokens.bottom_edge_fade_ramp_fraction),
        )
        .map(|stops| (stops, f64::from(tokens.bottom_edge_fade_floor_alpha)))
    } else {
        None
    };

    let transaction = objc::runtime::Class::get("CATransaction");
    if let Some(transaction) = transaction {
        let _: () = msg_send![transaction, begin];
        let _: () = msg_send![transaction, setDisableActions: YES];
    }
    match stops {
        None => {
            if ours {
                let _: () = msg_send![layer, setMask: nil];
            }
        }
        Some(([ramp_bottom, band_top], floor_alpha)) => {
            let mask: id = if ours {
                current
            } else if let Some(gradient_class) = objc::runtime::Class::get("CAGradientLayer") {
                let mask: id = msg_send![gradient_class, layer];
                if mask != nil {
                    let _: () = msg_send![mask, setName: mask_name];
                    let _: () = msg_send![mask, setStartPoint: NSPoint::new(0.5, 0.0)];
                    let _: () = msg_send![mask, setEndPoint: NSPoint::new(0.5, 1.0)];
                }
                mask
            } else {
                nil
            };
            if mask != nil {
                let white: id = msg_send![class!(NSColor), whiteColor];
                let faded: id = msg_send![white, colorWithAlphaComponent: floor_alpha];
                let faded_cg: id = msg_send![faded, CGColor];
                let full_cg: id = msg_send![white, CGColor];
                let colors = [faded_cg, faded_cg, full_cg, full_cg];
                let colors: id = msg_send![
                    class!(NSArray),
                    arrayWithObjects: colors.as_ptr()
                    count: colors.len()
                ];
                let locations = [0.0, ramp_bottom, band_top, 1.0].map(|value: f64| -> id {
                    msg_send![class!(NSNumber), numberWithDouble: value]
                });
                let locations: id = msg_send![
                    class!(NSArray),
                    arrayWithObjects: locations.as_ptr()
                    count: locations.len()
                ];
                let _: () = msg_send![mask, setColors: colors];
                let _: () = msg_send![mask, setLocations: locations];
                // Horizontally oversized so an entry width morph never
                // exposes an unmasked (hidden) strip at either edge.
                const WIDTH_SLACK: f64 = 4096.0;
                let _: () = msg_send![
                    mask,
                    setFrame: NSRect::new(
                        NSPoint::new(layer_bounds.origin.x - WIDTH_SLACK, layer_bounds.origin.y),
                        NSSize::new(
                            layer_bounds.size.width + WIDTH_SLACK * 2.0,
                            layer_bounds.size.height,
                        ),
                    )
                ];
                if !ours {
                    let _: () = msg_send![layer, setMask: mask];
                }
            }
        }
    }
    if let Some(transaction) = transaction {
        let _: () = msg_send![transaction, commit];
    }
    log_main_list_bottom_edge_fade(match stops {
        Some(([ramp_bottom_unit, band_top_unit], floor_alpha)) => BottomEdgeFadeState::Installed {
            layer_height: layer_bounds.size.height,
            band_top_unit,
            ramp_bottom_unit,
            floor_alpha,
        },
        None => BottomEdgeFadeState::Absent,
    });
}

#[cfg(target_os = "macos")]
#[derive(Clone, Copy, Debug, PartialEq)]
enum BottomEdgeFadeState {
    Absent,
    DeferredForeignMask,
    Installed {
        layer_height: f64,
        band_top_unit: f64,
        ramp_bottom_unit: f64,
        floor_alpha: f64,
    },
}

/// Change-only receipt for the bottom-edge fade owner (runs every render).
#[cfg(target_os = "macos")]
fn log_main_list_bottom_edge_fade(state: BottomEdgeFadeState) {
    static LAST: std::sync::Mutex<Option<BottomEdgeFadeState>> = std::sync::Mutex::new(None);
    let mut last = LAST
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if *last != Some(state) {
        tracing::info!(
            target: "script_kit::footer_popup",
            event = "main_list_bottom_edge_fade",
            state = ?state,
            "main_list_bottom_edge_fade"
        );
        *last = Some(state);
    }
}

/// Registry of the floating-footer window per parent (`(parent_ptr, footer_ptr)`).
///
/// The footer window is deliberately NOT attached via `addChildWindow:` —
/// attached children join the parent's window-server SHADOW GROUP, which puts
/// the capsule shapes back into the parent's shadow shape (the hairline
/// bridge between capsules, probe-proven). Ordering and visibility are
/// managed manually instead: frame/order in `sync_float_footer_child_frame`
/// (render-driven) and hide in the platform `orderOut:` choke points.
#[cfg(target_os = "macos")]
static FLOAT_FOOTER_WINDOWS: std::sync::Mutex<Vec<(usize, u64, usize)>> =
    std::sync::Mutex::new(Vec::new());

/// Find the floating-footer window registered for `ns_window`, if any.
#[cfg(target_os = "macos")]
unsafe fn float_footer_child_window(ns_window: id) -> id {
    let Some((_, binding, _)) = native_footer_binding(ns_window) else {
        return nil;
    };
    let guard = FLOAT_FOOTER_WINDOWS
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    guard
        .iter()
        .find(|(parent, generation, _)| {
            *parent == ns_window as usize && *generation == binding.host_generation
        })
        .map(|(_, _, footer)| *footer as id)
        .unwrap_or(nil)
}

/// Create (or reuse) the borderless, non-activating child panel that carries
/// the ENTIRE floating footer (host view: buttons, keycaps, left-info, and
/// their per-button glass capsules) below the main container.
///
/// Why a separate window:
/// - NSGlassEffectViews in the SAME window as the Tahoe backdrop auto-merge
///   with it across the 8px container gap (a full-width meniscus "shelf"
///   line bridging the capsules — user-reported).
/// - Any pixels left in the main window's strip (button text, keycaps) put
///   those rows back into the window-server shadow shape, which bridges them
///   into a rectangular rim around the strip.
///
/// Moving the whole footer out empties the strip completely: the main
/// window's shadow hugs the container, and the footer's glass samples the
/// desktop directly. The child has no shadow of its own.
#[cfg(target_os = "macos")]
unsafe fn ensure_float_footer_child_window(ns_window: id) -> id {
    use cocoa::foundation::{NSPoint, NSRect, NSSize};
    use objc::{class, msg_send, sel, sel_impl};

    if crate::runtime_policy::check(crate::runtime_policy::ExternalEffect::NativeVisibility)
        .is_err()
    {
        return nil;
    }
    let Some((_, binding, _)) = native_footer_binding(ns_window) else {
        return nil;
    };
    let existing = float_footer_child_window(ns_window);
    if existing != nil {
        return existing;
    }

    let frame: NSRect = msg_send![ns_window, frame];
    let child_frame = NSRect::new(
        NSPoint::new(
            frame.origin.x,
            frame.origin.y - f64::from(main_window_float_footer_strip_height()),
        ),
        NSSize::new(frame.size.width, footer_height()),
    );
    let child: id = msg_send![class!(NSPanel), alloc];
    // styleMask 128 = borderless non-activating panel; backing 2 = buffered.
    let child: id = msg_send![
        child,
        initWithContentRect: child_frame
        styleMask: 128u64
        backing: 2u64
        defer: NO
    ];
    if child == nil {
        return nil;
    }
    let clear: id = msg_send![class!(NSColor), clearColor];
    let _: () = msg_send![child, setBackgroundColor: clear];
    let _: () = msg_send![child, setOpaque: NO];
    let _: () = msg_send![child, setHasShadow: NO];
    let _: () = msg_send![child, setReleasedWhenClosed: NO];
    let _: () = msg_send![child, setBecomesKeyOnlyIfNeeded: YES];
    let level: isize = msg_send![ns_window, level];
    let _: () = msg_send![child, setLevel: level];

    let content: id = msg_send![child, contentView];
    if content != nil {
        let identifier = ns_string(FLOAT_FOOTER_LAYER_ID);
        if identifier != nil {
            let _: () = msg_send![content, setIdentifier: identifier];
        }
        let _: () = msg_send![content, setWantsLayer: YES];
    }

    // Match the parent's Spaces/collection behavior so the footer follows the
    // launcher across Spaces and fullscreen setups.
    let collection_behavior: u64 = msg_send![ns_window, collectionBehavior];
    let _: () = msg_send![child, setCollectionBehavior: collection_behavior];

    FLOAT_FOOTER_WINDOWS
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
        .push((ns_window as usize, binding.host_generation, child as usize));

    tracing::info!(
        target: "script_kit::footer_popup",
        event = "float_footer_child_window_installed",
        height = footer_height(),
        "Installed floating-footer window (unattached, shadow-group-free) below the main container"
    );
    child
}

/// Hide and unregister the floating-footer window, if present.
#[cfg(target_os = "macos")]
unsafe fn remove_float_footer_child_window(ns_window: id) {
    use objc::{msg_send, sel, sel_impl};
    let generation = FOOTER_HOSTS
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .values()
        .find(|host| host.native_window == ns_window as usize)
        .map(|host| host.host_generation);
    let Some(generation) = generation else {
        return;
    };
    let mut windows = FLOAT_FOOTER_WINDOWS
        .lock()
        .unwrap_or_else(|p| p.into_inner());
    if let Some(index) = windows
        .iter()
        .position(|(parent, lifetime, _)| *parent == ns_window as usize && *lifetime == generation)
    {
        let (_, _, child) = windows.remove(index);
        let child = child as id;
        let _: () = msg_send![child, orderOut: nil];
        let _: () = msg_send![child, close];
        let _: () = msg_send![child, release];
    }
}

/// Hide the floating-footer window alongside its parent (called from the
/// platform `orderOut:` choke points — the footer is unattached, so AppKit
/// will not hide it for us). Keeps the registration for the next show.
#[cfg(target_os = "macos")]
pub(crate) fn hide_float_footer_for_window(ns_window: id) {
    if crate::runtime_policy::check(crate::runtime_policy::ExternalEffect::NativeVisibility)
        .is_err()
    {
        return;
    }
    use objc::{msg_send, sel, sel_impl};

    // SAFETY: called on the main thread from the platform hide paths; the
    // registered footer window pointer is retained for the process lifetime.
    unsafe {
        let child = float_footer_child_window(ns_window);
        if child != nil {
            let _: () = msg_send![child, orderOut: nil];
        }
    }
}

/// Keep the floating-footer window glued to the strip BELOW the main
/// window's frame (the frame ends at the container; the strip is outside it —
/// see `window_resize::physical_main_window_height`) and mirror the parent's
/// on-screen state (unattached window: manual ordering) and appearance (the
/// capsule glass must adapt to the same light/dark appearance as the main
/// window's backdrop, not the child's own resolved appearance).
#[cfg(target_os = "macos")]
unsafe fn sync_float_footer_child_frame(ns_window: id) {
    use cocoa::foundation::{NSPoint, NSRect, NSSize};
    use objc::{msg_send, sel, sel_impl};

    if crate::runtime_policy::check(crate::runtime_policy::ExternalEffect::NativeVisibility)
        .is_err()
    {
        return;
    }
    let child = float_footer_child_window(ns_window);
    if child == nil {
        return;
    }
    let main_frame: NSRect = msg_send![ns_window, frame];
    let strip = f64::from(main_window_float_footer_strip_height());
    let child_frame = NSRect::new(
        NSPoint::new(main_frame.origin.x, main_frame.origin.y - strip),
        NSSize::new(main_frame.size.width, footer_height()),
    );
    let _: () = msg_send![child, setFrame: child_frame display: YES];

    let parent_appearance: id = msg_send![ns_window, effectiveAppearance];
    if parent_appearance != nil {
        let _: () = msg_send![child, setAppearance: parent_appearance];
    }

    // Re-assert shadowlessness every pass: the capsule shapes otherwise get a
    // window-server shadow whose row spans bridge the gaps between capsules
    // into a hairline shelf (probe-proven).
    let child_has_shadow: cocoa::base::BOOL = msg_send![child, hasShadow];
    if child_has_shadow == YES {
        let _: () = msg_send![child, setHasShadow: NO];
        tracing::info!(
            target: "script_kit::footer_popup",
            event = "float_footer_shadow_reasserted",
            "Floating footer window shadow was re-enabled by AppKit; disabled again"
        );
    }
    let _: () = msg_send![child, invalidateShadow];

    let parent_visible: cocoa::base::BOOL = msg_send![ns_window, isVisible];
    let child_visible: cocoa::base::BOOL = msg_send![child, isVisible];
    if parent_visible == YES {
        let parent_number: isize = msg_send![ns_window, windowNumber];
        let _: () = msg_send![child, orderWindow: 1isize relativeTo: parent_number];
    } else if child_visible == YES {
        let _: () = msg_send![child, orderOut: nil];
    }
}

/// Resolve the view that footer host lookups should search: the floating
/// child window's contentView when the float footer is active, otherwise the
/// main window's contentView (blur-era in-window host).
#[cfg(target_os = "macos")]
unsafe fn reusable_window_footer_search_root(ns_window: id) -> id {
    use objc::{msg_send, sel, sel_impl};

    let child = float_footer_child_window(ns_window);
    if child != nil {
        let content: id = msg_send![child, contentView];
        if content != nil {
            return content;
        }
    }
    msg_send![ns_window, contentView]
}

/// Main-window footer lookup root. Tahoe uses the same-window glass
/// container's inner view; fallback mode keeps the existing in-content host.
#[cfg(target_os = "macos")]
unsafe fn main_window_footer_search_root(ns_window: id) -> id {
    use objc::{msg_send, sel, sel_impl};

    let glass_root = main_window_footer_glass_root(ns_window);
    if glass_root != nil {
        glass_root
    } else {
        msg_send![ns_window, contentView]
    }
}

/// Debug aid (SCRIPT_KIT_GLASS_BAND_DEBUG=1): walk the contentView tree and
/// log every view whose frame intersects the transparent footer strip, with
/// visibility/alpha/layer state — used to find what still contributes pixels
/// (and therefore window-server shape) inside the strip.
#[cfg(target_os = "macos")]
unsafe fn log_strip_views_debug(ns_window: id) {
    use objc::{msg_send, sel, sel_impl};

    if std::env::var("SCRIPT_KIT_GLASS_BAND_DEBUG").is_err() {
        return;
    }
    let content_view: id = msg_send![ns_window, contentView];
    if content_view == nil {
        return;
    }
    let strip = f64::from(main_window_float_footer_strip_height());
    if strip <= 0.0 {
        return;
    }

    unsafe fn walk(view: id, content_view: id, strip: f64, depth: usize, out: &mut Vec<String>) {
        use objc::{msg_send, sel, sel_impl};
        let subviews: id = msg_send![view, subviews];
        if subviews == nil {
            return;
        }
        let count: usize = msg_send![subviews, count];
        for index in 0..count {
            let child: id = msg_send![subviews, objectAtIndex: index];
            if child == nil {
                continue;
            }
            let frame: cocoa::foundation::NSRect = msg_send![child, frame];
            let superview: id = msg_send![child, superview];
            let origin_in_content: cocoa::foundation::NSPoint = msg_send![
                content_view,
                convertPoint: frame.origin
                fromView: superview
            ];
            if origin_in_content.y < strip + 2.0 {
                let hidden: cocoa::base::BOOL = msg_send![child, isHidden];
                let alpha: f64 = msg_send![child, alphaValue];
                let cls: id = msg_send![child, class];
                let cls_name: id = msg_send![cls, className];
                let utf8: *const std::os::raw::c_char = msg_send![cls_name, UTF8String];
                let cls_name = if utf8.is_null() {
                    String::new()
                } else {
                    std::ffi::CStr::from_ptr(utf8)
                        .to_string_lossy()
                        .into_owned()
                };
                let layer: id = msg_send![child, layer];
                let layer_bg_alpha = if layer != nil {
                    let bg: *const std::ffi::c_void = msg_send![layer, backgroundColor];
                    if bg.is_null() {
                        -1.0
                    } else {
                        #[link(name = "CoreGraphics", kind = "framework")]
                        extern "C" {
                            fn CGColorGetAlpha(color: *const std::ffi::c_void) -> f64;
                        }
                        CGColorGetAlpha(bg)
                    }
                } else {
                    -2.0
                };
                out.push(format!(
                    "{}{} y={:.1} frame=({:.1},{:.1},{:.1},{:.1}) hidden={} alpha={:.4} layer_bg_alpha={:.4}",
                    "  ".repeat(depth),
                    cls_name,
                    origin_in_content.y,
                    frame.origin.x,
                    frame.origin.y,
                    frame.size.width,
                    frame.size.height,
                    hidden == YES,
                    alpha,
                    layer_bg_alpha,
                ));
            }
            walk(child, content_view, strip, depth + 1, out);
        }
    }

    let mut out = Vec::new();
    walk(content_view, content_view, strip, 0, &mut out);
    let has_shadow: cocoa::base::BOOL = msg_send![ns_window, hasShadow];
    tracing::info!(
        target: "script_kit::footer_popup",
        event = "glass_strip_view_dump",
        window_has_shadow = has_shadow == YES,
        views = %out.join(" | "),
        "Views intersecting the transparent footer strip"
    );
}
