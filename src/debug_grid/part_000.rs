use gpui::{
    div, px, rgba, Bounds, InteractiveElement, IntoElement, ParentElement, Pixels, Point, Size,
    Styled,
};
/// Configuration for the debug grid overlay
#[derive(Clone, Debug)]
pub struct GridConfig {
    /// Grid spacing in pixels (typically 8 or 16)
    pub grid_size: u32,
    /// Show component bounding boxes
    pub show_bounds: bool,
    /// Show padding/margin visualization (CSS DevTools style)
    pub show_box_model: bool,
    /// Show alignment snap lines between components
    pub show_alignment_guides: bool,
    /// Show component dimensions in labels (e.g., "Run (55x28)")
    pub show_dimensions: bool,
    /// Which components to show bounds for
    pub depth: GridDepth,
    /// Color scheme for the overlay
    pub color_scheme: GridColorScheme,
}
impl Default for GridConfig {
    fn default() -> Self {
        Self {
            grid_size: 8,
            show_bounds: true,
            show_box_model: false,
            show_alignment_guides: true,
            show_dimensions: false,
            depth: GridDepth::Prompts,
            color_scheme: GridColorScheme::default(),
        }
    }
}
/// Controls which components to show in the overlay
#[derive(Clone, Debug, Default)]
pub enum GridDepth {
    /// Top-level prompts only (ArgPrompt, DivPrompt, etc.)
    #[default]
    Prompts,
    /// All rendered elements
    All,
    /// Specific named components
    Components(Vec<String>),
}
/// Box model measurements (padding/margin)
#[derive(Clone, Debug, Default)]
pub struct BoxModel {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}
impl BoxModel {
    /// Create a uniform box model with the same value on all sides
    pub fn uniform(value: f32) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    /// Create a box model with horizontal and vertical values
    pub fn symmetric(vertical: f32, horizontal: f32) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }
}
/// Type of component for color coding bounds
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum ComponentType {
    /// ArgPrompt, DivPrompt, EditorPrompt, etc.
    Prompt,
    /// Text inputs, search box
    Input,
    /// Action buttons
    Button,
    /// Script list, choice list
    List,
    /// Individual list items
    ListItem,
    /// Headers, titles
    Header,
    /// Generic containers
    #[default]
    Container,
    /// Other unclassified components
    Other,
}
/// Bounds info for a component
#[derive(Clone, Debug)]
pub struct ComponentBounds {
    /// Component name/type for labeling
    pub name: String,
    /// Bounding rectangle
    pub bounds: Bounds<Pixels>,
    /// Type of component (for color coding)
    pub component_type: ComponentType,
    /// Padding values (inner spacing)
    pub padding: Option<BoxModel>,
    /// Margin values (outer spacing)
    pub margin: Option<BoxModel>,
}
impl ComponentBounds {
    /// Create a new ComponentBounds with just name and bounds
    pub fn new(name: impl Into<String>, bounds: Bounds<Pixels>) -> Self {
        Self {
            name: name.into(),
            bounds,
            component_type: ComponentType::Container,
            padding: None,
            margin: None,
        }
    }

    /// Set the component type
    pub fn with_type(mut self, component_type: ComponentType) -> Self {
        self.component_type = component_type;
        self
    }

    /// Set padding values
    pub fn with_padding(mut self, padding: BoxModel) -> Self {
        self.padding = Some(padding);
        self
    }

    /// Set margin values
    pub fn with_margin(mut self, margin: BoxModel) -> Self {
        self.margin = Some(margin);
        self
    }
}
/// Color scheme for the debug grid overlay
///
/// All colors are in 0xRRGGBBAA format (RGBA with alpha)
#[derive(Clone, Debug)]
pub struct GridColorScheme {
    /// Grid line color (semi-transparent gray)
    pub grid_lines: u32,
    /// Prompt bounding box color
    pub prompt_bounds: u32,
    /// Input bounding box color
    pub input_bounds: u32,
    /// Button bounding box color
    pub button_bounds: u32,
    /// List bounding box color
    pub list_bounds: u32,
    /// List item bounding box color
    pub list_item_bounds: u32,
    /// Header bounding box color
    pub header_bounds: u32,
    /// Container bounding box color
    pub container_bounds: u32,
    /// Other component bounding box color
    pub other_bounds: u32,
    /// Padding visualization color (green-ish, semi-transparent)
    pub padding_fill: u32,
    /// Margin visualization color (orange-ish, semi-transparent)
    pub margin_fill: u32,
    /// Alignment guide color (hot pink)
    pub alignment_guide: u32,
    /// Label background color
    pub label_background: u32,
    /// Label text color
    pub label_text: u32,
}
impl Default for GridColorScheme {
    fn default() -> Self {
        Self {
            grid_lines: 0x80808040,       // Gray at 25% opacity
            prompt_bounds: 0xFF6B6BFF,    // Red (full opacity for border)
            input_bounds: 0x4ECDC4FF,     // Teal
            button_bounds: 0xFFE66DFF,    // Yellow
            list_bounds: 0x95E1D3FF,      // Mint
            list_item_bounds: 0xA8E6CFFF, // Light mint
            header_bounds: 0xDDA0DDFF,    // Plum
            container_bounds: 0x87CEEBFF, // Sky blue
            other_bounds: 0xD3D3D3FF,     // Light gray
            padding_fill: 0x98D8AA40,     // Green at 25% opacity
            margin_fill: 0xF7DC6F40,      // Orange at 25% opacity
            alignment_guide: 0xFF69B4AA,  // Hot pink at 66% opacity
            label_background: 0x000000CC, // Black at 80% opacity
            label_text: 0xFFFFFFFF,       // White
        }
    }
}
impl GridColorScheme {
    /// Get the bounds color for a given component type
    pub fn color_for_type(&self, component_type: &ComponentType) -> u32 {
        match component_type {
            ComponentType::Prompt => self.prompt_bounds,
            ComponentType::Input => self.input_bounds,
            ComponentType::Button => self.button_bounds,
            ComponentType::List => self.list_bounds,
            ComponentType::ListItem => self.list_item_bounds,
            ComponentType::Header => self.header_bounds,
            ComponentType::Container => self.container_bounds,
            ComponentType::Other => self.other_bounds,
        }
    }
}
/// Helper to extract f32 from Pixels
fn pixels_to_f32(p: Pixels) -> f32 {
    let val: f64 = p.into();
    val as f32
}
/// Render the complete grid overlay
///
/// This is the main entry point for rendering the debug overlay.
/// It combines grid lines, component bounds, box model visualization,
/// and alignment guides based on the configuration.
pub fn render_grid_overlay(
    config: &GridConfig,
    bounds: Bounds<Pixels>,
    components: &[ComponentBounds],
) -> impl IntoElement {
    let colors = &config.color_scheme;

    div()
        .absolute()
        .top(px(0.))
        .left(px(0.))
        .w(bounds.size.width)
        .h(bounds.size.height)
        // Pointer-events: none equivalent - overlay should not capture events
        .occlude()
        .children(
            // Grid lines
            Some(render_grid_lines(
                bounds,
                config.grid_size,
                colors.grid_lines,
            )),
        )
        .children(
            // Component bounds
            if config.show_bounds {
                Some(render_all_component_bounds(
                    components,
                    colors,
                    config.show_dimensions,
                ))
            } else {
                None
            },
        )
        .children(
            // Box model visualization
            if config.show_box_model {
                Some(render_all_box_models(components, colors))
            } else {
                None
            },
        )
        .children(
            // Alignment guides
            if config.show_alignment_guides {
                Some(render_alignment_guides(components, colors.alignment_guide))
            } else {
                None
            },
        )
}
/// Render grid lines at the specified interval
pub fn render_grid_lines(bounds: Bounds<Pixels>, grid_size: u32, color: u32) -> impl IntoElement {
    let grid_size_f = grid_size as f32;
    let width = pixels_to_f32(bounds.size.width);
    let height = pixels_to_f32(bounds.size.height);

    // Calculate number of lines
    let v_lines = (width / grid_size_f).ceil() as usize;
    let h_lines = (height / grid_size_f).ceil() as usize;

    div()
        .absolute()
        .top(px(0.))
        .left(px(0.))
        .w(bounds.size.width)
        .h(bounds.size.height)
        // Vertical lines
        .children((0..=v_lines).map(move |i| {
            let x = i as f32 * grid_size_f;
            div()
                .absolute()
                .left(px(x))
                .top(px(0.))
                .w(px(1.))
                .h(bounds.size.height)
                .bg(rgba(color))
        }))
        // Horizontal lines
        .children((0..=h_lines).map(move |i| {
            let y = i as f32 * grid_size_f;
            div()
                .absolute()
                .left(px(0.))
                .top(px(y))
                .w(bounds.size.width)
                .h(px(1.))
                .bg(rgba(color))
        }))
}
/// Render a single component's bounding box with label
///
/// If `show_dimensions` is true, the label will include the component's
/// width and height in pixels, e.g., "Header (500x45)".
pub fn render_component_bounds(
    component: &ComponentBounds,
    colors: &GridColorScheme,
    show_dimensions: bool,
) -> impl IntoElement {
    let color = colors.color_for_type(&component.component_type);
    let bounds = component.bounds;

    // Format label with optional dimensions
    let label = if show_dimensions {
        format!(
            "{} ({}x{})",
            component.name,
            pixels_to_f32(bounds.size.width) as i32,
            pixels_to_f32(bounds.size.height) as i32
        )
    } else {
        component.name.clone()
    };

    div()
        .absolute()
        .left(bounds.origin.x)
        .top(bounds.origin.y)
        .w(bounds.size.width)
        .h(bounds.size.height)
        .border_1()
        .border_color(rgba(color))
        // Label in top-left corner
        .child(
            div()
                .absolute()
                .left(px(0.))
                .top(px(0.))
                .px(px(4.))
                .py(px(2.))
                .bg(rgba(colors.label_background))
                .text_color(rgba(colors.label_text))
                .text_xs()
                .child(label),
        )
}
/// Render all component bounds
fn render_all_component_bounds(
    components: &[ComponentBounds],
    colors: &GridColorScheme,
    show_dimensions: bool,
) -> impl IntoElement {
    div().absolute().children(
        components
            .iter()
            .map(move |c| render_component_bounds(c, colors, show_dimensions)),
    )
}
/// Render box model visualization (padding and margin)
///
/// CSS DevTools style:
/// - Padding: green inner region
/// - Margin: orange outer region
pub fn render_box_model(component: &ComponentBounds, colors: &GridColorScheme) -> impl IntoElement {
    let bounds = component.bounds;

    div()
        .absolute()
        .children(
            // Render margin (outer region)
            component.margin.as_ref().map(|margin| {
                // Top margin
                let top = div()
                    .absolute()
                    .left(bounds.origin.x - px(margin.left))
                    .top(bounds.origin.y - px(margin.top))
                    .w(bounds.size.width + px(margin.left + margin.right))
                    .h(px(margin.top))
                    .bg(rgba(colors.margin_fill));

                // Bottom margin
                let bottom = div()
                    .absolute()
                    .left(bounds.origin.x - px(margin.left))
                    .top(bounds.origin.y + bounds.size.height)
                    .w(bounds.size.width + px(margin.left + margin.right))
                    .h(px(margin.bottom))
                    .bg(rgba(colors.margin_fill));

                // Left margin
                let left = div()
                    .absolute()
                    .left(bounds.origin.x - px(margin.left))
                    .top(bounds.origin.y)
                    .w(px(margin.left))
                    .h(bounds.size.height)
                    .bg(rgba(colors.margin_fill));

                // Right margin
                let right = div()
                    .absolute()
                    .left(bounds.origin.x + bounds.size.width)
                    .top(bounds.origin.y)
                    .w(px(margin.right))
                    .h(bounds.size.height)
                    .bg(rgba(colors.margin_fill));

                div()
                    .absolute()
                    .child(top)
                    .child(bottom)
                    .child(left)
                    .child(right)
            }),
        )
        .children(
            // Render padding (inner region)
            component.padding.as_ref().map(|padding| {
                // Top padding
                let top = div()
                    .absolute()
                    .left(bounds.origin.x)
                    .top(bounds.origin.y)
                    .w(bounds.size.width)
                    .h(px(padding.top))
                    .bg(rgba(colors.padding_fill));

                // Bottom padding
                let bottom = div()
                    .absolute()
                    .left(bounds.origin.x)
                    .top(bounds.origin.y + bounds.size.height - px(padding.bottom))
                    .w(bounds.size.width)
                    .h(px(padding.bottom))
                    .bg(rgba(colors.padding_fill));

                // Left padding
                let left = div()
                    .absolute()
                    .left(bounds.origin.x)
                    .top(bounds.origin.y + px(padding.top))
                    .w(px(padding.left))
                    .h(bounds.size.height - px(padding.top + padding.bottom))
                    .bg(rgba(colors.padding_fill));

                // Right padding
                let right = div()
                    .absolute()
                    .left(bounds.origin.x + bounds.size.width - px(padding.right))
                    .top(bounds.origin.y + px(padding.top))
                    .w(px(padding.right))
                    .h(bounds.size.height - px(padding.top + padding.bottom))
                    .bg(rgba(colors.padding_fill));

                div()
                    .absolute()
                    .child(top)
                    .child(bottom)
                    .child(left)
                    .child(right)
            }),
        )
}
