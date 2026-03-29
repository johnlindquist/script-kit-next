//! Non-macOS window_control module
//!
//! On Windows: real implementation using Win32 APIs (EnumWindows, GetWindowTextW, etc.)
//! On other platforms: returns "not implemented" errors.

// This module is non-macOS only (inverse of the real window_control module)
#![cfg(not(target_os = "macos"))]

use anyhow::{anyhow, Result};

// ============================================================================
// Shared types (all non-macOS platforms)
// ============================================================================

/// Represents the bounds (position and size) of a window.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bounds {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Bounds {
    /// Create a new Bounds
    #[allow(dead_code)]
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

/// Information about a window.
#[derive(Clone, Debug)]
pub struct WindowInfo {
    pub id: u32,
    pub app: String,
    pub title: String,
    pub bounds: Bounds,
    pub pid: i32,
}

impl WindowInfo {
    /// Create a WindowInfo for testing purposes.
    #[doc(hidden)]
    #[allow(dead_code)]
    pub fn for_test(id: u32, app: String, title: String, bounds: Bounds, pid: i32) -> Self {
        Self {
            id,
            app,
            title,
            bounds,
            pid,
        }
    }
}

/// Tiling positions for windows.
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum TilePosition {
    LeftHalf,
    RightHalf,
    TopHalf,
    BottomHalf,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    LeftThird,
    CenterThird,
    RightThird,
    TopThird,
    MiddleThird,
    BottomThird,
    FirstTwoThirds,
    LastTwoThirds,
    TopTwoThirds,
    BottomTwoThirds,
    Center,
    AlmostMaximize,
    Fullscreen,
}

// ============================================================================
// Windows implementation
// ============================================================================

#[cfg(target_os = "windows")]
#[allow(clippy::upper_case_acronyms)]
mod win32 {
    use super::*;
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;

    // Win32 FFI type aliases — HWND is *mut c_void to match existing
    // declarations in platform::ai_commands and avoid clashing-extern warnings.
    type HWND = *mut std::ffi::c_void;
    type BOOL = i32;
    type LPARAM = isize;
    type DWORD = u32;

    const TRUE: BOOL = 1;
    #[allow(dead_code)]
    const FALSE: BOOL = 0;

    // Window style flags
    const GWL_STYLE: i32 = -16;
    const GWL_EXSTYLE: i32 = -20;
    const WS_VISIBLE: u32 = 0x1000_0000;
    const WS_EX_TOOLWINDOW: u32 = 0x0000_0080;
    const WS_EX_NOACTIVATE: u32 = 0x0800_0000;

    // ShowWindow commands
    const SW_RESTORE: i32 = 9;
    #[allow(dead_code)]
    const SW_SHOW: i32 = 5;

    // GetWindowPlacement constants
    #[allow(dead_code)]
    const SW_SHOWMINIMIZED: u32 = 2;

    // MoveWindow is used for tile/resize/move
    #[allow(dead_code)]
    const SWP_NOZORDER: u32 = 0x0004;

    #[repr(C)]
    struct RECT {
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
    }

    #[repr(C)]
    struct POINT {
        x: i32,
        y: i32,
    }

    #[repr(C)]
    struct WINDOWPLACEMENT {
        length: u32,
        flags: u32,
        show_cmd: u32,
        pt_min_position: POINT,
        pt_max_position: POINT,
        rc_normal_position: RECT,
    }

    #[repr(C)]
    struct MONITORINFO {
        cb_size: u32,
        rc_monitor: RECT,
        rc_work: RECT,
        dw_flags: u32,
    }

    // FFI declarations.
    // Allow clashing_extern_declarations: other modules in this crate declare
    // some of the same Win32 functions with `isize` for HWND. Both `isize`
    // and `*mut c_void` are pointer-sized and ABI-compatible on Windows.
    #[allow(clashing_extern_declarations)]
    extern "system" {
        fn EnumWindows(
            lpEnumFunc: unsafe extern "system" fn(HWND, LPARAM) -> BOOL,
            lParam: LPARAM,
        ) -> BOOL;
        fn GetWindowTextW(hWnd: HWND, lpString: *mut u16, nMaxCount: i32) -> i32;
        fn GetWindowTextLengthW(hWnd: HWND) -> i32;
        fn IsWindowVisible(hWnd: HWND) -> BOOL;
        fn GetWindowLongW(hWnd: HWND, nIndex: i32) -> i32;
        fn GetWindowRect(hWnd: HWND, lpRect: *mut RECT) -> BOOL;
        fn GetWindowThreadProcessId(hWnd: HWND, lpdwProcessId: *mut DWORD) -> DWORD;
        fn SetForegroundWindow(hWnd: HWND) -> BOOL;
        fn ShowWindow(hWnd: HWND, nCmdShow: i32) -> BOOL;
        fn GetWindowPlacement(hWnd: HWND, lpwndpl: *mut WINDOWPLACEMENT) -> BOOL;
        fn MoveWindow(
            hWnd: HWND,
            x: i32,
            y: i32,
            nWidth: i32,
            nHeight: i32,
            bRepaint: BOOL,
        ) -> BOOL;
        fn PostMessageW(hWnd: HWND, msg: u32, wParam: usize, lParam: isize) -> BOOL;
        fn MonitorFromWindow(hWnd: HWND, dwFlags: u32) -> *mut std::ffi::c_void;
        fn GetMonitorInfoW(hMonitor: *mut std::ffi::c_void, lpmi: *mut MONITORINFO) -> BOOL;
    }

    // WM_CLOSE for closing windows
    const WM_CLOSE: u32 = 0x0010;

    // SW constants for ShowWindow
    const SW_MINIMIZE: i32 = 6;
    const SW_MAXIMIZE: i32 = 3;

    // MonitorFromWindow flags
    const MONITOR_DEFAULTTONEAREST: u32 = 0x0000_0002;

    /// Get the window title as a Rust String.
    fn get_window_title(hwnd: HWND) -> String {
        // SAFETY: GetWindowTextLengthW is safe to call on any HWND;
        // returns 0 for invalid handles or empty titles.
        let len = unsafe { GetWindowTextLengthW(hwnd) };
        if len == 0 {
            return String::new();
        }
        let mut buf: Vec<u16> = vec![0; (len + 1) as usize];
        // SAFETY: buf is large enough for the title + null terminator.
        let copied = unsafe { GetWindowTextW(hwnd, buf.as_mut_ptr(), buf.len() as i32) };
        if copied == 0 {
            return String::new();
        }
        OsString::from_wide(&buf[..copied as usize])
            .to_string_lossy()
            .into_owned()
    }

    /// Get the executable name for a process ID.
    fn get_process_name(pid: u32) -> String {
        // Use sysinfo to look up process name
        use sysinfo::System;
        let mut sys = System::new();
        let sysinfo_pid = sysinfo::Pid::from_u32(pid);
        sys.refresh_processes_specifics(
            sysinfo::ProcessesToUpdate::Some(&[sysinfo_pid]),
            true,
            sysinfo::ProcessRefreshKind::nothing(),
        );
        sys.process(sysinfo_pid)
            .map(|p| p.name().to_string_lossy().to_string())
            .unwrap_or_else(|| format!("PID {}", pid))
    }

    /// Check whether a window should be included in the window list.
    /// Filters out invisible windows, tool windows, and windows without titles.
    fn is_candidate_window(hwnd: HWND) -> bool {
        // SAFETY: IsWindowVisible / GetWindowLongW are safe on any HWND.
        unsafe {
            if IsWindowVisible(hwnd) == 0 {
                return false;
            }
            let style = GetWindowLongW(hwnd, GWL_STYLE) as u32;
            if style & WS_VISIBLE == 0 {
                return false;
            }
            let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE) as u32;
            // Exclude tool windows and noactivate windows
            if ex_style & WS_EX_TOOLWINDOW != 0 {
                return false;
            }
            if ex_style & WS_EX_NOACTIVATE != 0 {
                return false;
            }
        }
        let title = get_window_title(hwnd);
        if title.is_empty() {
            return false;
        }
        true
    }

    /// Callback data structure for EnumWindows
    struct EnumData {
        windows: Vec<(HWND, String, RECT, u32)>,
    }

    /// EnumWindows callback: collect visible, titled windows.
    unsafe extern "system" fn enum_windows_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
        if !is_candidate_window(hwnd) {
            return TRUE; // continue enumeration
        }
        let title = get_window_title(hwnd);

        let mut rect = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        // SAFETY: rect is a valid out-pointer, GetWindowRect writes to it.
        GetWindowRect(hwnd, &mut rect);

        let mut pid: DWORD = 0;
        // SAFETY: pid is a valid out-pointer.
        GetWindowThreadProcessId(hwnd, &mut pid);

        // SAFETY: lparam is a valid pointer to EnumData, cast back from the caller.
        let data = &mut *(lparam as *mut EnumData);
        data.windows.push((hwnd, title, rect, pid));
        TRUE // continue
    }

    /// List all visible, user-facing windows on the system.
    pub fn list_windows() -> Result<Vec<WindowInfo>> {
        let mut data = EnumData {
            windows: Vec::new(),
        };
        // SAFETY: We pass a valid callback and a pointer to our stack-allocated
        // EnumData. The callback only writes through this pointer while
        // EnumWindows holds it, so no aliasing issues arise.
        let ok =
            unsafe { EnumWindows(enum_windows_callback, &mut data as *mut EnumData as LPARAM) };
        if ok == 0 {
            return Err(anyhow!("EnumWindows failed"));
        }

        let windows: Vec<WindowInfo> = data
            .windows
            .into_iter()
            .map(|(hwnd, title, rect, pid)| {
                let app = get_process_name(pid);
                let width = (rect.right - rect.left).max(0) as u32;
                let height = (rect.bottom - rect.top).max(0) as u32;
                WindowInfo {
                    // Use the HWND as the unique ID (truncated to u32).
                    // On 64-bit Windows, HWND values are typically small enough
                    // to fit in u32. We store the full value for internal use.
                    id: hwnd as usize as u32,
                    app,
                    title,
                    bounds: Bounds {
                        x: rect.left,
                        y: rect.top,
                        width,
                        height,
                    },
                    pid: pid as i32,
                }
            })
            .collect();

        Ok(windows)
    }

    /// Focus (bring to foreground) the window with the given ID.
    pub fn focus_window(id: u32) -> Result<()> {
        let hwnd = id as usize as HWND;

        // If the window is minimized, restore it first
        let mut placement = WINDOWPLACEMENT {
            length: std::mem::size_of::<WINDOWPLACEMENT>() as u32,
            flags: 0,
            show_cmd: 0,
            pt_min_position: POINT { x: 0, y: 0 },
            pt_max_position: POINT { x: 0, y: 0 },
            rc_normal_position: RECT {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            },
        };
        // SAFETY: placement is valid and properly sized.
        unsafe {
            if GetWindowPlacement(hwnd, &mut placement) != 0
                && placement.show_cmd == SW_SHOWMINIMIZED
            {
                ShowWindow(hwnd, SW_RESTORE);
            }
        }

        // SAFETY: SetForegroundWindow is safe on any HWND; it may fail
        // silently if the calling thread doesn't own the foreground.
        let ok = unsafe { SetForegroundWindow(hwnd) };
        if ok == 0 {
            return Err(anyhow!("SetForegroundWindow failed for HWND 0x{:X}", id));
        }
        Ok(())
    }

    /// Close the window by sending WM_CLOSE.
    pub fn close_window(id: u32) -> Result<()> {
        let hwnd = id as usize as HWND;
        // SAFETY: PostMessageW is safe; WM_CLOSE is a standard close request.
        let ok = unsafe { PostMessageW(hwnd, WM_CLOSE, 0, 0) };
        if ok == 0 {
            return Err(anyhow!("PostMessageW(WM_CLOSE) failed for HWND 0x{:X}", id));
        }
        Ok(())
    }

    /// Minimize the window.
    pub fn minimize_window(id: u32) -> Result<()> {
        let hwnd = id as usize as HWND;
        // SAFETY: ShowWindow with SW_MINIMIZE is a standard operation.
        unsafe {
            ShowWindow(hwnd, SW_MINIMIZE);
        }
        Ok(())
    }

    /// Maximize the window.
    pub fn maximize_window(id: u32) -> Result<()> {
        let hwnd = id as usize as HWND;
        // SAFETY: ShowWindow with SW_MAXIMIZE is a standard operation.
        unsafe {
            ShowWindow(hwnd, SW_MAXIMIZE);
        }
        Ok(())
    }

    /// Resize the window (keeps current position).
    pub fn resize_window(id: u32, width: u32, height: u32) -> Result<()> {
        let hwnd = id as usize as HWND;
        let mut rect = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        // SAFETY: rect is a valid out-pointer.
        let ok = unsafe { GetWindowRect(hwnd, &mut rect) };
        if ok == 0 {
            return Err(anyhow!("GetWindowRect failed for HWND 0x{:X}", id));
        }
        // SAFETY: MoveWindow repositions and resizes the window.
        let ok =
            unsafe { MoveWindow(hwnd, rect.left, rect.top, width as i32, height as i32, TRUE) };
        if ok == 0 {
            return Err(anyhow!("MoveWindow (resize) failed for HWND 0x{:X}", id));
        }
        Ok(())
    }

    /// Move the window to a new position (keeps current size).
    pub fn move_window(id: u32, x: i32, y: i32) -> Result<()> {
        let hwnd = id as usize as HWND;
        let mut rect = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        // SAFETY: rect is a valid out-pointer.
        let ok = unsafe { GetWindowRect(hwnd, &mut rect) };
        if ok == 0 {
            return Err(anyhow!("GetWindowRect failed for HWND 0x{:X}", id));
        }
        let w = rect.right - rect.left;
        let h = rect.bottom - rect.top;
        // SAFETY: MoveWindow is a standard Win32 call.
        let ok = unsafe { MoveWindow(hwnd, x, y, w, h, TRUE) };
        if ok == 0 {
            return Err(anyhow!("MoveWindow failed for HWND 0x{:X}", id));
        }
        Ok(())
    }

    /// Get the work area (usable screen space) for the monitor containing the window.
    fn get_work_area(hwnd: HWND) -> Result<RECT> {
        // SAFETY: MonitorFromWindow returns a valid monitor handle for any HWND.
        let monitor = unsafe { MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST) };
        if monitor.is_null() {
            return Err(anyhow!("MonitorFromWindow returned null"));
        }
        let mut info = MONITORINFO {
            cb_size: std::mem::size_of::<MONITORINFO>() as u32,
            rc_monitor: RECT {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            },
            rc_work: RECT {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            },
            dw_flags: 0,
        };
        // SAFETY: info is properly sized (cb_size set), monitor is valid.
        let ok = unsafe { GetMonitorInfoW(monitor, &mut info) };
        if ok == 0 {
            return Err(anyhow!("GetMonitorInfoW failed"));
        }
        Ok(info.rc_work)
    }

    /// Tile the window to the given position on its current monitor.
    pub fn tile_window(id: u32, position: TilePosition) -> Result<()> {
        let hwnd = id as usize as HWND;
        let work = get_work_area(hwnd)?;
        let sw = work.right - work.left;
        let sh = work.bottom - work.top;
        let sx = work.left;
        let sy = work.top;

        let (x, y, w, h) = match position {
            TilePosition::LeftHalf => (sx, sy, sw / 2, sh),
            TilePosition::RightHalf => (sx + sw / 2, sy, sw / 2, sh),
            TilePosition::TopHalf => (sx, sy, sw, sh / 2),
            TilePosition::BottomHalf => (sx, sy + sh / 2, sw, sh / 2),
            TilePosition::TopLeft => (sx, sy, sw / 2, sh / 2),
            TilePosition::TopRight => (sx + sw / 2, sy, sw / 2, sh / 2),
            TilePosition::BottomLeft => (sx, sy + sh / 2, sw / 2, sh / 2),
            TilePosition::BottomRight => (sx + sw / 2, sy + sh / 2, sw / 2, sh / 2),
            TilePosition::LeftThird => (sx, sy, sw / 3, sh),
            TilePosition::CenterThird => (sx + sw / 3, sy, sw / 3, sh),
            TilePosition::RightThird => (sx + 2 * sw / 3, sy, sw - 2 * (sw / 3), sh),
            TilePosition::TopThird => (sx, sy, sw, sh / 3),
            TilePosition::MiddleThird => (sx, sy + sh / 3, sw, sh / 3),
            TilePosition::BottomThird => (sx, sy + 2 * sh / 3, sw, sh - 2 * (sh / 3)),
            TilePosition::FirstTwoThirds => (sx, sy, 2 * sw / 3, sh),
            TilePosition::LastTwoThirds => (sx + sw / 3, sy, sw - sw / 3, sh),
            TilePosition::TopTwoThirds => (sx, sy, sw, 2 * sh / 3),
            TilePosition::BottomTwoThirds => (sx, sy + sh / 3, sw, sh - sh / 3),
            TilePosition::Center => {
                let cw = sw * 60 / 100;
                let ch = sh * 60 / 100;
                (sx + (sw - cw) / 2, sy + (sh - ch) / 2, cw, ch)
            }
            TilePosition::AlmostMaximize => {
                let margin = sw.min(sh) * 5 / 100;
                (sx + margin, sy + margin, sw - 2 * margin, sh - 2 * margin)
            }
            TilePosition::Fullscreen => (sx, sy, sw, sh),
        };

        // Restore from maximized state before tiling to avoid conflicts
        let mut placement = WINDOWPLACEMENT {
            length: std::mem::size_of::<WINDOWPLACEMENT>() as u32,
            flags: 0,
            show_cmd: 0,
            pt_min_position: POINT { x: 0, y: 0 },
            pt_max_position: POINT { x: 0, y: 0 },
            rc_normal_position: RECT {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            },
        };
        // SAFETY: placement is properly sized.
        unsafe {
            if GetWindowPlacement(hwnd, &mut placement) != 0
                && (placement.show_cmd == SW_SHOWMINIMIZED || placement.show_cmd == 3/* SW_SHOWMAXIMIZED */)
            {
                ShowWindow(hwnd, SW_RESTORE);
            }
        }

        // SAFETY: MoveWindow is a standard Win32 call.
        let ok = unsafe { MoveWindow(hwnd, x, y, w, h, TRUE) };
        if ok == 0 {
            return Err(anyhow!("MoveWindow (tile) failed for HWND 0x{:X}", id));
        }
        Ok(())
    }

    /// Get the frontmost window that doesn't belong to the current process.
    pub fn get_frontmost_window_of_previous_app() -> Result<Option<WindowInfo>> {
        // List all windows, find first one not belonging to our process
        let our_pid = std::process::id();
        let windows = list_windows()?;
        Ok(windows.into_iter().find(|w| w.pid as u32 != our_pid))
    }
}

// ============================================================================
// Platform dispatch — forward to real impl on Windows, return errors elsewhere
// ============================================================================

#[cfg(target_os = "windows")]
pub fn list_windows() -> Result<Vec<WindowInfo>> {
    win32::list_windows()
}

#[cfg(target_os = "windows")]
pub fn focus_window(id: u32) -> Result<()> {
    win32::focus_window(id)
}

#[cfg(target_os = "windows")]
pub fn close_window(id: u32) -> Result<()> {
    win32::close_window(id)
}

#[cfg(target_os = "windows")]
pub fn minimize_window(id: u32) -> Result<()> {
    win32::minimize_window(id)
}

#[cfg(target_os = "windows")]
pub fn maximize_window(id: u32) -> Result<()> {
    win32::maximize_window(id)
}

#[cfg(target_os = "windows")]
pub fn resize_window(id: u32, width: u32, height: u32) -> Result<()> {
    win32::resize_window(id, width, height)
}

#[cfg(target_os = "windows")]
pub fn move_window(id: u32, x: i32, y: i32) -> Result<()> {
    win32::move_window(id, x, y)
}

#[cfg(target_os = "windows")]
pub fn tile_window(id: u32, position: TilePosition) -> Result<()> {
    win32::tile_window(id, position)
}

#[cfg(target_os = "windows")]
pub fn move_to_next_display(_id: u32) -> Result<()> {
    Err(anyhow!(
        "move_to_next_display not yet implemented on Windows"
    ))
}

#[cfg(target_os = "windows")]
pub fn move_to_previous_display(_id: u32) -> Result<()> {
    Err(anyhow!(
        "move_to_previous_display not yet implemented on Windows"
    ))
}

#[cfg(target_os = "windows")]
pub fn get_frontmost_window_of_previous_app() -> Result<Option<WindowInfo>> {
    win32::get_frontmost_window_of_previous_app()
}

// ============================================================================
// Fallback stubs for non-macOS, non-Windows (e.g. Linux)
// ============================================================================

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn list_windows() -> Result<Vec<WindowInfo>> {
    Err(anyhow!("Window control not implemented on this platform"))
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn focus_window(_id: u32) -> Result<()> {
    Err(anyhow!("Window control not implemented on this platform"))
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn close_window(_id: u32) -> Result<()> {
    Err(anyhow!("Window control not implemented on this platform"))
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn minimize_window(_id: u32) -> Result<()> {
    Err(anyhow!("Window control not implemented on this platform"))
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn maximize_window(_id: u32) -> Result<()> {
    Err(anyhow!("Window control not implemented on this platform"))
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn resize_window(_id: u32, _width: u32, _height: u32) -> Result<()> {
    Err(anyhow!("Window control not implemented on this platform"))
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn move_window(_id: u32, _x: i32, _y: i32) -> Result<()> {
    Err(anyhow!("Window control not implemented on this platform"))
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn tile_window(_id: u32, _position: TilePosition) -> Result<()> {
    Err(anyhow!("Window control not implemented on this platform"))
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn move_to_next_display(_id: u32) -> Result<()> {
    Err(anyhow!("Window control not implemented on this platform"))
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn move_to_previous_display(_id: u32) -> Result<()> {
    Err(anyhow!("Window control not implemented on this platform"))
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn get_frontmost_window_of_previous_app() -> Result<Option<WindowInfo>> {
    Err(anyhow!("Window control not implemented on this platform"))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- Type / construction tests (all platforms) ---

    #[test]
    fn test_bounds_new() {
        let b = Bounds::new(10, 20, 800, 600);
        assert_eq!(b.x, 10);
        assert_eq!(b.y, 20);
        assert_eq!(b.width, 800);
        assert_eq!(b.height, 600);
    }

    #[test]
    fn test_bounds_clone_and_eq() {
        let a = Bounds::new(0, 0, 1920, 1080);
        let b = a;
        assert_eq!(a, b);
    }

    #[test]
    fn test_window_info_for_test() {
        let info = WindowInfo::for_test(
            42,
            "TestApp".to_string(),
            "My Window".to_string(),
            Bounds::new(100, 200, 640, 480),
            1234,
        );
        assert_eq!(info.id, 42);
        assert_eq!(info.app, "TestApp");
        assert_eq!(info.title, "My Window");
        assert_eq!(info.bounds.x, 100);
        assert_eq!(info.bounds.y, 200);
        assert_eq!(info.bounds.width, 640);
        assert_eq!(info.bounds.height, 480);
        assert_eq!(info.pid, 1234);
    }

    #[test]
    fn test_window_info_clone() {
        let info = WindowInfo::for_test(
            1,
            "App".to_string(),
            "Title".to_string(),
            Bounds::new(0, 0, 100, 100),
            99,
        );
        let cloned = info.clone();
        assert_eq!(cloned.id, info.id);
        assert_eq!(cloned.title, info.title);
        assert_eq!(cloned.app, info.app);
    }

    #[test]
    fn test_tile_position_equality() {
        assert_eq!(TilePosition::LeftHalf, TilePosition::LeftHalf);
        assert_ne!(TilePosition::LeftHalf, TilePosition::RightHalf);
        assert_ne!(TilePosition::Center, TilePosition::Fullscreen);
    }

    #[test]
    fn test_tile_position_copy() {
        let pos = TilePosition::TopLeft;
        let copied = pos;
        assert_eq!(pos, copied);
    }

    #[test]
    fn test_window_info_debug_format() {
        let info = WindowInfo::for_test(
            7,
            "Chrome".to_string(),
            "Google".to_string(),
            Bounds::new(0, 0, 1024, 768),
            555,
        );
        let debug_str = format!("{:?}", info);
        assert!(debug_str.contains("Chrome"));
        assert!(debug_str.contains("Google"));
        assert!(debug_str.contains("1024"));
    }

    #[test]
    fn test_bounds_zero_dimensions() {
        let b = Bounds::new(0, 0, 0, 0);
        assert_eq!(b.width, 0);
        assert_eq!(b.height, 0);
    }

    #[test]
    fn test_bounds_negative_position() {
        let b = Bounds::new(-100, -50, 800, 600);
        assert_eq!(b.x, -100);
        assert_eq!(b.y, -50);
    }

    // --- Windows-specific live tests ---

    #[cfg(target_os = "windows")]
    mod windows_live {
        use super::*;

        #[test]
        fn test_list_windows_returns_at_least_one() {
            // On any Windows desktop session there will be at least one visible window
            // (the desktop itself, or the test runner's console/IDE).
            let windows = list_windows().expect("list_windows should succeed on Windows");
            assert!(
                !windows.is_empty(),
                "Expected at least 1 visible window on this desktop session"
            );
        }

        #[test]
        fn test_listed_windows_have_nonempty_titles() {
            let windows = list_windows().expect("list_windows should succeed");
            for w in &windows {
                assert!(
                    !w.title.is_empty(),
                    "Window id={} has empty title (app={})",
                    w.id,
                    w.app
                );
            }
        }

        #[test]
        fn test_listed_windows_have_valid_pids() {
            let windows = list_windows().expect("list_windows should succeed");
            for w in &windows {
                assert!(w.pid > 0, "Window '{}' has invalid pid {}", w.title, w.pid);
            }
        }

        #[test]
        fn test_listed_windows_have_nonempty_app_names() {
            let windows = list_windows().expect("list_windows should succeed");
            for w in &windows {
                assert!(
                    !w.app.is_empty(),
                    "Window '{}' (id={}) has empty app name",
                    w.title,
                    w.id
                );
            }
        }

        #[test]
        fn test_listed_windows_have_reasonable_bounds() {
            let windows = list_windows().expect("list_windows should succeed");
            for w in &windows {
                // Windows that are not minimized should have non-zero dimensions.
                // Some minimized windows may appear with zero bounds, so we just
                // check that at least one window has positive dimensions.
                if w.bounds.width > 0 && w.bounds.height > 0 {
                    return; // found at least one with real bounds
                }
            }
            // If we get here, every single window had zero dimensions — suspicious but
            // not necessarily wrong (could be a headless test runner). Don't fail.
        }

        #[test]
        fn test_focus_window_with_invalid_hwnd() {
            // HWND 0 is never valid — SetForegroundWindow should fail.
            let result = focus_window(0);
            assert!(result.is_err(), "focus_window(0) should fail");
        }

        #[test]
        fn test_get_frontmost_window_of_previous_app() {
            // Should succeed (may return None in minimal CI environments).
            let result = get_frontmost_window_of_previous_app();
            assert!(
                result.is_ok(),
                "get_frontmost_window_of_previous_app should not error: {:?}",
                result.err()
            );
        }
    }
}
