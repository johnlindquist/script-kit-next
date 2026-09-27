use std::fs;

const PROOF_MATRIX: &str = include_str!("../scripts/devtools/liquid-glass-proof.ts");
const FILE_SEARCH_MINI_LAYOUT_RECEIPT: &str = r#"
{
  "surfaceKind": "FileSearchMini",
  "contentGlassNodes": [],
  "contentNativeMaterialNodes": [],
  "glassLayerViolations": [],
  "windowBackdrop": 1
}
"#;

fn read_source(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
}

#[test]
fn tahoe_liquid_glass_is_gated_and_uses_shared_theme_tint() {
    let platform = read_source("src/platform/secondary_window_config.rs");
    let main_vibrancy = read_source("src/platform/vibrancy_config.rs");
    let ui_foundation = read_source("src/ui_foundation/mod.rs");
    let terminal_command_bar = read_source("src/terminal/command_bar_ui/render.rs");
    let file_search = read_source("src/file_search/mod.rs");

    assert!(
        platform.contains("NSClassFromString")
            && platform.contains("c\"NSGlassEffectView\".as_ptr()")
            && platform.contains("class availability is the capability gate"),
        "Liquid Glass must only be enabled when the macOS 26 NSGlassEffectView API is present"
    );
    assert!(
        platform.contains("unsafe fn liquid_glass_tint_color()")
            && platform.contains("crate::ui_foundation::main_window_matched_background_rgba(&theme)"),
        "Tahoe glass capability code must preserve the shared theme tint helper for future contentView-backed glass elements"
    );
    assert!(
        ui_foundation.contains("pub fn main_window_matched_background_rgba(theme: &Theme) -> u32")
            && ui_foundation
                .contains("pub fn main_window_matched_background(theme: &Theme) -> Rgba")
            && ui_foundation.contains("main_window_matched_background_rgba(theme)"),
        "GPUI and native liquid-glass backgrounds must share one theme-derived tint helper"
    );
    assert!(
        main_vibrancy
            .contains("configure_tahoe_window_backdrop(window, \"PANEL\", \"Main window\")"),
        "Main-window vibrancy must use the semantic Tahoe window-backdrop hook"
    );
    assert!(
        platform.contains("configure_tahoe_window_backdrop(window, \"APPEARANCE\", &title_string)")
            && platform.contains("should_refresh_secondary_window_appearance(&title_string)"),
        "Theme/appearance refresh must revisit existing secondary native backdrops"
    );
    assert!(
        main_vibrancy
            .contains("crate::ui_foundation::main_window_matched_background_rgba(&theme)")
            && main_vibrancy.contains("background_tint,")
            && main_vibrancy.contains("glass_bits,"),
        "Main-window Liquid Glass refresh de-dupe must include the shared theme tint, not just dark/material state"
    );
    assert!(
        terminal_command_bar.contains("AppChromeColors::from_theme(&self.theme).popup_surface_rgba")
            && !terminal_command_bar.contains("let opacity = if self.theme.is_vibrancy_enabled()"),
        "Terminal command bar must share AppChromeColors popup_surface_rgba instead of hardcoded 0.50/0.95 alpha"
    );
    assert!(
        file_search.contains("AppChromeColors::from_theme(&theme)")
            && file_search.contains("chrome.popup_surface_rgba")
            && !file_search.contains(".bg(rgb(theme.colors.background.title_bar))"),
        "File-search drag preview must use shared popup chrome tokens instead of painting title_bar directly"
    );
    for forbidden in [
        "unsafe fn configure_tahoe_liquid_glass_background",
        "find_existing_liquid_glass_background",
        "Tahoe Liquid Glass background configured",
        "fitted to GPUI content without participating in app layout",
        "addSubview: view positioned: -1isize relativeTo: cocoa::base::nil",
    ] {
        assert!(
            !platform.contains(forbidden),
            "NSGlassEffectView must not be installed or named as a full-window background sibling; forbidden marker: {forbidden}"
        );
    }
    assert!(
        !main_vibrancy
            .contains("configure_tahoe_liquid_glass_background(window, \"PANEL\", \"Main window\")"),
        "Main-window vibrancy must not install NSGlassEffectView as a full-window content background"
    );
}

#[test]
fn root_window_layout_material_is_backdrop_not_content_glass() {
    let layout = read_source("src/app_layout/build_layout_info.rs");
    let tokens = read_source("src/ui/chrome/tokens.rs");
    let devtools_layout = read_source("scripts/devtools/layout.ts");

    assert!(
        tokens.contains("CHROME_LAYER_WINDOW_BACKDROP")
            && tokens.contains("MATERIAL_NATIVE_WINDOW_BACKDROP"),
        "Liquid Glass layout receipts need explicit window-backdrop vocabulary so backdrop material is not reported as content"
    );

    let root_window = layout
        .split("LayoutComponentInfo::new(\"Window\"")
        .nth(1)
        .and_then(|tail| tail.split(");").next())
        .expect("build_layout_info must expose the root Window layout component");
    assert!(
        root_window.contains("CHROME_LAYER_WINDOW_BACKDROP")
            && root_window.contains("MATERIAL_NATIVE_WINDOW_BACKDROP")
            && root_window.contains("window.backdrop"),
        "Root Window must be classified as a system/window backdrop layer, not content glass"
    );
    assert!(
        !root_window.contains("CHROME_LAYER_CONTENT")
            && !root_window.contains("MATERIAL_NS_VISUAL_EFFECT"),
        "Root Window layout metadata must not report CHROME_LAYER_CONTENT + NS visual effect material"
    );
    assert!(
        devtools_layout.contains("glassLayerViolations")
            && devtools_layout.contains("contentNativeMaterialNodes")
            && devtools_layout.contains("NSVisualEffectView")
            && devtools_layout.contains("NSGlassEffectView")
            && devtools_layout.contains("nativeWindowBackdrop"),
        "layout.ts must fail content-layer native/AppKit material instead of silently treating it as content"
    );
    let proof = read_source("scripts/devtools/liquid-glass-proof.ts");
    assert!(
        proof.contains("glassLayerViolations")
            && proof.contains("contentNativeMaterialNodes")
            && proof.contains("glassLayerViolations === 0"),
        "Liquid Glass proof classification must fail layouts with content-layer AppKit glass/material violations"
    );
}

#[test]
fn footer_native_host_uses_theme_vibrancy_and_reports_installed_surface() {
    let footer = read_source("src/footer_popup.rs");

    for needle in [
        "MainWindowFooterHostSnapshot",
        "native_host_installed",
        "active_main_window_footer_surface",
        "ensure_main_window_footer_host(gpui_view, ns_window)",
        "refresh_main_footer_host(ns_window, config)",
        "update_main_window_footer_host_state(",
        "installed.then_some(config.surface)",
        "AppChromeColors::from_theme(theme)",
        "theme.get_vibrancy().material",
        "setAppearance:",
        "setMaterial: material",
        "setState: 1isize",
        "setBlendingMode: 1isize",
        "setEmphasized: is_dark",
    ] {
        assert!(
            footer.contains(needle),
            "native main-window footer host must preserve theme/vibrancy/state marker: {needle}"
        );
    }
}

#[test]
fn devtools_targets_match_agent_chat_by_semantic_surface_alias() {
    let targets = read_source("scripts/devtools/lib/target-identity.ts");
    assert!(
        targets.contains("[\"snapshot.semanticSurface\", snapshot.semanticSurface]")
            && targets.contains("[\"listedWindow.semanticSurface\", listedWindow.semanticSurface]"),
        "targets.ts must consider runtime semanticSurface candidates for strict surface matching"
    );
    assert!(
        targets.contains("function acceptedSurfaceValues")
            && targets.contains("expectedSurfaceKind === \"AgentChat\"")
            && targets.contains("values.add(\"agentChatChat\")"),
        "AgentChat strict target matching must accept the agentChatChat automation semantic surface"
    );
    assert!(
        targets.contains("acceptedValues.has(candidate.value)"),
        "surfaceMatch must match against accepted surface aliases, not only exact surfaceKind strings"
    );
    assert!(
        targets.contains("matchedCandidate")
            && targets.contains("acceptedValues: [...acceptedValues]"),
        "surfaceMatch receipts must explain which semantic alias matched"
    );
}

#[test]
fn file_search_mini_layout_receipt_uses_window_backdrop() {
    let receipt = FILE_SEARCH_MINI_LAYOUT_RECEIPT;

    assert!(
        receipt.contains("\"surfaceKind\": \"FileSearchMini\""),
        "test must pin the FileSearchMini layout receipt, not a neighboring file-search surface"
    );
    assert!(
        receipt.contains("\"contentGlassNodes\": []"),
        "FileSearchMini root Window must not be counted as content glass"
    );
    assert!(
        receipt.contains("\"contentNativeMaterialNodes\": []"),
        "FileSearchMini root Window must not be counted as content-owned AppKit/native material"
    );
    assert!(
        receipt.contains("\"glassLayerViolations\": []"),
        "FileSearchMini must have no content-layer glass/material violations"
    );
    assert!(
        receipt.contains("\"windowBackdrop\": 1"),
        "FileSearchMini must classify exactly one root window backdrop layer"
    );
    assert!(
        !receipt.contains(
            "\"chromeLayer\": \"content\",\n        \"materialSource\": \"NSVisualEffectView\""
        ),
        "FileSearchMini root Window must not regress to content + NSVisualEffectView"
    );
}

#[test]
fn proof_matrix_filters_image_diff_receipts_by_receipt_assertions() {
    let image_diff = read_source("scripts/devtools/image-diff.ts");
    assert!(
        PROOF_MATRIX.contains("function imageDiffUsability"),
        "Liquid Glass matrix must validate image-diff receipts before counting them"
    );
    for needle in [
        "classification",
        "diffMaskWritten",
        "changedPixelsMeasured",
        "sameSizeRequired",
        "dimensions.sameSize",
        "surfaceKind === \"ConfirmPrompt\"",
        "window-priority-confirm-layout-after.json",
        "window-priority-confirm-screenshot-after.json",
        "ignored ConfirmPrompt screenshot evidence",
        "appRenderReadbackBlocked",
        "appRenderBlockedSurfaceCount",
        "numeric-proof-app-render-blocked",
        "osScreenshotBlockers",
        "osScreenshotBlockerCounts",
        "osCapture",
        "osCaptureBlockerCode",
        "macos-windowserver-capture-blocked",
        "screen-rect-capture-blocked",
        "layoutReceiptFreshnessLimitationSurfaceCount",
        "staleLayoutEvidenceSurfaceCount",
        "legacy layout receipt lacks explicit cornerRadiusTokens",
        "GPUI render readback was unavailable or unsupported",
    ] {
        assert!(
            PROOF_MATRIX.contains(needle),
            "Liquid Glass proof matrix must inspect `{needle}`"
        );
    }
    for needle in [
        "--require-os-evidence",
        "--red-receipt",
        "--green-receipt",
        "red-os-evidence-missing",
        "green-os-evidence-missing",
        "countsAsCompositorEvidence",
        "captureIdentity",
    ] {
        assert!(
            image_diff.contains(needle),
            "image-diff must inspect OS compositor evidence marker `{needle}`"
        );
    }
}
