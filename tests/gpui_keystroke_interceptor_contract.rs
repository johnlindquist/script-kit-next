const GPUI_WINDOW_SOURCE: &str = include_str!("../vendor/gpui/src/window.rs");

fn function_body<'a>(source: &'a str, signature: &str) -> &'a str {
    let start = source
        .find(signature)
        .unwrap_or_else(|| panic!("missing function signature: {signature}"));
    let tail = &source[start..];
    let body_start = tail
        .find('{')
        .unwrap_or_else(|| panic!("missing body for function: {signature}"));
    let mut depth = 0usize;
    let mut end = None;
    for (offset, ch) in tail[body_start..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    end = Some(body_start + offset + ch.len_utf8());
                    break;
                }
            }
            _ => {}
        }
    }
    &tail[..end.unwrap_or_else(|| panic!("unterminated function: {signature}"))]
}

/// GPUI keystroke interceptors are global and registered in order. A Script Kit
/// menu-syntax Enter accept calls `stop_propagation`; if later interceptors
/// still run for that same key, stale main-list shortcut metadata can execute a
/// script after the picker already accepted. The vendored GPUI test harness is
/// not runnable in this checkout because package tests fail on missing vendored
/// font assets, so this contract locks the dispatch-loop invariant directly.
#[test]
fn keystroke_interceptor_dispatch_skips_later_callbacks_after_stop() {
    let body = function_body(
        GPUI_WINDOW_SOURCE,
        "pub(crate) fn dispatch_keystroke_interceptors",
    );

    let skip_guard = body
        .find("if propagation_stopped || !cx.propagate_event")
        .expect("dispatch must check propagation before invoking each interceptor");
    let skip_return = body[skip_guard..]
        .find("return true;")
        .map(|offset| skip_guard + offset)
        .expect("skipped interceptors must be retained, not unsubscribed");
    let callback_call = body
        .find("let keep = (callback)(")
        .expect("dispatch must still invoke live interceptors");
    let stopped_after_callback = body[callback_call..]
        .find("if !cx.propagate_event")
        .map(|offset| callback_call + offset)
        .expect("dispatch must remember when an interceptor stops propagation");

    assert!(
        skip_guard < callback_call,
        "propagation must be checked before invoking the next interceptor"
    );
    assert!(
        skip_return < callback_call,
        "the skipped-interceptor branch must return before callback invocation"
    );
    assert!(
        stopped_after_callback > callback_call,
        "dispatch must set the stopped flag after a callback stops propagation"
    );
    assert!(
        body.contains("keystroke_interceptor_skipped_after_stop"),
        "trace logging must prove later interceptors were skipped"
    );
    assert!(
        body.contains("keystroke_interceptor_stopped_propagation"),
        "trace logging must prove which interceptor stopped propagation"
    );
}
