app.run(move |cx: &mut App| {
    include!("runtime_init.rs");
    include!("runtime_window.rs");
    include!("runtime_tray_hotkeys.rs");
    include!("runtime_watchers_scheduler.rs");
    include!("runtime_stdin.rs");
    include!("runtime_shutdown.rs");
});
