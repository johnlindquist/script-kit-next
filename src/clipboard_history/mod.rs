//! Clipboard History Module
//!
//! Provides SQLite-backed clipboard history with background monitoring.
//!
//! ## Features
//! - Stores text and base64-encoded images
//! - Background polling every 500ms
//! - Time-based retention (default 30 days)
//! - Pin/unpin entries to prevent deletion
//! - Pagination support for lazy loading
//! - Time-based grouping (Today, Yesterday, This Week, etc.)
//! - OCR text storage for image entries
//!
//! ## Module Structure
//! - `types`: Core types (ContentType, TimeGroup, ClipboardEntry)
//! - `config`: Retention and text length configuration
//! - `cache`: LRU caching for images and entries
//! - `database`: SQLite operations (CRUD, migrations)
//! - `image`: Image encoding/decoding (PNG, RGBA)
//! - `monitor`: Background clipboard polling and maintenance
//! - `clipboard`: System clipboard operations

mod blob_store;
mod cache;
mod change_detection;
mod clipboard;
mod config;
mod database;
mod db_worker;
mod image;
mod monitor;
mod types;

// Re-export public API
// These exports form the public API of the clipboard_history module.
// Some may appear unused in this crate but are used by external consumers.

// Types
#[allow(unused_imports)]
pub use types::{
    classify_timestamp, group_entries_by_time, ClipboardEntry, ClipboardEntryMeta, ContentType,
    TimeGroup,
};

// DB Worker (new architecture - message passing instead of global mutex)
#[allow(unused_imports)]
pub use db_worker::{get_db_sender, start_db_worker, DbRequest};

// Config
#[allow(unused_imports)]
pub use config::{
    get_max_text_content_len, get_retention_days, set_max_text_content_len, set_retention_days,
};

// Cache
#[allow(unused_imports)]
pub use cache::{cache_image, get_cached_entries, get_cached_image};

// Database operations
#[allow(unused_imports)]
pub use database::{
    clear_history, get_clipboard_history, get_clipboard_history_meta, get_clipboard_history_page,
    get_entry_by_id, get_entry_content, get_total_entry_count, pin_entry, remove_entry,
    trim_oversize_text_entries, unpin_entry, update_ocr_text,
};

// Image operations
#[allow(unused_imports)]
pub use image::decode_to_render_image;

// Monitor/Init
#[allow(unused_imports)]
pub use monitor::{init_clipboard_history, stop_clipboard_monitoring};

// Clipboard operations
pub use clipboard::copy_entry_to_clipboard;

// Test-only exports
#[cfg(test)]
#[allow(unused_imports)]
pub use types::classify_timestamp_with_now;
