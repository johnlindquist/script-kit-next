use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::OnceLock;

static LOG_FILE: OnceLock<Mutex<File>> = OnceLock::new();

pub fn init() {
    let path = std::env::temp_dir().join("script-kit-gpui.log");
    println!("========================================");
    println!("[SCRIPT-KIT-GPUI] Log file: {}", path.display());
    println!("========================================");
    
    if let Ok(file) = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)  // Start fresh each run
        .open(&path)
    {
        let _ = LOG_FILE.set(Mutex::new(file));
        log("APP", "Application started");
    }
}

pub fn log(category: &str, message: &str) {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    
    let line = format!("[{}] [{}] {}", timestamp, category, message);
    
    // Always print to stdout for immediate feedback
    println!("{}", line);
    
    // Also write to file for AI to read
    if let Some(mutex) = LOG_FILE.get() {
        if let Ok(mut file) = mutex.lock() {
            let _ = writeln!(file, "{}", line);
            let _ = file.flush();
        }
    }
}

pub fn log_path() -> std::path::PathBuf {
    std::env::temp_dir().join("script-kit-gpui.log")
}
