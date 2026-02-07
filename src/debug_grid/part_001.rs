/// Render all box models
fn render_all_box_models(
    components: &[ComponentBounds],
    colors: &GridColorScheme,
) -> impl IntoElement {
    div()
        .absolute()
        .children(components.iter().map(|c| render_box_model(c, colors)))
}
/// Alignment edge for detecting aligned components
#[derive(Debug, Clone, Copy, PartialEq)]
enum AlignmentEdge {
    Left(f32),
    Right(f32),
    Top(f32),
    Bottom(f32),
}
/// Render alignment guides between aligned components
///
/// Detects components that share the same x or y coordinates
/// and draws dashed lines connecting their aligned edges.
pub fn render_alignment_guides(components: &[ComponentBounds], color: u32) -> impl IntoElement {
    // Threshold for considering edges "aligned" (within N pixels)
    const ALIGNMENT_THRESHOLD: f32 = 2.0;

    let mut vertical_guides: Vec<f32> = Vec::new();
    let mut horizontal_guides: Vec<f32> = Vec::new();

    // Collect all edges
    let mut edges: Vec<AlignmentEdge> = Vec::new();
    for component in components {
        let b = &component.bounds;
        let x: f32 = pixels_to_f32(b.origin.x);
        let y: f32 = pixels_to_f32(b.origin.y);
        let w: f32 = pixels_to_f32(b.size.width);
        let h: f32 = pixels_to_f32(b.size.height);
        edges.push(AlignmentEdge::Left(x));
        edges.push(AlignmentEdge::Right(x + w));
        edges.push(AlignmentEdge::Top(y));
        edges.push(AlignmentEdge::Bottom(y + h));
    }

    // Find aligned vertical edges (left/right)
    for i in 0..edges.len() {
        for j in (i + 1)..edges.len() {
            let (val_i, val_j, is_vertical) = match (&edges[i], &edges[j]) {
                (AlignmentEdge::Left(a), AlignmentEdge::Left(b))
                | (AlignmentEdge::Right(a), AlignmentEdge::Right(b))
                | (AlignmentEdge::Left(a), AlignmentEdge::Right(b))
                | (AlignmentEdge::Right(a), AlignmentEdge::Left(b)) => (*a, *b, true),
                (AlignmentEdge::Top(a), AlignmentEdge::Top(b))
                | (AlignmentEdge::Bottom(a), AlignmentEdge::Bottom(b))
                | (AlignmentEdge::Top(a), AlignmentEdge::Bottom(b))
                | (AlignmentEdge::Bottom(a), AlignmentEdge::Top(b)) => (*a, *b, false),
                _ => continue,
            };

            if (val_i - val_j).abs() < ALIGNMENT_THRESHOLD {
                let avg = (val_i + val_j) / 2.0;
                if is_vertical {
                    if !vertical_guides
                        .iter()
                        .any(|&v| (v - avg).abs() < ALIGNMENT_THRESHOLD)
                    {
                        vertical_guides.push(avg);
                    }
                } else if !horizontal_guides
                    .iter()
                    .any(|&v| (v - avg).abs() < ALIGNMENT_THRESHOLD)
                {
                    horizontal_guides.push(avg);
                }
            }
        }
    }

    // Calculate overall bounds for guide lines
    let (min_x, max_x, min_y, max_y) = components.iter().fold(
        (f32::MAX, f32::MIN, f32::MAX, f32::MIN),
        |(min_x, max_x, min_y, max_y), c| {
            let b = &c.bounds;
            let x: f32 = pixels_to_f32(b.origin.x);
            let y: f32 = pixels_to_f32(b.origin.y);
            let w: f32 = pixels_to_f32(b.size.width);
            let h: f32 = pixels_to_f32(b.size.height);
            (
                min_x.min(x),
                max_x.max(x + w),
                min_y.min(y),
                max_y.max(y + h),
            )
        },
    );

    div()
        .absolute()
        // Vertical alignment guides
        .children(
            vertical_guides
                .into_iter()
                .map(move |x| render_dashed_line_vertical(x, min_y, max_y, color)),
        )
        // Horizontal alignment guides
        .children(
            horizontal_guides
                .into_iter()
                .map(move |y| render_dashed_line_horizontal(y, min_x, max_x, color)),
        )
}
/// Render a vertical dashed line
fn render_dashed_line_vertical(x: f32, y_start: f32, y_end: f32, color: u32) -> impl IntoElement {
    const DASH_LENGTH: f32 = 4.0;
    const GAP_LENGTH: f32 = 4.0;
    const LINE_WIDTH: f32 = 1.0;

    let total_length = y_end - y_start;
    let dash_count = (total_length / (DASH_LENGTH + GAP_LENGTH)).ceil() as usize;

    div().absolute().children((0..dash_count).map(move |i| {
        let y = y_start + (i as f32 * (DASH_LENGTH + GAP_LENGTH));
        let remaining = y_end - y;
        let dash_height = DASH_LENGTH.min(remaining);

        div()
            .absolute()
            .left(px(x))
            .top(px(y))
            .w(px(LINE_WIDTH))
            .h(px(dash_height))
            .bg(rgba(color))
    }))
}
/// Render a horizontal dashed line
fn render_dashed_line_horizontal(y: f32, x_start: f32, x_end: f32, color: u32) -> impl IntoElement {
    const DASH_LENGTH: f32 = 4.0;
    const GAP_LENGTH: f32 = 4.0;
    const LINE_WIDTH: f32 = 1.0;

    let total_length = x_end - x_start;
    let dash_count = (total_length / (DASH_LENGTH + GAP_LENGTH)).ceil() as usize;

    div().absolute().children((0..dash_count).map(move |i| {
        let x = x_start + (i as f32 * (DASH_LENGTH + GAP_LENGTH));
        let remaining = x_end - x;
        let dash_width = DASH_LENGTH.min(remaining);

        div()
            .absolute()
            .left(px(x))
            .top(px(y))
            .w(px(dash_width))
            .h(px(LINE_WIDTH))
            .bg(rgba(color))
    }))
}
/// Helper function to create bounds from origin and size
pub fn bounds_from_origin_size(x: f32, y: f32, width: f32, height: f32) -> Bounds<Pixels> {
    Bounds {
        origin: Point { x: px(x), y: px(y) },
        size: Size {
            width: px(width),
            height: px(height),
        },
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_grid_config() {
        let config = GridConfig::default();
        assert_eq!(config.grid_size, 8);
        assert!(config.show_bounds);
        assert!(!config.show_box_model);
        assert!(config.show_alignment_guides);
        assert!(!config.show_dimensions); // Off by default
    }

    #[test]
    fn test_show_dimensions_config() {
        let config = GridConfig {
            show_dimensions: true,
            ..Default::default()
        };
        assert!(config.show_dimensions);
    }

    #[test]
    fn test_default_color_scheme() {
        let colors = GridColorScheme::default();
        assert_eq!(colors.grid_lines, 0x80808040);
        assert_eq!(colors.prompt_bounds, 0xFF6B6BFF);
    }

    #[test]
    fn test_color_for_type() {
        let colors = GridColorScheme::default();
        assert_eq!(
            colors.color_for_type(&ComponentType::Prompt),
            colors.prompt_bounds
        );
        assert_eq!(
            colors.color_for_type(&ComponentType::Input),
            colors.input_bounds
        );
        assert_eq!(
            colors.color_for_type(&ComponentType::Button),
            colors.button_bounds
        );
    }

    #[test]
    fn test_box_model_uniform() {
        let bm = BoxModel::uniform(8.0);
        assert_eq!(bm.top, 8.0);
        assert_eq!(bm.right, 8.0);
        assert_eq!(bm.bottom, 8.0);
        assert_eq!(bm.left, 8.0);
    }

    #[test]
    fn test_box_model_symmetric() {
        let bm = BoxModel::symmetric(10.0, 20.0);
        assert_eq!(bm.top, 10.0);
        assert_eq!(bm.bottom, 10.0);
        assert_eq!(bm.left, 20.0);
        assert_eq!(bm.right, 20.0);
    }

    #[test]
    fn test_component_bounds_builder() {
        let bounds = bounds_from_origin_size(0.0, 0.0, 100.0, 50.0);
        let component = ComponentBounds::new("Test", bounds)
            .with_type(ComponentType::Button)
            .with_padding(BoxModel::uniform(8.0))
            .with_margin(BoxModel::symmetric(4.0, 8.0));

        assert_eq!(component.name, "Test");
        assert_eq!(component.component_type, ComponentType::Button);
        assert!(component.padding.is_some());
        assert!(component.margin.is_some());
    }

    #[test]
    fn test_bounds_from_origin_size() {
        let bounds = bounds_from_origin_size(10.0, 20.0, 100.0, 50.0);
        assert_eq!(pixels_to_f32(bounds.origin.x), 10.0);
        assert_eq!(pixels_to_f32(bounds.origin.y), 20.0);
        assert_eq!(pixels_to_f32(bounds.size.width), 100.0);
        assert_eq!(pixels_to_f32(bounds.size.height), 50.0);
    }

    #[test]
    fn test_pixels_to_f32() {
        assert_eq!(pixels_to_f32(px(10.0)), 10.0);
        assert_eq!(pixels_to_f32(px(0.0)), 0.0);
        assert_eq!(pixels_to_f32(px(100.5)), 100.5);
    }
}
