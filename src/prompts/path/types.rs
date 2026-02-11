use super::*;

/// Callback for prompt submission
/// Signature: (id: String, value: Option<String>)
pub type SubmitCallback = Arc<dyn Fn(String, Option<String>) + Send + Sync>;

/// Events emitted by PathPrompt for parent handling
/// Uses GPUI's EventEmitter pattern instead of mutex polling
#[derive(Debug, Clone)]
pub enum PathPromptEvent {
    /// Request to show actions dialog for the given path
    ShowActions(PathInfo),
    /// Request to close actions dialog
    CloseActions,
}

/// Information about a file/folder path for context-aware actions
/// Used for path-specific actions in the actions dialog
#[derive(Debug, Clone)]
pub struct PathInfo {
    /// Display name of the file/folder
    pub name: String,
    /// Full path to the file/folder
    pub path: String,
    /// Whether this is a directory
    pub is_dir: bool,
}

impl PathInfo {
    pub fn new(name: impl Into<String>, path: impl Into<String>, is_dir: bool) -> Self {
        PathInfo {
            name: name.into(),
            path: path.into(),
            is_dir,
        }
    }
}

/// Callback for showing actions dialog
/// Signature: (path_info: PathInfo)
pub type ShowActionsCallback = Arc<dyn Fn(PathInfo) + Send + Sync>;

/// Callback for closing actions dialog (toggle behavior)
/// Signature: ()
pub type CloseActionsCallback = Arc<dyn Fn() + Send + Sync>;

/// PathPrompt - File/folder picker
///
/// Provides a file browser interface for selecting files or directories.
/// Supports starting from a specified path and filtering by name.
pub struct PathPrompt {
    /// Unique ID for this prompt instance
    pub id: String,
    /// Starting directory path (defaults to home if None)
    pub start_path: Option<String>,
    /// Hint text to display
    pub hint: Option<String>,
    /// Current directory being browsed
    pub current_path: String,
    /// Cached "{current_path}/" prefix for header rendering
    pub path_prefix: String,
    /// Filter text for narrowing down results
    pub filter_text: String,
    /// Currently selected index in the list
    pub selected_index: usize,
    /// List of entries in current directory
    pub entries: Vec<PathEntry>,
    /// Filtered entries based on filter_text
    pub filtered_entries: Vec<PathEntry>,
    /// Focus handle for keyboard input
    pub focus_handle: FocusHandle,
    /// Callback when user submits a selection
    pub on_submit: SubmitCallback,
    /// Theme for styling
    pub theme: Arc<theme::Theme>,
    /// Design variant for styling
    pub design_variant: DesignVariant,
    /// Scroll handle for the list
    pub list_scroll_handle: UniformListScrollHandle,
    /// Optional callback to show actions dialog
    pub on_show_actions: Option<ShowActionsCallback>,
    /// Optional callback to close actions dialog (for toggle behavior)
    pub on_close_actions: Option<CloseActionsCallback>,
    /// Shared state tracking if actions dialog is currently showing
    /// Used by PathPrompt to implement toggle behavior for Cmd+K
    pub actions_showing: Arc<Mutex<bool>>,
    /// Shared state for actions search text (displayed in header when actions showing)
    pub actions_search_text: Arc<Mutex<String>>,
    /// Whether to show blinking cursor (for focused state)
    pub cursor_visible: bool,
}

/// A file system entry (file or directory)
#[derive(Clone, Debug)]
pub struct PathEntry {
    /// Display name
    pub name: String,
    /// Full path
    pub path: String,
    /// Whether this is a directory
    pub is_dir: bool,
}
