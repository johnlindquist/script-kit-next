pub struct JsonlReader<R: Read> {
    reader: BufReader<R>,
    /// Reusable line buffer - cleared and reused per read to avoid allocations
    line_buffer: String,
    /// Reusable byte buffer for bounded line reads
    byte_buffer: Vec<u8>,
}

impl<R: Read> JsonlReader<R> {
    /// Create a new JSONL reader
    pub fn new(reader: R) -> Self {
        JsonlReader {
            reader: BufReader::new(reader),
            // Pre-allocate reasonable capacity for typical JSON messages
            line_buffer: String::with_capacity(1024),
            byte_buffer: Vec::with_capacity(1024),
        }
    }

    fn read_next_line(&mut self, max_line_bytes: usize) -> Result<LineRead, std::io::Error> {
        self.byte_buffer.clear();
        let mut total_bytes = 0usize;
        let mut saw_any_data = false;

        loop {
            let available = self.reader.fill_buf()?;
            if available.is_empty() {
                if !saw_any_data {
                    return Ok(LineRead::Eof);
                }

                self.decode_line_buffer();
                return Ok(LineRead::Line);
            }

            saw_any_data = true;
            let newline_pos = available.iter().position(|&byte| byte == b'\n');
            let consumed_len = newline_pos.map_or(available.len(), |idx| idx + 1);

            if self.byte_buffer.len() < max_line_bytes {
                let remaining = max_line_bytes - self.byte_buffer.len();
                let copy_len = remaining.min(consumed_len);
                self.byte_buffer.extend_from_slice(&available[..copy_len]);
            }

            self.reader.consume(consumed_len);
            total_bytes = total_bytes.saturating_add(consumed_len);

            if total_bytes > max_line_bytes {
                // Drain the remainder of the oversized line so parsing can continue
                // on the next JSONL message.
                if newline_pos.is_none() {
                    loop {
                        let remaining = self.reader.fill_buf()?;
                        if remaining.is_empty() {
                            break;
                        }

                        if let Some(next_newline_pos) =
                            remaining.iter().position(|&byte| byte == b'\n')
                        {
                            self.reader.consume(next_newline_pos + 1);
                            total_bytes = total_bytes.saturating_add(next_newline_pos + 1);
                            break;
                        }

                        let chunk_len = remaining.len();
                        self.reader.consume(chunk_len);
                        total_bytes = total_bytes.saturating_add(chunk_len);
                    }
                }

                self.decode_line_buffer();
                return Ok(LineRead::TooLong { raw_len: total_bytes });
            }

            if newline_pos.is_some() {
                self.decode_line_buffer();
                return Ok(LineRead::Line);
            }
        }
    }

    fn decode_line_buffer(&mut self) {
        self.line_buffer.clear();
        let decoded = String::from_utf8_lossy(&self.byte_buffer);
        self.line_buffer.push_str(decoded.as_ref());
    }

    /// Read the next message from the stream
    ///
    /// # Returns
    /// * `Ok(Some(Message))` - Successfully parsed message
    /// * `Ok(None)` - End of stream
    /// * `Err(e)` - Parse error
    pub fn next_message(&mut self) -> Result<Option<Message>, Box<dyn std::error::Error>> {
        // Use loop instead of recursion to prevent stack overflow on many empty lines
        loop {
            match self.read_next_line(MAX_PROTOCOL_LINE_BYTES)? {
                LineRead::Eof => {
                    debug!("Reached end of JSONL stream");
                    return Ok(None);
                }
                LineRead::Line => {
                    let trimmed = self.line_buffer.trim_end_matches(['\r', '\n']);
                    debug!(
                        bytes_read = self.line_buffer.len(),
                        "Read line from JSONL stream"
                    );
                    if trimmed.trim().is_empty() {
                        debug!("Skipping empty line in JSONL stream");
                        continue; // Skip empty lines (loop instead of recursion)
                    }
                    let msg = parse_message(trimmed)?;
                    return Ok(Some(msg));
                }
                LineRead::TooLong { raw_len } => {
                    return Err(Box::new(std::io::Error::new(
                        ErrorKind::InvalidData,
                        format!(
                            "JSONL line exceeds {} bytes (received {raw_len} bytes)",
                            MAX_PROTOCOL_LINE_BYTES
                        ),
                    )));
                }
            }
        }
    }

    /// Read the next message with graceful unknown type handling
    ///
    /// Unlike `next_message`, this method uses `parse_message_graceful` to
    /// handle unknown message types without errors. Unknown types are logged
    /// and skipped, continuing to read the next message.
    ///
    /// # Logging
    /// All logging is consolidated here (reader layer). The parse_message_graceful
    /// function does not log - it returns structured results for this layer to handle.
    ///
    /// # Returns
    /// * `Ok(Some(Message))` - Successfully parsed known message
    /// * `Ok(None)` - End of stream
    /// * `Err(e)` - IO error (not parse errors for unknown types)
    pub fn next_message_graceful(&mut self) -> Result<Option<Message>, std::io::Error> {
        self.next_message_graceful_with_handler(|_| {})
    }

    /// Read the next message with graceful unknown type handling, reporting parse issues.
    pub(crate) fn next_message_graceful_with_handler<F>(
        &mut self,
        mut on_issue: F,
    ) -> Result<Option<Message>, std::io::Error>
    where
        F: FnMut(ParseIssue),
    {
        loop {
            match self.read_next_line(MAX_PROTOCOL_LINE_BYTES)? {
                LineRead::Eof => {
                    debug!("Reached end of JSONL stream");
                    return Ok(None);
                }
                LineRead::Line => {
                    let trimmed = self.line_buffer.trim_end_matches(['\r', '\n']);
                    if trimmed.trim().is_empty() {
                        debug!("Skipping empty line in JSONL stream");
                        continue;
                    }

                    // Get preview for logging (security: truncate large payloads)
                    let (preview, raw_len) = log_preview(trimmed);

                    match parse_message_graceful(trimmed) {
                        ParseResult::Ok(msg) => {
                            // Set correlation ID for this protocol message
                            // Use message ID or generate a unique one
                            let msg_id = msg
                                .id()
                                .map(|s| s.to_string())
                                .unwrap_or_else(|| format!("msg:{}", Uuid::new_v4()));
                            let _guard =
                                crate::logging::set_correlation_id(format!("protocol:{}", msg_id));

                            debug!(message_id = ?msg.id(), "Successfully parsed message");
                            return Ok(Some(msg));
                        }
                        ParseResult::MissingType { .. } => {
                            let issue = ParseIssue::new(
                                ParseIssueKind::MissingType,
                                None,
                                None,
                                preview.to_string(),
                                raw_len,
                            );
                            // Set correlation ID for this parse error
                            let _guard =
                                crate::logging::set_correlation_id(issue.correlation_id.clone());

                            warn!(
                                correlation_id = %issue.correlation_id,
                                raw_preview = %issue.raw_preview,
                                raw_len = issue.raw_len,
                                "Skipping message with missing 'type' field"
                            );
                            on_issue(issue);
                            continue;
                        }
                        ParseResult::UnknownType { message_type, .. } => {
                            let issue = ParseIssue::new(
                                ParseIssueKind::UnknownType,
                                Some(message_type.clone()),
                                None,
                                preview.to_string(),
                                raw_len,
                            );
                            // Set correlation ID for this parse error
                            let _guard =
                                crate::logging::set_correlation_id(issue.correlation_id.clone());

                            warn!(
                                correlation_id = %issue.correlation_id,
                                message_type = %message_type,
                                raw_preview = %issue.raw_preview,
                                raw_len = issue.raw_len,
                                "Skipping unknown message type"
                            );
                            on_issue(issue);
                            continue;
                        }
                        ParseResult::InvalidPayload {
                            message_type,
                            error,
                            ..
                        } => {
                            let issue = ParseIssue::new(
                                ParseIssueKind::InvalidPayload,
                                Some(message_type.clone()),
                                Some(error.clone()),
                                preview.to_string(),
                                raw_len,
                            );
                            // Set correlation ID for this parse error
                            let _guard =
                                crate::logging::set_correlation_id(issue.correlation_id.clone());

                            warn!(
                                correlation_id = %issue.correlation_id,
                                message_type = %message_type,
                                error = %error,
                                raw_preview = %issue.raw_preview,
                                raw_len = issue.raw_len,
                                "Skipping message with invalid payload"
                            );
                            on_issue(issue);
                            continue;
                        }
                        ParseResult::ParseError(e) => {
                            let issue = ParseIssue::new(
                                ParseIssueKind::ParseError,
                                None,
                                Some(e.to_string()),
                                preview.to_string(),
                                raw_len,
                            );
                            // Set correlation ID for this parse error
                            let _guard =
                                crate::logging::set_correlation_id(issue.correlation_id.clone());

                            // Log but continue - graceful degradation
                            warn!(
                                correlation_id = %issue.correlation_id,
                                error = %e,
                                raw_preview = %issue.raw_preview,
                                raw_len = issue.raw_len,
                                "Skipping malformed JSON message"
                            );
                            on_issue(issue);
                            continue;
                        }
                    }
                }
                LineRead::TooLong { raw_len } => {
                    let (preview, _) = log_preview(&self.line_buffer);
                    let issue = ParseIssue::new(
                        ParseIssueKind::LineTooLong,
                        None,
                        Some(format!(
                            "JSONL line exceeds {} bytes (received {raw_len} bytes)",
                            MAX_PROTOCOL_LINE_BYTES
                        )),
                        preview.to_string(),
                        raw_len,
                    );
                    let _guard = crate::logging::set_correlation_id(issue.correlation_id.clone());

                    warn!(
                        correlation_id = %issue.correlation_id,
                        raw_preview = %issue.raw_preview,
                        raw_len = issue.raw_len,
                        max_line_bytes = MAX_PROTOCOL_LINE_BYTES,
                        "Skipping oversized JSONL message"
                    );
                    on_issue(issue);
                    continue;
                }
            }
        }
    }
}
