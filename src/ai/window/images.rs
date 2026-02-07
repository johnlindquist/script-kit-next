use super::*;

impl AiApp {
    pub(super) fn on_input_change(&mut self, _cx: &mut Context<Self>) {
        // TODO: Handle input changes (e.g., streaming, auto-complete)
    }

    /// Handle paste event - check for clipboard images
    ///
    /// If clipboard contains an image, encode it as base64 and store as pending_image.
    /// If clipboard contains text, let the normal input handling process it.
    ///
    /// Returns true if an image was pasted (caller should not process text).
    pub(super) fn handle_paste_for_image(&mut self, cx: &mut Context<Self>) -> bool {
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

        div().flex().items_center().gap_2().px_3().py_1().child(
            div()
                .id("pending-image-preview")
                .flex()
                .items_center()
                .gap_2()
                .px_2()
                .py_1()
                .rounded_md()
                .bg(cx.theme().muted.opacity(0.3))
                .border_1()
                .border_color(cx.theme().accent.opacity(0.5))
                // Thumbnail or fallback icon
                .when_some(cached_thumbnail, |el, render_img| {
                    el.child(
                        div()
                            .size(px(36.))
                            .rounded(px(4.))
                            .overflow_hidden()
                            .flex_shrink_0()
                            .child(
                                img(move |_window: &mut Window, _cx: &mut App| {
                                    Some(Ok(render_img.clone()))
                                })
                                .w(px(36.))
                                .h(px(36.))
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
                        .size(px(16.))
                        .rounded_full()
                        .cursor_pointer()
                        .hover(|s| s.bg(cx.theme().danger.opacity(0.3)))
                        .on_mouse_down(
                            gpui::MouseButton::Left,
                            cx.listener(|this, _, _, cx| {
                                this.remove_pending_image(cx);
                            }),
                        )
                        .child(
                            svg()
                                .external_path(LocalIconName::Close.external_path())
                                .size(px(10.))
                                .text_color(cx.theme().muted_foreground),
                        ),
                ),
        )
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
