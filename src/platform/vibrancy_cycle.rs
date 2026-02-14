/// Cycle through ALL vibrancy options - material, appearance, blending mode, emphasized.
/// Press Cmd+Shift+M repeatedly to find what works.
/// Returns a description of the current configuration.
#[cfg(target_os = "macos")]
pub fn cycle_vibrancy_material() -> String {
    use std::sync::atomic::Ordering;

    if require_main_thread("cycle_vibrancy_material") {
        return "ERROR: Not on main thread".to_string();
    }

    let materials = ns_visual_effect_material::ALL_MATERIALS;
    let appearances = APPEARANCE_OPTIONS;

    // Get current indices
    let mat_idx = CURRENT_MATERIAL_INDEX.load(Ordering::SeqCst);
    let app_idx = CURRENT_APPEARANCE_INDEX.load(Ordering::SeqCst);
    let blend_mode = CURRENT_BLENDING_MODE.load(Ordering::SeqCst);

    // Increment material, wrap around and bump appearance when materials exhausted
    let new_mat_idx = (mat_idx + 1) % materials.len();
    CURRENT_MATERIAL_INDEX.store(new_mat_idx, Ordering::SeqCst);

    // When materials wrap, cycle appearance
    if new_mat_idx == 0 {
        let new_app_idx = (app_idx + 1) % appearances.len();
        CURRENT_APPEARANCE_INDEX.store(new_app_idx, Ordering::SeqCst);

        // When appearances wrap, toggle blending mode
        if new_app_idx == 0 {
            CURRENT_BLENDING_MODE.store(blend_mode ^ 1, Ordering::SeqCst);
        }
    }

    // Get current values after update
    let mat_idx = CURRENT_MATERIAL_INDEX.load(Ordering::SeqCst);
    let app_idx = CURRENT_APPEARANCE_INDEX.load(Ordering::SeqCst);
    let blend_mode = CURRENT_BLENDING_MODE.load(Ordering::SeqCst);

    let (material_value, material_name) = materials[mat_idx];
    let appearance_name = appearances[app_idx];
    let blend_name = if blend_mode == 0 { "Behind" } else { "Within" };

    // SAFETY: Main thread verified. Window from WindowManager is valid.
    // NSAppearance and NSVisualEffectView methods are standard AppKit APIs.
    // Appearance and content_view pointers are nil-checked before use.
    unsafe {
        let window = match window_manager::get_main_window() {
            Some(w) => w,
            None => return "ERROR: No main window".to_string(),
        };

        // Set window appearance
        if appearance_name != "None" {
            let appearance_id: id = match appearance_name {
                "DarkAqua" => NSAppearanceNameDarkAqua,
                "VibrantDark" => NSAppearanceNameVibrantDark,
                "Aqua" => NSAppearanceNameAqua,
                "VibrantLight" => NSAppearanceNameVibrantLight,
                _ => nil,
            };
            if !appearance_id.is_null() {
                let appearance: id =
                    msg_send![class!(NSAppearance), appearanceNamed: appearance_id];
                if !appearance.is_null() {
                    let _: () = msg_send![window, setAppearance: appearance];
                }
            }
        } else {
            // Clear appearance - use system default
            let _: () = msg_send![window, setAppearance: nil];
        }

        let content_view: id = msg_send![window, contentView];
        if content_view.is_null() {
            return "ERROR: No content view".to_string();
        }

        let subviews: id = msg_send![content_view, subviews];
        if subviews.is_null() {
            return "ERROR: No subviews".to_string();
        }

        let count: usize = msg_send![subviews, count];
        for i in 0..count {
            let subview: id = msg_send![subviews, objectAtIndex: i];
            let is_visual_effect_view: bool =
                msg_send![subview, isKindOfClass: class!(NSVisualEffectView)];

            if is_visual_effect_view {
                // Set material
                let _: () = msg_send![subview, setMaterial: material_value];

                // Set blending mode
                let _: () = msg_send![subview, setBlendingMode: blend_mode as isize];

                // Always active
                let _: () = msg_send![subview, setState: 1isize];

                // Toggle emphasized based on material index (try both)
                let emphasized = mat_idx.is_multiple_of(2);
                let _: () = msg_send![subview, setEmphasized: emphasized];

                // Force redraw
                let _: () = msg_send![subview, setNeedsDisplay: true];
                let _: () = msg_send![window, display];

                let msg = format!(
                    "{} | {} | {} | {}",
                    material_name,
                    appearance_name,
                    blend_name,
                    if emphasized { "Emph" } else { "NoEmph" }
                );
                logging::log("VIBRANCY", &msg);
                return msg;
            }
        }
    }

    "ERROR: No NSVisualEffectView found".to_string()
}

#[cfg(not(target_os = "macos"))]
pub fn cycle_vibrancy_material() -> String {
    "Vibrancy cycling not supported on this platform".to_string()
}

/// Get the current material name for display in the UI
#[cfg(target_os = "macos")]
pub fn get_current_material_name() -> String {
    use std::sync::atomic::Ordering;

    let materials = ns_visual_effect_material::ALL_MATERIALS;
    let mat_idx = CURRENT_MATERIAL_INDEX.load(Ordering::SeqCst);

    if mat_idx < materials.len() {
        // Extract just the material name without the number
        let full_name = materials[mat_idx].1;
        // Format: "Popover (6)" -> "Popover"
        full_name
            .split(" (")
            .next()
            .unwrap_or(full_name)
            .to_string()
    } else {
        "Unknown".to_string()
    }
}

#[cfg(not(target_os = "macos"))]
pub fn get_current_material_name() -> String {
    "N/A".to_string()
}

/// Get the current appearance name for display in the UI
#[cfg(target_os = "macos")]
pub fn get_current_appearance_name() -> String {
    use std::sync::atomic::Ordering;

    let app_idx = CURRENT_APPEARANCE_INDEX.load(Ordering::SeqCst);
    if app_idx < APPEARANCE_OPTIONS.len() {
        APPEARANCE_OPTIONS[app_idx].to_string()
    } else {
        "Unknown".to_string()
    }
}

#[cfg(not(target_os = "macos"))]
pub fn get_current_appearance_name() -> String {
    "N/A".to_string()
}

/// Cycle through appearance options only (VibrantLight, VibrantDark, Aqua, DarkAqua, None)
/// Press Cmd+L to cycle through appearances without changing the material.
/// Returns a description of the current appearance.
#[cfg(target_os = "macos")]
pub fn cycle_appearance() -> String {
    use std::sync::atomic::Ordering;

    if require_main_thread("cycle_appearance") {
        return "ERROR: Not on main thread".to_string();
    }

    let appearances = APPEARANCE_OPTIONS;

    // Get current index and increment
    let app_idx = CURRENT_APPEARANCE_INDEX.load(Ordering::SeqCst);
    let new_app_idx = (app_idx + 1) % appearances.len();
    CURRENT_APPEARANCE_INDEX.store(new_app_idx, Ordering::SeqCst);

    let appearance_name = appearances[new_app_idx];

    // SAFETY: Main thread verified. Window from WindowManager is valid.
    // NSAppearance pointers are nil-checked. display is a standard NSWindow method.
    unsafe {
        let window = match window_manager::get_main_window() {
            Some(w) => w,
            None => return "ERROR: No main window".to_string(),
        };

        // Set window appearance
        if appearance_name != "None" {
            let appearance_id: id = match appearance_name {
                "DarkAqua" => NSAppearanceNameDarkAqua,
                "VibrantDark" => NSAppearanceNameVibrantDark,
                "Aqua" => NSAppearanceNameAqua,
                "VibrantLight" => NSAppearanceNameVibrantLight,
                _ => nil,
            };
            if !appearance_id.is_null() {
                let appearance: id =
                    msg_send![class!(NSAppearance), appearanceNamed: appearance_id];
                if !appearance.is_null() {
                    let _: () = msg_send![window, setAppearance: appearance];
                }
            }
        } else {
            // Clear appearance - use system default
            let _: () = msg_send![window, setAppearance: nil];
        }

        // Force window refresh
        let _: () = msg_send![window, display];

        let msg = format!("Appearance: {}", appearance_name);
        logging::log("VIBRANCY", &msg);
        msg
    }
}

#[cfg(not(target_os = "macos"))]
pub fn cycle_appearance() -> String {
    "Appearance cycling not supported on this platform".to_string()
}
