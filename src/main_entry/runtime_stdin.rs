// External command listener - receives commands via stdin (event-driven, no polling)
let stdin_rx = start_stdin_listener();
let window_for_stdin = window;
let app_entity_for_stdin = app_entity.clone();

// Track if we've received any stdin commands (for timeout warning)
static STDIN_RECEIVED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

// Spawn a timeout warning task - helps AI agents detect when they forgot to use stdin protocol
cx.spawn(async move |_cx: &mut gpui::AsyncApp| {
    Timer::after(std::time::Duration::from_secs(2)).await;
    if !STDIN_RECEIVED.load(std::sync::atomic::Ordering::SeqCst) {
        logging::log("STDIN", "");
        logging::log(
            "STDIN",
            "╔════════════════════════════════════════════════════════════════════════════╗",
        );
        logging::log(
            "STDIN",
            "║  WARNING: No stdin JSON received after 2 seconds                          ║",
        );
        logging::log(
            "STDIN",
            "║                                                                            ║",
        );
        logging::log(
            "STDIN",
            "║  If you're testing, use the stdin JSON protocol:                          ║",
        );
        logging::log(
            "STDIN",
            "║  echo '{\"type\":\"run\",\"path\":\"...\"}' | ./target/debug/script-kit-gpui     ║",
        );
        logging::log(
            "STDIN",
            "║                                                                            ║",
        );
        logging::log(
            "STDIN",
            "║  Command line args do NOT work:                                           ║",
        );
        logging::log(
            "STDIN",
            "║  ./target/debug/script-kit-gpui test.ts  # WRONG - does nothing!          ║",
        );
        logging::log(
            "STDIN",
            "╚════════════════════════════════════════════════════════════════════════════╝",
        );
        logging::log("STDIN", "");
    }
})
.detach();

cx.spawn(async move |cx: &mut gpui::AsyncApp| {
    logging::log("STDIN", "Async stdin command handler started");

    // Event-driven: recv().await yields until a command arrives
    while let Ok(ExternalCommandEnvelope {
        command: cmd,
        correlation_id,
    }) = stdin_rx.recv().await
    {
        let _guard = logging::set_correlation_id(correlation_id);
        // Mark that we've received stdin (clears the timeout warning)
        STDIN_RECEIVED.store(true, std::sync::atomic::Ordering::SeqCst);
        logging::log(
            "STDIN",
            &format!("Processing external command type={}", cmd.command_type()),
        );

        let app_entity_inner = app_entity_for_stdin.clone();
        let _ = cx.update(|cx| {
            // Use the Root window to get Window reference, then update the app entity
            let _ = window_for_stdin.update(cx, |_root, window, root_cx| {
                app_entity_inner.update(root_cx, |view, ctx| {
                    // Note: We have both `window` from Root and `view` from entity here
                    // ctx is Context<ScriptListApp>, window is &mut Window
                    match cmd {
                        include!("runtime_stdin_match_core.rs");
                        include!("runtime_stdin_match_simulate_key.rs");
                        include!("runtime_stdin_match_tail.rs");
                    }
                    ctx.notify();
                }); // close app_entity_inner.update
            }); // close window_for_stdin.update
        }); // close cx.update
    }

    logging::log("STDIN", "Async stdin command handler exiting");
})
.detach();
