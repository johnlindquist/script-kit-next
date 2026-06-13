//! Post-copy clipboard tracking hooks.
//!
//! Clipboard copies are tracked by the sediment pipeline, not by a post-copy
//! popup. This module only keeps the quiet "Kept" HUD bridge for auto-kept
//! content and accepts the old post-copy-menu config as a no-op for backward
//! compatibility with existing config files.

use std::sync::{LazyLock, OnceLock};

use anyhow::Result;
use gpui::App;
use tracing::info;

/// Deprecated post-copy popup configuration.
///
/// The fields are still deserialized from `clipboardHistoryPostCopyMenu` so
/// existing user config remains valid, but copied content no longer opens a
/// post-copy popup or installs a modifier-tap event monitor.
#[derive(Debug, Clone)]
pub struct PostCopyMenuConfig {
    pub enabled: bool,
    pub tap_window_ms: u64,
    pub trigger_modifiers: Vec<String>,
}

impl Default for PostCopyMenuConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            tap_window_ms: 2500,
            trigger_modifiers: vec!["meta".to_string()],
        }
    }
}

/// Optional HUD whisper hook (registered from the binary crate where `hud_manager` lives).
pub type KeptHudWhisperFn = fn(&mut App);

static KEPT_HUD_WHISPER: OnceLock<KeptHudWhisperFn> = OnceLock::new();
static INSTALLED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

#[derive(Debug, Clone)]
enum PostCopyUiEvent {
    ShowKeptHud,
}

static POST_COPY_UI_CHANNEL: LazyLock<(
    async_channel::Sender<PostCopyUiEvent>,
    async_channel::Receiver<PostCopyUiEvent>,
)> = LazyLock::new(|| async_channel::bounded(32));

/// Register the quiet "Kept" HUD handler (call from app startup before install).
pub fn register_kept_hud_whisper(handler: KeptHudWhisperFn) {
    let _ = KEPT_HUD_WHISPER.set(handler);
}

/// Accept deprecated post-copy popup settings without enabling popup behavior.
pub fn configure_post_copy_menu(_config: PostCopyMenuConfig) {}

/// Notify the post-copy lane that a text entry was stored.
///
/// This intentionally does not open UI. Brain/sediment storage is handled by
/// `process_text_sediment` before this hook is called.
pub fn notify_text_copy_stored(_entry_id: &str) {}

/// Queue a quiet HUD whisper for an auto-keep (ADR 0004).
pub fn request_kept_hud_whisper() {
    let _ = POST_COPY_UI_CHANNEL
        .0
        .try_send(PostCopyUiEvent::ShowKeptHud);
}

/// Install the post-copy HUD bridge. Idempotent and popup-free.
pub fn install_post_copy_tracker(cx: &mut App) -> Result<()> {
    if INSTALLED.swap(true, std::sync::atomic::Ordering::SeqCst) {
        return Ok(());
    }

    let rx = POST_COPY_UI_CHANNEL.1.clone();
    cx.spawn(async move |cx: &mut gpui::AsyncApp| {
        while let Ok(event) = rx.recv().await {
            cx.update(|cx| match event {
                PostCopyUiEvent::ShowKeptHud => {
                    if let Some(show) = KEPT_HUD_WHISPER.get() {
                        show(cx);
                    }
                }
            });
        }
    })
    .detach();

    info!("post-copy clipboard tracker installed without popup UI");
    Ok(())
}
