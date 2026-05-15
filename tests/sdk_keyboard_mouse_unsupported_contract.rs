const SDK: &str = include_str!("../scripts/kit-sdk.ts");

fn body_after(marker: &str) -> &str {
    let start = SDK.find(marker).expect("missing SDK marker");
    &SDK[start..]
}

fn function_body(marker: &str) -> &str {
    let tail = body_after(marker);
    let end = tail
        .find("\n  },")
        .or_else(|| tail.find("\n  }"))
        .expect("missing function body end");
    &tail[..end]
}

#[test]
fn keyboard_helpers_throw_typed_unsupported_errors_before_send() {
    let keyboard_type = function_body("type(_text: string): Promise<never> {");
    assert!(keyboard_type.contains("rejectUnsupportedSdkFeature('keyboard.type'"));
    assert!(keyboard_type.contains("batch.setInput"));
    assert!(
        !keyboard_type.contains("send("),
        "keyboard.type must not send a fire-and-forget message"
    );

    let keyboard_tap = function_body("tap(..._keys: string[]): Promise<never> {");
    assert!(keyboard_tap.contains("rejectUnsupportedSdkFeature('keyboard.tap'"));
    assert!(keyboard_tap.contains("simulateKey plus getState/getElements/waitFor"));
    assert!(
        !keyboard_tap.contains("send("),
        "keyboard.tap must not send a fire-and-forget message"
    );
}

#[test]
fn mouse_helpers_throw_typed_unsupported_errors_before_send() {
    for (marker, api) in [
        (
            "move(_positions: Position[]): Promise<never> {",
            "mouse.move",
        ),
        ("leftClick(): Promise<never> {", "mouse.leftClick"),
        ("rightClick(): Promise<never> {", "mouse.rightClick"),
        (
            "setPosition(_position: Position): Promise<never> {",
            "mouse.setPosition",
        ),
    ] {
        let body = function_body(marker);
        assert!(body.contains(&format!("rejectUnsupportedSdkFeature('{api}'")));
        assert!(body.contains("state-first automation") || body.contains("semantic action APIs"));
        assert!(
            !body.contains("send("),
            "{api} must not send a fire-and-forget message"
        );
    }
}

#[test]
fn unsupported_sdk_api_error_shape_is_stable() {
    assert!(SDK.contains("export class UnsupportedSdkFeatureError extends Error"));
    assert!(
        SDK.contains("readonly code: UnsupportedSdkFeatureCode = 'ERR_UNSUPPORTED_SDK_FEATURE'")
    );
    assert!(SDK.contains("readonly supported = false as const;"));
    assert!(SDK.contains("readonly feature: string;"));
    assert!(SDK.contains("readonly alternatives: string[];"));
    assert!(SDK.contains("this.name = 'UnsupportedSdkFeatureError';"));
}
