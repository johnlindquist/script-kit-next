use super::image::NativeImage;
use super::quartz;
use crate::{
    CaptureOptions, CaptureTarget, ImageFormat, Point, Rect, Result, ScreenshotError,
    WindowFrameMode,
};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

pub(super) fn capture_system(
    target: CaptureTarget,
    options: &CaptureOptions,
) -> Result<(NativeImage, CaptureTarget)> {
    let (mut args, resolved_target) = args_for_target(target, options, false)?;
    let path = temp_path(options.format);

    args.push(path.as_os_str().to_os_string());
    let output = Command::new("/usr/sbin/screencapture")
        .args(&args)
        .output()?;
    if !output.status.success() {
        let _ = std::fs::remove_file(&path);
        return Err(ScreenshotError::SystemCapture(format_command_error(
            output.stderr,
        )));
    }

    let bytes = std::fs::read(&path)?;
    let _ = std::fs::remove_file(&path);
    let image = NativeImage::from_encoded_bytes(&bytes)?;
    Ok((image, resolved_target))
}

pub(super) fn capture_clipboard(target: CaptureTarget, options: &CaptureOptions) -> Result<()> {
    let (mut args, _) = args_for_target(target, options, true)?;
    args.push(OsString::from("-c"));
    let output = Command::new("/usr/sbin/screencapture")
        .args(&args)
        .output()?;
    if !output.status.success() {
        return Err(ScreenshotError::SystemCapture(format_command_error(
            output.stderr,
        )));
    }
    Ok(())
}

pub(super) fn open_screen_recording_settings() -> Result<()> {
    // Apple has changed settings URLs across macOS releases. This URL works on recent macOS;
    // failures simply fall back to opening Privacy & Security.
    let primary = "x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture";
    let fallback = "x-apple.systempreferences:com.apple.preference.security";
    let status = Command::new("/usr/bin/open").arg(primary).status();
    match status {
        Ok(s) if s.success() => Ok(()),
        _ => {
            let status = Command::new("/usr/bin/open").arg(fallback).status()?;
            if status.success() {
                Ok(())
            } else {
                Err(ScreenshotError::SystemCapture(
                    "failed to open System Settings".into(),
                ))
            }
        }
    }
}

fn args_for_target(
    target: CaptureTarget,
    options: &CaptureOptions,
    clipboard: bool,
) -> Result<(Vec<OsString>, CaptureTarget)> {
    if options.include_cursor
        && matches!(
            target,
            CaptureTarget::Interactive
                | CaptureTarget::InteractiveSelection
                | CaptureTarget::InteractiveWindow
                | CaptureTarget::InteractiveToolbar
        )
    {
        return Err(ScreenshotError::InvalidInput(
            "the system screencapture tool only allows cursor capture in non-interactive modes"
                .into(),
        ));
    }
    if matches!(options.window_frame, WindowFrameMode::ShadowOnly) {
        return Err(ScreenshotError::UnsupportedBackend("the system screencapture backend cannot capture only a window shadow; use CoreGraphics"));
    }

    let mut args = Vec::new();

    if !options.play_sound {
        args.push(OsString::from("-x"));
    }
    if options.include_cursor {
        args.push(OsString::from("-C"));
    }
    if matches!(options.window_frame, WindowFrameMode::WithoutShadow) {
        args.push(OsString::from("-o"));
    }
    if options.delay.as_millis() > 0 {
        args.push(OsString::from("-T"));
        let millis = options.delay.as_millis();
        let seconds = ((millis + 999) / 1000).max(1);
        args.push(OsString::from(seconds.to_string()));
    }
    if !clipboard {
        args.push(OsString::from("-t"));
        args.push(OsString::from(screencapture_format(options.format)));
    }

    let resolved = match target {
        CaptureTarget::AllDisplays => CaptureTarget::AllDisplays,
        CaptureTarget::MainDisplay => {
            args.push(OsString::from("-m"));
            CaptureTarget::MainDisplay
        }
        CaptureTarget::DisplayOrdinal(ordinal) => {
            if ordinal == 0 {
                return Err(ScreenshotError::InvalidInput(
                    "display ordinal is 1-based".into(),
                ));
            }
            args.push(OsString::from("-D"));
            args.push(OsString::from(ordinal.to_string()));
            CaptureTarget::DisplayOrdinal(ordinal)
        }
        CaptureTarget::Display(display_id) => {
            let display = quartz::displays()?
                .into_iter()
                .find(|d| d.id == display_id)
                .ok_or_else(|| ScreenshotError::NotFound(format!("display {display_id}")))?;
            push_region_args(&mut args, display.bounds);
            CaptureTarget::Display(display_id)
        }
        CaptureTarget::Region(rect) => {
            validate_rect(rect)?;
            push_region_args(&mut args, rect);
            CaptureTarget::Region(rect)
        }
        CaptureTarget::DisplayRegion { display_id, rect } => {
            validate_rect(rect)?;
            let display = quartz::displays()?
                .into_iter()
                .find(|d| d.id == display_id)
                .ok_or_else(|| ScreenshotError::NotFound(format!("display {display_id}")))?;
            let global = Rect::new(
                display.bounds.x + rect.x,
                display.bounds.y + rect.y,
                rect.width,
                rect.height,
            );
            push_region_args(&mut args, global);
            CaptureTarget::Region(global)
        }
        CaptureTarget::Window(id) => {
            push_window_args(&mut args, id);
            CaptureTarget::Window(id)
        }
        CaptureTarget::FrontmostWindow => {
            let id = quartz::frontmost_window()?.id;
            push_window_args(&mut args, id);
            CaptureTarget::Window(id)
        }
        CaptureTarget::WindowUnderCursor => {
            let id = quartz::window_at_point(quartz::mouse_location()?)?.id;
            push_window_args(&mut args, id);
            CaptureTarget::Window(id)
        }
        CaptureTarget::WindowAtPoint(point) => {
            let id = quartz::window_at_point(point)?.id;
            push_window_args(&mut args, id);
            CaptureTarget::Window(id)
        }
        CaptureTarget::Interactive => {
            args.push(OsString::from("-i"));
            CaptureTarget::Interactive
        }
        CaptureTarget::InteractiveSelection => {
            args.push(OsString::from("-i"));
            args.push(OsString::from("-s"));
            CaptureTarget::InteractiveSelection
        }
        CaptureTarget::InteractiveWindow => {
            args.push(OsString::from("-i"));
            args.push(OsString::from("-w"));
            CaptureTarget::InteractiveWindow
        }
        CaptureTarget::InteractiveToolbar => {
            args.push(OsString::from("-i"));
            args.push(OsString::from("-U"));
            CaptureTarget::InteractiveToolbar
        }
        CaptureTarget::TouchBar => {
            if options.include_cursor {
                return Err(ScreenshotError::InvalidInput(
                    "Touch Bar captures cannot include the cursor".into(),
                ));
            }
            args.push(OsString::from("-b"));
            CaptureTarget::TouchBar
        }
        CaptureTarget::Windows(ids) => {
            if ids.len() == 1 {
                push_window_args(&mut args, ids[0]);
                CaptureTarget::Window(ids[0])
            } else {
                return Err(ScreenshotError::UnsupportedBackend("/usr/sbin/screencapture can capture one window by id; use CoreGraphics for multi-window composites"));
            }
        }
        CaptureTarget::Application(_pid) => {
            return Err(ScreenshotError::UnsupportedBackend(match options.include_cursor {
                true => "application screenshots with cursor are not supported by the system backend; capture a window/region or use ScreenCaptureKit directly",
                false => "application screenshots are handled by the CoreGraphics backend",
            }));
        }
        CaptureTarget::VisibleWindows { .. } => {
            return Err(ScreenshotError::UnsupportedBackend(
                "visible-window composites with exclusions are handled by the CoreGraphics backend",
            ));
        }
    };

    Ok((args, resolved))
}

fn push_window_args(args: &mut Vec<OsString>, id: u32) {
    args.push(OsString::from("-l"));
    args.push(OsString::from(id.to_string()));
}

fn push_region_args(args: &mut Vec<OsString>, rect: Rect) {
    args.push(OsString::from("-R"));
    args.push(OsString::from(format!(
        "{},{},{},{}",
        round_for_cli(rect.x),
        round_for_cli(rect.y),
        round_for_cli(rect.width),
        round_for_cli(rect.height),
    )));
}

fn round_for_cli(value: f64) -> i64 {
    value.round() as i64
}

fn screencapture_format(format: ImageFormat) -> &'static str {
    match format {
        ImageFormat::Png => "png",
        ImageFormat::Jpeg { .. } => "jpg",
        ImageFormat::Tiff => "tiff",
        ImageFormat::Heic { .. } => "heic",
        ImageFormat::Pdf => "pdf",
        ImageFormat::Bmp => "bmp",
    }
}

fn temp_path(format: ImageFormat) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or_default();
    std::env::temp_dir().join(format!(
        "macos-screenshot-kit-{}-{nanos}.{}",
        std::process::id(),
        format.extension(),
    ))
}

fn validate_rect(rect: Rect) -> Result<()> {
    if !rect.x.is_finite()
        || !rect.y.is_finite()
        || !rect.width.is_finite()
        || !rect.height.is_finite()
        || rect.is_empty()
    {
        Err(ScreenshotError::InvalidInput(format!(
            "invalid capture rectangle: {rect:?}"
        )))
    } else {
        Ok(())
    }
}

fn format_command_error(stderr: Vec<u8>) -> String {
    let text = String::from_utf8_lossy(&stderr).trim().to_string();
    if text.is_empty() {
        "screencapture returned a non-zero exit code".into()
    } else {
        text
    }
}

#[allow(dead_code)]
fn _keep(_: Point, _: &Path) {}
