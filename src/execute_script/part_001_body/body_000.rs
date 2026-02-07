        logging::log(
            "EXEC",
            &format!("Starting interactive execution: {}", script.name),
        );

        // Store script path for error reporting in reader thread
        let script_path_for_errors = script.path.to_string_lossy().to_string();

        match executor::execute_script_interactive(&script.path) {
            Ok(session) => {
                logging::log("EXEC", "Interactive session started successfully");

                // Store PID for explicit cleanup (belt-and-suspenders approach)
                let pid = session.pid();
                self.current_script_pid = Some(pid);
                logging::log("EXEC", &format!("Stored script PID {} for cleanup", pid));

                *self.script_session.lock() = Some(session);

                // Create async_channel for script thread to send prompt messages to UI (event-driven)
                // P1-6: Use bounded channel to prevent unbounded memory growth from slow UI
                // Capacity of 100 is generous (scripts rarely send > 10 messages/sec)
                let (tx, rx) = async_channel::bounded(100);
                let rx_for_listener = rx.clone();
                self.prompt_receiver = Some(rx);

                // Spawn event-driven listener for prompt messages (replaces 50ms polling)
                cx.spawn(async move |this, cx| {
                    logging::log("EXEC", "Prompt message listener started (event-driven)");

                    // Event-driven: recv().await yields until a message arrives
                    while let Ok(msg) = rx_for_listener.recv().await {
                        logging::log("EXEC", &format!("Prompt message received: {:?}", msg));
                        let _ = cx.update(|cx| {
                            this.update(cx, |app, cx| {
                                app.handle_prompt_message(msg, cx);
                            })
                        });
                    }

                    logging::log("EXEC", "Prompt message listener exiting (channel closed)");
                })
                .detach();

                // We need separate threads for reading and writing to avoid deadlock
                // The read thread blocks on receive_message(), so we can't check for responses in the same loop

                // Take ownership of the session and split it
                let session = match take_active_script_session(
                    &self.script_session,
                    &script.name,
                    &script.path,
                ) {
                    Ok(session) => session,
                    Err(error) => {
                        logging::log("EXEC", &error);
                        self.last_output = Some(SharedString::from(format!("✗ Error: {}", error)));
                        cx.notify();
                        return;
                    }
                };
                let split = session.split();

                let mut stdin = split.stdin;
                let mut stdout_reader = split.stdout_reader;
                // Capture stderr for error reporting - we'll read it in real-time for debugging
                let stderr_handle = split.stderr;
                // CRITICAL: Keep process_handle and child alive - they kill the process on drop!
                // We move them into the reader thread so they live until the script exits.
                let _process_handle = split.process_handle;
                let mut _child = split.child;

                // Stderr reader thread - tees output to both logs AND a ring buffer
                // The buffer is used for post-mortem error reporting when script exits non-zero
                // FIX: Previously we consumed stderr in a thread but passed None to reader,
                // which meant stderr was never available for error messages. Now we use
                // spawn_stderr_reader which returns a StderrCapture containing both the buffer
                // AND a JoinHandle so we can wait for stderr to fully drain before reading.
                let stderr_capture = stderr_handle.map(|stderr| {
                    executor::spawn_stderr_reader(stderr, script_path_for_errors.clone())
                });

                // Move the capture into the reader thread - it owns both buffer and join handle
                // The reader thread will wait for stderr to drain before reading contents

                // Channel for sending responses from UI to writer thread
                // FIX: Use bounded channel to prevent OOM from slow script/blocked stdin
                // Capacity of 100 matches the prompt channel - generous for normal use
                // If the script isn't reading stdin, backpressure will block senders
                let (response_tx, response_rx) = mpsc::sync_channel::<Message>(100);

                // Clone response_tx for the reader thread to handle direct responses
                // (e.g., getSelectedText, setSelectedText, checkAccessibility)
                let reader_response_tx = response_tx.clone();

                // Writer thread - handles sending responses to script
