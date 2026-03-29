//! Test Enter key behavior with Claude Code under different terminal configs.
//! Run: cargo run --example pty_enter_test

use alacritty_terminal::event::{Event as AlacrittyEvent, EventListener};
use alacritty_terminal::grid::Dimensions;
use alacritty_terminal::term::{Config as TermConfig, Term};
use alacritty_terminal::vte::ansi::Processor;
use parking_lot::Mutex as PLMutex;
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::io::{Read, Write};
use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Clone)]
struct Proxy(Arc<PLMutex<Vec<AlacrittyEvent>>>);
impl Proxy {
    fn new() -> Self {
        Self(Arc::new(PLMutex::new(Vec::new())))
    }
    fn take(&self) -> Vec<AlacrittyEvent> {
        std::mem::take(&mut *self.0.lock())
    }
}
impl EventListener for Proxy {
    fn send_event(&self, event: AlacrittyEvent) {
        self.0.lock().push(event);
    }
}
struct Size;
impl Dimensions for Size {
    fn total_lines(&self) -> usize {
        24
    }
    fn screen_lines(&self) -> usize {
        24
    }
    fn columns(&self) -> usize {
        80
    }
}

fn escape_str(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c == '\x1b' {
                "ESC".to_string()
            } else if c.is_control() {
                format!("\\x{:02x}", c as u32)
            } else {
                c.to_string()
            }
        })
        .collect()
}

fn test_with_env(label: &str, term_program: Option<&str>) {
    println!("\n============================================================");
    println!("TEST: {}", label);
    println!("  TERM_PROGRAM={}", term_program.unwrap_or("(unset)"));
    println!("============================================================");

    let pty_system = native_pty_system();
    let size = PtySize {
        rows: 24,
        cols: 80,
        pixel_width: 0,
        pixel_height: 0,
    };
    let pair = pty_system.openpty(size).expect("openpty");

    let mut cmd = CommandBuilder::new("/bin/zsh");
    cmd.env_clear();
    cmd.env("TERM", "xterm-256color");
    cmd.env("COLORTERM", "truecolor");
    for key in ["HOME", "PATH", "USER", "SHELL", "LANG", "TMPDIR"] {
        if let Ok(v) = std::env::var(key) {
            cmd.env(key, v);
        }
    }
    if let Some(tp) = term_program {
        cmd.env("TERM_PROGRAM", tp);
    }

    let mut child = pair.slave.spawn_command(cmd).expect("spawn");
    let reader = pair.master.try_clone_reader().expect("reader");
    let writer = Arc::new(Mutex::new(pair.master.take_writer().expect("writer")));

    // Background reader thread
    let (tx, rx) = mpsc::channel::<Vec<u8>>();
    let _reader_thread = std::thread::spawn(move || {
        let mut reader = reader;
        let mut buf = [0u8; 4096];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    let _ = tx.send(buf[..n].to_vec());
                }
                Err(_) => break,
            }
        }
    });

    let config = TermConfig {
        scrolling_history: 100,
        kitty_keyboard: true,
        ..TermConfig::default()
    };
    let proxy = Proxy::new();
    let mut term = Term::new(config, &Size, proxy.clone());
    let mut proc: Processor = Processor::new();

    // Process PTY output for a duration, writing PtyWrite responses back
    let drain = |rx: &mpsc::Receiver<Vec<u8>>,
                 writer: &Arc<Mutex<Box<dyn Write + Send>>>,
                 term: &mut Term<Proxy>,
                 proc: &mut Processor,
                 proxy: &Proxy,
                 secs: u64,
                 label: &str|
     -> String {
        let start = Instant::now();
        let dur = Duration::from_secs(secs);
        let mut all = Vec::new();
        let mut writes = Vec::new();
        while start.elapsed() < dur {
            match rx.recv_timeout(Duration::from_millis(100)) {
                Ok(data) => {
                    all.extend_from_slice(&data);
                    proc.advance(term, &data);
                    for ev in proxy.take() {
                        if let AlacrittyEvent::PtyWrite(text) = ev {
                            writes.push(text.clone());
                            if let Ok(mut w) = writer.lock() {
                                let _ = w.write_all(text.as_bytes());
                                let _ = w.flush();
                            }
                        }
                    }
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
        if !writes.is_empty() {
            println!("  [{label}] PtyWrite responses:");
            for w in &writes {
                println!("    -> {}", escape_str(w));
            }
        }
        String::from_utf8_lossy(&all).to_string()
    };

    let send = |writer: &Arc<Mutex<Box<dyn Write + Send>>>, data: &[u8]| {
        if let Ok(mut w) = writer.lock() {
            let _ = w.write_all(data);
            let _ = w.flush();
        }
    };

    // Wait for shell
    drain(&rx, &writer, &mut term, &mut proc, &proxy, 2, "shell-init");

    // Start claude interactively
    println!("  Starting: claude");
    send(&writer, b"claude\r");

    // Wait for Claude to initialize (it takes a few seconds)
    let init_output = drain(
        &rx,
        &writer,
        &mut term,
        &mut proc,
        &proxy,
        10,
        "claude-init",
    );

    // Check what Claude detected
    let has_kitty_push = init_output.contains("\x1b[>1u")
        || init_output.contains("\x1b[>") && init_output.contains("u");
    let has_bracketed = init_output.contains("\x1b[?2004h");
    let has_prompt =
        init_output.contains("❯") || init_output.contains(">") || init_output.contains("Claude");
    println!(
        "  Claude init: kitty_push={} bracketed_paste={} has_prompt={}",
        has_kitty_push, has_bracketed, has_prompt
    );

    // Type "hello" then send \r (Enter)
    println!("  Sending: 'hello' + \\r");
    send(&writer, b"hello");
    std::thread::sleep(Duration::from_millis(300));
    send(&writer, b"\r");

    let after_enter = drain(&rx, &writer, &mut term, &mut proc, &proxy, 8, "after-enter");

    // Analyze: did it submit (thinking/processing) or just newline?
    let submitted = after_enter.contains("Thinking")
        || after_enter.contains("thinking")
        || after_enter.contains("⏳")
        || after_enter.contains("◐")
        || after_enter.contains("Claude") && after_enter.contains("hello");
    println!("  RESULT: Enter submitted = {}", submitted);

    // Show readable output
    let clean: String = after_enter
        .chars()
        .filter(|c| c.is_ascii_graphic() || c.is_ascii_whitespace())
        .collect();
    let lines: Vec<&str> = clean.lines().filter(|l| !l.trim().is_empty()).collect();
    println!("  Output after Enter ({} lines):", lines.len());
    for line in lines.iter().take(10) {
        println!("    | {}", line.trim());
    }

    // Cleanup
    send(&writer, b"\x03"); // Ctrl+C
    std::thread::sleep(Duration::from_millis(500));
    send(&writer, b"\x03");
    std::thread::sleep(Duration::from_millis(300));
    send(&writer, b"exit\r");
    std::thread::sleep(Duration::from_millis(500));
    let _ = child.kill();
    let _ = child.wait();
    println!("  Done.");
}

fn main() {
    println!("PTY Enter Key Test -- Claude Code Interactive");
    println!("==============================================\n");

    test_with_env("No TERM_PROGRAM", None);
    test_with_env("TERM_PROGRAM=xterm", Some("xterm"));

    println!("\n=== All tests complete ===");
}
