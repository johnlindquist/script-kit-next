use std::path::{Path, PathBuf};

fn resolve_existing_path(path: &Path, action: &str) -> Result<PathBuf, String> {
    std::fs::canonicalize(path).map_err(|error| {
        let error_message = format!(
            "platform_{}_failed: stage=canonicalize_path path={} error={}",
            action,
            path.display(),
            error
        );
        tracing::error!(
            action = action,
            stage = "canonicalize_path",
            path = %path.display(),
            error = %error,
            "platform path action failed"
        );
        error_message
    })
}

/// Reveal the path in Finder on macOS.
///
/// On non-macOS platforms, opens the containing folder with the system default handler.
pub fn reveal_in_finder(path: &Path) -> Result<(), String> {
    let resolved_path = resolve_existing_path(path, "reveal_in_finder")?;
    reveal_in_finder_impl(&resolved_path)
}

#[cfg(target_os = "macos")]
fn reveal_in_finder_impl(path: &Path) -> Result<(), String> {
    if require_main_thread("reveal_in_finder") {
        let error_message = format!(
            "platform_reveal_in_finder_failed: stage=main_thread_required path={} error=AppKit requires main thread",
            path.display()
        );
        tracing::error!(
            action = "reveal_in_finder",
            stage = "main_thread_required",
            path = %path.display(),
            "platform path action failed"
        );
        return Err(error_message);
    }

    let path_str = path.to_string_lossy();

    // SAFETY: Main thread is verified above. Objective-C objects are checked for nil
    // before use and the method calls are standard NSWorkspace/NSURL APIs.
    unsafe {
        let workspace: cocoa::base::id =
            objc::msg_send![objc::class!(NSWorkspace), sharedWorkspace];
        if workspace == cocoa::base::nil {
            let error_message = format!(
                "platform_reveal_in_finder_failed: stage=workspace_unavailable path={} error=NSWorkspace.sharedWorkspace returned nil",
                path.display()
            );
            tracing::error!(
                action = "reveal_in_finder",
                stage = "workspace_unavailable",
                path = %path.display(),
                "platform path action failed"
            );
            return Err(error_message);
        }

        let ns_path = cocoa::foundation::NSString::alloc(cocoa::base::nil).init_str(&path_str);
        if ns_path == cocoa::base::nil {
            let error_message = format!(
                "platform_reveal_in_finder_failed: stage=nsstring_creation path={} error=Failed to create NSString",
                path.display()
            );
            tracing::error!(
                action = "reveal_in_finder",
                stage = "nsstring_creation",
                path = %path.display(),
                "platform path action failed"
            );
            return Err(error_message);
        }

        let file_url: cocoa::base::id =
            objc::msg_send![objc::class!(NSURL), fileURLWithPath: ns_path];
        if file_url == cocoa::base::nil {
            let _: () = objc::msg_send![ns_path, release];
            let error_message = format!(
                "platform_reveal_in_finder_failed: stage=file_url_creation path={} error=Failed to create file URL",
                path.display()
            );
            tracing::error!(
                action = "reveal_in_finder",
                stage = "file_url_creation",
                path = %path.display(),
                "platform path action failed"
            );
            return Err(error_message);
        }

        let urls: cocoa::base::id =
            objc::msg_send![objc::class!(NSArray), arrayWithObject: file_url];
        if urls == cocoa::base::nil {
            let _: () = objc::msg_send![ns_path, release];
            let error_message = format!(
                "platform_reveal_in_finder_failed: stage=url_array_creation path={} error=Failed to create NSArray",
                path.display()
            );
            tracing::error!(
                action = "reveal_in_finder",
                stage = "url_array_creation",
                path = %path.display(),
                "platform path action failed"
            );
            return Err(error_message);
        }

        tracing::debug!(
            action = "reveal_in_finder",
            stage = "dispatch_finder_select",
            path = %path.display(),
            "dispatching NSWorkspace reveal request"
        );

        let _: () = objc::msg_send![workspace, activateFileViewerSelectingURLs: urls];
        let _: () = objc::msg_send![ns_path, release];

        tracing::info!(
            action = "reveal_in_finder",
            stage = "completed",
            path = %path.display(),
            "platform path action completed"
        );
        Ok(())
    }
}

#[cfg(not(target_os = "macos"))]
fn reveal_in_finder_impl(path: &Path) -> Result<(), String> {
    let target = if path.is_dir() {
        path
    } else {
        path.parent().unwrap_or(path)
    };

    tracing::debug!(
        action = "reveal_in_finder",
        stage = "fallback_open_parent",
        path = %path.display(),
        target = %target.display(),
        "using non-macOS reveal fallback"
    );

    open::that(target).map_err(|error| {
        let error_message = format!(
            "platform_reveal_in_finder_failed: stage=fallback_open_parent path={} target={} error={}",
            path.display(),
            target.display(),
            error
        );
        tracing::error!(
            action = "reveal_in_finder",
            stage = "fallback_open_parent",
            path = %path.display(),
            target = %target.display(),
            error = %error,
            "platform path action failed"
        );
        error_message
    })?;

    tracing::info!(
        action = "reveal_in_finder",
        stage = "completed_fallback",
        path = %path.display(),
        target = %target.display(),
        "platform path action completed"
    );

    Ok(())
}

/// Open a path with the default application.
///
/// On macOS, this uses `NSWorkspace.openURL` with a `file://` URL.
pub fn open_in_default_app(path: &Path) -> Result<(), String> {
    let resolved_path = resolve_existing_path(path, "open_in_default_app")?;
    open_in_default_app_impl(&resolved_path)
}

#[cfg(target_os = "macos")]
fn open_in_default_app_impl(path: &Path) -> Result<(), String> {
    if require_main_thread("open_in_default_app") {
        let error_message = format!(
            "platform_open_in_default_app_failed: stage=main_thread_required path={} error=AppKit requires main thread",
            path.display()
        );
        tracing::error!(
            action = "open_in_default_app",
            stage = "main_thread_required",
            path = %path.display(),
            "platform path action failed"
        );
        return Err(error_message);
    }

    let path_str = path.to_string_lossy();

    // SAFETY: Main thread is verified above. Objective-C objects are checked for nil
    // before use and the method calls are standard NSWorkspace/NSURL APIs.
    unsafe {
        let workspace: cocoa::base::id =
            objc::msg_send![objc::class!(NSWorkspace), sharedWorkspace];
        if workspace == cocoa::base::nil {
            let error_message = format!(
                "platform_open_in_default_app_failed: stage=workspace_unavailable path={} error=NSWorkspace.sharedWorkspace returned nil",
                path.display()
            );
            tracing::error!(
                action = "open_in_default_app",
                stage = "workspace_unavailable",
                path = %path.display(),
                "platform path action failed"
            );
            return Err(error_message);
        }

        let ns_path = cocoa::foundation::NSString::alloc(cocoa::base::nil).init_str(&path_str);
        if ns_path == cocoa::base::nil {
            let error_message = format!(
                "platform_open_in_default_app_failed: stage=nsstring_creation path={} error=Failed to create NSString",
                path.display()
            );
            tracing::error!(
                action = "open_in_default_app",
                stage = "nsstring_creation",
                path = %path.display(),
                "platform path action failed"
            );
            return Err(error_message);
        }

        let file_url: cocoa::base::id =
            objc::msg_send![objc::class!(NSURL), fileURLWithPath: ns_path];
        if file_url == cocoa::base::nil {
            let _: () = objc::msg_send![ns_path, release];
            let error_message = format!(
                "platform_open_in_default_app_failed: stage=file_url_creation path={} error=Failed to create file URL",
                path.display()
            );
            tracing::error!(
                action = "open_in_default_app",
                stage = "file_url_creation",
                path = %path.display(),
                "platform path action failed"
            );
            return Err(error_message);
        }

        tracing::debug!(
            action = "open_in_default_app",
            stage = "dispatch_open_url",
            path = %path.display(),
            "dispatching NSWorkspace open request"
        );

        let did_open: bool = objc::msg_send![workspace, openURL: file_url];
        let _: () = objc::msg_send![ns_path, release];

        if !did_open {
            let error_message = format!(
                "platform_open_in_default_app_failed: stage=open_url path={} error=NSWorkspace.openURL returned false",
                path.display()
            );
            tracing::error!(
                action = "open_in_default_app",
                stage = "open_url",
                path = %path.display(),
                "platform path action failed"
            );
            return Err(error_message);
        }

        tracing::info!(
            action = "open_in_default_app",
            stage = "completed",
            path = %path.display(),
            "platform path action completed"
        );
        Ok(())
    }
}

#[cfg(not(target_os = "macos"))]
fn open_in_default_app_impl(path: &Path) -> Result<(), String> {
    tracing::debug!(
        action = "open_in_default_app",
        stage = "fallback_open",
        path = %path.display(),
        "using non-macOS open fallback"
    );

    open::that(path).map_err(|error| {
        let error_message = format!(
            "platform_open_in_default_app_failed: stage=fallback_open path={} error={}",
            path.display(),
            error
        );
        tracing::error!(
            action = "open_in_default_app",
            stage = "fallback_open",
            path = %path.display(),
            error = %error,
            "platform path action failed"
        );
        error_message
    })?;

    tracing::info!(
        action = "open_in_default_app",
        stage = "completed_fallback",
        path = %path.display(),
        "platform path action completed"
    );

    Ok(())
}

/// Copy text to the system clipboard.
///
/// On macOS this uses `NSPasteboard`. On other platforms it falls back to `arboard`.
pub fn copy_text_to_clipboard(text: &str) -> Result<(), String> {
    copy_text_to_clipboard_impl(text)
}

#[cfg(target_os = "macos")]
fn copy_text_to_clipboard_impl(text: &str) -> Result<(), String> {
    if require_main_thread("copy_text_to_clipboard") {
        let error_message =
            "platform_copy_text_to_clipboard_failed: stage=main_thread_required error=AppKit requires main thread"
                .to_string();
        tracing::error!(
            action = "copy_text_to_clipboard",
            stage = "main_thread_required",
            "platform clipboard action failed"
        );
        return Err(error_message);
    }

    // SAFETY: Main thread is verified above. Objective-C objects are checked for nil
    // before use and the method calls are standard NSPasteboard APIs.
    unsafe {
        let pasteboard: cocoa::base::id =
            objc::msg_send![objc::class!(NSPasteboard), generalPasteboard];
        if pasteboard == cocoa::base::nil {
            let error_message =
                "platform_copy_text_to_clipboard_failed: stage=pasteboard_unavailable error=NSPasteboard.generalPasteboard returned nil".to_string();
            tracing::error!(
                action = "copy_text_to_clipboard",
                stage = "pasteboard_unavailable",
                "platform clipboard action failed"
            );
            return Err(error_message);
        }

        let _: i64 = objc::msg_send![pasteboard, clearContents];

        let ns_text = cocoa::foundation::NSString::alloc(cocoa::base::nil).init_str(text);
        if ns_text == cocoa::base::nil {
            let error_message =
                "platform_copy_text_to_clipboard_failed: stage=nsstring_creation error=Failed to create NSString".to_string();
            tracing::error!(
                action = "copy_text_to_clipboard",
                stage = "nsstring_creation",
                "platform clipboard action failed"
            );
            return Err(error_message);
        }

        let objects: cocoa::base::id =
            objc::msg_send![objc::class!(NSArray), arrayWithObject: ns_text];
        if objects == cocoa::base::nil {
            let _: () = objc::msg_send![ns_text, release];
            let error_message =
                "platform_copy_text_to_clipboard_failed: stage=object_array_creation error=Failed to create NSArray".to_string();
            tracing::error!(
                action = "copy_text_to_clipboard",
                stage = "object_array_creation",
                "platform clipboard action failed"
            );
            return Err(error_message);
        }

        let did_write: bool = objc::msg_send![pasteboard, writeObjects: objects];
        let _: () = objc::msg_send![ns_text, release];

        if !did_write {
            let error_message =
                "platform_copy_text_to_clipboard_failed: stage=write_objects error=NSPasteboard.writeObjects returned false".to_string();
            tracing::error!(
                action = "copy_text_to_clipboard",
                stage = "write_objects",
                "platform clipboard action failed"
            );
            return Err(error_message);
        }

        tracing::info!(
            action = "copy_text_to_clipboard",
            stage = "completed",
            text_length = text.len(),
            "platform clipboard action completed"
        );
        Ok(())
    }
}

#[cfg(not(target_os = "macos"))]
fn copy_text_to_clipboard_impl(text: &str) -> Result<(), String> {
    tracing::debug!(
        action = "copy_text_to_clipboard",
        stage = "fallback_arboard",
        text_length = text.len(),
        "using non-macOS clipboard fallback"
    );

    let mut clipboard = arboard::Clipboard::new().map_err(|error| {
        let error_message = format!(
            "platform_copy_text_to_clipboard_failed: stage=fallback_clipboard_init error={}",
            error
        );
        tracing::error!(
            action = "copy_text_to_clipboard",
            stage = "fallback_clipboard_init",
            error = %error,
            "platform clipboard action failed"
        );
        error_message
    })?;

    clipboard.set_text(text.to_string()).map_err(|error| {
        let error_message = format!(
            "platform_copy_text_to_clipboard_failed: stage=fallback_clipboard_set_text error={}",
            error
        );
        tracing::error!(
            action = "copy_text_to_clipboard",
            stage = "fallback_clipboard_set_text",
            error = %error,
            "platform clipboard action failed"
        );
        error_message
    })?;

    tracing::info!(
        action = "copy_text_to_clipboard",
        stage = "completed_fallback",
        text_length = text.len(),
        "platform clipboard action completed"
    );

    Ok(())
}
