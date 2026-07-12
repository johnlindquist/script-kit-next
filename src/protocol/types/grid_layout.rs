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

/// Rounded-corner radii in pixels for visual compliance receipts.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LayoutCornerRadius {
    pub top_left: f32,
    pub top_right: f32,
    pub bottom_right: f32,
    pub bottom_left: f32,
}

impl LayoutCornerRadius {
    pub fn uniform(radius: f32) -> Self {
        Self {
            top_left: radius,
            top_right: radius,
            bottom_right: radius,
            bottom_left: radius,
        }
    }
}

/// Visual/style metadata used by DevTools to audit Tahoe/Liquid Glass UI rules.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LayoutVisualStyle {
    /// content, functionalChrome, navigationChrome, floatingTransient, or statusOnly.
    pub chrome_layer: String,
    /// none, solidThemeToken, NSVisualEffectView, NSGlassEffectView, GPUIRgba, or unknown.
    pub material_source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub corner_radius: Option<LayoutCornerRadius>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visual_bounds: Option<LayoutBounds>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hit_bounds: Option<LayoutBounds>,
    /// Internal text/content inset (bezel/edge -> first glyph) for input-like
    /// controls. Lets the Apple-guideline conformance engine MEASURE search-field
    /// padding against the native NSTextField baseline (9pt H / 3pt V on macOS 26)
    /// instead of reporting it as an `unmeasured` gap. Serialized as `contentInsets`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_insets: Option<BoxModelSides>,
    /// Rendered text metrics for text-bearing nodes (e.g. the search input). Lets
    /// the Apple-guideline conformance engine classify font size/weight/line-height
    /// against the measured-native macOS baseline (13pt Regular / 16pt line height)
    /// instead of guessing. Emitted from the SAME accessors the renderer uses, so
    /// the receipt cannot drift from what is actually drawn. Serialized as `typography`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub typography: Option<LayoutTypographyInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exception: Option<String>,
}

/// Rendered typography metrics for a text-bearing layout node.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LayoutTypographyInfo {
    /// Semantic role: searchInput, resultPrimary, resultSecondary, resultMeta.
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_family: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_size_pt: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_weight: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_weight_numeric: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_height_pt: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_align: Option<String>,
}

/// Component type for categorization
///
/// # Forward Compatibility
/// The `Unknown` variant with `#[serde(other)]` ensures forward compatibility:
/// if a newer protocol version adds new component types, older receivers
/// will deserialize them as `Unknown` instead of failing entirely.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub enum LayoutComponentType {
    Prompt,
    Input,
    Button,
    List,
    /// `listItem` in the current protocol, with legacy `listitem` support.
    #[serde(alias = "listitem")]
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
    /// Optional visual/style receipt for Liquid Glass and accessibility audits.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visual_style: Option<LayoutVisualStyle>,
    /// How the bounds were obtained. Fidelity probes require `paint-time`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub measurement_provenance: Option<String>,
    /// Coordinate system for `bounds`; paint receipts use window logical pixels.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coordinate_space: Option<String>,
    /// Rectangular portion of `bounds` that survives the ancestor clip chain.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visible_bounds: Option<LayoutBounds>,
    /// Active ancestor content mask used to derive `visible_bounds`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clip_bounds: Option<LayoutBounds>,
    /// Monotonic identity of the completed GPUI frame that supplied these bounds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub measurement_frame_generation: Option<u64>,
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
            visual_style: None,
            measurement_provenance: None,
            coordinate_space: None,
            visible_bounds: None,
            clip_bounds: None,
            measurement_frame_generation: None,
        }
    }

    pub fn with_measurement(
        mut self,
        provenance: impl Into<String>,
        coordinate_space: impl Into<String>,
    ) -> Self {
        self.measurement_provenance = Some(provenance.into());
        self.coordinate_space = Some(coordinate_space.into());
        self
    }

    pub fn with_measurement_frame(mut self, generation: u64) -> Self {
        self.measurement_frame_generation = Some(generation);
        self
    }

    #[allow(clippy::too_many_arguments)]
    pub fn with_paint_visibility(
        mut self,
        visible_x: f32,
        visible_y: f32,
        visible_width: f32,
        visible_height: f32,
        clip_x: f32,
        clip_y: f32,
        clip_width: f32,
        clip_height: f32,
    ) -> Self {
        self.visible_bounds = Some(LayoutBounds {
            x: visible_x,
            y: visible_y,
            width: visible_width,
            height: visible_height,
        });
        self.clip_bounds = Some(LayoutBounds {
            x: clip_x,
            y: clip_y,
            width: clip_width,
            height: clip_height,
        });
        self
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

    pub fn with_visual_style(
        mut self,
        chrome_layer: impl Into<String>,
        material_source: impl Into<String>,
        corner_radius: Option<f32>,
    ) -> Self {
        self.visual_style = Some(LayoutVisualStyle {
            chrome_layer: chrome_layer.into(),
            material_source: material_source.into(),
            token_source: None,
            corner_radius: corner_radius.map(LayoutCornerRadius::uniform),
            visual_bounds: Some(self.bounds.clone()),
            hit_bounds: Some(self.bounds.clone()),
            content_insets: None,
            typography: None,
            exception: None,
        });
        self
    }

    /// Declare the rendered typography for a text-bearing node so the Apple-guideline
    /// conformance engine can classify it against the measured-native baseline.
    #[allow(clippy::too_many_arguments)]
    pub fn with_typography(
        mut self,
        role: impl Into<String>,
        font_family: Option<String>,
        font_size_pt: f32,
        font_weight: impl Into<String>,
        font_weight_numeric: f32,
        line_height_pt: f32,
        text_align: impl Into<String>,
    ) -> Self {
        let style = self
            .visual_style
            .get_or_insert_with(LayoutVisualStyle::default);
        style.typography = Some(LayoutTypographyInfo {
            role: role.into(),
            font_family,
            font_size_pt: Some(font_size_pt),
            font_weight: Some(font_weight.into()),
            font_weight_numeric: Some(font_weight_numeric),
            line_height_pt: Some(line_height_pt),
            text_align: Some(text_align.into()),
        });
        self
    }

    /// Declare the internal content inset (edge -> first glyph) for an input-like
    /// control so the Apple-guideline conformance engine can measure its padding.
    pub fn with_content_insets(mut self, top: f32, right: f32, bottom: f32, left: f32) -> Self {
        let style = self
            .visual_style
            .get_or_insert_with(LayoutVisualStyle::default);
        style.content_insets = Some(BoxModelSides {
            top,
            right,
            bottom,
            left,
        });
        self
    }

    pub fn with_visual_token(mut self, token_source: impl Into<String>) -> Self {
        let style = self
            .visual_style
            .get_or_insert_with(LayoutVisualStyle::default);
        style.token_source = Some(token_source.into());
        self
    }

    pub fn with_hit_bounds(mut self, x: f32, y: f32, width: f32, height: f32) -> Self {
        let style = self
            .visual_style
            .get_or_insert_with(LayoutVisualStyle::default);
        style.hit_bounds = Some(LayoutBounds {
            x,
            y,
            width,
            height,
        });
        self
    }

    pub fn with_visual_exception(mut self, exception: impl Into<String>) -> Self {
        let style = self
            .visual_style
            .get_or_insert_with(LayoutVisualStyle::default);
        style.exception = Some(exception.into());
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
    /// Handler-form-specific layout details for DevTools focus/scroll receipts.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "handlerForm"
    )]
    pub handler_form: Option<serde_json::Value>,
    /// Timestamp when layout was captured (ISO 8601)
    pub timestamp: String,
}

#[cfg(test)]
mod tests {
    use super::{LayoutComponentInfo, LayoutComponentType};

    #[test]
    fn visual_style_serializes_for_devtools_receipts() {
        let component = LayoutComponentInfo::new("SearchInput", LayoutComponentType::Input)
            .with_bounds(16.0, 11.0, 534.0, 22.0)
            .with_visual_style("functionalChrome", "solidThemeToken", Some(14.0))
            .with_visual_token("chrome.searchInput")
            .with_hit_bounds(16.0, 8.0, 534.0, 28.0);

        let json = serde_json::to_value(component).expect("serialize visual style component");
        let style = json
            .get("visualStyle")
            .expect("visualStyle should be emitted");

        assert_eq!(style["chromeLayer"], "functionalChrome");
        assert_eq!(style["materialSource"], "solidThemeToken");
        assert_eq!(style["tokenSource"], "chrome.searchInput");
        assert_eq!(style["cornerRadius"]["topLeft"], 14.0);
        assert_eq!(style["visualBounds"]["height"], 22.0);
        assert_eq!(style["hitBounds"]["height"], 28.0);
    }

    #[test]
    fn visual_exception_marks_dense_controls() {
        let component = LayoutComponentInfo::new("LogoButton", LayoutComponentType::Button)
            .with_bounds(714.0, 8.0, 20.0, 28.0)
            .with_visual_style("functionalChrome", "solidThemeToken", Some(14.0))
            .with_hit_bounds(714.0, 8.0, 28.0, 28.0)
            .with_visual_exception("compactIconButton");

        let json = serde_json::to_value(component).expect("serialize visual exception");
        assert_eq!(json["visualStyle"]["exception"], "compactIconButton");
        assert_eq!(json["visualStyle"]["hitBounds"]["width"], 28.0);
    }

    #[test]
    fn paint_measurement_metadata_serializes_explicitly() {
        let component =
            LayoutComponentInfo::new("agent-chat-composer-input", LayoutComponentType::Input)
                .with_bounds(12.0, 34.0, 400.0, 28.0)
                .with_measurement("paint-time", "window")
                .with_paint_visibility(12.0, 40.0, 400.0, 22.0, 0.0, 40.0, 750.0, 404.0)
                .with_measurement_frame(42);

        let json = serde_json::to_value(component).expect("serialize paint measurement");
        assert_eq!(json["measurementProvenance"], "paint-time");
        assert_eq!(json["coordinateSpace"], "window");
        assert_eq!(json["measurementFrameGeneration"], 42);
        assert_eq!(json["visibleBounds"]["y"], 40.0);
        assert_eq!(json["clipBounds"]["height"], 404.0);
    }
}
