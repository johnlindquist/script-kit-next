# Form & Input Handling - Expert Bundle

## Overview

Script Kit provides multiple input prompts: arg (selection), editor (text), form (multi-field), and div (custom HTML). Each follows consistent patterns for keyboard handling and value submission.

## Input Types

### Arg Prompt (src/prompts.rs)

```rust
pub struct ArgPrompt {
    placeholder: String,
    input_value: String,
    choices: Vec<Choice>,
    filtered_choices: Vec<Arc<Choice>>,
    selected_index: usize,
    focus_handle: FocusHandle,
    list_scroll_handle: UniformListScrollHandle,
}

impl ArgPrompt {
    pub fn new(placeholder: &str, choices: Vec<Choice>, cx: &mut Context<Self>) -> Self {
        let filtered = choices.iter().cloned().map(Arc::new).collect();
        Self {
            placeholder: placeholder.to_string(),
            input_value: String::new(),
            choices,
            filtered_choices: filtered,
            selected_index: 0,
            focus_handle: cx.focus_handle(),
            list_scroll_handle: UniformListScrollHandle::new(),
        }
    }

    fn filter_choices(&mut self) {
        let query = self.input_value.to_lowercase();
        self.filtered_choices = self.choices
            .iter()
            .filter(|c| {
                c.name.to_lowercase().contains(&query) ||
                c.description.as_ref()
                    .map(|d| d.to_lowercase().contains(&query))
                    .unwrap_or(false)
            })
            .cloned()
            .map(Arc::new)
            .collect();
        self.selected_index = 0;
    }
}
```

### Editor Prompt

```rust
pub struct EditorPrompt {
    content: String,
    language: Option<String>,
    line_numbers: bool,
    focus_handle: FocusHandle,
    cursor_position: (usize, usize), // (line, column)
}

impl EditorPrompt {
    pub fn new(
        initial_content: &str,
        language: Option<&str>,
        cx: &mut Context<Self>,
    ) -> Self {
        Self {
            content: initial_content.to_string(),
            language: language.map(String::from),
            line_numbers: true,
            focus_handle: cx.focus_handle(),
            cursor_position: (0, 0),
        }
    }

    pub fn get_value(&self) -> &str {
        &self.content
    }

    fn insert_text(&mut self, text: &str) {
        // Insert at cursor position
        let (line, col) = self.cursor_position;
        let lines: Vec<&str> = self.content.lines().collect();
        
        if line < lines.len() {
            // ... insertion logic
        }
    }
}
```

### Form Prompt (src/form_prompt.rs)

```rust
pub struct FormPrompt {
    fields: Vec<FormField>,
    focused_field_index: usize,
    focus_handle: FocusHandle,
}

pub struct FormField {
    pub name: String,
    pub label: String,
    pub field_type: FieldType,
    pub value: String,
    pub placeholder: Option<String>,
    pub required: bool,
}

pub enum FieldType {
    Text,
    Password,
    Number,
    Email,
    TextArea,
    Select(Vec<String>),
    Checkbox,
}

impl FormPrompt {
    fn focus_next_field(&mut self, cx: &mut Context<Self>) {
        if self.focused_field_index < self.fields.len() - 1 {
            self.focused_field_index += 1;
            cx.notify();
        }
    }

    fn focus_prev_field(&mut self, cx: &mut Context<Self>) {
        if self.focused_field_index > 0 {
            self.focused_field_index -= 1;
            cx.notify();
        }
    }

    fn get_values(&self) -> HashMap<String, String> {
        self.fields
            .iter()
            .map(|f| (f.name.clone(), f.value.clone()))
            .collect()
    }

    fn validate(&self) -> Vec<ValidationError> {
        self.fields
            .iter()
            .filter_map(|f| {
                if f.required && f.value.is_empty() {
                    Some(ValidationError {
                        field: f.name.clone(),
                        message: format!("{} is required", f.label),
                    })
                } else {
                    None
                }
            })
            .collect()
    }
}
```

## Keyboard Handling

### Key Event Pattern

```rust
impl ArgPrompt {
    fn handle_key_down(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let key = event.key.as_ref().map(|k| k.as_str()).unwrap_or("");
        
        match key {
            // Arrow keys (handle both variants!)
            "up" | "arrowup" => {
                self.move_selection_up(cx);
            }
            "down" | "arrowdown" => {
                self.move_selection_down(cx);
            }
            
            // Submit
            "enter" | "Enter" => {
                self.submit(cx);
            }
            
            // Cancel
            "escape" | "Escape" => {
                self.cancel(cx);
            }
            
            // Tab navigation
            "tab" | "Tab" => {
                if event.modifiers.shift {
                    self.focus_prev(cx);
                } else {
                    self.focus_next(cx);
                }
            }
            
            // Page navigation
            "pageup" | "PageUp" => {
                self.page_up(cx);
            }
            "pagedown" | "PageDown" => {
                self.page_down(cx);
            }
            
            // Home/End
            "home" | "Home" => {
                self.select_first(cx);
            }
            "end" | "End" => {
                self.select_last(cx);
            }
            
            _ => {}
        }
    }
}
```

### Input Coalescing

```rust
pub struct InputCoalescer {
    pending: bool,
    latest: Option<String>,
}

impl InputCoalescer {
    pub fn queue(&mut self, value: String) -> bool {
        self.latest = Some(value);
        if self.pending {
            false // Already processing
        } else {
            self.pending = true;
            true // Start new batch
        }
    }

    pub fn take(&mut self) -> Option<String> {
        if self.pending {
            self.pending = false;
            self.latest.take()
        } else {
            None
        }
    }
}

impl ArgPrompt {
    fn handle_input_change(&mut self, text: &str, cx: &mut Context<Self>) {
        if self.input_coalescer.queue(text.to_string()) {
            // Schedule filter update
            cx.spawn(|this, mut cx| async move {
                Timer::after(Duration::from_millis(16)).await;
                let _ = this.update(&mut cx, |prompt, cx| {
                    if let Some(latest) = prompt.input_coalescer.take() {
                        prompt.input_value = latest;
                        prompt.filter_choices();
                        cx.notify();
                    }
                });
            }).detach();
        }
    }
}
```

## Selection Management

### List Selection

```rust
impl ArgPrompt {
    fn move_selection_up(&mut self, cx: &mut Context<Self>) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            self.scroll_to_selected();
            cx.notify();
        }
    }

    fn move_selection_down(&mut self, cx: &mut Context<Self>) {
        if self.selected_index < self.filtered_choices.len().saturating_sub(1) {
            self.selected_index += 1;
            self.scroll_to_selected();
            cx.notify();
        }
    }

    fn scroll_to_selected(&self) {
        self.list_scroll_handle.scroll_to_item(
            self.selected_index,
            ScrollStrategy::Nearest,
        );
    }

    fn select_first(&mut self, cx: &mut Context<Self>) {
        self.selected_index = 0;
        self.scroll_to_selected();
        cx.notify();
    }

    fn select_last(&mut self, cx: &mut Context<Self>) {
        self.selected_index = self.filtered_choices.len().saturating_sub(1);
        self.scroll_to_selected();
        cx.notify();
    }
}
```

### Multi-Select

```rust
pub struct MultiSelectPrompt {
    choices: Vec<Choice>,
    selected_indices: HashSet<usize>,
    focused_index: usize,
}

impl MultiSelectPrompt {
    fn toggle_selection(&mut self, cx: &mut Context<Self>) {
        if self.selected_indices.contains(&self.focused_index) {
            self.selected_indices.remove(&self.focused_index);
        } else {
            self.selected_indices.insert(self.focused_index);
        }
        cx.notify();
    }

    fn select_all(&mut self, cx: &mut Context<Self>) {
        self.selected_indices = (0..self.choices.len()).collect();
        cx.notify();
    }

    fn select_none(&mut self, cx: &mut Context<Self>) {
        self.selected_indices.clear();
        cx.notify();
    }

    fn get_selected_values(&self) -> Vec<&Choice> {
        self.selected_indices
            .iter()
            .filter_map(|&i| self.choices.get(i))
            .collect()
    }
}
```

## Validation

### Field Validation

```rust
pub enum ValidationRule {
    Required,
    MinLength(usize),
    MaxLength(usize),
    Pattern(Regex),
    Email,
    Number,
    Custom(Box<dyn Fn(&str) -> Option<String>>),
}

pub struct ValidationError {
    pub field: String,
    pub message: String,
}

impl FormField {
    fn validate(&self, rules: &[ValidationRule]) -> Option<ValidationError> {
        for rule in rules {
            let error = match rule {
                ValidationRule::Required => {
                    if self.value.is_empty() {
                        Some(format!("{} is required", self.label))
                    } else {
                        None
                    }
                }
                ValidationRule::MinLength(min) => {
                    if self.value.len() < *min {
                        Some(format!("{} must be at least {} characters", self.label, min))
                    } else {
                        None
                    }
                }
                ValidationRule::Email => {
                    let re = Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$").unwrap();
                    if !re.is_match(&self.value) {
                        Some(format!("{} must be a valid email", self.label))
                    } else {
                        None
                    }
                }
                _ => None,
            };
            
            if let Some(msg) = error {
                return Some(ValidationError {
                    field: self.name.clone(),
                    message: msg,
                });
            }
        }
        None
    }
}
```

## Submission

### Submit Pattern

```rust
impl ArgPrompt {
    fn submit(&mut self, cx: &mut Context<Self>) {
        if let Some(choice) = self.filtered_choices.get(self.selected_index) {
            let value = choice.value.clone().unwrap_or_else(|| choice.name.clone());
            
            // Send to script
            cx.emit(PromptEvent::Submit(serde_json::json!({
                "value": value,
                "name": choice.name,
                "index": self.selected_index,
            })));
        }
    }

    fn cancel(&mut self, cx: &mut Context<Self>) {
        cx.emit(PromptEvent::Cancel);
    }
}

pub enum PromptEvent {
    Submit(serde_json::Value),
    Cancel,
    InputChanged(String),
}
```

### Form Submission

```rust
impl FormPrompt {
    fn submit(&mut self, cx: &mut Context<Self>) {
        // Validate all fields
        let errors = self.validate();
        
        if errors.is_empty() {
            let values = self.get_values();
            cx.emit(PromptEvent::Submit(serde_json::to_value(&values).unwrap()));
        } else {
            // Show validation errors
            self.validation_errors = errors;
            cx.notify();
        }
    }
}
```

## Focus Management

### Focus Handle

```rust
impl Focusable for ArgPrompt {
    fn focus_handle(&self, _cx: &Context<Self>) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for ArgPrompt {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(Self::handle_key_down))
            // ...
    }
}
```

### Auto-Focus

```rust
impl ArgPrompt {
    fn focus_input(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.focus_handle.focus(window);
        cx.notify();
    }
}

// On mount
cx.spawn(|this, mut cx| async move {
    Timer::after(Duration::from_millis(50)).await;
    let _ = this.update(&mut cx, |prompt, cx| {
        prompt.focus_input(cx.window(), cx);
    });
}).detach();
```

## Rendering Input

### Text Input

```rust
fn render_input(&self, cx: &mut Context<Self>) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .h(px(44.0))
        .px_3()
        .border_b_1()
        .border_color(rgb(self.theme.colors.ui.border))
        .child(
            // Search icon
            Icon::new(IconName::Search)
                .size_4()
                .text_color(rgb(self.theme.colors.text.secondary))
        )
        .child(
            div()
                .flex_1()
                .pl_2()
                .child(
                    input()
                        .value(&self.input_value)
                        .placeholder(&self.placeholder)
                        .on_change(cx.listener(|this, text, _, cx| {
                            this.handle_input_change(&text, cx);
                        }))
                )
        )
}
```

## Best Practices

1. **Handle both key variants** - `"up"` and `"arrowup"`, etc.
2. **Coalesce rapid input** - prevent lag from fast typing
3. **Use scroll handles** - keep selection visible
4. **Validate before submit** - show errors inline
5. **Auto-focus on mount** - ready for input immediately
6. **Track focus handle** - enable keyboard navigation
7. **Emit events** - decouple submission from handling

## Summary

| Prompt Type | Primary Input | Submit Key | Cancel Key |
|-------------|---------------|------------|------------|
| arg | Selection from list | Enter | Escape |
| editor | Multi-line text | Cmd+Enter | Escape |
| form | Multiple fields | Enter (on last field) | Escape |
| div | Custom HTML | Varies | Escape |
