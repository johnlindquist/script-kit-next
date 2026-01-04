🧩 Packing 8 file(s)...
📝 Files selected:
  • src/app_layout.rs
  • src/app_impl.rs
  • src/main.rs
  • src/app_render.rs
  • src/app_actions.rs
  • src/lib.rs
  • src/app_navigation.rs
  • src/app_execute.rs
This file is a merged representation of the filtered codebase, combined into a single document by packx.

<file_summary>
This section contains a summary of this file.

<purpose>
This file contains a packed representation of filtered repository contents.
It is designed to be easily consumable by AI systems for analysis, code review,
or other automated processes.
</purpose>

<usage_guidelines>
- Treat this file as a snapshot of the repository's state
- Be aware that this file may contain sensitive information
</usage_guidelines>

<notes>
- Files were filtered by packx based on content and extension matching
- Total files included: 8
</notes>
</file_summary>

<directory_structure>
src/app_layout.rs
src/app_impl.rs
src/main.rs
src/app_render.rs
src/app_actions.rs
src/lib.rs
src/app_navigation.rs
src/app_execute.rs
</directory_structure>

<files>
This section contains the contents of the repository's files.

<file path="src/app_layout.rs">
// Layout calculation methods for ScriptListApp
// This file is included via include!() macro in main.rs
// Contains: build_component_bounds, build_layout_info

impl ScriptListApp {
    /// Build component bounds for the debug grid overlay based on current view
    ///
    /// This creates approximate bounds for major UI components based on our
    /// known layout structure. Since GPUI doesn't expose layout bounds at runtime,
    /// we calculate them based on our layout constants.
    fn build_component_bounds(
        &self,
        window_size: gpui::Size<gpui::Pixels>,
    ) -> Vec<debug_grid::ComponentBounds> {
        use debug_grid::{BoxModel, ComponentBounds, ComponentType};

        let mut bounds = Vec::new();
        let width = window_size.width;
        let height = window_size.height;

        // Layout constants from panel.rs and list_item.rs
        // Header: py(HEADER_PADDING_Y=8) + max(input=22px, buttons=28px) + py(8) + divider(1px)
        // The buttons are 28px tall, input is 22px, so header content height is 28px
        // Total: 8 + 28 + 8 + 1 = 45px
        const HEADER_PADDING_Y: f32 = 8.0;
        const HEADER_PADDING_X: f32 = 16.0;
        const BUTTON_HEIGHT: f32 = 28.0;
        const DIVIDER_HEIGHT: f32 = 1.0;
        let header_height = px(HEADER_PADDING_Y * 2.0 + BUTTON_HEIGHT + DIVIDER_HEIGHT); // 45px

        // Content padding matches HEADER_PADDING_X
        let content_padding = HEADER_PADDING_X;

        // Main content area (below header)
        let content_top = header_height;
        let content_height = height - header_height;

        // Determine the current view type and build appropriate bounds
        let view_name = match &self.current_view {
            AppView::ScriptList => "ScriptList",
            AppView::ArgPrompt { .. } => "ArgPrompt",
            AppView::DivPrompt { .. } => "DivPrompt",
            AppView::EditorPrompt { .. } => "EditorPrompt",
            AppView::TermPrompt { .. } => "TermPrompt",
            AppView::FormPrompt { .. } => "FormPrompt",
            AppView::SelectPrompt { .. } => "SelectPrompt",
            AppView::PathPrompt { .. } => "PathPrompt",
            AppView::EnvPrompt { .. } => "EnvPrompt",
            AppView::DropPrompt { .. } => "DropPrompt",
            AppView::TemplatePrompt { .. } => "TemplatePrompt",
            AppView::ClipboardHistoryView { .. } => "ClipboardHistory",
            AppView::AppLauncherView { .. } => "AppLauncher",
            AppView::WindowSwitcherView { .. } => "WindowSwitcher",
            AppView::DesignGalleryView { .. } => "DesignGallery",
            AppView::ActionsDialog => "ActionsDialog",
        };

        // Header bounds (includes padding + input + divider) - common to all views
        bounds.push(
            ComponentBounds::new(
                "Header",
                gpui::Bounds {
                    origin: gpui::point(px(0.), px(0.)),
                    size: gpui::size(width, header_height),
                },
            )
            .with_type(ComponentType::Header)
            .with_padding(BoxModel::symmetric(HEADER_PADDING_Y, content_padding)),
        );

        // Build view-specific bounds
        match &self.current_view {
            AppView::ScriptList => {
                // ScriptList has left panel (50%) + right preview panel (50%)
                let list_width = width * 0.5;
                let item_height = px(48.0);

                bounds.push(
                    ComponentBounds::new(
                        "ScriptList",
                        gpui::Bounds {
                            origin: gpui::point(px(0.), content_top),
                            size: gpui::size(list_width, content_height),
                        },
                    )
                    .with_type(ComponentType::List)
                    .with_padding(BoxModel::uniform(0.0)),
                );

                // Add sample list items
                for i in 0..5 {
                    let item_top = content_top + px(i as f32 * 48.0);
                    if item_top + item_height > height {
                        break;
                    }
                    bounds.push(
                        ComponentBounds::new(
                            format!("ListItem[{}]", i),
                            gpui::Bounds {
                                origin: gpui::point(px(0.), item_top),
                                size: gpui::size(list_width, item_height),
                            },
                        )
                        .with_type(ComponentType::ListItem)
                        .with_padding(BoxModel::symmetric(12.0, content_padding))
                        .with_margin(BoxModel::uniform(0.0)),
                    );
                }

                // Preview panel (right side)
                bounds.push(
                    ComponentBounds::new(
                        "PreviewPanel",
                        gpui::Bounds {
                            origin: gpui::point(list_width, content_top),
                            size: gpui::size(width - list_width, content_height),
                        },
                    )
                    .with_type(ComponentType::Container)
                    .with_padding(BoxModel::uniform(content_padding)),
                );
            }

            AppView::DivPrompt { .. } => {
                // DivPrompt takes full width below header
                bounds.push(
                    ComponentBounds::new(
                        "DivContent",
                        gpui::Bounds {
                            origin: gpui::point(px(0.), content_top),
                            size: gpui::size(width, content_height),
                        },
                    )
                    .with_type(ComponentType::Prompt)
                    .with_padding(BoxModel::uniform(content_padding)),
                );
            }

            AppView::EditorPrompt { .. } => {
                // EditorPrompt takes full width below header
                bounds.push(
                    ComponentBounds::new(
                        "EditorContent",
                        gpui::Bounds {
                            origin: gpui::point(px(0.), content_top),
                            size: gpui::size(width, content_height),
                        },
                    )
                    .with_type(ComponentType::Prompt)
                    .with_padding(BoxModel::uniform(content_padding)),
                );
            }

            AppView::TermPrompt { .. } => {
                // TermPrompt takes full width below header
                bounds.push(
                    ComponentBounds::new(
                        "TerminalContent",
                        gpui::Bounds {
                            origin: gpui::point(px(0.), content_top),
                            size: gpui::size(width, content_height),
                        },
                    )
                    .with_type(ComponentType::Prompt)
                    .with_padding(BoxModel::uniform(content_padding)),
                );
            }

            AppView::ArgPrompt { choices, .. } => {
                // ArgPrompt may have choices list
                if choices.is_empty() {
                    // No choices - just input area
                    bounds.push(
                        ComponentBounds::new(
                            "ArgInput",
                            gpui::Bounds {
                                origin: gpui::point(px(0.), content_top),
                                size: gpui::size(width, content_height),
                            },
                        )
                        .with_type(ComponentType::Prompt)
                        .with_padding(BoxModel::uniform(content_padding)),
                    );
                } else {
                    // Has choices - show list
                    let item_height = px(48.0);
                    bounds.push(
                        ComponentBounds::new(
                            "ChoicesList",
                            gpui::Bounds {
                                origin: gpui::point(px(0.), content_top),
                                size: gpui::size(width, content_height),
                            },
                        )
                        .with_type(ComponentType::List)
                        .with_padding(BoxModel::uniform(0.0)),
                    );

                    // Add choice items
                    for i in 0..choices.len().min(5) {
                        let item_top = content_top + px(i as f32 * 48.0);
                        if item_top + item_height > height {
                            break;
                        }
                        bounds.push(
                            ComponentBounds::new(
                                format!("Choice[{}]", i),
                                gpui::Bounds {
                                    origin: gpui::point(px(0.), item_top),
                                    size: gpui::size(width, item_height),
                                },
                            )
                            .with_type(ComponentType::ListItem)
                            .with_padding(BoxModel::symmetric(12.0, content_padding)),
                        );
                    }
                }
            }

            AppView::FormPrompt { .. } => {
                bounds.push(
                    ComponentBounds::new(
                        "FormContent",
                        gpui::Bounds {
                            origin: gpui::point(px(0.), content_top),
                            size: gpui::size(width, content_height),
                        },
                    )
                    .with_type(ComponentType::Prompt)
                    .with_padding(BoxModel::uniform(content_padding)),
                );
            }

            AppView::SelectPrompt { .. } | AppView::PathPrompt { .. } => {
                // List-based prompts
                let item_height = px(48.0);
                bounds.push(
                    ComponentBounds::new(
                        view_name,
                        gpui::Bounds {
                            origin: gpui::point(px(0.), content_top),
                            size: gpui::size(width, content_height),
                        },
                    )
                    .with_type(ComponentType::List)
                    .with_padding(BoxModel::uniform(0.0)),
                );

                for i in 0..5 {
                    let item_top = content_top + px(i as f32 * 48.0);
                    if item_top + item_height > height {
                        break;
                    }
                    bounds.push(
                        ComponentBounds::new(
                            format!("Item[{}]", i),
                            gpui::Bounds {
                                origin: gpui::point(px(0.), item_top),
                                size: gpui::size(width, item_height),
                            },
                        )
                        .with_type(ComponentType::ListItem)
                        .with_padding(BoxModel::symmetric(12.0, content_padding)),
                    );
                }
            }

            // Other prompts - generic full-width content
            _ => {
                bounds.push(
                    ComponentBounds::new(
                        view_name,
                        gpui::Bounds {
                            origin: gpui::point(px(0.), content_top),
                            size: gpui::size(width, content_height),
                        },
                    )
                    .with_type(ComponentType::Prompt)
                    .with_padding(BoxModel::uniform(content_padding)),
                );
            }
        }

        // Only add header detail bounds for ScriptList view (the original behavior)
        if matches!(self.current_view, AppView::ScriptList) {
            let list_width = width * 0.5;

            // Input field in header
            // Positioned at: px(HEADER_PADDING_X) = 16, py(HEADER_PADDING_Y) = 8
            // The input is vertically centered in the header (which has 28px content height)
            // Input height is ~22px (CURSOR_HEIGHT_LG=18 + CURSOR_MARGIN_Y*2=4)
            const INPUT_HEIGHT: f32 = 22.0;
            let input_x = px(content_padding);
            let input_y = px(HEADER_PADDING_Y + (BUTTON_HEIGHT - INPUT_HEIGHT) / 2.0); // Vertically centered
                                                                                       // Input takes flex-1, estimate it takes most of the header width before buttons
                                                                                       // Buttons area is roughly: Run(50) + divider(20) + Actions(70) + divider(20) + Logo(16) + padding(16) = ~192px
            let buttons_area_width = px(200.);
            let input_width = width - px(content_padding) - buttons_area_width;

            bounds.push(
                ComponentBounds::new(
                    "SearchInput",
                    gpui::Bounds {
                        origin: gpui::point(input_x, input_y),
                        size: gpui::size(input_width, px(INPUT_HEIGHT)),
                    },
                )
                .with_type(ComponentType::Input)
                .with_padding(BoxModel::symmetric(0.0, 0.0)),
            );

            // Header buttons (right side)
            // Buttons are h(28px) positioned at top of content area (after top padding)
            let button_height = px(BUTTON_HEIGHT);
            let button_y = px(HEADER_PADDING_Y); // Buttons at top of content area

            // Buttons layout from right to left:
            // [SearchInput flex-1] [Run ~45px] [|] [Actions ~70px] [|] [Logo 16px] [padding 16px]
            // Spacing: gap=12, divider ~8px each side = ~20px between groups
            let logo_size = px(16.);
            let right_padding = px(content_padding);

            // Logo (Script Kit icon) - rightmost, 16x16 vertically centered in button area
            let logo_x = width - right_padding - logo_size;
            let logo_y = px(HEADER_PADDING_Y + (BUTTON_HEIGHT - 16.0) / 2.0); // Vertically centered
            bounds.push(
                ComponentBounds::new(
                    "Lg", // Short name for Logo to fit in small space
                    gpui::Bounds {
                        origin: gpui::point(logo_x, logo_y),
                        size: gpui::size(logo_size, logo_size),
                    },
                )
                .with_type(ComponentType::Other)
                .with_padding(BoxModel::uniform(0.0)),
            );

            // Actions button - left of divider, left of logo
            // Actual button text "Actions ⌘K" is roughly 80-90px wide
            let actions_width = px(85.);
            let actions_x = logo_x - px(24.) - actions_width; // ~24px for divider + spacing

            bounds.push(
                ComponentBounds::new(
                    "Actions", // Shortened from ActionsButton
                    gpui::Bounds {
                        origin: gpui::point(actions_x, button_y),
                        size: gpui::size(actions_width, button_height),
                    },
                )
                .with_type(ComponentType::Button)
                .with_padding(BoxModel::symmetric(4.0, 8.0)),
            );

            // Run button - left of divider, left of Actions
            // Actual button text "Run ↵" is roughly 50-60px wide
            let run_width = px(55.);
            let run_x = actions_x - px(24.) - run_width; // ~24px for divider + spacing

            bounds.push(
                ComponentBounds::new(
                    "Run", // Shortened from RunButton
                    gpui::Bounds {
                        origin: gpui::point(run_x, button_y),
                        size: gpui::size(run_width, button_height),
                    },
                )
                .with_type(ComponentType::Button)
                .with_padding(BoxModel::symmetric(4.0, 8.0)),
            );

            // Preview panel contents (right 50% of window)
            // Preview has its own padding, content starts at list_width + padding
            let preview_padding = 16.0_f32;
            let preview_left = list_width + px(preview_padding);
            let preview_width = width * 0.5 - px(preview_padding * 2.0);

            // Script path label (small text at top of preview)
            bounds.push(
                ComponentBounds::new(
                    "ScriptPath",
                    gpui::Bounds {
                        origin: gpui::point(preview_left, content_top + px(8.)),
                        size: gpui::size(preview_width, px(16.)),
                    },
                )
                .with_type(ComponentType::Other)
                .with_padding(BoxModel::symmetric(2.0, 0.0)),
            );

            // Script title (large heading)
            bounds.push(
                ComponentBounds::new(
                    "ScriptTitle",
                    gpui::Bounds {
                        origin: gpui::point(preview_left, content_top + px(32.)),
                        size: gpui::size(preview_width, px(32.)),
                    },
                )
                .with_type(ComponentType::Header)
                .with_padding(BoxModel::symmetric(4.0, 0.0)),
            );

            // Description label
            bounds.push(
                ComponentBounds::new(
                    "DescLabel", // Shortened
                    gpui::Bounds {
                        origin: gpui::point(preview_left, content_top + px(72.)),
                        size: gpui::size(px(80.), px(16.)),
                    },
                )
                .with_type(ComponentType::Other)
                .with_padding(BoxModel::uniform(2.0)),
            );

            // Description value
            bounds.push(
                ComponentBounds::new(
                    "DescValue", // Shortened
                    gpui::Bounds {
                        origin: gpui::point(preview_left, content_top + px(92.)),
                        size: gpui::size(preview_width, px(20.)),
                    },
                )
                .with_type(ComponentType::Other)
                .with_padding(BoxModel::symmetric(2.0, 0.0)),
            );

            // Code Preview label
            bounds.push(
                ComponentBounds::new(
                    "CodeLabel", // Shortened from CodePreviewLabel
                    gpui::Bounds {
                        origin: gpui::point(preview_left, content_top + px(130.)),
                        size: gpui::size(px(100.), px(16.)),
                    },
                )
                .with_type(ComponentType::Other)
                .with_padding(BoxModel::uniform(2.0)),
            );

            // Code preview area
            bounds.push(
                ComponentBounds::new(
                    "CodePreview",
                    gpui::Bounds {
                        origin: gpui::point(preview_left, content_top + px(150.)),
                        size: gpui::size(preview_width, height - content_top - px(170.)),
                    },
                )
                .with_type(ComponentType::Container)
                .with_padding(BoxModel::uniform(12.0)),
            );

            // List item icons (left side of each list item)
            // Icons are typically 24x24, positioned with some padding from left edge
            // Item height is 48px, icon vertically centered: (48 - 24) / 2 = 12px from top
            let item_height = px(48.0);
            for i in 0..5 {
                let item_top = content_top + px(i as f32 * 48.0);
                if item_top + item_height > height {
                    break;
                }
                bounds.push(
                    ComponentBounds::new(
                        format!("Icon[{}]", i),
                        gpui::Bounds {
                            origin: gpui::point(px(content_padding), item_top + px(12.)),
                            size: gpui::size(px(24.), px(24.)),
                        },
                    )
                    .with_type(ComponentType::Other)
                    .with_padding(BoxModel::uniform(0.0)),
                );
            }
        } // End of ScriptList-specific bounds

        bounds
    }

    /// Build complete layout info for the getLayoutInfo() SDK function.
    ///
    /// This provides AI agents with detailed component information including:
    /// - Bounds (position and size)
    /// - Box model (padding, margin, gap)
    /// - Flex properties (direction, grow, align)
    /// - Human-readable explanations of why components are sized as they are
    ///
    /// This is the "why" function - it explains the layout, not just shows it.
    pub fn build_layout_info(&self, _cx: &mut gpui::Context<Self>) -> protocol::LayoutInfo {
        use protocol::{LayoutComponentInfo, LayoutComponentType, LayoutInfo};

        // TODO: Get actual window size once we have access to window in this context
        // For now, use default values
        let window_width = 750.0_f32;
        let window_height = 500.0_f32;

        // Determine current prompt type
        let prompt_type = match &self.current_view {
            AppView::ScriptList => "mainMenu",
            AppView::ArgPrompt { .. } => "arg",
            AppView::DivPrompt { .. } => "div",
            AppView::FormPrompt { .. } => "form",
            AppView::TermPrompt { .. } => "term",
            AppView::EditorPrompt { .. } => "editor",
            AppView::SelectPrompt { .. } => "select",
            AppView::PathPrompt { .. } => "path",
            AppView::EnvPrompt { .. } => "env",
            AppView::DropPrompt { .. } => "drop",
            AppView::TemplatePrompt { .. } => "template",
            AppView::ClipboardHistoryView { .. } => "clipboardHistory",
            AppView::AppLauncherView { .. } => "appLauncher",
            AppView::WindowSwitcherView { .. } => "windowSwitcher",
            AppView::DesignGalleryView { .. } => "designGallery",
            AppView::ActionsDialog => "actionsDialog",
        };

        let mut components = Vec::new();

        // Layout constants (same as build_component_bounds)
        const HEADER_PADDING_Y: f32 = 8.0;
        const HEADER_PADDING_X: f32 = 16.0;
        const BUTTON_HEIGHT: f32 = 28.0;
        const DIVIDER_HEIGHT: f32 = 1.0;
        let header_height = HEADER_PADDING_Y * 2.0 + BUTTON_HEIGHT + DIVIDER_HEIGHT; // 45px
        let list_width = window_width * 0.5;
        let content_top = header_height;
        let content_height = window_height - header_height;

        // Root container
        components.push(
            LayoutComponentInfo::new("Window", LayoutComponentType::Container)
                .with_bounds(0.0, 0.0, window_width, window_height)
                .with_flex_column()
                .with_depth(0)
                .with_explanation("Root window container. Uses flex-column layout."),
        );

        // Header
        components.push(
            LayoutComponentInfo::new("Header", LayoutComponentType::Header)
                .with_bounds(0.0, 0.0, window_width, header_height)
                .with_padding(HEADER_PADDING_Y, HEADER_PADDING_X, HEADER_PADDING_Y, HEADER_PADDING_X)
                .with_flex_row()
                .with_depth(1)
                .with_parent("Window")
                .with_explanation(format!(
                    "Height = padding({}) + content({}) + padding({}) + divider({}) = {}px. Uses flex-row with items-center.",
                    HEADER_PADDING_Y, BUTTON_HEIGHT, HEADER_PADDING_Y, DIVIDER_HEIGHT, header_height
                )),
        );

        // Search input in header
        const INPUT_HEIGHT: f32 = 22.0;
        let input_y = HEADER_PADDING_Y + (BUTTON_HEIGHT - INPUT_HEIGHT) / 2.0;
        let buttons_area_width = 200.0;
        let input_width = window_width - HEADER_PADDING_X - buttons_area_width;

        components.push(
            LayoutComponentInfo::new("SearchInput", LayoutComponentType::Input)
                .with_bounds(HEADER_PADDING_X, input_y, input_width, INPUT_HEIGHT)
                .with_flex_grow(1.0)
                .with_depth(2)
                .with_parent("Header")
                .with_explanation(format!(
                    "flex-grow:1 fills remaining space. Width = window({}) - padding({}) - buttons_area({}) = {}px. Vertically centered in header.",
                    window_width, HEADER_PADDING_X, buttons_area_width, input_width
                )),
        );

        // Content area
        components.push(
            LayoutComponentInfo::new("ContentArea", LayoutComponentType::Container)
                .with_bounds(0.0, content_top, window_width, content_height)
                .with_flex_row()
                .with_flex_grow(1.0)
                .with_depth(1)
                .with_parent("Window")
                .with_explanation(
                    "flex-grow:1 fills remaining height after header. Uses flex-row to create side-by-side panels.".to_string()
                ),
        );

        // Script list (left panel) - 50% width
        components.push(
            LayoutComponentInfo::new("ScriptList", LayoutComponentType::List)
                .with_bounds(0.0, content_top, list_width, content_height)
                .with_flex_column()
                .with_depth(2)
                .with_parent("ContentArea")
                .with_explanation(format!(
                    "Width = 50% of window = {}px. Uses uniform_list for virtualized scrolling with 48px item height.",
                    list_width
                )),
        );

        // Preview panel (right panel) - remaining 50%
        let preview_width = window_width - list_width;
        components.push(
            LayoutComponentInfo::new("PreviewPanel", LayoutComponentType::Panel)
                .with_bounds(list_width, content_top, preview_width, content_height)
                .with_padding(16.0, 16.0, 16.0, 16.0)
                .with_flex_column()
                .with_depth(2)
                .with_parent("ContentArea")
                .with_explanation(format!(
                    "Width = remaining 50% = {}px. Has 16px padding on all sides.",
                    preview_width
                )),
        );

        // List items (sample of first few visible)
        const LIST_ITEM_HEIGHT: f32 = 48.0;
        let visible_items = ((content_height / LIST_ITEM_HEIGHT) as usize).min(5);
        for i in 0..visible_items {
            let item_top = content_top + (i as f32 * LIST_ITEM_HEIGHT);
            components.push(
                LayoutComponentInfo::new(format!("ListItem[{}]", i), LayoutComponentType::ListItem)
                    .with_bounds(0.0, item_top, list_width, LIST_ITEM_HEIGHT)
                    .with_padding(12.0, 16.0, 12.0, 16.0)
                    .with_gap(8.0)
                    .with_flex_row()
                    .with_depth(3)
                    .with_parent("ScriptList")
                    .with_explanation(format!(
                        "Fixed height = {}px. Uses flex-row with gap:8px for icon + text layout. Padding: 12px vertical, 16px horizontal.",
                        LIST_ITEM_HEIGHT
                    )),
            );
        }

        // Button group in header
        let button_y = HEADER_PADDING_Y;
        let button_height = BUTTON_HEIGHT;

        // Logo button (rightmost)
        let logo_x = window_width - HEADER_PADDING_X - 20.0;
        components.push(
            LayoutComponentInfo::new("LogoButton", LayoutComponentType::Button)
                .with_bounds(logo_x, button_y, 20.0, button_height)
                .with_padding(4.0, 4.0, 4.0, 4.0)
                .with_depth(2)
                .with_parent("Header")
                .with_explanation("Fixed 20px width. Positioned at right edge with 16px margin."),
        );

        // Actions button
        let actions_width = 85.0;
        let actions_x = logo_x - 24.0 - actions_width;
        components.push(
            LayoutComponentInfo::new("ActionsButton", LayoutComponentType::Button)
                .with_bounds(actions_x, button_y, actions_width, button_height)
                .with_padding(4.0, 8.0, 4.0, 8.0)
                .with_depth(2)
                .with_parent("Header")
                .with_explanation(format!(
                    "Width = {}px. Positioned left of logo with 24px spacing (includes divider).",
                    actions_width
                )),
        );

        // Run button
        let run_width = 55.0;
        let run_x = actions_x - 24.0 - run_width;
        components.push(
            LayoutComponentInfo::new("RunButton", LayoutComponentType::Button)
                .with_bounds(run_x, button_y, run_width, button_height)
                .with_padding(4.0, 8.0, 4.0, 8.0)
                .with_depth(2)
                .with_parent("Header")
                .with_explanation(format!(
                    "Width = {}px. Positioned left of Actions with 24px spacing.",
                    run_width
                )),
        );

        LayoutInfo {
            window_width,
            window_height,
            prompt_type: prompt_type.to_string(),
            components,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
}

</file>

<file path="src/app_impl.rs">
impl ScriptListApp {
    fn new(config: config::Config, bun_available: bool, window: &mut Window, cx: &mut Context<Self>) -> Self {
        // PERF: Measure script loading time
        let load_start = std::time::Instant::now();
        let scripts = scripts::read_scripts();
        let scripts_elapsed = load_start.elapsed();

        let scriptlets_start = std::time::Instant::now();
        let scriptlets = scripts::read_scriptlets();
        let scriptlets_elapsed = scriptlets_start.elapsed();

        let theme = theme::load_theme();
        // Config is now passed in from main() to avoid duplicate load (~100-300ms savings)

        // Load frecency data for suggested section tracking
        let suggested_config = config.get_suggested();
        let mut frecency_store = FrecencyStore::with_config(&suggested_config);
        frecency_store.load().ok(); // Ignore errors - starts fresh if file doesn't exist

        // Load built-in entries based on config
        let builtin_entries = builtins::get_builtin_entries(&config.get_builtins());

        // Apps are loaded in the background to avoid blocking startup
        // Start with empty list, will be populated asynchronously
        let apps = Vec::new();

        let total_elapsed = load_start.elapsed();
        logging::log("PERF", &format!(
            "Startup loading: {:.2}ms total ({} scripts in {:.2}ms, {} scriptlets in {:.2}ms, apps loading in background)",
            total_elapsed.as_secs_f64() * 1000.0,
            scripts.len(),
            scripts_elapsed.as_secs_f64() * 1000.0,
            scriptlets.len(),
            scriptlets_elapsed.as_secs_f64() * 1000.0
        ));
        logging::log(
            "APP",
            &format!("Loaded {} scripts from ~/.sk/kit/scripts", scripts.len()),
        );
        logging::log(
            "APP",
            &format!(
                "Loaded {} scriptlets from ~/.sk/kit/scriptlets/scriptlets.md",
                scriptlets.len()
            ),
        );
        logging::log(
            "APP",
            &format!("Loaded {} built-in features", builtin_entries.len()),
        );
        logging::log("APP", "Applications loading in background...");
        logging::log("APP", "Loaded theme with system appearance detection");
        logging::log(
            "APP",
            &format!(
                "Loaded config: hotkey={:?}+{}, bun_path={:?}",
                config.hotkey.modifiers, config.hotkey.key, config.bun_path
            ),
        );

        // Load apps in background thread to avoid blocking startup
        let app_launcher_enabled = config.get_builtins().app_launcher;
        if app_launcher_enabled {
            // Use a channel to send loaded apps back to main thread
            let (tx, rx) =
                std::sync::mpsc::channel::<(Vec<app_launcher::AppInfo>, std::time::Duration)>();

            // Spawn background thread for app scanning
            std::thread::spawn(move || {
                let start = std::time::Instant::now();
                let apps = app_launcher::scan_applications().clone();
                let elapsed = start.elapsed();
                let _ = tx.send((apps, elapsed));
            });

            // Poll for results using a spawned task
            cx.spawn(async move |this, cx| {
                // Poll the channel periodically
                loop {
                    Timer::after(std::time::Duration::from_millis(50)).await;
                    match rx.try_recv() {
                        Ok((apps, elapsed)) => {
                            let app_count = apps.len();
                            let _ = cx.update(|cx| {
                                this.update(cx, |app, cx| {
                                    app.apps = apps;
                                    // Invalidate caches since apps changed
                                    app.filter_cache_key = String::from("\0_APPS_LOADED_\0");
                                    app.grouped_cache_key = String::from("\0_APPS_LOADED_\0");
                                    logging::log(
                                        "APP",
                                        &format!(
                                            "Background app loading complete: {} apps in {:.2}ms",
                                            app_count,
                                            elapsed.as_secs_f64() * 1000.0
                                        ),
                                    );
                                    cx.notify();
                                })
                            });
                            break;
                        }
                        Err(std::sync::mpsc::TryRecvError::Empty) => continue,
                        Err(std::sync::mpsc::TryRecvError::Disconnected) => break,
                    }
                }
            })
            .detach();
        }
        logging::log("UI", "Script Kit logo SVG loaded for header rendering");

        // Start cursor blink timer - updates all inputs that track cursor visibility
        cx.spawn(async move |this, cx| {
            loop {
                Timer::after(std::time::Duration::from_millis(530)).await;
                let _ = cx.update(|cx| {
                    this.update(cx, |app, cx| {
                        // Skip cursor blink when window is hidden or no input is focused
                        if !script_kit_gpui::is_main_window_visible()
                            || app.focused_input == FocusedInput::None
                        {
                            return;
                        }

                        app.cursor_visible = !app.cursor_visible;
                        // Also update ActionsDialog cursor if it exists
                        if let Some(ref dialog) = app.actions_dialog {
                            dialog.update(cx, |d, _cx| {
                                d.set_cursor_visible(app.cursor_visible);
                            });
                        }
                        cx.notify();
                    })
                });
            }
        })
        .detach();

        let gpui_input_state =
            cx.new(|cx| InputState::new(window, cx).placeholder(DEFAULT_PLACEHOLDER));
        let gpui_input_subscription = cx.subscribe_in(&gpui_input_state, window, {
            move |this, _, event: &InputEvent, window, cx| match event {
                InputEvent::Focus => {
                    this.gpui_input_focused = true;
                    this.focused_input = FocusedInput::MainFilter;
                    cx.notify();
                }
                InputEvent::Blur => {
                    this.gpui_input_focused = false;
                    if this.focused_input == FocusedInput::MainFilter {
                        this.focused_input = FocusedInput::None;
                    }
                    cx.notify();
                }
                InputEvent::Change => {
                    this.handle_filter_input_change(window, cx);
                }
                InputEvent::PressEnter { .. } => {
                    if matches!(this.current_view, AppView::ScriptList) && !this.show_actions_popup
                    {
                        this.execute_selected(cx);
                    }
                }
            }
        });

        let mut app = ScriptListApp {
            scripts,
            scriptlets,
            builtin_entries,
            apps,
            selected_index: 0,
            filter_text: String::new(),
            gpui_input_state,
            gpui_input_focused: false,
            gpui_input_subscriptions: vec![gpui_input_subscription],
            suppress_filter_events: false,
            pending_filter_sync: false,
            last_output: None,
            focus_handle: cx.focus_handle(),
            show_logs: false,
            theme,
            config,
            // Scroll activity tracking: start with scrollbar hidden
            is_scrolling: false,
            last_scroll_time: None,
            current_view: AppView::ScriptList,
            script_session: Arc::new(ParkingMutex::new(None)),
            arg_input: TextInputState::new(),
            arg_selected_index: 0,
            prompt_receiver: None,
            response_sender: None,
            // Variable-height list state for main menu (section headers at 24px, items at 48px)
            // Start with 0 items, will be reset when grouped_items changes
            // .measure_all() ensures all items are measured upfront for correct scroll height
            main_list_state: ListState::new(0, ListAlignment::Top, px(100.)).measure_all(),
            list_scroll_handle: UniformListScrollHandle::new(),
            arg_list_scroll_handle: UniformListScrollHandle::new(),
            clipboard_list_scroll_handle: UniformListScrollHandle::new(),
            window_list_scroll_handle: UniformListScrollHandle::new(),
            design_gallery_scroll_handle: UniformListScrollHandle::new(),
            show_actions_popup: false,
            actions_dialog: None,
            cursor_visible: true,
            focused_input: FocusedInput::MainFilter,
            current_script_pid: None,
            // P1: Initialize filter cache
            cached_filtered_results: Vec::new(),
            filter_cache_key: String::from("\0_UNINITIALIZED_\0"), // Sentinel value to force initial compute
            // P1: Initialize grouped results cache (Arc for cheap clone)
            cached_grouped_items: Arc::from([]),
            cached_grouped_flat_results: Arc::from([]),
            grouped_cache_key: String::from("\0_UNINITIALIZED_\0"), // Sentinel value to force initial compute
            // P3: Two-stage filter coalescing
            computed_filter_text: String::new(),
            filter_coalescer: FilterCoalescer::new(),
            // Scroll stabilization: start with no last scrolled index
            last_scrolled_index: None,
            // Preview cache: start empty, will populate on first render
            preview_cache_path: None,
            preview_cache_lines: Vec::new(),
            // Design system: start with default design
            current_design: DesignVariant::default(),
            // Toast manager: initialize for error notifications
            toast_manager: ToastManager::new(),
            // Clipboard image cache: decoded RenderImages for thumbnails/preview
            clipboard_image_cache: std::collections::HashMap::new(),
            // Frecency store for tracking script usage
            frecency_store,
            // Mouse hover tracking - starts as None (no item hovered)
            hovered_index: None,
            // P0-2: Initialize hover debounce timer
            last_hover_notify: std::time::Instant::now(),
            // Pending path action - starts as None (Arc<Mutex<>> for callback access)
            pending_path_action: Arc::new(Mutex::new(None)),
            // Signal to close path actions dialog
            close_path_actions: Arc::new(Mutex::new(false)),
            // Shared state: path actions dialog visibility (for toggle behavior)
            path_actions_showing: Arc::new(Mutex::new(false)),
            // Shared state: path actions search text (for header display)
            path_actions_search_text: Arc::new(Mutex::new(String::new())),
            // Pending path action result - action_id + path_info to execute
            pending_path_action_result: Arc::new(Mutex::new(None)),
            // Alias/shortcut registries - populated below
            alias_registry: std::collections::HashMap::new(),
            shortcut_registry: std::collections::HashMap::new(),
            // SDK actions - starts empty, populated by setActions() from scripts
            sdk_actions: None,
            action_shortcuts: std::collections::HashMap::new(),
            // Debug grid overlay - check env var at startup
            grid_config: if std::env::var("SCRIPT_KIT_DEBUG_GRID").is_ok() {
                logging::log("DEBUG_GRID", "SCRIPT_KIT_DEBUG_GRID env var set - enabling grid overlay");
                Some(debug_grid::GridConfig::default())
            } else {
                None
            },
            // Navigation coalescing for rapid arrow key events
            nav_coalescer: NavCoalescer::new(),
            // Window focus tracking - for detecting focus lost and auto-dismissing prompts
            was_window_focused: false,
            // Scroll stabilization: track last scrolled index for each handle
            last_scrolled_main: None,
            last_scrolled_arg: None,
            last_scrolled_clipboard: None,
            last_scrolled_window: None,
            last_scrolled_design_gallery: None,
            // Show warning banner when bun is not available
            show_bun_warning: !bun_available,
            // Pending confirmation for dangerous actions
            pending_confirmation: None,
            // Menu bar integration: Now handled by frontmost_app_tracker module
            // which pre-fetches menu items in background when apps activate
        };

        // Build initial alias/shortcut registries (conflicts logged, not shown via HUD on startup)
        let conflicts = app.rebuild_registries();
        if !conflicts.is_empty() {
            logging::log(
                "STARTUP",
                &format!(
                    "Found {} alias/shortcut conflicts on startup",
                    conflicts.len()
                ),
            );
        }

        app
    }

    /// Switch to a different design variant
    ///
    /// Cycle to the next design variant.
    /// Use Cmd+1 to cycle through all designs.
    fn cycle_design(&mut self, cx: &mut Context<Self>) {
        let old_design = self.current_design;
        let new_design = old_design.next();
        let all_designs = DesignVariant::all();
        let old_idx = all_designs
            .iter()
            .position(|&v| v == old_design)
            .unwrap_or(0);
        let new_idx = all_designs
            .iter()
            .position(|&v| v == new_design)
            .unwrap_or(0);

        logging::log(
            "DESIGN",
            &format!(
                "Cycling design: {} ({}) -> {} ({}) [total: {}]",
                old_design.name(),
                old_idx,
                new_design.name(),
                new_idx,
                all_designs.len()
            ),
        );
        logging::log(
            "DESIGN",
            &format!(
                "Design '{}': {}",
                new_design.name(),
                new_design.description()
            ),
        );

        self.current_design = new_design;
        logging::log(
            "DESIGN",
            &format!("self.current_design is now: {:?}", self.current_design),
        );
        cx.notify();
    }

    fn update_theme(&mut self, cx: &mut Context<Self>) {
        self.theme = theme::load_theme();
        logging::log("APP", "Theme reloaded based on system appearance");
        cx.notify();
    }

    fn update_config(&mut self, cx: &mut Context<Self>) {
        self.config = config::load_config();
        clipboard_history::set_max_text_content_len(
            self.config.get_clipboard_history_max_text_length(),
        );
        logging::log(
            "APP",
            &format!("Config reloaded: padding={:?}", self.config.get_padding()),
        );
        cx.notify();
    }

    fn refresh_scripts(&mut self, cx: &mut Context<Self>) {
        self.scripts = scripts::read_scripts();
        self.scriptlets = scripts::read_scriptlets();
        self.selected_index = 0;
        self.last_scrolled_index = None;
        // Use main_list_state for variable-height list (not the legacy list_scroll_handle)
        self.main_list_state.scroll_to_reveal_item(0);
        self.last_scrolled_index = Some(0);
        self.invalidate_filter_cache();
        self.invalidate_grouped_cache();

        // Rebuild alias/shortcut registries and show HUD for any conflicts
        let conflicts = self.rebuild_registries();
        for conflict in conflicts {
            self.show_hud(conflict, Some(4000), cx); // 4s for conflict messages
        }

        logging::log(
            "APP",
            &format!(
                "Scripts refreshed: {} scripts, {} scriptlets loaded",
                self.scripts.len(),
                self.scriptlets.len()
            ),
        );
        cx.notify();
    }

    /// Dismiss the bun warning banner
    fn dismiss_bun_warning(&mut self, cx: &mut Context<Self>) {
        logging::log("APP", "Bun warning banner dismissed by user");
        self.show_bun_warning = false;
        cx.notify();
    }

    /// Open bun.sh in the default browser
    fn open_bun_website(&self) {
        logging::log("APP", "Opening https://bun.sh in default browser");
        if let Err(e) = std::process::Command::new("open")
            .arg("https://bun.sh")
            .spawn()
        {
            logging::log("APP", &format!("Failed to open bun.sh: {}", e));
        }
    }

    /// Handle incremental scriptlet file change
    ///
    /// Instead of reloading all scriptlets, this method:
    /// 1. Parses only the changed file
    /// 2. Diffs against cached state to find what changed
    /// 3. Updates hotkeys/expand triggers incrementally
    /// 4. Updates the scriptlets list
    ///
    /// # Arguments
    /// * `path` - Path to the changed/deleted scriptlet file
    /// * `is_deleted` - Whether the file was deleted (vs created/modified)
    /// * `cx` - The context for UI updates
    fn handle_scriptlet_file_change(
        &mut self,
        path: &std::path::Path,
        is_deleted: bool,
        cx: &mut Context<Self>,
    ) {
        use script_kit_gpui::scriptlet_cache::{diff_scriptlets, CachedScriptlet};

        logging::log(
            "APP",
            &format!(
                "Incremental scriptlet change: {} (deleted={})",
                path.display(),
                is_deleted
            ),
        );

        // Get old cached scriptlets for this file (if any)
        // Note: We're using a simple approach here - comparing name+shortcut+expand+alias
        let old_scriptlets: Vec<CachedScriptlet> = self
            .scriptlets
            .iter()
            .filter(|s| {
                s.file_path
                    .as_ref()
                    .map(|fp| fp.starts_with(&path.to_string_lossy().to_string()))
                    .unwrap_or(false)
            })
            .map(|s| {
                CachedScriptlet::new(
                    s.name.clone(),
                    s.shortcut.clone(),
                    s.expand.clone(),
                    s.alias.clone(),
                    s.file_path.clone().unwrap_or_default(),
                )
            })
            .collect();

        // Parse new scriptlets from file (empty if deleted)
        let new_scripts_scriptlets = if is_deleted {
            vec![]
        } else {
            scripts::read_scriptlets_from_file(path)
        };

        let new_scriptlets: Vec<CachedScriptlet> = new_scripts_scriptlets
            .iter()
            .map(|s| {
                CachedScriptlet::new(
                    s.name.clone(),
                    s.shortcut.clone(),
                    s.expand.clone(),
                    s.alias.clone(),
                    s.file_path.clone().unwrap_or_default(),
                )
            })
            .collect();

        // Compute diff
        let diff = diff_scriptlets(&old_scriptlets, &new_scriptlets);

        if diff.is_empty() {
            logging::log(
                "APP",
                &format!("No changes detected in {}", path.display()),
            );
            return;
        }

        logging::log(
            "APP",
            &format!(
                "Scriptlet diff: {} added, {} removed, {} shortcut changes, {} expand changes, {} alias changes",
                diff.added.len(),
                diff.removed.len(),
                diff.shortcut_changes.len(),
                diff.expand_changes.len(),
                diff.alias_changes.len()
            ),
        );

        // Apply hotkey changes
        for removed in &diff.removed {
            if removed.shortcut.is_some() {
                if let Err(e) = hotkeys::unregister_script_hotkey(&removed.file_path) {
                    logging::log(
                        "HOTKEY",
                        &format!("Failed to unregister hotkey for {}: {}", removed.name, e),
                    );
                }
            }
        }

        for added in &diff.added {
            if let Some(ref shortcut) = added.shortcut {
                if let Err(e) = hotkeys::register_script_hotkey(&added.file_path, shortcut) {
                    logging::log(
                        "HOTKEY",
                        &format!("Failed to register hotkey for {}: {}", added.name, e),
                    );
                }
            }
        }

        for change in &diff.shortcut_changes {
            if let Err(e) = hotkeys::update_script_hotkey(
                &change.file_path,
                change.old.as_deref(),
                change.new.as_deref(),
            ) {
                logging::log(
                    "HOTKEY",
                    &format!("Failed to update hotkey for {}: {}", change.name, e),
                );
            }
        }

        // Apply expand manager changes (macOS only)
        #[cfg(target_os = "macos")]
        {
            // For removed scriptlets, clear their triggers
            for removed in &diff.removed {
                if removed.expand.is_some() {
                    // We'd need access to the expand manager here
                    // For now, log that we would clear triggers
                    logging::log(
                        "EXPAND",
                        &format!("Would clear expand trigger for removed: {}", removed.name),
                    );
                }
            }

            // For added scriptlets with expand, register them
            for added in &diff.added {
                if added.expand.is_some() {
                    logging::log(
                        "EXPAND",
                        &format!("Would register expand trigger for added: {}", added.name),
                    );
                }
            }

            // For changed expand triggers, update them
            for change in &diff.expand_changes {
                logging::log(
                    "EXPAND",
                    &format!(
                        "Would update expand trigger for {}: {:?} -> {:?}",
                        change.name, change.old, change.new
                    ),
                );
            }
        }

        // Update the scriptlets list
        // Remove old scriptlets from this file
        let path_str = path.to_string_lossy().to_string();
        self.scriptlets
            .retain(|s| !s.file_path.as_ref().map(|fp| fp.starts_with(&path_str)).unwrap_or(false));

        // Add new scriptlets from this file
        self.scriptlets.extend(new_scripts_scriptlets);

        // Sort by name to maintain consistent ordering
        self.scriptlets.sort_by(|a, b| a.name.cmp(&b.name));

        // Invalidate caches
        self.invalidate_filter_cache();
        self.invalidate_grouped_cache();

        // Rebuild alias/shortcut registries for this file's scriptlets
        let conflicts = self.rebuild_registries();
        for conflict in conflicts {
            self.show_hud(conflict, Some(4000), cx);
        }

        logging::log(
            "APP",
            &format!(
                "Scriptlet file updated incrementally: {} now has {} total scriptlets",
                path.display(),
                self.scriptlets.len()
            ),
        );

        cx.notify();
    }

    /// Get unified filtered results combining scripts and scriptlets
    /// Helper to get filter text as string (for compatibility with existing code)
    fn filter_text(&self) -> &str {
        self.filter_text.as_str()
    }

    /// P1: Now uses caching - invalidates only when filter_text changes
    fn filtered_results(&self) -> Vec<scripts::SearchResult> {
        let filter_text = self.filter_text();
        // P1: Return cached results if filter hasn't changed
        if filter_text == self.filter_cache_key {
            logging::log_debug(
                "CACHE",
                &format!("Filter cache HIT for '{}'", filter_text),
            );
            return self.cached_filtered_results.clone();
        }

        // P1: Cache miss - need to recompute (will be done by get_filtered_results_mut)
        logging::log_debug(
            "CACHE",
            &format!(
                "Filter cache MISS - need recompute for '{}' (cached key: '{}')",
                filter_text, self.filter_cache_key
            ),
        );

        // PERF: Measure search time (only log when actually filtering)
        let search_start = std::time::Instant::now();
        let results =
            scripts::fuzzy_search_unified(&self.scripts, &self.scriptlets, filter_text);
        let search_elapsed = search_start.elapsed();

        // Only log search performance when there's an active filter
        if !filter_text.is_empty() {
            logging::log(
                "PERF",
                &format!(
                    "Search '{}' took {:.2}ms ({} results from {} total)",
                    filter_text,
                    search_elapsed.as_secs_f64() * 1000.0,
                    results.len(),
                    self.scripts.len() + self.scriptlets.len()
                ),
            );
        }
        results
    }

    /// P1: Get filtered results with cache update (mutable version)
    /// Call this when you need to ensure cache is updated
    fn get_filtered_results_cached(&mut self) -> &Vec<scripts::SearchResult> {
        if self.filter_text != self.filter_cache_key {
            logging::log_debug(
                "CACHE",
                &format!("Filter cache MISS - recomputing for '{}'", self.filter_text),
            );
            let search_start = std::time::Instant::now();
            self.cached_filtered_results = scripts::fuzzy_search_unified_all(
                &self.scripts,
                &self.scriptlets,
                &self.builtin_entries,
                &self.apps,
                &self.filter_text,
            );
            self.filter_cache_key = self.filter_text.clone();
            let search_elapsed = search_start.elapsed();

            if !self.filter_text.is_empty() {
                logging::log(
                    "PERF",
                    &format!(
                        "Search '{}' took {:.2}ms ({} results from {} total)",
                        self.filter_text,
                        search_elapsed.as_secs_f64() * 1000.0,
                        self.cached_filtered_results.len(),
                        self.scripts.len()
                            + self.scriptlets.len()
                            + self.builtin_entries.len()
                            + self.apps.len()
                    ),
                );
            }
        } else {
            logging::log_debug(
                "CACHE",
                &format!("Filter cache HIT for '{}'", self.filter_text),
            );
        }
        &self.cached_filtered_results
    }

    /// P1: Invalidate filter cache (call when scripts/scriptlets change)
    #[allow(dead_code)]
    fn invalidate_filter_cache(&mut self) {
        logging::log_debug("CACHE", "Filter cache INVALIDATED");
        self.filter_cache_key = String::from("\0_INVALIDATED_\0");
    }

    /// P1: Get grouped results with caching - avoids recomputing 9+ times per keystroke
    ///
    /// This is the ONLY place that should call scripts::get_grouped_results().
    /// P3: Cache is keyed off computed_filter_text (not filter_text) for two-stage filtering.
    ///
    /// P1-Arc: Returns Arc clones for cheap sharing with render closures.
    fn get_grouped_results_cached(
        &mut self,
    ) -> (Arc<[GroupedListItem]>, Arc<[scripts::SearchResult]>) {
        // P3: Key off computed_filter_text for two-stage filtering
        if self.computed_filter_text == self.grouped_cache_key {
            logging::log_debug(
                "CACHE",
                &format!("Grouped cache HIT for '{}'", self.computed_filter_text),
            );
            return (
                self.cached_grouped_items.clone(),
                self.cached_grouped_flat_results.clone(),
            );
        }

        // Cache miss - need to recompute
        logging::log_debug(
            "CACHE",
            &format!(
                "Grouped cache MISS - recomputing for '{}'",
                self.computed_filter_text
            ),
        );

        let start = std::time::Instant::now();
        let suggested_config = self.config.get_suggested();
        
        // Get menu bar items from the background tracker (pre-fetched when apps activate)
        #[cfg(target_os = "macos")]
        let (menu_bar_items, menu_bar_bundle_id): (Vec<menu_bar::MenuBarItem>, Option<String>) = {
            let cached = frontmost_app_tracker::get_cached_menu_items();
            let bundle_id = frontmost_app_tracker::get_last_real_app().map(|a| a.bundle_id);
            // No conversion needed - tracker is compiled as part of binary crate
            // so it already returns binary crate types
            (cached, bundle_id)
        };
        #[cfg(not(target_os = "macos"))]
        let (menu_bar_items, menu_bar_bundle_id): (Vec<menu_bar::MenuBarItem>, Option<String>) = (Vec::new(), None);
        
        logging::log("APP", &format!(
            "get_grouped_results: filter='{}', menu_bar_items={}, bundle_id={:?}",
            self.computed_filter_text,
            menu_bar_items.len(),
            menu_bar_bundle_id
        ));
        let (grouped_items, flat_results) = get_grouped_results(
            &self.scripts,
            &self.scriptlets,
            &self.builtin_entries,
            &self.apps,
            &self.frecency_store,
            &self.computed_filter_text,
            &suggested_config,
            &menu_bar_items,
            menu_bar_bundle_id.as_deref(),
        );
        let elapsed = start.elapsed();

        // P1-Arc: Convert to Arc<[T]> for cheap clone
        self.cached_grouped_items = grouped_items.into();
        self.cached_grouped_flat_results = flat_results.into();
        self.grouped_cache_key = self.computed_filter_text.clone();

        if !self.computed_filter_text.is_empty() {
            logging::log_debug(
                "CACHE",
                &format!(
                    "Grouped results computed in {:.2}ms for '{}' ({} items)",
                    elapsed.as_secs_f64() * 1000.0,
                    self.computed_filter_text,
                    self.cached_grouped_items.len()
                ),
            );
        }

        (
            self.cached_grouped_items.clone(),
            self.cached_grouped_flat_results.clone(),
        )
    }

    /// P1: Invalidate grouped results cache (call when scripts/scriptlets/apps change)
    fn invalidate_grouped_cache(&mut self) {
        logging::log_debug("CACHE", "Grouped cache INVALIDATED");
        self.grouped_cache_key = String::from("\0_INVALIDATED_\0");
        // Also reset computed_filter_text to force recompute
        self.computed_filter_text = String::from("\0_INVALIDATED_\0");
    }

    /// Get the currently selected search result, correctly mapping from grouped index.
    ///
    /// This function handles the mapping from `selected_index` (which is the visual
    /// position in the grouped list including section headers) to the actual
    /// `SearchResult` in the flat results array.
    ///
    /// Returns `None` if:
    /// - The selected index points to a section header (headers aren't selectable)
    /// - The selected index is out of bounds
    /// - No results exist
    fn get_selected_result(&mut self) -> Option<scripts::SearchResult> {
        let selected_index = self.selected_index;
        let (grouped_items, flat_results) = self.get_grouped_results_cached();

        match grouped_items.get(selected_index) {
            Some(GroupedListItem::Item(idx)) => flat_results.get(*idx).cloned(),
            _ => None,
        }
    }

    /// Get or update the preview cache for syntax-highlighted code lines.
    /// Only re-reads and re-highlights when the script path actually changes.
    /// Returns cached lines if path matches, otherwise updates cache and returns new lines.
    fn get_or_update_preview_cache(
        &mut self,
        script_path: &str,
        lang: &str,
    ) -> &[syntax::HighlightedLine] {
        // Check if cache is valid for this path
        if self.preview_cache_path.as_deref() == Some(script_path)
            && !self.preview_cache_lines.is_empty()
        {
            logging::log_debug("CACHE", &format!("Preview cache HIT for '{}'", script_path));
            return &self.preview_cache_lines;
        }

        // Cache miss - need to re-read and re-highlight
        logging::log_debug(
            "CACHE",
            &format!("Preview cache MISS - loading '{}'", script_path),
        );

        self.preview_cache_path = Some(script_path.to_string());
        self.preview_cache_lines = match std::fs::read_to_string(script_path) {
            Ok(content) => {
                // Only take first 15 lines for preview
                let preview: String = content.lines().take(15).collect::<Vec<_>>().join("\n");
                syntax::highlight_code_lines(&preview, lang)
            }
            Err(e) => {
                logging::log("ERROR", &format!("Failed to read preview: {}", e));
                Vec::new()
            }
        };

        &self.preview_cache_lines
    }

    /// Invalidate the preview cache (call when selection might change to different script)
    #[allow(dead_code)]
    fn invalidate_preview_cache(&mut self) {
        self.preview_cache_path = None;
        self.preview_cache_lines.clear();
    }

    #[allow(dead_code)]
    fn filtered_scripts(&self) -> Vec<Arc<scripts::Script>> {
        let filter_text = self.filter_text();
        if filter_text.is_empty() {
            self.scripts.clone()
        } else {
            let filter_lower = filter_text.to_lowercase();
            self.scripts
                .iter()
                .filter(|s| s.name.to_lowercase().contains(&filter_lower))
                .cloned()
                .collect()
        }
    }

    /// Find a script or scriptlet by alias (case-insensitive exact match)
    /// Uses O(1) registry lookup instead of O(n) iteration
    fn find_alias_match(&self, alias: &str) -> Option<AliasMatch> {
        let alias_lower = alias.to_lowercase();

        // O(1) lookup in registry
        if let Some(path) = self.alias_registry.get(&alias_lower) {
            // Find the script/scriptlet by path
            for script in &self.scripts {
                if script.path.to_string_lossy() == *path {
                    logging::log(
                        "ALIAS",
                        &format!("Found script match: '{}' -> '{}'", alias, script.name),
                    );
                    return Some(AliasMatch::Script(script.clone()));
                }
            }

            // Check scriptlets by file_path or name
            for scriptlet in &self.scriptlets {
                let scriptlet_path = scriptlet.file_path.as_ref().unwrap_or(&scriptlet.name);
                if scriptlet_path == path {
                    logging::log(
                        "ALIAS",
                        &format!("Found scriptlet match: '{}' -> '{}'", alias, scriptlet.name),
                    );
                    return Some(AliasMatch::Scriptlet(scriptlet.clone()));
                }
            }

            // Path in registry but not found in current scripts (stale entry)
            logging::log(
                "ALIAS",
                &format!(
                    "Stale registry entry: '{}' -> '{}' (not found)",
                    alias, path
                ),
            );
        }

        None
    }


    fn execute_selected(&mut self, cx: &mut Context<Self>) {
        // Get grouped results to map from selected_index to actual result (cached)
        let (grouped_items, flat_results) = self.get_grouped_results_cached();
        // Clone to avoid borrow issues with self mutation below
        let grouped_items = grouped_items.clone();
        let flat_results = flat_results.clone();

        // Get the grouped item at selected_index and extract the result index
        let result_idx = match grouped_items.get(self.selected_index) {
            Some(GroupedListItem::Item(idx)) => Some(*idx),
            Some(GroupedListItem::SectionHeader(_)) => None, // Section headers are not selectable
            None => None,
        };

        if let Some(idx) = result_idx {
            if let Some(result) = flat_results.get(idx).cloned() {
                // Record frecency usage before executing
                let frecency_path = match &result {
                    scripts::SearchResult::Script(sm) => {
                        sm.script.path.to_string_lossy().to_string()
                    }
                    scripts::SearchResult::App(am) => am.app.path.to_string_lossy().to_string(),
                    scripts::SearchResult::BuiltIn(bm) => format!("builtin:{}", bm.entry.name),
                    scripts::SearchResult::Scriptlet(sm) => {
                        format!("scriptlet:{}", sm.scriptlet.name)
                    }
                    scripts::SearchResult::Window(wm) => {
                        format!("window:{}:{}", wm.window.app, wm.window.title)
                    }
                    scripts::SearchResult::Agent(am) => {
                        format!("agent:{}", am.agent.path.to_string_lossy())
                    }
                };
                self.frecency_store.record_use(&frecency_path);
                self.frecency_store.save().ok(); // Best-effort save
                self.invalidate_grouped_cache(); // Invalidate cache so next show reflects frecency

                match result {
                    scripts::SearchResult::Script(script_match) => {
                        logging::log(
                            "EXEC",
                            &format!("Executing script: {}", script_match.script.name),
                        );
                        self.execute_interactive(&script_match.script, cx);
                    }
                    scripts::SearchResult::Scriptlet(scriptlet_match) => {
                        logging::log(
                            "EXEC",
                            &format!("Executing scriptlet: {}", scriptlet_match.scriptlet.name),
                        );
                        self.execute_scriptlet(&scriptlet_match.scriptlet, cx);
                    }
                    scripts::SearchResult::BuiltIn(builtin_match) => {
                        logging::log(
                            "EXEC",
                            &format!("Executing built-in: {}", builtin_match.entry.name),
                        );
                        self.execute_builtin(&builtin_match.entry, cx);
                    }
                    scripts::SearchResult::App(app_match) => {
                        logging::log("EXEC", &format!("Launching app: {}", app_match.app.name));
                        self.execute_app(&app_match.app, cx);
                    }
                    scripts::SearchResult::Window(window_match) => {
                        logging::log(
                            "EXEC",
                            &format!("Focusing window: {}", window_match.window.title),
                        );
                        self.execute_window_focus(&window_match.window, cx);
                    }
                    scripts::SearchResult::Agent(agent_match) => {
                        logging::log(
                            "EXEC",
                            &format!("Agent selected: {}", agent_match.agent.name),
                        );
                        // TODO: Implement agent execution via mdflow
                        self.last_output = Some(SharedString::from(format!(
                            "Agent execution not yet implemented: {}",
                            agent_match.agent.name
                        )));
                    }
                }
            }
        }
    }

    fn handle_filter_input_change(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.suppress_filter_events {
            return;
        }

        let new_text = self.gpui_input_state.read(cx).value().to_string();
        if new_text == self.filter_text {
            return;
        }

        // Clear pending confirmation when typing (user is changing context)
        if self.pending_confirmation.is_some() {
            self.pending_confirmation = None;
        }

        let previous_text = std::mem::replace(&mut self.filter_text, new_text.clone());
        self.selected_index = 0;
        self.last_scrolled_index = None;
        // Use main_list_state for variable-height list (not the legacy list_scroll_handle)
        self.main_list_state.scroll_to_reveal_item(0);
        self.last_scrolled_index = Some(0);

        if new_text.ends_with(' ') {
            let trimmed = new_text.trim_end_matches(' ');
            if !trimmed.is_empty() && trimmed == previous_text {
                if let Some(alias_match) = self.find_alias_match(trimmed) {
                    logging::log(
                        "ALIAS",
                        &format!("Alias '{}' triggered execution", trimmed),
                    );
                    match alias_match {
                        AliasMatch::Script(script) => {
                            self.execute_interactive(&script, cx);
                        }
                        AliasMatch::Scriptlet(scriptlet) => {
                            self.execute_scriptlet(&scriptlet, cx);
                        }
                    }
                    self.clear_filter(window, cx);
                    return;
                }
            }
        }

        // P3: Notify immediately so UI updates (responsive typing)
        cx.notify();

        // Menu bar items are now pre-fetched by frontmost_app_tracker
        // No lazy loading needed - items are already in cache when we open

        self.queue_filter_compute(new_text, cx);
    }

    fn queue_filter_compute(&mut self, value: String, cx: &mut Context<Self>) {
        // P3: Debounce expensive search/window resize work.
        // Use 8ms debounce (half a frame) to batch rapid keystrokes.
        if self.filter_coalescer.queue(value) {
            cx.spawn(async move |this, cx| {
                // Wait 8ms for coalescing window (half frame at 60fps)
                Timer::after(std::time::Duration::from_millis(8)).await;

                let _ = cx.update(|cx| {
                    this.update(cx, |app, cx| {
                        if let Some(latest) = app.filter_coalescer.take_latest() {
                            if app.computed_filter_text != latest {
                                app.computed_filter_text = latest;
                                // This will trigger cache recompute on next get_grouped_results_cached()
                                app.update_window_size();
                                cx.notify();
                            }
                        }
                    })
                });
            })
            .detach();
        }
    }

    fn set_filter_text_immediate(
        &mut self,
        text: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.suppress_filter_events = true;
        self.filter_text = text.clone();
        self.gpui_input_state.update(cx, |state, cx| {
            state.set_value(text.clone(), window, cx);
        });
        self.suppress_filter_events = false;
        self.pending_filter_sync = false;

        self.selected_index = 0;
        self.last_scrolled_index = None;
        self.main_list_state.scroll_to_reveal_item(0);
        self.last_scrolled_index = Some(0);

        // Menu bar items are now pre-fetched by frontmost_app_tracker
        // No lazy loading needed - items are already in cache when we open

        self.computed_filter_text = text;
        self.filter_coalescer.reset();
        self.update_window_size();
        cx.notify();
    }

    fn clear_filter(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.set_filter_text_immediate(String::new(), window, cx);
    }

    fn sync_filter_input_if_needed(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.pending_filter_sync {
            return;
        }

        let desired = self.filter_text.clone();
        let current = self.gpui_input_state.read(cx).value().to_string();
        if current == desired {
            self.pending_filter_sync = false;
            return;
        }

        self.suppress_filter_events = true;
        self.gpui_input_state.update(cx, |state, cx| {
            state.set_value(desired.clone(), window, cx);
        });
        self.suppress_filter_events = false;
        self.pending_filter_sync = false;
    }

    fn toggle_logs(&mut self, cx: &mut Context<Self>) {
        self.show_logs = !self.show_logs;
        cx.notify();
    }

    /// Update window size based on current view and item count.
    /// This implements dynamic window resizing:
    /// - Script list: resize based on filtered results (including section headers)
    /// - Arg prompt: resize based on filtered choices
    /// - Div/Editor/Term: use full height
    fn update_window_size(&mut self) {
        let (view_type, item_count) = match &self.current_view {
            AppView::ScriptList => {
                // Get grouped results which includes section headers (cached)
                let (grouped_items, _) = self.get_grouped_results_cached();
                let count = grouped_items.len();
                (ViewType::ScriptList, count)
            }
            AppView::ArgPrompt { choices, .. } => {
                let filtered = self.get_filtered_arg_choices(choices);
                if filtered.is_empty() && choices.is_empty() {
                    (ViewType::ArgPromptNoChoices, 0)
                } else {
                    (ViewType::ArgPromptWithChoices, filtered.len())
                }
            }
            AppView::DivPrompt { .. } => (ViewType::DivPrompt, 0),
            AppView::FormPrompt { .. } => (ViewType::DivPrompt, 0), // Use DivPrompt size for forms
            AppView::EditorPrompt { .. } => {
                (ViewType::EditorPrompt, 0)
            }
            AppView::SelectPrompt { .. } => (ViewType::ArgPromptWithChoices, 0),
            AppView::PathPrompt { .. } => (ViewType::DivPrompt, 0),
            AppView::EnvPrompt { .. } => (ViewType::ArgPromptNoChoices, 0), // Env prompt is a simple input
            AppView::DropPrompt { .. } => (ViewType::DivPrompt, 0), // Drop prompt uses div size for drop zone
            AppView::TemplatePrompt { .. } => (ViewType::DivPrompt, 0), // Template prompt uses div size
            AppView::TermPrompt { .. } => (ViewType::TermPrompt, 0),
            AppView::ActionsDialog => {
                // Actions dialog is an overlay, don't resize
                return;
            }
            // Clipboard history and app launcher use standard height (same as script list)
            AppView::ClipboardHistoryView {
                entries, filter, ..
            } => {
                let filtered_count = if filter.is_empty() {
                    entries.len()
                } else {
                    let filter_lower = filter.to_lowercase();
                    entries
                        .iter()
                        .filter(|e| e.content.to_lowercase().contains(&filter_lower))
                        .count()
                };
                (ViewType::ScriptList, filtered_count)
            }
            AppView::AppLauncherView { apps, filter, .. } => {
                let filtered_count = if filter.is_empty() {
                    apps.len()
                } else {
                    let filter_lower = filter.to_lowercase();
                    apps.iter()
                        .filter(|a| a.name.to_lowercase().contains(&filter_lower))
                        .count()
                };
                (ViewType::ScriptList, filtered_count)
            }
            AppView::WindowSwitcherView {
                windows, filter, ..
            } => {
                let filtered_count = if filter.is_empty() {
                    windows.len()
                } else {
                    let filter_lower = filter.to_lowercase();
                    windows
                        .iter()
                        .filter(|w| {
                            w.title.to_lowercase().contains(&filter_lower)
                                || w.app.to_lowercase().contains(&filter_lower)
                        })
                        .count()
                };
                (ViewType::ScriptList, filtered_count)
            }
            AppView::DesignGalleryView { filter, .. } => {
                // Calculate total gallery items (separators + icons)
                let total_items = designs::separator_variations::SeparatorStyle::count()
                    + designs::icon_variations::total_icon_count();
                let filtered_count = if filter.is_empty() {
                    total_items
                } else {
                    // For now, return total - filtering can be added later
                    total_items
                };
                (ViewType::ScriptList, filtered_count)
            }
        };

        let target_height = height_for_view(view_type, item_count);
        resize_first_window_to_height(target_height);
    }

    fn set_prompt_input(&mut self, text: String, cx: &mut Context<Self>) {
        match &mut self.current_view {
            AppView::ArgPrompt { .. } => {
                self.arg_input.set_text(text);
                self.arg_selected_index = 0;
                self.arg_list_scroll_handle
                    .scroll_to_item(0, ScrollStrategy::Top);
                self.update_window_size();
                cx.notify();
            }
            AppView::PathPrompt { entity, .. } => {
                entity.update(cx, |prompt, cx| prompt.set_input(text, cx));
            }
            AppView::SelectPrompt { entity, .. } => {
                entity.update(cx, |prompt, cx| prompt.set_input(text, cx));
            }
            AppView::EnvPrompt { entity, .. } => {
                entity.update(cx, |prompt, cx| prompt.set_input(text, cx));
            }
            AppView::TemplatePrompt { entity, .. } => {
                entity.update(cx, |prompt, cx| prompt.set_input(text, cx));
            }
            AppView::FormPrompt { entity, .. } => {
                entity.update(cx, |prompt, cx| prompt.set_input(text, cx));
            }
            _ => {}
        }
    }

    /// Helper to get filtered arg choices without cloning
    fn get_filtered_arg_choices<'a>(&self, choices: &'a [Choice]) -> Vec<&'a Choice> {
        if self.arg_input.is_empty() {
            choices.iter().collect()
        } else {
            let filter = self.arg_input.text().to_lowercase();
            choices
                .iter()
                .filter(|c| c.name.to_lowercase().contains(&filter))
                .collect()
        }
    }

    fn focus_main_filter(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.focused_input = FocusedInput::MainFilter;
        let input_state = self.gpui_input_state.clone();
        input_state.update(cx, |state, cx| {
            state.focus(window, cx);
        });
    }

    fn toggle_actions(&mut self, cx: &mut Context<Self>, window: &mut Window) {
        logging::log("KEY", "Toggling actions popup");
        if self.show_actions_popup {
            // Close - return focus to main filter
            self.show_actions_popup = false;
            self.actions_dialog = None;
            self.focus_main_filter(window, cx);
            logging::log("FOCUS", "Actions closed, focus returned to MainFilter");
        } else {
            // Open - create dialog entity
            self.show_actions_popup = true;
            self.focused_input = FocusedInput::ActionsSearch;
            let script_info = self.get_focused_script_info();

            let theme_arc = std::sync::Arc::new(self.theme.clone());
            let dialog = cx.new(|cx| {
                let focus_handle = cx.focus_handle();
                ActionsDialog::with_script(
                    focus_handle,
                    std::sync::Arc::new(|_action_id| {}), // Callback handled separately
                    script_info,
                    theme_arc,
                )
            });

            // Hide the dialog's built-in search input since header already has search
            dialog.update(cx, |d, _| d.set_hide_search(true));

            // Focus the dialog's internal focus handle
            let dialog_focus_handle = dialog.read(cx).focus_handle.clone();
            self.actions_dialog = Some(dialog.clone());
            window.focus(&dialog_focus_handle, cx);
            logging::log("FOCUS", "Actions opened, focus moved to ActionsSearch");
        }
        cx.notify();
    }

    /// Toggle actions dialog for arg prompts with SDK-defined actions
    fn toggle_arg_actions(&mut self, cx: &mut Context<Self>, window: &mut Window) {
        logging::log(
            "KEY",
            &format!(
                "toggle_arg_actions called: show_actions_popup={}, actions_dialog.is_some={}, sdk_actions.is_some={}",
                self.show_actions_popup,
                self.actions_dialog.is_some(),
                self.sdk_actions.is_some()
            ),
        );
        if self.show_actions_popup {
            // Close - return focus to arg prompt
            self.show_actions_popup = false;
            self.actions_dialog = None;
            self.focused_input = FocusedInput::ArgPrompt;
            window.focus(&self.focus_handle, cx);
            logging::log("FOCUS", "Arg actions closed, focus returned to ArgPrompt");
        } else {
            // Check if we have SDK actions
            if let Some(ref sdk_actions) = self.sdk_actions {
                logging::log("KEY", &format!("SDK actions count: {}", sdk_actions.len()));
                if !sdk_actions.is_empty() {
                    // Open - create dialog entity with SDK actions
                    self.show_actions_popup = true;
                    self.focused_input = FocusedInput::ActionsSearch;

                    let theme_arc = std::sync::Arc::new(self.theme.clone());
                    let sdk_actions_clone = sdk_actions.clone();
                    let dialog = cx.new(|cx| {
                        let focus_handle = cx.focus_handle();
                        let mut dialog = ActionsDialog::with_script(
                            focus_handle,
                            std::sync::Arc::new(|_action_id| {}), // Callback handled separately
                            None,                                 // No script info for arg prompts
                            theme_arc,
                        );
                        // Set SDK actions to replace built-in actions
                        dialog.set_sdk_actions(sdk_actions_clone);
                        dialog
                    });

                    // Hide the dialog's built-in search input since header already has search
                    dialog.update(cx, |d, _| d.set_hide_search(true));

                    // Focus the dialog's internal focus handle
                    let dialog_focus_handle = dialog.read(cx).focus_handle.clone();
                    self.actions_dialog = Some(dialog.clone());
                    window.focus(&dialog_focus_handle, cx);
                    logging::log(
                        "FOCUS",
                        &format!(
                            "Arg actions OPENED: show_actions_popup={}, actions_dialog.is_some={}",
                            self.show_actions_popup,
                            self.actions_dialog.is_some()
                        ),
                    );
                } else {
                    logging::log("KEY", "No SDK actions available to show (empty list)");
                }
            } else {
                logging::log("KEY", "No SDK actions defined for this arg prompt (None)");
            }
        }
        cx.notify();
    }


    /// Edit a script in configured editor (config.editor > $EDITOR > "code")
    #[allow(dead_code)]
    fn edit_script(&mut self, path: &std::path::Path) {
        let editor = self.config.get_editor();
        logging::log(
            "UI",
            &format!("Opening script in editor '{}': {}", editor, path.display()),
        );
        let path_str = path.to_string_lossy().to_string();

        std::thread::spawn(move || {
            use std::process::Command;
            match Command::new(&editor).arg(&path_str).spawn() {
                Ok(_) => logging::log("UI", &format!("Successfully spawned editor: {}", editor)),
                Err(e) => logging::log(
                    "ERROR",
                    &format!("Failed to spawn editor '{}': {}", editor, e),
                ),
            }
        });
    }

    /// Execute a path action from the actions dialog
    /// Handles actions like copy_path, open_in_finder, open_in_editor, etc.
    fn execute_path_action(
        &mut self,
        action_id: &str,
        path_info: &PathInfo,
        path_prompt_entity: &Entity<PathPrompt>,
        cx: &mut Context<Self>,
    ) {
        logging::log(
            "UI",
            &format!(
                "Executing path action '{}' for: {} (is_dir={})",
                action_id, path_info.path, path_info.is_dir
            ),
        );

        match action_id {
            "select_file" | "open_directory" => {
                // For select/open, trigger submission through the path prompt
                // We need to trigger the submit callback with this path
                path_prompt_entity.update(cx, |prompt, cx| {
                    // Find the index of this path in filtered_entries and submit it
                    if let Some(idx) = prompt
                        .filtered_entries
                        .iter()
                        .position(|e| e.path == path_info.path)
                    {
                        prompt.selected_index = idx;
                    }
                    // For directories, navigate into them; for files, submit
                    if path_info.is_dir && action_id == "open_directory" {
                        prompt.navigate_to(&path_info.path, cx);
                    } else {
                        // Submit the selected path
                        let id = prompt.id.clone();
                        let path = path_info.path.clone();
                        (prompt.on_submit)(id, Some(path));
                    }
                });
            }
            "copy_path" => {
                // Copy full path to clipboard
                #[cfg(target_os = "macos")]
                {
                    use std::io::Write;
                    use std::process::{Command, Stdio};

                    match Command::new("pbcopy").stdin(Stdio::piped()).spawn() {
                        Ok(mut child) => {
                            if let Some(ref mut stdin) = child.stdin {
                                if stdin.write_all(path_info.path.as_bytes()).is_ok() {
                                    let _ = child.wait();
                                    logging::log(
                                        "UI",
                                        &format!("Copied path to clipboard: {}", path_info.path),
                                    );
                                    self.last_output = Some(SharedString::from(format!(
                                        "Copied: {}",
                                        path_info.path
                                    )));
                                } else {
                                    logging::log("ERROR", "Failed to write to pbcopy stdin");
                                    self.last_output =
                                        Some(SharedString::from("Failed to copy path"));
                                }
                            }
                        }
                        Err(e) => {
                            logging::log("ERROR", &format!("Failed to spawn pbcopy: {}", e));
                            self.last_output = Some(SharedString::from("Failed to copy path"));
                        }
                    }
                }

                #[cfg(not(target_os = "macos"))]
                {
                    use arboard::Clipboard;
                    match Clipboard::new() {
                        Ok(mut clipboard) => match clipboard.set_text(&path_info.path) {
                            Ok(_) => {
                                logging::log(
                                    "UI",
                                    &format!("Copied path to clipboard: {}", path_info.path),
                                );
                                self.last_output =
                                    Some(SharedString::from(format!("Copied: {}", path_info.path)));
                            }
                            Err(e) => {
                                logging::log("ERROR", &format!("Failed to copy path: {}", e));
                                self.last_output = Some(SharedString::from("Failed to copy path"));
                            }
                        },
                        Err(e) => {
                            logging::log("ERROR", &format!("Failed to access clipboard: {}", e));
                            self.last_output =
                                Some(SharedString::from("Failed to access clipboard"));
                        }
                    }
                }
            }
            "copy_filename" => {
                // Copy just the filename to clipboard
                #[cfg(target_os = "macos")]
                {
                    use std::io::Write;
                    use std::process::{Command, Stdio};

                    match Command::new("pbcopy").stdin(Stdio::piped()).spawn() {
                        Ok(mut child) => {
                            if let Some(ref mut stdin) = child.stdin {
                                if stdin.write_all(path_info.name.as_bytes()).is_ok() {
                                    let _ = child.wait();
                                    logging::log(
                                        "UI",
                                        &format!(
                                            "Copied filename to clipboard: {}",
                                            path_info.name
                                        ),
                                    );
                                    self.last_output = Some(SharedString::from(format!(
                                        "Copied: {}",
                                        path_info.name
                                    )));
                                }
                            }
                        }
                        Err(e) => {
                            logging::log("ERROR", &format!("Failed to spawn pbcopy: {}", e));
                        }
                    }
                }
            }
            "open_in_finder" => {
                // Reveal in Finder (macOS)
                #[cfg(target_os = "macos")]
                {
                    use std::process::Command;
                    let path_to_reveal = if path_info.is_dir {
                        path_info.path.clone()
                    } else {
                        // For files, reveal the containing folder with the file selected
                        path_info.path.clone()
                    };

                    match Command::new("open").args(["-R", &path_to_reveal]).spawn() {
                        Ok(_) => {
                            logging::log("UI", &format!("Revealed in Finder: {}", path_info.path));
                            // Hide window and set reset flag after opening external app
                            script_kit_gpui::set_main_window_visible(false);
                            NEEDS_RESET.store(true, Ordering::SeqCst);
                            cx.hide();
                        }
                        Err(e) => {
                            logging::log("ERROR", &format!("Failed to reveal in Finder: {}", e));
                            self.last_output =
                                Some(SharedString::from("Failed to reveal in Finder"));
                        }
                    }
                }
            }
            "open_in_editor" => {
                // Open in configured editor
                let editor = self.config.get_editor();
                let path_str = path_info.path.clone();
                logging::log(
                    "UI",
                    &format!("Opening in editor '{}': {}", editor, path_str),
                );

                match std::process::Command::new(&editor).arg(&path_str).spawn() {
                    Ok(_) => {
                        logging::log("UI", &format!("Opened in editor: {}", path_str));
                        // Hide window and set reset flag after opening external app
                        script_kit_gpui::set_main_window_visible(false);
                        NEEDS_RESET.store(true, Ordering::SeqCst);
                        cx.hide();
                    }
                    Err(e) => {
                        logging::log("ERROR", &format!("Failed to open in editor: {}", e));
                        self.last_output = Some(SharedString::from("Failed to open in editor"));
                    }
                }
            }
            "open_in_terminal" => {
                // Open terminal at this location
                #[cfg(target_os = "macos")]
                {
                    use std::process::Command;
                    // Get the directory (if file, use parent directory)
                    let dir_path = if path_info.is_dir {
                        path_info.path.clone()
                    } else {
                        std::path::Path::new(&path_info.path)
                            .parent()
                            .map(|p| p.to_string_lossy().to_string())
                            .unwrap_or_else(|| path_info.path.clone())
                    };

                    // Try iTerm first, fall back to Terminal.app
                    let script = format!(
                        r#"tell application "Terminal"
                            do script "cd '{}'"
                            activate
                        end tell"#,
                        dir_path
                    );

                    match Command::new("osascript").args(["-e", &script]).spawn() {
                        Ok(_) => {
                            logging::log("UI", &format!("Opened terminal at: {}", dir_path));
                            // Hide window and set reset flag after opening external app
                            script_kit_gpui::set_main_window_visible(false);
                            NEEDS_RESET.store(true, Ordering::SeqCst);
                            cx.hide();
                        }
                        Err(e) => {
                            logging::log("ERROR", &format!("Failed to open terminal: {}", e));
                            self.last_output = Some(SharedString::from("Failed to open terminal"));
                        }
                    }
                }
            }
            "move_to_trash" => {
                // Move to trash (macOS)
                #[cfg(target_os = "macos")]
                {
                    use std::process::Command;
                    let path_str = path_info.path.clone();
                    let name = path_info.name.clone();

                    // Use AppleScript to move to trash (preserves undo capability)
                    let script = format!(
                        r#"tell application "Finder"
                            delete POSIX file "{}"
                        end tell"#,
                        path_str
                    );

                    match Command::new("osascript").args(["-e", &script]).spawn() {
                        Ok(mut child) => {
                            // Wait for completion and check result
                            match child.wait() {
                                Ok(status) if status.success() => {
                                    logging::log("UI", &format!("Moved to trash: {}", path_str));
                                    self.last_output = Some(SharedString::from(format!(
                                        "Moved to Trash: {}",
                                        name
                                    )));
                                    // Refresh the path prompt to show the file is gone
                                    path_prompt_entity.update(cx, |prompt, cx| {
                                        let current = prompt.current_path.clone();
                                        prompt.navigate_to(&current, cx);
                                    });
                                }
                                _ => {
                                    logging::log(
                                        "ERROR",
                                        &format!("Failed to move to trash: {}", path_str),
                                    );
                                    self.last_output =
                                        Some(SharedString::from("Failed to move to Trash"));
                                }
                            }
                        }
                        Err(e) => {
                            logging::log("ERROR", &format!("Failed to spawn trash command: {}", e));
                            self.last_output = Some(SharedString::from("Failed to move to Trash"));
                        }
                    }
                }
            }
            _ => {
                logging::log("UI", &format!("Unknown path action: {}", action_id));
            }
        }

        cx.notify();
    }

    /// Execute a scriptlet (simple code snippet from .md file)
    fn execute_scriptlet(&mut self, scriptlet: &scripts::Scriptlet, cx: &mut Context<Self>) {
        logging::log(
            "EXEC",
            &format!(
                "Executing scriptlet: {} (tool: {})",
                scriptlet.name, scriptlet.tool
            ),
        );

        let tool = scriptlet.tool.to_lowercase();

        // TypeScript/Kit scriptlets need to run interactively (they may use SDK prompts)
        // These should be spawned like regular scripts, not run synchronously
        if matches!(tool.as_str(), "kit" | "ts" | "bun" | "deno" | "js") {
            logging::log(
                "EXEC",
                &format!(
                    "TypeScript scriptlet '{}' - running interactively",
                    scriptlet.name
                ),
            );

            // Write scriptlet content to a temp file
            let temp_dir = std::env::temp_dir();
            let temp_file = temp_dir.join(format!(
                "scriptlet-{}-{}.ts",
                scriptlet.name.to_lowercase().replace(' ', "-"),
                std::process::id()
            ));

            if let Err(e) = std::fs::write(&temp_file, &scriptlet.code) {
                logging::log(
                    "ERROR",
                    &format!("Failed to write temp scriptlet file: {}", e),
                );
                self.toast_manager.push(
                    components::toast::Toast::error(
                        format!("Failed to write scriptlet: {}", e),
                        &self.theme,
                    )
                    .duration_ms(Some(5000)),
                );
                cx.notify();
                return;
            }

            // Create a Script struct and run it interactively
            let script = scripts::Script {
                name: scriptlet.name.clone(),
                description: scriptlet.description.clone(),
                path: temp_file,
                extension: "ts".to_string(),
                icon: None,
                alias: None,
                shortcut: None,
                typed_metadata: None,
                schema: None,
            };

            self.execute_interactive(&script, cx);
            return;
        }

        // For non-TypeScript tools (bash, python, etc.), run synchronously
        // These don't use the SDK and won't block waiting for input

        // Convert scripts::Scriptlet to scriptlets::Scriptlet for executor
        let exec_scriptlet = scriptlets::Scriptlet {
            name: scriptlet.name.clone(),
            command: scriptlet.command.clone().unwrap_or_else(|| {
                // Generate command slug from name if not present
                scriptlet.name.to_lowercase().replace(' ', "-")
            }),
            tool: scriptlet.tool.clone(),
            scriptlet_content: scriptlet.code.clone(),
            inputs: vec![], // TODO: Parse inputs from code if needed
            group: scriptlet.group.clone().unwrap_or_default(),
            preview: None,
            metadata: scriptlets::ScriptletMetadata {
                shortcut: scriptlet.shortcut.clone(),
                expand: scriptlet.expand.clone(),
                description: scriptlet.description.clone(),
                ..Default::default()
            },
            typed_metadata: None,
            schema: None,
            kit: None,
            source_path: scriptlet.file_path.clone(),
        };

        // Execute with default options (no inputs for now)
        let options = executor::ScriptletExecOptions::default();

        match executor::run_scriptlet(&exec_scriptlet, options) {
            Ok(result) => {
                if result.success {
                    logging::log(
                        "EXEC",
                        &format!(
                            "Scriptlet '{}' succeeded: exit={}",
                            scriptlet.name, result.exit_code
                        ),
                    );

                    // Handle special tool types that need interactive prompts
                    if tool == "template" && !result.stdout.is_empty() {
                        // Template tool: show template prompt with the content
                        let id = format!("scriptlet-template-{}", uuid::Uuid::new_v4());
                        logging::log(
                            "EXEC",
                            &format!(
                                "Template scriptlet '{}' - showing template prompt",
                                scriptlet.name
                            ),
                        );
                        self.handle_prompt_message(
                            PromptMessage::ShowTemplate {
                                id,
                                template: result.stdout.clone(),
                            },
                            cx,
                        );
                        return;
                    }

                    // Store output if any
                    if !result.stdout.is_empty() {
                        self.last_output = Some(SharedString::from(result.stdout.clone()));
                    }

                    // Hide window after successful execution
                    script_kit_gpui::set_main_window_visible(false);
                    cx.hide();
                } else {
                    // Execution failed (non-zero exit code)
                    let error_msg = if !result.stderr.is_empty() {
                        result.stderr.clone()
                    } else {
                        format!("Exit code: {}", result.exit_code)
                    };

                    logging::log(
                        "ERROR",
                        &format!("Scriptlet '{}' failed: {}", scriptlet.name, error_msg),
                    );

                    self.toast_manager.push(
                        components::toast::Toast::error(
                            format!("Scriptlet failed: {}", error_msg),
                            &self.theme,
                        )
                        .duration_ms(Some(5000)),
                    );
                    cx.notify();
                }
            }
            Err(e) => {
                logging::log(
                    "ERROR",
                    &format!("Failed to execute scriptlet '{}': {}", scriptlet.name, e),
                );

                self.toast_manager.push(
                    components::toast::Toast::error(
                        format!("Failed to execute: {}", e),
                        &self.theme,
                    )
                    .duration_ms(Some(5000)),
                );
                cx.notify();
            }
        }
    }

    /// Execute a script or scriptlet by its file path
    /// Used by global shortcuts to directly invoke scripts
    #[allow(dead_code)]
    fn execute_script_by_path(&mut self, path: &str, cx: &mut Context<Self>) {
        logging::log("EXEC", &format!("Executing script by path: {}", path));

        // Check if it's a scriptlet (contains #)
        if path.contains('#') {
            // It's a scriptlet path like "/path/to/file.md#command"
            if let Some(scriptlet) = self
                .scriptlets
                .iter()
                .find(|s| s.file_path.as_ref().map(|p| p == path).unwrap_or(false))
            {
                let scriptlet_clone = scriptlet.clone();
                self.execute_scriptlet(&scriptlet_clone, cx);
                return;
            }
            logging::log("ERROR", &format!("Scriptlet not found: {}", path));
            return;
        }

        // It's a regular script - find by path
        if let Some(script) = self
            .scripts
            .iter()
            .find(|s| s.path.to_string_lossy() == path)
        {
            let script_clone = script.clone();
            self.execute_interactive(&script_clone, cx);
            return;
        }

        // Not found in loaded scripts - try to execute directly as a file
        let script_path = std::path::PathBuf::from(path);
        if script_path.exists() {
            let name = script_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("script")
                .to_string();

            let script = scripts::Script {
                name,
                path: script_path.clone(),
                extension: script_path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("ts")
                    .to_string(),
                description: None,
                icon: None,
                alias: None,
                shortcut: None,
                typed_metadata: None,
                schema: None,
            };

            self.execute_interactive(&script, cx);
        } else {
            logging::log("ERROR", &format!("Script file not found: {}", path));
        }
    }

    /// Cancel the currently running script and clean up all state
    fn cancel_script_execution(&mut self, cx: &mut Context<Self>) {
        logging::log("EXEC", "=== Canceling script execution ===");

        // Send cancel message to script (Exit with cancel code)
        if let Some(ref sender) = self.response_sender {
            // Try to send Exit message to terminate the script cleanly
            let exit_msg = Message::Exit {
                code: Some(1), // Non-zero code indicates cancellation
                message: Some("Cancelled by user".to_string()),
            };
            match sender.send(exit_msg) {
                Ok(()) => logging::log("EXEC", "Sent Exit message to script"),
                Err(e) => logging::log(
                    "EXEC",
                    &format!("Failed to send Exit: {} (script may have exited)", e),
                ),
            }
        } else {
            logging::log("EXEC", "No response_sender - script may not be running");
        }

        // Belt-and-suspenders: Force-kill the process group using stored PID
        // This ensures cleanup even if Drop doesn't fire properly
        if let Some(pid) = self.current_script_pid.take() {
            logging::log(
                "CLEANUP",
                &format!("Force-killing script process group {}", pid),
            );
            #[cfg(unix)]
            {
                let _ = std::process::Command::new("kill")
                    .args(["-9", &format!("-{}", pid)])
                    .output();
            }
        }

        // Abort script session if it exists
        {
            let mut session_guard = self.script_session.lock();
            if let Some(_session) = session_guard.take() {
                logging::log("EXEC", "Cleared script session");
            }
        }

        // Reset to script list view
        self.reset_to_script_list(cx);
        logging::log("EXEC", "=== Script cancellation complete ===");
    }

    /// Flush pending toasts from ToastManager to gpui-component's NotificationList
    ///
    /// This should be called at the start of render() where we have window access.
    /// The ToastManager acts as a staging queue for toasts pushed from callbacks
    /// that don't have window access.
    fn flush_pending_toasts(&mut self, window: &mut gpui::Window, cx: &mut gpui::App) {
        use gpui_component::WindowExt;

        let pending = self.toast_manager.drain_pending();
        for toast in pending {
            let notification = pending_toast_to_notification(&toast);
            window.push_notification(notification, cx);
        }
    }

    /// Close window and reset to default state (Cmd+W global handler)
    ///
    /// This method handles the global Cmd+W shortcut which should work
    /// regardless of what prompt or view is currently active. It:
    /// 1. Cancels any running script
    /// 2. Resets state to the default script list
    /// 3. Hides the window
    fn close_and_reset_window(&mut self, cx: &mut Context<Self>) {
        logging::log("VISIBILITY", "=== Close and reset window ===");

        // Update visibility state FIRST to prevent race conditions
        script_kit_gpui::set_main_window_visible(false);
        logging::log("VISIBILITY", "WINDOW_VISIBLE set to: false");

        // If in a prompt, cancel the script execution
        if self.is_in_prompt() {
            logging::log("VISIBILITY", "In prompt mode - canceling script before hiding");
            self.cancel_script_execution(cx);
        } else {
            // Just reset to script list (clears filter, selection, scroll)
            self.reset_to_script_list(cx);
        }

        // Check if Notes or AI windows are open BEFORE hiding
        let notes_open = notes::is_notes_window_open();
        let ai_open = ai::is_ai_window_open();
        logging::log(
            "VISIBILITY",
            &format!(
                "Secondary windows: notes_open={}, ai_open={}",
                notes_open, ai_open
            ),
        );

        // CRITICAL: Only hide main window if Notes/AI are open
        // cx.hide() hides the ENTIRE app (all windows), so we use
        // platform::hide_main_window() to hide only the main window
        if notes_open || ai_open {
            logging::log(
                "VISIBILITY",
                "Using hide_main_window() - secondary windows are open",
            );
            platform::hide_main_window();
        } else {
            logging::log("VISIBILITY", "Using cx.hide() - no secondary windows");
            cx.hide();
        }
        logging::log("VISIBILITY", "=== Window closed ===");
    }

    /// Handle global keyboard shortcuts with configurable dismissability
    ///
    /// Returns `true` if the shortcut was handled (caller should return early)
    ///
    /// # Arguments
    /// * `event` - The key down event to check
    /// * `is_dismissable` - If true, ESC key will also close the window (for prompts like arg, div, form, etc.)
    ///   If false, only Cmd+W closes the window (for prompts like term, editor)
    /// * `cx` - The context
    ///
    /// # Handled shortcuts
    /// - Cmd+W: Always closes window and resets to default state
    /// - Escape: Only closes window if `is_dismissable` is true AND actions popup is not showing
    fn handle_global_shortcut_with_options(
        &mut self,
        event: &gpui::KeyDownEvent,
        is_dismissable: bool,
        cx: &mut Context<Self>,
    ) -> bool {
        let key_str = event.keystroke.key.to_lowercase();
        let has_cmd = event.keystroke.modifiers.platform;

        // Cmd+W always closes window
        if has_cmd && key_str == "w" {
            logging::log("KEY", "Cmd+W - closing window");
            self.close_and_reset_window(cx);
            return true;
        }

        // ESC closes dismissable prompts (when actions popup is not showing)
        if is_dismissable && key_str == "escape" && !self.show_actions_popup {
            logging::log("KEY", "ESC in dismissable prompt - closing window");
            self.close_and_reset_window(cx);
            return true;
        }

        false
    }

    /// Check if the current view is a dismissable prompt
    ///
    /// Dismissable prompts are those that feel "closeable" with escape:
    /// - ArgPrompt, DivPrompt, FormPrompt, SelectPrompt, PathPrompt, EnvPrompt, DropPrompt, TemplatePrompt
    /// - Built-in views (ClipboardHistory, AppLauncher, WindowSwitcher, DesignGallery)
    /// - ScriptList
    ///
    /// Non-dismissable prompts:
    /// - TermPrompt, EditorPrompt (these require explicit Cmd+W to close)
    #[allow(dead_code)]
    fn is_dismissable_view(&self) -> bool {
        !matches!(
            self.current_view,
            AppView::TermPrompt { .. } | AppView::EditorPrompt { .. }
        )
    }

    /// Show a HUD (heads-up display) overlay message
    ///
    /// This creates a separate floating window positioned at bottom-center of the
    /// screen containing the mouse cursor. The HUD is independent of the main
    /// Script Kit window and will remain visible even when the main window is hidden.
    ///
    /// Position: Bottom-center (85% down screen)
    /// Duration: 2000ms default, configurable
    /// Shape: Pill (40px tall, variable width)
    fn show_hud(&mut self, text: String, duration_ms: Option<u64>, cx: &mut Context<Self>) {
        // Delegate to the HUD manager which creates a separate floating window
        // This ensures the HUD is visible even when the main app window is hidden
        hud_manager::show_hud(text, duration_ms, cx);
    }

    /// Show the debug grid overlay with specified options
    ///
    /// This method converts protocol::GridOptions to debug_grid::GridConfig
    /// and enables the grid overlay rendering.
    fn show_grid(&mut self, options: protocol::GridOptions, cx: &mut Context<Self>) {
        use debug_grid::{GridColorScheme, GridConfig, GridDepth};
        use protocol::GridDepthOption;

        // Convert protocol depth to debug_grid depth
        let depth = match &options.depth {
            GridDepthOption::Preset(s) if s == "all" => GridDepth::All,
            GridDepthOption::Preset(_) => GridDepth::Prompts,
            GridDepthOption::Components(names) => GridDepth::Components(names.clone()),
        };

        self.grid_config = Some(GridConfig {
            grid_size: options.grid_size,
            show_bounds: options.show_bounds,
            show_box_model: options.show_box_model,
            show_alignment_guides: options.show_alignment_guides,
            show_dimensions: options.show_dimensions,
            depth,
            color_scheme: GridColorScheme::default(),
        });

        logging::log(
            "DEBUG_GRID",
            &format!(
                "Grid overlay enabled: size={}, bounds={}, box_model={}, guides={}, dimensions={}",
                options.grid_size,
                options.show_bounds,
                options.show_box_model,
                options.show_alignment_guides,
                options.show_dimensions
            ),
        );

        cx.notify();
    }

    /// Hide the debug grid overlay
    fn hide_grid(&mut self, cx: &mut Context<Self>) {
        self.grid_config = None;
        logging::log("DEBUG_GRID", "Grid overlay hidden");
        cx.notify();
    }


    /// Rebuild alias and shortcut registries from current scripts/scriptlets.
    /// Returns a list of conflict messages (if any) for HUD display.
    /// Conflict rule: first-registered wins - duplicates are blocked.
    fn rebuild_registries(&mut self) -> Vec<String> {
        let mut conflicts = Vec::new();
        self.alias_registry.clear();
        self.shortcut_registry.clear();

        // Register script aliases
        for script in &self.scripts {
            if let Some(ref alias) = script.alias {
                let alias_lower = alias.to_lowercase();
                if let Some(existing_path) = self.alias_registry.get(&alias_lower) {
                    conflicts.push(format!(
                        "Alias conflict: '{}' already used by {}",
                        alias,
                        std::path::Path::new(existing_path)
                            .file_name()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_else(|| existing_path.clone())
                    ));
                    logging::log(
                        "ALIAS",
                        &format!(
                            "Conflict: alias '{}' in {} blocked (already used by {})",
                            alias,
                            script.path.display(),
                            existing_path
                        ),
                    );
                } else {
                    self.alias_registry
                        .insert(alias_lower, script.path.to_string_lossy().to_string());
                }
            }
        }

        // Register scriptlet aliases
        for scriptlet in &self.scriptlets {
            if let Some(ref alias) = scriptlet.alias {
                let alias_lower = alias.to_lowercase();
                if let Some(existing_path) = self.alias_registry.get(&alias_lower) {
                    conflicts.push(format!(
                        "Alias conflict: '{}' already used by {}",
                        alias,
                        std::path::Path::new(existing_path)
                            .file_name()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_else(|| existing_path.clone())
                    ));
                    logging::log(
                        "ALIAS",
                        &format!(
                            "Conflict: alias '{}' in {} blocked (already used by {})",
                            alias, scriptlet.name, existing_path
                        ),
                    );
                } else {
                    let path = scriptlet
                        .file_path
                        .clone()
                        .unwrap_or_else(|| scriptlet.name.clone());
                    self.alias_registry.insert(alias_lower, path);
                }
            }

            // Register scriptlet shortcuts
            if let Some(ref shortcut) = scriptlet.shortcut {
                let shortcut_lower = shortcut.to_lowercase();
                if let Some(existing_path) = self.shortcut_registry.get(&shortcut_lower) {
                    conflicts.push(format!(
                        "Shortcut conflict: '{}' already used by {}",
                        shortcut,
                        std::path::Path::new(existing_path)
                            .file_name()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_else(|| existing_path.clone())
                    ));
                    logging::log(
                        "SHORTCUT",
                        &format!(
                            "Conflict: shortcut '{}' in {} blocked (already used by {})",
                            shortcut, scriptlet.name, existing_path
                        ),
                    );
                } else {
                    let path = scriptlet
                        .file_path
                        .clone()
                        .unwrap_or_else(|| scriptlet.name.clone());
                    self.shortcut_registry.insert(shortcut_lower, path);
                }
            }
        }

        logging::log(
            "REGISTRY",
            &format!(
                "Rebuilt registries: {} aliases, {} shortcuts, {} conflicts",
                self.alias_registry.len(),
                self.shortcut_registry.len(),
                conflicts.len()
            ),
        );

        conflicts
    }

    /// Reset all state and return to the script list view.
    /// This clears all prompt state and resizes the window appropriately.
    fn reset_to_script_list(&mut self, cx: &mut Context<Self>) {
        let old_view = match &self.current_view {
            AppView::ScriptList => "ScriptList",
            AppView::ActionsDialog => "ActionsDialog",
            AppView::ArgPrompt { .. } => "ArgPrompt",
            AppView::DivPrompt { .. } => "DivPrompt",
            AppView::FormPrompt { .. } => "FormPrompt",
            AppView::TermPrompt { .. } => "TermPrompt",
            AppView::EditorPrompt { .. } => "EditorPrompt",
            AppView::SelectPrompt { .. } => "SelectPrompt",
            AppView::PathPrompt { .. } => "PathPrompt",
            AppView::EnvPrompt { .. } => "EnvPrompt",
            AppView::DropPrompt { .. } => "DropPrompt",
            AppView::TemplatePrompt { .. } => "TemplatePrompt",
            AppView::ClipboardHistoryView { .. } => "ClipboardHistoryView",
            AppView::AppLauncherView { .. } => "AppLauncherView",
            AppView::WindowSwitcherView { .. } => "WindowSwitcherView",
            AppView::DesignGalleryView { .. } => "DesignGalleryView",
        };

        let old_focused_input = self.focused_input;
        logging::log(
            "UI",
            &format!(
                "Resetting to script list (was: {}, focused_input: {:?})",
                old_view, old_focused_input
            ),
        );

        // Belt-and-suspenders: Force-kill the process group using stored PID
        // This runs BEFORE clearing channels to ensure cleanup even if Drop doesn't fire
        if let Some(pid) = self.current_script_pid.take() {
            logging::log(
                "CLEANUP",
                &format!("Force-killing script process group {} during reset", pid),
            );
            #[cfg(unix)]
            {
                let _ = std::process::Command::new("kill")
                    .args(["-9", &format!("-{}", pid)])
                    .output();
            }
        }

        // Reset view
        self.current_view = AppView::ScriptList;

        // CRITICAL: Reset focused_input to MainFilter so the cursor appears
        // This was a bug where focused_input could remain as ArgPrompt/None after
        // script exit, causing the cursor to not show in the main filter.
        self.focused_input = FocusedInput::MainFilter;
        self.gpui_input_focused = false;
        logging::log(
            "FOCUS",
            "Reset focused_input to MainFilter for cursor display",
        );

        // Clear arg prompt state
        self.arg_input.clear();
        self.arg_selected_index = 0;
        // P0: Reset arg scroll handle
        self.arg_list_scroll_handle
            .scroll_to_item(0, ScrollStrategy::Top);

        // Clear filter and selection state for fresh menu
        self.filter_text.clear();
        self.computed_filter_text.clear();
        self.filter_coalescer.reset();
        self.pending_filter_sync = true;
        self.selected_index = 0;
        self.last_scrolled_index = None;
        // Use main_list_state for variable-height list (not the legacy list_scroll_handle)
        self.main_list_state.scroll_to_reveal_item(0);
        self.last_scrolled_index = Some(0);

        // Resize window for script list content
        let count = self.scripts.len() + self.scriptlets.len();
        resize_first_window_to_height(height_for_view(ViewType::ScriptList, count));

        // Clear output
        self.last_output = None;

        // Clear channels (they will be dropped, closing the connections)
        self.prompt_receiver = None;
        self.response_sender = None;

        // Clear script session (parking_lot mutex never poisons)
        *self.script_session.lock() = None;

        // Clear actions popup state (prevents stale actions dialog from persisting)
        self.show_actions_popup = false;
        self.actions_dialog = None;

        // Clear pending path action and close signal
        if let Ok(mut guard) = self.pending_path_action.lock() {
            *guard = None;
        }
        if let Ok(mut guard) = self.close_path_actions.lock() {
            *guard = false;
        }

        logging::log(
            "UI",
            "State reset complete - view is now ScriptList (filter, selection, scroll cleared)",
        );
        cx.notify();
    }

    /// Check if we're currently in a prompt view (script is running)
    fn is_in_prompt(&self) -> bool {
        matches!(
            self.current_view,
            AppView::ArgPrompt { .. }
                | AppView::DivPrompt { .. }
                | AppView::FormPrompt { .. }
                | AppView::TermPrompt { .. }
                | AppView::EditorPrompt { .. }
                | AppView::ClipboardHistoryView { .. }
                | AppView::AppLauncherView { .. }
                | AppView::WindowSwitcherView { .. }
                | AppView::DesignGalleryView { .. }
        )
    }

    /// Submit a response to the current prompt
    fn submit_prompt_response(
        &mut self,
        id: String,
        value: Option<String>,
        _cx: &mut Context<Self>,
    ) {
        logging::log(
            "UI",
            &format!("Submitting response for {}: {:?}", id, value),
        );

        let response = Message::Submit { id, value };

        if let Some(ref sender) = self.response_sender {
            match sender.send(response) {
                Ok(()) => {
                    logging::log("UI", "Response queued for script");
                }
                Err(e) => {
                    logging::log("UI", &format!("Failed to queue response: {}", e));
                }
            }
        } else {
            logging::log("UI", "No response sender available");
        }

        // Return to waiting state (script will send next prompt or exit)
        // Don't change view here - wait for next message from script
    }

    /// Get filtered choices for arg prompt
    fn filtered_arg_choices(&self) -> Vec<(usize, &Choice)> {
        if let AppView::ArgPrompt { choices, .. } = &self.current_view {
            if self.arg_input.is_empty() {
                choices.iter().enumerate().collect()
            } else {
                let filter = self.arg_input.text().to_lowercase();
                choices
                    .iter()
                    .enumerate()
                    .filter(|(_, c)| c.name.to_lowercase().contains(&filter))
                    .collect()
            }
        } else {
            vec![]
        }
    }

    /// P0: Get filtered choices as owned data for uniform_list closure
    fn get_filtered_arg_choices_owned(&self) -> Vec<(usize, Choice)> {
        if let AppView::ArgPrompt { choices, .. } = &self.current_view {
            if self.arg_input.is_empty() {
                choices
                    .iter()
                    .enumerate()
                    .map(|(i, c)| (i, c.clone()))
                    .collect()
            } else {
                let filter = self.arg_input.text().to_lowercase();
                choices
                    .iter()
                    .enumerate()
                    .filter(|(_, c)| c.name.to_lowercase().contains(&filter))
                    .map(|(i, c)| (i, c.clone()))
                    .collect()
            }
        } else {
            vec![]
        }
    }

    /// Convert hex color to rgba with opacity from theme
    fn hex_to_rgba_with_opacity(&self, hex: u32, opacity: f32) -> u32 {
        // Convert opacity (0.0-1.0) to alpha byte (0-255)
        let alpha = (opacity.clamp(0.0, 1.0) * 255.0) as u32;
        (hex << 8) | alpha
    }

    /// Create box shadows from theme configuration
    fn create_box_shadows(&self) -> Vec<BoxShadow> {
        let shadow_config = self.theme.get_drop_shadow();

        if !shadow_config.enabled {
            return vec![];
        }

        // Convert hex color to HSLA
        // For black (0x000000), we use h=0, s=0, l=0
        let r = ((shadow_config.color >> 16) & 0xFF) as f32 / 255.0;
        let g = ((shadow_config.color >> 8) & 0xFF) as f32 / 255.0;
        let b = (shadow_config.color & 0xFF) as f32 / 255.0;

        // Simple RGB to HSL conversion for shadow color
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let l = (max + min) / 2.0;

        let (h, s) = if max == min {
            (0.0, 0.0) // achromatic
        } else {
            let d = max - min;
            let s = if l > 0.5 {
                d / (2.0 - max - min)
            } else {
                d / (max + min)
            };
            let h = if max == r {
                (g - b) / d + if g < b { 6.0 } else { 0.0 }
            } else if max == g {
                (b - r) / d + 2.0
            } else {
                (r - g) / d + 4.0
            };
            (h / 6.0, s)
        };

        vec![BoxShadow {
            color: hsla(h, s, l, shadow_config.opacity),
            offset: point(px(shadow_config.offset_x), px(shadow_config.offset_y)),
            blur_radius: px(shadow_config.blur_radius),
            spread_radius: px(shadow_config.spread_radius),
        }]
    }
}

// Note: convert_menu_bar_items/convert_menu_bar_item functions were removed
// because frontmost_app_tracker is now compiled as part of the binary crate
// (via `mod frontmost_app_tracker` in main.rs) so it returns binary types directly.

</file>

<file path="src/main.rs">
#![allow(unexpected_cfgs)]

use gpui::{
    div, hsla, list, point, prelude::*, px, rgb, rgba, size, svg, uniform_list, AnyElement, App,
    Application, BoxShadow, Context, ElementId, Entity, FocusHandle, Focusable, ListAlignment,
    ListSizingBehavior, ListState, Render, ScrollStrategy, SharedString, Subscription, Timer,
    UniformListScrollHandle, Window, WindowBackgroundAppearance, WindowBounds, WindowHandle,
    WindowOptions,
};

// gpui-component Root wrapper for theme and context provision
use gpui_component::input::{Input, InputEvent, InputState};
use gpui_component::notification::{Notification, NotificationType};
use gpui_component::Root;
use gpui_component::{Sizable, Size};
use std::sync::atomic::{AtomicBool, Ordering};

mod process_manager;
use cocoa::base::id;
use cocoa::foundation::NSRect;
use process_manager::PROCESS_MANAGER;

// Platform utilities - mouse position, display info, window movement, screenshots
use platform::{
    calculate_eye_line_bounds_on_mouse_display, capture_app_screenshot, capture_window_by_title,
    move_first_window_to_bounds,
};
#[macro_use]
extern crate objc;

mod actions;
mod agents;
mod ai;
mod components;
mod config;
mod designs;
mod editor;
mod error;
mod executor;
mod filter_coalescer;
mod form_prompt;
#[allow(dead_code)] // TODO: Re-enable once hotkey_pollers is updated for Root wrapper
mod hotkey_pollers;
mod hotkeys;
mod list_item;
mod logging;
mod login_item;
mod navigation;
mod panel;
mod perf;
mod platform;
mod prompts;
mod protocol;
mod scripts;
#[cfg(target_os = "macos")]
mod selected_text;
mod setup;
mod shortcuts;
mod stdin_commands;
mod syntax;
mod term_prompt;
mod terminal;
mod theme;
mod transitions;
mod tray;
mod utils;
mod warning_banner;
mod watcher;
mod window_manager;
mod window_resize;

// Phase 1 system API modules
mod clipboard_history;
mod file_search;
mod toast_manager;
mod window_control;

// System actions - macOS AppleScript-based system commands
#[cfg(target_os = "macos")]
mod system_actions;

// Script creation - Create new scripts and scriptlets
mod script_creation;

// Permissions wizard - Check and request macOS permissions
mod permissions_wizard;

// Built-in features registry
mod app_launcher;
mod builtins;
mod menu_bar;

// Frontmost app tracker - Background observer for tracking active application
#[cfg(target_os = "macos")]
mod frontmost_app_tracker;

// Frecency tracking for script usage
mod frecency;

// Scriptlet parsing and variable substitution
mod scriptlets;

// Typed metadata parser for new `metadata = {}` global syntax
mod metadata_parser;

// Schema parser for `schema = { input: {}, output: {} }` definitions
mod schema_parser;

// Scriptlet codefence metadata parser for ```metadata and ```schema blocks
mod scriptlet_metadata;

// VSCode snippet syntax parser for template() SDK function
mod snippet;

// HTML form parsing for form() prompt
mod form_parser;

// Centralized template variable substitution system
mod template_variables;

// Text expansion system components (macOS only)
mod expand_matcher;
#[cfg(target_os = "macos")]
mod keyboard_monitor;
mod text_injector;

// Expand manager - text expansion system integration
#[cfg(target_os = "macos")]
mod expand_manager;

// Script scheduling with cron expressions and natural language
mod scheduler;

// HUD manager - system-level overlay notifications (separate floating windows)
mod hud_manager;

// Debug grid overlay for visual testing
mod debug_grid;

// MCP Server modules for AI agent integration
mod mcp_kit_tools;
mod mcp_protocol;
mod mcp_resources;
mod mcp_script_tools;
mod mcp_server;
mod mcp_streaming;

// Notes - Raycast Notes feature parity (separate floating window)
mod notes;

use crate::components::text_input::TextInputState;
use crate::components::toast::{Toast, ToastAction};
use crate::error::ErrorSeverity;
use crate::filter_coalescer::FilterCoalescer;
use crate::form_prompt::FormPromptState;
// TODO: Re-enable when hotkey_pollers.rs is updated for Root wrapper
// use crate::hotkey_pollers::start_hotkey_event_handler;
use crate::navigation::{NavCoalescer, NavDirection, NavRecord};
use crate::toast_manager::{PendingToast, ToastManager};
use components::ToastVariant;
use editor::EditorPrompt;
use prompts::{
    ContainerOptions, ContainerPadding, DivPrompt, DropPrompt, EnvPrompt, PathInfo, PathPrompt,
    SelectPrompt, TemplatePrompt,
};
use tray::{TrayManager, TrayMenuAction};
use warning_banner::{WarningBanner, WarningBannerColors};
use window_resize::{
    defer_resize_to_view, height_for_view, initial_window_height, reset_resize_debounce,
    resize_first_window_to_height, ViewType,
};

use components::{
    Button, ButtonColors, ButtonVariant, FormFieldColors, Scrollbar, ScrollbarColors,
};
use designs::{get_tokens, render_design_item, DesignVariant};
use frecency::FrecencyStore;
use list_item::{
    render_section_header, GroupedListItem, ListItem, ListItemColors, LIST_ITEM_HEIGHT,
    SECTION_HEADER_HEIGHT,
};
use scripts::get_grouped_results;
// strip_html_tags removed - DivPrompt now renders HTML properly

use actions::{ActionsDialog, ScriptInfo};
use panel::{
    CURSOR_GAP_X, CURSOR_HEIGHT_LG, CURSOR_MARGIN_Y, CURSOR_WIDTH, DEFAULT_PLACEHOLDER, HEADER_GAP,
    HEADER_PADDING_X, HEADER_PADDING_Y,
};
use parking_lot::Mutex as ParkingMutex;
use protocol::{Choice, Message, ProtocolAction};
use std::sync::{mpsc, Arc, Mutex};
use syntax::highlight_code_lines;

/// Channel for sending prompt messages from script thread to UI
#[allow(dead_code)]
type PromptChannel = (mpsc::Sender<PromptMessage>, mpsc::Receiver<PromptMessage>);

// Import utilities from modules
use stdin_commands::{start_stdin_listener, ExternalCommand};
use utils::render_path_with_highlights;

// Global state for hotkey signaling between threads
static NEEDS_RESET: AtomicBool = AtomicBool::new(false); // Track if window needs reset to script list on next show

pub use script_kit_gpui::{is_main_window_visible, set_main_window_visible};
static PANEL_CONFIGURED: AtomicBool = AtomicBool::new(false); // Track if floating panel has been configured (one-time setup on first show)
static SHUTDOWN_REQUESTED: AtomicBool = AtomicBool::new(false); // Track if shutdown signal received (prevents new script spawns)

/// Convert our ToastVariant to gpui-component's NotificationType
fn toast_variant_to_notification_type(variant: ToastVariant) -> NotificationType {
    match variant {
        ToastVariant::Success => NotificationType::Success,
        ToastVariant::Warning => NotificationType::Warning,
        ToastVariant::Error => NotificationType::Error,
        ToastVariant::Info => NotificationType::Info,
    }
}

/// Convert a PendingToast to a gpui-component Notification
fn pending_toast_to_notification(toast: &PendingToast) -> Notification {
    let notification_type = toast_variant_to_notification_type(toast.variant);

    let mut notification = Notification::new()
        .message(&toast.message)
        .with_type(notification_type);

    // Add title for errors/warnings (makes them stand out more)
    match toast.variant {
        ToastVariant::Error => {
            notification = notification.title("Error");
        }
        ToastVariant::Warning => {
            notification = notification.title("Warning");
        }
        _ => {}
    }

    // Note: gpui-component Notification has fixed 5s autohide
    // For persistent toasts, set autohide(false)
    if toast.duration_ms.is_none() {
        notification = notification.autohide(false);
    }

    notification
}

/// Check if shutdown has been requested (prevents new script spawns during shutdown)
#[allow(dead_code)]
pub fn is_shutting_down() -> bool {
    SHUTDOWN_REQUESTED.load(Ordering::SeqCst)
}

/// Register bundled JetBrains Mono font with GPUI's text system
///
/// This embeds the font files directly in the binary and registers them
/// at application startup, making "JetBrains Mono" available as a font family.
fn register_bundled_fonts(cx: &mut App) {
    use std::borrow::Cow;

    // Embed font files at compile time
    static JETBRAINS_MONO_REGULAR: &[u8] =
        include_bytes!("../assets/fonts/JetBrainsMono-Regular.ttf");
    static JETBRAINS_MONO_BOLD: &[u8] = include_bytes!("../assets/fonts/JetBrainsMono-Bold.ttf");
    static JETBRAINS_MONO_ITALIC: &[u8] =
        include_bytes!("../assets/fonts/JetBrainsMono-Italic.ttf");
    static JETBRAINS_MONO_BOLD_ITALIC: &[u8] =
        include_bytes!("../assets/fonts/JetBrainsMono-BoldItalic.ttf");
    static JETBRAINS_MONO_MEDIUM: &[u8] =
        include_bytes!("../assets/fonts/JetBrainsMono-Medium.ttf");
    static JETBRAINS_MONO_SEMIBOLD: &[u8] =
        include_bytes!("../assets/fonts/JetBrainsMono-SemiBold.ttf");

    let fonts: Vec<Cow<'static, [u8]>> = vec![
        Cow::Borrowed(JETBRAINS_MONO_REGULAR),
        Cow::Borrowed(JETBRAINS_MONO_BOLD),
        Cow::Borrowed(JETBRAINS_MONO_ITALIC),
        Cow::Borrowed(JETBRAINS_MONO_BOLD_ITALIC),
        Cow::Borrowed(JETBRAINS_MONO_MEDIUM),
        Cow::Borrowed(JETBRAINS_MONO_SEMIBOLD),
    ];

    match cx.text_system().add_fonts(fonts) {
        Ok(()) => {
            logging::log("FONT", "Registered JetBrains Mono font family (6 styles)");
        }
        Err(e) => {
            logging::log(
                "FONT",
                &format!(
                    "Failed to register JetBrains Mono: {}. Falling back to system font.",
                    e
                ),
            );
        }
    }
}

/// Application state - what view are we currently showing
#[derive(Debug, Clone)]
enum AppView {
    /// Showing the script list
    ScriptList,
    /// Showing the actions dialog (mini searchable popup)
    #[allow(dead_code)]
    ActionsDialog,
    /// Showing an arg prompt from a script
    ArgPrompt {
        id: String,
        placeholder: String,
        choices: Vec<Choice>,
        actions: Option<Vec<ProtocolAction>>,
    },
    /// Showing a div prompt from a script
    DivPrompt {
        #[allow(dead_code)]
        id: String,
        entity: Entity<DivPrompt>,
    },
    /// Showing a form prompt from a script (HTML form with submit button)
    FormPrompt {
        #[allow(dead_code)]
        id: String,
        entity: Entity<FormPromptState>,
    },
    /// Showing a terminal prompt from a script
    TermPrompt {
        #[allow(dead_code)]
        id: String,
        entity: Entity<term_prompt::TermPrompt>,
    },
    /// Showing an editor prompt from a script (gpui-component based with Find/Replace)
    EditorPrompt {
        #[allow(dead_code)]
        id: String,
        entity: Entity<EditorPrompt>,
        /// Separate focus handle for the editor (not shared with parent)
        /// Note: This is kept for API compatibility but focus is managed via entity.focus()
        #[allow(dead_code)]
        focus_handle: FocusHandle,
    },
    /// Showing a select prompt from a script (multi-select)
    SelectPrompt {
        #[allow(dead_code)]
        id: String,
        entity: Entity<SelectPrompt>,
    },
    /// Showing a path prompt from a script (file/folder picker)
    PathPrompt {
        #[allow(dead_code)]
        id: String,
        entity: Entity<PathPrompt>,
        focus_handle: FocusHandle,
    },
    /// Showing env prompt for environment variable input with keyring storage
    EnvPrompt {
        #[allow(dead_code)]
        id: String,
        entity: Entity<EnvPrompt>,
    },
    /// Showing drop prompt for drag and drop file handling
    DropPrompt {
        #[allow(dead_code)]
        id: String,
        entity: Entity<DropPrompt>,
    },
    /// Showing template prompt for string template editing
    TemplatePrompt {
        #[allow(dead_code)]
        id: String,
        entity: Entity<TemplatePrompt>,
    },
    /// Showing clipboard history
    ClipboardHistoryView {
        entries: Vec<clipboard_history::ClipboardEntry>,
        filter: String,
        selected_index: usize,
    },
    /// Showing app launcher
    AppLauncherView {
        apps: Vec<app_launcher::AppInfo>,
        filter: String,
        selected_index: usize,
    },
    /// Showing window switcher
    WindowSwitcherView {
        windows: Vec<window_control::WindowInfo>,
        filter: String,
        selected_index: usize,
    },
    /// Showing design gallery (separator and icon variations)
    DesignGalleryView {
        filter: String,
        selected_index: usize,
    },
}

/// Wrapper to hold a script session that can be shared across async boundaries
/// Uses parking_lot::Mutex which doesn't poison on panic, avoiding .unwrap() calls
type SharedSession = Arc<ParkingMutex<Option<executor::ScriptSession>>>;

/// Tracks which input field currently has focus for cursor display
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FocusedInput {
    /// Main script list filter input
    MainFilter,
    /// Actions dialog search input
    ActionsSearch,
    /// Arg prompt input (when running a script)
    ArgPrompt,
    /// No input focused (e.g., terminal prompt)
    None,
}

/// Messages sent from the prompt poller back to the main app
#[derive(Debug, Clone)]
enum PromptMessage {
    ShowArg {
        id: String,
        placeholder: String,
        choices: Vec<Choice>,
        actions: Option<Vec<ProtocolAction>>,
    },
    ShowDiv {
        id: String,
        html: String,
        /// Tailwind classes for the content container
        container_classes: Option<String>,
        actions: Option<Vec<ProtocolAction>>,
        /// Placeholder text (header)
        placeholder: Option<String>,
        /// Hint text
        hint: Option<String>,
        /// Footer text
        footer: Option<String>,
        /// Container background color
        container_bg: Option<String>,
        /// Container padding (number or "none")
        container_padding: Option<serde_json::Value>,
        /// Container opacity (0-100)
        opacity: Option<u8>,
    },
    ShowForm {
        id: String,
        html: String,
        actions: Option<Vec<ProtocolAction>>,
    },
    ShowTerm {
        id: String,
        command: Option<String>,
        actions: Option<Vec<ProtocolAction>>,
    },
    ShowEditor {
        id: String,
        content: Option<String>,
        language: Option<String>,
        template: Option<String>,
        actions: Option<Vec<ProtocolAction>>,
    },
    /// Path picker prompt for file/folder selection
    ShowPath {
        id: String,
        start_path: Option<String>,
        hint: Option<String>,
    },
    /// Environment variable prompt with optional secret handling
    ShowEnv {
        id: String,
        key: String,
        prompt: Option<String>,
        secret: bool,
    },
    /// Drag and drop prompt for file uploads
    ShowDrop {
        id: String,
        placeholder: Option<String>,
        hint: Option<String>,
    },
    /// Template prompt for tab-through string templates
    ShowTemplate {
        id: String,
        template: String,
    },
    /// Multi-select prompt from choices
    ShowSelect {
        id: String,
        placeholder: Option<String>,
        choices: Vec<Choice>,
        multiple: bool,
    },
    HideWindow,
    OpenBrowser {
        url: String,
    },
    ScriptExit,
    /// External command to run a script by path
    RunScript {
        path: String,
    },
    /// Script error with detailed information for toast display
    ScriptError {
        error_message: String,
        stderr_output: Option<String>,
        exit_code: Option<i32>,
        stack_trace: Option<String>,
        script_path: String,
        suggestions: Vec<String>,
    },
    /// Protocol parsing error reported from script stdout
    ProtocolError {
        correlation_id: String,
        summary: String,
        details: Option<String>,
        severity: ErrorSeverity,
        script_path: String,
    },
    /// Unhandled message type from script - shows warning toast
    UnhandledMessage {
        message_type: String,
    },
    /// Request to get current UI state - triggers StateResult response
    GetState {
        request_id: String,
    },
    /// Request to get layout info with component tree and computed styles
    GetLayoutInfo {
        request_id: String,
    },
    /// Force submit the current prompt with a value (from SDK's submit() function)
    ForceSubmit {
        value: serde_json::Value,
    },
    /// Set the current prompt input text
    SetInput {
        text: String,
    },
    /// Show HUD overlay message
    ShowHud {
        text: String,
        duration_ms: Option<u64>,
    },
    /// Set SDK actions for the ActionsDialog
    SetActions {
        actions: Vec<protocol::ProtocolAction>,
    },
    /// Show the debug grid overlay
    ShowGrid {
        options: protocol::GridOptions,
    },
    /// Hide the debug grid overlay
    HideGrid,
}

struct ScriptListApp {
    /// H1 Optimization: Arc-wrapped scripts for cheap cloning during filter operations
    scripts: Vec<std::sync::Arc<scripts::Script>>,
    /// H1 Optimization: Arc-wrapped scriptlets for cheap cloning during filter operations
    scriptlets: Vec<std::sync::Arc<scripts::Scriptlet>>,
    builtin_entries: Vec<builtins::BuiltInEntry>,
    /// Cached list of installed applications for main search
    apps: Vec<app_launcher::AppInfo>,
    selected_index: usize,
    /// Main menu filter text (mirrors gpui-component input state)
    filter_text: String,
    /// gpui-component input state for the main filter
    gpui_input_state: Entity<InputState>,
    gpui_input_focused: bool,
    #[allow(dead_code)]
    gpui_input_subscriptions: Vec<Subscription>,
    /// Suppress handling of programmatic InputEvent::Change updates.
    suppress_filter_events: bool,
    /// Sync gpui input text on next render when window access is available.
    pending_filter_sync: bool,
    last_output: Option<SharedString>,
    focus_handle: FocusHandle,
    show_logs: bool,
    theme: theme::Theme,
    #[allow(dead_code)]
    config: config::Config,
    // Scroll activity tracking for scrollbar fade
    /// Whether scroll activity is happening (scrollbar should be visible)
    is_scrolling: bool,
    /// Timestamp of last scroll activity (for fade-out timer)
    last_scroll_time: Option<std::time::Instant>,
    // Interactive script state
    current_view: AppView,
    script_session: SharedSession,
    // Prompt-specific state (used when view is ArgPrompt or DivPrompt)
    // Uses TextInputState for selection and clipboard support
    arg_input: TextInputState,
    arg_selected_index: usize,
    // Channel for receiving prompt messages from script thread (async_channel for event-driven)
    prompt_receiver: Option<async_channel::Receiver<PromptMessage>>,
    // Channel for sending responses back to script
    response_sender: Option<mpsc::Sender<Message>>,
    // List state for variable-height list (supports section headers at 24px + items at 48px)
    main_list_state: ListState,
    // Scroll handle for uniform_list (still used for backward compat in some views)
    list_scroll_handle: UniformListScrollHandle,
    // P0: Scroll handle for virtualized arg prompt choices
    arg_list_scroll_handle: UniformListScrollHandle,
    // Scroll handle for clipboard history list
    clipboard_list_scroll_handle: UniformListScrollHandle,
    // Scroll handle for window switcher list
    window_list_scroll_handle: UniformListScrollHandle,
    // Scroll handle for design gallery list
    design_gallery_scroll_handle: UniformListScrollHandle,
    // Actions popup overlay
    show_actions_popup: bool,
    // ActionsDialog entity for focus management
    actions_dialog: Option<Entity<ActionsDialog>>,
    // Cursor blink state and focus tracking
    cursor_visible: bool,
    /// Which input currently has focus (for cursor display)
    focused_input: FocusedInput,
    // Current script process PID for explicit cleanup (belt-and-suspenders)
    current_script_pid: Option<u32>,
    // P1: Cache for filtered_results() - invalidate on filter_text change only
    cached_filtered_results: Vec<scripts::SearchResult>,
    filter_cache_key: String,
    // P1: Cache for get_grouped_results() - invalidate on filter_text change only
    // This avoids recomputing grouped results 9+ times per keystroke
    // P1-Arc: Use Arc<[T]> for cheap clone in render closures
    cached_grouped_items: Arc<[GroupedListItem]>,
    cached_grouped_flat_results: Arc<[scripts::SearchResult]>,
    grouped_cache_key: String,
    // P3: Two-stage filter - display vs search separation with coalescing
    /// What the search cache is built from (may lag behind filter_text during rapid typing)
    computed_filter_text: String,
    /// Coalesces filter updates and keeps only the latest value per tick
    filter_coalescer: FilterCoalescer,
    // Scroll stabilization: track last scrolled-to index to avoid redundant scroll_to_item calls
    last_scrolled_index: Option<usize>,
    // Preview cache: avoid re-reading file and re-highlighting on every render
    preview_cache_path: Option<String>,
    preview_cache_lines: Vec<syntax::HighlightedLine>,
    // Current design variant for hot-swappable UI designs
    current_design: DesignVariant,
    // Toast manager for notification queue
    toast_manager: ToastManager,
    // Cache for decoded clipboard images (entry_id -> RenderImage)
    clipboard_image_cache: std::collections::HashMap<String, Arc<gpui::RenderImage>>,
    // Frecency store for tracking script usage
    frecency_store: FrecencyStore,
    // Mouse hover tracking - independent from selected_index (keyboard focus)
    // hovered_index shows subtle visual feedback, selected_index shows full focus styling
    hovered_index: Option<usize>,
    // P0-2: Debounce hover notify calls (16ms window to reduce 50% unnecessary re-renders)
    last_hover_notify: std::time::Instant,
    // Pending path action - when set, show ActionsDialog for this path
    // Uses Arc<Mutex<>> so callbacks can write to it
    pending_path_action: Arc<Mutex<Option<PathInfo>>>,
    // Signal to close path actions dialog (set by callback on Escape/__cancel__)
    close_path_actions: Arc<Mutex<bool>>,
    // Shared state: whether path actions dialog is currently showing
    // Used by PathPrompt to implement toggle behavior for Cmd+K
    path_actions_showing: Arc<Mutex<bool>>,
    // Shared state: current search text in path actions dialog
    // Used by PathPrompt to display search in header (like main menu does)
    path_actions_search_text: Arc<Mutex<String>>,
    // Pending path action result - when set, execute this action on the stored path
    // Tuple of (action_id, path_info) to handle the action
    pending_path_action_result: Arc<Mutex<Option<(String, PathInfo)>>>,
    /// Alias registry: lowercase_alias -> script_path (for O(1) lookup)
    /// Conflict rule: first-registered wins
    alias_registry: std::collections::HashMap<String, String>,
    /// Shortcut registry: shortcut -> script_path (for O(1) lookup)
    /// Conflict rule: first-registered wins
    shortcut_registry: std::collections::HashMap<String, String>,
    /// SDK actions set via setActions() - stored for trigger_action_by_name lookup
    sdk_actions: Option<Vec<protocol::ProtocolAction>>,
    /// SDK action shortcuts: normalized_shortcut -> action_name (for O(1) lookup)
    action_shortcuts: std::collections::HashMap<String, String>,
    /// Debug grid overlay configuration (None = hidden)
    grid_config: Option<debug_grid::GridConfig>,
    // Navigation coalescing for rapid arrow key events (20ms window)
    nav_coalescer: NavCoalescer,
    // Window focus tracking - for detecting focus lost and auto-dismissing prompts
    // When window loses focus while in a dismissable prompt, close and reset
    was_window_focused: bool,
    // Show warning banner when bun is not available
    show_bun_warning: bool,
    // Pending confirmation: when set, the entry with this ID is awaiting confirmation
    // Used for dangerous actions like Shut Down, Restart, Log Out, Empty Trash
    pending_confirmation: Option<String>,
    // Scroll stabilization: track last scrolled-to index for each scroll handle
    #[allow(dead_code)]
    last_scrolled_main: Option<usize>,
    #[allow(dead_code)]
    last_scrolled_arg: Option<usize>,
    #[allow(dead_code)]
    last_scrolled_clipboard: Option<usize>,
    #[allow(dead_code)]
    last_scrolled_window: Option<usize>,
    #[allow(dead_code)]
    last_scrolled_design_gallery: Option<usize>,
    // Menu bar integration: Now handled by frontmost_app_tracker module
    // which pre-fetches menu items in background when apps activate
}

/// Result of alias matching - either a Script or Scriptlet
#[derive(Clone, Debug)]
enum AliasMatch {
    Script(Arc<scripts::Script>),
    Scriptlet(Arc<scripts::Scriptlet>),
}

// Core ScriptListApp implementation extracted to app_impl.rs
include!("app_impl.rs");

// Script execution logic (execute_interactive) extracted
include!("execute_script.rs");

// Prompt message handling (handle_prompt_message) extracted
include!("prompt_handler.rs");

// App navigation methods (selection movement, scrolling)
include!("app_navigation.rs");

// App execution methods (execute_builtin, execute_app, execute_window_focus)
include!("app_execute.rs");

// App actions handling (handle_action, trigger_action_by_name)
include!("app_actions.rs");

// Layout calculation methods (build_component_bounds, build_layout_info)
include!("app_layout.rs");

impl Focusable for ScriptListApp {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for ScriptListApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Flush any pending toasts to gpui-component's NotificationList
        // This is needed because toast push sites don't have window access
        self.flush_pending_toasts(window, cx);

        // Focus-lost auto-dismiss: Close dismissable prompts when the main window loses focus
        // This includes focus loss to other app windows like Notes/AI.
        let is_window_focused = platform::is_main_window_focused();
        if self.was_window_focused && !is_window_focused {
            // Window just lost focus (user clicked another window)
            // Only auto-dismiss if we're in a dismissable view AND window is visible
            if self.is_dismissable_view() && script_kit_gpui::is_main_window_visible() {
                logging::log(
                    "FOCUS",
                    "Main window lost focus while in dismissable view - closing",
                );
                self.close_and_reset_window(cx);
            }
        }
        self.was_window_focused = is_window_focused;

        // P0-4: Focus handling using reference match (avoids clone for focus check)
        // Focus handling depends on the view:
        // - For EditorPrompt: Use its own focus handle (not the parent's)
        // - For other views: Use the parent's focus handle
        //
        // Only enforce focus when the main window is currently focused.
        if is_window_focused {
            match &self.current_view {
                AppView::EditorPrompt { entity, .. } => {
                    // EditorPrompt uses gpui-component's Input which has its own internal
                    // focus handle. But if actions dialog is showing, focus the dialog instead.
                    if self.show_actions_popup {
                        if let Some(ref dialog) = self.actions_dialog {
                            let dialog_focus_handle = dialog.read(cx).focus_handle.clone();
                            let is_focused = dialog_focus_handle.is_focused(window);
                            if !is_focused {
                                window.focus(&dialog_focus_handle, cx);
                            }
                        }
                    } else {
                        entity.update(cx, |editor, cx| {
                            editor.focus(window, cx);
                        });
                    }
                }
                AppView::PathPrompt { focus_handle, .. } => {
                    // PathPrompt has its own focus handle - focus it
                    // But if actions dialog is showing, focus the dialog instead
                    if self.show_actions_popup {
                        if let Some(ref dialog) = self.actions_dialog {
                            let dialog_focus_handle = dialog.read(cx).focus_handle.clone();
                            let is_focused = dialog_focus_handle.is_focused(window);
                            if !is_focused {
                                window.focus(&dialog_focus_handle, cx);
                            }
                        }
                    } else {
                        let is_focused = focus_handle.is_focused(window);
                        if !is_focused {
                            let fh = focus_handle.clone();
                            window.focus(&fh, cx);
                        }
                    }
                }
                AppView::FormPrompt { entity, .. } => {
                    // FormPrompt uses delegated Focusable - get focus handle from the currently focused field
                    // This prevents the parent from stealing focus from form text fields
                    let form_focus_handle = entity.read(cx).focus_handle(cx);
                    let is_focused = form_focus_handle.is_focused(window);
                    if !is_focused {
                        window.focus(&form_focus_handle, cx);
                    }
                }
                AppView::ScriptList => {
                    self.sync_filter_input_if_needed(window, cx);

                    if self.show_actions_popup {
                        if let Some(ref dialog) = self.actions_dialog {
                            let dialog_focus_handle = dialog.read(cx).focus_handle.clone();
                            let is_focused = dialog_focus_handle.is_focused(window);
                            if !is_focused {
                                window.focus(&dialog_focus_handle, cx);
                            }
                        }
                    } else {
                        let input_state = self.gpui_input_state.clone();
                        let is_focused = input_state.read(cx).focus_handle(cx).is_focused(window);
                        if !is_focused {
                            input_state.update(cx, |state, cx| {
                                state.focus(window, cx);
                            });
                        }
                    }
                }
                _ => {
                    // Other views use the parent's focus handle
                    let is_focused = self.focus_handle.is_focused(window);
                    if !is_focused {
                        window.focus(&self.focus_handle, cx);
                    }
                }
            }
        }

        // NOTE: Prompt messages are now handled via event-driven async_channel listener
        // spawned in execute_interactive() - no polling needed in render()

        // P0-4: Clone current_view only for dispatch (needed to call &mut self methods)
        // The clone is unavoidable due to borrow checker: we need &mut self for render methods
        // but also need to match on self.current_view. Future optimization: refactor render
        // methods to take &str/&[T] references instead of owned values.
        //
        // HUD is now handled by hud_manager as a separate floating window
        // No need to render it as part of this view
        let current_view = self.current_view.clone();
        let main_content: AnyElement = match current_view {
            AppView::ScriptList => self.render_script_list(cx).into_any_element(),
            AppView::ActionsDialog => self.render_actions_dialog(cx),
            AppView::ArgPrompt {
                id,
                placeholder,
                choices,
                actions,
            } => self
                .render_arg_prompt(id, placeholder, choices, actions, cx)
                .into_any_element(),
            AppView::DivPrompt { entity, .. } => {
                self.render_div_prompt(entity, cx).into_any_element()
            }
            AppView::FormPrompt { entity, .. } => {
                self.render_form_prompt(entity, cx).into_any_element()
            }
            AppView::TermPrompt { entity, .. } => {
                self.render_term_prompt(entity, cx).into_any_element()
            }
            AppView::EditorPrompt { entity, .. } => {
                self.render_editor_prompt(entity, cx).into_any_element()
            }
            AppView::SelectPrompt { entity, .. } => {
                self.render_select_prompt(entity, cx).into_any_element()
            }
            AppView::PathPrompt { entity, .. } => {
                self.render_path_prompt(entity, cx).into_any_element()
            }
            AppView::EnvPrompt { entity, .. } => {
                self.render_env_prompt(entity, cx).into_any_element()
            }
            AppView::DropPrompt { entity, .. } => {
                self.render_drop_prompt(entity, cx).into_any_element()
            }
            AppView::TemplatePrompt { entity, .. } => {
                self.render_template_prompt(entity, cx).into_any_element()
            }
            AppView::ClipboardHistoryView {
                entries,
                filter,
                selected_index,
            } => self
                .render_clipboard_history(entries, filter, selected_index, cx)
                .into_any_element(),
            AppView::AppLauncherView {
                apps,
                filter,
                selected_index,
            } => self
                .render_app_launcher(apps, filter, selected_index, cx)
                .into_any_element(),
            AppView::WindowSwitcherView {
                windows,
                filter,
                selected_index,
            } => self
                .render_window_switcher(windows, filter, selected_index, cx)
                .into_any_element(),
            AppView::DesignGalleryView {
                filter,
                selected_index,
            } => self
                .render_design_gallery(filter, selected_index, cx)
                .into_any_element(),
        };

        // Wrap content in a container that can have the debug grid overlay
        let window_bounds = window.bounds();
        let window_size = gpui::size(window_bounds.size.width, window_bounds.size.height);

        // Clone grid_config for use in the closure
        let grid_config = self.grid_config.clone();

        // Build component bounds for the current view (for debug overlay)
        let component_bounds = self.build_component_bounds(window_size);

        // Build warning banner if needed (bun not available)
        let warning_banner = if self.show_bun_warning {
            let banner_colors = WarningBannerColors::from_theme(&self.theme);
            let entity = cx.entity().downgrade();
            let entity_for_dismiss = entity.clone();

            Some(
                div().w_full().px(px(12.)).pt(px(8.)).child(
                    WarningBanner::new(
                        "bun is not installed. Click to download from bun.sh",
                        banner_colors,
                    )
                    .on_click(Box::new(move |_event, _window, cx| {
                        if let Some(app) = entity.upgrade() {
                            app.update(cx, |this, _cx| {
                                this.open_bun_website();
                            });
                        }
                    }))
                    .on_dismiss(Box::new(move |_event, _window, cx| {
                        if let Some(app) = entity_for_dismiss.upgrade() {
                            app.update(cx, |this, cx| {
                                this.dismiss_bun_warning(cx);
                            });
                        }
                    })),
                ),
            )
        } else {
            None
        };

        div()
            .w_full()
            .h_full()
            .relative()
            .flex()
            .flex_col()
            // Warning banner appears at the top when bun is not available
            .when_some(warning_banner, |container, banner| container.child(banner))
            // Main content takes remaining space
            .child(div().flex_1().w_full().min_h(px(0.)).child(main_content))
            .when_some(grid_config, |container, config| {
                let overlay_bounds = gpui::Bounds {
                    origin: gpui::point(px(0.), px(0.)),
                    size: window_size,
                };
                container.child(debug_grid::render_grid_overlay(
                    &config,
                    overlay_bounds,
                    &component_bounds,
                ))
            })
    }
}

// Render methods extracted to app_render.rs for maintainability
include!("app_render.rs");

// Builtin view render methods (clipboard, app launcher, window switcher)
include!("render_builtins.rs");

// Prompt render methods - split into separate files for maintainability
// Each file adds render_*_prompt methods to ScriptListApp via impl blocks
include!("render_prompts/arg.rs");
include!("render_prompts/div.rs");
include!("render_prompts/form.rs");
include!("render_prompts/term.rs");
include!("render_prompts/editor.rs");
include!("render_prompts/path.rs");
include!("render_prompts/other.rs");

// Script list render method
include!("render_script_list.rs");

fn main() {
    logging::init();

    // Migrate from legacy ~/.kenv to new ~/.sk/kit structure (one-time migration)
    // This must happen BEFORE ensure_kit_setup() so the new path is used
    if setup::migrate_from_kenv() {
        logging::log("APP", "Migrated from ~/.kenv to ~/.sk/kit");
    }

    // Ensure ~/.sk/kit environment is properly set up (directories, SDK, config, etc.)
    // This is idempotent - it creates missing directories and files without overwriting user configs
    let setup_result = setup::ensure_kit_setup();
    if setup_result.is_fresh_install {
        logging::log(
            "APP",
            &format!(
                "Fresh install detected - created ~/.sk/kit at {}",
                setup_result.kit_path.display()
            ),
        );
    }
    for warning in &setup_result.warnings {
        logging::log("APP", &format!("Setup warning: {}", warning));
    }
    if !setup_result.bun_available {
        logging::log(
            "APP",
            "Warning: bun not found in PATH or common locations. Scripts may not run.",
        );
    }

    // Write main PID file for orphan detection on crash
    if let Err(e) = PROCESS_MANAGER.write_main_pid() {
        logging::log("APP", &format!("Failed to write main PID file: {}", e));
    } else {
        logging::log("APP", "Main PID file written");
    }

    // Clean up any orphaned processes from a previous crash
    let orphans_killed = PROCESS_MANAGER.cleanup_orphans();
    if orphans_killed > 0 {
        logging::log(
            "APP",
            &format!(
                "Cleaned up {} orphaned process(es) from previous session",
                orphans_killed
            ),
        );
    }

    // Register signal handlers for graceful shutdown
    // Using libc directly since ctrlc crate is not available
    #[cfg(unix)]
    {
        extern "C" fn handle_signal(sig: libc::c_int) {
            logging::log("SIGNAL", &format!("Received signal {}", sig));

            // Set shutdown flag to prevent new script spawns
            SHUTDOWN_REQUESTED.store(true, Ordering::SeqCst);

            // Kill all tracked child processes
            logging::log("SIGNAL", "Killing all child processes");
            PROCESS_MANAGER.kill_all_processes();

            // Remove main PID file
            PROCESS_MANAGER.remove_main_pid();

            // For SIGINT/SIGTERM, exit gracefully
            // Note: We can't call cx.quit() from here since we're in a signal handler
            // The process will terminate after killing children
            logging::log("SIGNAL", "Exiting after signal cleanup");
            std::process::exit(0);
        }

        unsafe {
            // Register handlers for common termination signals
            libc::signal(libc::SIGINT, handle_signal as libc::sighandler_t);
            libc::signal(libc::SIGTERM, handle_signal as libc::sighandler_t);
            libc::signal(libc::SIGHUP, handle_signal as libc::sighandler_t);
            logging::log(
                "APP",
                "Signal handlers registered (SIGINT, SIGTERM, SIGHUP)",
            );
        }
    }

    // Load config early so we can use it for hotkey registration AND clipboard history settings
    // This avoids duplicate config::load_config() calls (~100-300ms startup savings)
    let loaded_config = config::load_config();
    logging::log(
        "APP",
        &format!(
            "Loaded config: hotkey={:?}+{}, bun_path={:?}",
            loaded_config.hotkey.modifiers, loaded_config.hotkey.key, loaded_config.bun_path
        ),
    );
    clipboard_history::set_max_text_content_len(
        loaded_config.get_clipboard_history_max_text_length(),
    );

    // Initialize clipboard history monitoring (background thread)
    if let Err(e) = clipboard_history::init_clipboard_history() {
        logging::log(
            "APP",
            &format!("Failed to initialize clipboard history: {}", e),
        );
    } else {
        logging::log("APP", "Clipboard history monitoring initialized");
    }

    // Initialize text expansion system (background thread with keyboard monitoring)
    // This must be done early, before the GPUI run loop starts
    #[cfg(target_os = "macos")]
    {
        use expand_manager::ExpandManager;

        // Spawn initialization in a thread to not block startup
        std::thread::spawn(move || {
            logging::log("EXPAND", "Initializing text expansion system");

            // Check accessibility permissions first
            if !ExpandManager::has_accessibility_permission() {
                logging::log(
                    "EXPAND",
                    "Accessibility permissions not granted - text expansion disabled",
                );
                logging::log(
                    "EXPAND",
                    "Enable in System Preferences > Privacy & Security > Accessibility",
                );
                return;
            }

            let mut manager = ExpandManager::new();

            // Load scriptlets with expand triggers
            match manager.load_scriptlets() {
                Ok(count) => {
                    if count == 0 {
                        logging::log("EXPAND", "No expand triggers found in scriptlets");
                        return;
                    }
                    logging::log("EXPAND", &format!("Loaded {} expand triggers", count));
                }
                Err(e) => {
                    logging::log("EXPAND", &format!("Failed to load scriptlets: {}", e));
                    return;
                }
            }

            // Enable keyboard monitoring
            match manager.enable() {
                Ok(()) => {
                    logging::log("EXPAND", "Text expansion system enabled");

                    // List registered triggers
                    for (trigger, name) in manager.list_triggers() {
                        logging::log("EXPAND", &format!("  Trigger '{}' -> {}", trigger, name));
                    }

                    // Keep the manager alive - it will run until the process exits
                    // The keyboard monitor thread is managed by the KeyboardMonitor
                    std::mem::forget(manager);
                }
                Err(e) => {
                    logging::log(
                        "EXPAND",
                        &format!("Failed to enable text expansion: {:?}", e),
                    );
                }
            }
        });
    }

    // Clone before start_hotkey_listener consumes original
    let config_for_app = loaded_config.clone();

    // Start MCP server for AI agent integration
    // Server runs on localhost:43210 with Bearer token authentication
    // Discovery file written to ~/.sk/kit/server.json
    let _mcp_handle = match mcp_server::McpServer::with_defaults() {
        Ok(server) => match server.start() {
            Ok(handle) => {
                logging::log(
                    "MCP",
                    &format!(
                        "MCP server started on {} (token in ~/.sk/kit/agent-token)",
                        server.url()
                    ),
                );
                Some(handle)
            }
            Err(e) => {
                logging::log("MCP", &format!("Failed to start MCP server: {}", e));
                None
            }
        },
        Err(e) => {
            logging::log("MCP", &format!("Failed to create MCP server: {}", e));
            None
        }
    };

    hotkeys::start_hotkey_listener(loaded_config);

    let (mut appearance_watcher, appearance_rx) = watcher::AppearanceWatcher::new();
    if let Err(e) = appearance_watcher.start() {
        logging::log("APP", &format!("Failed to start appearance watcher: {}", e));
    }

    let (mut config_watcher, config_rx) = watcher::ConfigWatcher::new();
    if let Err(e) = config_watcher.start() {
        logging::log("APP", &format!("Failed to start config watcher: {}", e));
    }

    let (mut script_watcher, script_rx) = watcher::ScriptWatcher::new();
    if let Err(e) = script_watcher.start() {
        logging::log("APP", &format!("Failed to start script watcher: {}", e));
    }

    // Initialize script scheduler
    // Creates the scheduler and scans for scripts with // Cron: or // Schedule: metadata
    let (mut scheduler, scheduler_rx) = scheduler::Scheduler::new();
    let scheduled_count = scripts::register_scheduled_scripts(&scheduler);
    logging::log(
        "APP",
        &format!("Registered {} scheduled scripts", scheduled_count),
    );

    // Start the scheduler background thread (checks every 30 seconds for due scripts)
    if scheduled_count > 0 {
        if let Err(e) = scheduler.start() {
            logging::log("APP", &format!("Failed to start scheduler: {}", e));
        } else {
            logging::log("APP", "Scheduler started successfully");
        }
    } else {
        logging::log("APP", "No scheduled scripts found, scheduler not started");
    }

    // Wrap scheduler in Arc<Mutex<>> for thread-safe access (needed for re-scanning on file changes)
    let scheduler = Arc::new(Mutex::new(scheduler));

    Application::new().run(move |cx: &mut App| {
        logging::log("APP", "GPUI Application starting");

        // Configure as accessory app FIRST, before any windows are created
        // This is equivalent to LSUIElement=true in Info.plist:
        // - No Dock icon
        // - No menu bar ownership (critical for window actions to work)
        platform::configure_as_accessory_app();

        // Start frontmost app tracker - watches for app activations and pre-fetches menu bar items
        // Must be started after configure_as_accessory_app() so we're correctly classified
        #[cfg(target_os = "macos")]
        frontmost_app_tracker::start_tracking();

        // Register bundled JetBrains Mono font
        // This makes "JetBrains Mono" available as a font family for the editor
        register_bundled_fonts(cx);

        // Initialize gpui-component (theme, context providers)
        // Must be called before opening windows that use Root wrapper
        gpui_component::init(cx);

        // Sync Script Kit theme with gpui-component's ThemeColor system
        // This ensures all gpui-component widgets use our colors
        theme::sync_gpui_component_theme(cx);

        // Initialize tray icon and menu
        // MUST be done after Application::new() creates the NSApplication
        let tray_manager = match TrayManager::new() {
            Ok(tm) => {
                logging::log("TRAY", "Tray icon initialized successfully");
                Some(tm)
            }
            Err(e) => {
                logging::log("TRAY", &format!("Failed to initialize tray icon: {}", e));
                None
            }
        };

        // Calculate window bounds: centered on display with mouse, at eye-line height
        let window_size = size(px(750.), initial_window_height());
        let bounds = calculate_eye_line_bounds_on_mouse_display(window_size);

        // Load theme to determine window background appearance (vibrancy)
        let initial_theme = theme::load_theme();
        let window_background = if initial_theme.is_vibrancy_enabled() {
            WindowBackgroundAppearance::Blurred
        } else {
            WindowBackgroundAppearance::Opaque
        };
        logging::log(
            "THEME",
            &format!(
                "Window background appearance: {:?} (vibrancy_enabled={})",
                window_background,
                initial_theme.is_vibrancy_enabled()
            ),
        );

        // Store the ScriptListApp entity for direct access (needed since Root wraps the view)
        let app_entity_holder: Arc<Mutex<Option<Entity<ScriptListApp>>>> = Arc::new(Mutex::new(None));
        let app_entity_for_closure = app_entity_holder.clone();

        // Capture bun_available for use in window creation
        let bun_available = setup_result.bun_available;

        let window: WindowHandle<Root> = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: None,
                is_movable: true,
                window_background,
                show: false, // Start hidden - only show on hotkey press
                focus: false, // Don't focus on creation
                ..Default::default()
            },
            |window, cx| {
                logging::log("APP", "Window opened, creating ScriptListApp wrapped in Root");
                let view = cx.new(|cx| ScriptListApp::new(config_for_app, bun_available, window, cx));
                // Store the entity for external access
                *app_entity_for_closure.lock().unwrap() = Some(view.clone());
                cx.new(|cx| Root::new(view, window, cx))
            },
        )
        .unwrap();

        // Extract the app entity for use in callbacks
        let app_entity = app_entity_holder.lock().unwrap().clone().expect("App entity should be set");

        // Set initial focus via the Root window
        // We access the app entity within the window context to properly focus it
        let app_entity_for_focus = app_entity.clone();
        window
            .update(cx, |_root, win, root_cx| {
                app_entity_for_focus.update(root_cx, |view, ctx| {
                    let focus_handle = view.focus_handle(ctx);
                    win.focus(&focus_handle, ctx);
                    logging::log("APP", "Focus set on ScriptListApp via Root");
                });
            })
            .unwrap();

        // Register the main window with WindowManager before tray init
        // This must happen after GPUI creates the window but before tray creates its windows
        // so we can reliably find our main window by its expected size (~750x500)
        window_manager::find_and_register_main_window();

        // Window starts hidden - no activation, no panel configuration yet
        // Panel will be configured on first show via hotkey
        // WINDOW_VISIBLE is already false by default (static initializer)
        logging::log("HOTKEY", "Window created but not shown (use hotkey to show)");

        // Main window hotkey listener - uses Entity<ScriptListApp> instead of WindowHandle
        let app_entity_for_hotkey = app_entity.clone();
        let window_for_hotkey = window;
        cx.spawn(async move |cx: &mut gpui::AsyncApp| {
            logging::log("HOTKEY", "Main hotkey listener started");
            while let Ok(()) = hotkeys::hotkey_channel().1.recv().await {
                logging::log("VISIBILITY", "");
                logging::log("VISIBILITY", "╔════════════════════════════════════════════════════════════╗");
                logging::log("VISIBILITY", "║  HOTKEY TRIGGERED - TOGGLE WINDOW                          ║");
                logging::log("VISIBILITY", "╚════════════════════════════════════════════════════════════╝");

                let is_visible = script_kit_gpui::is_main_window_visible();
                logging::log("VISIBILITY", &format!("State: WINDOW_VISIBLE={}", is_visible));

                let app_entity_inner = app_entity_for_hotkey.clone();
                let window_inner = window_for_hotkey;

                if is_visible {
                    logging::log("VISIBILITY", "Decision: HIDE");
                    script_kit_gpui::set_main_window_visible(false);

                    // Check if Notes or AI windows are open BEFORE the closure
                    let notes_open = notes::is_notes_window_open();
                    let ai_open = ai::is_ai_window_open();
                    logging::log(
                        "VISIBILITY",
                        &format!(
                            "Secondary windows: notes_open={}, ai_open={}",
                            notes_open, ai_open
                        ),
                    );

                    let _ = cx.update(move |cx: &mut gpui::App| {
                        // Cancel any active prompt and reset UI
                        app_entity_inner.update(cx, |view, ctx| {
                            if view.is_in_prompt() {
                                logging::log("HOTKEY", "Canceling prompt before hiding");
                                view.cancel_script_execution(ctx);
                            }
                            view.reset_to_script_list(ctx);
                        });

                        // CRITICAL: Only hide main window if Notes/AI are open
                        // cx.hide() hides the ENTIRE app (all windows), so we use
                        // platform::hide_main_window() to hide only the main window
                        if notes_open || ai_open {
                            logging::log(
                                "HOTKEY",
                                "Using hide_main_window() - secondary windows are open",
                            );
                            platform::hide_main_window();
                        } else {
                            logging::log("HOTKEY", "Using cx.hide() - no secondary windows");
                            cx.hide();
                        }
                        logging::log("HOTKEY", "Main window hidden");
                    });
                } else {
                    logging::log("VISIBILITY", "Decision: SHOW");

                    script_kit_gpui::set_main_window_visible(true);

                    let _ = cx.update(move |cx: &mut gpui::App| {
                        // Position window on mouse display at eye-line
                        platform::ensure_move_to_active_space();

                        let window_size = gpui::size(px(750.), initial_window_height());
                        let bounds = platform::calculate_eye_line_bounds_on_mouse_display(window_size);
                        platform::move_first_window_to_bounds(&bounds);

                        // Configure as floating panel on first show
                        if !PANEL_CONFIGURED.load(std::sync::atomic::Ordering::SeqCst) {
                            platform::configure_as_floating_panel();
                            PANEL_CONFIGURED.store(true, std::sync::atomic::Ordering::SeqCst);
                        }

                        // Activate window and focus input
                        cx.activate(true);
                        let _ = window_inner.update(cx, |_root, window, _cx| {
                            window.activate_window();
                        });

                        // Focus the input and check for any missed reset
                        // (reset should happen on hide, but this is a safety net)
                        app_entity_inner.update(cx, |view, ctx| {
                            let focus_handle = view.focus_handle(ctx);
                            window_inner.update(ctx, |_root, window, _cx| {
                                window.focus(&focus_handle, _cx);
                            }).ok();

                            // Safety net: if NEEDS_RESET is still true, reset now
                            if NEEDS_RESET.compare_exchange(
                                true,
                                false,
                                Ordering::SeqCst,
                                Ordering::SeqCst,
                            ).is_ok() {
                                logging::log(
                                    "VISIBILITY",
                                    "NEEDS_RESET was true (safety net) - resetting to script list",
                                );
                                view.reset_to_script_list(ctx);
                            }
                        });

                        logging::log("HOTKEY", "Window shown and activated");
                    });
                }
            }
            logging::log("HOTKEY", "Main hotkey listener exiting");
        }).detach();

        // Notes hotkey listener - event-driven via async_channel
        // The hotkey thread dispatches via GPUI's ForegroundExecutor, which wakes this task
        // This works even before main window activates because the executor is initialized first
        cx.spawn(async move |cx: &mut gpui::AsyncApp| {
            logging::log("HOTKEY", "Notes hotkey listener started (event-driven)");
            // Event-driven: .recv().await blocks until a message arrives
            // This is more efficient than polling and responds immediately
            while let Ok(()) = hotkeys::notes_hotkey_channel().1.recv().await {
                logging::log("HOTKEY", "Notes hotkey triggered - opening notes window");
                let _ = cx.update(|cx: &mut gpui::App| {
                    if let Err(e) = notes::open_notes_window(cx) {
                        logging::log("HOTKEY", &format!("Failed to open notes window: {}", e));
                    }
                });
            }
            logging::log("HOTKEY", "Notes hotkey listener exiting (channel closed)");
        }).detach();

        // AI hotkey listener - event-driven via async_channel
        // Same pattern as Notes hotkey - works immediately on app launch
        cx.spawn(async move |cx: &mut gpui::AsyncApp| {
            logging::log("HOTKEY", "AI hotkey listener started (event-driven)");
            // Event-driven: .recv().await blocks until a message arrives
            while let Ok(()) = hotkeys::ai_hotkey_channel().1.recv().await {
                logging::log("HOTKEY", "AI hotkey triggered - opening AI window");
                let _ = cx.update(|cx: &mut gpui::App| {
                    if let Err(e) = ai::open_ai_window(cx) {
                        logging::log("HOTKEY", &format!("Failed to open AI window: {}", e));
                    }
                });
            }
            logging::log("HOTKEY", "AI hotkey listener exiting (channel closed)");
        }).detach();

        // Appearance change watcher - event-driven with async_channel
        let app_entity_for_appearance = app_entity.clone();
        cx.spawn(async move |cx: &mut gpui::AsyncApp| {
            // Event-driven: blocks until appearance change event received
            while let Ok(_event) = appearance_rx.recv().await {
                logging::log("APP", "System appearance changed, updating theme");
                let _ = cx.update(|cx| {
                    // Sync gpui-component theme with new system appearance
                    theme::sync_gpui_component_theme(cx);

                    app_entity_for_appearance.update(cx, |view, ctx| {
                        view.update_theme(ctx);
                    });
                });
            }
            logging::log("APP", "Appearance watcher channel closed");
        }).detach();

        // Config reload watcher - watches ~/.sk/kit/config.ts for changes
        let app_entity_for_config = app_entity.clone();
        cx.spawn(async move |cx: &mut gpui::AsyncApp| {
            loop {
                Timer::after(std::time::Duration::from_millis(200)).await;

                if config_rx.try_recv().is_ok() {
                    logging::log("APP", "Config file changed, reloading");
                    let _ = cx.update(|cx| {
                        app_entity_for_config.update(cx, |view, ctx| {
                            view.update_config(ctx);
                        });
                    });
                }
            }
        }).detach();

        // Script/scriptlets reload watcher - watches ~/.sk/kit/*/scripts/ and ~/.sk/kit/*/scriptlets/
        // Uses incremental updates for scriptlet files, full reload for scripts
        // Also re-scans for scheduled scripts to pick up new/modified schedules
        let app_entity_for_scripts = app_entity.clone();
        let scheduler_for_scripts = scheduler.clone();
        cx.spawn(async move |cx: &mut gpui::AsyncApp| {
            use watcher::ScriptReloadEvent;

            loop {
                Timer::after(std::time::Duration::from_millis(200)).await;

                // Drain all pending events
                while let Ok(event) = script_rx.try_recv() {
                    match event {
                        ScriptReloadEvent::FileChanged(path) | ScriptReloadEvent::FileCreated(path) => {
                            // Check if it's a scriptlet file (markdown in scriptlets directory)
                            let is_scriptlet = path.extension().map(|e| e == "md").unwrap_or(false);

                            if is_scriptlet {
                                logging::log("APP", &format!("Scriptlet file changed: {}", path.display()));
                                let path_clone = path.clone();
                                let _ = cx.update(|cx| {
                                    app_entity_for_scripts.update(cx, |view, ctx| {
                                        view.handle_scriptlet_file_change(&path_clone, false, ctx);
                                    });
                                });
                            } else {
                                logging::log("APP", &format!("Script file changed: {}", path.display()));
                                // Re-scan for scheduled scripts when script files change
                                if let Ok(scheduler_guard) = scheduler_for_scripts.lock() {
                                    let new_count = scripts::register_scheduled_scripts(&scheduler_guard);
                                    if new_count > 0 {
                                        logging::log("APP", &format!("Re-registered {} scheduled scripts after file change", new_count));
                                    }
                                }
                                let _ = cx.update(|cx| {
                                    app_entity_for_scripts.update(cx, |view, ctx| {
                                        view.refresh_scripts(ctx);
                                    });
                                });
                            }
                        }
                        ScriptReloadEvent::FileDeleted(path) => {
                            let is_scriptlet = path.extension().map(|e| e == "md").unwrap_or(false);

                            if is_scriptlet {
                                logging::log("APP", &format!("Scriptlet file deleted: {}", path.display()));
                                let path_clone = path.clone();
                                let _ = cx.update(|cx| {
                                    app_entity_for_scripts.update(cx, |view, ctx| {
                                        view.handle_scriptlet_file_change(&path_clone, true, ctx);
                                    });
                                });
                            } else {
                                logging::log("APP", &format!("Script file deleted: {}", path.display()));
                                let _ = cx.update(|cx| {
                                    app_entity_for_scripts.update(cx, |view, ctx| {
                                        view.refresh_scripts(ctx);
                                    });
                                });
                            }
                        }
                        ScriptReloadEvent::FullReload => {
                            logging::log("APP", "Full script/scriptlet reload requested");
                            // Re-scan for scheduled scripts
                            if let Ok(scheduler_guard) = scheduler_for_scripts.lock() {
                                let new_count = scripts::register_scheduled_scripts(&scheduler_guard);
                                if new_count > 0 {
                                    logging::log("APP", &format!("Re-registered {} scheduled scripts after full reload", new_count));
                                }
                            }
                            let _ = cx.update(|cx| {
                                app_entity_for_scripts.update(cx, |view, ctx| {
                                    view.refresh_scripts(ctx);
                                });
                            });
                        }
                    }
                }
            }
        }).detach();

        // NOTE: Prompt message listener is now spawned per-script in execute_interactive()
        // using event-driven async_channel instead of 50ms polling

        // Scheduler event handler - runs scripts when their cron schedule triggers
        // Uses std::sync::mpsc::Receiver which requires a polling approach
        let _window_for_scheduler = window;
        std::thread::spawn(move || {
            logging::log("APP", "Scheduler event handler started");

            loop {
                // Use recv_timeout to periodically check for events without blocking forever
                match scheduler_rx.recv_timeout(std::time::Duration::from_secs(1)) {
                    Ok(event) => {
                        match event {
                            scheduler::SchedulerEvent::RunScript(path) => {
                                logging::log("SCHEDULER", &format!("Executing scheduled script: {}", path.display()));

                                // Execute the script using the existing executor infrastructure
                                // This spawns it in the background without blocking the scheduler
                                let path_str = path.to_string_lossy().to_string();

                                // Use bun to run the script directly (non-interactive for scheduled scripts)
                                // Find bun path (same logic as executor)
                                let bun_path = std::env::var("BUN_PATH")
                                    .ok()
                                    .or_else(|| {
                                        // Check common locations
                                        for candidate in &[
                                            "/opt/homebrew/bin/bun",
                                            "/usr/local/bin/bun",
                                            std::env::var("HOME").ok().map(|h| format!("{}/.bun/bin/bun", h)).unwrap_or_default().as_str(),
                                        ] {
                                            if std::path::Path::new(candidate).exists() {
                                                return Some(candidate.to_string());
                                            }
                                        }
                                        None
                                    })
                                    .unwrap_or_else(|| "bun".to_string());

                                // Spawn bun process to run the script
                                match std::process::Command::new(&bun_path)
                                    .arg("run")
                                    .arg("--preload")
                                    .arg(format!("{}/.sk/kit/sdk/kit-sdk.ts", std::env::var("HOME").unwrap_or_default()))
                                    .arg(&path_str)
                                    .stdout(std::process::Stdio::piped())
                                    .stderr(std::process::Stdio::piped())
                                    .spawn()
                                {
                                    Ok(child) => {
                                        let pid = child.id();
                                        // Track the process
                                        PROCESS_MANAGER.register_process(pid, &path_str);
                                        logging::log("SCHEDULER", &format!("Spawned scheduled script PID {}: {}", pid, path_str));

                                        // Wait for completion in a separate thread to not block scheduler
                                        let path_for_log = path_str.clone();
                                        std::thread::spawn(move || {
                                            match child.wait_with_output() {
                                                Ok(output) => {
                                                    // Unregister the process now that it's done
                                                    PROCESS_MANAGER.unregister_process(pid);

                                                    if output.status.success() {
                                                        logging::log("SCHEDULER", &format!("Scheduled script completed: {}", path_for_log));
                                                    } else {
                                                        let stderr = String::from_utf8_lossy(&output.stderr);
                                                        logging::log("SCHEDULER", &format!("Scheduled script failed: {} - {}", path_for_log, stderr));
                                                    }
                                                }
                                                Err(e) => {
                                                    // Unregister on error too
                                                    PROCESS_MANAGER.unregister_process(pid);
                                                    logging::log("SCHEDULER", &format!("Scheduled script error: {} - {}", path_for_log, e));
                                                }
                                            }
                                        });
                                    }
                                    Err(e) => {
                                        logging::log("SCHEDULER", &format!("Failed to spawn scheduled script: {} - {}", path_str, e));
                                    }
                                }
                            }
                            scheduler::SchedulerEvent::Error(msg) => {
                                logging::log("SCHEDULER", &format!("Scheduler error: {}", msg));
                            }
                        }
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        // Normal timeout, continue loop
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                        logging::log("APP", "Scheduler event channel disconnected, exiting handler");
                        break;
                    }
                }
            }
        });

        // Test command file watcher - allows smoke tests to trigger script execution
        let app_entity_for_test = app_entity.clone();
        cx.spawn(async move |cx: &mut gpui::AsyncApp| {
            let cmd_file = std::path::PathBuf::from("/tmp/script-kit-gpui-cmd.txt");
            loop {
                Timer::after(std::time::Duration::from_millis(500)).await;

                if cmd_file.exists() {
                    if let Ok(content) = std::fs::read_to_string(&cmd_file) {
                        let _ = std::fs::remove_file(&cmd_file); // Remove immediately to prevent re-processing

                        for line in content.lines() {
                            if line.starts_with("run:") {
                                let script_name = line.strip_prefix("run:").unwrap_or("").trim();
                                logging::log("TEST", &format!("Test command: run script '{}'", script_name));

                                let script_name_owned = script_name.to_string();
                                let app_entity_inner = app_entity_for_test.clone();
                                let _ = cx.update(|cx| {
                                    app_entity_inner.update(cx, |view, ctx| {
                                        // Find and run the script interactively
                                        if let Some(script) = view.scripts.iter().find(|s| s.name == script_name_owned || s.path.to_string_lossy().contains(&script_name_owned)).cloned() {
                                            logging::log("TEST", &format!("Found script: {}", script.name));
                                            view.execute_interactive(&script, ctx);
                                        } else {
                                            logging::log("TEST", &format!("Script not found: {}", script_name_owned));
                                        }
                                    });
                                });
                            }
                        }
                    }
                }
            }
        }).detach();

        // External command listener - receives commands via stdin (event-driven, no polling)
        let stdin_rx = start_stdin_listener();
        let window_for_stdin = window;
        let app_entity_for_stdin = app_entity.clone();

        // Track if we've received any stdin commands (for timeout warning)
        static STDIN_RECEIVED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

        // Spawn a timeout warning task - helps AI agents detect when they forgot to use stdin protocol
        cx.spawn(async move |_cx: &mut gpui::AsyncApp| {
            Timer::after(std::time::Duration::from_secs(2)).await;
            if !STDIN_RECEIVED.load(std::sync::atomic::Ordering::SeqCst) {
                logging::log("STDIN", "");
                logging::log("STDIN", "╔════════════════════════════════════════════════════════════════════════════╗");
                logging::log("STDIN", "║  WARNING: No stdin JSON received after 2 seconds                          ║");
                logging::log("STDIN", "║                                                                            ║");
                logging::log("STDIN", "║  If you're testing, use the stdin JSON protocol:                          ║");
                logging::log("STDIN", "║  echo '{\"type\":\"run\",\"path\":\"...\"}' | ./target/debug/script-kit-gpui     ║");
                logging::log("STDIN", "║                                                                            ║");
                logging::log("STDIN", "║  Command line args do NOT work:                                           ║");
                logging::log("STDIN", "║  ./target/debug/script-kit-gpui test.ts  # WRONG - does nothing!          ║");
                logging::log("STDIN", "╚════════════════════════════════════════════════════════════════════════════╝");
                logging::log("STDIN", "");
            }
        }).detach();

        cx.spawn(async move |cx: &mut gpui::AsyncApp| {
            logging::log("STDIN", "Async stdin command handler started");

            // Event-driven: recv().await yields until a command arrives
            while let Ok(cmd) = stdin_rx.recv().await {
                // Mark that we've received stdin (clears the timeout warning)
                STDIN_RECEIVED.store(true, std::sync::atomic::Ordering::SeqCst);
                logging::log("STDIN", &format!("Processing external command: {:?}", cmd));

                let app_entity_inner = app_entity_for_stdin.clone();
                let _ = cx.update(|cx| {
                    // Use the Root window to get Window reference, then update the app entity
                    let _ = window_for_stdin.update(cx, |_root, window, root_cx| {
                        app_entity_inner.update(root_cx, |view, ctx| {
                            // Note: We have both `window` from Root and `view` from entity here
                            // ctx is Context<ScriptListApp>, window is &mut Window
                        match cmd {
                            ExternalCommand::Run { ref path, ref request_id } => {
                                let rid = request_id.as_deref().unwrap_or("-");
                                logging::log("STDIN", &format!("[{}] Executing script: {}", rid, path));
                                // Show and focus window - match hotkey handler setup for consistency
                                script_kit_gpui::set_main_window_visible(true);

                                // Position window on mouse display at eye-line (same as hotkey handler)
                                platform::ensure_move_to_active_space();
                                let window_size = gpui::size(px(750.), initial_window_height());
                                let bounds = platform::calculate_eye_line_bounds_on_mouse_display(window_size);
                                platform::move_first_window_to_bounds(&bounds);

                                // Configure as floating panel on first show (same as hotkey handler)
                                if !PANEL_CONFIGURED.load(std::sync::atomic::Ordering::SeqCst) {
                                    platform::configure_as_floating_panel();
                                    PANEL_CONFIGURED.store(true, std::sync::atomic::Ordering::SeqCst);
                                }

                                ctx.activate(true);
                                window.activate_window();
                                let focus_handle = view.focus_handle(ctx);
                                window.focus(&focus_handle, ctx);

                                // Send RunScript message to be handled
                                view.handle_prompt_message(PromptMessage::RunScript { path: path.clone() }, ctx);
                            }
                            ExternalCommand::Show { ref request_id } => {
                                let rid = request_id.as_deref().unwrap_or("-");
                                logging::log("STDIN", &format!("[{}] Showing window", rid));

                                // Menu bar tracking is now handled by frontmost_app_tracker
                                // which pre-fetches menu items in background when apps activate

                                // Show and focus window - match hotkey handler setup for consistency
                                script_kit_gpui::set_main_window_visible(true);

                                // Position window on mouse display at eye-line (same as hotkey handler)
                                platform::ensure_move_to_active_space();
                                let window_size = gpui::size(px(750.), initial_window_height());
                                let bounds = platform::calculate_eye_line_bounds_on_mouse_display(window_size);
                                platform::move_first_window_to_bounds(&bounds);

                                // Configure as floating panel on first show (same as hotkey handler)
                                if !PANEL_CONFIGURED.load(std::sync::atomic::Ordering::SeqCst) {
                                    platform::configure_as_floating_panel();
                                    PANEL_CONFIGURED.store(true, std::sync::atomic::Ordering::SeqCst);
                                }

                                ctx.activate(true);
                                window.activate_window();
                                let focus_handle = view.focus_handle(ctx);
                                window.focus(&focus_handle, ctx);
                            }
                            ExternalCommand::Hide { ref request_id } => {
                                let rid = request_id.as_deref().unwrap_or("-");
                                logging::log("STDIN", &format!("[{}] Hiding main window", rid));
                                script_kit_gpui::set_main_window_visible(false);
                                ctx.hide();
                            }
                            ExternalCommand::SetFilter { ref text, ref request_id } => {
                                let rid = request_id.as_deref().unwrap_or("-");
                                logging::log("STDIN", &format!("[{}] Setting filter to: '{}'", rid, text));
                                view.set_filter_text_immediate(text.clone(), window, ctx);
                                let _ = view.get_filtered_results_cached(); // Update cache
                            }
                            ExternalCommand::TriggerBuiltin { ref name } => {
                                logging::log("STDIN", &format!("Triggering built-in: '{}'", name));
                                // Match built-in name and trigger the corresponding feature
                                match name.to_lowercase().as_str() {
                                    "design-gallery" | "designgallery" | "design gallery" => {
                                        view.current_view = AppView::DesignGalleryView {
                                            filter: String::new(),
                                            selected_index: 0,
                                        };
                                        view.update_window_size();
                                    }
                                    "clipboard" | "clipboard-history" | "clipboardhistory" => {
                                        let entries = clipboard_history::get_cached_entries(100);
                                        view.current_view = AppView::ClipboardHistoryView {
                                            entries,
                                            filter: String::new(),
                                            selected_index: 0,
                                        };
                                        view.update_window_size();
                                    }
                                    "apps" | "app-launcher" | "applauncher" => {
                                        let apps = view.apps.clone();
                                        view.current_view = AppView::AppLauncherView {
                                            apps,
                                            filter: String::new(),
                                            selected_index: 0,
                                        };
                                        view.update_window_size();
                                    }
                                    _ => {
                                        logging::log("ERROR", &format!("Unknown built-in: '{}'", name));
                                    }
                                }
                            }
                            ExternalCommand::SimulateKey { ref key, ref modifiers } => {
                                logging::log("STDIN", &format!("Simulating key: '{}' with modifiers: {:?}", key, modifiers));

                                // Parse modifiers
                                let has_cmd = modifiers.iter().any(|m| m == "cmd" || m == "meta" || m == "command");
                                let has_shift = modifiers.iter().any(|m| m == "shift");
                                let _has_alt = modifiers.iter().any(|m| m == "alt" || m == "option");
                                let _has_ctrl = modifiers.iter().any(|m| m == "ctrl" || m == "control");

                                // Handle key based on current view
                                let key_lower = key.to_lowercase();

                                match &view.current_view {
                                    AppView::ScriptList => {
                                        // Main script list key handling
                                        if has_cmd && key_lower == "k" {
                                            logging::log("STDIN", "SimulateKey: Cmd+K - toggle actions");
                                            view.toggle_actions(ctx, window);
                                        } else {
                                            match key_lower.as_str() {
                                                "up" | "arrowup" => {
                                                    // Use move_selection_up to properly skip section headers
                                                    view.move_selection_up(ctx);
                                                }
                                                "down" | "arrowdown" => {
                                                    // Use move_selection_down to properly skip section headers
                                                    view.move_selection_down(ctx);
                                                }
                                                "enter" => {
                                                    logging::log("STDIN", "SimulateKey: Enter - execute selected");
                                                    view.execute_selected(ctx);
                                                }
                                                "escape" => {
                                                    logging::log("STDIN", "SimulateKey: Escape - clear filter or hide");
                                                    if !view.filter_text.is_empty() {
                                                        view.clear_filter(window, ctx);
                                                    } else {
                                                        script_kit_gpui::set_main_window_visible(false);
                                                        ctx.hide();
                                                    }
                                                }
                                                _ => {
                                                    logging::log("STDIN", &format!("SimulateKey: Unhandled key '{}' in ScriptList", key_lower));
                                                }
                                            }
                                        }
                                    }
                                    AppView::PathPrompt { entity, .. } => {
                                        // Path prompt key handling
                                        logging::log("STDIN", &format!("SimulateKey: Dispatching '{}' to PathPrompt", key_lower));
                                        let entity_clone = entity.clone();
                                        entity_clone.update(ctx, |path_prompt: &mut PathPrompt, path_cx| {
                                            if has_cmd && key_lower == "k" {
                                                path_prompt.toggle_actions(path_cx);
                                            } else {
                                                match key_lower.as_str() {
                                                    "up" | "arrowup" => path_prompt.move_up(path_cx),
                                                    "down" | "arrowdown" => path_prompt.move_down(path_cx),
                                                    "enter" => path_prompt.handle_enter(path_cx),
                                                    "escape" => path_prompt.submit_cancel(),
                                                    "left" | "arrowleft" => path_prompt.navigate_to_parent(path_cx),
                                                    "right" | "arrowright" => path_prompt.navigate_into_selected(path_cx),
                                                    _ => {
                                                        logging::log("STDIN", &format!("SimulateKey: Unhandled key '{}' in PathPrompt", key_lower));
                                                    }
                                                }
                                            }
                                        });
                                    }
                                    AppView::ArgPrompt { id, .. } => {
                                        // Arg prompt key handling via SimulateKey
                                        logging::log("STDIN", &format!("SimulateKey: Dispatching '{}' to ArgPrompt (actions_popup={})", key_lower, view.show_actions_popup));

                                        // Check for Cmd+K to toggle actions popup
                                        if has_cmd && key_lower == "k" {
                                            logging::log("STDIN", "SimulateKey: Cmd+K - toggle arg actions");
                                            view.toggle_arg_actions(ctx, window);
                                        } else if view.show_actions_popup {
                                            // If actions popup is open, route to it
                                            if let Some(ref dialog) = view.actions_dialog {
                                                match key_lower.as_str() {
                                                    "up" | "arrowup" => {
                                                        logging::log("STDIN", "SimulateKey: Up in actions dialog");
                                                        dialog.update(ctx, |d, cx| d.move_up(cx));
                                                    }
                                                    "down" | "arrowdown" => {
                                                        logging::log("STDIN", "SimulateKey: Down in actions dialog");
                                                        dialog.update(ctx, |d, cx| d.move_down(cx));
                                                    }
                                                    "enter" => {
                                                        logging::log("STDIN", "SimulateKey: Enter in actions dialog");
                                                        let action_id = dialog.read(ctx).get_selected_action_id();
                                                        let should_close = dialog.read(ctx).selected_action_should_close();
                                                        if let Some(action_id) = action_id {
                                                            logging::log("ACTIONS", &format!("SimulateKey: Executing action: {} (close={})", action_id, should_close));
                                                            if should_close {
                                                                view.show_actions_popup = false;
                                                                view.actions_dialog = None;
                                                                view.focused_input = FocusedInput::ArgPrompt;
                                                                window.focus(&view.focus_handle, ctx);
                                                            }
                                                            view.trigger_action_by_name(&action_id, ctx);
                                                        }
                                                    }
                                                    "escape" => {
                                                        logging::log("STDIN", "SimulateKey: Escape - close actions dialog");
                                                        view.show_actions_popup = false;
                                                        view.actions_dialog = None;
                                                        view.focused_input = FocusedInput::ArgPrompt;
                                                        window.focus(&view.focus_handle, ctx);
                                                    }
                                                    _ => {
                                                        logging::log("STDIN", &format!("SimulateKey: Unhandled key '{}' in ArgPrompt actions dialog", key_lower));
                                                    }
                                                }
                                            }
                                        } else {
                                            // Normal arg prompt key handling
                                            let prompt_id = id.clone();
                                            match key_lower.as_str() {
                                                "up" | "arrowup" => {
                                                    if view.arg_selected_index > 0 {
                                                        view.arg_selected_index -= 1;
                                                        view.arg_list_scroll_handle.scroll_to_item(view.arg_selected_index, ScrollStrategy::Nearest);
                                                        logging::log("STDIN", &format!("SimulateKey: Arg up, index={}", view.arg_selected_index));
                                                    }
                                                }
                                                "down" | "arrowdown" => {
                                                    let filtered = view.filtered_arg_choices();
                                                    if view.arg_selected_index < filtered.len().saturating_sub(1) {
                                                        view.arg_selected_index += 1;
                                                        view.arg_list_scroll_handle.scroll_to_item(view.arg_selected_index, ScrollStrategy::Nearest);
                                                        logging::log("STDIN", &format!("SimulateKey: Arg down, index={}", view.arg_selected_index));
                                                    }
                                                }
                                                "enter" => {
                                                    logging::log("STDIN", "SimulateKey: Enter - submit selection");
                                                    let filtered = view.filtered_arg_choices();
                                                    if let Some((_, choice)) = filtered.get(view.arg_selected_index) {
                                                        let value = choice.value.clone();
                                                        view.submit_prompt_response(prompt_id, Some(value), ctx);
                                                    } else if !view.arg_input.is_empty() {
                                                        let value = view.arg_input.text().to_string();
                                                        view.submit_prompt_response(prompt_id, Some(value), ctx);
                                                    }
                                                }
                                                "escape" => {
                                                    logging::log("STDIN", "SimulateKey: Escape - cancel script");
                                                    view.submit_prompt_response(prompt_id, None, ctx);
                                                    view.cancel_script_execution(ctx);
                                                }
                                                _ => {
                                                    logging::log("STDIN", &format!("SimulateKey: Unhandled key '{}' in ArgPrompt", key_lower));
                                                }
                                            }
                                        }
                                    }
                                    AppView::EditorPrompt { entity, id, .. } => {
                                        // Editor prompt key handling for template/snippet navigation and choice popup
                                        logging::log("STDIN", &format!("SimulateKey: Dispatching '{}' to EditorPrompt", key_lower));
                                        let entity_clone = entity.clone();
                                        let prompt_id_clone = id.clone();

                                        // Check if choice popup is visible
                                        let has_choice_popup = entity_clone.update(ctx, |editor: &mut EditorPrompt, _| {
                                            editor.is_choice_popup_visible()
                                        });

                                        if has_choice_popup {
                                            // Handle choice popup navigation
                                            match key_lower.as_str() {
                                                "up" | "arrowup" => {
                                                    logging::log("STDIN", "SimulateKey: Up in choice popup");
                                                    entity_clone.update(ctx, |editor, cx| {
                                                        editor.choice_popup_up_public(cx);
                                                    });
                                                }
                                                "down" | "arrowdown" => {
                                                    logging::log("STDIN", "SimulateKey: Down in choice popup");
                                                    entity_clone.update(ctx, |editor, cx| {
                                                        editor.choice_popup_down_public(cx);
                                                    });
                                                }
                                                "enter" if !has_cmd => {
                                                    logging::log("STDIN", "SimulateKey: Enter in choice popup - confirming");
                                                    entity_clone.update(ctx, |editor, cx| {
                                                        editor.choice_popup_confirm_public(window, cx);
                                                    });
                                                }
                                                "escape" => {
                                                    logging::log("STDIN", "SimulateKey: Escape in choice popup - cancelling");
                                                    entity_clone.update(ctx, |editor, cx| {
                                                        editor.choice_popup_cancel_public(cx);
                                                    });
                                                }
                                                "tab" if !has_shift => {
                                                    logging::log("STDIN", "SimulateKey: Tab in choice popup - confirm and next");
                                                    entity_clone.update(ctx, |editor, cx| {
                                                        editor.choice_popup_confirm_public(window, cx);
                                                        editor.next_tabstop_public(window, cx);
                                                    });
                                                }
                                                _ => {
                                                    logging::log("STDIN", &format!("SimulateKey: Unhandled key '{}' in choice popup", key_lower));
                                                }
                                            }
                                        } else if key_lower == "tab" && !has_cmd {
                                            // Handle Tab key for snippet navigation
                                            entity_clone.update(ctx, |editor: &mut EditorPrompt, editor_cx| {
                                                logging::log("STDIN", "SimulateKey: Tab in EditorPrompt - calling next_tabstop");
                                                if editor.in_snippet_mode() {
                                                    editor.next_tabstop_public(window, editor_cx);
                                                } else {
                                                    logging::log("STDIN", "SimulateKey: Tab - not in snippet mode");
                                                }
                                            });
                                        } else if key_lower == "enter" && has_cmd {
                                            // Cmd+Enter submits - get content from editor
                                            logging::log("STDIN", "SimulateKey: Cmd+Enter in EditorPrompt - submitting");
                                            let content = entity_clone.update(ctx, |editor, editor_cx| {
                                                editor.content(editor_cx)
                                            });
                                            view.submit_prompt_response(prompt_id_clone.clone(), Some(content), ctx);
                                        } else if key_lower == "escape" && !has_cmd {
                                            logging::log("STDIN", "SimulateKey: Escape in EditorPrompt - cancelling");
                                            view.submit_prompt_response(prompt_id_clone.clone(), None, ctx);
                                            view.cancel_script_execution(ctx);
                                        } else {
                                            logging::log("STDIN", &format!("SimulateKey: Unhandled key '{}' in EditorPrompt", key_lower));
                                        }
                                    }
                                    _ => {
                                        logging::log("STDIN", &format!("SimulateKey: View {:?} not supported for key simulation", std::mem::discriminant(&view.current_view)));
                                    }
                                }
                            }
                            ExternalCommand::OpenNotes => {
                                logging::log("STDIN", "Opening notes window via stdin command");
                                if let Err(e) = notes::open_notes_window(ctx) {
                                    logging::log("STDIN", &format!("Failed to open notes window: {}", e));
                                }
                            }
                            ExternalCommand::OpenAi => {
                                logging::log("STDIN", "Opening AI window via stdin command");
                                if let Err(e) = ai::open_ai_window(ctx) {
                                    logging::log("STDIN", &format!("Failed to open AI window: {}", e));
                                }
                            }
                            ExternalCommand::OpenAiWithMockData => {
                                logging::log("STDIN", "Opening AI window with mock data via stdin command");
                                // First insert mock data
                                if let Err(e) = ai::insert_mock_data() {
                                    logging::log("STDIN", &format!("Failed to insert mock data: {}", e));
                                } else {
                                    logging::log("STDIN", "Mock data inserted successfully");
                                }
                                // Then open the window
                                if let Err(e) = ai::open_ai_window(ctx) {
                                    logging::log("STDIN", &format!("Failed to open AI window: {}", e));
                                }
                            }
                            ExternalCommand::CaptureWindow { title, path } => {
                                logging::log("STDIN", &format!("Capturing window with title '{}' to '{}'", title, path));
                                match capture_window_by_title(&title, false) {
                                    Ok((png_data, width, height)) => {
                                        // Save to file
                                        if let Err(e) = std::fs::write(&path, &png_data) {
                                            logging::log("STDIN", &format!("Failed to write screenshot: {}", e));
                                        } else {
                                            logging::log("STDIN", &format!("Screenshot saved: {} ({}x{})", path, width, height));
                                        }
                                    }
                                    Err(e) => {
                                        logging::log("STDIN", &format!("Failed to capture window: {}", e));
                                    }
                                }
                            }
                            ExternalCommand::SetAiSearch { text } => {
                                logging::log("STDIN", &format!("Setting AI search filter to: {}", text));
                                ai::set_ai_search(ctx, &text);
                            }
                            ExternalCommand::SetAiInput { text, submit } => {
                                logging::log("STDIN", &format!("Setting AI input to: {} (submit={})", text, submit));
                                ai::set_ai_input(ctx, &text, submit);
                            }
                            ExternalCommand::ShowGrid { grid_size, show_bounds, show_box_model, show_alignment_guides, show_dimensions, ref depth } => {
                                logging::log("STDIN", &format!(
                                    "ShowGrid: size={}, bounds={}, box_model={}, guides={}, dimensions={}, depth={:?}",
                                    grid_size, show_bounds, show_box_model, show_alignment_guides, show_dimensions, depth
                                ));
                                let options = protocol::GridOptions {
                                    grid_size,
                                    show_bounds,
                                    show_box_model,
                                    show_alignment_guides,
                                    show_dimensions,
                                    depth: depth.clone(),
                                    color_scheme: None,
                                };
                                view.show_grid(options, ctx);
                            }
                            ExternalCommand::HideGrid => {
                                logging::log("STDIN", "HideGrid: hiding debug grid overlay");
                                view.hide_grid(ctx);
                            }
                        }
                        ctx.notify();
                        }); // close app_entity_inner.update
                    }); // close window_for_stdin.update
                }); // close cx.update
            }

            logging::log("STDIN", "Async stdin command handler exiting");
        }).detach();

        // Tray menu event handler - polls for menu events
        // Clone config for use in tray handler
        let config_for_tray = config::load_config();
        if let Some(tray_mgr) = tray_manager {
            let window_for_tray = window;
            let app_entity_for_tray = app_entity.clone();
            cx.spawn(async move |cx: &mut gpui::AsyncApp| {
                logging::log("TRAY", "Tray menu event handler started");

                loop {
                    // Poll for tray menu events every 100ms
                    Timer::after(std::time::Duration::from_millis(100)).await;

                    // Check for menu events
                    if let Ok(event) = tray_mgr.menu_event_receiver().try_recv() {
                        match tray_mgr.match_menu_event(&event) {
                            Some(TrayMenuAction::OpenScriptKit) => {
                                logging::log("TRAY", "Open Script Kit menu item clicked");
                                let app_entity_inner = app_entity_for_tray.clone();
                                let _ = cx.update(|cx| {
                                    // Show and focus window (same logic as hotkey handler)
                                    script_kit_gpui::set_main_window_visible(true);

                                    // Calculate new bounds on display with mouse
                                    let window_size = size(px(750.), initial_window_height());
                                    let new_bounds = calculate_eye_line_bounds_on_mouse_display(window_size);

                                    // Move window first
                                    move_first_window_to_bounds(&new_bounds);

                                    // Activate the app
                                    cx.activate(true);

                                    // Configure as floating panel on first show
                                    if !PANEL_CONFIGURED.swap(true, Ordering::SeqCst) {
                                        platform::configure_as_floating_panel();
                                    }

                                    // Focus the window via Root, then update app entity
                                    let _ = window_for_tray.update(cx, |_root, win, root_cx| {
                                        win.activate_window();
                                        app_entity_inner.update(root_cx, |view, ctx| {
                                            let focus_handle = view.focus_handle(ctx);
                                            win.focus(&focus_handle, ctx);

                                            // Reset if needed and ensure proper sizing
                                            reset_resize_debounce();

                                            if NEEDS_RESET.compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst).is_ok() {
                                                view.reset_to_script_list(ctx);
                                            } else {
                                                view.update_window_size();
                                            }
                                        });
                                    });
                                });
                            }
                            Some(TrayMenuAction::OpenNotes) => {
                                logging::log("TRAY", "Notes menu item clicked");
                                let _ = cx.update(|cx| {
                                    if let Err(e) = notes::open_notes_window(cx) {
                                        logging::log(
                                            "TRAY",
                                            &format!("Failed to open notes window: {}", e),
                                        );
                                    }
                                });
                            }
                            Some(TrayMenuAction::OpenAiChat) => {
                                logging::log("TRAY", "AI Chat menu item clicked");
                                let _ = cx.update(|cx| {
                                    if let Err(e) = ai::open_ai_window(cx) {
                                        logging::log(
                                            "TRAY",
                                            &format!("Failed to open AI window: {}", e),
                                        );
                                    }
                                });
                            }
                            Some(TrayMenuAction::LaunchAtLogin) => {
                                // Toggle is handled inside match_menu_event
                                logging::log("TRAY", "Launch at Login toggled");
                            }
                            Some(TrayMenuAction::Settings) => {
                                logging::log("TRAY", "Settings menu item clicked");
                                // Open config file in editor
                                let editor = config_for_tray.get_editor();
                                let config_path = shellexpand::tilde("~/.sk/kit/config.ts").to_string();

                                logging::log("TRAY", &format!("Opening {} in editor '{}'", config_path, editor));
                                match std::process::Command::new(&editor)
                                    .arg(&config_path)
                                    .spawn()
                                {
                                    Ok(_) => logging::log("TRAY", &format!("Spawned editor: {}", editor)),
                                    Err(e) => logging::log("TRAY", &format!("Failed to spawn editor '{}': {}", editor, e)),
                                }
                            }
                            Some(TrayMenuAction::OpenOnGitHub) => {
                                logging::log("TRAY", "Open on GitHub menu item clicked");
                                let url = "https://github.com/script-kit/app";
                                if let Err(e) = open::that(url) {
                                    logging::log("TRAY", &format!("Failed to open GitHub URL: {}", e));
                                }
                            }
                            Some(TrayMenuAction::OpenManual) => {
                                logging::log("TRAY", "Manual menu item clicked");
                                let url = "https://scriptkit.com";
                                if let Err(e) = open::that(url) {
                                    logging::log("TRAY", &format!("Failed to open manual URL: {}", e));
                                }
                            }
                            Some(TrayMenuAction::JoinCommunity) => {
                                logging::log("TRAY", "Join Community menu item clicked");
                                let url = "https://discord.gg/qnUX4XqJQd";
                                if let Err(e) = open::that(url) {
                                    logging::log("TRAY", &format!("Failed to open Discord URL: {}", e));
                                }
                            }
                            Some(TrayMenuAction::FollowUs) => {
                                logging::log("TRAY", "Follow Us menu item clicked");
                                let url = "https://twitter.com/scriptkitapp";
                                if let Err(e) = open::that(url) {
                                    logging::log("TRAY", &format!("Failed to open Twitter URL: {}", e));
                                }
                            }
                            Some(TrayMenuAction::Quit) => {
                                logging::log("TRAY", "Quit menu item clicked");
                                // Clean up processes and PID file before quitting
                                PROCESS_MANAGER.kill_all_processes();
                                PROCESS_MANAGER.remove_main_pid();
                                let _ = cx.update(|cx| {
                                    cx.quit();
                                });
                                break; // Exit the polling loop
                            }
                            None => {
                                logging::log("TRAY", "Unknown menu event received");
                            }
                        }
                    }
                }

                logging::log("TRAY", "Tray menu event handler exiting");
            }).detach();
        }

        logging::log("APP", "Application ready - Cmd+; to show, Esc to hide, Cmd+K for actions");
    });
}

#[cfg(test)]
mod tests {
    use super::{is_main_window_visible, set_main_window_visible};

    #[test]
    fn main_window_visibility_is_shared_with_library() {
        set_main_window_visible(false);
        script_kit_gpui::set_main_window_visible(false);

        set_main_window_visible(true);
        assert!(
            script_kit_gpui::is_main_window_visible(),
            "library visibility should mirror main visibility"
        );

        script_kit_gpui::set_main_window_visible(false);
        assert!(
            !is_main_window_visible(),
            "main visibility should mirror library visibility"
        );
    }
}

</file>

<file path="src/app_render.rs">
impl ScriptListApp {
    /// Read the first N lines of a script file for preview
    #[allow(dead_code)]
    fn read_script_preview(path: &std::path::Path, max_lines: usize) -> String {
        match std::fs::read_to_string(path) {
            Ok(content) => {
                let preview: String = content
                    .lines()
                    .take(max_lines)
                    .collect::<Vec<_>>()
                    .join("\n");
                logging::log(
                    "UI",
                    &format!(
                        "Preview loaded: {} ({} lines read)",
                        path.file_name().unwrap_or_default().to_string_lossy(),
                        content.lines().count().min(max_lines)
                    ),
                );
                preview
            }
            Err(e) => {
                logging::log("UI", &format!("Preview error: {} - {}", path.display(), e));
                format!("Error reading file: {}", e)
            }
        }
    }

    // NOTE: render_toasts() removed - now using gpui-component's NotificationList
    // via the Root wrapper. Toasts are flushed via flush_pending_toasts() in render().
    // See toast_manager.rs for the queue and main.rs for the flush logic.

    /// Render the preview panel showing details of the selected script/scriptlet
    fn render_preview_panel(&mut self, _cx: &mut Context<Self>) -> impl IntoElement {
        // Get grouped results to map from selected_index to actual result (cached)
        // Clone to avoid borrow issues with self.selected_index access
        let selected_index = self.selected_index;
        let (grouped_items, flat_results) = self.get_grouped_results_cached();
        let grouped_items = grouped_items.clone();
        let flat_results = flat_results.clone();

        // Get the result index from the grouped item
        let selected_result = match grouped_items.get(selected_index) {
            Some(GroupedListItem::Item(idx)) => flat_results.get(*idx).cloned(),
            _ => None,
        };

        // Use design tokens for GLOBAL theming - design applies to ALL components
        let tokens = get_tokens(self.current_design);
        let colors = tokens.colors();
        let spacing = tokens.spacing();
        let typography = tokens.typography();
        let visual = tokens.visual();

        // Map design tokens to local variables (all designs use tokens now)
        let bg_main = colors.background;
        let ui_border = colors.border;
        let text_primary = colors.text_primary;
        let text_muted = colors.text_muted;
        let text_secondary = colors.text_secondary;
        let bg_search_box = colors.background_tertiary;
        let border_radius = visual.radius_md;
        let font_family = typography.font_family;

        // Preview panel container with left border separator
        let mut panel = div()
            .w_full()
            .h_full()
            .bg(rgb(bg_main))
            .border_l_1()
            .border_color(rgba((ui_border << 8) | 0x80))
            .p(px(spacing.padding_lg))
            .flex()
            .flex_col()
            .overflow_y_hidden()
            .font_family(font_family);

        // P4: Compute match indices lazily for visible preview (only one result at a time)
        let computed_filter = self.computed_filter_text.clone();

        match selected_result {
            Some(ref result) => {
                // P4: Lazy match indices computation for preview panel
                let match_indices =
                    scripts::compute_match_indices_for_result(result, &computed_filter);

                match result {
                    scripts::SearchResult::Script(script_match) => {
                        let script = &script_match.script;

                        // Source indicator with match highlighting (e.g., "script: foo.ts")
                        let filename = &script_match.filename;
                        // P4: Use lazily computed indices instead of stored (empty) ones
                        let filename_indices = &match_indices.filename_indices;

                        // Render filename with highlighted matched characters
                        let path_segments =
                            render_path_with_highlights(filename, filename, filename_indices);
                        let accent_color = colors.accent;

                        let mut path_div = div()
                            .flex()
                            .flex_row()
                            .text_xs()
                            .font_family(typography.font_family_mono)
                            .pb(px(spacing.padding_xs))
                            .overflow_x_hidden()
                            .child(
                                div()
                                    .text_color(rgba((text_muted << 8) | 0x99))
                                    .child("script: "),
                            );

                        for (text, is_highlighted) in path_segments {
                            let color = if is_highlighted {
                                rgb(accent_color)
                            } else {
                                rgba((text_muted << 8) | 0x99)
                            };
                            path_div = path_div.child(div().text_color(color).child(text));
                        }

                        panel = panel.child(path_div);

                        // Script name header
                        panel = panel.child(
                            div()
                                .text_lg()
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(rgb(text_primary))
                                .pb(px(spacing.padding_sm))
                                .child(format!("{}.{}", script.name, script.extension)),
                        );

                        // Description (if present)
                        if let Some(desc) = &script.description {
                            panel = panel.child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .pb(px(spacing.padding_md))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(text_muted))
                                            .pb(px(spacing.padding_xs / 2.0))
                                            .child("Description"),
                                    )
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(rgb(text_secondary))
                                            .child(desc.clone()),
                                    ),
                            );
                        }

                        // Divider
                        panel = panel.child(
                            div()
                                .w_full()
                                .h(px(visual.border_thin))
                                .bg(rgba((ui_border << 8) | 0x60))
                                .my(px(spacing.padding_sm)),
                        );

                        // Code preview header
                        panel = panel.child(
                            div()
                                .text_xs()
                                .text_color(rgb(text_muted))
                                .pb(px(spacing.padding_sm))
                                .child("Code Preview"),
                        );

                        // Use cached syntax-highlighted lines (avoids file I/O and highlighting on every render)
                        let script_path = script.path.to_string_lossy().to_string();
                        let lang = script.extension.clone();
                        let lines = self
                            .get_or_update_preview_cache(&script_path, &lang)
                            .to_vec();

                        // Build code container - render line by line with monospace font
                        let mut code_container = div()
                            .w_full()
                            .min_w(px(280.))
                            .p(px(spacing.padding_md))
                            .rounded(px(border_radius))
                            .bg(rgba((bg_search_box << 8) | 0x80))
                            .overflow_hidden()
                            .flex()
                            .flex_col();

                        // Render each line as a row of spans with monospace font
                        for line in lines {
                            let mut line_div = div()
                                .flex()
                                .flex_row()
                                .w_full()
                                .font_family(typography.font_family_mono)
                                .text_xs()
                                .min_h(px(spacing.padding_lg)); // Line height

                            if line.spans.is_empty() {
                                // Empty line - add a space to preserve height
                                line_div = line_div.child(" ");
                            } else {
                                for span in line.spans {
                                    line_div = line_div
                                        .child(div().text_color(rgb(span.color)).child(span.text));
                                }
                            }

                            code_container = code_container.child(line_div);
                        }

                        panel = panel.child(code_container);
                    }
                    scripts::SearchResult::Scriptlet(scriptlet_match) => {
                        let scriptlet = &scriptlet_match.scriptlet;

                        // Source indicator with match highlighting (e.g., "scriptlet: foo.md")
                        if let Some(ref display_file_path) = scriptlet_match.display_file_path {
                            // P4: Use lazily computed indices instead of stored (empty) ones
                            let filename_indices = &match_indices.filename_indices;

                            // Render filename with highlighted matched characters
                            let path_segments = render_path_with_highlights(
                                display_file_path,
                                display_file_path,
                                filename_indices,
                            );
                            let accent_color = colors.accent;

                            let mut path_div = div()
                                .flex()
                                .flex_row()
                                .text_xs()
                                .font_family(typography.font_family_mono)
                                .pb(px(spacing.padding_xs))
                                .overflow_x_hidden()
                                .child(
                                    div()
                                        .text_color(rgba((text_muted << 8) | 0x99))
                                        .child("scriptlet: "),
                                );

                            for (text, is_highlighted) in path_segments {
                                let color = if is_highlighted {
                                    rgb(accent_color)
                                } else {
                                    rgba((text_muted << 8) | 0x99)
                                };
                                path_div = path_div.child(div().text_color(color).child(text));
                            }

                            panel = panel.child(path_div);
                        }

                        // Scriptlet name header
                        panel = panel.child(
                            div()
                                .text_lg()
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(rgb(text_primary))
                                .pb(px(spacing.padding_sm))
                                .child(scriptlet.name.clone()),
                        );

                        // Description (if present)
                        if let Some(desc) = &scriptlet.description {
                            panel = panel.child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .pb(px(spacing.padding_md))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(text_muted))
                                            .pb(px(spacing.padding_xs / 2.0))
                                            .child("Description"),
                                    )
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(rgb(text_secondary))
                                            .child(desc.clone()),
                                    ),
                            );
                        }

                        // Shortcut (if present)
                        if let Some(shortcut) = &scriptlet.shortcut {
                            panel = panel.child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .pb(px(spacing.padding_md))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(text_muted))
                                            .pb(px(spacing.padding_xs / 2.0))
                                            .child("Hotkey"),
                                    )
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(rgb(text_secondary))
                                            .child(shortcut.clone()),
                                    ),
                            );
                        }

                        // Divider
                        panel = panel.child(
                            div()
                                .w_full()
                                .h(px(visual.border_thin))
                                .bg(rgba((ui_border << 8) | 0x60))
                                .my(px(spacing.padding_sm)),
                        );

                        // Content preview header
                        panel = panel.child(
                            div()
                                .text_xs()
                                .text_color(rgb(text_muted))
                                .pb(px(spacing.padding_sm))
                                .child("Content Preview"),
                        );

                        // Display scriptlet code with syntax highlighting (first 15 lines)
                        // Note: Scriptlets store code in memory, no file I/O needed (no cache benefit)
                        let code_preview: String = scriptlet
                            .code
                            .lines()
                            .take(15)
                            .collect::<Vec<_>>()
                            .join("\n");

                        // Determine language from tool (bash, js, etc.)
                        let lang = match scriptlet.tool.as_str() {
                            "bash" | "zsh" | "sh" => "bash",
                            "node" | "bun" => "js",
                            _ => &scriptlet.tool,
                        };
                        let lines = highlight_code_lines(&code_preview, lang);

                        // Build code container - render line by line with monospace font
                        let mut code_container = div()
                            .w_full()
                            .min_w(px(280.))
                            .p(px(spacing.padding_md))
                            .rounded(px(border_radius))
                            .bg(rgba((bg_search_box << 8) | 0x80))
                            .overflow_hidden()
                            .flex()
                            .flex_col();

                        // Render each line as a row of spans with monospace font
                        for line in lines {
                            let mut line_div = div()
                                .flex()
                                .flex_row()
                                .w_full()
                                .font_family(typography.font_family_mono)
                                .text_xs()
                                .min_h(px(spacing.padding_lg)); // Line height

                            if line.spans.is_empty() {
                                // Empty line - add a space to preserve height
                                line_div = line_div.child(" ");
                            } else {
                                for span in line.spans {
                                    line_div = line_div
                                        .child(div().text_color(rgb(span.color)).child(span.text));
                                }
                            }

                            code_container = code_container.child(line_div);
                        }

                        panel = panel.child(code_container);
                    }
                    scripts::SearchResult::BuiltIn(builtin_match) => {
                        let builtin = &builtin_match.entry;

                        // Built-in name header
                        panel = panel.child(
                            div()
                                .text_lg()
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(rgb(text_primary))
                                .pb(px(spacing.padding_sm))
                                .child(builtin.name.clone()),
                        );

                        // Description
                        panel = panel.child(
                            div()
                                .flex()
                                .flex_col()
                                .pb(px(spacing.padding_md))
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(text_muted))
                                        .pb(px(spacing.padding_xs / 2.0))
                                        .child("Description"),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(rgb(text_secondary))
                                        .child(builtin.description.clone()),
                                ),
                        );

                        // Keywords
                        if !builtin.keywords.is_empty() {
                            panel = panel.child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .pb(px(spacing.padding_md))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(text_muted))
                                            .pb(px(spacing.padding_xs / 2.0))
                                            .child("Keywords"),
                                    )
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(rgb(text_secondary))
                                            .child(builtin.keywords.join(", ")),
                                    ),
                            );
                        }

                        // Divider
                        panel = panel.child(
                            div()
                                .w_full()
                                .h(px(visual.border_thin))
                                .bg(rgba((ui_border << 8) | 0x60))
                                .my(px(spacing.padding_sm)),
                        );

                        // Feature type indicator
                        let feature_type: String = match &builtin.feature {
                            builtins::BuiltInFeature::ClipboardHistory => {
                                "Clipboard History Manager".to_string()
                            }
                            builtins::BuiltInFeature::AppLauncher => {
                                "Application Launcher".to_string()
                            }
                            builtins::BuiltInFeature::App(name) => name.clone(),
                            builtins::BuiltInFeature::WindowSwitcher => {
                                "Window Manager".to_string()
                            }
                            builtins::BuiltInFeature::DesignGallery => "Design Gallery".to_string(),
                            builtins::BuiltInFeature::AiChat => "AI Assistant".to_string(),
                            builtins::BuiltInFeature::Notes => "Notes & Scratchpad".to_string(),
                            builtins::BuiltInFeature::MenuBarAction(_) => {
                                "Menu Bar Action".to_string()
                            }
                            builtins::BuiltInFeature::SystemAction(_) => {
                                "System Action".to_string()
                            }
                            builtins::BuiltInFeature::WindowAction(_) => {
                                "Window Action".to_string()
                            }
                            builtins::BuiltInFeature::NotesCommand(_) => {
                                "Notes Command".to_string()
                            }
                            builtins::BuiltInFeature::AiCommand(_) => "AI Command".to_string(),
                            builtins::BuiltInFeature::ScriptCommand(_) => {
                                "Script Creation".to_string()
                            }
                            builtins::BuiltInFeature::PermissionCommand(_) => {
                                "Permission Management".to_string()
                            }
                            builtins::BuiltInFeature::FrecencyCommand(_) => {
                                "Suggested Items".to_string()
                            }
                        };
                        panel = panel.child(
                            div()
                                .flex()
                                .flex_col()
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(text_muted))
                                        .pb(px(spacing.padding_xs / 2.0))
                                        .child("Feature Type"),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(rgb(text_secondary))
                                        .child(feature_type),
                                ),
                        );
                    }
                    scripts::SearchResult::App(app_match) => {
                        let app = &app_match.app;

                        // App name header
                        panel = panel.child(
                            div()
                                .text_lg()
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(rgb(text_primary))
                                .pb(px(spacing.padding_sm))
                                .child(app.name.clone()),
                        );

                        // Path
                        panel = panel.child(
                            div()
                                .flex()
                                .flex_col()
                                .pb(px(spacing.padding_md))
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(text_muted))
                                        .pb(px(spacing.padding_xs / 2.0))
                                        .child("Path"),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(rgb(text_secondary))
                                        .child(app.path.to_string_lossy().to_string()),
                                ),
                        );

                        // Bundle ID (if available)
                        if let Some(bundle_id) = &app.bundle_id {
                            panel = panel.child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .pb(px(spacing.padding_md))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(text_muted))
                                            .pb(px(spacing.padding_xs / 2.0))
                                            .child("Bundle ID"),
                                    )
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(rgb(text_secondary))
                                            .child(bundle_id.clone()),
                                    ),
                            );
                        }

                        // Divider
                        panel = panel.child(
                            div()
                                .w_full()
                                .h(px(visual.border_thin))
                                .bg(rgba((ui_border << 8) | 0x60))
                                .my(px(spacing.padding_sm)),
                        );

                        // Type indicator
                        panel = panel.child(
                            div()
                                .flex()
                                .flex_col()
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(text_muted))
                                        .pb(px(spacing.padding_xs / 2.0))
                                        .child("Type"),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(rgb(text_secondary))
                                        .child("Application"),
                                ),
                        );
                    }
                    scripts::SearchResult::Window(window_match) => {
                        let window = &window_match.window;

                        // Window title header
                        panel = panel.child(
                            div()
                                .text_lg()
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(rgb(text_primary))
                                .pb(px(spacing.padding_sm))
                                .child(window.title.clone()),
                        );

                        // App name
                        panel = panel.child(
                            div()
                                .flex()
                                .flex_col()
                                .pb(px(spacing.padding_md))
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(text_muted))
                                        .pb(px(spacing.padding_xs / 2.0))
                                        .child("Application"),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(rgb(text_secondary))
                                        .child(window.app.clone()),
                                ),
                        );

                        // Bounds
                        panel = panel.child(
                            div()
                                .flex()
                                .flex_col()
                                .pb(px(spacing.padding_md))
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(text_muted))
                                        .pb(px(spacing.padding_xs / 2.0))
                                        .child("Position & Size"),
                                )
                                .child(div().text_sm().text_color(rgb(text_secondary)).child(
                                    format!(
                                        "{}×{} at ({}, {})",
                                        window.bounds.width,
                                        window.bounds.height,
                                        window.bounds.x,
                                        window.bounds.y
                                    ),
                                )),
                        );

                        // Divider
                        panel = panel.child(
                            div()
                                .w_full()
                                .h(px(visual.border_thin))
                                .bg(rgba((ui_border << 8) | 0x60))
                                .my(px(spacing.padding_sm)),
                        );

                        // Type indicator
                        panel = panel.child(
                            div()
                                .flex()
                                .flex_col()
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(text_muted))
                                        .pb(px(spacing.padding_xs / 2.0))
                                        .child("Type"),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(rgb(text_secondary))
                                        .child("Window"),
                                ),
                        );
                    }
                    scripts::SearchResult::Agent(agent_match) => {
                        let agent = &agent_match.agent;

                        // Source indicator with agent path
                        let filename = agent
                            .path
                            .file_name()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_else(|| "agent".to_string());

                        let mut path_div = div()
                            .flex()
                            .flex_row()
                            .text_xs()
                            .font_family(typography.font_family_mono)
                            .pb(px(spacing.padding_xs))
                            .overflow_x_hidden()
                            .child(
                                div()
                                    .text_color(rgba((text_muted << 8) | 0x99))
                                    .child("agent: "),
                            );

                        path_div = path_div.child(
                            div()
                                .text_color(rgba((text_muted << 8) | 0x99))
                                .child(filename),
                        );

                        panel = panel.child(path_div);

                        // Agent name header
                        panel = panel.child(
                            div()
                                .text_lg()
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(rgb(text_primary))
                                .pb(px(spacing.padding_sm))
                                .child(agent.name.clone()),
                        );

                        // Description
                        if let Some(desc) = &agent.description {
                            panel = panel.child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(text_secondary))
                                    .pb(px(spacing.padding_md))
                                    .child(desc.clone()),
                            );
                        }

                        // Backend info
                        panel = panel.child(
                            div()
                                .flex()
                                .flex_col()
                                .pb(px(spacing.padding_md))
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(text_muted))
                                        .pb(px(spacing.padding_xs / 2.0))
                                        .child("Backend"),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(rgb(text_secondary))
                                        .child(format!("{:?}", agent.backend)),
                                ),
                        );

                        // Kit info if available
                        if let Some(kit) = &agent.kit {
                            panel = panel.child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .pb(px(spacing.padding_md))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(text_muted))
                                            .pb(px(spacing.padding_xs / 2.0))
                                            .child("Kit"),
                                    )
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(rgb(text_secondary))
                                            .child(kit.clone()),
                                    ),
                            );
                        }

                        // Divider
                        panel = panel.child(
                            div()
                                .w_full()
                                .h(px(visual.border_thin))
                                .bg(rgba((ui_border << 8) | 0x60))
                                .my(px(spacing.padding_sm)),
                        );

                        // Type indicator
                        panel = panel.child(
                            div()
                                .flex()
                                .flex_col()
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(text_muted))
                                        .pb(px(spacing.padding_xs / 2.0))
                                        .child("Type"),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(rgb(text_secondary))
                                        .child("Agent"),
                                ),
                        );
                    }
                }
            }
            None => {
                logging::log("UI", "Preview panel: No selection");
                // Empty state
                panel = panel.child(
                    div()
                        .w_full()
                        .h_full()
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_color(rgb(text_muted))
                        .child(
                            if self.filter_text.is_empty()
                                && self.scripts.is_empty()
                                && self.scriptlets.is_empty()
                            {
                                "No scripts or snippets found"
                            } else if !self.filter_text.is_empty() {
                                "No matching scripts"
                            } else {
                                "Select a script to preview"
                            },
                        ),
                );
            }
        }

        panel
    }

    /// Get the ScriptInfo for the currently focused/selected script
    fn get_focused_script_info(&mut self) -> Option<ScriptInfo> {
        // Get grouped results to map from selected_index to actual result (cached)
        let (grouped_items, flat_results) = self.get_grouped_results_cached();
        // Clone to avoid borrow issues
        let grouped_items = grouped_items.clone();
        let flat_results = flat_results.clone();

        // Get the result index from the grouped item
        let result_idx = match grouped_items.get(self.selected_index) {
            Some(GroupedListItem::Item(idx)) => Some(*idx),
            _ => None,
        };

        if let Some(idx) = result_idx {
            if let Some(result) = flat_results.get(idx) {
                match result {
                    scripts::SearchResult::Script(m) => Some(ScriptInfo::new(
                        &m.script.name,
                        m.script.path.to_string_lossy(),
                    )),
                    scripts::SearchResult::Scriptlet(m) => {
                        // Scriptlets don't have a path, use name as identifier
                        Some(ScriptInfo::new(
                            &m.scriptlet.name,
                            format!("scriptlet:{}", &m.scriptlet.name),
                        ))
                    }
                    scripts::SearchResult::BuiltIn(m) => {
                        // Built-ins use their id as identifier
                        Some(ScriptInfo::new(
                            &m.entry.name,
                            format!("builtin:{}", &m.entry.id),
                        ))
                    }
                    scripts::SearchResult::App(m) => {
                        // Apps use their path as identifier
                        Some(ScriptInfo::new(
                            &m.app.name,
                            m.app.path.to_string_lossy().to_string(),
                        ))
                    }
                    scripts::SearchResult::Window(m) => {
                        // Windows use their id as identifier
                        Some(ScriptInfo::new(
                            &m.window.title,
                            format!("window:{}", m.window.id),
                        ))
                    }
                    scripts::SearchResult::Agent(m) => {
                        // Agents use their path as identifier
                        Some(ScriptInfo::new(
                            &m.agent.name,
                            format!("agent:{}", m.agent.path.to_string_lossy()),
                        ))
                    }
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    fn render_actions_dialog(&mut self, cx: &mut Context<Self>) -> AnyElement {
        // Use design tokens for GLOBAL theming
        let tokens = get_tokens(self.current_design);
        let design_colors = tokens.colors();
        let design_spacing = tokens.spacing();
        let design_typography = tokens.typography();
        let design_visual = tokens.visual();

        // Use design tokens for global theming
        let opacity = self.theme.get_opacity();
        let bg_hex = design_colors.background;
        let bg_with_alpha = self.hex_to_rgba_with_opacity(bg_hex, opacity.main);
        let box_shadows = self.create_box_shadows();

        // Key handler for actions dialog
        let handle_key = cx.listener(
            move |this: &mut Self,
                  event: &gpui::KeyDownEvent,
                  _window: &mut Window,
                  cx: &mut Context<Self>| {
                // Global shortcuts (Cmd+W, ESC closes window from ActionsDialog too)
                // ActionsDialog has no other key handling, so we just call the global handler
                let _ = this.handle_global_shortcut_with_options(event, true, cx);
            },
        );

        // Simple actions dialog stub with design tokens
        div()
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .bg(rgba(bg_with_alpha))
            .shadow(box_shadows)
            .rounded(px(design_visual.radius_lg))
            .p(px(design_spacing.padding_xl))
            .text_color(rgb(design_colors.text_primary))
            .font_family(design_typography.font_family)
            .key_context("actions_dialog")
            .track_focus(&self.focus_handle)
            .on_key_down(handle_key)
            .child(div().text_lg().child("Actions (Cmd+K)"))
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(design_colors.text_muted))
                    .mt(px(design_spacing.margin_md))
                    .child("• Create script\n• Edit script\n• Reload\n• Settings\n• Quit"),
            )
            .child(
                div()
                    .mt(px(design_spacing.margin_lg))
                    .text_xs()
                    .text_color(rgb(design_colors.text_dimmed))
                    .child("Press Esc to close"),
            )
            .into_any_element()
    }
}

/// Helper function to render a group header style item with actual visual styling
fn render_group_header_item(
    ix: usize,
    is_selected: bool,
    style: &designs::group_header_variations::GroupHeaderStyle,
    spacing: &designs::DesignSpacing,
    typography: &designs::DesignTypography,
    visual: &designs::DesignVisual,
    colors: &designs::DesignColors,
) -> AnyElement {
    use designs::group_header_variations::GroupHeaderStyle;

    let name_owned = style.name().to_string();
    let desc_owned = style.description().to_string();

    let mut item_div = div()
        .id(ElementId::NamedInteger("gallery-header".into(), ix as u64))
        .w_full()
        .h(px(LIST_ITEM_HEIGHT))
        .px(px(spacing.padding_lg))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(spacing.gap_md));

    if is_selected {
        item_div = item_div.bg(rgb(colors.background_selected));
    }

    // Create the preview element based on the style
    let preview = match style {
        // Text Only styles - vary font weight and style
        GroupHeaderStyle::UppercaseLeft => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .items_center()
            .px(px(8.0))
            .text_xs()
            .font_weight(gpui::FontWeight::SEMIBOLD)
            .text_color(rgb(colors.text_secondary))
            .child("MAIN"),
        GroupHeaderStyle::UppercaseCenter => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .items_center()
            .justify_center()
            .text_xs()
            .font_weight(gpui::FontWeight::SEMIBOLD)
            .text_color(rgb(colors.text_secondary))
            .child("MAIN"),
        GroupHeaderStyle::SmallCapsLeft => {
            div()
                .w(px(140.0))
                .h(px(28.0))
                .rounded(px(visual.radius_sm))
                .bg(rgba((colors.background_secondary << 8) | 0x60))
                .flex()
                .items_center()
                .px(px(8.0))
                .text_xs()
                .font_weight(gpui::FontWeight::MEDIUM)
                .text_color(rgb(colors.text_secondary))
                .child("MAIN") // Would use font-variant: small-caps if available
        }
        GroupHeaderStyle::BoldLeft => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .items_center()
            .px(px(8.0))
            .text_xs()
            .font_weight(gpui::FontWeight::BOLD)
            .text_color(rgb(colors.text_primary))
            .child("MAIN"),
        GroupHeaderStyle::LightLeft => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .items_center()
            .px(px(8.0))
            .text_xs()
            .font_weight(gpui::FontWeight::LIGHT)
            .text_color(rgb(colors.text_muted))
            .child("MAIN"),
        GroupHeaderStyle::MonospaceLeft => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .items_center()
            .px(px(8.0))
            .text_xs()
            .font_family(typography.font_family_mono)
            .text_color(rgb(colors.text_secondary))
            .child("MAIN"),

        // With Lines styles
        GroupHeaderStyle::LineLeft => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .flex_row()
            .items_center()
            .gap(px(6.0))
            .px(px(8.0))
            .child(div().w(px(24.0)).h(px(1.0)).bg(rgb(colors.border)))
            .child(
                div()
                    .text_xs()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(rgb(colors.text_secondary))
                    .child("MAIN"),
            ),
        GroupHeaderStyle::LineRight => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .flex_row()
            .items_center()
            .gap(px(6.0))
            .px(px(8.0))
            .child(
                div()
                    .text_xs()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(rgb(colors.text_secondary))
                    .child("MAIN"),
            )
            .child(div().flex_1().h(px(1.0)).bg(rgb(colors.border))),
        GroupHeaderStyle::LineBothSides => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .flex_row()
            .items_center()
            .gap(px(6.0))
            .px(px(8.0))
            .child(div().flex_1().h(px(1.0)).bg(rgb(colors.border)))
            .child(
                div()
                    .text_xs()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(rgb(colors.text_secondary))
                    .child("MAIN"),
            )
            .child(div().flex_1().h(px(1.0)).bg(rgb(colors.border))),
        GroupHeaderStyle::LineBelow => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .flex_col()
            .justify_center()
            .px(px(8.0))
            .gap(px(2.0))
            .child(
                div()
                    .text_xs()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(rgb(colors.text_secondary))
                    .child("MAIN"),
            )
            .child(div().w(px(40.0)).h(px(1.0)).bg(rgb(colors.border))),
        GroupHeaderStyle::LineAbove => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .flex_col()
            .justify_center()
            .px(px(8.0))
            .gap(px(2.0))
            .child(div().w(px(40.0)).h(px(1.0)).bg(rgb(colors.border)))
            .child(
                div()
                    .text_xs()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(rgb(colors.text_secondary))
                    .child("MAIN"),
            ),
        GroupHeaderStyle::DoubleLine => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .flex_col()
            .justify_center()
            .items_center()
            .gap(px(1.0))
            .child(div().w(px(100.0)).h(px(1.0)).bg(rgb(colors.border)))
            .child(
                div()
                    .text_xs()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(rgb(colors.text_secondary))
                    .child("MAIN"),
            )
            .child(div().w(px(100.0)).h(px(1.0)).bg(rgb(colors.border))),

        // With Background styles
        GroupHeaderStyle::PillBackground => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .items_center()
            .px(px(8.0))
            .child(
                div()
                    .px(px(8.0))
                    .py(px(2.0))
                    .rounded(px(10.0))
                    .bg(rgba((colors.accent << 8) | 0x30))
                    .text_xs()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(rgb(colors.accent))
                    .child("MAIN"),
            ),
        GroupHeaderStyle::FullWidthBackground => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.accent << 8) | 0x20))
            .flex()
            .items_center()
            .px(px(8.0))
            .text_xs()
            .font_weight(gpui::FontWeight::SEMIBOLD)
            .text_color(rgb(colors.text_primary))
            .child("MAIN"),
        GroupHeaderStyle::SubtleBackground => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x90))
            .flex()
            .items_center()
            .px(px(8.0))
            .text_xs()
            .font_weight(gpui::FontWeight::MEDIUM)
            .text_color(rgb(colors.text_secondary))
            .child("MAIN"),
        GroupHeaderStyle::GradientFade => {
            // Simulated with opacity fade
            div()
                .w(px(140.0))
                .h(px(28.0))
                .rounded(px(visual.radius_sm))
                .bg(rgba((colors.background_secondary << 8) | 0x60))
                .flex()
                .items_center()
                .px(px(8.0))
                .child(
                    div()
                        .px(px(16.0))
                        .text_xs()
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .text_color(rgb(colors.text_secondary))
                        .child("~  MAIN  ~"),
                )
        }
        GroupHeaderStyle::BorderedBox => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .items_center()
            .px(px(8.0))
            .child(
                div()
                    .px(px(8.0))
                    .py(px(2.0))
                    .border_1()
                    .border_color(rgb(colors.border))
                    .rounded(px(2.0))
                    .text_xs()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(rgb(colors.text_secondary))
                    .child("MAIN"),
            ),

        // Minimal styles
        GroupHeaderStyle::DotPrefix => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .flex_row()
            .items_center()
            .gap(px(6.0))
            .px(px(8.0))
            .child(
                div()
                    .w(px(4.0))
                    .h(px(4.0))
                    .rounded(px(2.0))
                    .bg(rgb(colors.text_muted)),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(colors.text_secondary))
                    .child("MAIN"),
            ),
        GroupHeaderStyle::DashPrefix => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .items_center()
            .px(px(8.0))
            .text_xs()
            .text_color(rgb(colors.text_secondary))
            .child("- MAIN"),
        GroupHeaderStyle::BulletPrefix => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .flex_row()
            .items_center()
            .gap(px(6.0))
            .px(px(8.0))
            .child(
                div()
                    .w(px(6.0))
                    .h(px(6.0))
                    .rounded(px(3.0))
                    .bg(rgb(colors.accent)),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(colors.text_secondary))
                    .child("MAIN"),
            ),
        GroupHeaderStyle::ArrowPrefix => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .items_center()
            .px(px(8.0))
            .text_xs()
            .text_color(rgb(colors.text_secondary))
            .child("\u{25B8} MAIN"),
        GroupHeaderStyle::ChevronPrefix => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .items_center()
            .px(px(8.0))
            .text_xs()
            .text_color(rgb(colors.text_secondary))
            .child("\u{203A} MAIN"),
        GroupHeaderStyle::Dimmed => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .items_center()
            .px(px(8.0))
            .text_xs()
            .opacity(0.5)
            .text_color(rgb(colors.text_muted))
            .child("MAIN"),

        // Decorative styles
        GroupHeaderStyle::Bracketed => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .items_center()
            .px(px(8.0))
            .text_xs()
            .text_color(rgb(colors.text_secondary))
            .child("[MAIN]"),
        GroupHeaderStyle::Quoted => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .items_center()
            .px(px(8.0))
            .text_xs()
            .text_color(rgb(colors.text_secondary))
            .child("\"MAIN\""),
        GroupHeaderStyle::Tagged => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .items_center()
            .px(px(8.0))
            .child(
                div()
                    .px(px(6.0))
                    .py(px(1.0))
                    .bg(rgba((colors.accent << 8) | 0x40))
                    .rounded(px(2.0))
                    .text_xs()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(rgb(colors.accent))
                    .child("MAIN"),
            ),
        GroupHeaderStyle::Numbered => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .flex_row()
            .items_center()
            .gap(px(6.0))
            .px(px(8.0))
            .child(
                div()
                    .text_xs()
                    .font_weight(gpui::FontWeight::BOLD)
                    .text_color(rgb(colors.accent))
                    .child("01."),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(colors.text_secondary))
                    .child("MAIN"),
            ),
        GroupHeaderStyle::IconPrefix => div()
            .w(px(140.0))
            .h(px(28.0))
            .rounded(px(visual.radius_sm))
            .bg(rgba((colors.background_secondary << 8) | 0x60))
            .flex()
            .flex_row()
            .items_center()
            .gap(px(6.0))
            .px(px(8.0))
            .child(
                div()
                    .w(px(8.0))
                    .h(px(8.0))
                    .bg(rgb(colors.accent))
                    .rounded(px(1.0)),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(colors.text_secondary))
                    .child("MAIN"),
            ),
    };

    item_div
        // Preview element
        .child(preview)
        // Name and description
        .child(
            div()
                .flex_1()
                .flex()
                .flex_col()
                .gap(px(2.0))
                .child(
                    div()
                        .text_sm()
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .text_color(rgb(colors.text_primary))
                        .child(name_owned),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(colors.text_muted))
                        .child(desc_owned),
                ),
        )
        .into_any_element()
}

</file>

<file path="src/app_actions.rs">
// Actions handling methods - extracted from app_impl.rs
// This file is included via include!() macro in main.rs
// Contains: handle_action, trigger_action_by_name

impl ScriptListApp {
    /// Handle action selection from the actions dialog
    fn handle_action(&mut self, action_id: String, cx: &mut Context<Self>) {
        logging::log("UI", &format!("Action selected: {}", action_id));

        // Close the dialog and return to script list
        self.current_view = AppView::ScriptList;

        match action_id.as_str() {
            "create_script" => {
                logging::log("UI", "Create script action - opening scripts folder");
                // Open ~/.sk/kit/scripts/ in Finder for now (future: create script dialog)
                let scripts_dir = shellexpand::tilde("~/.sk/kit/scripts").to_string();
                std::thread::spawn(move || {
                    use std::process::Command;
                    match Command::new("open").arg(&scripts_dir).spawn() {
                        Ok(_) => {
                            logging::log("UI", &format!("Opened scripts folder: {}", scripts_dir))
                        }
                        Err(e) => {
                            logging::log("ERROR", &format!("Failed to open scripts folder: {}", e))
                        }
                    }
                });
                self.last_output = Some(SharedString::from("Opened scripts folder"));
                // Hide window after opening folder and set reset flag
                script_kit_gpui::set_main_window_visible(false);
                NEEDS_RESET.store(true, Ordering::SeqCst);
                cx.hide();
            }
            "run_script" => {
                logging::log("UI", "Run script action");
                self.execute_selected(cx);
            }
            "view_logs" => {
                logging::log("UI", "View logs action");
                self.toggle_logs(cx);
            }
            "reveal_in_finder" => {
                logging::log("UI", "Reveal in Finder action");
                if let Some(result) = self.get_selected_result() {
                    match result {
                        scripts::SearchResult::Script(script_match) => {
                            let path_str = script_match.script.path.to_string_lossy().to_string();
                            std::thread::spawn(move || {
                                use std::process::Command;
                                match Command::new("open").arg("-R").arg(&path_str).spawn() {
                                    Ok(_) => logging::log(
                                        "UI",
                                        &format!("Revealed in Finder: {}", path_str),
                                    ),
                                    Err(e) => logging::log(
                                        "ERROR",
                                        &format!("Failed to reveal in Finder: {}", e),
                                    ),
                                }
                            });
                            self.last_output = Some(SharedString::from("Revealed in Finder"));
                            // Hide window after revealing in Finder and set reset flag
                            script_kit_gpui::set_main_window_visible(false);
                            NEEDS_RESET.store(true, Ordering::SeqCst);
                            cx.hide();
                        }
                        scripts::SearchResult::Scriptlet(_) => {
                            self.last_output =
                                Some(SharedString::from("Cannot reveal scriptlets in Finder"));
                        }
                        scripts::SearchResult::BuiltIn(_) => {
                            self.last_output =
                                Some(SharedString::from("Cannot reveal built-in features"));
                        }
                        scripts::SearchResult::App(app_match) => {
                            let path_str = app_match.app.path.to_string_lossy().to_string();
                            std::thread::spawn(move || {
                                use std::process::Command;
                                match Command::new("open").arg("-R").arg(&path_str).spawn() {
                                    Ok(_) => logging::log(
                                        "UI",
                                        &format!("Revealed app in Finder: {}", path_str),
                                    ),
                                    Err(e) => logging::log(
                                        "ERROR",
                                        &format!("Failed to reveal app in Finder: {}", e),
                                    ),
                                }
                            });
                            self.last_output = Some(SharedString::from("Revealed app in Finder"));
                            // Hide window after revealing app in Finder and set reset flag
                            script_kit_gpui::set_main_window_visible(false);
                            NEEDS_RESET.store(true, Ordering::SeqCst);
                            cx.hide();
                        }
                        scripts::SearchResult::Window(_) => {
                            self.last_output =
                                Some(SharedString::from("Cannot reveal windows in Finder"));
                        }
                        scripts::SearchResult::Agent(agent_match) => {
                            let path_str = agent_match.agent.path.to_string_lossy().to_string();
                            std::thread::spawn(move || {
                                use std::process::Command;
                                match Command::new("open").arg("-R").arg(&path_str).spawn() {
                                    Ok(_) => logging::log(
                                        "UI",
                                        &format!("Revealed agent in Finder: {}", path_str),
                                    ),
                                    Err(e) => logging::log(
                                        "ERROR",
                                        &format!("Failed to reveal agent in Finder: {}", e),
                                    ),
                                }
                            });
                            self.last_output = Some(SharedString::from("Revealed agent in Finder"));
                            script_kit_gpui::set_main_window_visible(false);
                            NEEDS_RESET.store(true, Ordering::SeqCst);
                            cx.hide();
                        }
                    }
                } else {
                    self.last_output = Some(SharedString::from("No item selected"));
                }
            }
            "copy_path" => {
                logging::log("UI", "Copy path action");
                if let Some(result) = self.get_selected_result() {
                    let path_opt = match result {
                        scripts::SearchResult::Script(script_match) => {
                            Some(script_match.script.path.to_string_lossy().to_string())
                        }
                        scripts::SearchResult::App(app_match) => {
                            Some(app_match.app.path.to_string_lossy().to_string())
                        }
                        scripts::SearchResult::Scriptlet(_) => {
                            self.last_output =
                                Some(SharedString::from("Cannot copy scriptlet path"));
                            None
                        }
                        scripts::SearchResult::BuiltIn(_) => {
                            self.last_output =
                                Some(SharedString::from("Cannot copy built-in path"));
                            None
                        }
                        scripts::SearchResult::Window(_) => {
                            self.last_output = Some(SharedString::from("Cannot copy window path"));
                            None
                        }
                        scripts::SearchResult::Agent(agent_match) => {
                            Some(agent_match.agent.path.to_string_lossy().to_string())
                        }
                    };

                    if let Some(path_str) = path_opt {
                        // Use pbcopy on macOS for reliable clipboard access
                        #[cfg(target_os = "macos")]
                        {
                            use std::io::Write;
                            use std::process::{Command, Stdio};

                            match Command::new("pbcopy").stdin(Stdio::piped()).spawn() {
                                Ok(mut child) => {
                                    if let Some(ref mut stdin) = child.stdin {
                                        if stdin.write_all(path_str.as_bytes()).is_ok() {
                                            let _ = child.wait();
                                            logging::log(
                                                "UI",
                                                &format!("Copied path to clipboard: {}", path_str),
                                            );
                                            self.last_output = Some(SharedString::from(format!(
                                                "Copied: {}",
                                                path_str
                                            )));
                                        } else {
                                            logging::log(
                                                "ERROR",
                                                "Failed to write to pbcopy stdin",
                                            );
                                            self.last_output =
                                                Some(SharedString::from("Failed to copy path"));
                                        }
                                    }
                                }
                                Err(e) => {
                                    logging::log(
                                        "ERROR",
                                        &format!("Failed to spawn pbcopy: {}", e),
                                    );
                                    self.last_output =
                                        Some(SharedString::from("Failed to copy path"));
                                }
                            }
                        }

                        // Fallback for non-macOS platforms
                        #[cfg(not(target_os = "macos"))]
                        {
                            use arboard::Clipboard;
                            match Clipboard::new() {
                                Ok(mut clipboard) => match clipboard.set_text(&path_str) {
                                    Ok(_) => {
                                        logging::log(
                                            "UI",
                                            &format!("Copied path to clipboard: {}", path_str),
                                        );
                                        self.last_output = Some(SharedString::from(format!(
                                            "Copied: {}",
                                            path_str
                                        )));
                                    }
                                    Err(e) => {
                                        logging::log(
                                            "ERROR",
                                            &format!("Failed to copy path: {}", e),
                                        );
                                        self.last_output =
                                            Some(SharedString::from("Failed to copy path"));
                                    }
                                },
                                Err(e) => {
                                    logging::log(
                                        "ERROR",
                                        &format!("Failed to access clipboard: {}", e),
                                    );
                                    self.last_output =
                                        Some(SharedString::from("Failed to access clipboard"));
                                }
                            }
                        }
                    }
                } else {
                    self.last_output = Some(SharedString::from("No item selected"));
                }
            }
            "edit_script" => {
                logging::log("UI", "Edit script action");
                if let Some(result) = self.get_selected_result() {
                    match result {
                        scripts::SearchResult::Script(script_match) => {
                            self.edit_script(&script_match.script.path);
                            // Hide window after opening editor and set reset flag
                            script_kit_gpui::set_main_window_visible(false);
                            NEEDS_RESET.store(true, Ordering::SeqCst);
                            cx.hide();
                        }
                        scripts::SearchResult::Scriptlet(_) => {
                            self.last_output = Some(SharedString::from("Cannot edit scriptlets"));
                        }
                        scripts::SearchResult::BuiltIn(_) => {
                            self.last_output =
                                Some(SharedString::from("Cannot edit built-in features"));
                        }
                        scripts::SearchResult::App(_) => {
                            self.last_output = Some(SharedString::from("Cannot edit applications"));
                        }
                        scripts::SearchResult::Window(_) => {
                            self.last_output = Some(SharedString::from("Cannot edit windows"));
                        }
                        scripts::SearchResult::Agent(agent_match) => {
                            self.edit_script(&agent_match.agent.path);
                            // Hide window after opening editor and set reset flag
                            script_kit_gpui::set_main_window_visible(false);
                            NEEDS_RESET.store(true, Ordering::SeqCst);
                            cx.hide();
                        }
                    }
                } else {
                    self.last_output = Some(SharedString::from("No script selected"));
                }
            }
            "reload_scripts" => {
                logging::log("UI", "Reload scripts action");
                self.refresh_scripts(cx);
                self.last_output = Some(SharedString::from("Scripts reloaded"));
            }
            "settings" => {
                logging::log("UI", "Settings action");
                self.last_output = Some(SharedString::from("Settings (TODO)"));
            }
            "quit" => {
                logging::log("UI", "Quit action");
                // Clean up processes and PID file before quitting
                PROCESS_MANAGER.kill_all_processes();
                PROCESS_MANAGER.remove_main_pid();
                cx.quit();
            }
            "__cancel__" => {
                logging::log("UI", "Actions dialog cancelled");
            }
            _ => {
                // Check if this is an SDK action with has_action=true
                if let Some(ref actions) = self.sdk_actions {
                    if let Some(action) = actions.iter().find(|a| a.name == action_id) {
                        if action.has_action {
                            // Send ActionTriggered back to SDK
                            logging::log(
                                "ACTIONS",
                                &format!(
                                    "SDK action with handler: '{}' (has_action=true), sending ActionTriggered",
                                    action_id
                                ),
                            );
                            if let Some(ref sender) = self.response_sender {
                                let msg = protocol::Message::action_triggered(
                                    action_id.clone(),
                                    action.value.clone(),
                                    self.arg_input.text().to_string(),
                                );
                                if let Err(e) = sender.send(msg) {
                                    logging::log(
                                        "ERROR",
                                        &format!("Failed to send ActionTriggered: {}", e),
                                    );
                                }
                            }
                        } else if let Some(ref value) = action.value {
                            // Submit value directly (has_action=false with value)
                            logging::log(
                                "ACTIONS",
                                &format!(
                                    "SDK action without handler: '{}' (has_action=false), submitting value: {:?}",
                                    action_id, value
                                ),
                            );
                            if let Some(ref sender) = self.response_sender {
                                let msg = protocol::Message::Submit {
                                    id: "action".to_string(),
                                    value: Some(value.clone()),
                                };
                                if let Err(e) = sender.send(msg) {
                                    logging::log("ERROR", &format!("Failed to send Submit: {}", e));
                                }
                            }
                        } else {
                            logging::log(
                                "ACTIONS",
                                &format!(
                                    "SDK action '{}' has no value and has_action=false",
                                    action_id
                                ),
                            );
                        }
                    } else {
                        logging::log("UI", &format!("Unknown action: {}", action_id));
                    }
                } else {
                    logging::log("UI", &format!("Unknown action: {}", action_id));
                }
            }
        }

        cx.notify();
    }

    /// Trigger an SDK action by name
    /// Returns true if the action was found and triggered
    fn trigger_action_by_name(&mut self, action_name: &str, cx: &mut Context<Self>) -> bool {
        if let Some(ref actions) = self.sdk_actions {
            if let Some(action) = actions.iter().find(|a| a.name == action_name) {
                logging::log(
                    "ACTIONS",
                    &format!(
                        "Triggering SDK action '{}' via shortcut (has_action={})",
                        action_name, action.has_action
                    ),
                );

                if action.has_action {
                    // Send ActionTriggered back to SDK
                    if let Some(ref sender) = self.response_sender {
                        let msg = protocol::Message::action_triggered(
                            action_name.to_string(),
                            action.value.clone(),
                            self.arg_input.text().to_string(),
                        );
                        if let Err(e) = sender.send(msg) {
                            logging::log(
                                "ERROR",
                                &format!("Failed to send ActionTriggered: {}", e),
                            );
                        }
                    }
                } else if let Some(ref value) = action.value {
                    // Submit value directly
                    if let Some(ref sender) = self.response_sender {
                        let msg = protocol::Message::Submit {
                            id: "action".to_string(),
                            value: Some(value.clone()),
                        };
                        if let Err(e) = sender.send(msg) {
                            logging::log("ERROR", &format!("Failed to send Submit: {}", e));
                        }
                    }
                }

                cx.notify();
                return true;
            }
        }
        false
    }
}

</file>

<file path="src/lib.rs">
#![allow(unexpected_cfgs)]

//! Script Kit GPUI - A GPUI-based launcher for Script Kit
//!
//! This library provides the core functionality for executing scripts
//! with bidirectional JSONL communication.

pub mod components;
pub mod config;
pub mod debug_grid;
pub mod designs;
pub mod editor;
pub mod error;
pub mod executor;
pub mod form_prompt;
pub mod hotkeys;
pub mod list_item;
pub mod logging;
pub mod navigation;
pub mod panel;
pub mod perf;
pub mod platform;
pub mod prompts;
pub mod protocol;
pub mod scripts;
pub mod selected_text;
pub mod shortcuts;
pub mod syntax;
pub mod term_prompt;
pub mod terminal;
pub mod theme;
pub mod toast_manager;
#[cfg(not(test))]
pub mod tray;
pub mod utils;
pub mod warning_banner;
pub mod window_manager;
pub mod window_resize;

// Phase 1 system API modules
pub mod clipboard_history;
pub mod file_search;
pub mod window_control;

// System actions - macOS AppleScript-based system commands
#[cfg(target_os = "macos")]
pub mod system_actions;

// Script creation - Create new scripts and scriptlets
pub mod script_creation;

// Permissions wizard - Check and request macOS permissions
pub mod permissions_wizard;

// Menu bar reader - macOS Accessibility API for reading app menus
// Provides get_frontmost_menu_bar() with recursive parsing up to 3 levels
#[cfg(target_os = "macos")]
pub mod menu_bar;

// Menu executor - Execute menu actions via Accessibility API
// Navigates AX hierarchy and performs AXPress on menu items
#[cfg(target_os = "macos")]
pub mod menu_executor;

// Menu cache - SQLite-backed menu bar data caching
// Caches application menu hierarchies by bundle_id to avoid expensive rescans
#[cfg(target_os = "macos")]
pub mod menu_cache;

// Frontmost app tracker - Background observer for tracking active application
// Pre-fetches menu bar items when apps activate (before Script Kit opens)
#[cfg(target_os = "macos")]
pub mod frontmost_app_tracker;

// Built-in features registry
pub mod app_launcher;
pub mod builtins;

// Frecency tracking for script usage
pub mod frecency;

// Process management for tracking bun script processes
pub mod process_manager;

// Scriptlet parsing and variable substitution
pub mod scriptlets;

// Scriptlet cache for tracking per-file state with change detection
// Used by file watchers to diff scriptlet changes and update registrations incrementally
pub mod scriptlet_cache;

// Typed metadata parser for new `metadata = {}` global syntax
pub mod metadata_parser;

// Schema parser for `schema = { input: {}, output: {} }` definitions
pub mod schema_parser;

// Scriptlet codefence metadata parser for ```metadata and ```schema blocks
pub mod scriptlet_metadata;

// VSCode snippet syntax parser for template() SDK function
pub mod snippet;

// HTML form parsing for form() prompt
pub mod form_parser;

// Centralized template variable substitution system
// Used by expand_manager, template prompts, and future template features
pub mod template_variables;

// Text injection for text expansion/snippet systems
pub mod text_injector;

// Expand trigger matching for text expansion
pub mod expand_matcher;

// Global keyboard monitoring for system-wide keystroke capture
// Required for text expansion triggers typed in any application
#[cfg(target_os = "macos")]
pub mod keyboard_monitor;

// Expand manager - ties together keyboard monitoring, trigger matching,
// and text injection for the complete text expansion system
#[cfg(target_os = "macos")]
pub mod expand_manager;

// OCR module - macOS Vision framework integration
#[cfg(feature = "ocr")]
pub mod ocr;

// Script scheduling with cron expressions and natural language
pub mod scheduler;

// Kenv environment setup and initialization
// Ensures ~/.sk/kit exists with required directories and starter files
pub mod setup;

// Storybook - Component preview system for development
pub mod storybook;

// Stories - Component story definitions for the storybook
pub mod stories;

// MCP Server - HTTP server for Model Context Protocol integration
// Provides localhost:43210 endpoint with Bearer token auth
pub mod mcp_server;

// MCP Streaming - Server-Sent Events (SSE) and audit logging
// Provides real-time event streaming and tool call audit logs
pub mod mcp_streaming;

// MCP Protocol - JSON-RPC 2.0 protocol handler for MCP
// Handles request parsing, method routing, and response generation
pub mod mcp_protocol;

// MCP Kit Tools - kit/* namespace tools for app control
// Provides kit/show, kit/hide, kit/state tools
pub mod mcp_kit_tools;

// MCP Script Tools - scripts/* namespace auto-generated tools
// Scripts with schema.input become MCP tools automatically
pub mod mcp_script_tools;

// MCP Resources - read-only data resources for MCP clients
// Provides kit://state, scripts://, and scriptlets:// resources
pub mod mcp_resources;

// Stdin commands - external command handling via stdin
// Provides JSON command protocol for testing and automation
pub mod stdin_commands;

// Notes - Raycast Notes feature parity
// Separate floating window for note-taking with gpui-component
pub mod notes;

// AI Chat - Separate floating window for AI conversations
// BYOK (Bring Your Own Key) with SQLite storage at ~/.sk/kit/ai-chats.db
pub mod ai;

// Agents - mdflow agent integration
// Executable markdown prompts that run against Claude, Gemini, Codex, or Copilot
// Located in ~/.sk/kit/*/agents/*.md
pub mod agents;

// macOS launch-at-login via SMAppService
// Uses SMAppService on macOS 13+ for modern login item management
pub mod login_item;

// UI transitions/animations (self-contained module, no external crate dependency)
// Provides TransitionColor, Opacity, SlideOffset, AppearTransition, HoverState
// and easing functions (ease_out_quad, ease_in_quad, etc.)
// Used for smooth hover effects, toast animations, and other UI transitions
pub mod transitions;

// File watchers for theme, config, scripts, and system appearance
pub mod watcher;

// Window state management tests - code audit to prevent regressions
// Verifies that app_execute.rs uses close_and_reset_window() correctly
#[cfg(test)]
mod window_state_tests;

// Shared window visibility state
// Used to track main window visibility across the app
// Notes/AI windows use this to decide whether to hide the app after closing
use std::sync::atomic::{AtomicBool, Ordering};

/// Global state tracking whether the main window is visible
/// - Used by hotkey toggle to show/hide main window
/// - Used by Notes/AI to prevent main window from appearing when they close
pub static MAIN_WINDOW_VISIBLE: AtomicBool = AtomicBool::new(false);

/// Check if the main window is currently visible
pub fn is_main_window_visible() -> bool {
    MAIN_WINDOW_VISIBLE.load(Ordering::SeqCst)
}

/// Set the main window visibility state
pub fn set_main_window_visible(visible: bool) {
    MAIN_WINDOW_VISIBLE.store(visible, Ordering::SeqCst);
}

</file>

<file path="src/app_navigation.rs">
// App navigation methods - extracted from app_impl.rs
// This file is included via include!() macro in main.rs
// Contains: move_selection_up, move_selection_down, scroll_to_selected, etc.

impl ScriptListApp {
    fn move_selection_up(&mut self, cx: &mut Context<Self>) {
        // Clear pending confirmation when changing selection
        if self.pending_confirmation.is_some() {
            self.pending_confirmation = None;
        }

        // Get grouped results to check for section headers (cached)
        let (grouped_items, _) = self.get_grouped_results_cached();
        // Clone to avoid borrow issues with self mutation below
        let grouped_items = grouped_items.clone();

        // Find the first selectable (non-header) item index
        let first_selectable = grouped_items
            .iter()
            .position(|item| matches!(item, GroupedListItem::Item(_)));

        // If already at or before first selectable, can't go further up
        if let Some(first) = first_selectable {
            if self.selected_index <= first {
                // Already at the first selectable item, stay here
                return;
            }
        }

        if self.selected_index > 0 {
            let mut new_index = self.selected_index - 1;

            // Skip section headers when moving up
            while new_index > 0 {
                if let Some(GroupedListItem::SectionHeader(_)) = grouped_items.get(new_index) {
                    new_index -= 1;
                } else {
                    break;
                }
            }

            // Make sure we didn't land on a section header at index 0
            if let Some(GroupedListItem::SectionHeader(_)) = grouped_items.get(new_index) {
                // Stay at current position if we can't find a valid item
                return;
            }

            self.selected_index = new_index;
            self.scroll_to_selected_if_needed("keyboard_up");
            self.trigger_scroll_activity(cx);
            cx.notify();
        }
    }

    fn move_selection_down(&mut self, cx: &mut Context<Self>) {
        // Clear pending confirmation when changing selection
        if self.pending_confirmation.is_some() {
            self.pending_confirmation = None;
        }

        // Get grouped results to check for section headers (cached)
        let (grouped_items, _) = self.get_grouped_results_cached();
        // Clone to avoid borrow issues with self mutation below
        let grouped_items = grouped_items.clone();

        let item_count = grouped_items.len();

        // Find the last selectable (non-header) item index
        let last_selectable = grouped_items
            .iter()
            .rposition(|item| matches!(item, GroupedListItem::Item(_)));

        // If already at or after last selectable, can't go further down
        if let Some(last) = last_selectable {
            if self.selected_index >= last {
                // Already at the last selectable item, stay here
                return;
            }
        }

        if self.selected_index < item_count.saturating_sub(1) {
            let mut new_index = self.selected_index + 1;

            // Skip section headers when moving down
            while new_index < item_count.saturating_sub(1) {
                if let Some(GroupedListItem::SectionHeader(_)) = grouped_items.get(new_index) {
                    new_index += 1;
                } else {
                    break;
                }
            }

            // Make sure we didn't land on a section header at the end
            if let Some(GroupedListItem::SectionHeader(_)) = grouped_items.get(new_index) {
                // Stay at current position if we can't find a valid item
                return;
            }

            self.selected_index = new_index;
            self.scroll_to_selected_if_needed("keyboard_down");
            self.trigger_scroll_activity(cx);
            cx.notify();
        }
    }

    /// Scroll stabilization helper: only call scroll_to_reveal_item if we haven't already scrolled to this index.
    /// This prevents scroll jitter from redundant scroll calls.
    ///
    /// NOTE: Uses main_list_state (ListState) for the variable-height list() component,
    /// not the legacy list_scroll_handle (UniformListScrollHandle).
    fn scroll_to_selected_if_needed(&mut self, _reason: &str) {
        let target = self.selected_index;

        // Check if we've already scrolled to this index
        if self.last_scrolled_index == Some(target) {
            return;
        }

        // Use perf guard for scroll timing
        let _scroll_perf = crate::perf::ScrollPerfGuard::new();

        // Perform the scroll using ListState for variable-height list
        // This scrolls the actual list() component used in render_script_list
        self.main_list_state.scroll_to_reveal_item(target);
        self.last_scrolled_index = Some(target);
    }

    /// Trigger scroll activity - shows the scrollbar and schedules fade-out
    ///
    /// This should be called whenever scroll-related activity occurs:
    /// - Keyboard up/down navigation
    /// - scroll_to_item calls
    /// - Mouse wheel scrolling (if tracked)
    fn trigger_scroll_activity(&mut self, cx: &mut Context<Self>) {
        self.is_scrolling = true;
        self.last_scroll_time = Some(std::time::Instant::now());

        // Schedule fade-out after 1000ms of inactivity
        cx.spawn(async move |this, cx| {
            Timer::after(std::time::Duration::from_millis(1000)).await;
            let _ = cx.update(|cx| {
                this.update(cx, |app, cx| {
                    // Only hide if no new scroll activity occurred
                    if let Some(last_time) = app.last_scroll_time {
                        if last_time.elapsed() >= std::time::Duration::from_millis(1000) {
                            app.is_scrolling = false;
                            cx.notify();
                        }
                    }
                })
            });
        })
        .detach();

        cx.notify();
    }

    /// Apply a coalesced navigation delta in the given direction
    fn apply_nav_delta(&mut self, dir: NavDirection, delta: i32, cx: &mut Context<Self>) {
        let signed = match dir {
            NavDirection::Up => -delta,
            NavDirection::Down => delta,
        };
        self.move_selection_by(signed, cx);
    }

    /// Move selection by a signed delta (positive = down, negative = up)
    /// Used by NavCoalescer for batched movements
    ///
    /// IMPORTANT: This must use grouped results and skip section headers,
    /// just like move_selection_up/down. Otherwise, holding arrow keys
    /// can land on headers causing navigation to feel "stuck".
    fn move_selection_by(&mut self, delta: i32, cx: &mut Context<Self>) {
        // Clear pending confirmation when changing selection
        if self.pending_confirmation.is_some() {
            self.pending_confirmation = None;
        }

        // Get grouped results to check for section headers (cached)
        let (grouped_items, _) = self.get_grouped_results_cached();
        // Clone to avoid borrow issues with self mutation below
        let grouped_items = grouped_items.clone();

        let len = grouped_items.len();
        if len == 0 {
            self.selected_index = 0;
            return;
        }

        // Find bounds for selectable items (non-headers)
        let first_selectable = grouped_items
            .iter()
            .position(|item| matches!(item, GroupedListItem::Item(_)));
        let last_selectable = grouped_items
            .iter()
            .rposition(|item| matches!(item, GroupedListItem::Item(_)));

        // If no selectable items, nothing to do
        let (first, last) = match (first_selectable, last_selectable) {
            (Some(f), Some(l)) => (f, l),
            _ => return,
        };

        // Calculate target index, clamping to valid range
        let target = (self.selected_index as i32 + delta).clamp(first as i32, last as i32) as usize;

        // If moving down (positive delta), skip headers forward
        // If moving up (negative delta), skip headers backward
        let new_index = if delta > 0 {
            // Moving down - find next non-header at or after target
            let mut idx = target;
            while idx <= last {
                if matches!(grouped_items.get(idx), Some(GroupedListItem::Item(_))) {
                    break;
                }
                idx += 1;
            }
            idx.min(last)
        } else if delta < 0 {
            // Moving up - find next non-header at or before target
            let mut idx = target;
            while idx >= first {
                if matches!(grouped_items.get(idx), Some(GroupedListItem::Item(_))) {
                    break;
                }
                if idx == 0 {
                    break;
                }
                idx -= 1;
            }
            idx.max(first)
        } else {
            // delta == 0, no movement
            self.selected_index
        };

        // Final validation: ensure we're not on a header
        if matches!(grouped_items.get(new_index), Some(GroupedListItem::SectionHeader(_))) {
            // Can't find a valid position, stay put
            return;
        }

        if new_index != self.selected_index {
            self.selected_index = new_index;
            self.scroll_to_selected_if_needed("coalesced_nav");
            self.trigger_scroll_activity(cx);
            cx.notify();
        }
    }

    /// Handle mouse wheel scroll events by converting to item-based scrolling.
    ///
    /// This bypasses GPUI's pixel-based scroll which has height calculation issues
    /// with variable-height items. Instead, we convert the scroll delta to item
    /// indices and use scroll_to_reveal_item() like keyboard navigation does.
    ///
    /// # Arguments
    /// * `delta_lines` - Scroll delta in "lines" (positive = scroll content up/view down)
    #[allow(dead_code)]
    pub fn handle_scroll_wheel(&mut self, delta_lines: f32, cx: &mut Context<Self>) {
        // Get current scroll position from ListState
        let current_item = self.main_list_state.logical_scroll_top().item_ix;

        // Get grouped results to find valid bounds
        let (grouped_items, _) = self.get_grouped_results_cached();
        let grouped_items = grouped_items.clone();
        let max_item = grouped_items.len().saturating_sub(1);

        // Convert delta to items (negative delta = scroll down in content = move to higher indices)
        // Round to avoid tiny scrolls being ignored
        let items_to_scroll = (-delta_lines).round() as i32;

        // Calculate new target item, clamping to valid range
        let new_item = (current_item as i32 + items_to_scroll).clamp(0, max_item as i32) as usize;

        tracing::debug!(
            target: "SCROLL_STATE",
            delta_lines,
            current_item,
            new_item,
            items_to_scroll,
            "Mouse wheel scroll"
        );

        // Only scroll if we're moving to a different item
        if new_item != current_item {
            self.main_list_state.scroll_to_reveal_item(new_item);
            self.trigger_scroll_activity(cx);
            cx.notify();
        }
    }

    /// Ensure the navigation flush task is running. Spawns a background task
    /// that periodically flushes pending navigation deltas.
    fn ensure_nav_flush_task(&mut self, cx: &mut Context<Self>) {
        if self.nav_coalescer.flush_task_running {
            return;
        }
        self.nav_coalescer.flush_task_running = true;
        cx.spawn(async move |this, cx| {
            loop {
                Timer::after(NavCoalescer::WINDOW).await;
                let keep_running = cx
                    .update(|cx| {
                        this.update(cx, |this, cx| {
                            // Flush any pending navigation delta
                            if let Some((dir, delta)) = this.nav_coalescer.flush_pending() {
                                this.apply_nav_delta(dir, delta, cx);
                            }
                            // Check if we should keep running
                            let now = std::time::Instant::now();
                            let recently_active = now.duration_since(this.nav_coalescer.last_event)
                                < NavCoalescer::WINDOW;
                            if !recently_active && this.nav_coalescer.pending_delta == 0 {
                                // No recent activity and no pending delta - stop the task
                                this.nav_coalescer.flush_task_running = false;
                                this.nav_coalescer.reset();
                                false
                            } else {
                                true
                            }
                        })
                    })
                    .unwrap_or(Ok(false))
                    .unwrap_or(false);
                if !keep_running {
                    break;
                }
            }
        })
        .detach();
    }
}

</file>

<file path="src/app_execute.rs">
// App execution methods - extracted from app_impl.rs
// This file is included via include!() macro in main.rs
// Contains: execute_builtin, execute_app, execute_window_focus

impl ScriptListApp {
    fn execute_builtin(&mut self, entry: &builtins::BuiltInEntry, cx: &mut Context<Self>) {
        logging::log(
            "EXEC",
            &format!("Executing built-in: {} (id: {})", entry.name, entry.id),
        );

        // Check if this command requires confirmation
        if self.config.requires_confirmation(&entry.id) {
            // Check if we're already in confirmation mode for this entry
            if self.pending_confirmation.as_ref() == Some(&entry.id) {
                // User confirmed - clear pending and proceed with execution
                logging::log("EXEC", &format!("Confirmed: {}", entry.id));
                self.pending_confirmation = None;
                // Fall through to execute
            } else {
                // First press - enter confirmation mode
                logging::log("EXEC", &format!("Awaiting confirmation: {}", entry.id));
                self.pending_confirmation = Some(entry.id.clone());
                cx.notify();
                return; // Don't execute yet
            }
        }

        match &entry.feature {
            builtins::BuiltInFeature::ClipboardHistory => {
                logging::log("EXEC", "Opening Clipboard History");
                // Use cached entries for faster loading
                let entries = clipboard_history::get_cached_entries(100);
                logging::log(
                    "EXEC",
                    &format!("Loaded {} clipboard entries (cached)", entries.len()),
                );
                // Initial selected_index should be 0 (first entry)
                // Note: clipboard history uses a flat list without section headers
                self.current_view = AppView::ClipboardHistoryView {
                    entries,
                    filter: String::new(),
                    selected_index: 0,
                };
                // Use standard height for clipboard history view
                defer_resize_to_view(ViewType::ScriptList, 0, cx);
                cx.notify();
            }
            builtins::BuiltInFeature::AppLauncher => {
                logging::log("EXEC", "Opening App Launcher");
                let apps = app_launcher::scan_applications().clone();
                logging::log("EXEC", &format!("Loaded {} applications", apps.len()));
                self.current_view = AppView::AppLauncherView {
                    apps,
                    filter: String::new(),
                    selected_index: 0,
                };
                // Use standard height for app launcher view
                defer_resize_to_view(ViewType::ScriptList, 0, cx);
                cx.notify();
            }
            builtins::BuiltInFeature::App(app_name) => {
                logging::log("EXEC", &format!("Launching app: {}", app_name));
                // Find and launch the specific application
                let apps = app_launcher::scan_applications();
                if let Some(app) = apps.iter().find(|a| a.name == *app_name) {
                    if let Err(e) = app_launcher::launch_application(app) {
                        logging::log("ERROR", &format!("Failed to launch {}: {}", app_name, e));
                        self.last_output = Some(SharedString::from(format!(
                            "Failed to launch: {}",
                            app_name
                        )));
                    } else {
                        logging::log("EXEC", &format!("Launched app: {}", app_name));
                        self.close_and_reset_window(cx);
                    }
                } else {
                    logging::log("ERROR", &format!("App not found: {}", app_name));
                    self.last_output =
                        Some(SharedString::from(format!("App not found: {}", app_name)));
                }
                cx.notify();
            }
            builtins::BuiltInFeature::WindowSwitcher => {
                logging::log("EXEC", "Opening Window Switcher");
                // Load windows when view is opened (windows change frequently)
                match window_control::list_windows() {
                    Ok(windows) => {
                        logging::log("EXEC", &format!("Loaded {} windows", windows.len()));
                        self.current_view = AppView::WindowSwitcherView {
                            windows,
                            filter: String::new(),
                            selected_index: 0,
                        };
                        // Use standard height for window switcher view
                        defer_resize_to_view(ViewType::ScriptList, 0, cx);
                    }
                    Err(e) => {
                        logging::log("ERROR", &format!("Failed to list windows: {}", e));
                        self.toast_manager.push(
                            components::toast::Toast::error(
                                format!("Failed to list windows: {}", e),
                                &self.theme,
                            )
                            .duration_ms(Some(5000)),
                        );
                    }
                }
                cx.notify();
            }
            builtins::BuiltInFeature::DesignGallery => {
                logging::log("EXEC", "Opening Design Gallery");
                self.current_view = AppView::DesignGalleryView {
                    filter: String::new(),
                    selected_index: 0,
                };
                // Use standard height for design gallery view
                defer_resize_to_view(ViewType::ScriptList, 0, cx);
                cx.notify();
            }
            builtins::BuiltInFeature::AiChat => {
                logging::log("EXEC", "Opening AI Chat window");
                // Reset state, hide main window, and open AI window
                script_kit_gpui::set_main_window_visible(false);
                self.reset_to_script_list(cx);
                platform::hide_main_window();
                if let Err(e) = ai::open_ai_window(cx) {
                    logging::log("ERROR", &format!("Failed to open AI window: {}", e));
                    self.toast_manager.push(
                        components::toast::Toast::error(
                            format!("Failed to open AI: {}", e),
                            &self.theme,
                        )
                        .duration_ms(Some(5000)),
                    );
                    cx.notify();
                }
            }
            builtins::BuiltInFeature::Notes => {
                logging::log("EXEC", "Opening Notes window");
                // Reset state, hide main window, and open Notes window
                script_kit_gpui::set_main_window_visible(false);
                self.reset_to_script_list(cx);
                platform::hide_main_window();
                if let Err(e) = notes::open_notes_window(cx) {
                    logging::log("ERROR", &format!("Failed to open Notes window: {}", e));
                    self.toast_manager.push(
                        components::toast::Toast::error(
                            format!("Failed to open Notes: {}", e),
                            &self.theme,
                        )
                        .duration_ms(Some(5000)),
                    );
                    cx.notify();
                }
            }
            builtins::BuiltInFeature::MenuBarAction(action) => {
                logging::log(
                    "EXEC",
                    &format!(
                        "Executing menu bar action: {} -> {}",
                        action.bundle_id,
                        action.menu_path.join(" → ")
                    ),
                );
                // Execute menu action via accessibility API
                #[cfg(target_os = "macos")]
                {
                    match script_kit_gpui::menu_executor::execute_menu_action(
                        &action.bundle_id,
                        &action.menu_path,
                    ) {
                        Ok(()) => {
                            logging::log("EXEC", "Menu action executed successfully");
                            self.close_and_reset_window(cx);
                        }
                        Err(e) => {
                            logging::log("ERROR", &format!("Menu action failed: {}", e));
                            self.toast_manager.push(
                                components::toast::Toast::error(
                                    format!("Menu action failed: {}", e),
                                    &self.theme,
                                )
                                .duration_ms(Some(5000)),
                            );
                            cx.notify();
                        }
                    }
                }
                #[cfg(not(target_os = "macos"))]
                {
                    logging::log("WARN", "Menu bar actions only supported on macOS");
                    self.toast_manager.push(
                        components::toast::Toast::warning(
                            "Menu bar actions are only supported on macOS",
                            &self.theme,
                        )
                        .duration_ms(Some(3000)),
                    );
                    cx.notify();
                }
            }

            // =========================================================================
            // System Actions
            // =========================================================================
            builtins::BuiltInFeature::SystemAction(action_type) => {
                logging::log(
                    "EXEC",
                    &format!("Executing system action: {:?}", action_type),
                );

                #[cfg(target_os = "macos")]
                {
                    use builtins::SystemActionType;

                    let result = match action_type {
                        // Power management
                        SystemActionType::EmptyTrash => system_actions::empty_trash(),
                        SystemActionType::LockScreen => system_actions::lock_screen(),
                        SystemActionType::Sleep => system_actions::sleep(),
                        SystemActionType::Restart => system_actions::restart(),
                        SystemActionType::ShutDown => system_actions::shut_down(),
                        SystemActionType::LogOut => system_actions::log_out(),

                        // UI controls
                        SystemActionType::ToggleDarkMode => system_actions::toggle_dark_mode(),
                        SystemActionType::ShowDesktop => system_actions::show_desktop(),
                        SystemActionType::MissionControl => system_actions::mission_control(),
                        SystemActionType::Launchpad => system_actions::launchpad(),
                        SystemActionType::ForceQuitApps => system_actions::force_quit_apps(),

                        // Volume controls (preset levels)
                        SystemActionType::Volume0 => system_actions::set_volume(0),
                        SystemActionType::Volume25 => system_actions::set_volume(25),
                        SystemActionType::Volume50 => system_actions::set_volume(50),
                        SystemActionType::Volume75 => system_actions::set_volume(75),
                        SystemActionType::Volume100 => system_actions::set_volume(100),
                        SystemActionType::VolumeMute => system_actions::volume_mute(),

                        // Brightness controls (preset levels)
                        SystemActionType::Brightness0 => system_actions::set_brightness(0),
                        SystemActionType::Brightness25 => system_actions::set_brightness(25),
                        SystemActionType::Brightness50 => system_actions::set_brightness(50),
                        SystemActionType::Brightness75 => system_actions::set_brightness(75),
                        SystemActionType::Brightness100 => system_actions::set_brightness(100),

                        // Dev/test actions
                        #[cfg(debug_assertions)]
                        SystemActionType::TestConfirmation => {
                            self.toast_manager.push(
                                components::toast::Toast::success(
                                    "Confirmation test passed!",
                                    &self.theme,
                                )
                                .duration_ms(Some(3000)),
                            );
                            cx.notify();
                            return; // Don't hide window for test
                        }

                        // App control
                        SystemActionType::QuitScriptKit => {
                            logging::log("EXEC", "Quitting Script Kit");
                            cx.quit();
                            return;
                        }

                        // System utilities
                        SystemActionType::ToggleDoNotDisturb => {
                            system_actions::toggle_do_not_disturb()
                        }
                        SystemActionType::StartScreenSaver => system_actions::start_screen_saver(),

                        // System Preferences
                        SystemActionType::OpenSystemPreferences => {
                            system_actions::open_system_preferences_main()
                        }
                        SystemActionType::OpenPrivacySettings => {
                            system_actions::open_privacy_settings()
                        }
                        SystemActionType::OpenDisplaySettings => {
                            system_actions::open_display_settings()
                        }
                        SystemActionType::OpenSoundSettings => {
                            system_actions::open_sound_settings()
                        }
                        SystemActionType::OpenNetworkSettings => {
                            system_actions::open_network_settings()
                        }
                        SystemActionType::OpenKeyboardSettings => {
                            system_actions::open_keyboard_settings()
                        }
                        SystemActionType::OpenBluetoothSettings => {
                            system_actions::open_bluetooth_settings()
                        }
                        SystemActionType::OpenNotificationsSettings => {
                            system_actions::open_notifications_settings()
                        }
                    };

                    match result {
                        Ok(()) => {
                            logging::log("EXEC", "System action executed successfully");
                            self.close_and_reset_window(cx);
                        }
                        Err(e) => {
                            logging::log("ERROR", &format!("System action failed: {}", e));
                            self.toast_manager.push(
                                components::toast::Toast::error(
                                    format!("System action failed: {}", e),
                                    &self.theme,
                                )
                                .duration_ms(Some(5000)),
                            );
                            cx.notify();
                        }
                    }
                }

                #[cfg(not(target_os = "macos"))]
                {
                    logging::log("WARN", "System actions only supported on macOS");
                    self.toast_manager.push(
                        components::toast::Toast::warning(
                            "System actions are only supported on macOS",
                            &self.theme,
                        )
                        .duration_ms(Some(3000)),
                    );
                    cx.notify();
                }
            }

            // =========================================================================
            // Window Actions (for frontmost window of the PREVIOUS app)
            // =========================================================================
            builtins::BuiltInFeature::WindowAction(action_type) => {
                logging::log(
                    "EXEC",
                    &format!("Executing window action: {:?}", action_type),
                );

                // Get the frontmost window of the app that was active before Script Kit.
                // Since Script Kit is an LSUIElement (accessory app), it doesn't take
                // menu bar ownership. The menu bar owner is the previously active app.
                match window_control::get_frontmost_window_of_previous_app() {
                    Ok(Some(target_window)) => {
                        use builtins::WindowActionType;
                        use window_control::TilePosition;

                        logging::log(
                            "EXEC",
                            &format!(
                                "Target window: {} - {} (id: {})",
                                target_window.app, target_window.title, target_window.id
                            ),
                        );

                        let result = match action_type {
                            WindowActionType::TileLeft => window_control::tile_window(
                                target_window.id,
                                TilePosition::LeftHalf,
                            ),
                            WindowActionType::TileRight => window_control::tile_window(
                                target_window.id,
                                TilePosition::RightHalf,
                            ),
                            WindowActionType::TileTop => {
                                window_control::tile_window(target_window.id, TilePosition::TopHalf)
                            }
                            WindowActionType::TileBottom => window_control::tile_window(
                                target_window.id,
                                TilePosition::BottomHalf,
                            ),
                            WindowActionType::Maximize => {
                                window_control::maximize_window(target_window.id)
                            }
                            WindowActionType::Minimize => {
                                window_control::minimize_window(target_window.id)
                            }
                        };

                        match result {
                            Ok(()) => {
                                logging::log("EXEC", "Window action executed successfully");
                                // Reset and hide - does the reset work immediately while hiding
                                self.close_and_reset_window(cx);
                            }
                            Err(e) => {
                                logging::log("ERROR", &format!("Window action failed: {}", e));
                                self.toast_manager.push(
                                    components::toast::Toast::error(
                                        format!("Window action failed: {}", e),
                                        &self.theme,
                                    )
                                    .duration_ms(Some(5000)),
                                );
                                cx.notify();
                            }
                        }
                    }
                    Ok(None) => {
                        logging::log("WARN", "No windows found for previous app");
                        self.toast_manager.push(
                            components::toast::Toast::warning(
                                "No windows available to manage",
                                &self.theme,
                            )
                            .duration_ms(Some(3000)),
                        );
                        cx.notify();
                    }
                    Err(e) => {
                        logging::log("ERROR", &format!("Failed to get target window: {}", e));
                        self.toast_manager.push(
                            components::toast::Toast::error(
                                format!("Failed to find target window: {}", e),
                                &self.theme,
                            )
                            .duration_ms(Some(5000)),
                        );
                        cx.notify();
                    }
                }
            }

            // =========================================================================
            // Notes Commands
            // =========================================================================
            builtins::BuiltInFeature::NotesCommand(cmd_type) => {
                logging::log("EXEC", &format!("Executing notes command: {:?}", cmd_type));

                use builtins::NotesCommandType;

                // All notes commands: reset state, hide main window, open notes
                script_kit_gpui::set_main_window_visible(false);
                self.reset_to_script_list(cx);
                platform::hide_main_window();

                let result = match cmd_type {
                    NotesCommandType::OpenNotes
                    | NotesCommandType::NewNote
                    | NotesCommandType::SearchNotes => notes::open_notes_window(cx),
                    NotesCommandType::QuickCapture => notes::quick_capture(cx),
                };

                if let Err(e) = result {
                    logging::log("ERROR", &format!("Notes command failed: {}", e));
                    self.toast_manager.push(
                        components::toast::Toast::error(
                            format!("Notes command failed: {}", e),
                            &self.theme,
                        )
                        .duration_ms(Some(5000)),
                    );
                    cx.notify();
                }
            }

            // =========================================================================
            // AI Commands
            // =========================================================================
            builtins::BuiltInFeature::AiCommand(cmd_type) => {
                logging::log("EXEC", &format!("Executing AI command: {:?}", cmd_type));

                // All AI commands: reset state, hide main window, open AI
                script_kit_gpui::set_main_window_visible(false);
                self.reset_to_script_list(cx);
                platform::hide_main_window();

                if let Err(e) = ai::open_ai_window(cx) {
                    logging::log("ERROR", &format!("AI command failed: {}", e));
                    self.toast_manager.push(
                        components::toast::Toast::error(
                            format!("Failed to open AI: {}", e),
                            &self.theme,
                        )
                        .duration_ms(Some(5000)),
                    );
                    cx.notify();
                }
            }

            // =========================================================================
            // Script Commands
            // =========================================================================
            builtins::BuiltInFeature::ScriptCommand(cmd_type) => {
                logging::log("EXEC", &format!("Executing script command: {:?}", cmd_type));

                use builtins::ScriptCommandType;

                let (create_result, item_type) = match cmd_type {
                    ScriptCommandType::NewScript => {
                        (script_creation::create_new_script("untitled"), "script")
                    }
                    ScriptCommandType::NewScriptlet => (
                        script_creation::create_new_scriptlet("untitled"),
                        "scriptlet",
                    ),
                };

                match create_result {
                    Ok(path) => {
                        logging::log("EXEC", &format!("Created new {}: {:?}", item_type, path));
                        if let Err(e) = script_creation::open_in_editor(&path, &self.config) {
                            logging::log("ERROR", &format!("Failed to open in editor: {}", e));
                            self.toast_manager.push(
                                components::toast::Toast::error(
                                    format!(
                                        "Created {} but failed to open editor: {}",
                                        item_type, e
                                    ),
                                    &self.theme,
                                )
                                .duration_ms(Some(5000)),
                            );
                        } else {
                            self.toast_manager.push(
                                components::toast::Toast::success(
                                    format!("New {} created and opened in editor", item_type),
                                    &self.theme,
                                )
                                .duration_ms(Some(3000)),
                            );
                        }
                        self.close_and_reset_window(cx);
                    }
                    Err(e) => {
                        logging::log("ERROR", &format!("Failed to create {}: {}", item_type, e));
                        self.toast_manager.push(
                            components::toast::Toast::error(
                                format!("Failed to create {}: {}", item_type, e),
                                &self.theme,
                            )
                            .duration_ms(Some(5000)),
                        );
                        cx.notify();
                    }
                }
            }

            // =========================================================================
            // Permission Commands
            // =========================================================================
            builtins::BuiltInFeature::PermissionCommand(cmd_type) => {
                logging::log(
                    "EXEC",
                    &format!("Executing permission command: {:?}", cmd_type),
                );

                use builtins::PermissionCommandType;

                match cmd_type {
                    PermissionCommandType::CheckPermissions => {
                        let status = permissions_wizard::check_all_permissions();
                        if status.all_granted() {
                            self.toast_manager.push(
                                components::toast::Toast::success(
                                    "All permissions granted!",
                                    &self.theme,
                                )
                                .duration_ms(Some(3000)),
                            );
                        } else {
                            let missing: Vec<_> = status
                                .missing_permissions()
                                .iter()
                                .map(|p| p.permission_type.name())
                                .collect();
                            self.toast_manager.push(
                                components::toast::Toast::warning(
                                    format!("Missing permissions: {}", missing.join(", ")),
                                    &self.theme,
                                )
                                .duration_ms(Some(5000)),
                            );
                        }
                        cx.notify();
                    }
                    PermissionCommandType::RequestAccessibility => {
                        let granted = permissions_wizard::request_accessibility_permission();
                        if granted {
                            self.toast_manager.push(
                                components::toast::Toast::success(
                                    "Accessibility permission granted!",
                                    &self.theme,
                                )
                                .duration_ms(Some(3000)),
                            );
                        } else {
                            self.toast_manager.push(
                                components::toast::Toast::warning(
                                    "Accessibility permission not granted. Some features may not work.",
                                    &self.theme,
                                )
                                .duration_ms(Some(5000)),
                            );
                        }
                        cx.notify();
                    }
                    PermissionCommandType::OpenAccessibilitySettings => {
                        if let Err(e) = permissions_wizard::open_accessibility_settings() {
                            logging::log(
                                "ERROR",
                                &format!("Failed to open accessibility settings: {}", e),
                            );
                            self.toast_manager.push(
                                components::toast::Toast::error(
                                    format!("Failed to open settings: {}", e),
                                    &self.theme,
                                )
                                .duration_ms(Some(5000)),
                            );
                            cx.notify();
                        } else {
                            self.close_and_reset_window(cx);
                        }
                    }
                }
            }

            // =========================================================================
            // Frecency/Suggested Commands
            // =========================================================================
            builtins::BuiltInFeature::FrecencyCommand(cmd_type) => {
                logging::log(
                    "EXEC",
                    &format!("Executing frecency command: {:?}", cmd_type),
                );

                use builtins::FrecencyCommandType;

                match cmd_type {
                    FrecencyCommandType::ClearSuggested => {
                        // Clear all frecency data
                        self.frecency_store.clear();
                        if let Err(e) = self.frecency_store.save() {
                            logging::log("ERROR", &format!("Failed to save frecency data: {}", e));
                            self.toast_manager.push(
                                components::toast::Toast::error(
                                    format!("Failed to clear suggested: {}", e),
                                    &self.theme,
                                )
                                .duration_ms(Some(5000)),
                            );
                        } else {
                            logging::log("EXEC", "Cleared all suggested items");
                            // Invalidate the grouped cache so the UI updates
                            self.invalidate_grouped_cache();
                            // Reset the main input and window to clean state
                            self.reset_to_script_list(cx);
                            self.toast_manager.push(
                                components::toast::Toast::success(
                                    "Suggested items cleared",
                                    &self.theme,
                                )
                                .duration_ms(Some(3000)),
                            );
                        }
                        // Note: cx.notify() is called by reset_to_script_list, but we still need it for error case
                        cx.notify();
                    }
                }
            }
        }
    }

    /// Execute an application directly from the main search results
    fn execute_app(&mut self, app: &app_launcher::AppInfo, cx: &mut Context<Self>) {
        logging::log("EXEC", &format!("Launching app from search: {}", app.name));

        if let Err(e) = app_launcher::launch_application(app) {
            logging::log("ERROR", &format!("Failed to launch {}: {}", app.name, e));
            self.last_output = Some(SharedString::from(format!(
                "Failed to launch: {}",
                app.name
            )));
            cx.notify();
        } else {
            logging::log("EXEC", &format!("Launched app: {}", app.name));
            self.close_and_reset_window(cx);
        }
    }

    /// Focus a window from the main search results
    fn execute_window_focus(
        &mut self,
        window: &window_control::WindowInfo,
        cx: &mut Context<Self>,
    ) {
        logging::log(
            "EXEC",
            &format!("Focusing window: {} - {}", window.app, window.title),
        );

        if let Err(e) = window_control::focus_window(window.id) {
            logging::log("ERROR", &format!("Failed to focus window: {}", e));
            self.toast_manager.push(
                components::toast::Toast::error(
                    format!("Failed to focus window: {}", e),
                    &self.theme,
                )
                .duration_ms(Some(5000)),
            );
            cx.notify();
        } else {
            logging::log("EXEC", &format!("Focused window: {}", window.title));
            self.close_and_reset_window(cx);
        }
    }
}

</file>

</files>
📊 Pack Summary:
────────────────
  Total Files: 8 files
  Search Mode: ripgrep (fast)
  Total Tokens: ~69.7K (69,718 exact)
  Total Chars: 388,292 chars
       Output: -

📁 Extensions Found:
────────────────────
  .rs

📂 Top 10 Files (by tokens):
──────────────────────────
     20.8K - src/main.rs
     20.0K - src/app_impl.rs
     10.4K - src/app_render.rs
      6.0K - src/app_layout.rs
      5.2K - src/app_execute.rs
      3.1K - src/app_actions.rs
      2.7K - src/app_navigation.rs
      1.5K - src/lib.rs
