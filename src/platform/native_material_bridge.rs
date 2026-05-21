// Tahoe's NSGlassEffectView is only available through newer macOS SDK/runtime
// combinations. Keep this bridge runtime-gated so older SDK builds still work,
// and let existing NSVisualEffectView hosts use the legacy material fallback
// until each native host can be safely created as a glass-backed view.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NativeMaterialKind {
    TahoeGlassRegular,
    LegacyVibrancy,
    OpaqueFallback,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct NativeMaterialSelection {
    pub kind: NativeMaterialKind,
    pub requested_glass: bool,
    pub glass_available: bool,
}

impl NativeMaterialSelection {
    pub fn label(self) -> &'static str {
        match self.kind {
            NativeMaterialKind::TahoeGlassRegular => "Tahoe Glass",
            NativeMaterialKind::LegacyVibrancy => "Legacy Vibrancy",
            NativeMaterialKind::OpaqueFallback => "Opaque",
        }
    }
}

pub(crate) fn native_material_selection(
    vibrancy_enabled: bool,
    accessibility: SystemAppearanceAccessibility,
) -> NativeMaterialSelection {
    match native_material_policy(vibrancy_enabled, accessibility) {
        NativeMaterialPolicy::OpaqueFallback => NativeMaterialSelection {
            kind: NativeMaterialKind::OpaqueFallback,
            requested_glass: false,
            glass_available: false,
        },
        NativeMaterialPolicy::LegacyVibrancy => NativeMaterialSelection {
            kind: NativeMaterialKind::LegacyVibrancy,
            requested_glass: false,
            glass_available: false,
        },
        NativeMaterialPolicy::GlassEligible => {
            let glass_available = tahoe_glass_effect_view_available();
            NativeMaterialSelection {
                kind: if glass_available {
                    NativeMaterialKind::TahoeGlassRegular
                } else {
                    NativeMaterialKind::LegacyVibrancy
                },
                requested_glass: true,
                glass_available,
            }
        }
    }
}

#[cfg(target_os = "macos")]
pub(crate) fn tahoe_glass_effect_view_available() -> bool {
    objc::runtime::Class::get("NSGlassEffectView").is_some()
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn tahoe_glass_effect_view_available() -> bool {
    false
}

#[cfg(target_os = "macos")]
pub(crate) unsafe fn configure_native_material_view(
    view: cocoa::base::id,
    selection: NativeMaterialSelection,
    is_dark: bool,
    material: crate::theme::VibrancyMaterial,
    log_target: &str,
    view_name: &str,
) -> bool {
    use cocoa::base::nil;
    use objc::{msg_send, sel, sel_impl};

    if view == nil {
        return false;
    }

    match selection.kind {
        NativeMaterialKind::OpaqueFallback => {
            let layer: cocoa::base::id = msg_send![view, layer];
            if layer != nil {
                let _: () = msg_send![layer, setMasksToBounds: true];
            }
            crate::logging::log(
                log_target,
                &format!("{view_name}: native material resolved to opaque accessibility fallback"),
            );
            false
        }
        NativeMaterialKind::TahoeGlassRegular => {
            let is_glass = objc::runtime::Class::get("NSGlassEffectView")
                .map(|cls| {
                    let cls_ptr = cls as *const objc::runtime::Class;
                    let matches: bool = msg_send![view, isKindOfClass: cls_ptr];
                    matches
                })
                .unwrap_or(false);
            if is_glass {
                let _: () = msg_send![view, setWantsLayer: true];
                crate::logging::log(
                    log_target,
                    &format!("{view_name}: configured Tahoe NSGlassEffectView regular material"),
                );
                true
            } else {
                configure_legacy_visual_effect_view(view, is_dark, material);
                crate::logging::log(
                    log_target,
                    &format!(
                        "{view_name}: Tahoe glass available, using legacy view fallback until host can be glass-backed"
                    ),
                );
                false
            }
        }
        NativeMaterialKind::LegacyVibrancy => {
            configure_legacy_visual_effect_view(view, is_dark, material);
            crate::logging::log(
                log_target,
                &format!("{view_name}: configured legacy NSVisualEffectView material"),
            );
            false
        }
    }
}

#[cfg(target_os = "macos")]
unsafe fn configure_legacy_visual_effect_view(
    view: cocoa::base::id,
    is_dark: bool,
    material: crate::theme::VibrancyMaterial,
) {
    use cocoa::base::nil;
    use objc::{class, msg_send, sel, sel_impl};

    let is_visual_effect_view: bool = msg_send![view, isKindOfClass: class!(NSVisualEffectView)];
    if !is_visual_effect_view {
        return;
    }

    let appearance_name = if is_dark {
        c"NSAppearanceNameVibrantDark".as_ptr()
    } else {
        c"NSAppearanceNameVibrantLight".as_ptr()
    };
    let appearance_name: cocoa::base::id =
        msg_send![class!(NSString), stringWithUTF8String: appearance_name];
    if appearance_name != nil {
        let appearance: cocoa::base::id =
            msg_send![class!(NSAppearance), appearanceNamed: appearance_name];
        if appearance != nil {
            let _: () = msg_send![view, setAppearance: appearance];
        }
    }

    let material_value = match material {
        crate::theme::VibrancyMaterial::Hud => ns_visual_effect_material::HUD_WINDOW,
        crate::theme::VibrancyMaterial::Popover => ns_visual_effect_material::POPOVER,
        crate::theme::VibrancyMaterial::Menu => ns_visual_effect_material::MENU,
        crate::theme::VibrancyMaterial::Sidebar => ns_visual_effect_material::SIDEBAR,
        crate::theme::VibrancyMaterial::Content => ns_visual_effect_material::CONTENT_BACKGROUND,
    };
    let _: () = msg_send![view, setMaterial: material_value];
    let _: () = msg_send![view, setState: if is_dark { 1isize } else { 0isize }];
    let _: () = msg_send![view, setBlendingMode: 0isize];
    let _: () = msg_send![view, setEmphasized: is_dark];
}

#[cfg(test)]
mod native_material_bridge_tests {
    use super::{native_material_selection, NativeMaterialKind};
    use crate::platform::SystemAppearanceAccessibility;

    #[test]
    fn reduced_transparency_selects_opaque_fallback() {
        let selection = native_material_selection(
            true,
            SystemAppearanceAccessibility {
                reduce_transparency: true,
                increase_contrast: false,
                reduce_motion: false,
            },
        );

        assert_eq!(selection.kind, NativeMaterialKind::OpaqueFallback);
    }

    #[test]
    fn disabled_vibrancy_selects_legacy_non_glass_path() {
        let selection = native_material_selection(false, SystemAppearanceAccessibility::default());

        assert_eq!(selection.kind, NativeMaterialKind::LegacyVibrancy);
        assert!(!selection.requested_glass);
    }
}
