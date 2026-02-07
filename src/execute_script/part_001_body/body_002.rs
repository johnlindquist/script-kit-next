                // Reader thread - handles receiving messages from script (blocking is OK here)
                // CRITICAL: Move _process_handle and _child into this thread to keep them alive!
                // When the reader thread exits, they'll be dropped and the process killed.
                let script_path_clone = script_path_for_errors.clone();
                std::thread::spawn(move || {
                    include!("reader_thread_000.rs");
                    include!("reader_thread_001.rs");
                    include!("reader_thread_002.rs");
                });

                // Store the response sender for the UI to use
                self.response_sender = Some(response_tx);
