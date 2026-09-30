/// Opt-in, frame-by-frame timeline of a main-window glass entry. While
/// `GLASS_ENTRY_TRACE_FLAG` exists, every main entry logs one
/// `event=glass_entry_trace` line per sample for `GLASS_ENTRY_TRACE_SPAN_NS`:
/// model/presentation opacity of the window, backdrop material, GPUI content
/// roots and footer capsules, plus every CA animation attached in the content
/// view's layer tree with how far into its duration it is. `gap_ms` exposes
/// main-thread stalls between samples.
#[cfg(target_os = "macos")]
const GLASS_ENTRY_TRACE_FLAG: &str = "/tmp/sk-glass-entry-trace";

#[cfg(target_os = "macos")]
const GLASS_ENTRY_TRACE_SPAN_NS: u64 = 400_000_000;

#[cfg(target_os = "macos")]
const GLASS_ENTRY_TRACE_INTERVAL_SECONDS: f64 = 1.0 / 240.0;

#[cfg(target_os = "macos")]
struct GlassEntryTrace {
    window: usize,
    backdrop: usize,
    armed_ns: u64,
    last_ns: u64,
    seq: u32,
}

#[cfg(target_os = "macos")]
static GLASS_ENTRY_TRACE: std::sync::Mutex<Option<GlassEntryTrace>> = std::sync::Mutex::new(None);

#[cfg(target_os = "macos")]
#[link(name = "QuartzCore", kind = "framework")]
extern "C" {
    fn CACurrentMediaTime() -> f64;
}

#[cfg(target_os = "macos")]
unsafe fn glass_entry_trace_string(value: id) -> String {
    if value == nil {
        return String::new();
    }
    let utf8: *const std::os::raw::c_char = msg_send![value, UTF8String];
    if utf8.is_null() {
        return String::new();
    }
    std::ffi::CStr::from_ptr(utf8).to_string_lossy().into_owned()
}

#[cfg(target_os = "macos")]
unsafe fn glass_entry_trace_describe(value: id) -> String {
    if value == nil {
        "nil".into()
    } else {
        glass_entry_trace_string(msg_send![value, description])
    }
}

/// `model/presentation` opacity; `nopres` means nothing has been committed
/// to the render tree for that layer yet.
#[cfg(target_os = "macos")]
unsafe fn glass_entry_trace_opacity(view: id) -> String {
    if view == nil {
        return "-".into();
    }
    let alpha: f64 = msg_send![view, alphaValue];
    let layer: id = msg_send![view, layer];
    if layer == nil {
        return format!("{alpha:.3}/nolayer");
    }
    let presentation: id = msg_send![layer, presentationLayer];
    if presentation == nil {
        return format!("{alpha:.3}/nopres");
    }
    let shown: f32 = msg_send![presentation, opacity];
    format!("{alpha:.3}/{shown:.3}")
}

#[cfg(target_os = "macos")]
unsafe fn glass_entry_trace_style(view: id) -> String {
    let style_ok: bool = msg_send![view, respondsToSelector: sel!(style)];
    let tint_ok: bool = msg_send![view, respondsToSelector: sel!(tintColor)];
    if !style_ok || !tint_ok {
        return "-".into();
    }
    let style: isize = msg_send![view, style];
    let tint: id = msg_send![view, tintColor];
    if tint == nil {
        format!("s{style}:tint=nil")
    } else {
        let tint_alpha: f64 = msg_send![tint, alphaComponent];
        format!("s{style}:tint_a={tint_alpha:.3}")
    }
}

#[cfg(target_os = "macos")]
unsafe fn glass_entry_trace_collect_animations(
    layer: id,
    depth: usize,
    now: f64,
    out: &mut Vec<String>,
) {
    if layer == nil || depth > 12 || out.len() >= 32 {
        return;
    }
    let keys: id = msg_send![layer, animationKeys];
    if keys != nil {
        let count: usize = msg_send![keys, count];
        let class_name = glass_entry_trace_string(msg_send![layer, className]);
        let local_now: f64 = msg_send![layer, convertTime: now fromLayer: nil];
        for index in 0..count {
            let key: id = msg_send![keys, objectAtIndex: index];
            let animation: id = msg_send![layer, animationForKey: key];
            if animation == nil {
                continue;
            }
            let has_key_path: bool = msg_send![animation, respondsToSelector: sel!(keyPath)];
            let key_path = if has_key_path {
                glass_entry_trace_string(msg_send![animation, keyPath])
            } else {
                String::new()
            };
            let has_from: bool = msg_send![animation, respondsToSelector: sel!(fromValue)];
            let (from, to) = if has_from {
                (
                    glass_entry_trace_describe(msg_send![animation, fromValue]),
                    glass_entry_trace_describe(msg_send![animation, toValue]),
                )
            } else {
                ("-".into(), "-".into())
            };
            let duration: f64 = msg_send![animation, duration];
            let begin: f64 = msg_send![animation, beginTime];
            out.push(format!(
                "{}@d{}:{}[{}]{}->{},dur={:.0}ms,in={:.1}ms",
                class_name,
                depth,
                glass_entry_trace_string(key),
                key_path,
                from,
                to,
                duration * 1000.0,
                (local_now - begin) * 1000.0,
            ));
        }
    }
    let sublayers: id = msg_send![layer, sublayers];
    if sublayers != nil {
        let count: usize = msg_send![sublayers, count];
        for index in 0..count {
            let child: id = msg_send![sublayers, objectAtIndex: index];
            glass_entry_trace_collect_animations(child, depth + 1, now, out);
        }
    }
}

#[cfg(target_os = "macos")]
unsafe fn glass_entry_trace_sample(trace: &mut GlassEntryTrace) -> u64 {
    let window = trace.window as id;
    let backdrop = trace.backdrop as id;
    let now_ns = crate::platform::host_clock::host_time_ns();
    let t_ns = now_ns.saturating_sub(trace.armed_ns);
    let gap_ms = now_ns.saturating_sub(trace.last_ns) as f64 / 1e6;
    trace.last_ns = now_ns;

    let window_alpha: f64 = msg_send![window, alphaValue];
    let visible: bool = msg_send![window, isVisible];
    let content_view: id = msg_send![window, contentView];
    let footer_host = crate::footer_popup::main_window_footer_entry_alpha_target(window);
    let mut content = Vec::new();
    if content_view != nil {
        let subviews: id = msg_send![content_view, subviews];
        let count: usize = if subviews == nil {
            0
        } else {
            msg_send![subviews, count]
        };
        for index in 0..count {
            let child: id = msg_send![subviews, objectAtIndex: index];
            if child != backdrop && child != footer_host {
                content.push(glass_entry_trace_opacity(child));
            }
        }
    }
    let capsules: Vec<String> = crate::footer_popup::main_window_footer_entry_capsules(window)
        .into_iter()
        .map(|capsule| {
            format!(
                "{}|{}",
                glass_entry_trace_opacity(capsule),
                glass_entry_trace_style(capsule)
            )
        })
        .collect();
    let mut animations = Vec::new();
    if content_view != nil {
        let root: id = msg_send![content_view, layer];
        glass_entry_trace_collect_animations(root, 0, CACurrentMediaTime(), &mut animations);
    }
    let entry_state = GLASS_COHERENT_ENTRY_STATES
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .contains_key(&(window as usize));
    logging::log(
        "PANEL",
        &format!(
            "event=glass_entry_trace seq={} t_ms={:.1} gap_ms={:.1} visible={} entry_state={} win_alpha={:.3} backdrop={}|{} footer_host={} content=[{}] capsules=[{}] animations=[{}]",
            trace.seq,
            t_ns as f64 / 1e6,
            gap_ms,
            visible,
            entry_state,
            window_alpha,
            glass_entry_trace_opacity(backdrop),
            glass_entry_trace_style(backdrop),
            glass_entry_trace_opacity(footer_host),
            content.join(","),
            capsules.join(","),
            animations.join(" "),
        ),
    );
    trace.seq += 1;
    t_ns
}

/// Called once the entry's animations are armed; `armed_ns` is the host time
/// the entry began configuring, so `t_ms` lines up with its other receipts.
#[cfg(target_os = "macos")]
unsafe fn begin_glass_entry_trace(window: id, backdrop: id, armed_ns: u64) {
    if !std::path::Path::new(GLASS_ENTRY_TRACE_FLAG).exists() {
        return;
    }
    let _: () = msg_send![
        class!(NSObject),
        cancelPreviousPerformRequestsWithTarget: backdrop
        selector: sel!(sampleOwnWindowEntryTrace)
        object: nil
    ];
    let mut slot = GLASS_ENTRY_TRACE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let trace = slot.insert(GlassEntryTrace {
        window: window as usize,
        backdrop: backdrop as usize,
        armed_ns,
        last_ns: armed_ns,
        seq: 0,
    });
    glass_entry_trace_sample(trace);
    drop(slot);
    let _: () = msg_send![backdrop,
        performSelector: sel!(sampleOwnWindowEntryTrace)
        withObject: nil
        afterDelay: GLASS_ENTRY_TRACE_INTERVAL_SECONDS
    ];
}

#[cfg(target_os = "macos")]
extern "C" fn tahoe_glass_backdrop_sample_entry_trace(
    this: &objc::runtime::Object,
    _: objc::runtime::Sel,
) {
    unsafe {
        let view = this as *const objc::runtime::Object as id;
        let mut slot = GLASS_ENTRY_TRACE
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(trace) = slot.as_mut() else {
            return;
        };
        let window: id = msg_send![view, window];
        if trace.backdrop != view as usize || window == nil || trace.window != window as usize {
            *slot = None;
            return;
        }
        if glass_entry_trace_sample(trace) >= GLASS_ENTRY_TRACE_SPAN_NS {
            *slot = None;
            return;
        }
        drop(slot);
        let _: () = msg_send![view,
            performSelector: sel!(sampleOwnWindowEntryTrace)
            withObject: nil
            afterDelay: GLASS_ENTRY_TRACE_INTERVAL_SECONDS
        ];
    }
}
