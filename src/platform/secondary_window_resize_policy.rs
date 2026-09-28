#[cfg(target_os = "macos")]
pub fn set_window_resizable(window: &mut gpui::Window, resizable: bool) {
    use cocoa::base::id;
    use objc::{msg_send, sel, sel_impl};

    const NS_WINDOW_STYLE_MASK_RESIZABLE: u64 = 1 << 3;

    let Ok(window_handle) = raw_window_handle::HasWindowHandle::window_handle(window) else {
        return;
    };
    let raw_window_handle::RawWindowHandle::AppKit(appkit) = window_handle.as_raw() else {
        return;
    };
    let ns_view = appkit.ns_view.as_ptr() as id;
    unsafe {
        let ns_window: id = msg_send![ns_view, window];
        if ns_window.is_null() {
            return;
        }
        let current_style_mask: u64 = msg_send![ns_window, styleMask];
        let next_style_mask = if resizable {
            current_style_mask | NS_WINDOW_STYLE_MASK_RESIZABLE
        } else {
            current_style_mask & !NS_WINDOW_STYLE_MASK_RESIZABLE
        };
        if next_style_mask != current_style_mask {
            let _: () = msg_send![ns_window, setStyleMask: next_style_mask];
            if Some(ns_window) == crate::window_manager::get_main_window() {
                for button_type in 0..=2 {
                    let button: id = msg_send![ns_window, standardWindowButton: button_type];
                    if !button.is_null() {
                        let _: () = msg_send![button, setHidden: true];
                    }
                }
            }
        }
    }
}

#[cfg(not(target_os = "macos"))]
pub fn set_window_resizable(_window: &mut gpui::Window, _resizable: bool) {}
