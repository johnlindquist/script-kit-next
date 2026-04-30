//! GPUI-friendly macOS Accessibility (AXUIElement) client.
//!
//! This crate is intentionally split into two layers:
//! - A platform-neutral public API (`AxClient`, `AxElement`, snapshots, observers).
//! - A macOS backend that talks to `ApplicationServices.framework` through
//!   `AXUIElement`, `AXObserver`, and screenshot/window-friendly CoreGraphics metadata.
//!
//! The core API does **not** depend on GPUI. Enable the `gpui` feature for the
//! thin `gpui_bridge` module, or feed `AxEvent` values from the observer receiver
//! into your own GPUI entity.

use std::{collections::HashSet, fmt, sync::mpsc, thread, time::Duration};

#[cfg(target_os = "macos")]
#[path = "platform/macos.rs"]
mod platform;

#[cfg(not(target_os = "macos"))]
#[path = "platform/unsupported.rs"]
mod platform;

#[cfg(feature = "gpui")]
pub mod gpui_bridge;

/// Common AX attribute names.
pub mod attr {
    pub const ROLE: &str = "AXRole";
    pub const SUBROLE: &str = "AXSubrole";
    pub const ROLE_DESCRIPTION: &str = "AXRoleDescription";
    pub const TITLE: &str = "AXTitle";
    pub const TITLE_UI_ELEMENT: &str = "AXTitleUIElement";
    pub const VALUE: &str = "AXValue";
    pub const VALUE_DESCRIPTION: &str = "AXValueDescription";
    pub const DESCRIPTION: &str = "AXDescription";
    pub const HELP: &str = "AXHelp";
    pub const IDENTIFIER: &str = "AXIdentifier";
    pub const ENABLED: &str = "AXEnabled";
    pub const FOCUSED: &str = "AXFocused";
    pub const POSITION: &str = "AXPosition";
    pub const SIZE: &str = "AXSize";
    pub const PARENT: &str = "AXParent";
    pub const CHILDREN: &str = "AXChildren";
    pub const CHILDREN_IN_NAVIGATION_ORDER: &str = "AXChildrenInNavigationOrder";
    pub const CONTENTS: &str = "AXContents";
    pub const WINDOWS: &str = "AXWindows";
    pub const WINDOW: &str = "AXWindow";
    pub const MAIN: &str = "AXMain";
    pub const MAIN_WINDOW: &str = "AXMainWindow";
    pub const FOCUSED_WINDOW: &str = "AXFocusedWindow";
    pub const FOCUSED_UI_ELEMENT: &str = "AXFocusedUIElement";
    pub const FOCUSED_APPLICATION: &str = "AXFocusedApplication";
    pub const TOP_LEVEL_UI_ELEMENT: &str = "AXTopLevelUIElement";
    pub const MENU_BAR: &str = "AXMenuBar";
    pub const EXTRAS_MENU_BAR: &str = "AXExtrasMenuBar";
    pub const SHOWN_MENU_UI_ELEMENT: &str = "AXShownMenuUIElement";
    pub const SELECTED_TEXT: &str = "AXSelectedText";
    pub const SELECTED_TEXT_RANGE: &str = "AXSelectedTextRange";
    pub const SELECTED_TEXT_RANGES: &str = "AXSelectedTextRanges";
    pub const VISIBLE_TEXT: &str = "AXVisibleText";
    pub const VISIBLE_CHARACTER_RANGE: &str = "AXVisibleCharacterRange";
    pub const NUMBER_OF_CHARACTERS: &str = "AXNumberOfCharacters";
    pub const INSERTION_POINT_LINE_NUMBER: &str = "AXInsertionPointLineNumber";
    pub const FRONTMOST: &str = "AXFrontmost";
    pub const HIDDEN: &str = "AXHidden";
    pub const MINIMIZED: &str = "AXMinimized";
    pub const MODAL: &str = "AXModal";
    pub const EXPANDED: &str = "AXExpanded";
    pub const SELECTED: &str = "AXSelected";
    pub const VISIBLE: &str = "AXVisible";
    pub const DOCUMENT: &str = "AXDocument";
    pub const URL: &str = "AXURL";
    pub const FILENAME: &str = "AXFilename";
    pub const CLOSE_BUTTON: &str = "AXCloseButton";
    pub const MINIMIZE_BUTTON: &str = "AXMinimizeButton";
    pub const ZOOM_BUTTON: &str = "AXZoomButton";
    pub const FULL_SCREEN_BUTTON: &str = "AXFullScreenButton";
    pub const DEFAULT_BUTTON: &str = "AXDefaultButton";
    pub const CANCEL_BUTTON: &str = "AXCancelButton";
    pub const TOOLBAR_BUTTON: &str = "AXToolbarButton";
    pub const PROXY: &str = "AXProxy";
    pub const TABS: &str = "AXTabs";
    pub const ROWS: &str = "AXRows";
    pub const COLUMNS: &str = "AXColumns";
    pub const SELECTED_CHILDREN: &str = "AXSelectedChildren";
    pub const SELECTED_ROWS: &str = "AXSelectedRows";
    pub const SELECTED_COLUMNS: &str = "AXSelectedColumns";
    pub const VISIBLE_ROWS: &str = "AXVisibleRows";
    pub const VISIBLE_COLUMNS: &str = "AXVisibleColumns";
    pub const VISIBLE_CELLS: &str = "AXVisibleCells";
    pub const HORIZONTAL_SCROLL_BAR: &str = "AXHorizontalScrollBar";
    pub const VERTICAL_SCROLL_BAR: &str = "AXVerticalScrollBar";
    pub const SPLITTERS: &str = "AXSplitters";
}

/// Common AX role names.
pub mod role {
    pub const APPLICATION: &str = "AXApplication";
    pub const SYSTEM_WIDE: &str = "AXSystemWide";
    pub const WINDOW: &str = "AXWindow";
    pub const DRAWER: &str = "AXDrawer";
    pub const SHEET: &str = "AXSheet";
    pub const DIALOG: &str = "AXDialog";
    pub const GROUP: &str = "AXGroup";
    pub const BUTTON: &str = "AXButton";
    pub const RADIO_BUTTON: &str = "AXRadioButton";
    pub const CHECK_BOX: &str = "AXCheckBox";
    pub const POP_UP_BUTTON: &str = "AXPopUpButton";
    pub const MENU_BUTTON: &str = "AXMenuButton";
    pub const MENU_BAR: &str = "AXMenuBar";
    pub const MENU_BAR_ITEM: &str = "AXMenuBarItem";
    pub const MENU: &str = "AXMenu";
    pub const MENU_ITEM: &str = "AXMenuItem";
    pub const TEXT_FIELD: &str = "AXTextField";
    pub const TEXT_AREA: &str = "AXTextArea";
    pub const STATIC_TEXT: &str = "AXStaticText";
    pub const IMAGE: &str = "AXImage";
    pub const SCROLL_AREA: &str = "AXScrollArea";
    pub const SCROLL_BAR: &str = "AXScrollBar";
    pub const TABLE: &str = "AXTable";
    pub const OUTLINE: &str = "AXOutline";
    pub const BROWSER: &str = "AXBrowser";
    pub const ROW: &str = "AXRow";
    pub const COLUMN: &str = "AXColumn";
    pub const CELL: &str = "AXCell";
    pub const LIST: &str = "AXList";
    pub const TAB_GROUP: &str = "AXTabGroup";
    pub const TOOLBAR: &str = "AXToolbar";
    pub const SPLITTER: &str = "AXSplitter";
    pub const SLIDER: &str = "AXSlider";
    pub const VALUE_INDICATOR: &str = "AXValueIndicator";
    pub const WEB_AREA: &str = "AXWebArea";
    pub const UNKNOWN: &str = "AXUnknown";
}

/// Common AX subrole names.
pub mod subrole {
    pub const STANDARD_WINDOW: &str = "AXStandardWindow";
    pub const FLOATING_WINDOW: &str = "AXFloatingWindow";
    pub const SYSTEM_FLOATING_WINDOW: &str = "AXSystemFloatingWindow";
    pub const SYSTEM_DIALOG: &str = "AXSystemDialog";
    pub const CLOSE_BUTTON: &str = "AXCloseButton";
    pub const MINIMIZE_BUTTON: &str = "AXMinimizeButton";
    pub const ZOOM_BUTTON: &str = "AXZoomButton";
    pub const FULL_SCREEN_BUTTON: &str = "AXFullScreenButton";
    pub const SEARCH_FIELD: &str = "AXSearchField";
    pub const SECURE_TEXT_FIELD: &str = "AXSecureTextField";
    pub const TABLE_ROW: &str = "AXTableRow";
    pub const OUTLINE_ROW: &str = "AXOutlineRow";
    pub const SORT_BUTTON: &str = "AXSortButton";
    pub const TOOLBAR_BUTTON: &str = "AXToolbarButton";
}

/// Common AX action names.
pub mod action {
    pub const PRESS: &str = "AXPress";
    pub const INCREMENT: &str = "AXIncrement";
    pub const DECREMENT: &str = "AXDecrement";
    pub const CONFIRM: &str = "AXConfirm";
    pub const CANCEL: &str = "AXCancel";
    pub const SHOW_MENU: &str = "AXShowMenu";
    pub const SHOW_DEFAULT_UI: &str = "AXShowDefaultUI";
    pub const SHOW_ALTERNATE_UI: &str = "AXShowAlternateUI";
    pub const RAISE: &str = "AXRaise";
    pub const PICK: &str = "AXPick";
    pub const SCROLL_TO_VISIBLE: &str = "AXScrollToVisible";
}

/// Common AX parameterized attribute names.
pub mod param {
    pub const STRING_FOR_RANGE: &str = "AXStringForRange";
    pub const ATTRIBUTED_STRING_FOR_RANGE: &str = "AXAttributedStringForRange";
    pub const RTF_FOR_RANGE: &str = "AXRTFForRange";
    pub const BOUNDS_FOR_RANGE: &str = "AXBoundsForRange";
    pub const LINE_FOR_INDEX: &str = "AXLineForIndex";
    pub const RANGE_FOR_LINE: &str = "AXRangeForLine";
    pub const RANGE_FOR_INDEX: &str = "AXRangeForIndex";
    pub const RANGE_FOR_POSITION: &str = "AXRangeForPosition";
    pub const CELL_FOR_COLUMN_AND_ROW: &str = "AXCellForColumnAndRow";
    pub const LAYOUT_POINT_FOR_SCREEN_POINT: &str = "AXLayoutPointForScreenPoint";
    pub const LAYOUT_SIZE_FOR_SCREEN_SIZE: &str = "AXLayoutSizeForScreenSize";
    pub const SCREEN_POINT_FOR_LAYOUT_POINT: &str = "AXScreenPointForLayoutPoint";
    pub const SCREEN_SIZE_FOR_LAYOUT_SIZE: &str = "AXScreenSizeForLayoutSize";
}

/// Common AX notification names.
pub mod notification {
    pub const APPLICATION_ACTIVATED: &str = "AXApplicationActivated";
    pub const APPLICATION_DEACTIVATED: &str = "AXApplicationDeactivated";
    pub const APPLICATION_HIDDEN: &str = "AXApplicationHidden";
    pub const APPLICATION_SHOWN: &str = "AXApplicationShown";
    pub const FOCUSED_WINDOW_CHANGED: &str = "AXFocusedWindowChanged";
    pub const FOCUSED_UI_ELEMENT_CHANGED: &str = "AXFocusedUIElementChanged";
    pub const MAIN_WINDOW_CHANGED: &str = "AXMainWindowChanged";
    pub const WINDOW_CREATED: &str = "AXWindowCreated";
    pub const WINDOW_MOVED: &str = "AXWindowMoved";
    pub const WINDOW_RESIZED: &str = "AXWindowResized";
    pub const WINDOW_MINIATURIZED: &str = "AXWindowMiniaturized";
    pub const WINDOW_DEMINIATURIZED: &str = "AXWindowDeminiaturized";
    pub const DRAWER_CREATED: &str = "AXDrawerCreated";
    pub const SHEET_CREATED: &str = "AXSheetCreated";
    pub const UI_ELEMENT_DESTROYED: &str = "AXUIElementDestroyed";
    pub const CREATED: &str = "AXCreated";
    pub const MOVED: &str = "AXMoved";
    pub const RESIZED: &str = "AXResized";
    pub const TITLE_CHANGED: &str = "AXTitleChanged";
    pub const VALUE_CHANGED: &str = "AXValueChanged";
    pub const SELECTED_TEXT_CHANGED: &str = "AXSelectedTextChanged";
    pub const SELECTED_CHILDREN_CHANGED: &str = "AXSelectedChildrenChanged";
    pub const SELECTED_CHILDREN_MOVED: &str = "AXSelectedChildrenMoved";
    pub const SELECTED_ROWS_CHANGED: &str = "AXSelectedRowsChanged";
    pub const SELECTED_COLUMNS_CHANGED: &str = "AXSelectedColumnsChanged";
    pub const SELECTED_CELLS_CHANGED: &str = "AXSelectedCellsChanged";
    pub const ROW_COUNT_CHANGED: &str = "AXRowCountChanged";
    pub const ROW_EXPANDED: &str = "AXRowExpanded";
    pub const ROW_COLLAPSED: &str = "AXRowCollapsed";
    pub const LAYOUT_CHANGED: &str = "AXLayoutChanged";
    pub const MENU_OPENED: &str = "AXMenuOpened";
    pub const MENU_CLOSED: &str = "AXMenuClosed";
    pub const MENU_ITEM_SELECTED: &str = "AXMenuItemSelected";
    pub const HELP_TAG_CREATED: &str = "AXHelpTagCreated";
    pub const ANNOUNCEMENT_REQUESTED: &str = "AXAnnouncementRequested";
    pub const UNITS_CHANGED: &str = "AXUnitsChanged";
}

pub type Result<T> = std::result::Result<T, AxError>;

/// Error type returned by the high-level AX API.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AxError {
    /// The current OS is not macOS.
    UnsupportedPlatform { platform: &'static str },
    /// Accessibility permission is not granted to the current process.
    NotTrusted,
    /// The system accessibility API is disabled or unavailable.
    ApiDisabled,
    /// A Core Foundation, CoreGraphics, or Accessibility function returned a null pointer.
    NullPointer(&'static str),
    /// The target process or element did not expose the requested value.
    NoValue { attribute: String },
    /// The target process or element does not support the requested attribute.
    AttributeUnsupported { attribute: String },
    /// The target process or element does not support the requested action.
    ActionUnsupported { action: String },
    /// The target process or element does not support the requested notification.
    NotificationUnsupported { notification: String },
    /// The AXUIElement became invalid. This is common when windows close.
    InvalidElement,
    /// The target app did not answer the AX request before the messaging timeout.
    CannotComplete,
    /// Returned value had a different Core Foundation / AX type than expected.
    TypeMismatch { expected: &'static str, actual: String },
    /// Observer setup or run-loop management failed.
    Observer(String),
    /// Window-list or window-metadata query failed.
    WindowList(String),
    /// Generic AX error code.
    Ax { code: i32, message: String },
}

impl fmt::Display for AxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AxError::UnsupportedPlatform { platform } => {
                write!(f, "macOS Accessibility is not available on {platform}")
            }
            AxError::NotTrusted => write!(
                f,
                "accessibility permission is not granted; enable it in System Settings → Privacy & Security → Accessibility"
            ),
            AxError::ApiDisabled => write!(f, "the macOS Accessibility API is disabled or unavailable"),
            AxError::NullPointer(name) => write!(f, "{name} returned a null pointer"),
            AxError::NoValue { attribute } => write!(f, "attribute {attribute} has no value"),
            AxError::AttributeUnsupported { attribute } => {
                write!(f, "attribute {attribute} is not supported")
            }
            AxError::ActionUnsupported { action } => write!(f, "action {action} is not supported"),
            AxError::NotificationUnsupported { notification } => {
                write!(f, "notification {notification} is not supported")
            }
            AxError::InvalidElement => write!(f, "accessibility element is invalid"),
            AxError::CannotComplete => write!(
                f,
                "accessibility request could not complete; the target app may be busy or the timeout may be too low"
            ),
            AxError::TypeMismatch { expected, actual } => {
                write!(f, "expected {expected}, got {actual}")
            }
            AxError::Observer(message) => write!(f, "observer error: {message}"),
            AxError::WindowList(message) => write!(f, "window-list error: {message}"),
            AxError::Ax { code, message } => write!(f, "AX error {code}: {message}"),
        }
    }
}

impl std::error::Error for AxError {}

/// Options used when creating an [`AxClient`].
#[derive(Debug, Clone, Copy)]
pub struct AxClientOptions {
    /// Ask macOS to show the Accessibility permission prompt when permission is missing.
    pub prompt_for_permission: bool,
    /// Per-process AX messaging timeout. Keep this low for UI apps.
    pub messaging_timeout: Option<Duration>,
}

impl Default for AxClientOptions {
    fn default() -> Self {
        Self {
            prompt_for_permission: true,
            messaging_timeout: Some(Duration::from_millis(800)),
        }
    }
}

/// Options for recursive tree snapshots.
#[derive(Debug, Clone, Copy)]
pub struct TreeOptions {
    pub max_depth: usize,
    pub max_children_per_node: usize,
    /// Include children even for large container roles. Leave this false for fast UI refreshes.
    pub include_all_children: bool,
}

impl Default for TreeOptions {
    fn default() -> Self {
        Self {
            max_depth: 4,
            max_children_per_node: 64,
            include_all_children: false,
        }
    }
}

/// Options for in-process AX tree searches.
#[derive(Debug, Clone, Copy)]
pub struct SearchOptions {
    pub max_depth: usize,
    pub max_children_per_node: usize,
    /// `0` means no explicit result limit.
    pub max_results: usize,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            max_depth: 8,
            max_children_per_node: 128,
            max_results: 32,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Size {
    pub width: f64,
    pub height: f64,
}

impl Size {
    pub const fn new(width: f64, height: f64) -> Self {
        Self { width, height }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub origin: Point,
    pub size: Size,
}

impl Rect {
    pub const fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            origin: Point { x, y },
            size: Size { width, height },
        }
    }

    pub fn x(&self) -> f64 {
        self.origin.x
    }

    pub fn y(&self) -> f64 {
        self.origin.y
    }

    pub fn width(&self) -> f64 {
        self.size.width
    }

    pub fn height(&self) -> f64 {
        self.size.height
    }

    pub fn min_x(&self) -> f64 {
        self.origin.x
    }

    pub fn min_y(&self) -> f64 {
        self.origin.y
    }

    pub fn max_x(&self) -> f64 {
        self.origin.x + self.size.width
    }

    pub fn max_y(&self) -> f64 {
        self.origin.y + self.size.height
    }

    pub fn center(&self) -> Point {
        Point::new(
            self.origin.x + self.size.width / 2.0,
            self.origin.y + self.size.height / 2.0,
        )
    }

    pub fn contains_point(&self, point: Point) -> bool {
        point.x >= self.min_x()
            && point.x <= self.max_x()
            && point.y >= self.min_y()
            && point.y <= self.max_y()
    }

    pub fn inset(&self, dx: f64, dy: f64) -> Self {
        Self::new(
            self.origin.x + dx,
            self.origin.y + dy,
            (self.size.width - dx * 2.0).max(0.0),
            (self.size.height - dy * 2.0).max(0.0),
        )
    }

    pub fn translate(&self, dx: f64, dy: f64) -> Self {
        Self::new(
            self.origin.x + dx,
            self.origin.y + dy,
            self.size.width,
            self.size.height,
        )
    }

    pub fn with_origin(&self, origin: Point) -> Self {
        Self { origin, size: self.size }
    }

    pub fn with_size(&self, size: Size) -> Self {
        Self { origin: self.origin, size }
    }

    pub fn edge_distance(&self, other: &Rect) -> f64 {
        (self.origin.x - other.origin.x).abs()
            + (self.origin.y - other.origin.y).abs()
            + (self.size.width - other.size.width).abs()
            + (self.size.height - other.size.height).abs()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextRange {
    pub location: i64,
    pub length: i64,
}

impl TextRange {
    pub const fn new(location: i64, length: i64) -> Self {
        Self { location, length }
    }
}

/// Query options for CoreGraphics window metadata.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WindowQuery {
    /// Match only currently visible/on-screen windows.
    pub on_screen_only: bool,
    /// Include desktop elements such as the wallpaper and desktop icons.
    pub include_desktop_elements: bool,
    /// Restrict the query to a single owner process.
    pub owner_pid: Option<i32>,
    /// Restrict the query to a CoreGraphics layer. Normal application windows are usually layer 0.
    pub layer: Option<i64>,
    /// Drop fully transparent or nearly transparent windows.
    pub min_alpha: Option<f64>,
}

impl Default for WindowQuery {
    fn default() -> Self {
        Self {
            on_screen_only: true,
            include_desktop_elements: false,
            owner_pid: None,
            layer: None,
            min_alpha: Some(0.01),
        }
    }
}

impl WindowQuery {
    pub const fn all() -> Self {
        Self {
            on_screen_only: false,
            include_desktop_elements: true,
            owner_pid: None,
            layer: None,
            min_alpha: None,
        }
    }

    pub const fn on_screen() -> Self {
        Self {
            on_screen_only: true,
            include_desktop_elements: false,
            owner_pid: None,
            layer: None,
            min_alpha: Some(0.01),
        }
    }

    pub fn owner_pid(mut self, pid: i32) -> Self {
        self.owner_pid = Some(pid);
        self
    }

    pub fn layer(mut self, layer: i64) -> Self {
        self.layer = Some(layer);
        self
    }

    pub fn regular_windows(mut self) -> Self {
        self.layer = Some(0);
        self.min_alpha = Some(0.01);
        self.include_desktop_elements = false;
        self
    }

    pub fn include_desktop_elements(mut self, include: bool) -> Self {
        self.include_desktop_elements = include;
        self
    }

    pub fn matches(&self, window: &WindowInfo) -> bool {
        if let Some(pid) = self.owner_pid {
            if window.owner_pid != pid {
                return false;
            }
        }
        if let Some(layer) = self.layer {
            if window.layer != layer {
                return false;
            }
        }
        if let Some(min_alpha) = self.min_alpha {
            if window.alpha.unwrap_or(1.0) < min_alpha {
                return false;
            }
        }
        if self.on_screen_only && window.is_on_screen == Some(false) {
            return false;
        }
        true
    }
}

/// CoreGraphics metadata for a desktop window.
///
/// Use this when building screenshot pickers, switchers, window managers, or bridges
/// from AX windows to capture APIs. AX does not expose `CGWindowID`; CoreGraphics does.
#[derive(Debug, Clone, PartialEq)]
pub struct WindowInfo {
    pub id: u32,
    pub owner_pid: i32,
    pub owner_name: Option<String>,
    pub title: Option<String>,
    pub bounds: Rect,
    pub layer: i64,
    pub alpha: Option<f64>,
    pub is_on_screen: Option<bool>,
    pub sharing_state: Option<i64>,
    pub memory_usage: Option<i64>,
}

impl WindowInfo {
    pub fn contains_point(&self, point: Point) -> bool {
        self.bounds.contains_point(point)
    }

    pub fn center(&self) -> Point {
        self.bounds.center()
    }

    pub fn is_regular_application_window(&self) -> bool {
        self.layer == 0 && self.alpha.unwrap_or(1.0) > 0.0 && self.is_on_screen != Some(false)
    }
}

/// CoreGraphics display metadata for monitor-aware window management and picking.
#[derive(Debug, Clone, PartialEq)]
pub struct DisplayInfo {
    pub id: u32,
    pub bounds: Rect,
    pub is_main: bool,
}

impl DisplayInfo {
    pub fn contains_point(&self, point: Point) -> bool {
        self.bounds.contains_point(point)
    }

    pub fn center(&self) -> Point {
        self.bounds.center()
    }
}

/// A value returned by the Accessibility API.
#[derive(Debug, Clone, PartialEq)]
pub enum AxValue {
    String(String),
    Bool(bool),
    I64(i64),
    F64(f64),
    Element(AxElement),
    Elements(Vec<AxElement>),
    Point(Point),
    Size(Size),
    Rect(Rect),
    Range(TextRange),
    Array(Vec<AxValue>),
    Null,
    Unsupported(String),
}

/// A settable or parameter value for AX calls.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SettableValue<'a> {
    String(&'a str),
    Bool(bool),
    I64(i64),
    F64(f64),
    Point(Point),
    Size(Size),
    Rect(Rect),
    Range(TextRange),
    Element(&'a AxElement),
}

/// Snapshot that is cheap to send across threads and into GPUI entities.
#[derive(Debug, Clone, PartialEq)]
pub struct ElementSnapshot {
    pub pid: Option<i32>,
    pub role: Option<String>,
    pub subrole: Option<String>,
    pub role_description: Option<String>,
    pub title: Option<String>,
    pub value: Option<String>,
    pub description: Option<String>,
    pub identifier: Option<String>,
    pub enabled: Option<bool>,
    pub focused: Option<bool>,
    pub selected: Option<bool>,
    pub visible: Option<bool>,
    pub expanded: Option<bool>,
    pub main: Option<bool>,
    pub minimized: Option<bool>,
    pub hidden: Option<bool>,
    pub frame: Option<Rect>,
    pub children: Vec<ElementSnapshot>,
}

impl ElementSnapshot {
    pub fn label(&self) -> String {
        self.title
            .clone()
            .or_else(|| self.value.clone())
            .or_else(|| self.description.clone())
            .or_else(|| self.identifier.clone())
            .unwrap_or_default()
    }
}

/// AX observer event. This type is `Send` and intended for UI frameworks.
#[derive(Debug, Clone, PartialEq)]
pub struct AxEvent {
    pub pid: i32,
    pub notification: String,
    pub element: Option<ElementSnapshot>,
}

/// Main entry point for macOS Accessibility reads and writes.
#[derive(Clone)]
pub struct AxClient(platform::AxClientImpl);

impl fmt::Debug for AxClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AxClient").finish_non_exhaustive()
    }
}

impl AxClient {
    pub fn new(options: AxClientOptions) -> Result<Self> {
        platform::AxClientImpl::new(options).map(Self)
    }

    pub fn trusted(prompt: bool) -> Result<bool> {
        platform::AxClientImpl::trusted(prompt)
    }

    pub fn system_wide(&self) -> Result<AxElement> {
        self.0.system_wide().map(AxElement)
    }

    pub fn application(&self, pid: i32) -> Result<AxElement> {
        self.0.application(pid).map(AxElement)
    }

    pub fn focused_application(&self) -> Result<AxElement> {
        self.0.focused_application().map(AxElement)
    }

    pub fn focused_element(&self) -> Result<AxElement> {
        self.0.focused_element().map(AxElement)
    }

    pub fn focused_window(&self) -> Result<AxElement> {
        self.focused_application()?.focused_window()
    }

    pub fn element_at_position(&self, point: Point) -> Result<AxElement> {
        self.0.element_at_position(point).map(AxElement)
    }

    pub fn mouse_location(&self) -> Result<Point> {
        self.0.mouse_location()
    }

    pub fn element_at_mouse(&self) -> Result<AxElement> {
        self.element_at_position(self.mouse_location()?)
    }

    /// Returns CoreGraphics window metadata for the current user session.
    pub fn window_list(&self, query: WindowQuery) -> Result<Vec<WindowInfo>> {
        self.0.window_list(query)
    }

    pub fn visible_windows(&self) -> Result<Vec<WindowInfo>> {
        self.window_list(WindowQuery::default().regular_windows())
    }

    pub fn active_displays(&self) -> Result<Vec<DisplayInfo>> {
        self.0.active_displays()
    }

    pub fn main_display(&self) -> Result<Option<DisplayInfo>> {
        Ok(self.active_displays()?.into_iter().find(|display| display.is_main))
    }

    pub fn display_containing_point(&self, point: Point) -> Result<Option<DisplayInfo>> {
        Ok(self
            .active_displays()?
            .into_iter()
            .find(|display| display.contains_point(point)))
    }

    pub fn display_containing_rect(&self, rect: Rect) -> Result<Option<DisplayInfo>> {
        let mut displays = self.active_displays()?;
        displays.sort_by(|a, b| {
            intersection_area(&b.bounds, &rect)
                .partial_cmp(&intersection_area(&a.bounds, &rect))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        Ok(displays.into_iter().find(|display| intersection_area(&display.bounds, &rect) > 0.0))
    }

    pub fn windows_for_pid(&self, pid: i32) -> Result<Vec<WindowInfo>> {
        self.window_list(WindowQuery::default().regular_windows().owner_pid(pid))
    }

    pub fn running_window_owner_pids(&self) -> Result<Vec<i32>> {
        let mut seen = HashSet::new();
        let mut pids = Vec::new();
        for window in self.visible_windows()? {
            if seen.insert(window.owner_pid) {
                pids.push(window.owner_pid);
            }
        }
        Ok(pids)
    }

    pub fn window_at_position(&self, point: Point, query: WindowQuery) -> Result<Option<WindowInfo>> {
        Ok(self
            .window_list(query)?
            .into_iter()
            .find(|window| window.contains_point(point)))
    }

    pub fn window_at_mouse(&self, query: WindowQuery) -> Result<Option<WindowInfo>> {
        self.window_at_position(self.mouse_location()?, query)
    }

    /// Best-effort bridge from an AX window element to the CoreGraphics `CGWindowID` metadata.
    pub fn window_info_for_element(&self, element: &AxElement) -> Result<Option<WindowInfo>> {
        let pid = element.pid()?;
        let frame = element.frame().ok().flatten();
        let title = element.title().ok().flatten();
        let mut candidates = self.windows_for_pid(pid)?;

        if let Some(title) = title.as_ref().filter(|title| !title.is_empty()) {
            let exact = candidates
                .iter()
                .filter(|window| window.title.as_deref() == Some(title.as_str()))
                .cloned()
                .collect::<Vec<_>>();
            if !exact.is_empty() {
                candidates = exact;
            }
        }

        if let Some(frame) = frame {
            candidates.sort_by(|a, b| {
                a.bounds
                    .edge_distance(&frame)
                    .partial_cmp(&b.bounds.edge_distance(&frame))
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        }

        Ok(candidates.into_iter().next())
    }

    pub fn focused_window_info(&self) -> Result<Option<WindowInfo>> {
        let window = self.focused_window()?;
        self.window_info_for_element(&window)
    }

    /// Observe notifications on an application process. The returned observer stops on drop.
    pub fn observe_application<I, S>(&self, pid: i32, notifications: I) -> Result<(AxObserver, mpsc::Receiver<AxEvent>)>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let notifications = notifications.into_iter().map(Into::into).collect::<Vec<_>>();
        let (inner, rx) = self.0.observe_application(pid, notifications)?;
        Ok((AxObserver(inner), rx))
    }

    /// Observe the same notifications across many application processes and merge them into one receiver.
    pub fn observe_applications<P, I, S>(
        &self,
        pids: P,
        notifications: I,
    ) -> Result<(AxObserverGroup, mpsc::Receiver<AxEvent>)>
    where
        P: IntoIterator<Item = i32>,
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let notifications = notifications.into_iter().map(Into::into).collect::<Vec<_>>();
        let (merged_tx, merged_rx) = mpsc::channel();
        let mut observers = Vec::new();
        let mut forwarders = Vec::new();

        for pid in pids {
            let (observer, rx) = self.observe_application(pid, notifications.clone())?;
            let tx = merged_tx.clone();
            let forwarder = thread::spawn(move || {
                for event in rx {
                    if tx.send(event).is_err() {
                        break;
                    }
                }
            });
            observers.push(observer);
            forwarders.push(forwarder);
        }

        drop(merged_tx);
        Ok((AxObserverGroup { observers, forwarders }, merged_rx))
    }

    /// Observe common window-manager notifications on the apps that currently own visible windows.
    pub fn observe_visible_window_apps<I, S>(&self, notifications: I) -> Result<(AxObserverGroup, mpsc::Receiver<AxEvent>)>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.observe_applications(self.running_window_owner_pids()?, notifications)
    }
}

/// An accessibility object. On macOS this owns an `AXUIElementRef`.
#[derive(Clone)]
pub struct AxElement(platform::AxElementImpl);

impl fmt::Debug for AxElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AxElement")
            .field("pid", &self.pid().ok())
            .field("role", &self.string_attribute(attr::ROLE).ok().flatten())
            .field("title", &self.string_attribute(attr::TITLE).ok().flatten())
            .finish()
    }
}

impl PartialEq for AxElement {
    fn eq(&self, other: &Self) -> bool {
        self.0.ptr_eq(&other.0)
    }
}

impl AxElement {
    pub fn pid(&self) -> Result<i32> {
        self.0.pid()
    }

    pub fn attribute_names(&self) -> Result<Vec<String>> {
        self.0.attribute_names()
    }

    pub fn parameterized_attribute_names(&self) -> Result<Vec<String>> {
        self.0.parameterized_attribute_names()
    }

    pub fn action_names(&self) -> Result<Vec<String>> {
        self.0.action_names()
    }

    pub fn attribute(&self, attribute: impl AsRef<str>) -> Result<Option<AxValue>> {
        self.0.attribute(attribute.as_ref()).map(|opt| opt.map(AxValue::from))
    }

    pub fn parameterized_attribute(
        &self,
        attribute: impl AsRef<str>,
        parameter: SettableValue<'_>,
    ) -> Result<Option<AxValue>> {
        self.0
            .parameterized_attribute(attribute.as_ref(), parameter)
            .map(|opt| opt.map(AxValue::from))
    }

    pub fn string_attribute(&self, attribute: impl AsRef<str>) -> Result<Option<String>> {
        self.0.string_attribute(attribute.as_ref())
    }

    pub fn bool_attribute(&self, attribute: impl AsRef<str>) -> Result<Option<bool>> {
        self.0.bool_attribute(attribute.as_ref())
    }

    pub fn point_attribute(&self, attribute: impl AsRef<str>) -> Result<Option<Point>> {
        match self.attribute(attribute)? {
            Some(AxValue::Point(value)) => Ok(Some(value)),
            Some(AxValue::Null) | None => Ok(None),
            Some(other) => Err(AxError::TypeMismatch {
                expected: "CGPoint AXValue",
                actual: ax_value_kind(&other).to_string(),
            }),
        }
    }

    pub fn size_attribute(&self, attribute: impl AsRef<str>) -> Result<Option<Size>> {
        match self.attribute(attribute)? {
            Some(AxValue::Size(value)) => Ok(Some(value)),
            Some(AxValue::Null) | None => Ok(None),
            Some(other) => Err(AxError::TypeMismatch {
                expected: "CGSize AXValue",
                actual: ax_value_kind(&other).to_string(),
            }),
        }
    }

    pub fn rect_attribute(&self, attribute: impl AsRef<str>) -> Result<Option<Rect>> {
        match self.attribute(attribute)? {
            Some(AxValue::Rect(value)) => Ok(Some(value)),
            Some(AxValue::Null) | None => Ok(None),
            Some(other) => Err(AxError::TypeMismatch {
                expected: "CGRect AXValue",
                actual: ax_value_kind(&other).to_string(),
            }),
        }
    }

    pub fn range_attribute(&self, attribute: impl AsRef<str>) -> Result<Option<TextRange>> {
        match self.attribute(attribute)? {
            Some(AxValue::Range(value)) => Ok(Some(value)),
            Some(AxValue::Null) | None => Ok(None),
            Some(other) => Err(AxError::TypeMismatch {
                expected: "CFRange AXValue",
                actual: ax_value_kind(&other).to_string(),
            }),
        }
    }

    pub fn element_attribute(&self, attribute: impl AsRef<str>) -> Result<Option<AxElement>> {
        self.0.element_attribute(attribute.as_ref()).map(|opt| opt.map(AxElement))
    }

    pub fn elements_attribute(&self, attribute: impl AsRef<str>) -> Result<Vec<AxElement>> {
        match self.attribute(attribute)? {
            Some(AxValue::Elements(values)) => Ok(values),
            Some(AxValue::Element(value)) => Ok(vec![value]),
            Some(AxValue::Array(values)) => values
                .into_iter()
                .map(|value| match value {
                    AxValue::Element(element) => Ok(element),
                    other => Err(AxError::TypeMismatch {
                        expected: "array of AXUIElement",
                        actual: ax_value_kind(&other).to_string(),
                    }),
                })
                .collect(),
            Some(AxValue::Null) | None => Ok(Vec::new()),
            Some(other) => Err(AxError::TypeMismatch {
                expected: "array of AXUIElement",
                actual: ax_value_kind(&other).to_string(),
            }),
        }
    }

    pub fn children(&self) -> Result<Vec<AxElement>> {
        self.0.children().map(|items| items.into_iter().map(AxElement).collect())
    }

    pub fn frame(&self) -> Result<Option<Rect>> {
        self.0.frame()
    }

    pub fn snapshot(&self, options: TreeOptions) -> Result<ElementSnapshot> {
        self.0.snapshot(options)
    }

    pub fn is_attribute_settable(&self, attribute: impl AsRef<str>) -> Result<bool> {
        self.0.is_attribute_settable(attribute.as_ref())
    }

    pub fn set_attribute(&self, attribute: impl AsRef<str>, value: SettableValue<'_>) -> Result<()> {
        self.0.set_attribute(attribute.as_ref(), value)
    }

    pub fn set_bool_attribute(&self, attribute: impl AsRef<str>, value: bool) -> Result<()> {
        self.set_attribute(attribute, SettableValue::Bool(value))
    }

    pub fn set_value(&self, value: &str) -> Result<()> {
        self.set_attribute(attr::VALUE, SettableValue::String(value))
    }

    pub fn set_position(&self, point: Point) -> Result<()> {
        self.set_attribute(attr::POSITION, SettableValue::Point(point))
    }

    pub fn set_size(&self, size: Size) -> Result<()> {
        self.set_attribute(attr::SIZE, SettableValue::Size(size))
    }

    pub fn set_frame(&self, rect: Rect) -> Result<()> {
        // Many apps accept position and size but not a single CGRect attribute.
        self.set_position(rect.origin)?;
        self.set_size(rect.size)
    }

    pub fn move_by(&self, dx: f64, dy: f64) -> Result<()> {
        let frame = self.frame()?.ok_or_else(|| AxError::NoValue {
            attribute: attr::POSITION.to_string(),
        })?;
        self.set_position(Point::new(frame.origin.x + dx, frame.origin.y + dy))
    }

    pub fn resize_by(&self, dw: f64, dh: f64) -> Result<()> {
        let frame = self.frame()?.ok_or_else(|| AxError::NoValue {
            attribute: attr::SIZE.to_string(),
        })?;
        self.set_size(Size::new(
            (frame.size.width + dw).max(1.0),
            (frame.size.height + dh).max(1.0),
        ))
    }

    pub fn perform_action(&self, action: impl AsRef<str>) -> Result<()> {
        self.0.perform_action(action.as_ref())
    }

    pub fn press(&self) -> Result<()> {
        self.perform_action(action::PRESS)
    }

    pub fn raise(&self) -> Result<()> {
        self.perform_action(action::RAISE)
    }

    pub fn show_menu(&self) -> Result<()> {
        self.perform_action(action::SHOW_MENU)
    }

    pub fn scroll_to_visible(&self) -> Result<()> {
        self.perform_action(action::SCROLL_TO_VISIBLE)
    }

    pub fn role(&self) -> Result<Option<String>> {
        self.string_attribute(attr::ROLE)
    }

    pub fn subrole(&self) -> Result<Option<String>> {
        self.string_attribute(attr::SUBROLE)
    }

    pub fn title(&self) -> Result<Option<String>> {
        self.string_attribute(attr::TITLE)
    }

    pub fn value_string(&self) -> Result<Option<String>> {
        self.string_attribute(attr::VALUE)
    }

    pub fn description(&self) -> Result<Option<String>> {
        self.string_attribute(attr::DESCRIPTION)
    }

    pub fn identifier(&self) -> Result<Option<String>> {
        self.string_attribute(attr::IDENTIFIER)
    }

    pub fn label(&self) -> Result<String> {
        Ok(self
            .title()?
            .or_else(|| self.value_string().ok().flatten())
            .or_else(|| self.description().ok().flatten())
            .or_else(|| self.identifier().ok().flatten())
            .unwrap_or_default())
    }

    pub fn is_role(&self, expected_role: &str) -> Result<bool> {
        Ok(self.role()?.as_deref() == Some(expected_role))
    }

    pub fn is_enabled(&self) -> Result<Option<bool>> {
        self.bool_attribute(attr::ENABLED)
    }

    pub fn is_focused(&self) -> Result<Option<bool>> {
        self.bool_attribute(attr::FOCUSED)
    }

    pub fn is_selected(&self) -> Result<Option<bool>> {
        self.bool_attribute(attr::SELECTED)
    }

    pub fn is_minimized(&self) -> Result<Option<bool>> {
        self.bool_attribute(attr::MINIMIZED)
    }

    pub fn is_main(&self) -> Result<Option<bool>> {
        self.bool_attribute(attr::MAIN)
    }

    pub fn parent(&self) -> Result<Option<AxElement>> {
        self.element_attribute(attr::PARENT)
    }

    pub fn window(&self) -> Result<Option<AxElement>> {
        self.element_attribute(attr::WINDOW)
    }

    pub fn top_level_element(&self) -> Result<Option<AxElement>> {
        self.element_attribute(attr::TOP_LEVEL_UI_ELEMENT)
    }

    pub fn windows(&self) -> Result<Vec<AxElement>> {
        self.elements_attribute(attr::WINDOWS)
    }

    pub fn focused_window(&self) -> Result<AxElement> {
        self.element_attribute(attr::FOCUSED_WINDOW)?.ok_or_else(|| AxError::NoValue {
            attribute: attr::FOCUSED_WINDOW.to_string(),
        })
    }

    pub fn main_window(&self) -> Result<Option<AxElement>> {
        self.element_attribute(attr::MAIN_WINDOW)
    }

    pub fn menu_bar(&self) -> Result<Option<AxElement>> {
        self.element_attribute(attr::MENU_BAR)
    }

    pub fn close_button(&self) -> Result<Option<AxElement>> {
        self.element_attribute(attr::CLOSE_BUTTON)
    }

    pub fn minimize_button(&self) -> Result<Option<AxElement>> {
        self.element_attribute(attr::MINIMIZE_BUTTON)
    }

    pub fn zoom_button(&self) -> Result<Option<AxElement>> {
        self.element_attribute(attr::ZOOM_BUTTON)
    }

    pub fn full_screen_button(&self) -> Result<Option<AxElement>> {
        self.element_attribute(attr::FULL_SCREEN_BUTTON)
    }

    pub fn bring_to_front(&self) -> Result<()> {
        if self.is_role(role::APPLICATION).unwrap_or(false) {
            self.set_bool_attribute(attr::FRONTMOST, true)
        } else {
            let _ = self.set_bool_attribute(attr::MAIN, true);
            self.raise()
        }
    }

    pub fn focus(&self) -> Result<()> {
        match self.set_bool_attribute(attr::FOCUSED, true) {
            Ok(()) => Ok(()),
            Err(_) => self.raise(),
        }
    }

    pub fn set_minimized(&self, minimized: bool) -> Result<()> {
        self.set_bool_attribute(attr::MINIMIZED, minimized)
    }

    pub fn minimize(&self) -> Result<()> {
        if let Some(button) = self.minimize_button()? {
            button.press()
        } else {
            self.set_minimized(true)
        }
    }

    pub fn unminimize(&self) -> Result<()> {
        self.set_minimized(false)
    }

    pub fn close_window(&self) -> Result<()> {
        if let Some(button) = self.close_button()? {
            button.press()
        } else {
            self.perform_action(action::CANCEL)
        }
    }

    pub fn zoom_window(&self) -> Result<()> {
        self.zoom_button()?.ok_or_else(|| AxError::NoValue {
            attribute: attr::ZOOM_BUTTON.to_string(),
        })?.press()
    }

    pub fn toggle_full_screen(&self) -> Result<()> {
        self.full_screen_button()?.ok_or_else(|| AxError::NoValue {
            attribute: attr::FULL_SCREEN_BUTTON.to_string(),
        })?.press()
    }

    pub fn selected_text(&self) -> Result<Option<String>> {
        self.string_attribute(attr::SELECTED_TEXT)
    }

    pub fn selected_text_range(&self) -> Result<Option<TextRange>> {
        self.range_attribute(attr::SELECTED_TEXT_RANGE)
    }

    pub fn visible_character_range(&self) -> Result<Option<TextRange>> {
        self.range_attribute(attr::VISIBLE_CHARACTER_RANGE)
    }

    pub fn string_for_range(&self, range: TextRange) -> Result<Option<String>> {
        match self.parameterized_attribute(param::STRING_FOR_RANGE, SettableValue::Range(range))? {
            Some(AxValue::String(value)) => Ok(Some(value)),
            Some(AxValue::Null) | None => Ok(None),
            Some(other) => Err(AxError::TypeMismatch {
                expected: "CFString",
                actual: ax_value_kind(&other).to_string(),
            }),
        }
    }

    pub fn bounds_for_range(&self, range: TextRange) -> Result<Option<Rect>> {
        match self.parameterized_attribute(param::BOUNDS_FOR_RANGE, SettableValue::Range(range))? {
            Some(AxValue::Rect(value)) => Ok(Some(value)),
            Some(AxValue::Null) | None => Ok(None),
            Some(other) => Err(AxError::TypeMismatch {
                expected: "CGRect AXValue",
                actual: ax_value_kind(&other).to_string(),
            }),
        }
    }

    pub fn range_for_position(&self, point: Point) -> Result<Option<TextRange>> {
        match self.parameterized_attribute(param::RANGE_FOR_POSITION, SettableValue::Point(point))? {
            Some(AxValue::Range(value)) => Ok(Some(value)),
            Some(AxValue::Null) | None => Ok(None),
            Some(other) => Err(AxError::TypeMismatch {
                expected: "CFRange AXValue",
                actual: ax_value_kind(&other).to_string(),
            }),
        }
    }

    pub fn find_all<P>(&self, options: SearchOptions, mut predicate: P) -> Result<Vec<AxElement>>
    where
        P: FnMut(&AxElement) -> Result<bool>,
    {
        let mut results = Vec::new();
        self.find_all_inner(options, 0, &mut predicate, &mut results)?;
        Ok(results)
    }

    pub fn find_first<P>(&self, mut options: SearchOptions, predicate: P) -> Result<Option<AxElement>>
    where
        P: FnMut(&AxElement) -> Result<bool>,
    {
        options.max_results = 1;
        Ok(self.find_all(options, predicate)?.into_iter().next())
    }

    pub fn find_first_by_role(&self, role: impl AsRef<str>, options: SearchOptions) -> Result<Option<AxElement>> {
        let role = role.as_ref().to_string();
        self.find_first(options, move |element| Ok(element.role()?.as_deref() == Some(role.as_str())))
    }

    pub fn find_all_by_role(&self, role: impl AsRef<str>, options: SearchOptions) -> Result<Vec<AxElement>> {
        let role = role.as_ref().to_string();
        self.find_all(options, move |element| Ok(element.role()?.as_deref() == Some(role.as_str())))
    }

    pub fn find_first_by_title(&self, title: impl AsRef<str>, options: SearchOptions) -> Result<Option<AxElement>> {
        let title = title.as_ref().to_string();
        self.find_first(options, move |element| Ok(element.title()?.as_deref() == Some(title.as_str())))
    }

    pub fn find_first_by_label_containing(
        &self,
        needle: impl AsRef<str>,
        options: SearchOptions,
    ) -> Result<Option<AxElement>> {
        let needle = needle.as_ref().to_lowercase();
        self.find_first(options, move |element| {
            Ok(element.label()?.to_lowercase().contains(&needle))
        })
    }

    pub fn menu_item_by_path<S: AsRef<str>>(&self, path: &[S]) -> Result<Option<AxElement>> {
        if path.is_empty() {
            return Ok(None);
        }

        let mut current = if self.is_role(role::MENU_BAR).unwrap_or(false) {
            self.clone()
        } else {
            match self.menu_bar()? {
                Some(menu_bar) => menu_bar,
                None => return Ok(None),
            }
        };

        for (index, segment) in path.iter().enumerate() {
            let segment = segment.as_ref();
            let Some(next) = find_menu_child(&current, segment)? else {
                return Ok(None);
            };
            current = next;
            if index + 1 < path.len() {
                let _ = current.perform_action(action::SHOW_MENU);
                let _ = current.perform_action(action::PRESS);
            }
        }

        Ok(Some(current))
    }

    pub fn press_menu_item_by_path<S: AsRef<str>>(&self, path: &[S]) -> Result<bool> {
        let Some(item) = self.menu_item_by_path(path)? else {
            return Ok(false);
        };
        item.press()?;
        Ok(true)
    }

    fn find_all_inner<P>(
        &self,
        options: SearchOptions,
        depth: usize,
        predicate: &mut P,
        results: &mut Vec<AxElement>,
    ) -> Result<()>
    where
        P: FnMut(&AxElement) -> Result<bool>,
    {
        if options.max_results != 0 && results.len() >= options.max_results {
            return Ok(());
        }

        if predicate(self)? {
            results.push(self.clone());
            if options.max_results != 0 && results.len() >= options.max_results {
                return Ok(());
            }
        }

        if depth >= options.max_depth {
            return Ok(());
        }

        let mut children = match self.children() {
            Ok(children) => children,
            Err(_) => return Ok(()),
        };
        if children.len() > options.max_children_per_node {
            children.truncate(options.max_children_per_node);
        }

        for child in children {
            child.find_all_inner(options, depth + 1, predicate, results)?;
            if options.max_results != 0 && results.len() >= options.max_results {
                break;
            }
        }

        Ok(())
    }
}

/// RAII handle for an AXObserver run-loop thread.
pub struct AxObserver(platform::AxObserverImpl);

impl fmt::Debug for AxObserver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AxObserver").finish_non_exhaustive()
    }
}

/// RAII handle for many AX observers merged into one event receiver.
pub struct AxObserverGroup {
    observers: Vec<AxObserver>,
    forwarders: Vec<thread::JoinHandle<()>>,
}

impl fmt::Debug for AxObserverGroup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AxObserverGroup")
            .field("observer_count", &self.observers.len())
            .finish_non_exhaustive()
    }
}

impl Drop for AxObserverGroup {
    fn drop(&mut self) {
        self.observers.clear();
        while let Some(handle) = self.forwarders.pop() {
            let _ = handle.join();
        }
    }
}

/// Platform-private AX value. Converted into public [`AxValue`] at the boundary.
#[derive(Debug, Clone)]
pub(crate) enum PlatformValue {
    String(String),
    Bool(bool),
    I64(i64),
    F64(f64),
    Element(platform::AxElementImpl),
    Elements(Vec<platform::AxElementImpl>),
    Point(Point),
    Size(Size),
    Rect(Rect),
    Range(TextRange),
    Array(Vec<PlatformValue>),
    Null,
    Unsupported(String),
}

impl From<PlatformValue> for AxValue {
    fn from(value: PlatformValue) -> Self {
        match value {
            PlatformValue::String(v) => AxValue::String(v),
            PlatformValue::Bool(v) => AxValue::Bool(v),
            PlatformValue::I64(v) => AxValue::I64(v),
            PlatformValue::F64(v) => AxValue::F64(v),
            PlatformValue::Element(v) => AxValue::Element(AxElement(v)),
            PlatformValue::Elements(v) => AxValue::Elements(v.into_iter().map(AxElement).collect()),
            PlatformValue::Point(v) => AxValue::Point(v),
            PlatformValue::Size(v) => AxValue::Size(v),
            PlatformValue::Rect(v) => AxValue::Rect(v),
            PlatformValue::Range(v) => AxValue::Range(v),
            PlatformValue::Array(v) => AxValue::Array(v.into_iter().map(AxValue::from).collect()),
            PlatformValue::Null => AxValue::Null,
            PlatformValue::Unsupported(v) => AxValue::Unsupported(v),
        }
    }
}

fn ax_value_kind(value: &AxValue) -> &'static str {
    match value {
        AxValue::String(_) => "CFString",
        AxValue::Bool(_) => "CFBoolean",
        AxValue::I64(_) | AxValue::F64(_) => "CFNumber",
        AxValue::Element(_) => "AXUIElement",
        AxValue::Elements(_) => "array of AXUIElement",
        AxValue::Point(_) => "CGPoint AXValue",
        AxValue::Size(_) => "CGSize AXValue",
        AxValue::Rect(_) => "CGRect AXValue",
        AxValue::Range(_) => "CFRange AXValue",
        AxValue::Array(_) => "CFArray",
        AxValue::Null => "null",
        AxValue::Unsupported(_) => "unsupported CFType",
    }
}

fn intersection_area(a: &Rect, b: &Rect) -> f64 {
    let min_x = a.min_x().max(b.min_x());
    let min_y = a.min_y().max(b.min_y());
    let max_x = a.max_x().min(b.max_x());
    let max_y = a.max_y().min(b.max_y());
    ((max_x - min_x).max(0.0)) * ((max_y - min_y).max(0.0))
}

fn find_menu_child(container: &AxElement, title: &str) -> Result<Option<AxElement>> {
    let mut candidates = container.children().unwrap_or_default();
    let direct = candidates.clone();
    for child in direct {
        if let Ok(mut grandchildren) = child.children() {
            candidates.append(&mut grandchildren);
        }
    }

    for child in candidates {
        let child_title = child.title().ok().flatten();
        let child_label = child.label().unwrap_or_default();
        if child_title.as_deref() == Some(title) || child_label == title {
            return Ok(Some(child));
        }
    }

    Ok(None)
}
