# Menu Bar Reader Reference

Detailed documentation for `src/menu_bar.rs` - AX-based menu bar reading.

## Table of Contents

1. [FFI Bindings](#ffi-bindings)
2. [AX Attributes](#ax-attributes)
3. [Data Types](#data-types)
4. [Public API](#public-api)
5. [Recursive Parsing](#recursive-parsing)
6. [Menu Bar Owner Detection](#menu-bar-owner-detection)

## FFI Bindings

### CoreFoundation

```rust
#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFRelease(cf: *const c_void);
    fn CFStringCreateWithCString(...) -> CFStringRef;
    fn CFStringGetCString(...) -> bool;
    fn CFArrayGetCount(array: CFArrayRef) -> i64;
    fn CFArrayGetValueAtIndex(array: CFArrayRef, index: i64) -> CFTypeRef;
    fn CFGetTypeID(cf: CFTypeRef) -> u64;
    fn CFStringGetTypeID() -> u64;
    fn CFNumberGetValue(...) -> bool;
    fn CFBooleanGetValue(boolean: CFTypeRef) -> bool;
}
```

### ApplicationServices (Accessibility)

```rust
#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXUIElementCreateApplication(pid: i32) -> AXUIElementRef;
    fn AXUIElementCopyAttributeValue(
        element: AXUIElementRef,
        attribute: CFStringRef,
        value: *mut CFTypeRef,
    ) -> i32;
}
```

### AXError Codes

| Code | Constant | Meaning |
|------|----------|---------|
| 0 | `kAXErrorSuccess` | Operation succeeded |
| -25211 | `kAXErrorAPIDisabled` | Accessibility disabled system-wide |
| -25212 | `kAXErrorNoValue` | Attribute has no value |

## AX Attributes

| Constant | Value | Description |
|----------|-------|-------------|
| `AX_MENU_BAR` | "AXMenuBar" | Menu bar element |
| `AX_CHILDREN` | "AXChildren" | Child elements array |
| `AX_TITLE` | "AXTitle" | Display title |
| `AX_ROLE` | "AXRole" | Element role type |
| `AX_ENABLED` | "AXEnabled" | Whether clickable |
| `AX_MENU_ITEM_CMD_CHAR` | "AXMenuItemCmdChar" | Shortcut key |
| `AX_MENU_ITEM_CMD_MODIFIERS` | "AXMenuItemCmdModifiers" | Modifier bitmask |

### AX Roles

| Role | Description |
|------|-------------|
| `AXMenuBarItem` | Top-level menu (File, Edit) |
| `AXMenu` | Container for menu items |
| `AXMenuItem` | Clickable menu item |

## Data Types

### ModifierFlags (bitflags)

```rust
bitflags! {
    pub struct ModifierFlags: u32 {
        const COMMAND = 256;   // Cmd/Meta
        const SHIFT = 512;     // Shift
        const OPTION = 2048;   // Alt/Option
        const CONTROL = 4096;  // Control
    }
}
```

### KeyboardShortcut

```rust
pub struct KeyboardShortcut {
    pub key: String,              // "S", "N", "Q"
    pub modifiers: ModifierFlags, // Combined flags
}

impl KeyboardShortcut {
    // Create from AX values (raw from accessibility API)
    pub fn from_ax_values(cmd_char: &str, cmd_modifiers: u32) -> Self;
    
    // Display string with Unicode symbols: "⌃⌥⇧⌘S"
    pub fn to_display_string(&self) -> String;
}
```

Display order follows macOS convention: Control (⌃) > Option (⌥) > Shift (⇧) > Command (⌘)

### MenuBarItem

```rust
pub struct MenuBarItem {
    pub title: String,
    pub enabled: bool,
    pub shortcut: Option<KeyboardShortcut>,
    pub children: Vec<MenuBarItem>,
    pub ax_element_path: Vec<usize>, // Navigation path for execution
}

impl MenuBarItem {
    pub fn separator(path: Vec<usize>) -> Self;
    pub fn is_separator(&self) -> bool;
}
```

### MenuCache (in-memory, deprecated)

```rust
pub struct MenuCache {
    pub bundle_id: String,
    pub menu_json: Option<String>,
    pub last_scanned: Option<Instant>,
}

impl MenuCache {
    pub fn is_stale(&self, max_age: Duration) -> bool;
}
```

> Note: Use `menu_cache.rs` SQLite implementation instead for persistence.

## Public API

### get_frontmost_menu_bar

```rust
pub fn get_frontmost_menu_bar() -> Result<Vec<MenuBarItem>>
```

Gets menu bar of the **menu bar owning** application (not necessarily frontmost).

**Important**: Script Kit is `LSUIElement`, so it doesn't own the menu bar even when active.

**Returns**: Top-level menu items (File, Edit, View...) with children up to 3 levels deep.

**Errors**:
- No accessibility permission
- No menu bar owner found
- Failed to read menu bar

### get_menu_bar_for_pid

```rust
pub fn get_menu_bar_for_pid(pid: i32) -> Result<Vec<MenuBarItem>>
```

Gets menu bar for specific application by PID. Use when you've pre-captured the target PID.

## Recursive Parsing

### Hierarchy Traversal

```
parse_menu_item(AXMenuBarItem, path=[0], depth=0)
    |
    +-- parse_submenu_children(element, path=[0], depth=0)
            |
            +-- finds AXMenu child
            +-- iterates AXMenuItem children
                    |
                    +-- parse_menu_item(AXMenuItem, path=[0,0], depth=1)
                            |
                            +-- parse_submenu_children(element, path=[0,0], depth=1)
                                    ... recursion up to MAX_MENU_DEPTH=3
```

### parse_menu_item

```rust
fn parse_menu_item(
    element: AXUIElementRef, 
    path: Vec<usize>, 
    depth: usize
) -> Option<MenuBarItem>
```

1. Check if separator (empty title or separator role) → return separator item
2. Get title, enabled state
3. Get keyboard shortcut (AXMenuItemCmdChar + AXMenuItemCmdModifiers)
4. If depth < MAX_MENU_DEPTH, recursively parse children
5. Return MenuBarItem with all data

### parse_submenu_children

```rust
fn parse_submenu_children(
    element: AXUIElementRef,
    parent_path: &[usize],
    depth: usize
) -> Vec<MenuBarItem>
```

1. Get AXChildren of element
2. Find child with role "AXMenu" (the actual menu container)
3. Get AXChildren of the AXMenu
4. For each AXMenuItem child, call parse_menu_item recursively
5. Build path by appending index to parent_path

### Separator Detection

```rust
fn is_menu_separator(element: AXUIElementRef) -> bool
```

Separator if ANY of:
- Role contains "Separator"
- Title is empty or whitespace-only
- No title attribute at all

Separator items have `title = "---"` and `enabled = false`.

## Menu Bar Owner Detection

### get_menu_bar_owner_pid

```rust
fn get_menu_bar_owner_pid() -> Result<i32>
```

Uses Objective-C runtime to query:
```objc
[[NSWorkspace sharedWorkspace] menuBarOwningApplication]
```

**Why not frontmostApplication?**

Script Kit is `LSUIElement` (accessory app). When activated:
- It becomes frontmost (receives input)
- But does NOT take menu bar ownership
- Previous app's menus remain visible

This is **intentional** - users want to interact with the previous app's menus.

### Memory Safety

All CFTypeRef values must be released after use:

```rust
fn cf_release(cf: CFTypeRef) {
    if !cf.is_null() {
        unsafe { CFRelease(cf); }
    }
}
```

Pattern used throughout:
```rust
let value = get_ax_attribute(element, attr)?;
// ... use value ...
cf_release(value);
```
