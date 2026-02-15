const FILE_SEARCH_PREVIEW_THUMBNAIL_MAX_BYTES: u64 = 20 * 1024 * 1024;
const FILE_SEARCH_PREVIEW_THUMBNAIL_MAX_DIMENSION: u32 = 8_000;
const FILE_SEARCH_PREVIEW_THUMBNAIL_MAX_SIDE_PX: f32 = 280.0;

#[derive(Debug)]
struct FileSearchThumbnailPreviewImage {
    image: Arc<gpui::RenderImage>,
    width: u32,
    height: u32,
}

#[derive(Debug)]
enum FileSearchThumbnailLoadFailure {
    FileTooLarge {
        size_bytes: u64,
    },
    ResolutionTooLarge {
        width: u32,
        height: u32,
        max_dimension: u32,
    },
    UnsupportedFormat,
    UnableToGenerate {
        reason: String,
    },
}

impl FileSearchThumbnailLoadFailure {
    fn preview_message(&self) -> String {
        match self {
            FileSearchThumbnailLoadFailure::FileTooLarge { size_bytes } => {
                let size_mb = (*size_bytes as f64) / (1024.0 * 1024.0);
                format!("File too large for thumbnail preview ({size_mb:.1} MB)")
            }
            FileSearchThumbnailLoadFailure::ResolutionTooLarge {
                width,
                height,
                max_dimension,
            } => {
                format!(
                    "Image resolution too large for preview ({}x{}, max {}x{})",
                    width, height, max_dimension, max_dimension
                )
            }
            FileSearchThumbnailLoadFailure::UnsupportedFormat => {
                "Preview not available for this format".to_string()
            }
            FileSearchThumbnailLoadFailure::UnableToGenerate { reason } => {
                format!("Unable to generate preview: {reason}")
            }
        }
    }
}

fn file_search_thumbnail_extension(path: &str) -> Option<String> {
    std::path::Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(str::to_ascii_lowercase)
}

fn file_search_thumbnail_is_decodable_extension(path: &str) -> bool {
    matches!(
        file_search_thumbnail_extension(path).as_deref(),
        Some("png" | "jpg" | "jpeg" | "gif" | "bmp" | "webp" | "tiff" | "tif" | "ico")
    )
}

fn file_search_thumbnail_display_size(width: u32, height: u32, max_side_px: f32) -> (f32, f32) {
    let width_f = width as f32;
    let height_f = height as f32;

    if width_f <= 0.0 || height_f <= 0.0 {
        return (max_side_px, max_side_px);
    }

    let scale = (max_side_px / width_f).min(max_side_px / height_f).min(1.0);
    (width_f * scale, height_f * scale)
}

fn load_file_search_thumbnail_preview(
    path: &str,
    max_bytes: u64,
    max_dimension: u32,
) -> Result<FileSearchThumbnailPreviewImage, FileSearchThumbnailLoadFailure> {
    use anyhow::Context as _;
    use image::GenericImageView as _;

    if !file_search_thumbnail_is_decodable_extension(path) {
        return Err(FileSearchThumbnailLoadFailure::UnsupportedFormat);
    }

    let metadata = std::fs::metadata(path)
        .with_context(|| format!("failed to read metadata for '{}'", path))
        .map_err(|error| FileSearchThumbnailLoadFailure::UnableToGenerate {
            reason: error.to_string(),
        })?;

    let size_bytes = metadata.len();
    if size_bytes > max_bytes {
        return Err(FileSearchThumbnailLoadFailure::FileTooLarge { size_bytes });
    }

    let image_bytes = std::fs::read(path)
        .with_context(|| format!("failed to read image bytes for '{}'", path))
        .map_err(|error| FileSearchThumbnailLoadFailure::UnableToGenerate {
            reason: error.to_string(),
        })?;

    let decoded_image = match image::load_from_memory(&image_bytes) {
        Ok(image) => image,
        Err(image_error) => {
            if matches!(image_error, image::ImageError::Unsupported(_)) {
                return Err(FileSearchThumbnailLoadFailure::UnsupportedFormat);
            }

            let reason = anyhow::Error::new(image_error)
                .context(format!("failed to decode image data for '{}'", path))
                .to_string();
            return Err(FileSearchThumbnailLoadFailure::UnableToGenerate { reason });
        }
    };

    let (width, height) = decoded_image.dimensions();
    if width > max_dimension || height > max_dimension {
        return Err(FileSearchThumbnailLoadFailure::ResolutionTooLarge {
            width,
            height,
            max_dimension,
        });
    }

    let mut bgra = decoded_image.to_rgba8();
    for pixel in bgra.chunks_exact_mut(4) {
        pixel.swap(0, 2);
    }

    let frame = image::Frame::new(bgra);
    let render_image = gpui::RenderImage::new(smallvec::smallvec![frame]);

    Ok(FileSearchThumbnailPreviewImage {
        image: Arc::new(render_image),
        width,
        height,
    })
}

impl ScriptListApp {
    fn ensure_file_search_preview_thumbnail(
        &mut self,
        selected_file: Option<&file_search::FileResult>,
        cx: &mut Context<Self>,
    ) {
        let thumbnail_path = selected_file
            .filter(|file| file_search::is_thumbnail_preview_supported(&file.path))
            .map(|file| file.path.clone());

        let Some(path) = thumbnail_path else {
            if !matches!(
                self.file_search_preview_thumbnail,
                FileSearchThumbnailPreviewState::Idle
            ) {
                tracing::debug!("file_search_thumbnail_preview_state_transition: idle");
                self.file_search_preview_thumbnail = FileSearchThumbnailPreviewState::Idle;
                cx.notify();
            }
            return;
        };

        let already_loaded_for_path = match &self.file_search_preview_thumbnail {
            FileSearchThumbnailPreviewState::Loading { path: current_path }
            | FileSearchThumbnailPreviewState::Ready {
                path: current_path, ..
            }
            | FileSearchThumbnailPreviewState::Unavailable {
                path: current_path, ..
            } => current_path == &path,
            FileSearchThumbnailPreviewState::Idle => false,
        };

        if already_loaded_for_path {
            return;
        }

        tracing::debug!(
            path = %path,
            "file_search_thumbnail_preview_state_transition: loading"
        );
        self.file_search_preview_thumbnail = FileSearchThumbnailPreviewState::Loading {
            path: path.clone(),
        };
        cx.notify();

        cx.spawn(async move |this, cx| {
            let (tx, rx) = std::sync::mpsc::channel();
            let path_for_decode = path.clone();
            std::thread::spawn(move || {
                let result = load_file_search_thumbnail_preview(
                    &path_for_decode,
                    FILE_SEARCH_PREVIEW_THUMBNAIL_MAX_BYTES,
                    FILE_SEARCH_PREVIEW_THUMBNAIL_MAX_DIMENSION,
                );
                let _ = tx.send(result);
            });

            let decode_result = loop {
                Timer::after(std::time::Duration::from_millis(16)).await;
                match rx.try_recv() {
                    Ok(result) => break result,
                    Err(std::sync::mpsc::TryRecvError::Empty) => {}
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        break Err(FileSearchThumbnailLoadFailure::UnableToGenerate {
                            reason: "thumbnail worker disconnected".to_string(),
                        });
                    }
                }
            };

            let _ = cx.update(|cx| {
                this.update(cx, |app, cx| {
                    let is_still_current_request = matches!(
                        &app.file_search_preview_thumbnail,
                        FileSearchThumbnailPreviewState::Loading { path: current_path }
                            if current_path == &path
                    );

                    if !is_still_current_request {
                        tracing::debug!(
                            path = %path,
                            "file_search_thumbnail_preview_stale_result_ignored"
                        );
                        return;
                    }

                    match decode_result {
                        Ok(loaded) => {
                            tracing::debug!(
                                path = %path,
                                width = loaded.width,
                                height = loaded.height,
                                "file_search_thumbnail_preview_state_transition: ready"
                            );
                            app.file_search_preview_thumbnail =
                                FileSearchThumbnailPreviewState::Ready {
                                    path: path.clone(),
                                    image: loaded.image,
                                    width: loaded.width,
                                    height: loaded.height,
                                };
                        }
                        Err(error) => {
                            let message = error.preview_message();
                            tracing::warn!(
                                path = %path,
                                reason = %message,
                                "file_search_thumbnail_preview_state_transition: unavailable"
                            );
                            app.file_search_preview_thumbnail =
                                FileSearchThumbnailPreviewState::Unavailable {
                                    path: path.clone(),
                                    message,
                                };
                        }
                    }

                    cx.notify();
                })
            });
        })
        .detach();
    }

    /// Render file search view with 50/50 split (list + preview)
    pub(crate) fn render_file_search(
        &mut self,
        query: &str,
        selected_index: usize,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        use crate::file_search::{self, FileType};

        // Use design tokens for spacing/visual, theme for colors
        let tokens = get_tokens(self.current_design);
        let design_spacing = tokens.spacing();
        let _design_typography = tokens.typography();
        let design_visual = tokens.visual();

        let _opacity = self.theme.get_opacity();
        // bg_with_alpha removed - let vibrancy show through from Root (matches main menu)
        // Removed: box_shadows - shadows on transparent elements block vibrancy
        let _box_shadows = self.create_box_shadows();

        // Color values for use in closures
        let text_primary = self.theme.colors.text.primary;
        let text_muted = self.theme.colors.text.muted;
        let text_dimmed = self.theme.colors.text.dimmed;
        let ui_border = self.theme.colors.ui.border;
        let _accent_color = self.theme.colors.accent.selected;
        let list_hover = self.theme.colors.accent.selected_subtle;
        let list_selected = self.theme.colors.accent.selected_subtle;
        // Use theme opacity for vibrancy-compatible selection/hover (matches main menu)
        let opacity = self.theme.get_opacity();
        let selected_alpha = (opacity.selected * 255.0) as u32;
        let hover_alpha = (opacity.hover * 255.0) as u32;

        // Get selected file for preview (if any)
        // Use display indices to map visible index -> actual result index.
        // Compute this before borrowing display_indices for the rest of render,
        // so we can safely call ensure_file_search_preview_thumbnail(&mut self, ...).
        let selected_file = self
            .file_search_display_indices
            .get(selected_index)
            .and_then(|&result_idx| self.cached_file_results.get(result_idx))
            .cloned();
        self.ensure_file_search_preview_thumbnail(selected_file.as_ref(), cx);

        // Use pre-computed display indices instead of running Nucleo in render
        // This is CRITICAL for animation performance - render must be cheap
        // The display_indices are computed in recompute_file_search_display_indices()
        // which is called when:
        // 1. Results change (new directory listing or search results)
        // 2. Filter pattern changes (user types in existing directory)
        // 3. Loading completes
        let display_indices = &self.file_search_display_indices;
        let filtered_len = display_indices.len();

        // Log render state (throttled - only when state changes meaningfully)
        static LAST_LOG: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        let last = LAST_LOG.load(std::sync::atomic::Ordering::Relaxed);
        if now_ms.saturating_sub(last) > 500 {
            // Log at most every 500ms
            LAST_LOG.store(now_ms, std::sync::atomic::Ordering::Relaxed);
            logging::log(
                "SEARCH",
                &format!(
                    "render_file_search: query='{}' loading={} cached={} display={} selected={}",
                    query,
                    self.file_search_loading,
                    self.cached_file_results.len(),
                    filtered_len,
                    selected_index
                ),
            );
        }

        // Key handler for file search
        let handle_key = cx.listener(
            move |this: &mut Self,
                  event: &gpui::KeyDownEvent,
                  window: &mut Window,
                  cx: &mut Context<Self>| {
                // Hide cursor while typing - automatically shows when mouse moves
                this.hide_mouse_cursor(cx);

                // If the shortcut recorder is active, don't process any key events.
                // The recorder has its own key handlers and should receive all key events.
                if this.shortcut_recorder_state.is_some() {
                    return;
                }

                let key_str = event.keystroke.key.to_lowercase();
                let key_char = event.keystroke.key_char.as_deref();
                let has_cmd = event.keystroke.modifiers.platform;

                let modifiers = &event.keystroke.modifiers;

                // Route keys to actions dialog first if it's open
                match this.route_key_to_actions_dialog(
                    &key_str,
                    key_char,
                    modifiers,
                    ActionsDialogHost::FileSearch,
                    window,
                    cx,
                ) {
                    ActionsRoute::NotHandled => {
                        // Actions dialog not open - continue to file search key handling
                    }
                    ActionsRoute::Handled => {
                        // Key was consumed by actions dialog
                        return;
                    }
                    ActionsRoute::Execute { action_id } => {
                        // User selected an action - execute it
                        // Use handle_action instead of trigger_action_by_name to support
                        // both built-in actions (open_file, quick_look, etc.) and SDK actions
                        this.handle_action(action_id, cx);
                        return;
                    }
                }

                // ESC: Clear query first if present, otherwise go back/close
                if key_str == "escape" {
                    if !this.clear_builtin_view_filter(cx) {
                        this.go_back_or_close(window, cx);
                    }
                    return;
                }

                // Cmd+W closes window
                if has_cmd && key_str == "w" {
                    logging::log("KEY", "Cmd+W - closing window");
                    this.close_and_reset_window(cx);
                    return;
                }

                if let AppView::FileSearchView {
                    query: _,
                    selected_index,
                } = &mut this.current_view
                {
                    // Use pre-computed display_indices to get the selected file
                    // This is consistent with what render shows and avoids re-running Nucleo
                    let get_selected_file = || {
                        this.file_search_display_indices
                            .get(*selected_index)
                            .and_then(|&idx| this.cached_file_results.get(idx))
                            .cloned()
                    };

                    match key_str.as_str() {
                        // Arrow keys are handled by arrow_interceptor in app_impl.rs
                        // which calls stop_propagation(). This is the single source of truth
                        // for arrow key handling in FileSearchView.
                        "up" | "arrowup" | "down" | "arrowdown" => {
                            // Already handled by interceptor, no-op here
                        }
                        // Tab/Shift+Tab handled by intercept_keystrokes in app_impl.rs
                        // (interceptor fires BEFORE input component can capture Tab)
                        "enter" | "return" => {
                            // Check for Cmd+Enter (reveal in finder) first
                            if has_cmd {
                                if let Some(file) = get_selected_file() {
                                    let _ = file_search::reveal_in_finder(&file.path);
                                }
                            } else {
                                // Open file with default app
                                if let Some(file) = get_selected_file() {
                                    let _ = file_search::open_file(&file.path);
                                    // Close window after opening file
                                    this.close_and_reset_window(cx);
                                }
                            }
                        }
                        _ => {
                            // Handle Cmd+K (toggle actions)
                            if has_cmd && key_str == "k" {
                                if let Some(file) = get_selected_file() {
                                    this.toggle_file_search_actions(&file, window, cx);
                                }
                                return;
                            }
                            // Handle Cmd+Y (Quick Look) - macOS only
                            #[cfg(target_os = "macos")]
                            if has_cmd && key_str == "y" {
                                if let Some(file) = get_selected_file() {
                                    let _ = file_search::quick_look(&file.path);
                                }
                                return;
                            }
                            // Handle Cmd+I (Show Info) - macOS only
                            #[cfg(target_os = "macos")]
                            if has_cmd && key_str == "i" {
                                if let Some(file) = get_selected_file() {
                                    let _ = file_search::show_info(&file.path);
                                }
                            }
                            // Handle Cmd+O (Open With) - macOS only
                            #[cfg(target_os = "macos")]
                            if has_cmd && key_str == "o" {
                                if let Some(file) = get_selected_file() {
                                    let _ = file_search::open_with(&file.path);
                                }
                            }
                        }
                    }
                }
            },
        );

        // Clone data for the uniform_list closure
        // Use display_indices to get files in the correct order (filtered + sorted)
        // Include the original result index for animation timestamp lookup
        let files_for_closure: Vec<(usize, file_search::FileResult)> = display_indices
            .iter()
            .filter_map(|&idx| self.cached_file_results.get(idx).map(|f| (idx, f.clone())))
            .collect();
        let current_selected = selected_index;
        let file_hovered = self.hovered_index;
        let file_input_mode = self.input_mode;
        let is_loading = self.file_search_loading;
        let click_entity_handle = cx.entity().downgrade();
        let hover_entity_handle = cx.entity().downgrade();

        // Use uniform_list for virtualized scrolling
        // Skeleton loading: show placeholder rows while loading and no results yet
        let list_element = if is_loading && filtered_len == 0 {
            // Loading with no results yet - show static skeleton rows
            let skeleton_bg = rgba((ui_border << 8) | 0x30); // ~18% opacity

            // Render 6 skeleton rows
            div()
                .w_full()
                .h_full()
                .flex()
                .flex_col()
                .children((0..6).map(|ix| {
                    div()
                        .id(ix)
                        .w_full()
                        .h(px(52.))
                        .flex()
                        .flex_row()
                        .items_center()
                        .px(px(12.))
                        .gap(px(12.))
                        // Icon placeholder
                        .child(div().w(px(24.)).h(px(24.)).rounded(px(6.)).bg(skeleton_bg))
                        // Text placeholders
                        .child(
                            div()
                                .flex_1()
                                .flex()
                                .flex_col()
                                .gap(px(6.))
                                .child(div().w(px(160.)).h(px(12.)).rounded(px(4.)).bg(skeleton_bg))
                                .child(
                                    div().w(px(240.)).h(px(10.)).rounded(px(4.)).bg(skeleton_bg),
                                ),
                        )
                        // Right side placeholders (size/time)
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .items_end()
                                .gap(px(6.))
                                .child(div().w(px(56.)).h(px(10.)).rounded(px(4.)).bg(skeleton_bg))
                                .child(div().w(px(72.)).h(px(10.)).rounded(px(4.)).bg(skeleton_bg)),
                        )
                }))
                .into_any_element()
        } else if filtered_len == 0 {
            // No results and not loading - show empty state message
            div()
                .w_full()
                .py(px(design_spacing.padding_xl))
                .text_center()
                .text_color(rgb(text_dimmed))
                .child(if query.is_empty() {
                    "Type to search files"
                } else {
                    "No files found"
                })
                .into_any_element()
        } else {
            uniform_list(
                "file-search-list",
                filtered_len,
                move |visible_range, _window, _cx| {
                    visible_range
                        .map(|ix| {
                            if let Some((_result_idx, file)) = files_for_closure.get(ix) {
                                let is_selected = ix == current_selected;
                                let is_hovered = !is_selected && file_hovered == Some(ix) && file_input_mode == InputMode::Mouse;

                                // Use theme opacity for vibrancy-compatible selection
                                let bg = if is_selected {
                                    rgba((list_selected << 8) | selected_alpha)
                                } else if is_hovered {
                                    rgba((list_hover << 8) | hover_alpha)
                                } else {
                                    gpui::transparent_black().into()
                                };
                                let hover_bg = rgba((list_hover << 8) | hover_alpha);
                                let is_mouse_mode = file_input_mode == InputMode::Mouse;
                                let show_thumbnail =
                                    file_search::is_thumbnail_preview_supported(&file.path);
                                let thumbnail_path = file.path.clone();
                                let fallback_icon =
                                    file_search::file_type_icon(file.file_type).to_string();

                                // Click handler: select on click, open file on double-click
                                let click_entity = click_entity_handle.clone();
                                let file_path = file.path.clone();
                                let click_handler = move |event: &gpui::ClickEvent,
                                                           _window: &mut Window,
                                                           cx: &mut gpui::App| {
                                    if let Some(app) = click_entity.upgrade() {
                                        let file_path = file_path.clone();
                                        app.update(cx, |this, cx| {
                                            if let AppView::FileSearchView {
                                                selected_index, ..
                                            } = &mut this.current_view
                                            {
                                                *selected_index = ix;
                                            }
                                            cx.notify();

                                            // Double-click: open file
                                            if let gpui::ClickEvent::Mouse(mouse_event) = event {
                                                if mouse_event.down.click_count == 2 {
                                                    logging::log(
                                                        "UI",
                                                        &format!(
                                                            "Double-click opening file: {}",
                                                            file_path
                                                        ),
                                                    );
                                                    let _ = file_search::open_file(&file_path);
                                                    this.close_and_reset_window(cx);
                                                }
                                            }
                                        });
                                    }
                                };

                                // Hover handler for mouse tracking
                                let hover_entity = hover_entity_handle.clone();
                                let hover_handler = move |hov: &bool, _window: &mut Window, cx: &mut gpui::App| {
                                    if let Some(app) = hover_entity.upgrade() {
                                        app.update(cx, |this, cx| {
                                            if *hov {
                                                this.input_mode = InputMode::Mouse;
                                                if this.hovered_index != Some(ix) {
                                                    this.hovered_index = Some(ix);
                                                    cx.notify();
                                                }
                                            } else if this.hovered_index == Some(ix) {
                                                this.hovered_index = None;
                                                cx.notify();
                                            }
                                        });
                                    }
                                };

                                div()
                                    .id(ix)
                                    .w_full()
                                    .h(px(52.))
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .px(px(12.))
                                    .gap(px(12.))
                                    .bg(bg)
                                    .cursor_pointer()
                                    .when(is_mouse_mode, |d| d.hover(move |s| s.bg(hover_bg)))
                                    .on_click(click_handler)
                                    .on_hover(hover_handler)
                                    .child(if show_thumbnail {
                                        let fallback_icon = fallback_icon.clone();
                                        div()
                                            .w(px(32.))
                                            .h(px(32.))
                                            .rounded(px(6.))
                                            .overflow_hidden()
                                            .bg(rgba((ui_border << 8) | 0x24))
                                            .flex_shrink_0()
                                            .child(
                                                gpui::img(std::path::PathBuf::from(
                                                    thumbnail_path,
                                                ))
                                                .w_full()
                                                .h_full()
                                                .object_fit(gpui::ObjectFit::Cover)
                                                .with_fallback(move || {
                                                    div()
                                                        .w_full()
                                                        .h_full()
                                                        .flex()
                                                        .items_center()
                                                        .justify_center()
                                                        .text_sm()
                                                        .text_color(rgb(text_muted))
                                                        .child(fallback_icon.clone())
                                                        .into_any_element()
                                                }),
                                            )
                                    } else {
                                        div()
                                            .w(px(32.))
                                            .h(px(32.))
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .text_lg()
                                            .text_color(rgb(text_muted))
                                            .flex_shrink_0()
                                            .child(file_search::file_type_icon(file.file_type))
                                    })
                                    .child(
                                        div()
                                            .flex_1()
                                            .flex()
                                            .flex_col()
                                            .gap(px(2.))
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .text_color(rgb(text_primary))
                                                    .child(file.name.clone()),
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(rgb(text_dimmed))
                                                    .child(file_search::shorten_path(&file.path)),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .items_end()
                                            .gap(px(2.))
                                            .child(
                                                div().text_xs().text_color(rgb(text_dimmed)).child(
                                                    file_search::format_file_size(file.size),
                                                ),
                                            )
                                            .child(
                                                div().text_xs().text_color(rgb(text_dimmed)).child(
                                                    file_search::format_relative_time(
                                                        file.modified,
                                                    ),
                                                ),
                                            ),
                                    )
                            } else {
                                div().id(ix).h(px(52.))
                            }
                        })
                        .collect()
                },
            )
            .h_full()
            .track_scroll(&self.file_search_scroll_handle)
            .into_any_element()
        };

        // Build preview panel content - matching main menu labeled section pattern
        let preview_content = if let Some(file) = &selected_file {
            let file_type_str = match file.file_type {
                FileType::Directory => "Folder",
                FileType::Image => "Image",
                FileType::Audio => "Audio",
                FileType::Video => "Video",
                FileType::Document => "Document",
                FileType::Application => "Application",
                FileType::File => "File",
                FileType::Other => "File",
            };
            let preview_supports_thumbnail =
                file_search::is_thumbnail_preview_supported(&file.path);
            let thumbnail_section = if preview_supports_thumbnail {
                let preview_body = match &self.file_search_preview_thumbnail {
                    FileSearchThumbnailPreviewState::Ready {
                        path,
                        image,
                        width,
                        height,
                    } if path == &file.path => {
                        let (display_width, display_height) = file_search_thumbnail_display_size(
                            *width,
                            *height,
                            FILE_SEARCH_PREVIEW_THUMBNAIL_MAX_SIDE_PX,
                        );
                        let image_for_render = image.clone();
                        div()
                            .w_full()
                            .flex()
                            .flex_col()
                            .items_center()
                            .justify_center()
                            .gap(px(design_spacing.gap_sm))
                            .child(
                                gpui::img(move |_window: &mut Window, _cx: &mut App| {
                                    Some(Ok(image_for_render.clone()))
                                })
                                .w(px(display_width))
                                .h(px(display_height))
                                .object_fit(gpui::ObjectFit::Contain)
                                .rounded(px(design_visual.radius_sm)),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(text_muted))
                                    .child(format!("{}×{} px", width, height)),
                            )
                            .into_any_element()
                    }
                    FileSearchThumbnailPreviewState::Unavailable { path, message }
                        if path == &file.path =>
                    {
                        div()
                            .w_full()
                            .text_sm()
                            .text_color(rgb(text_dimmed))
                            .child(message.clone())
                            .into_any_element()
                    }
                    FileSearchThumbnailPreviewState::Loading { path } if path == &file.path => {
                        div()
                            .w_full()
                            .text_sm()
                            .text_color(rgb(text_dimmed))
                            .child("Loading thumbnail...")
                            .into_any_element()
                    }
                    _ => div()
                        .w_full()
                        .text_sm()
                        .text_color(rgb(text_dimmed))
                        .child("Loading thumbnail...")
                        .into_any_element(),
                };

                Some(
                    div()
                        .flex()
                        .flex_col()
                        .pb(px(design_spacing.padding_md))
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(text_muted))
                                .pb(px(design_spacing.padding_xs / 2.0))
                                .child("Preview"),
                        )
                        .child(
                            div()
                                .w_full()
                                .min_h(px(FILE_SEARCH_PREVIEW_THUMBNAIL_MAX_SIDE_PX + 24.0))
                                .p(px(design_spacing.padding_sm))
                                .rounded(px(design_visual.radius_md))
                                .bg(rgba((ui_border << 8) | 0x24))
                                .flex()
                                .items_center()
                                .justify_center()
                                .overflow_hidden()
                                .child(preview_body),
                        ),
                )
            } else {
                None
            };

            div()
                .flex_1()
                .flex()
                .flex_col()
                .p(px(design_spacing.padding_lg))
                .gap(px(design_spacing.gap_md))
                .overflow_y_hidden()
                .when_some(thumbnail_section, |container, section| container.child(section))
                // Name section (labeled like main menu)
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .pb(px(design_spacing.padding_md))
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(text_muted))
                                .pb(px(design_spacing.padding_xs / 2.0))
                                .child("Name"),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap(px(design_spacing.gap_sm))
                                .child(
                                    div()
                                        .text_lg()
                                        .font_weight(gpui::FontWeight::SEMIBOLD)
                                        .text_color(rgb(text_primary))
                                        .child(file.name.clone()),
                                )
                                .child(
                                    div()
                                        .px(px(6.))
                                        .py(px(2.))
                                        .rounded(px(4.))
                                        .bg(rgba((ui_border << 8) | 0x40))
                                        .text_xs()
                                        .text_color(rgb(text_muted))
                                        .child(file_type_str),
                                ),
                        ),
                )
                // Path section (labeled)
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .pb(px(design_spacing.padding_md))
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(text_muted))
                                .pb(px(design_spacing.padding_xs / 2.0))
                                .child("Path"),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(rgb(text_dimmed))
                                .child(file.path.clone()),
                        ),
                )
                // Divider (like main menu)
                .child(
                    div()
                        .w_full()
                        .h(px(design_visual.border_thin))
                        .bg(rgba((ui_border << 8) | 0x60))
                        .my(px(design_spacing.padding_sm)),
                )
                // Details section (labeled)
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(text_muted))
                                .pb(px(design_spacing.padding_xs / 2.0))
                                .child("Details"),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(design_spacing.gap_sm))
                                .child(div().text_sm().text_color(rgb(text_dimmed)).child(format!(
                                    "Size: {}",
                                    file_search::format_file_size(file.size)
                                )))
                                .child(div().text_sm().text_color(rgb(text_dimmed)).child(format!(
                                    "Modified: {}",
                                    file_search::format_relative_time(file.modified)
                                )))
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(rgb(text_dimmed))
                                        .child(format!("Type: {}", file_type_str)),
                                ),
                        ),
                )
        } else if is_loading {
            // When loading, show empty preview (no distracting message)
            div().flex_1()
        } else {
            div().flex_1().flex().items_center().justify_center().child(
                div()
                    .text_sm()
                    .text_color(rgb(text_dimmed))
                    .child("No file selected"),
            )
        };

        // Main container - styled to match main menu exactly
        // NOTE: No border to match main menu (border adds visual padding/shift)
        div()
            .key_context("FileSearchView")
            .track_focus(&self.focus_handle)
            .on_key_down(handle_key)
            .w_full()
            .h_full()
            .flex()
            .flex_col()
            // Removed: .bg(rgba(bg_with_alpha)) - let vibrancy show through from Root
            // Removed: .shadow(box_shadows) - shadows on transparent elements block vibrancy
            .rounded(px(design_visual.radius_lg))
            // Header with search input - styled to match main menu exactly
            // Uses shared header constants (HEADER_PADDING_X/Y, CURSOR_HEIGHT_LG) for visual consistency.
            // The right-side element uses same py(4px) padding as main menu's "Ask AI" button
            // to ensure identical flex row height (28px) and input vertical centering.
            .child({
                // Calculate input height using same formula as main menu
                let input_height = CURSOR_HEIGHT_LG + (CURSOR_MARGIN_Y * 2.0);

                div()
                    .w_full()
                    .px(px(HEADER_PADDING_X))
                    .py(px(HEADER_PADDING_Y))
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(HEADER_GAP))
                    // Search input - matches main menu Input styling for visual consistency
                    // NOTE: Removed search icon to match main menu alignment exactly
                    .child(
                        div().flex_1().flex().flex_row().items_center().child(
                            Input::new(&self.gpui_input_state)
                                .w_full()
                                .h(px(input_height))
                                .px(px(0.))
                                .py(px(0.))
                                .with_size(Size::Size(px(_design_typography.font_size_xl)))
                                .appearance(false)
                                .bordered(false)
                                .focus_bordered(false),
                        ),
                    )
                    // Right-side element styled to match main menu's "Ask AI" button height
                    // Using fixed width to prevent layout shift when content changes
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .justify_end()
                            .py(px(4.))
                            .w(px(70.)) // Fixed width prevents layout shift
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(text_dimmed))
                                    .child(format!("{} files", filtered_len)),
                            ),
                    )
            })
            // Divider
            .child(
                div()
                    .mx(px(design_spacing.padding_lg))
                    .h(px(design_visual.border_thin))
                    .bg(rgba((ui_border << 8) | 0x60)),
            )
            // Main content: loading state OR empty state OR 50/50 split
            .child(if is_loading && filtered_len == 0 {
                // Loading state: full-width centered (no split, clean appearance)
                div()
                    .flex_1()
                    .w_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .min_h(px(0.))
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(text_dimmed))
                            .child("Searching..."),
                    )
            } else if filtered_len == 0 {
                // Empty state: single centered message (no awkward 50/50 split)
                div()
                    .flex_1()
                    .w_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .min_h(px(0.))
                    .child(
                        div().flex().flex_col().items_center().gap(px(8.)).child(
                            div()
                                .text_color(rgb(text_dimmed))
                                .child(if query.is_empty() {
                                    "Type to search files"
                                } else {
                                    "No files found"
                                }),
                        ),
                    )
            } else {
                // Normal state: 50/50 split with list and preview
                div()
                    .flex_1()
                    .w_full()
                    .flex()
                    .flex_row()
                    .min_h(px(0.))
                    .overflow_hidden()
                    // Left panel: file list (50%)
                    .child(
                        div()
                            .flex_1()
                            .h_full()
                            .overflow_hidden()
                            .border_r(px(design_visual.border_thin))
                            .border_color(rgba((ui_border << 8) | 0x40))
                            .child(list_element),
                    )
                    // Right panel: preview (50%)
                    .child(
                        div()
                            .flex_1()
                            .h_full()
                            .overflow_hidden()
                            .child(preview_content),
                    )
            })
            // Footer
            .child(PromptFooter::new(
                PromptFooterConfig::new()
                    .primary_label("Open")
                    .primary_shortcut("↵"),
                // Default config already has secondary_label="Actions", secondary_shortcut="⌘K", show_secondary=true
                PromptFooterColors::from_theme(&self.theme),
            ))
            .into_any_element()

    }
}

#[cfg(test)]
mod file_search_thumbnail_tests {
    use super::*;
    use image::{Rgba, RgbaImage};
    use tempfile::tempdir;

    #[test]
    fn test_file_search_thumbnail_display_size_scales_longest_side_when_over_limit() {
        let (width, height) = file_search_thumbnail_display_size(4000, 1000, 280.0);
        assert!((width - 280.0).abs() < f32::EPSILON);
        assert!((height - 70.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_file_search_thumbnail_is_decodable_extension_matches_supported_decoder_inputs() {
        assert!(file_search_thumbnail_is_decodable_extension("/tmp/sample.png"));
        assert!(file_search_thumbnail_is_decodable_extension("/tmp/sample.JPG"));
        assert!(file_search_thumbnail_is_decodable_extension("/tmp/sample.webp"));
        assert!(file_search_thumbnail_is_decodable_extension("/tmp/sample.tiff"));
        assert!(file_search_thumbnail_is_decodable_extension("/tmp/sample.ico"));
        assert!(!file_search_thumbnail_is_decodable_extension("/tmp/sample.svg"));
    }

    #[test]
    fn test_load_file_search_thumbnail_preview_returns_file_too_large_when_size_exceeds_limit() {
        let temp_dir = tempdir().expect("tempdir should be created");
        let image_path = temp_dir.path().join("too-big.png");
        std::fs::write(&image_path, vec![0_u8; 128]).expect("image bytes should be written");

        let result = load_file_search_thumbnail_preview(
            image_path
                .to_str()
                .expect("temp image path should be valid utf-8"),
            32,
            FILE_SEARCH_PREVIEW_THUMBNAIL_MAX_DIMENSION,
        );

        match result {
            Err(FileSearchThumbnailLoadFailure::FileTooLarge { size_bytes }) => {
                assert_eq!(size_bytes, 128);
            }
            other => panic!("Expected FileTooLarge error, got {:?}", other),
        }
    }

    #[test]
    fn test_load_file_search_thumbnail_preview_returns_resolution_too_large_when_dimension_exceeds_limit(
    ) {
        let temp_dir = tempdir().expect("tempdir should be created");
        let image_path = temp_dir.path().join("oversized.png");
        let img = RgbaImage::from_pixel(2, 2, Rgba([255, 0, 0, 255]));
        img.save(&image_path).expect("test image should be written");

        let result = load_file_search_thumbnail_preview(
            image_path
                .to_str()
                .expect("temp image path should be valid utf-8"),
            FILE_SEARCH_PREVIEW_THUMBNAIL_MAX_BYTES,
            1,
        );

        match result {
            Err(FileSearchThumbnailLoadFailure::ResolutionTooLarge {
                width,
                height,
                max_dimension,
            }) => {
                assert_eq!(width, 2);
                assert_eq!(height, 2);
                assert_eq!(max_dimension, 1);
            }
            other => panic!("Expected ResolutionTooLarge error, got {:?}", other),
        }
    }
}
