# Menu Executor Reference

Detailed documentation for `src/menu_executor.rs` - executing menu actions via AX APIs.

## Table of Contents

1. [Error Types](#error-types)
2. [Execution Flow](#execution-flow)
3. [Public API](#public-api)
4. [Navigation Algorithm](#navigation-algorithm)
5. [AXPress Action](#axpress-action)

## Error Types

### MenuExecutorError

```rust
#[derive(Error, Debug)]
pub enum MenuExecutorError {
    #[error("Menu item at path {path:?} is disabled")]
    MenuItemDisabled { path: Vec<String> },

    #[error("Menu item {path:?} not found in {searched_in}")]
    MenuItemNotFound { path: Vec<String>, searched_in: String },

    #[error("Application {bundle_id} is not frontmost")]
    AppNotFrontmost { bundle_id: String },

    #[error("Menu structure changed - expected {expected_path:?}: {reason}")]
    MenuStructureChanged { expected_path: Vec<String>, reason: String },

    #[error("Accessibility permission required")]
    AccessibilityPermissionDenied,

    #[error("Failed to perform AXPress: {0}")]
    ActionFailed(String),
}
```

### Error Scenarios

| Error | When It Occurs | Recovery |
|-------|----------------|----------|
| `MenuItemDisabled` | Item exists but grayed out | Check state before executing |
| `MenuItemNotFound` | Path doesn't match menu structure | Rescan menus, verify path |
| `AppNotFrontmost` | Target app not active | Activate app first |
| `MenuStructureChanged` | Menu differs from expected | Invalidate cache, rescan |
| `AccessibilityPermissionDenied` | No AX permission | User must grant in System Preferences |
| `ActionFailed` | AXPress returned error | Element may have become invalid |

## Execution Flow

```
execute_menu_action("com.apple.Safari", ["File", "New Window"])
    |
    1. Validate menu_path is not empty
    |
    2. Check accessibility permission
    |
    3. Verify target app is frontmost
    |    - get_frontmost_app_info() returns (pid, bundle_id)
    |    - Compare bundle_id with target
    |
    4. Create AXUIElement for app
    |    - AXUIElementCreateApplication(pid)
    |
    5. Get AXMenuBar from app
    |
    6. Navigate menu path
    |    - For each item except last: open submenu (AXPress)
    |    - For last item: check enabled, then AXPress
    |
    7. Cleanup CFTypeRef resources
```

## Public API

### execute_menu_action

```rust
pub fn execute_menu_action(bundle_id: &str, menu_path: &[String]) -> Result<()>
```

**Arguments**:
- `bundle_id`: Target app (e.g., "com.apple.Safari")
- `menu_path`: Path to menu item (e.g., ["File", "New Window"])

**Requirements**:
- Accessibility permission granted
- Target app must be frontmost (not just menu bar owner)
- Menu path must exist and final item must be enabled

**Example**:
```rust
// Execute "File" -> "Save As..." in TextEdit
execute_menu_action("com.apple.TextEdit", &[
    "File".to_string(),
    "Save As...".to_string()
])?;
```

### validate_menu_path

```rust
pub fn validate_menu_path(path: &[String]) -> Result<()>
```

Simple validation that path is not empty.

## Navigation Algorithm

### navigate_and_execute_menu_path

```rust
fn navigate_and_execute_menu_path(
    menu_bar: AXUIElementRef, 
    menu_path: &[String]
) -> Result<()>
```

**Algorithm**:

```
current_container = menu_bar
path_so_far = []

for each (index, title) in menu_path:
    path_so_far.push(title)
    
    children = get_ax_children(current_container)
    menu_item = find_menu_item_by_title(children, title)
    
    if menu_item is None:
        return MenuItemNotFound
    
    if index == last:
        if not enabled:
            return MenuItemDisabled
        perform_ax_action(menu_item, "AXPress")
        return Ok
    else:
        # Open submenu to continue navigation
        submenu = open_menu_at_element(menu_item)
        current_container = submenu
```

### find_menu_item_by_title

```rust
fn find_menu_item_by_title(
    children: CFArrayRef,
    count: i64,
    title: &str
) -> Option<AXUIElementRef>
```

Iterates children, comparing AXTitle attribute to target title. Returns first match.

### open_menu_at_element

```rust
fn open_menu_at_element(element: AXUIElementRef) -> Result<AXUIElementRef>
```

1. Perform AXPress on element (opens dropdown/submenu)
2. Wait 50ms for menu to open
3. Get AXChildren of element
4. Find child with role "AXMenu"
5. Return the AXMenu element

**Why wait?** Menu opening is asynchronous. 50ms gives macOS time to:
- Animate the menu open
- Populate the accessibility hierarchy
- Make children available for querying

## AXPress Action

### perform_ax_action

```rust
fn perform_ax_action(element: AXUIElementRef, action: &str) -> Result<()>
```

Calls `AXUIElementPerformAction(element, "AXPress")`.

**AXError codes**:

| Code | Constant | Meaning |
|------|----------|---------|
| 0 | `kAXErrorSuccess` | Action performed |
| -25211 | `kAXErrorAPIDisabled` | AX disabled |
| -25215 | `kAXErrorActionUnsupported` | Element doesn't support action |
| -25204 | `kAXErrorCannotComplete` | Element disabled or invalid |

### Why App Must Be Frontmost

For `AXPress` to work:
1. App must be frontmost (receiving input)
2. Menu bar must be accessible
3. Menu item must be enabled

If app is only menu bar owner but not frontmost:
- Menu is visible but not interactive
- AXPress may fail or have no effect

**Workaround**: Activate app before executing:
```rust
// Pseudo-code
NSRunningApplication::activate(app);
std::thread::sleep(Duration::from_millis(100));
execute_menu_action(bundle_id, path)?;
```

## Instrumentation

Both public functions use tracing:
```rust
#[instrument(skip(menu_path), fields(bundle_id = %bundle_id, path = ?menu_path))]
pub fn execute_menu_action(...)
```

Debug logs emitted at:
- App verification
- Each intermediate menu open
- Final item press
