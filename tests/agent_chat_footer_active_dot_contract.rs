//! Source-level contracts for the embedded Agent Chat footer activity indicator.

const FOOTER_POPUP_SOURCE: &str = include_str!("../src/footer_popup.rs");

fn fn_body<'a>(source: &'a str, signature: &str) -> &'a str {
    let start = source.find(signature).expect("signature must exist");
    let rest = &source[start..];
    let body_start = rest.find('{').expect("function body must start");
    let mut depth = 0usize;
    for (idx, ch) in rest[body_start..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return &rest[..body_start + idx + 1];
                }
            }
            _ => {}
        }
    }
    panic!("function body must close");
}

#[test]
fn native_profile_icon_pulse_uses_opacity_without_scaling() {
    assert!(
        FOOTER_POPUP_SOURCE.contains("FOOTER_ACTIVE_DOT_MIN_OPACITY"),
        "native active profile icon opacity constant must exist"
    );
    assert!(
        FOOTER_POPUP_SOURCE.contains("FOOTER_ACTIVE_DOT_HALF_CYCLE_SECONDS"),
        "native active dot half cycle seconds constant must exist"
    );
    assert!(
        FOOTER_POPUP_SOURCE.contains("update_footer_icon_layer(icon_layer, info);"),
        "native footer must pulse the profile icon for active Agent Chat states"
    );
    let body = fn_body(
        FOOTER_POPUP_SOURCE,
        "unsafe fn add_active_dot_pulse_animation(",
    );
    assert!(
        body.contains("ns_string(\"opacity\")") && body.contains("pulseOpacity"),
        "native active profile icon must pulse opacity"
    );
    assert!(
        !body.contains("transform.scale")
            && !body.contains("pulseScale")
            && !FOOTER_POPUP_SOURCE.contains("FOOTER_ACTIVE_DOT_MAX_SCALE")
            && !FOOTER_POPUP_SOURCE.contains("FOOTER_ACTIVE_DOT_MIN_SCALE"),
        "native active dot must not scale while pulsing"
    );
}

#[test]
fn native_dot_animation_uses_core_animation_duration_abi_safely() {
    let body = fn_body(
        FOOTER_POPUP_SOURCE,
        "unsafe fn add_active_dot_pulse_animation(",
    );

    assert!(
        body.contains("let duration: f64")
            && body.contains("setDuration: duration")
            && !body.contains("setDuration: FOOTER_ACTIVE_DOT_HALF_CYCLE_SECONDS"),
        "CABasicAnimation setDuration: must receive f64/CFTimeInterval, not an inferred numeric type"
    );
}
