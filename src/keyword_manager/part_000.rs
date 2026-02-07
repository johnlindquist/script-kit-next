use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use anyhow::Result;
use tracing::{debug, error, info, instrument, warn};
// Import from crate (these are declared in main.rs)
use crate::keyboard_monitor::{KeyEvent, KeyboardMonitor, KeyboardMonitorError};
use crate::keystroke_logger::keystroke_logger;
use crate::keyword_matcher::KeywordMatcher;
use crate::scripts::load_scriptlets;
use crate::template_variables::substitute_variables;
use crate::text_injector::{TextInjector, TextInjectorConfig};
/// Delay after stopping monitor before performing expansion (ms)
const STOP_DELAY_MS: u64 = 50;
/// Delay after expansion before restarting monitor (ms)
const RESTART_DELAY_MS: u64 = 100;
/// Configuration for the keyword manager
#[derive(Debug, Clone)]
pub struct KeywordManagerConfig {
    /// Configuration for text injection timing
    pub injector_config: TextInjectorConfig,
    /// Delay after stopping monitor before expansion (ms)
    pub stop_delay_ms: u64,
    /// Delay after expansion before restarting monitor (ms)
    #[allow(dead_code)]
    pub restart_delay_ms: u64,
}
impl Default for KeywordManagerConfig {
    fn default() -> Self {
        Self {
            injector_config: TextInjectorConfig::default(),
            stop_delay_ms: STOP_DELAY_MS,
            restart_delay_ms: RESTART_DELAY_MS,
        }
    }
}
/// Stored scriptlet information for expansion
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct KeywordScriptlet {
    /// The trigger keyword (e.g., ":sig")
    trigger: String,
    /// The scriptlet name
    name: String,
    /// The replacement text (scriptlet body)
    content: String,
    /// Tool type (for future use - execute vs paste)
    tool: String,
    /// Source file path (for debugging)
    source_path: Option<String>,
}
/// Manages the text expansion system
///
/// Coordinates keyboard monitoring, trigger detection, and text injection
/// to provide system-wide text expansion functionality.
pub struct KeywordManager {
    /// Configuration
    config: KeywordManagerConfig,
    /// Registered scriptlets by trigger keyword
    scriptlets: Arc<Mutex<HashMap<String, KeywordScriptlet>>>,
    /// The expand matcher for trigger detection
    matcher: Arc<Mutex<KeywordMatcher>>,
    /// Reverse lookup: file path -> set of triggers from that file
    /// Used for efficient clearing/updating of triggers when a file changes
    file_triggers: Arc<Mutex<HashMap<PathBuf, HashSet<String>>>>,
    /// The keyboard monitor (optional - created on enable)
    monitor: Option<KeyboardMonitor>,
    /// The text injector (reserved for future direct use)
    #[allow(dead_code)]
    injector: TextInjector,
    /// Whether the keyword system is enabled
    enabled: bool,
}
