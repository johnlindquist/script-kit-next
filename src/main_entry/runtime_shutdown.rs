        // Shutdown monitor task - checks SHUTDOWN_REQUESTED flag set by signal handler
        // Performs all cleanup on the main thread where it's safe to call logging,
        // mutexes, and other non-async-signal-safe functions.
        cx.spawn(async move |cx: &mut gpui::AsyncApp| {
            loop {
                // Check every 500ms for shutdown signal
                // 500ms is acceptable latency for graceful shutdown while reducing CPU wakeups
                Timer::after(std::time::Duration::from_millis(500)).await;

                if SHUTDOWN_REQUESTED.load(Ordering::SeqCst) {
                    logging::log("SHUTDOWN", "Shutdown signal detected, performing graceful cleanup");

                    // Kill all tracked child processes
                    logging::log("SHUTDOWN", "Killing all child processes");
                    PROCESS_MANAGER.kill_all_processes();

                    // Remove main PID file
                    PROCESS_MANAGER.remove_main_pid();

                    logging::log("SHUTDOWN", "Cleanup complete, quitting application");

                    // Quit the GPUI application
                    let _ = cx.update(|cx| {
                        cx.quit();
                    });

                    break;
                }
            }
        }).detach();

        logging::log("APP", "Application ready - Cmd+; to show, Esc to hide, Cmd+K for actions");
