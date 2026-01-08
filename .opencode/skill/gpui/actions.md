# Actions and Key Dispatch in GPUI

GPUI is designed for keyboard-first interactivity. Actions convert keystrokes into type-safe operations.

## Defining Actions

### Simple Actions with `#[gpui::action]`

```rust
use gpui::action;

#[gpui::action]
struct MoveUp;

#[gpui::action]
struct MoveDown;

#[gpui::action]
struct SelectAll;
```

### Multiple Actions with `actions!` Macro

```rust
use gpui::actions;

// Define multiple actions in a namespace
actions!(menu, [MoveUp, MoveDown, MoveLeft, MoveRight, Select, Cancel]);

// Editor actions
actions!(editor, [Copy, Cut, Paste, Undo, Redo]);
```

### Complex Actions with Fields

```rust
use gpui::action;
use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub enum Direction { Up, Down, Left, Right }

#[gpui::action]
struct Move {
    direction: Direction,
    select: bool,
}

#[gpui::action]
struct GoToLine { line: u32 }
```

Complex actions must derive `Clone` and `Deserialize` for keymap JSON support.

## Handling Actions

### The `on_action()` Handler

```rust
impl Render for Menu {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .on_action(|this: &mut Self, action: &MoveUp, window, cx| {
                this.move_selection(-1);
                cx.notify();
            })
            .on_action(|this: &mut Self, action: &MoveDown, window, cx| {
                this.move_selection(1);
                cx.notify();
            })
    }
}
```

### Accessing Action Fields

```rust
.on_action(|this: &mut Self, action: &Move, window, cx| {
    let distance = if action.select { 10 } else { 1 };
    match action.direction {
        Direction::Up => this.move_up(distance),
        Direction::Down => this.move_down(distance),
        Direction::Left => this.move_left(distance),
        Direction::Right => this.move_right(distance),
    }
    cx.notify();
})
```

## Key Context

### Setting Key Context

```rust
impl Render for Editor {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .key_context("editor")  // Required for key bindings to dispatch
            .on_action(|this: &mut Self, _: &Copy, window, cx| {
                this.copy_selection();
            })
            .on_action(|this: &mut Self, _: &Paste, window, cx| {
                this.paste();
            })
    }
}
```

### Nested Contexts

```rust
impl Render for App {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .key_context("workspace")
            .child(
                div()
                    .key_context("editor")
                    .on_action(|this: &mut Self, _: &Save, window, cx| {
                        // Editor-specific save
                    })
            )
            .on_action(|this: &mut Self, _: &Quit, window, cx| {
                // Workspace-level quit
            })
    }
}
```

## Keymap Configuration

### JSON Keymap Format

```json
[
  {
    "context": "menu",
    "bindings": {
      "up": "menu::MoveUp",
      "down": "menu::MoveDown",
      "enter": "menu::Select",
      "escape": "menu::Cancel"
    }
  },
  {
    "context": "editor",
    "bindings": {
      "ctrl-c": "editor::Copy",
      "ctrl-v": "editor::Paste",
      "ctrl-z": "editor::Undo",
      "ctrl-shift-z": "editor::Redo"
    }
  }
]
```

### Key Syntax

| Modifier | Syntax |
|----------|--------|
| Control | `ctrl-` |
| Shift | `shift-` |
| Alt/Option | `alt-` |
| Command (macOS) | `cmd-` |

Common keys: `a-z`, `0-9`, `up`, `down`, `left`, `right`, `enter`, `escape`, `tab`, `space`, `backspace`, `delete`

### Complex Actions in Keymap

```json
{
  "context": "editor",
  "bindings": {
    "up": "editor::Move",
    "shift-up": ["editor::Move", {"direction": "up", "select": true}],
    "ctrl-g": ["editor::GoToLine", {"line": 1}]
  }
}
```

## Complete Example: Keyboard-Navigable List

```rust
use gpui::prelude::*;
use gpui::{actions, div, Application, Context, Window};

actions!(list, [MoveUp, MoveDown, Select]);

struct SelectableList {
    items: Vec<String>,
    selected: usize,
}

impl SelectableList {
    fn move_up(&mut self) {
        if self.selected > 0 { self.selected -= 1; }
    }
    
    fn move_down(&mut self) {
        if self.selected < self.items.len().saturating_sub(1) {
            self.selected += 1;
        }
    }
}

impl Render for SelectableList {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .key_context("list")
            .on_action(|this: &mut Self, _: &MoveUp, window, cx| {
                this.move_up();
                cx.notify();
            })
            .on_action(|this: &mut Self, _: &MoveDown, window, cx| {
                this.move_down();
                cx.notify();
            })
            .on_action(|this: &mut Self, _: &Select, window, cx| {
                println!("Selected: {}", this.items[this.selected]);
            })
            .children(
                self.items.iter().enumerate().map(|(i, item)| {
                    div()
                        .child(item.clone())
                        .when(i == self.selected, |el| el.bg(gpui::rgb(0x3b82f6)))
                })
            )
    }
}
```

Keymap:
```json
[{"context": "list", "bindings": {"up": "list::MoveUp", "k": "list::MoveUp", "down": "list::MoveDown", "j": "list::MoveDown", "enter": "list::Select"}}]
```
