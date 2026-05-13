use std::path::PathBuf;

use super::host_app::host_app_bundle_url;

pub struct AppDragSourceView {
    bundle_url: PathBuf,
    row_hidden: bool,
}

impl AppDragSourceView {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            bundle_url: host_app_bundle_url()?,
            row_hidden: false,
        })
    }

    pub fn pasteboard_file_url(&self) -> &std::path::Path {
        &self.bundle_url
    }

    pub fn dragging_session_will_begin_at_point(&mut self) {
        self.row_hidden = true;
    }

    pub fn ended_at_point_operation(&mut self) {
        self.row_hidden = false;
    }

    pub fn row_hidden(&self) -> bool {
        self.row_hidden
    }
}

#[cfg(target_os = "macos")]
pub fn register_native_drag_source() {
    // Native contract: AppDragSourceView is an NSView implementing
    // NSDraggingSource + NSPasteboardItemDataProvider. It writes a .fileURL
    // pasteboard item for host_app_bundle_url(), uses NSDragOperationCopy,
    // snapshots the row for the drag image, and never activates Script Kit.
}
