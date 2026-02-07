use serde::{Deserialize, Serialize};

// ============================================================
// DEBUG GRID OVERLAY
// ============================================================

/// Options for the debug grid overlay
///
/// Used with ShowGrid message to configure the visual debugging overlay
/// that displays grid lines, component bounds, and alignment guides.
///
/// # Note on Default
/// The `Default` implementation manually matches the serde defaults to ensure
/// consistency between `GridOptions::default()` (Rust code) and deserialized
/// defaults (from JSON with missing fields).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GridOptions {
    /// Grid line spacing in pixels (8 or 16)
    #[serde(default = "default_grid_size")]
    pub grid_size: u32,

    /// Show component bounding boxes with labels
    #[serde(default)]
    pub show_bounds: bool,

    /// Show CSS box model (padding/margin) visualization
    #[serde(default)]
    pub show_box_model: bool,

    /// Show alignment guides between components
    #[serde(default)]
    pub show_alignment_guides: bool,

    /// Show component dimensions in labels (e.g., "Header (500x45)")
    #[serde(default)]
    pub show_dimensions: bool,

    /// Which components to show bounds for
    /// - "prompts": Top-level prompts only
    /// - "all": All rendered elements
    /// - ["name1", "name2"]: Specific component names
    #[serde(default)]
    pub depth: GridDepthOption,

    /// Optional custom color scheme
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color_scheme: Option<GridColorScheme>,
}

fn default_grid_size() -> u32 {
    8
}

/// Manual Default implementation to match serde defaults exactly.
/// This ensures GridOptions::default() produces the same values as
/// deserializing an empty JSON object {}.
impl Default for GridOptions {
    fn default() -> Self {
        Self {
            grid_size: default_grid_size(), // 8, not 0
            show_bounds: false,
            show_box_model: false,
            show_alignment_guides: false,
            show_dimensions: false,
            depth: GridDepthOption::default(),
            color_scheme: None,
        }
    }
}

/// Depth option for grid bounds display
///
/// Controls which components have their bounding boxes shown.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum GridDepthOption {
    /// Preset mode: "prompts" or "all"
    Preset(String),
    /// Specific component names to show bounds for
    Components(Vec<String>),
}

impl Default for GridDepthOption {
    fn default() -> Self {
        GridDepthOption::Preset("prompts".to_string())
    }
}

/// Custom color scheme for the debug grid overlay
///
/// All colors are in "#RRGGBBAA" or "#RRGGBB" hex format.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GridColorScheme {
    /// Color for grid lines
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grid_lines: Option<String>,

    /// Color for prompt bounding boxes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_bounds: Option<String>,

    /// Color for input bounding boxes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_bounds: Option<String>,

    /// Color for button bounding boxes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub button_bounds: Option<String>,

    /// Color for list bounding boxes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list_bounds: Option<String>,

    /// Fill color for padding visualization
    #[serde(skip_serializing_if = "Option::is_none")]
    pub padding_fill: Option<String>,

    /// Fill color for margin visualization
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin_fill: Option<String>,

    /// Color for alignment guide lines
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alignment_guide: Option<String>,
}

// ============================================================
// ERROR DATA
// ============================================================

/// Script error data for structured error reporting
///
/// Sent when a script execution fails, providing detailed error information
/// for display in the UI with actionable suggestions.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ScriptErrorData {
    /// User-friendly error message
    pub error_message: String,
    /// Raw stderr output if available
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stderr_output: Option<String>,
    /// Process exit code if available
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    /// Parsed stack trace if available
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stack_trace: Option<String>,
    /// Path to the script that failed
    pub script_path: String,
    /// Actionable fix suggestions
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub suggestions: Vec<String>,
    /// When the error occurred (ISO 8601 format)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
}

impl ScriptErrorData {
    /// Create a new ScriptErrorData with required fields
    pub fn new(error_message: String, script_path: String) -> Self {
        ScriptErrorData {
            error_message,
            stderr_output: None,
            exit_code: None,
            stack_trace: None,
            script_path,
            suggestions: Vec::new(),
            timestamp: None,
        }
    }

    /// Add stderr output
    pub fn with_stderr(mut self, stderr: String) -> Self {
        self.stderr_output = Some(stderr);
        self
    }

    /// Add exit code
    pub fn with_exit_code(mut self, code: i32) -> Self {
        self.exit_code = Some(code);
        self
    }

    /// Add stack trace
    pub fn with_stack_trace(mut self, trace: String) -> Self {
        self.stack_trace = Some(trace);
        self
    }

    /// Add suggestions
    pub fn with_suggestions(mut self, suggestions: Vec<String>) -> Self {
        self.suggestions = suggestions;
        self
    }

    /// Add a single suggestion
    pub fn add_suggestion(mut self, suggestion: String) -> Self {
        self.suggestions.push(suggestion);
        self
    }

    /// Add timestamp
    pub fn with_timestamp(mut self, timestamp: String) -> Self {
        self.timestamp = Some(timestamp);
        self
    }
}

// ============================================================
// LAYOUT INFO (AI Agent Debugging)
// ============================================================

/// Computed box model for a component (padding, margin, gap)
///
/// All values are in pixels. This provides the "why" behind spacing -
/// AI agents can understand if space comes from padding, margin, or gap.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ComputedBoxModel {
    /// Padding values (inner spacing)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub padding: Option<BoxModelSides>,
    /// Margin values (outer spacing)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin: Option<BoxModelSides>,
    /// Gap between flex/grid children
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gap: Option<f32>,
}

/// Box model sides (top, right, bottom, left)
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct BoxModelSides {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl BoxModelSides {
    pub fn uniform(value: f32) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    pub fn symmetric(vertical: f32, horizontal: f32) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }
}

/// Computed flex properties for a component
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ComputedFlexStyle {
    /// Flex direction: "row" or "column"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<String>,
    /// Flex grow value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grow: Option<f32>,
    /// Flex shrink value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shrink: Option<f32>,
    /// Align items: "start", "center", "end", "stretch"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub align_items: Option<String>,
    /// Justify content: "start", "center", "end", "space-between", etc.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub justify_content: Option<String>,
}

/// Bounding rectangle in pixels
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct LayoutBounds {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Component type for categorization
///
/// # Forward Compatibility
/// The `Unknown` variant with `#[serde(other)]` ensures forward compatibility:
/// if a newer protocol version adds new component types, older receivers
/// will deserialize them as `Unknown` instead of failing entirely.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum LayoutComponentType {
    Prompt,
    Input,
    Button,
    List,
    ListItem,
    Header,
    #[default]
    Container,
    Panel,
    Other,
    /// Unknown component type (forward compatibility fallback)
    /// When deserializing, any unrecognized type string becomes Unknown
    #[serde(other)]
    Unknown,
}

/// Information about a single component in the layout tree
///
/// This is the core data structure for `getLayoutInfo()`.
/// It provides everything an AI agent needs to understand "why"
/// a component is positioned/sized the way it is.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LayoutComponentInfo {
    /// Component name/identifier
    pub name: String,
    /// Component type for categorization
    #[serde(rename = "type")]
    pub component_type: LayoutComponentType,
    /// Bounding rectangle (absolute position and size)
    pub bounds: LayoutBounds,
    /// Computed box model (padding, margin, gap)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub box_model: Option<ComputedBoxModel>,
    /// Computed flex properties
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flex: Option<ComputedFlexStyle>,
    /// Nesting depth (0 = root, 1 = child of root, etc.)
    pub depth: u32,
    /// Parent component name (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    /// Child component names
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<String>,
    /// Human-readable explanation of why this component has its current size/position
    /// Example: "Height is 45px = padding(8) + content(28) + padding(8) + divider(1)"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explanation: Option<String>,
}

impl LayoutComponentInfo {
    pub fn new(name: impl Into<String>, component_type: LayoutComponentType) -> Self {
        Self {
            name: name.into(),
            component_type,
            bounds: LayoutBounds::default(),
            box_model: None,
            flex: None,
            depth: 0,
            parent: None,
            children: Vec::new(),
            explanation: None,
        }
    }

    pub fn with_bounds(mut self, x: f32, y: f32, width: f32, height: f32) -> Self {
        self.bounds = LayoutBounds {
            x,
            y,
            width,
            height,
        };
        self
    }

    pub fn with_padding(mut self, top: f32, right: f32, bottom: f32, left: f32) -> Self {
        let box_model = self.box_model.get_or_insert_with(ComputedBoxModel::default);
        box_model.padding = Some(BoxModelSides {
            top,
            right,
            bottom,
            left,
        });
        self
    }

    pub fn with_margin(mut self, top: f32, right: f32, bottom: f32, left: f32) -> Self {
        let box_model = self.box_model.get_or_insert_with(ComputedBoxModel::default);
        box_model.margin = Some(BoxModelSides {
            top,
            right,
            bottom,
            left,
        });
        self
    }

    pub fn with_gap(mut self, gap: f32) -> Self {
        let box_model = self.box_model.get_or_insert_with(ComputedBoxModel::default);
        box_model.gap = Some(gap);
        self
    }

    pub fn with_flex_column(mut self) -> Self {
        let flex = self.flex.get_or_insert_with(ComputedFlexStyle::default);
        flex.direction = Some("column".to_string());
        self
    }

    pub fn with_flex_row(mut self) -> Self {
        let flex = self.flex.get_or_insert_with(ComputedFlexStyle::default);
        flex.direction = Some("row".to_string());
        self
    }

    pub fn with_flex_grow(mut self, grow: f32) -> Self {
        let flex = self.flex.get_or_insert_with(ComputedFlexStyle::default);
        flex.grow = Some(grow);
        self
    }

    pub fn with_depth(mut self, depth: u32) -> Self {
        self.depth = depth;
        self
    }

    pub fn with_parent(mut self, parent: impl Into<String>) -> Self {
        self.parent = Some(parent.into());
        self
    }

    pub fn with_explanation(mut self, explanation: impl Into<String>) -> Self {
        self.explanation = Some(explanation.into());
        self
    }
}

/// Full layout information for the current UI state
///
/// Returned by `getLayoutInfo()` SDK function.
/// Contains the component tree and window-level information.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LayoutInfo {
    /// Window dimensions
    pub window_width: f32,
    pub window_height: f32,
    /// Current prompt type (e.g., "arg", "div", "editor", "mainMenu")
    pub prompt_type: String,
    /// All components in the layout tree
    pub components: Vec<LayoutComponentInfo>,
    /// Timestamp when layout was captured (ISO 8601)
    pub timestamp: String,
}
