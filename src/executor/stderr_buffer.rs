//! Stderr Ring Buffer for Script Error Capture
//!
//! This module provides a thread-safe ring buffer for capturing stderr output
//! from script processes. The buffer is used for post-mortem error reporting
//! when a script exits with a non-zero status.
//!
//! ## Design
//!
//! The buffer captures the most recent stderr output (up to a configurable limit)
//! while simultaneously forwarding to the logging system. This "tee" approach
//! allows real-time debugging while preserving error context for exit handling.
//!
//! ## Thread Safety
//!
//! The buffer uses `Arc<Mutex<VecDeque<String>>>` for thread-safe access from:
//! - The stderr reader thread (writes)
//! - The main reader thread on script exit (reads)

use std::collections::VecDeque;
use std::io::{BufRead, BufReader, Read};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use tracing::{debug, warn};

/// Default maximum number of lines to buffer
pub const DEFAULT_MAX_LINES: usize = 500;

/// Default maximum total bytes to buffer (4KB)
pub const DEFAULT_MAX_BYTES: usize = 4 * 1024;

/// A thread-safe ring buffer for stderr lines
#[derive(Debug, Clone)]
pub struct StderrBuffer {
    /// Buffered lines (newest at back)
    lines: Arc<Mutex<VecDeque<String>>>,
    /// Maximum number of lines to keep
    max_lines: usize,
    /// Maximum total bytes to keep (approximate)
    max_bytes: usize,
    /// Current byte count (approximate)
    current_bytes: Arc<Mutex<usize>>,
}

impl Default for StderrBuffer {
    fn default() -> Self {
        Self::new(DEFAULT_MAX_LINES, DEFAULT_MAX_BYTES)
    }
}

// Allow dead code - these methods are part of the public API and used in tests
#[allow(dead_code)]
impl StderrBuffer {
    /// Create a new stderr buffer with specified limits
    pub fn new(max_lines: usize, max_bytes: usize) -> Self {
        Self {
            lines: Arc::new(Mutex::new(VecDeque::with_capacity(max_lines.min(1024)))),
            max_lines,
            max_bytes,
            current_bytes: Arc::new(Mutex::new(0)),
        }
    }

    /// Add a line to the buffer, evicting old lines if necessary
    pub fn push_line(&self, line: String) {
        let line_bytes = line.len();

        let mut lines = self.lines.lock().unwrap();
        let mut current = self.current_bytes.lock().unwrap();

        // Evict old lines if we're over the byte limit
        while *current + line_bytes > self.max_bytes && !lines.is_empty() {
            if let Some(old) = lines.pop_front() {
                *current = current.saturating_sub(old.len());
            }
        }

        // Evict old lines if we're over the line limit
        while lines.len() >= self.max_lines {
            if let Some(old) = lines.pop_front() {
                *current = current.saturating_sub(old.len());
            }
        }

        // Add the new line
        *current += line_bytes;
        lines.push_back(line);
    }

    /// Get all buffered lines as a single string
    pub fn get_contents(&self) -> String {
        let lines = self.lines.lock().unwrap();
        lines.iter().cloned().collect::<Vec<_>>().join("\n")
    }

    /// Get the last N lines (or all if fewer exist)
    pub fn get_last_n_lines(&self, n: usize) -> Vec<String> {
        let lines = self.lines.lock().unwrap();
        let skip = lines.len().saturating_sub(n);
        lines.iter().skip(skip).cloned().collect()
    }

    /// Get the number of buffered lines
    pub fn len(&self) -> usize {
        self.lines.lock().unwrap().len()
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.lines.lock().unwrap().is_empty()
    }

    /// Clear the buffer
    pub fn clear(&self) {
        let mut lines = self.lines.lock().unwrap();
        let mut current = self.current_bytes.lock().unwrap();
        lines.clear();
        *current = 0;
    }

    /// Get approximate byte count
    pub fn byte_count(&self) -> usize {
        *self.current_bytes.lock().unwrap()
    }
}

/// Stderr capture handle containing both buffer and thread handle
///
/// This struct bundles the stderr buffer with the thread's JoinHandle,
/// allowing callers to wait for the stderr reader to complete before
/// snapshotting the buffer contents. This prevents the race condition
/// where stderr is read before all error output has been buffered.
#[derive(Debug)]
pub struct StderrCapture {
    /// The stderr ring buffer
    pub buffer: StderrBuffer,
    /// Handle to wait for the reader thread to complete
    pub join_handle: JoinHandle<()>,
}

impl StderrCapture {
    /// Wait for the stderr reader thread to complete, with timeout
    ///
    /// Returns true if the thread completed within the timeout, false otherwise.
    /// This should be called before reading the buffer to ensure all stderr
    /// has been captured.
    pub fn wait_with_timeout(&self, timeout: Duration) -> bool {
        // We can't join without consuming, so we poll is_finished
        let start = std::time::Instant::now();
        let poll_interval = Duration::from_millis(10);

        while start.elapsed() < timeout {
            if self.join_handle.is_finished() {
                return true;
            }
            std::thread::sleep(poll_interval);
        }

        self.join_handle.is_finished()
    }

    /// Get buffer contents after waiting for reader to complete
    ///
    /// Waits up to the specified timeout for the stderr reader to finish,
    /// then returns the buffer contents. This is the recommended way to
    /// read stderr after script exit.
    pub fn get_contents_with_timeout(&self, timeout: Duration) -> String {
        self.wait_with_timeout(timeout);
        self.buffer.get_contents()
    }
}

/// Spawn a stderr reader thread that tees output to both logging and a buffer
///
/// Returns a StderrCapture containing both the buffer handle and thread JoinHandle.
/// The JoinHandle allows callers to wait for the reader to complete before
/// snapshotting the buffer, preventing partial error captures.
///
/// # Example
/// ```ignore
/// let capture = spawn_stderr_reader(stderr, script_path);
/// // ... wait for process to exit ...
/// let stderr_text = capture.get_contents_with_timeout(Duration::from_millis(100));
/// ```
pub fn spawn_stderr_reader<R: Read + Send + 'static>(
    stderr: R,
    script_path: String,
) -> StderrCapture {
    let buffer = StderrBuffer::default();
    let buffer_clone = buffer.clone();

    let join_handle = thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line_result in reader.lines() {
            match line_result {
                Ok(line) => {
                    // Log in real-time
                    debug!(target: "SCRIPT", script_path = %script_path, "{}", line);
                    // Buffer for post-mortem
                    buffer_clone.push_line(line);
                }
                Err(e) => {
                    warn!(target: "SCRIPT", error = %e, "stderr read error");
                    break;
                }
            }
        }
        debug!(target: "SCRIPT", "stderr reader exiting");
    });

    StderrCapture {
        buffer,
        join_handle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_basic_operations() {
        let buffer = StderrBuffer::new(10, 1024);

        // Initially empty
        assert!(buffer.is_empty());
        assert_eq!(buffer.len(), 0);
        assert_eq!(buffer.get_contents(), "");

        // Add some lines
        buffer.push_line("line 1".to_string());
        buffer.push_line("line 2".to_string());
        buffer.push_line("line 3".to_string());

        assert!(!buffer.is_empty());
        assert_eq!(buffer.len(), 3);
        assert_eq!(buffer.get_contents(), "line 1\nline 2\nline 3");
    }

    #[test]
    fn test_buffer_line_limit() {
        let buffer = StderrBuffer::new(3, 1024); // Only keep 3 lines

        buffer.push_line("line 1".to_string());
        buffer.push_line("line 2".to_string());
        buffer.push_line("line 3".to_string());
        buffer.push_line("line 4".to_string()); // Should evict line 1

        assert_eq!(buffer.len(), 3);
        assert_eq!(buffer.get_contents(), "line 2\nline 3\nline 4");
    }

    #[test]
    fn test_buffer_byte_limit() {
        // Each "line X" is 6 bytes, buffer limit is 15 bytes
        let buffer = StderrBuffer::new(100, 15);

        buffer.push_line("line 1".to_string()); // 6 bytes, total 6
        buffer.push_line("line 2".to_string()); // 6 bytes, total 12
        buffer.push_line("line 3".to_string()); // 6 bytes, would be 18, evict line 1

        // Should have evicted "line 1" to make room
        assert_eq!(buffer.len(), 2);
        let contents = buffer.get_contents();
        assert!(!contents.contains("line 1"));
        assert!(contents.contains("line 2"));
        assert!(contents.contains("line 3"));
    }

    #[test]
    fn test_get_last_n_lines() {
        let buffer = StderrBuffer::new(10, 1024);

        buffer.push_line("line 1".to_string());
        buffer.push_line("line 2".to_string());
        buffer.push_line("line 3".to_string());
        buffer.push_line("line 4".to_string());
        buffer.push_line("line 5".to_string());

        // Get last 3 lines
        let last_3 = buffer.get_last_n_lines(3);
        assert_eq!(last_3.len(), 3);
        assert_eq!(last_3[0], "line 3");
        assert_eq!(last_3[1], "line 4");
        assert_eq!(last_3[2], "line 5");

        // Get more than available
        let all = buffer.get_last_n_lines(100);
        assert_eq!(all.len(), 5);
    }

    #[test]
    fn test_buffer_clear() {
        let buffer = StderrBuffer::new(10, 1024);

        buffer.push_line("line 1".to_string());
        buffer.push_line("line 2".to_string());

        assert_eq!(buffer.len(), 2);
        assert!(buffer.byte_count() > 0);

        buffer.clear();

        assert!(buffer.is_empty());
        assert_eq!(buffer.byte_count(), 0);
    }

    #[test]
    fn test_buffer_clone_shares_data() {
        let buffer1 = StderrBuffer::new(10, 1024);
        let buffer2 = buffer1.clone();

        buffer1.push_line("from buffer 1".to_string());
        buffer2.push_line("from buffer 2".to_string());

        // Both should see both lines (they share the same Arc)
        assert_eq!(buffer1.len(), 2);
        assert_eq!(buffer2.len(), 2);
        assert_eq!(buffer1.get_contents(), buffer2.get_contents());
    }

    #[test]
    fn test_buffer_thread_safety() {
        use std::thread;

        let buffer = StderrBuffer::new(1000, 100_000);
        let handles: Vec<_> = (0..10)
            .map(|i| {
                let b = buffer.clone();
                thread::spawn(move || {
                    for j in 0..100 {
                        b.push_line(format!("thread {} line {}", i, j));
                    }
                })
            })
            .collect();

        for h in handles {
            h.join().unwrap();
        }

        // All lines should be present (we have capacity for all 1000)
        assert_eq!(buffer.len(), 1000);
    }

    #[test]
    fn test_default_limits() {
        let buffer = StderrBuffer::default();

        // Verify default limits are reasonable
        assert_eq!(buffer.max_lines, DEFAULT_MAX_LINES);
        assert_eq!(buffer.max_bytes, DEFAULT_MAX_BYTES);
    }

    #[test]
    fn test_empty_line_handling() {
        let buffer = StderrBuffer::new(10, 1024);

        buffer.push_line("".to_string());
        buffer.push_line("non-empty".to_string());
        buffer.push_line("".to_string());

        assert_eq!(buffer.len(), 3);
        assert_eq!(buffer.get_contents(), "\nnon-empty\n");
    }

    #[test]
    fn test_unicode_handling() {
        let buffer = StderrBuffer::new(10, 1024);

        buffer.push_line("Error: 文件未找到".to_string());
        buffer.push_line("🚀 Launch failed".to_string());
        buffer.push_line("Привет мир".to_string());

        assert_eq!(buffer.len(), 3);
        let contents = buffer.get_contents();
        assert!(contents.contains("文件未找到"));
        assert!(contents.contains("🚀"));
        assert!(contents.contains("Привет"));
    }

    // ============================================================
    // StderrCapture Tests
    // ============================================================

    #[test]
    fn test_spawn_stderr_reader_returns_capture_with_buffer() {
        use std::io::Cursor;

        // Create mock stderr with test content
        let stderr_content = "Error line 1\nError line 2\nStack trace here\n";
        let mock_stderr = Cursor::new(stderr_content.as_bytes().to_vec());

        // Spawn the reader
        let capture = spawn_stderr_reader(mock_stderr, "/test/script.ts".to_string());

        // Wait for reader to complete (should be fast for small input)
        let completed = capture.wait_with_timeout(Duration::from_millis(500));
        assert!(completed, "Reader should complete quickly for small input");

        // Buffer should contain all lines
        let contents = capture.buffer.get_contents();
        assert!(contents.contains("Error line 1"), "Should contain first error line");
        assert!(contents.contains("Error line 2"), "Should contain second error line");
        assert!(contents.contains("Stack trace here"), "Should contain stack trace");
    }

    #[test]
    fn test_get_contents_with_timeout_waits_for_completion() {
        use std::io::Cursor;

        // Create mock stderr
        let stderr_content = "Test error output\n";
        let mock_stderr = Cursor::new(stderr_content.as_bytes().to_vec());

        let capture = spawn_stderr_reader(mock_stderr, "/test/script.ts".to_string());

        // get_contents_with_timeout should wait and return content
        let contents = capture.get_contents_with_timeout(Duration::from_millis(500));
        assert!(contents.contains("Test error output"), "Should contain error output");
    }

    #[test]
    fn test_stderr_capture_empty_stderr() {
        use std::io::Cursor;

        // Create empty mock stderr
        let mock_stderr = Cursor::new(Vec::<u8>::new());

        let capture = spawn_stderr_reader(mock_stderr, "/test/script.ts".to_string());

        // Wait for completion
        let completed = capture.wait_with_timeout(Duration::from_millis(100));
        assert!(completed, "Reader should complete for empty input");

        // Buffer should be empty
        let contents = capture.buffer.get_contents();
        assert!(contents.is_empty(), "Empty stderr should produce empty buffer");
    }

    #[test]
    fn test_stderr_capture_large_output() {
        use std::io::Cursor;

        // Create large stderr content
        let mut stderr_content = String::new();
        for i in 0..100 {
            stderr_content.push_str(&format!("Error line {}\n", i));
        }
        let mock_stderr = Cursor::new(stderr_content.as_bytes().to_vec());

        let capture = spawn_stderr_reader(mock_stderr, "/test/script.ts".to_string());

        // Wait for reader to complete
        let completed = capture.wait_with_timeout(Duration::from_millis(1000));
        assert!(completed, "Reader should complete for large input");

        // Buffer should have content (may be truncated due to limits)
        let contents = capture.buffer.get_contents();
        assert!(!contents.is_empty(), "Should have captured some output");
    }

    #[test]
    fn test_wait_with_timeout_returns_true_when_finished() {
        use std::io::Cursor;

        let mock_stderr = Cursor::new(b"quick\n".to_vec());
        let capture = spawn_stderr_reader(mock_stderr, "/test/script.ts".to_string());

        // Give it plenty of time
        let completed = capture.wait_with_timeout(Duration::from_secs(1));
        assert!(completed, "Should return true when thread finished");

        // Subsequent calls should still return true
        let completed2 = capture.wait_with_timeout(Duration::from_millis(1));
        assert!(completed2, "Should return true immediately if already finished");
    }
}
