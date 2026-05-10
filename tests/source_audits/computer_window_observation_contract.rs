// @lat: [[protocol#Protocol#Tool exposure]]
#[test]
fn computer_window_observation_is_additive_read_only_metadata() {
    let runtime = std::fs::read_to_string("src/computer_use/runtime_bridge.rs")
        .expect("read runtime_bridge.rs");
    let bridge = std::fs::read_to_string("src/computer_use/gpui_runtime_bridge.rs")
        .expect("read gpui_runtime_bridge.rs");
    let module = std::fs::read_to_string("src/computer_use/window_observation.rs")
        .expect("read window_observation.rs");
    let computer_mod =
        std::fs::read_to_string("src/computer_use/mod.rs").expect("read computer_use/mod.rs");
    let mcp_tools = std::fs::read_to_string("src/mcp_computer_use_tools.rs")
        .expect("read mcp_computer_use_tools.rs");
    let protocol = std::fs::read_to_string("lat.md/protocol.md").expect("read protocol docs");

    assert!(
        computer_mod.contains("pub mod window_observation;"),
        "window observation contract must be part of the computer_use module"
    );
    assert!(
        runtime.contains("pub observation: Option<ComputerUseWindowObservationV1>,"),
        "native window info must carry additive optional observation metadata"
    );
    assert!(
        runtime.contains("#[serde(skip_serializing_if = \"Option::is_none\")]"),
        "observation metadata must remain additive for callers that construct None"
    );

    for needle in [
        "pub struct ComputerUseWindowObservationV1",
        "#[serde(skip_serializing_if = \"Option::is_none\")]",
        "pub duplicate_group: Option<WindowDuplicateGroupV1>,",
        "pub title_fallback: Option<WindowTitleFallbackV1>,",
        "pub own_process_window_policy: Option<WindowOwnProcessPolicyV1>,",
        "pub enum WindowObservationMetadataQuality",
        "pub struct WindowCaptureCandidateV1",
        "pub enum WindowCaptureCandidateStatus",
        "pub enum WindowDisqualificationReason",
        "pub struct WindowCaptureThresholdsV1",
        "pub struct WindowDuplicateGroupV1",
        "pub enum WindowDuplicateGroupStatus",
        "pub enum WindowDuplicateSelectionBasis",
        "pub struct WindowDuplicateObservationInputV1",
        "pub fn window_duplicate_groups_v1(",
        "pub struct WindowTitleFallbackV1",
        "pub enum WindowTitleFallbackStatus",
        "pub enum WindowTitleFallbackSelectionBasis",
        "pub struct WindowTitleFallbackObservationInputV1",
        "pub fn window_title_fallbacks_v1(",
        "pub struct WindowOwnProcessPolicyV1",
        "pub enum WindowOwnProcessPolicyStatus",
        "pub fn window_own_process_policy_v1(",
        "pub const COMPUTER_USE_WINDOW_OBSERVATION_SCHEMA_VERSION: u32 = 1;",
        "pub const WINDOW_CAPTURE_REQUIRED_LAYER: i64 = 0;",
        "pub const WINDOW_CAPTURE_MIN_ALPHA: f64 = 0.01;",
        "pub const WINDOW_CAPTURE_MIN_WIDTH: u32 = 120;",
        "pub const WINDOW_CAPTURE_MIN_HEIGHT: u32 = 90;",
    ] {
        assert!(
            module.contains(needle),
            "window observation missing {needle}"
        );
    }

    let helper_body = extract_function_body(&module, "pub fn window_capture_candidate_v1(");
    for needle in [
        "layer != WINDOW_CAPTURE_REQUIRED_LAYER",
        "WindowDisqualificationReason::LayerNonZero",
        "value <= WINDOW_CAPTURE_MIN_ALPHA",
        "WindowDisqualificationReason::AlphaTooLow",
        "sharing_state == Some(CG_WINDOW_SHARING_NONE)",
        "WindowDisqualificationReason::SharingStateNone",
        "!is_on_screen",
        "WindowDisqualificationReason::NotOnScreen",
        "bounds.width < WINDOW_CAPTURE_MIN_WIDTH || bounds.height < WINDOW_CAPTURE_MIN_HEIGHT",
        "WindowDisqualificationReason::TooSmall",
        "alpha.is_none() || sharing_state.is_none()",
        "WindowDisqualificationReason::MetadataIncomplete",
        "WindowCaptureCandidateStatus::Candidate",
        "WindowCaptureCandidateStatus::Disqualified",
        "WindowCaptureCandidateStatus::Unknown",
    ] {
        assert!(
            helper_body.contains(needle),
            "capture candidate helper must pin {needle}"
        );
    }
    for forbidden in [
        "CoreGraphics",
        "CGWindowListCopyWindowInfo",
        "NSWorkspace",
        "AppKit",
        "objc::",
        "focus",
        "activate",
        "launch",
        "quit",
        "hide",
        "move",
        "resize",
        "click",
        "press",
        "execute",
        "request_accessibility_permission",
        "capture_targeted_screenshot",
        "WindowOwnProcessPolicy",
        "own_process",
        "is_excluded_from_windows_menu",
    ] {
        assert!(
            !helper_body.contains(forbidden),
            "capture candidate helper must stay pure/read-only; found {forbidden}"
        );
    }

    for needle in [
        "let k_window_alpha = CFString::new(\"kCGWindowAlpha\");",
        "let k_window_sharing_state = CFString::new(\"kCGWindowSharingState\");",
        "let alpha = cf_number_f64(dict_ref, &k_window_alpha);",
        "let sharing_state = cf_number_i64(dict_ref, &k_window_sharing_state);",
        "computer_use_window_observation_v1(&bounds, is_on_screen, layer, alpha, sharing_state)",
        "observation: Some(observation)",
    ] {
        assert!(
            bridge.contains(needle),
            "CoreGraphics window bridge must populate observation metadata: {needle}"
        );
    }

    let duplicate_helper_body =
        extract_function_body(&module, "pub fn window_duplicate_groups_v1(");
    for needle in [
        "candidate.native_window_id == window.native_window_id",
        "group_count < 2",
        "WindowDuplicateGroupStatus::Preferred",
        "WindowDuplicateGroupStatus::Duplicate",
        "preferred_z_order: preferred.z_order",
        "WindowDuplicateSelectionBasis::OnScreenThenLargestAreaThenLowestZOrder",
        "candidate.is_on_screen",
        "window_area(&candidate.bounds)",
        "std::cmp::Reverse(candidate.z_order)",
        "std::ptr::eq(preferred, window)",
    ] {
        assert!(
            duplicate_helper_body.contains(needle),
            "duplicate observation helper must pin {needle}"
        );
    }
    for forbidden in [
        "CoreGraphics",
        "CGWindowListCopyWindowInfo",
        "NSWorkspace",
        "AppKit",
        "retain(",
        "dedup",
        "remove(",
        "sort",
        "focus",
        "activate",
        "capture",
        "click",
        "press",
        "execute",
    ] {
        assert!(
            !duplicate_helper_body.contains(forbidden),
            "duplicate observation helper must stay diagnostic-only; found {forbidden}"
        );
    }

    let title_helper_body = extract_function_body(&module, "pub fn window_title_fallbacks_v1(");
    for needle in [
        "let eligible_candidate_count = windows.iter().filter(|window| window.is_eligible()).count();",
        "WindowTitleFallbackStatus::NonEmptyTitle",
        "WindowTitleFallbackStatus::EmptyTitleSoleCandidate",
        "WindowTitleFallbackStatus::EmptyTitleAmongMultipleCandidates",
        "WindowTitleFallbackSelectionBasis::PreferNonEmptyTitleThenAllowEmptyOnlyIfSoleCandidate",
        ".title",
        ".trim().is_empty()",
        "eligible_candidate_count == 1",
    ] {
        assert!(
            title_helper_body.contains(needle),
            "title fallback helper must pin {needle}"
        );
    }
    let title_eligibility_body = extract_function_body(&module, "fn is_eligible(&self) -> bool");
    for needle in [
        "self.capture_candidate_status == WindowCaptureCandidateStatus::Candidate",
        "self.duplicate_group_status != Some(WindowDuplicateGroupStatus::Duplicate)",
    ] {
        assert!(
            title_eligibility_body.contains(needle),
            "title fallback eligibility must pin {needle}"
        );
    }
    for forbidden in [
        "CoreGraphics",
        "CGWindowListCopyWindowInfo",
        "NSWorkspace",
        "AppKit",
        "retain(",
        "dedup",
        "remove(",
        "sort",
        "focus",
        "activate",
        "capture",
        "click",
        "press",
        "execute",
    ] {
        assert!(
            !title_helper_body.contains(forbidden),
            "title fallback helper must stay diagnostic-only; found {forbidden}"
        );
    }

    let own_policy_body = extract_function_body(&module, "pub fn window_own_process_policy_v1(");
    for needle in [
        "if !is_current_process_window",
        "WindowOwnProcessPolicyStatus::ExcludedFromWindowsMenu",
        "WindowOwnProcessPolicyStatus::IncludedInWindowsMenu",
        "WindowOwnProcessPolicyStatus::Unknown",
        "source: \"nsWindow\"",
        "is_excluded_from_windows_menu",
    ] {
        assert!(
            own_policy_body.contains(needle),
            "own-process policy helper must pin {needle}"
        );
    }
    for forbidden in [
        "CoreGraphics",
        "CGWindowListCopyWindowInfo",
        "NSWorkspace",
        "AppKit",
        "objc::",
        "focus",
        "activate",
        "launch",
        "quit",
        "hide",
        "move",
        "resize",
        "click",
        "press",
        "execute",
        "capture",
    ] {
        assert!(
            !own_policy_body.contains(forbidden),
            "own-process policy helper must stay pure/read-only; found {forbidden}"
        );
    }

    for needle in [
        "let duplicate_groups = window_duplicate_groups_v1(",
        ".iter()",
        "WindowDuplicateObservationInputV1",
        "observation.duplicate_group = duplicate_group;",
        "Ok(windows)",
    ] {
        assert!(
            bridge.contains(needle),
            "CoreGraphics bridge must annotate duplicate groups without changing returned rows: {needle}"
        );
    }

    for needle in [
        "let title_fallbacks = window_title_fallbacks_v1(",
        "WindowTitleFallbackObservationInputV1",
        "title: window.title.clone()",
        "capture_candidate_status: observation.capture_candidate.status.clone()",
        ".map(|group| group.status.clone())",
        "observation.title_fallback = title_fallback;",
        "Ok(windows)",
    ] {
        assert!(
            bridge.contains(needle),
            "CoreGraphics bridge must annotate title fallback without changing returned rows: {needle}"
        );
    }
    assert!(
        bridge
            .find("observation.duplicate_group = duplicate_group;")
            .expect("duplicate group assignment")
            < bridge
                .find("let title_fallbacks = window_title_fallbacks_v1(")
                .expect("title fallback assignment"),
        "title fallback must be computed after duplicate-group annotation"
    );

    for needle in [
        "let is_current_process_window = u32::try_from(pid).ok() == Some(std::process::id());",
        "let own_process_window_policy = window_own_process_policy_v1(",
        "if is_current_process_window",
        "ns_window_is_excluded_from_windows_menu(native_window_id)",
        "observation.own_process_window_policy = own_process_window_policy;",
        "fn ns_window_is_excluded_from_windows_menu(native_window_id: u32) -> Option<bool>",
        "windowWithWindowNumber: window_number",
        "isExcludedFromWindowsMenu",
    ] {
        assert!(
            bridge.contains(needle),
            "CoreGraphics bridge must annotate own-process policy behind a current-process guard: {needle}"
        );
    }
    let ns_policy_helper_body =
        extract_function_body(&bridge, "fn ns_window_is_excluded_from_windows_menu(");
    for forbidden in [
        "focus",
        "activate",
        "launch",
        "quit",
        "hide",
        "move",
        "resize",
        "click",
        "press",
        "execute",
        "capture",
        "CGWindowListCreateImage",
        "ScreenCaptureKit",
    ] {
        assert!(
            !ns_policy_helper_body.contains(forbidden),
            "NSWindow policy helper must stay read-only; found {forbidden}"
        );
    }

    assert!(
        !mcp_tools.contains("COMPUTER_WINDOW_OBSERVATION_TOOL")
            && !mcp_tools.contains("handle_window_observation"),
        "window observation is an additive nested contract, not a new MCP action tool"
    );
    assert!(
        !runtime.contains("own_process_window_policy"),
        "own-process policy must not add a ComputerUseRuntimeBridge trait method"
    );
    assert!(
        mcp_tools.contains("bundleIdChanged for pid"),
        "bundle-id stale ownership revalidation must remain intact"
    );

    for needle in [
        "ComputerUseWindowObservationV1",
        "captureCandidate",
        "metadataQuality",
        "duplicateGroup",
        "preferred",
        "duplicate",
        "onScreenThenLargestAreaThenLowestZOrder",
        "titleFallback",
        "nonEmptyTitle",
        "emptyTitleSoleCandidate",
        "emptyTitleAmongMultipleCandidates",
        "preferNonEmptyTitleThenAllowEmptyOnlyIfSoleCandidate",
        "ownProcessWindowPolicy",
        "isExcludedFromWindowsMenu",
        "includedInWindowsMenu",
        "excludedFromWindowsMenu",
        "layerNonZero",
        "alphaTooLow",
        "sharingStateNone",
        "notOnScreen",
        "tooSmall",
        "metadataIncomplete",
        "diagnostic only",
    ] {
        assert!(
            protocol.contains(needle),
            "protocol docs must describe window observation metadata: {needle}"
        );
    }
}

fn extract_function_body<'a>(source: &'a str, signature: &str) -> &'a str {
    let start = source.find(signature).expect("signature");
    let open = source[start..].find('{').expect("open brace") + start;
    extract_block_from_open_brace(source, open)
}

fn extract_block_from_open_brace(source: &str, open: usize) -> &str {
    let mut depth = 0usize;

    for (offset, ch) in source[open..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return &source[open..=open + offset];
                }
            }
            _ => {}
        }
    }

    panic!("braced block did not close")
}
