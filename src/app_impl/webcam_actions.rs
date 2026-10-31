use super::*;

impl ScriptListApp {
    pub(crate) fn webcam_photo_directory() -> std::path::PathBuf {
        if let Some(policy) = crate::runtime_policy::owned_evaluation() {
            return policy.root().join("webcam-photos");
        }
        if let Some(home) = dirs::home_dir() {
            let desktop = home.join("Desktop");
            if desktop.exists() {
                return desktop;
            }
        }

        let temp = std::env::temp_dir();
        if temp.exists() {
            temp
        } else {
            std::path::PathBuf::from("/tmp")
        }
    }

    fn save_private_webcam_photo_at(
        directory: &std::path::Path,
        timestamp: &str,
        png_data: &[u8],
    ) -> std::io::Result<std::path::PathBuf> {
        if timestamp.is_empty()
            || !timestamp
                .chars()
                .all(|character| character.is_ascii_digit() || character == '-')
        {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Webcam photo timestamp must be a safe filename component",
            ));
        }

        crate::atomic_file::write_private_unique_named_file(
            directory,
            &format!("webcam-photo-{timestamp}"),
            "png",
            png_data,
        )
    }

    #[cfg(target_os = "macos")]
    pub(crate) fn encode_webcam_frame_to_png(
        pixel_buffer: &core_video::pixel_buffer::CVPixelBuffer,
    ) -> Result<Vec<u8>, String> {
        use image::ImageEncoder;

        let lock_flags = core_video::pixel_buffer::kCVPixelBufferLock_ReadOnly;
        let lock_status = pixel_buffer.lock_base_address(lock_flags);
        if lock_status != core_video::r#return::kCVReturnSuccess {
            return Err(format!(
                "Failed to lock webcam frame (status={})",
                lock_status
            ));
        }

        let result = (|| -> Result<Vec<u8>, String> {
            if !pixel_buffer.is_planar() || pixel_buffer.get_plane_count() < 2 {
                return Err("Webcam frame format is not NV12".to_string());
            }

            let width = pixel_buffer.get_width_of_plane(0);
            let height = pixel_buffer.get_height_of_plane(0);
            if width == 0 || height == 0 {
                return Err("Webcam frame is empty".to_string());
            }

            let y_stride = pixel_buffer.get_bytes_per_row_of_plane(0);
            let uv_stride = pixel_buffer.get_bytes_per_row_of_plane(1);
            let uv_height = pixel_buffer.get_height_of_plane(1);

            // SAFETY: The pixel buffer is locked for reading above and the plane index is valid (checked via is_planar + plane_count >= 2).
            let y_plane_ptr = unsafe { pixel_buffer.get_base_address_of_plane(0) as *const u8 };
            // SAFETY: Same lock guard as y_plane_ptr; plane 1 exists per the plane_count check.
            let uv_plane_ptr = unsafe { pixel_buffer.get_base_address_of_plane(1) as *const u8 };
            if y_plane_ptr.is_null() || uv_plane_ptr.is_null() {
                return Err("Webcam frame memory is unavailable".to_string());
            }

            let y_plane_len = y_stride.checked_mul(height).ok_or_else(|| {
                format!(
                    "Webcam Y plane size overflow (stride={} height={})",
                    y_stride, height
                )
            })?;
            let uv_plane_len = uv_stride.checked_mul(uv_height).ok_or_else(|| {
                format!(
                    "Webcam UV plane size overflow (stride={} height={})",
                    uv_stride, uv_height
                )
            })?;
            let rgb_len = width
                .checked_mul(height)
                .and_then(|pixels| pixels.checked_mul(3))
                .ok_or_else(|| {
                    format!(
                        "Webcam RGB buffer size overflow (width={} height={})",
                        width, height
                    )
                })?;

            // SAFETY: y_plane_ptr is non-null (checked above), and y_plane_len = y_stride * height is within the locked buffer.
            let y_plane = unsafe { std::slice::from_raw_parts(y_plane_ptr, y_plane_len) };
            // SAFETY: uv_plane_ptr is non-null (checked above), and uv_plane_len = uv_stride * uv_height is within the locked buffer.
            let uv_plane = unsafe { std::slice::from_raw_parts(uv_plane_ptr, uv_plane_len) };

            let mut rgb = vec![0u8; rgb_len];

            for y in 0..height {
                let y_row = y * y_stride;
                let uv_row = (y / 2) * uv_stride;

                for x in 0..width {
                    let y_val = y_plane[y_row + x] as f32;
                    let uv_idx = uv_row + (x / 2) * 2;
                    if uv_idx + 1 >= uv_plane.len() {
                        continue;
                    }

                    let u = uv_plane[uv_idx] as f32 - 128.0;
                    let v = uv_plane[uv_idx + 1] as f32 - 128.0;

                    let r = (y_val + 1.402 * v).clamp(0.0, 255.0) as u8;
                    let g = (y_val - 0.344_136 * u - 0.714_136 * v).clamp(0.0, 255.0) as u8;
                    let b = (y_val + 1.772 * u).clamp(0.0, 255.0) as u8;

                    let idx = (y * width + x) * 3;
                    rgb[idx] = r;
                    rgb[idx + 1] = g;
                    rgb[idx + 2] = b;
                }
            }

            let mut png_data = Vec::new();
            let encoder = image::codecs::png::PngEncoder::new(&mut png_data);
            let png_width = u32::try_from(width)
                .map_err(|_| format!("Webcam frame width out of range: {}", width))?;
            let png_height = u32::try_from(height)
                .map_err(|_| format!("Webcam frame height out of range: {}", height))?;
            encoder
                .write_image(&rgb, png_width, png_height, image::ColorType::Rgb8.into())
                .map_err(|e| format!("Failed to encode webcam frame: {}", e))?;

            Ok(png_data)
        })();

        let unlock_status = pixel_buffer.unlock_base_address(lock_flags);
        if unlock_status != core_video::r#return::kCVReturnSuccess {
            tracing::error!(
                status = unlock_status,
                "Failed to unlock webcam frame after capture"
            );
        }

        result
    }

    /// Saves the frame on screen as a PNG and answers a waiting prompt with its path. Returns
    /// the saved path; failures are already said in the line above the input.
    #[cfg(target_os = "macos")]
    fn capture_webcam_photo(&mut self, cx: &mut Context<Self>) -> Option<std::path::PathBuf> {
        let (pixel_buffer, prompt_id) = match &self.current_view {
            AppView::WebcamView { entity } => {
                let prompt = entity.read(cx);
                (prompt.pixel_buffer.clone(), Some(prompt.base.id.clone()))
            }
            _ => (None, None),
        };
        let binding = self.prompt_completion.as_ref().filter(|binding| Some(binding.instance().id.as_str()) == prompt_id.as_deref()).cloned();
        if binding.is_none() && crate::runtime_policy::is_owned_evaluation() {
            self.notice_failed("That photo request has already finished", cx);
            return None;
        }
        // A waiting prompt is answered by the first photo; retakes are only saved.
        let binding = binding.filter(|binding| {
            let state = binding.observation();
            !state.retired && !state.completed
        });

        let Some(pixel_buffer) = pixel_buffer else {
            if let Some(binding) = &binding { binding.record_error(crate::prompt_completion::SubmissionError::InvalidInput); }
            cx.notify();
            self.notice_failed("No camera frame available yet", cx);
            return None;
        };

        let png_data = match Self::encode_webcam_frame_to_png(&pixel_buffer) {
            Ok(data) => data,
            Err(e) => {
                if let Some(binding) = &binding { binding.record_error(crate::prompt_completion::SubmissionError::InvalidInput); }
                tracing::error!(error = %e, "Failed to capture webcam photo");
                cx.notify();
                self.notice_failed(format!("Couldn't take the photo: {e}"), cx);
                return None;
            }
        };

        let save_dir = Self::webcam_photo_directory();
        if let Some(policy) = crate::runtime_policy::owned_evaluation() {
            if let Err(error) = policy.require_owned_path(&save_dir).map_err(|error| error.to_string())
                .and_then(|_| std::fs::create_dir_all(&save_dir).map_err(|error| error.to_string())) {
                if let Some(binding) = &binding { binding.record_error(crate::prompt_completion::SubmissionError::StorageFailure); }
                self.notice_failed(format!("Couldn't save the photo: {error}"), cx);
                return None;
            }
        }
        let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();

        match Self::save_private_webcam_photo_at(&save_dir, &timestamp, &png_data) {
            Ok(save_path) => {
                if let Some(binding) = &binding {
                    let outcome = crate::prompt_completion::PromptOutcome::Submitted(
                        crate::protocol::SubmitValue::Text(save_path.to_string_lossy().into_owned()),
                    );
                    if let Err(error) = binding.try_complete(outcome) {
                        self.notice_failed(format!("Couldn't send the photo to the script: {error}"), cx);
                        return None;
                    }
                }
                if crate::runtime_policy::is_owned_evaluation() {
                    self.mark_main_data_changed();
                    return Some(save_path);
                }
                let safe_path =
                    crate::logging::log_private_user_value(&save_path.display().to_string());
                tracing::info!(
                    category = "ACTIONS",
                    path_bytes = safe_path.raw_bytes,
                    path_sha256 = %safe_path.sha256,
                    "Webcam photo saved"
                );
                Some(save_path)
            }
            Err(error) => {
                if let Some(binding) = &binding { binding.record_error(crate::prompt_completion::SubmissionError::StorageFailure); }
                tracing::error!(reason = ?error.kind(), "Failed to save private webcam photo");
                cx.notify();
                self.notice_failed(
                    "Photo could not be saved safely. Check Desktop and try again.",
                    cx,
                );
                None
            }
        }
    }

    #[cfg(not(target_os = "macos"))]
    fn capture_webcam_photo(&mut self, cx: &mut Context<Self>) -> Option<std::path::PathBuf> {
        tracing::warn!(
            category = "ACTIONS",
            "Webcam capture requested on unsupported platform"
        );
        cx.notify();
        self.notice_unsupported_platform("Webcam capture", cx);
        None
    }

    fn current_webcam_entity(&self) -> Option<Entity<prompts::WebcamPrompt>> {
        match &self.current_view {
            AppView::WebcamView { entity } => Some(entity.clone()),
            _ => None,
        }
    }

    /// Takes the photo now (ending any countdown) and shows it for review with its copy actions.
    pub(crate) fn webcam_take_photo(&mut self, cx: &mut Context<Self>) -> bool {
        let Some(entity) = self.current_webcam_entity() else {
            return false;
        };
        entity.update(cx, |prompt, cx| {
            if prompt.session.end_countdown() {
                cx.notify();
            }
        });
        let Some(path) = self.capture_webcam_photo(cx) else {
            return false;
        };
        entity.update(cx, |prompt, cx| {
            // Capture ran synchronously on this thread, so the frame on screen is the one saved.
            #[cfg(target_os = "macos")]
            {
                prompt.still = prompt.pixel_buffer.clone();
            }
            prompt.session.captured(path.clone());
            cx.notify();
        });
        let folder = path
            .parent()
            .and_then(|parent| parent.file_name())
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| "disk".to_string());
        self.one_window_notice(
            crate::line_notice::INFO,
            format!("Photo saved to {folder}"),
            cx,
        );
        cx.notify();
        true
    }

    /// Counts down `seconds` on the picture, then takes the photo.
    pub(crate) fn start_webcam_countdown(&mut self, seconds: u8, cx: &mut Context<Self>) -> bool {
        let Some(entity) = self.current_webcam_entity() else {
            return false;
        };
        let generation = entity.update(cx, |prompt, cx| {
            let generation = prompt.session.start_countdown(seconds);
            cx.notify();
            generation
        });
        let Some(generation) = generation else {
            return false;
        };
        let prompt = entity.downgrade();
        cx.spawn(async move |this, cx| loop {
            cx.background_executor()
                .timer(std::time::Duration::from_secs(1))
                .await;
            let counting = this
                .update(cx, |app, cx| app.tick_webcam_countdown(&prompt, generation, cx))
                .unwrap_or(false);
            if !counting {
                break;
            }
        })
        .detach();
        true
    }

    /// One second of the countdown started at `generation`. Returns whether it is still counting.
    fn tick_webcam_countdown(
        &mut self,
        prompt: &gpui::WeakEntity<prompts::WebcamPrompt>,
        generation: u64,
        cx: &mut Context<Self>,
    ) -> bool {
        use crate::prompts::webcam_session::CountdownTick;

        let Some(entity) = prompt.upgrade() else {
            return false;
        };
        if self.current_webcam_entity().as_ref() != Some(&entity) {
            return false;
        }
        let tick = entity.update(cx, |prompt, cx| {
            let tick = prompt.session.tick(generation);
            cx.notify();
            tick
        });
        match tick {
            CountdownTick::Stale => false,
            CountdownTick::Remaining(_) => true,
            CountdownTick::Shoot => {
                self.webcam_take_photo(cx);
                false
            }
        }
    }

    /// Escape on the webcam steps back one phase. Returns whether it did (otherwise Escape
    /// dismisses the view as usual).
    pub(crate) fn webcam_step_back(&mut self, cx: &mut Context<Self>) -> bool {
        use crate::prompts::webcam_session::WebcamEscape;

        let Some(entity) = self.current_webcam_entity() else {
            return false;
        };
        let stepped = entity.update(cx, |prompt, cx| {
            let step = prompt.session.escape();
            #[cfg(target_os = "macos")]
            if step == WebcamEscape::Retake {
                prompt.still = None;
            }
            cx.notify();
            step
        });
        if stepped == WebcamEscape::Leave {
            return false;
        }
        cx.notify();
        true
    }

    /// ↵ on the webcam: the current phase's first action.
    pub(crate) fn webcam_run_primary(&mut self, cx: &mut Context<Self>) {
        let Some(entity) = self.current_webcam_entity() else {
            return;
        };
        let action_id = entity.read(cx).session.primary_action_id();
        let dctx = crate::action_helpers::DispatchContext::for_builtin("builtin/webcam");
        let _outcome = self.execute_webcam_action(action_id, &dctx, cx);
    }

    /// The ↵ label for the webcam footer; `None` without a context to read the view.
    pub(crate) fn webcam_primary_label(&self, cx: Option<&gpui::App>) -> &'static str {
        match (&self.current_view, cx) {
            (AppView::WebcamView { entity }, Some(cx)) => entity.read(cx).session.primary_label(),
            _ => crate::prompts::webcam_session::WebcamSession::default().primary_label(),
        }
    }

    pub(crate) fn webcam_review_path(&self, cx: &gpui::App) -> Option<std::path::PathBuf> {
        let entity = self.current_webcam_entity()?;
        let path = entity.read(cx).session.photo_path()?.to_path_buf();
        Some(path)
    }

    /// ⌘⇧C and the footer's Copy Path button. Returns whether a reviewed photo was there to copy.
    pub(crate) fn webcam_copy_path(&mut self, cx: &mut Context<Self>) -> bool {
        if self.webcam_review_path(cx).is_none() {
            return false;
        }
        let dctx = crate::action_helpers::DispatchContext::for_builtin("builtin/webcam");
        let _outcome = self.execute_webcam_action(
            crate::prompts::webcam_session::action_id::COPY_PATH,
            &dctx,
            cx,
        );
        true
    }

    /// Copies, reveals or opens the reviewed photo, then leaves so you can paste or look at it.
    fn webcam_finish_with_photo(
        &mut self,
        action_id: &str,
        cx: &mut Context<Self>,
    ) -> Result<&'static str, &'static str> {
        use crate::prompts::webcam_session::action_id as id;

        let Some(path) = self.webcam_review_path(cx) else {
            return Err("Take a photo first");
        };
        let path_text = path.to_string_lossy().into_owned();
        let (result, done, failed) = match action_id {
            id::COPY_IMAGE => (
                std::fs::read(&path).map_err(|error| error.to_string()).and_then(|png| {
                    crate::clipboard_history::copy_image_with_file_url(&png, &path)
                        .map_err(|error| error.to_string())
                }),
                "Copied the photo",
                "Couldn't copy the photo",
            ),
            id::COPY_PATH => (
                crate::platform::copy_text_to_clipboard(&path_text),
                "Copied the photo's path",
                "Couldn't copy the photo's path",
            ),
            id::REVEAL => (
                crate::file_search::reveal_in_finder(&path_text),
                "Showing the photo in Finder",
                "Couldn't show the photo in Finder",
            ),
            id::OPEN => (
                crate::file_search::open_file(&path_text),
                "Opened the photo",
                "Couldn't open the photo",
            ),
            _ => return Err("Unknown webcam action"),
        };
        if let Err(error) = result {
            let safe_error = crate::logging::log_private_user_value(&error);
            tracing::warn!(
                category = "ACTIONS",
                action = action_id,
                error_bytes = safe_error.raw_bytes,
                error_sha256 = %safe_error.sha256,
                "Webcam photo action failed"
            );
            return Err(failed);
        }
        Ok(done)
    }

    /// The ⌘K rows for the webcam's current phase.
    pub(crate) fn webcam_actions_for_dialog(&self, cx: &gpui::App) -> Vec<crate::actions::Action> {
        use crate::actions::{Action, ActionCategory};

        let rows = match self.current_webcam_entity() {
            Some(entity) => entity.read(cx).session.action_rows(),
            None => crate::prompts::webcam_session::WebcamSession::default().action_rows(),
        };
        rows.into_iter()
            .map(|row| {
                let action = Action::new(
                    row.id,
                    row.title,
                    Some(row.description.to_string()),
                    ActionCategory::ScriptContext,
                );
                match row.shortcut {
                    Some(shortcut) => action.with_shortcut(shortcut),
                    None => action,
                }
            })
            .collect()
    }

    pub(crate) fn execute_webcam_action(
        &mut self,
        action_id: &str,
        dctx: &crate::action_helpers::DispatchContext,
        cx: &mut Context<Self>,
    ) -> crate::action_helpers::DispatchOutcome {
        use crate::action_helpers::{DispatchOutcome, ERROR_ACTION_FAILED};
        use crate::prompts::webcam_session::{action_id as id, COUNTDOWN_LONG_SECONDS, COUNTDOWN_SHORT_SECONDS};

        tracing::info!(
            category = "UI",
            action = action_id,
            trace_id = %dctx.trace_id,
            surface = %dctx.surface,
            "Webcam action triggered"
        );
        let success = |detail: &'static str| {
            DispatchOutcome::success()
                .with_trace_id(dctx.trace_id.clone())
                .with_detail(detail)
        };
        let failure = |message: &str, detail: &'static str| {
            DispatchOutcome::error(ERROR_ACTION_FAILED, message)
                .with_trace_id(dctx.trace_id.clone())
                .with_detail(detail)
        };

        match action_id {
            id::CAPTURE => {
                if self.webcam_take_photo(cx) {
                    success("webcam_capture")
                } else {
                    failure("Failed to capture photo", "webcam_capture_failed")
                }
            }
            id::COUNTDOWN_SHORT | id::COUNTDOWN_LONG => {
                let seconds = if action_id == id::COUNTDOWN_SHORT {
                    COUNTDOWN_SHORT_SECONDS
                } else {
                    COUNTDOWN_LONG_SECONDS
                };
                if self.start_webcam_countdown(seconds, cx) {
                    success("webcam_countdown")
                } else {
                    self.notice_failed("A countdown starts from the live camera", cx);
                    failure("Countdown not available", "webcam_countdown_unavailable")
                }
            }
            id::CANCEL_COUNTDOWN => {
                let cancelled = self.current_webcam_entity().is_some_and(|entity| {
                    entity.update(cx, |prompt, cx| {
                        let cancelled = prompt.session.end_countdown();
                        cx.notify();
                        cancelled
                    })
                });
                if cancelled {
                    success("webcam_countdown_cancelled")
                } else {
                    failure("No countdown is running", "webcam_no_countdown")
                }
            }
            id::RETAKE => {
                let retaken = self.current_webcam_entity().is_some_and(|entity| {
                    entity.update(cx, |prompt, cx| {
                        let retaken = prompt.session.retake();
                        #[cfg(target_os = "macos")]
                        {
                            prompt.still = None;
                        }
                        cx.notify();
                        retaken
                    })
                });
                if retaken {
                    success("webcam_retake")
                } else {
                    failure("No photo to retake", "webcam_no_photo")
                }
            }
            id::COPY_IMAGE | id::COPY_PATH | id::REVEAL | id::OPEN => {
                match self.webcam_finish_with_photo(action_id, cx) {
                    Ok(done) => {
                        self.notice_done(done, cx);
                        if !crate::runtime_policy::is_owned_evaluation() {
                            self.hide_main_and_reset(cx);
                        }
                        success("webcam_photo_action")
                    }
                    Err(message) => {
                        self.notice_failed(message, cx);
                        failure(message, "webcam_photo_action_failed")
                    }
                }
            }
            id::CLOSE => {
                let binding = self.prompt_completion.as_ref().filter(|binding| matches!(&self.current_view, AppView::WebcamView { entity } if entity.read(cx).base.id == binding.instance().id));
                if let Some(binding) = binding {
                    // A photo already answered the prompt; closing only retires it.
                    if !binding.observation().completed {
                        if let Err(error) = binding.try_complete(crate::prompt_completion::PromptOutcome::Cancelled) {
                            return DispatchOutcome::error(ERROR_ACTION_FAILED, error.to_string());
                        }
                    }
                    binding.retire();
                }
                if crate::runtime_policy::is_owned_evaluation() {
                    self.mark_main_data_changed();
                    cx.notify();
                    return success("webcam_close");
                }
                cx.notify();
                self.one_window_notice(crate::line_notice::INFO, "Webcam closed".to_string(), cx);
                self.hide_main_and_reset(cx);
                success("webcam_close")
            }
            _ => {
                tracing::info!(
                    category = "UI",
                    action = action_id,
                    trace_id = %dctx.trace_id,
                    surface = %dctx.surface,
                    "Webcam action: unknown id, falling back to SDK action routing"
                );
                self.trigger_sdk_action_with_trace(action_id, &dctx.trace_id)
            }
        }
    }
}
