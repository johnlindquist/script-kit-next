use super::*;
use crate::theme::opacity::{OPACITY_HOVER, OPACITY_SELECTED};

#[derive(Clone, Debug)]
pub(super) enum ImageCacheSource {
    ChatHistory,
    PendingInput,
}

impl ImageCacheSource {
    fn as_str(&self) -> &'static str {
        match self {
            Self::ChatHistory => "chat_history",
            Self::PendingInput => "pending_input",
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct ImageCacheRequest {
    pub source: ImageCacheSource,
    pub source_id: String,
    pub base64_data: String,
}

impl ImageCacheRequest {
    pub fn new(
        source: ImageCacheSource,
        source_id: impl Into<String>,
        base64_data: String,
    ) -> Self {
        Self {
            source,
            source_id: source_id.into(),
            base64_data,
        }
    }

    pub fn cache_key(&self) -> String {
        AiApp::image_cache_key(&self.base64_data)
    }

    pub fn base64_len(&self) -> usize {
        self.base64_data.len()
    }
}

/// Intermediate result from background image preparation.
/// Contains decoded PNG bytes plus structured provenance for logging.
struct PreparedImageCacheWork {
    source: ImageCacheSource,
    source_id: String,
    cache_key: String,
    base64_len: usize,
    png_bytes: Vec<u8>,
}

impl AiApp {
    pub(super) fn on_input_change(&mut self, cx: &mut Context<Self>) {
        let value = self.input_state.read(cx).value().to_string();

        // Detect @mention trigger for context picker
        if let Some(at_query) = extract_at_query(&value) {
            if self.is_context_picker_open() {
                // Update existing picker with new query
                self.update_context_picker_query(at_query, cx);
            } else {
                // Open the picker (no Window needed for open_context_picker when
                // called from input change — pass a placeholder; the real Window
                // is only needed when accepting, which goes through keydown).
                let items = super::context_picker::build_picker_items(&at_query);
                tracing::info!(
                    target: "ai",
                    query = %at_query,
                    item_count = items.len(),
                    selected_index = 0,
                    "ai_context_picker_opened"
                );
                self.context_picker = Some(super::context_picker::types::ContextPickerState::new(
                    at_query, items,
                ));
                cx.notify();
            }
        } else if self.is_context_picker_open() {
            // No @ query detected — close the picker
            self.close_context_picker(cx);
        }

        // Context preflight is intentionally NOT run per-keystroke.
        // It only runs when the user explicitly adds context parts
        // (via /context slash commands or the context picker).
    }

    /// Handle paste event - check for clipboard images
    ///
    /// If clipboard contains an image, encode it as base64 and store as pending_image.
    /// If clipboard contains text, let the normal input handling process it.
    ///
    /// Returns true if an image was pasted (caller should not process text).
    pub(super) fn handle_paste_for_image(&mut self, cx: &mut Context<Self>) -> bool {
        use crate::prompts::chat::MAX_IMAGE_BYTES;

        // Use arboard to read clipboard since it handles images
        match arboard::Clipboard::new() {
            Ok(mut clipboard) => {
                // Check for image first
                if let Ok(image_data) = clipboard.get_image() {
                    // Convert image to base64 PNG
                    match crate::clipboard_history::encode_image_as_png(&image_data) {
                        Ok(encoded) => {
                            // Strip the "png:" prefix since we store raw base64
                            let base64_data =
                                encoded.strip_prefix("png:").unwrap_or(&encoded).to_string();

                            // Check raw PNG size via base64 decode
                            use base64::Engine;
                            if let Ok(png_bytes) =
                                base64::engine::general_purpose::STANDARD.decode(&base64_data)
                            {
                                if png_bytes.len() > MAX_IMAGE_BYTES {
                                    tracing::warn!(
                                        size_bytes = png_bytes.len(),
                                        max_bytes = MAX_IMAGE_BYTES,
                                        "Rejecting pasted image larger than 10 MB in AI window"
                                    );
                                    self.streaming_error = Some(
                                        "Pasted image exceeds 10 MB limit. Try a smaller image."
                                            .to_string(),
                                    );
                                    cx.notify();
                                    return false;
                                }
                            }

                            let size_kb = base64_data.len() / 1024;
                            info!(
                                width = image_data.width,
                                height = image_data.height,
                                size_kb = size_kb,
                                "Image pasted from clipboard"
                            );

                            self.cache_image_from_base64(&base64_data);
                            self.pending_image = Some(base64_data);
                            cx.notify();
                            return true;
                        }
                        Err(e) => {
                            tracing::warn!(error = %e, "Failed to encode pasted image");
                        }
                    }
                }
            }
            Err(e) => {
                tracing::warn!(error = %e, "Failed to access clipboard");
            }
        }
        false
    }

    /// Remove the pending image attachment
    pub(super) fn remove_pending_image(&mut self, cx: &mut Context<Self>) {
        if self.pending_image.is_some() {
            self.pending_image = None;
            info!("Pending image removed");
            cx.notify();
        }
    }

    /// Build a cache key for base64 image data (prefix + length).
    pub(super) fn image_cache_key(base64_data: &str) -> String {
        let prefix: String = base64_data.chars().take(64).collect();
        format!("{}:{}", prefix, base64_data.len())
    }

    /// Decode a base64 PNG and store it in the image cache.
    /// Call this eagerly when an image is attached (not during render).
    pub(super) fn cache_image_from_base64(&mut self, base64_data: &str) {
        let cache_key = Self::image_cache_key(base64_data);
        if self.image_cache.contains_key(&cache_key) {
            return;
        }

        use base64::Engine;
        let bytes = match base64::engine::general_purpose::STANDARD.decode(base64_data) {
            Ok(b) => b,
            Err(e) => {
                tracing::warn!(error = %e, "Failed to decode base64 image data");
                return;
            }
        };

        match crate::list_item::decode_png_to_render_image_with_bgra_conversion(&bytes) {
            Ok(render_image) => {
                info!(
                    cache_key_prefix = &cache_key[..cache_key.len().min(30)],
                    "Cached decoded image thumbnail"
                );
                self.image_cache.insert(cache_key, render_image);
            }
            Err(e) => {
                tracing::warn!(error = %e, "Failed to decode PNG image for thumbnail");
            }
        }
    }

    /// Look up a cached RenderImage by base64 data. Returns None if not cached.
    pub(super) fn get_cached_image(
        &self,
        base64_data: &str,
    ) -> Option<std::sync::Arc<RenderImage>> {
        let cache_key = Self::image_cache_key(base64_data);
        self.image_cache.get(&cache_key).cloned()
    }

    /// Cache all images from a slice of messages (call after loading messages).
    pub(super) fn cache_message_images(&mut self, messages: &[Message]) {
        for msg in messages {
            for attachment in &msg.images {
                self.cache_image_from_base64(&attachment.data);
            }
        }
    }

    /// Extract image payloads with provenance instead of bare Strings.
    pub(super) fn collect_message_image_payloads(messages: &[Message]) -> Vec<ImageCacheRequest> {
        let mut images = Vec::new();
        for message in messages {
            for (attachment_index, attachment) in message.images.iter().enumerate() {
                images.push(ImageCacheRequest::new(
                    ImageCacheSource::ChatHistory,
                    format!("message:{}#{}", message.id, attachment_index),
                    attachment.data.clone(),
                ));
            }
        }
        images
    }

    /// Decode base64 to PNG bytes on a background thread.
    fn prepare_image_cache_work(request: ImageCacheRequest) -> Option<PreparedImageCacheWork> {
        let started_at = std::time::Instant::now();
        let cache_key = request.cache_key();
        let base64_len = request.base64_len();

        use base64::Engine;
        let png_bytes = match base64::engine::general_purpose::STANDARD.decode(&request.base64_data)
        {
            Ok(bytes) => bytes,
            Err(error) => {
                tracing::warn!(
                    category = "AI",
                    event = "ai_image_cache_prepare",
                    source = request.source.as_str(),
                    source_id = %request.source_id,
                    cache_key = %cache_key,
                    base64_len,
                    status = "decode_failed",
                    error = %error,
                    duration_ms = started_at.elapsed().as_millis() as u64,
                    "Failed to decode base64 image data"
                );
                return None;
            }
        };

        tracing::info!(
            category = "AI",
            event = "ai_image_cache_prepare",
            source = request.source.as_str(),
            source_id = %request.source_id,
            cache_key = %cache_key,
            base64_len,
            png_bytes = png_bytes.len(),
            status = "decoded",
            duration_ms = started_at.elapsed().as_millis() as u64,
            "Prepared image cache work"
        );

        Some(PreparedImageCacheWork {
            source: request.source,
            source_id: request.source_id,
            cache_key,
            base64_len,
            png_bytes,
        })
    }

    /// Insert a pre-decoded image into the cache. Returns true if inserted.
    fn insert_prepared_render_image(&mut self, work: PreparedImageCacheWork) -> bool {
        let started_at = std::time::Instant::now();

        if self.image_cache.contains_key(&work.cache_key) {
            tracing::debug!(
                category = "AI",
                event = "ai_image_cache_insert",
                source = work.source.as_str(),
                source_id = %work.source_id,
                cache_key = %work.cache_key,
                status = "skipped_cached",
                "Image already cached before insert"
            );
            return false;
        }

        match crate::list_item::decode_png_to_render_image_with_bgra_conversion(&work.png_bytes) {
            Ok(render_image) => {
                self.image_cache
                    .insert(work.cache_key.clone(), render_image);
                tracing::info!(
                    category = "AI",
                    event = "ai_image_cache_insert",
                    source = work.source.as_str(),
                    source_id = %work.source_id,
                    cache_key = %work.cache_key,
                    base64_len = work.base64_len,
                    png_bytes = work.png_bytes.len(),
                    status = "inserted",
                    duration_ms = started_at.elapsed().as_millis() as u64,
                    "Inserted prepared image into cache"
                );
                true
            }
            Err(error) => {
                tracing::warn!(
                    category = "AI",
                    event = "ai_image_cache_insert",
                    source = work.source.as_str(),
                    source_id = %work.source_id,
                    cache_key = %work.cache_key,
                    base64_len = work.base64_len,
                    png_bytes = work.png_bytes.len(),
                    status = "render_decode_failed",
                    error = %error,
                    duration_ms = started_at.elapsed().as_millis() as u64,
                    "Failed to convert PNG bytes into render image"
                );
                false
            }
        }
    }

    pub(super) fn defer_cache_pending_image(
        &mut self,
        image_base64: String,
        cx: &mut Context<Self>,
    ) {
        self.defer_cache_requests(
            vec![ImageCacheRequest::new(
                ImageCacheSource::PendingInput,
                "pending_input#0",
                image_base64,
            )],
            cx,
        );
    }

    pub(super) fn defer_cache_message_images(
        &mut self,
        requests: Vec<ImageCacheRequest>,
        cx: &mut Context<Self>,
    ) {
        self.defer_cache_requests(requests, cx);
    }

    fn defer_cache_requests(&mut self, requests: Vec<ImageCacheRequest>, cx: &mut Context<Self>) {
        use std::collections::HashSet;

        let mut queued_keys = HashSet::new();

        let requests: Vec<ImageCacheRequest> = requests
            .into_iter()
            .filter_map(|request| {
                let cache_key = request.cache_key();

                if self.image_cache.contains_key(&cache_key) {
                    tracing::debug!(
                        category = "AI",
                        event = "ai_image_cache_enqueue",
                        source = request.source.as_str(),
                        source_id = %request.source_id,
                        cache_key = %cache_key,
                        status = "skipped_cached",
                        "Image already cached"
                    );
                    return None;
                }

                if !queued_keys.insert(cache_key.clone()) {
                    tracing::debug!(
                        category = "AI",
                        event = "ai_image_cache_enqueue",
                        source = request.source.as_str(),
                        source_id = %request.source_id,
                        cache_key = %cache_key,
                        status = "skipped_duplicate_in_batch",
                        "Duplicate image request skipped in batch"
                    );
                    return None;
                }

                tracing::info!(
                    category = "AI",
                    event = "ai_image_cache_enqueue",
                    source = request.source.as_str(),
                    source_id = %request.source_id,
                    cache_key = %cache_key,
                    base64_len = request.base64_len(),
                    status = "queued",
                    "Queued image cache request"
                );

                Some(request)
            })
            .collect();

        if requests.is_empty() {
            return;
        }

        let request_count = requests.len();
        let (result_tx, result_rx) =
            async_channel::bounded::<PreparedImageCacheWork>(request_count);

        cx.background_executor()
            .spawn(async move {
                for request in requests {
                    if let Some(work) = AiApp::prepare_image_cache_work(request) {
                        let _ = result_tx.send(work).await;
                    }
                }
            })
            .detach();

        cx.spawn(async move |this, cx| {
            while let Ok(work) = result_rx.recv().await {
                if this
                    .update(cx, |this, cx| {
                        if this.insert_prepared_render_image(work) {
                            cx.notify();
                        }
                    })
                    .is_err()
                {
                    break;
                }
            }
        })
        .detach();
    }

    /// Handle file drop - if it's an image, set it as pending image
    pub(super) fn handle_file_drop(&mut self, paths: &ExternalPaths, cx: &mut Context<Self>) {
        let paths = paths.paths();
        if paths.is_empty() {
            return;
        }

        // Only handle the first file for now
        let path = &paths[0];
        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase())
            .unwrap_or_default();

        // Check if it's an image file
        let is_image = matches!(
            extension.as_str(),
            "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp"
        );

        if !is_image {
            info!("Dropped file is not an image: {:?}", path);
            return;
        }

        // Read and encode the file as base64
        match std::fs::read(path) {
            Ok(data) => {
                use crate::prompts::chat::MAX_IMAGE_BYTES;
                if data.len() > MAX_IMAGE_BYTES {
                    tracing::warn!(
                        path = ?path,
                        size_bytes = data.len(),
                        max_bytes = MAX_IMAGE_BYTES,
                        "Rejecting dropped image larger than 10 MB in AI window"
                    );
                    self.streaming_error =
                        Some("Dropped image exceeds 10 MB limit. Try a smaller image.".to_string());
                    cx.notify();
                    return;
                }
                use base64::Engine;
                let base64_data = base64::engine::general_purpose::STANDARD.encode(&data);
                self.cache_image_from_base64(&base64_data);
                self.pending_image = Some(base64_data);
                info!("Image file dropped and attached: {:?}", path);
                cx.notify();
            }
            Err(e) => {
                info!("Failed to read dropped image file: {:?} - {}", path, e);
            }
        }
    }

    /// Render the pending image preview with thumbnail
    pub(super) fn render_pending_image_preview(&self, cx: &mut Context<Self>) -> impl IntoElement {
        // Try to get the cached decoded image for a thumbnail
        let cached_thumbnail = self
            .pending_image
            .as_ref()
            .and_then(|b64| self.get_cached_image(b64));
        let has_thumbnail = cached_thumbnail.is_some();

        div().flex().items_center().gap_2().px_3().py_2().child(
            div()
                .id("pending-image-preview")
                .flex()
                .items_center()
                .gap_2()
                .px_2()
                .py_1()
                .rounded_md()
                .bg(cx.theme().muted.opacity(OPACITY_HOVER))
                .border_1()
                .border_color(cx.theme().accent.opacity(OPACITY_SELECTED))
                // Thumbnail or fallback icon
                .when_some(cached_thumbnail, |el, render_img| {
                    el.child(
                        div()
                            .size(IMG_PENDING_THUMB_SIZE)
                            .rounded(IMG_PENDING_THUMB_RADIUS)
                            .overflow_hidden()
                            .flex_shrink_0()
                            .child(
                                img(move |_window: &mut Window, _cx: &mut App| {
                                    Some(Ok(render_img.clone()))
                                })
                                .w(IMG_PENDING_THUMB_SIZE)
                                .h(IMG_PENDING_THUMB_SIZE)
                                .object_fit(gpui::ObjectFit::Cover),
                            ),
                    )
                })
                .when(!has_thumbnail, |el| {
                    el.child(
                        svg()
                            .external_path(LocalIconName::File.external_path())
                            .size(px(14.))
                            .text_color(cx.theme().accent),
                    )
                })
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().foreground)
                        .child("Image attached"),
                )
                // Remove button
                .child(
                    div()
                        .id("remove-image-btn")
                        .flex()
                        .items_center()
                        .justify_center()
                        .size(px(24.))
                        .rounded_full()
                        .cursor_pointer()
                        .hover(|s| s.bg(cx.theme().danger.opacity(OPACITY_HOVER)))
                        .on_mouse_down(
                            gpui::MouseButton::Left,
                            cx.listener(|this, _, _, cx| {
                                this.remove_pending_image(cx);
                            }),
                        )
                        .child(
                            svg()
                                .external_path(LocalIconName::Close.external_path())
                                .size(px(14.))
                                .text_color(cx.theme().muted_foreground),
                        ),
                ),
        )
    }

    /// Open a native file picker dialog and add selected files as pending attachments.
    pub(super) fn open_file_picker(&mut self, cx: &mut Context<Self>) {
        info!(
            action = "open_file_picker",
            "Opening native file picker for attachments"
        );

        let rx = cx.prompt_for_paths(gpui::PathPromptOptions {
            files: true,
            directories: false,
            multiple: true,
            prompt: Some("Select files to attach".into()),
            allowed_extensions: Vec::new(),
        });

        cx.spawn(async move |this, cx| match rx.await {
            Ok(Ok(Some(paths))) => {
                let count = paths.len();
                let path_strings: Vec<String> = paths
                    .into_iter()
                    .map(|p| p.to_string_lossy().to_string())
                    .collect();
                this.update(cx, |this, cx| {
                    for path in &path_strings {
                        this.add_attachment(path.clone(), cx);
                    }
                    info!(
                        action = "file_picker_completed",
                        files_added = count,
                        "Files attached via file picker"
                    );
                })
                .ok();
            }
            Ok(Ok(None)) => {
                info!(
                    action = "file_picker_cancelled",
                    "User cancelled file picker"
                );
            }
            Ok(Err(e)) => {
                tracing::warn!(error = %e, "File picker returned error");
            }
            Err(_) => {
                tracing::warn!("File picker channel closed unexpectedly");
            }
        })
        .detach();
    }

    /// Open a native file picker filtered to image types and set selected image as pending.
    ///
    /// Selected images are read from disk, base64-encoded, cached, and set as the
    /// pending image attachment. Only the last selected image is kept (single pending image).
    pub(super) fn open_image_picker(&mut self, cx: &mut Context<Self>) {
        info!(
            action = "open_image_picker",
            "Opening native image file picker"
        );

        let rx = cx.prompt_for_paths(gpui::PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: Some("Select an image to attach".into()),
            allowed_extensions: vec![
                "png".to_string(),
                "jpg".to_string(),
                "jpeg".to_string(),
                "gif".to_string(),
                "webp".to_string(),
                "bmp".to_string(),
            ],
        });

        cx.spawn(async move |this, cx| {
            match rx.await {
                Ok(Ok(Some(paths))) => {
                    if let Some(path) = paths.first() {
                        let path = path.clone();
                        // Read the file on the background executor to avoid blocking UI
                        let file_data = match std::fs::read(&path) {
                            Ok(data) => data,
                            Err(e) => {
                                tracing::warn!(
                                    error = %e,
                                    path = %path.display(),
                                    "Failed to read selected image file"
                                );
                                return;
                            }
                        };

                        use crate::prompts::chat::MAX_IMAGE_BYTES;
                        if file_data.len() > MAX_IMAGE_BYTES {
                            tracing::warn!(
                                path = %path.display(),
                                size_bytes = file_data.len(),
                                max_bytes = MAX_IMAGE_BYTES,
                                "Rejecting image picker file larger than 10 MB in AI window"
                            );
                            this.update(cx, |this, cx| {
                                this.streaming_error = Some(
                                    "Selected image exceeds 10 MB limit. Try a smaller image."
                                        .to_string(),
                                );
                                cx.notify();
                            })
                            .ok();
                            return;
                        }

                        use base64::Engine;
                        let base64_data =
                            base64::engine::general_purpose::STANDARD.encode(&file_data);

                        this.update(cx, |this, cx| {
                            this.cache_image_from_base64(&base64_data);
                            this.pending_image = Some(base64_data);
                            info!(
                                action = "image_picker_completed",
                                path = %path.display(),
                                size_kb = file_data.len() / 1024,
                                "Image attached via image picker"
                            );
                            cx.notify();
                        })
                        .ok();
                    }
                }
                Ok(Ok(None)) => {
                    info!(
                        action = "image_picker_cancelled",
                        "User cancelled image picker"
                    );
                }
                Ok(Err(e)) => {
                    tracing::warn!(error = %e, "Image picker returned error");
                }
                Err(_) => {
                    tracing::warn!("Image picker channel closed unexpectedly");
                }
            }
        })
        .detach();
    }

    /// Launch interactive screen area capture and attach the result as a pending image.
    ///
    /// Runs macOS `screencapture -i` on a background thread which shows its own native
    /// fullscreen overlay. On completion, the captured image is base64 encoded and set
    /// as the pending image attachment. Escape cancels the capture.
    pub(super) fn capture_screen_area_attachment(&mut self, cx: &mut Context<Self>) {
        info!(
            action = "capture_screen_area_start",
            "Starting screen area capture for AI attachment"
        );

        // Run capture on background executor (screencapture -i blocks waiting for user)
        cx.spawn(async move |this, cx| {
            let capture_result = cx
                .background_executor()
                .spawn(async { crate::platform::capture_screen_area() })
                .await;

            match capture_result {
                Ok(Some(capture)) => {
                    use crate::prompts::chat::MAX_IMAGE_BYTES;
                    if capture.png_data.len() > MAX_IMAGE_BYTES {
                        tracing::warn!(
                            size_bytes = capture.png_data.len(),
                            max_bytes = MAX_IMAGE_BYTES,
                            "Rejecting screen capture larger than 10 MB in AI window"
                        );
                        this.update(cx, |this, cx| {
                            this.streaming_error = Some(
                                "Screen capture exceeds 10 MB limit. Try capturing a smaller area."
                                    .to_string(),
                            );
                            cx.notify();
                        })
                        .ok();
                        return;
                    }

                    use base64::Engine;
                    let base64_data =
                        base64::engine::general_purpose::STANDARD.encode(&capture.png_data);
                    let size_kb = capture.png_data.len() / 1024;

                    this.update(cx, |this, cx| {
                        this.cache_image_from_base64(&base64_data);
                        this.pending_image = Some(base64_data);
                        info!(
                            action = "capture_screen_area_attached",
                            width = capture.width,
                            height = capture.height,
                            size_kb = size_kb,
                            "Screen area captured and attached to AI chat"
                        );
                        cx.notify();
                    })
                    .ok();
                }
                Ok(None) => {
                    info!(
                        action = "capture_screen_area_cancelled",
                        "User cancelled screen area capture"
                    );
                }
                Err(e) => {
                    tracing::warn!(
                        action = "capture_screen_area_error",
                        error = %e,
                        "Screen area capture failed"
                    );
                }
            }
        })
        .detach();
    }

    /// Focus the main chat input
    /// Called when the window is opened to allow immediate typing
    pub fn focus_input(&self, window: &mut Window, cx: &mut Context<Self>) {
        self.input_state.update(cx, |state, cx| {
            // Focus and ensure cursor is at the end of any existing text
            // For empty input, this puts cursor at position 0 with proper blinking
            let text_len = state.text().len();
            state.set_selection(text_len, text_len, window, cx);
        });
        info!("AI input focused for immediate typing");
    }

    /// Focus the search input in the sidebar (Cmd+Shift+F)
    pub(super) fn focus_search(&self, window: &mut Window, cx: &mut Context<Self>) {
        self.search_state.update(cx, |state, cx| {
            let text_len = state.text().len();
            state.set_selection(text_len, text_len, window, cx);
        });
        info!("AI search focused via Cmd+Shift+F");
    }

    /// Request focus on next render cycle.
    /// This is used when bringing an existing window to front - the caller
    /// sets this flag via window.update() and the flag is processed in render().
    /// This pattern avoids the need for a global Entity<AiApp> reference.
    pub fn request_focus(&mut self, cx: &mut Context<Self>) {
        self.needs_focus_input = true;
        cx.notify(); // Trigger re-render to process the flag
    }
}

/// Extract the query text after the last `@` in the input.
///
/// Returns `Some(query)` when the caret is positioned after an `@` trigger
/// (i.e. `@` is the last special trigger character in the text, and there is
/// no whitespace between `@` and the end of the current word).
/// Returns `None` if there is no `@` or if `@` is followed by a space.
fn extract_at_query(input: &str) -> Option<String> {
    let at_pos = input.rfind('@')?;
    let after_at = &input[at_pos + 1..];

    // If there's a space right after @, the mention is complete or cancelled
    if after_at.starts_with(' ') {
        return None;
    }

    // Extract the word after @ (up to the next space or end of string)
    let query = match after_at.find(char::is_whitespace) {
        Some(end) => &after_at[..end],
        None => after_at,
    };

    Some(query.to_string())
}

#[cfg(test)]
mod at_query_tests {
    use super::extract_at_query;

    #[test]
    fn extract_at_query_returns_none_for_no_at() {
        assert_eq!(extract_at_query("hello world"), None);
    }

    #[test]
    fn extract_at_query_returns_empty_string_for_bare_at() {
        assert_eq!(extract_at_query("hello @"), Some(String::new()));
    }

    #[test]
    fn extract_at_query_returns_query_after_at() {
        assert_eq!(extract_at_query("hello @sel"), Some("sel".to_string()));
    }

    #[test]
    fn extract_at_query_returns_none_when_at_followed_by_space() {
        assert_eq!(extract_at_query("hello @ world"), None);
    }

    #[test]
    fn extract_at_query_uses_last_at() {
        assert_eq!(
            extract_at_query("@context hello @sel"),
            Some("sel".to_string())
        );
    }

    #[test]
    fn extract_at_query_full_mention_no_trailing_space() {
        assert_eq!(
            extract_at_query("@selection"),
            Some("selection".to_string())
        );
    }
}
