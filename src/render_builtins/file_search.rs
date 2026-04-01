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

/// Shared helper for file-search native drag.  Initiates the macOS drag
/// session and immediately clears GPUI's internal active-drag state so
/// that keyboard dismissal (Escape) keeps working afterward.
fn begin_file_search_native_drag(
    drag_path: &str,
    window: &mut gpui::Window,
    cx: &mut gpui::App,
) -> gpui::Entity<file_search::FileDragPayload> {
    if let Err(error) = crate::platform::begin_native_file_drag(drag_path) {
        tracing::warn!(
            path = %drag_path,
            error = %error,
            "failed to start native file drag"
        );
    }
    // GPUI started an internal drag because the row uses `.on_drag(...)`.
    // Once AppKit owns the drag session, clear GPUI's active drag state so
    // Escape can dismiss the view normally.
    cx.stop_active_drag(window);
    let name = std::path::Path::new(drag_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    cx.new(|_| file_search::FileDragPayload { name })
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
                cx.background_executor().timer(std::time::Duration::from_millis(16)).await;
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

    /// Render file search view with 50/50 split (list + preview) or mini list-only
    pub(crate) fn render_file_search(
        &mut self,
        query: &str,
        selected_index: usize,
        presentation: FileSearchPresentation,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        use crate::file_search::{self, FileType};

        crate::components::emit_prompt_chrome_audit(
            &crate::components::PromptChromeAudit::expanded("file_search", true),
        );
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
        // Clamp the display index so a stale selected_index from a shrinking
        // result set still resolves to a valid visible row.
        let clamped_selected_index = self.clamp_file_search_display_index(selected_index);
        let selected_file = clamped_selected_index
            .and_then(|display_index| self.file_search_result_at_display_index(display_index))
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

                let key = event.keystroke.key.as_str();
                let key_char = event.keystroke.key_char.as_deref();
                let has_cmd = event.keystroke.modifiers.platform;

                let modifiers = &event.keystroke.modifiers;

                // Route keys to actions dialog first if it's open
                match this.route_key_to_actions_dialog(
                    key,
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
                        this.handle_action(action_id, window, cx);
                        return;
                    }
                }

                // ESC: Clear query first if present, otherwise go back/close
                if is_key_escape(key) {
                    if !this.clear_builtin_view_filter(cx) {
                        this.go_back_or_close(window, cx);
                    }
                    return;
                }

                // Cmd+W closes window
                if has_cmd && key.eq_ignore_ascii_case("w") {
                    logging::log("KEY", "Cmd+W - closing window");
                    this.close_and_reset_window(cx);
                    return;
                }

                if let AppView::FileSearchView {
                    query: _,
                    selected_index,
                    ..
                } = &mut this.current_view
                {
                    // Copy the index so the mutable borrow on current_view is
                    // released before the closure borrows `this` immutably.
                    let sel_idx = *selected_index;

                    let get_selected_file = || {
                        this.clamp_file_search_display_index(sel_idx)
                            .and_then(|display_index| {
                                this.file_search_result_at_display_index(display_index)
                            })
                            .cloned()
                    };

                    // Space (unmodified) triggers Quick Look for non-directory files
                    let is_space = key.eq_ignore_ascii_case("space") || key_char == Some(" ");
                    if is_space
                        && !event.keystroke.modifiers.platform
                        && !event.keystroke.modifiers.shift
                        && !event.keystroke.modifiers.control
                        && !event.keystroke.modifiers.alt
                    {
                        if let Some(file) = get_selected_file() {
                            if file.file_type != FileType::Directory {
                                this.file_search_actions_path = Some(file.path.clone());
                                this.handle_action("quick_look".to_string(), window, cx);
                                cx.stop_propagation();
                                return;
                            }
                        }
                    }

                    match key {
                        // Arrow keys are handled by arrow_interceptor in app_impl.rs
                        // which calls stop_propagation(). This is the single source of truth
                        // for arrow key handling in FileSearchView.
                        _ if is_key_up(key) || is_key_down(key) => {
                            // Already handled by interceptor, no-op here
                        }
                        // Tab/Shift+Tab handled by intercept_keystrokes in app_impl.rs
                        // (interceptor fires BEFORE input component can capture Tab)
                        _ if is_key_enter(key) => {
                            if has_cmd {
                                // ⌘↵ / ⌘⇧↵ → launch AI harness with file context.
                                // Falls back to query-level intent when no row is selected,
                                // so this is never a dead keypress in mini file search.
                                let has_shift = event.keystroke.modifiers.shift;
                                let ai_args = if let AppView::FileSearchView {
                                    ref query,
                                    selected_index,
                                    ..
                                } = this.current_view
                                {
                                    Some((query.clone(), selected_index))
                                } else {
                                    None
                                };
                                if let Some((query, sel_idx)) = ai_args {
                                    if this.open_file_search_selection_or_query_in_tab_ai(
                                        &query,
                                        sel_idx,
                                        has_shift,
                                        cx,
                                    ) {
                                        cx.stop_propagation();
                                        return;
                                    }
                                }
                            } else {
                                if let Some(file) = get_selected_file() {
                                    if file.file_type == FileType::Directory {
                                        // Directory: browse inline
                                        let next_query = format!(
                                            "{}/",
                                            file_search::shorten_path(&file.path)
                                                .trim_end_matches('/')
                                        );
                                        let next_presentation = match &this.current_view {
                                            AppView::FileSearchView {
                                                presentation, ..
                                            } => *presentation,
                                            _ => FileSearchPresentation::Full,
                                        };
                                        this.open_file_search_view(
                                            next_query,
                                            next_presentation,
                                            cx,
                                        );
                                        cx.stop_propagation();
                                        return;
                                    }
                                    // File: open with default app and close
                                    let _ = file_search::open_file(&file.path);
                                    this.close_and_reset_window(cx);
                                }
                            }
                        }
                        _ => {
                            let has_shift = event.keystroke.modifiers.shift;

                            // Handle Cmd+K (toggle actions)
                            if has_cmd && key.eq_ignore_ascii_case("k") && !has_shift {
                                let selected = get_selected_file();
                                this.toggle_file_search_actions(selected.as_ref(), window, cx);
                                return;
                            }
                            // Handle Cmd+Shift+F (Reveal in Finder) — kept explicit
                            // because it is not a secondary command in the shared contract.
                            if has_cmd && has_shift && key.eq_ignore_ascii_case("f") {
                                if let Some(file) = get_selected_file() {
                                    this.file_search_actions_path = Some(file.path.clone());
                                    this.handle_action("reveal_in_finder".to_string(), window, cx);
                                }
                                return;
                            }
                            // All other Cmd shortcuts resolve through the shared
                            // secondary-command contract so the footer, action sheet,
                            // and keyboard dispatch stay in sync.
                            if let Some(file) = get_selected_file() {
                                let file_info = crate::file_search::FileInfo::from_result(&file);
                                if let Some(action_id) =
                                    crate::actions::resolve_file_search_secondary_action_id(
                                        key, has_cmd, has_shift, &file_info,
                                    )
                                {
                                    this.file_search_actions_path = Some(file.path.clone());
                                    this.handle_action(action_id.to_string(), window, cx);
                                    return;
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
        let current_selected = clamped_selected_index.unwrap_or(usize::MAX);
        let is_loading = self.file_search_loading;
        let click_entity_handle = cx.entity().downgrade();

        // Use uniform_list for virtualized scrolling
        // Skeleton loading: show placeholder rows while loading and no results yet
        let list_element = if is_loading && filtered_len == 0 {
            // Loading with no results yet - show static skeleton rows
            let skeleton_bg = rgba(crate::ui_foundation::hex_to_rgba_with_opacity(ui_border, crate::theme::opacity::OPACITY_SUBTLE));

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

                                // Use theme opacity for vibrancy-compatible selection
                                let bg = if is_selected {
                                    rgba((list_selected << 8) | selected_alpha)
                                } else {
                                    gpui::transparent_black().into()
                                };
                                let hover_bg = rgba((list_hover << 8) | hover_alpha);
                                let show_thumbnail =
                                    file_search::is_thumbnail_preview_supported(&file.path);
                                let thumbnail_path = file.path.clone();
                                let fallback_icon =
                                    file_search::file_type_icon(file.file_type).to_string();

                                // Click handler: select on click, open/browse on double-click
                                let click_entity = click_entity_handle.clone();
                                let file_path = file.path.clone();
                                let file_type = file.file_type;
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

                                            // Double-click: browse directory inline or open file
                                            if let gpui::ClickEvent::Mouse(mouse_event) = event {
                                                if mouse_event.down.click_count == 2 {
                                                    if file_type == FileType::Directory {
                                                        let next_query = format!(
                                                            "{}/",
                                                            file_search::shorten_path(&file_path)
                                                                .trim_end_matches('/')
                                                        );
                                                        let next_presentation =
                                                            match &this.current_view {
                                                                AppView::FileSearchView {
                                                                    presentation,
                                                                    ..
                                                                } => *presentation,
                                                                _ => FileSearchPresentation::Full,
                                                            };
                                                        this.open_file_search_view(
                                                            next_query,
                                                            next_presentation,
                                                            cx,
                                                        );
                                                    } else {
                                                        logging::log(
                                                            "UI",
                                                            &format!(
                                                                "Double-click opening file: {}",
                                                                file_path
                                                            ),
                                                        );
                                                        let _ =
                                                            file_search::open_file(&file_path);
                                                        this.close_and_reset_window(cx);
                                                    }
                                                }
                                            }
                                        });
                                    }
                                };

                                // Drag payload for native file drag-out
                                let drag_payload = file_search::FileDragPayload::from_result(file);
                                let drag_path_for_native = file.path.clone();

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
                                    .when(!is_selected, |d| d.hover(move |s| s.bg(hover_bg)))
                                    .tooltip(|window, cx| {
                                        gpui_component::tooltip::Tooltip::new(
                                            "Open selected file",
                                        )
                                        .key_binding(
                                            gpui::Keystroke::parse("enter")
                                                .ok()
                                                .map(gpui_component::kbd::Kbd::new),
                                        )
                                        .build(window, cx)
                                    })
                                    .on_click(click_handler)
                                    .on_drag(drag_payload, move |_payload, _position, window, cx| {
                                        begin_file_search_native_drag(&drag_path_for_native, window, cx)
                                    })
                                    .child(if show_thumbnail {
                                        let fallback_icon = fallback_icon.clone();
                                        div()
                                            .w(px(32.))
                                            .h(px(32.))
                                            .rounded(px(6.))
                                            .overflow_hidden()
                                            .bg(rgba(crate::ui_foundation::hex_to_rgba_with_opacity(ui_border, crate::theme::opacity::OPACITY_SUBTLE)))
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
        let list_scrollbar = self.builtin_uniform_list_scrollbar(
            &self.file_search_scroll_handle,
            filtered_len,
            8,
        );

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
                        // Chromeless preview: no header label, no card wrapper
                        .child(
                            div()
                                .w_full()
                                .min_h(px(FILE_SEARCH_PREVIEW_THUMBNAIL_MAX_SIDE_PX + 24.0))
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

            {
                let meta_row = |label: &str, value: String| {
                    div()
                        .flex()
                        .flex_row()
                        .gap(px(design_spacing.gap_md))
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(text_muted))
                                .child(label.to_string()),
                        )
                        .child(
                            div()
                                .ml_auto()
                                .text_xs()
                                .text_color(rgb(text_dimmed))
                                .child(value),
                        )
                };

                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .p(px(design_spacing.padding_lg))
                    .gap(px(design_spacing.gap_md))
                    .overflow_y_hidden()
                    .when_some(thumbnail_section, |container, section| container.child(section))
                    // Name (chromeless — no type badge pill)
                    .child(
                        div()
                            .text_lg()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(rgb(text_primary))
                            .child(file.name.clone()),
                    )
                    // Scrollable path (no section label — content is self-evident)
                    .child(crate::components::prompt_scroll_value_with_id(
                        "file-search-preview-path",
                        file.path.clone(),
                        rgb(text_dimmed),
                    ))
                    // Meta rows (no divider, no section label — spacing defines groups)
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(design_spacing.gap_sm))
                            .child(meta_row("Size", file_search::format_file_size(file.size)))
                            .child(meta_row(
                                "Modified",
                                file_search::format_relative_time(file.modified),
                            ))
                            .child(meta_row("Type", file_type_str.to_string())),
                    )
            }
        } else if is_loading {
            div()
                .flex_1()
                .flex()
                .items_center()
                .justify_center()
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(text_muted))
                        .child("Loading preview\u{2026}"),
                )
        } else {
            div()
                .flex_1()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap(px(6.0))
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(text_dimmed))
                        .child("No file selected"),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(text_muted))
                        .child("Use \u{2191}\u{2193} to inspect results"),
                )
        };

        let is_mini = matches!(presentation, FileSearchPresentation::Mini);

        // Compute mini-mode context for mode-aware chrome
        let is_directory_query = crate::file_search::parse_directory_path(query).is_some();
        let is_advanced_query = crate::file_search::looks_like_advanced_mdquery(query);
        let mode_label = if is_directory_query {
            "Browse"
        } else if is_advanced_query {
            "Spotlight+"
        } else {
            "Search"
        };

        let (empty_title, empty_subtitle) = if query.is_empty() {
            (
                "Type to search files",
                "Use ~/ to browse a folder inline, or press \u{2318}\u{21b5} to ask AI how to search",
            )
        } else if is_directory_query && query.ends_with('/') {
            (
                "Folder is empty",
                "Try another path, or press \u{2318}\u{21b5} for AI help deciding what to inspect next",
            )
        } else if is_directory_query {
            (
                "No matches in folder",
                "Keep typing to narrow the current directory, or press \u{2318}\u{21b5} for AI help refining the browse",
            )
        } else if is_advanced_query {
            (
                "No Spotlight matches",
                "Try a broader predicate, or press \u{2318}\u{21b5} for AI help rewriting the query",
            )
        } else {
            (
                "No files found",
                "Try a broader query, or press \u{2318}\u{21b5} to ask AI how to refine this search",
            )
        };

        // Footer: three-key pattern matching the universal design.
        // All secondary commands are discoverable via ⌘K Actions.
        let file_search_hints: Vec<SharedString> = if let Some(file) = selected_file.as_ref() {
            let primary = if file.file_type == FileType::Directory {
                "\u{21b5} Browse"
            } else {
                "\u{21b5} Open"
            };
            vec![primary.into(), "\u{2318}K Actions".into(), "Tab AI".into()]
        } else if self.file_search_current_dir.is_some() {
            // Browsing a concrete directory — ⌘K exposes directory-level actions
            // even without a selected row.
            vec!["\u{2318}K Actions".into(), "Tab AI".into()]
        } else if is_loading {
            vec!["Tab AI".into()]
        } else if filtered_len == 0 {
            vec!["Tab AI".into()]
        } else {
            vec!["Tab AI".into()]
        };


        // Header: bare input + file count (scaffold adds padding/layout)
        let input_height = CURSOR_HEIGHT_LG + (CURSOR_MARGIN_Y * 2.0);
        let header_element = div()
            .flex_1()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(HEADER_GAP))
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
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_end()
                    .py(px(4.))
                    .w(px(120.))
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(text_dimmed))
                            .child(if is_mini {
                                format!("{} \u{00b7} {}", mode_label, filtered_len)
                            } else {
                                format!("{} files", filtered_len)
                            }),
                    ),
            );

        // List pane: loading/empty/results with scrollbar overlay
        let list_pane = if is_loading && filtered_len == 0 {
            div()
                .w_full()
                .h_full()
                .flex()
                .items_center()
                .justify_center()
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(text_dimmed))
                        .child("Searching..."),
                )
        } else if filtered_len == 0 {
            div()
                .w_full()
                .h_full()
                .flex()
                .items_center()
                .justify_center()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap(px(6.))
                        .child(
                            div()
                                .text_sm()
                                .text_color(rgb(text_dimmed))
                                .child(empty_title),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(text_muted))
                                .child(empty_subtitle),
                        ),
                )
        } else {
            div()
                .relative()
                .w_full()
                .h_full()
                .child(list_element)
                .child(list_scrollbar)
        };

        // Preview pane: file detail or placeholder
        let preview_pane = preview_content;

        if is_mini {
            crate::components::render_minimal_list_prompt_scaffold(
                header_element,
                list_pane,
                file_search_hints.clone(),
                None,
            )
            .key_context("FileSearchView")
            .track_focus(&self.focus_handle)
            .on_key_down(handle_key)
            .into_any_element()
        } else {
            crate::components::render_expanded_view_scaffold_with_hints(
                header_element,
                list_pane,
                preview_pane,
                file_search_hints,
                None,
            )
            .key_context("FileSearchView")
            .track_focus(&self.focus_handle)
            .on_key_down(handle_key)
            .into_any_element()
        }

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

#[cfg(test)]
mod file_search_chrome_audit {
    #[test]
    fn file_search_uses_shared_expanded_view_shell() {
        let source = include_str!("file_search.rs");

        // Must route through the shared expanded-view scaffold
        assert!(
            source.contains("render_expanded_view_scaffold("),
            "file_search must use the shared expanded-view scaffold"
        );

        // Must emit expanded layout audit
        assert!(
            source.contains("PromptChromeAudit::expanded(\"file_search\""),
            "file_search must declare expanded layout mode"
        );

        // Must NOT have hand-rolled divider chrome (split string to avoid self-match)
        let divider_call = "SectionDivider".to_owned() + "::new()";
        assert!(
            !source.contains(&divider_call),
            "file_search must not use SectionDivider (scaffold owns structure)"
        );

        // Must NOT have legacy PromptFooter
        let legacy = "Prompt".to_owned() + "Footer::new(";
        assert!(
            !source.contains(&legacy),
            "file_search must not use PromptFooter (scaffold owns footer)"
        );

        // Must NOT have hand-rolled render_simple_hint_strip (scaffold owns footer)
        // The only render_simple_hint_strip usage should be inside the scaffold itself.
        // Count occurrences — none should appear in file_search.rs directly.
        let hint_strip_call = "render_simple_hint_strip(";
        let count = source.matches(hint_strip_call).count();
        assert_eq!(
            count, 0,
            "file_search must not call render_simple_hint_strip directly (scaffold owns footer, found {count})"
        );
    }

    #[test]
    fn file_search_has_no_hardcoded_alpha_fills() {
        let source = include_str!("file_search.rs");
        assert!(
            !source.contains("| 0x24)"),
            "file_search should not contain hardcoded alpha fill 0x24"
        );
        assert!(
            !source.contains("| 0x40)"),
            "file_search should not contain hardcoded alpha fill 0x40"
        );
    }

    #[test]
    fn file_search_emits_checkpoint_log() {
        let source = include_str!("file_search.rs");
        assert!(
            source.contains("file_search_chrome_checkpoint"),
            "file_search must emit a structured checkpoint log for migration verification"
        );
    }
}
