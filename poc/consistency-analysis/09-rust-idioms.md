# Rust Idioms and Conventions Analysis
## Script Kit GPUI Codebase

**Analysis Date**: January 30, 2026
**Codebase**: Script Kit GPUI (304 Rust source files)
**Focus**: Idiomatic Rust patterns, conventions, and code quality

---

## Executive Summary

The Script Kit GPUI codebase demonstrates **strong Rust idiom compliance** with excellent error handling, builder patterns, and modern Rust practices. The code follows Zed editor patterns extensively and maintains high consistency across modules. This analysis identifies best practices currently in use and opportunities for enhanced idiomaticity.

**Overall Idiom Score**: 8.5/10 - Mature, well-structured Rust

---

## 1. Error Handling & Result Types

### Current Practices (Excellent)

#### 1.1 Custom Error Types with `thiserror`
**Location**: `/Users/johnlindquist/dev/script-kit-gpui/src/error.rs`

```rust
#[derive(Error, Debug)]
pub enum ScriptKitError {
    #[error("Script execution failed: {message}")]
    ScriptExecution {
        message: String,
        script_path: Option<String>,
    },

    #[error("Failed to parse protocol message: {0}")]
    ProtocolParse(#[from] serde_json::Error),

    #[error("Theme loading failed for '{path}': {source}")]
    ThemeLoad {
        path: String,
        #[source]
        source: std::io::Error,
    },
}
```

**Idiom Quality**: ✅ Excellent
- Uses `thiserror` for ergonomic error definitions
- Proper `#[from]` attribute for automatic conversions
- `#[source]` attribute for error chaining
- Domain-specific error variants with contextual information

#### 1.2 Extension Traits for Error Handling
**Pattern**: `ResultExt` and `NotifyResultExt` traits

```rust
pub trait ResultExt<T> {
    fn log_err(self) -> Option<T>;
    fn warn_on_err(self) -> Option<T>;
}

impl<T, E: std::fmt::Debug> ResultExt<T> for std::result::Result<T, E> {
    #[track_caller]
    fn log_err(self) -> Option<T> {
        match self {
            Ok(value) => Some(value),
            Err(error) => {
                let caller = std::panic::Location::caller();
                error!(
                    error = ?error,
                    file = caller.file(),
                    line = caller.line(),
                    "Operation failed"
                );
                None
            }
        }
    }
}
```

**Idiom Quality**: ✅ Excellent
- Uses `#[track_caller]` for precise error location tracking (Zed pattern)
- Ergonomic `log_err()` and `warn_on_err()` methods
- Respects Rust's error handling philosophy
- Eliminates boilerplate `.map_err()` chains

**Recommendation**: This pattern is excellent. Consider documenting it as a team convention.

#### 1.3 Error Severity Classification
**Practice**: Error severity enum for UI display

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    Info,     // Blue - informational
    Warning,  // Yellow - recoverable
    Error,    // Red - operation failed
    Critical, // Red + modal - requires user action
}

impl ScriptKitError {
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            Self::ScriptExecution { .. } => ErrorSeverity::Error,
            Self::ProtocolParse(_) => ErrorSeverity::Warning,
            // ...
        }
    }
}
```

**Idiom Quality**: ✅ Good
- Provides domain-specific context for error display
- Separates error categorization from error definition
- Could benefit from being a trait (see Recommendations)

---

## 2. Builder Pattern Usage

### Current Implementations

#### 2.1 Builder Chain Pattern
**Location**: `/Users/johnlindquist/dev/script-kit-gpui/src/notification/types.rs`

```rust
pub struct NotificationAction {
    pub label: String,
    pub id: String,
    pub style: ActionStyle,
}

impl NotificationAction {
    pub fn new(label: impl Into<String>, id: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            id: id.into(),
            style: ActionStyle::Default,
        }
    }

    pub fn primary(label: impl Into<String>, id: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            id: id.into(),
            style: ActionStyle::Primary,
        }
    }

    pub fn destructive(label: impl Into<String>, id: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            id: id.into(),
            style: ActionStyle::Destructive,
        }
    }
}
```

**Idiom Quality**: ✅ Good
- Factory methods for common configurations
- Uses `impl Into<String>` for flexibility
- Clear semantic intent (`.primary()`, `.destructive()`)

**Enhancement Opportunity**: These could chain with `with_` methods:

```rust
impl NotificationAction {
    pub fn with_style(mut self, style: ActionStyle) -> Self {
        self.style = style;
        self
    }
}

// Usage
NotificationAction::new("Delete", "delete").with_style(ActionStyle::Destructive)
```

#### 2.2 Fluent Builder Pattern
**Location**: Multiple (tray, logging, etc.)

```rust
builder = builder
    .with_icon(icon)
    .with_tooltip("Script Kit")
    .with_menu(menu);

if is_template {
    builder = builder.with_icon_as_template(true);
}
```

**Idiom Quality**: ✅ Excellent
- Clean fluent API
- Conditional chaining supported
- Matches GPUI and Zed patterns

#### 2.3 Config Builder Pattern
**Location**: `/Users/johnlindquist/dev/script-kit-gpui/src/config/editor.rs`

```rust
#[derive(Debug, Clone)]
pub struct ConfigProperty {
    pub name: String,
    pub value: String,
}

impl ConfigProperty {
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }
}
```

**Idiom Quality**: ✅ Good
- Minimal but functional builder
- Uses `impl Into<T>` for ergonomics

---

## 3. Match Expressions & Pattern Matching

### 3.1 Exhaustive Matching
**Location**: `/Users/johnlindquist/dev/script-kit-gpui/src/action_helpers.rs`

```rust
pub fn extract_path_for_reveal(
    result: Option<&SearchResult>,
) -> Result<PathBuf, PathExtractionError> {
    match result {
        None => Err(PathExtractionError::NoSelection),
        Some(SearchResult::Script(m)) => Ok(m.script.path.clone()),
        Some(SearchResult::App(m)) => Ok(m.app.path.clone()),
        Some(SearchResult::Agent(m)) => Ok(m.agent.path.clone()),
        Some(SearchResult::Scriptlet(_)) => Err(PathExtractionError::UnsupportedType(
            SharedString::from("Cannot reveal scriptlets in Finder"),
        )),
        Some(SearchResult::BuiltIn(_)) => Err(PathExtractionError::UnsupportedType(
            SharedString::from("Cannot reveal built-in features"),
        )),
        Some(SearchResult::Window(_)) => Err(PathExtractionError::UnsupportedType(
            SharedString::from("Cannot reveal windows in Finder"),
        )),
        Some(SearchResult::Fallback(_)) => Err(PathExtractionError::UnsupportedType(
            SharedString::from("Cannot reveal fallback commands in Finder"),
        )),
    }
}
```

**Idiom Quality**: ✅ Excellent
- Exhaustive matching ensures compile-time correctness
- Compiler enforces handling all variants
- Clear error messages for each unsupported type
- Proper use of wildcard patterns (`_`) where value isn't needed

**Minor Issue**: Repetition in error handling

**Better Version**:
```rust
// Extract common error logic
fn unsupported_error(msg: &str) -> PathExtractionError {
    PathExtractionError::UnsupportedType(SharedString::from(msg))
}

match result {
    None => Err(PathExtractionError::NoSelection),
    Some(SearchResult::Script(m)) => Ok(m.script.path.clone()),
    Some(SearchResult::App(m)) => Ok(m.app.path.clone()),
    Some(SearchResult::Agent(m)) => Ok(m.agent.path.clone()),
    Some(SearchResult::Scriptlet(_)) => Err(unsupported_error("Cannot reveal scriptlets in Finder")),
    Some(SearchResult::BuiltIn(_)) => Err(unsupported_error("Cannot reveal built-in features")),
    Some(SearchResult::Window(_)) => Err(unsupported_error("Cannot reveal windows in Finder")),
    Some(SearchResult::Fallback(_)) => Err(unsupported_error("Cannot reveal fallback commands in Finder")),
}
```

### 3.2 Enum Dispatch Pattern
**Location**: `/Users/johnlindquist/dev/script-kit-gpui/src/hotkeys.rs`

```rust
#[derive(Clone, Debug, PartialEq)]
pub enum HotkeyAction {
    Main,
    Notes,
    Ai,
    ToggleLogs,
    Script(String),
}

// Dispatch in main hotkey loop
match action {
    Some(HotkeyAction::Main) => {
        let count = HOTKEY_TRIGGER_COUNT.fetch_add(1, Ordering::Relaxed);
        if hotkey_channel().0.try_send(()).is_err() {
            logging::log("HOTKEY", "Main hotkey channel full/closed");
        }
    }
    Some(HotkeyAction::Notes) => {
        dispatch_notes_hotkey();
    }
    Some(HotkeyAction::Ai) => {
        dispatch_ai_hotkey();
    }
    Some(HotkeyAction::Script(path)) => {
        logging::bench_start(&format!("hotkey:{}", path));
        if script_hotkey_channel().0.try_send(path.clone()).is_err() {
            logging::log("HOTKEY", &format!("Script channel full/closed for {}", path));
        }
    }
    None => {
        logging::log("HOTKEY", &format!("Unknown hotkey event id={}", event.id));
    }
}
```

**Idiom Quality**: ✅ Excellent
- Enum dispatch clearly separates concerns
- Exhaustive matching for correctness
- Associated data in enum variants (`Script(String)`)

### 3.3 Guard Clauses with `match`
**Location**: `/Users/johnlindquist/dev/script-kit-gpui/src/action_helpers.rs`

```rust
pub fn trigger_sdk_action(
    action_name: &str,
    action: &ProtocolAction,
    current_input: &str,
    sender: Option<&SyncSender<protocol::Message>>,
) -> bool {
    let Some(sender) = sender else {
        logging::log(
            "WARN",
            &format!("No response sender for SDK action '{}'", action_name),
        );
        return false;
    };

    let send_result = if action.has_action {
        // ...
    } else if let Some(ref value) = action.value {
        // ...
    } else {
        return false;
    };

    match send_result {
        Ok(()) => true,
        Err(std::sync::mpsc::TrySendError::Full(_)) => {
            logging::log("WARN", &format!("Response channel full - SDK action '{}' dropped", action_name));
            false
        }
        Err(std::sync::mpsc::TrySendError::Disconnected(_)) => {
            logging::log("UI", &format!("Response channel disconnected - script exited (action '{}')", action_name));
            false
        }
    }
}
```

**Idiom Quality**: ✅ Excellent
- Uses `let Some(x) = y else { return; }` (Rust 1.65+ pattern)
- Early exits with clear error messages
- Nested pattern matching for error variants
- `_` patterns for ignored values

---

## 4. Lifetime and Borrowing Patterns

### 4.1 Explicit Lifetime Parameters
**Location**: `/Users/johnlindquist/dev/script-kit-gpui/src/action_helpers.rs`

```rust
pub fn find_sdk_action<'a>(
    actions: Option<&'a [ProtocolAction]>,
    action_name: &str,
    warn_on_shadow: bool,
) -> Option<&'a ProtocolAction> {
    let actions = actions?;

    if warn_on_shadow && is_reserved_action_id(action_name) {
        logging::log("WARN", &format!("SDK action '{}' shadows a built-in action", action_name));
    }

    actions.iter().find(|a| a.name == action_name)
}
```

**Idiom Quality**: ✅ Excellent
- Explicit lifetime `'a` clearly communicates borrowing intent
- Return value lifetime tied to input lifetime
- Uses `Option::?` operator for ergonomic error handling

### 4.2 Static Lifetimes for Constants
**Location**: `/Users/johnlindquist/dev/script-kit-gpui/src/storybook/registry.rs`

```rust
pub fn all_stories() -> impl Iterator<Item = &'static StoryEntry> {
    // Returns references with 'static lifetime
}

pub fn all_categories() -> Vec<&'static str> {
    // Returns string slice references with 'static lifetime
}

pub trait Story: Send + Sync {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn category(&self) -> &'static str;
}
```

**Idiom Quality**: ✅ Excellent
- `&'static` for invariant data
- Compiler ensures type safety
- No runtime lifetime tracking needed

### 4.3 Borrowing in Global Statics
**Location**: `/Users/johnlindquist/dev/script-kit-gpui/src/hotkeys.rs`

```rust
static HOTKEY_ROUTES: OnceLock<RwLock<HotkeyRoutes>> = OnceLock::new();

fn routes() -> &'static RwLock<HotkeyRoutes> {
    HOTKEY_ROUTES.get_or_init(|| RwLock::new(HotkeyRoutes::new()))
}
```

**Idiom Quality**: ✅ Excellent
- `OnceLock<T>` for lazy initialization (modern Rust pattern)
- Returns `&'static` reference to initialized data
- Accessor function prevents direct access to static

**Alternative Consideration**: Could use `std::sync::Arc<RwLock<T>>` for more flexibility with thread spawning.

---

## 5. Type-State Pattern Opportunities

### 5.1 Potential Type-State Implementation
**Current Pattern**: Runtime state validation

```rust
pub struct HotkeyRoutes {
    routes: HashMap<u32, RegisteredHotkey>,
    script_paths: HashMap<String, u32>,
    main_id: Option<u32>,
    notes_id: Option<u32>,
    ai_id: Option<u32>,
    logs_id: Option<u32>,
}

impl HotkeyRoutes {
    fn new() -> Self {
        Self {
            routes: HashMap::new(),
            script_paths: HashMap::new(),
            main_id: None,
            notes_id: None,
            ai_id: None,
            logs_id: None,
        }
    }

    fn get_action(&self, id: u32) -> Option<HotkeyAction> {
        self.routes.get(&id).map(|r| r.action.clone())
    }
}
```

**Issue**: Optional fields that should often be populated. State validity is runtime-checked.

**Enhanced Type-State Approach**:
```rust
// Uninitialized state
pub struct HotkeyRoutes {
    routes: HashMap<u32, RegisteredHotkey>,
}

impl HotkeyRoutes {
    fn new() -> Self {
        Self { routes: HashMap::new() }
    }
}

// Initialized state marker
pub struct InitializedRoutes;

pub struct RegisteredHotkeys {
    routes: HashMap<u32, RegisteredHotkey>,
    main_id: u32,
    _state: InitializedRoutes,
}

impl RegisteredHotkeys {
    // Only available after initialization
    fn get_action(&self, id: u32) -> Option<HotkeyAction> {
        self.routes.get(&id).map(|r| r.action.clone())
    }
}
```

**Recommendation**: For hotkey routing, the current approach is practical. Type-state would add complexity without significant safety gain since the initialization is well-controlled.

---

## 6. Trait Implementations & Bounds

### 6.1 Trait Derivation
**Pattern**: Extensive use of derived traits

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum HotkeyAction {
    Main,
    Notes,
    Ai,
    ToggleLogs,
    Script(String),
}

#[derive(Clone, Debug)]
pub enum NotificationContent {
    Text(String),
    TitleMessage { title: String, message: String },
    Rich { icon: Option<IconRef>, title: String, message: Option<String> },
    Progress { title: String, progress: f32, message: Option<String> },
    Html(String),
}
```

**Idiom Quality**: ✅ Excellent
- Uses appropriate derived traits (Debug, Clone, PartialEq)
- Follows standard Rust conventions
- Compiler ensures trait implementations are consistent

### 6.2 Custom Trait Implementations
**Location**: `/Users/johnlindquist/dev/script-kit-gpui/src/error.rs`

```rust
pub trait ResultExt<T> {
    fn log_err(self) -> Option<T>;
    fn warn_on_err(self) -> Option<T>;
}

impl<T, E: std::fmt::Debug> ResultExt<T> for std::result::Result<T, E> {
    #[track_caller]
    fn log_err(self) -> Option<T> {
        match self {
            Ok(value) => Some(value),
            Err(error) => {
                let caller = std::panic::Location::caller();
                error!(
                    error = ?error,
                    file = caller.file(),
                    line = caller.line(),
                    "Operation failed"
                );
                None
            }
        }
    }
}
```

**Idiom Quality**: ✅ Excellent
- Blanket implementation for all `Result<T, E>` where `E: Debug`
- Uses trait bounds effectively
- `#[track_caller]` provides debugging context

### 6.3 Trait Bounds Usage
**Example**: Display implementation with bounds

```rust
impl std::fmt::Display for CommandValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}
```

**Idiom Quality**: ✅ Good
- Implements standard library traits
- Uses `&mut Formatter<'_>` idiomatically

---

## 7. Clippy Compliance

### 7.1 Lint Attributes
**Location**: Throughout codebase

```rust
#[allow(dead_code)]
pub fn unused_utility() { }

#[allow(clippy::while_let_on_iterator)]
while let Some(c) = chars.next() {
    // Implementation
}
```

**Idiom Quality**: ✅ Good
- Explicit `#[allow]` attributes document intentional suppression
- Specific lint names prevent over-suppression

**Observation**: `#[allow(dead_code)]` frequently used for:
- Extension traits that may not all be used immediately
- Public API functions that are part of the contract
- Test helper functions

### 7.2 Compiler Warnings Compliance
**Project Goal** (from CLAUDE.md):
```bash
cargo check && cargo clippy --all-targets -- -D warnings && cargo test
```

**Idiom Quality**: ✅ Excellent
- Project enforces `-D warnings` in CI
- All warnings treated as errors
- Comprehensive checking including test code

---

## 8. Specific Code Patterns

### 8.1 Snippet Parser (Advanced Pattern Matching)
**Location**: `/Users/johnlindquist/dev/script-kit-gpui/src/snippet.rs`

```rust
pub fn parse(template: &str) -> Self {
    let mut parts = Vec::new();
    let mut text = String::new();
    let mut char_count: usize = 0;
    let mut chars = template.chars().peekable();
    let mut current_text = String::new();

    while let Some(c) = chars.next() {
        if c == '$' {
            match chars.peek() {
                Some('$') => {
                    chars.next();
                    current_text.push('$');
                }
                Some('{') => {
                    // Flush current text
                    if !current_text.is_empty() {
                        text.push_str(&current_text);
                        char_count += current_text.chars().count();
                        parts.push(SnippetPart::Text(current_text.clone()));
                        current_text.clear();
                    }
                    chars.next(); // consume '{'
                    let tabstop = Self::parse_braced_tabstop(&mut chars, char_count);
                    // ...
                }
                Some(d) if d.is_ascii_digit() => {
                    // Parse simple tabstop
                }
                _ => {
                    current_text.push('$');
                }
            }
        } else {
            current_text.push(c);
        }
    }

    // Implementation continues...
}
```

**Idiom Quality**: ✅ Excellent
- Complex pattern matching handled clearly
- Guards (`if d.is_ascii_digit()`) for conditional matching
- State management with accumulator variables
- Proper resource cleanup (flushing buffers)

**Best Practices Demonstrated**:
1. Peekable iterator for lookahead
2. Guard clauses for validation
3. Accumulator pattern for building results
4. Exhaustive matching on enum variants

### 8.2 State Machine Pattern
**Location**: `/Users/johnlindquist/dev/script-kit-gpui/src/notification/types.rs`

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum NotificationPriority {
    Low = 0,
    #[default]
    Normal = 1,
    High = 2,
    Urgent = 3,
}

impl NotificationBehavior {
    pub fn default() -> Self {
        Self {
            duration: Some(Duration::from_secs(3)),
            dismissable: true,
            replace_key: None,
            sound: NotificationSound::None,
            priority: NotificationPriority::Normal,
        }
    }
}
```

**Idiom Quality**: ✅ Excellent
- Enum with discriminant values for ordering
- Default trait implementation with sensible defaults
- `#[default]` attribute (Rust 1.62+) for Default derive

---

## 9. Global Singletons & Thread-Safety

### 9.1 OnceLock Pattern
**Location**: `/Users/johnlindquist/dev/script-kit-gpui/src/hotkeys.rs`

```rust
static HOTKEY_ROUTES: OnceLock<RwLock<HotkeyRoutes>> = OnceLock::new();
static MAIN_MANAGER: OnceLock<Mutex<GlobalHotKeyManager>> = OnceLock::new();
static SCRIPT_HOTKEY_MANAGER: OnceLock<Mutex<ScriptHotkeyManager>> = OnceLock::new();

fn routes() -> &'static RwLock<HotkeyRoutes> {
    HOTKEY_ROUTES.get_or_init(|| RwLock::new(HotkeyRoutes::new()))
}
```

**Idiom Quality**: ✅ Excellent (Modern Rust)
- `OnceLock` (std::sync::OnceLock) replaces `lazy_static!` macro
- Zero runtime overhead after initialization
- Type-safe initialization
- Panic-free design (returns `None` on repeated initialization attempts)

**Thread-Safety Guarantees**:
- `RwLock<T>` for read-heavy workloads (hotkey dispatch)
- `Mutex<T>` for exclusive access (manager)

### 9.2 Atomic Operations
**Pattern**: Atomic types for lock-free operations

```rust
static HOTKEY_TRIGGER_COUNT: AtomicU64 = AtomicU64::new(0);

pub fn is_main_hotkey_registered() -> bool {
    MAIN_HOTKEY_REGISTERED.load(Ordering::Relaxed)
}

// In event loop
let count = HOTKEY_TRIGGER_COUNT.fetch_add(1, Ordering::Relaxed);
```

**Idiom Quality**: ✅ Excellent
- Uses `Ordering::Relaxed` for non-synchronizing operations
- Proper memory ordering semantics
- Minimal overhead for simple counters

---

## 10. Recommendations for Enhanced Idiomaticity

### 10.1 High Priority

#### 1. Reduce Match Arm Duplication
**Current Code** (action_helpers.rs):
```rust
Some(SearchResult::Scriptlet(_)) => Err(PathExtractionError::UnsupportedType(
    SharedString::from("Cannot reveal scriptlets in Finder"),
)),
Some(SearchResult::BuiltIn(_)) => Err(PathExtractionError::UnsupportedType(
    SharedString::from("Cannot reveal built-in features"),
)),
```

**Recommendation**:
```rust
// Define common errors as constants
const ERR_REVEAL_SCRIPTLET: &str = "Cannot reveal scriptlets in Finder";
const ERR_REVEAL_BUILTIN: &str = "Cannot reveal built-in features";

// Or use a helper function
fn unsupported_type(msg: &str) -> PathExtractionError {
    PathExtractionError::UnsupportedType(SharedString::from(msg))
}

match result {
    Some(SearchResult::Scriptlet(_)) => Err(unsupported_type(ERR_REVEAL_SCRIPTLET)),
    Some(SearchResult::BuiltIn(_)) => Err(unsupported_type(ERR_REVEAL_BUILTIN)),
}
```

#### 2. Implement Builder Methods with `with_` Prefix
**Current Code** (notification/types.rs):
```rust
pub fn primary(label: impl Into<String>, id: impl Into<String>) -> Self {
    Self { label: label.into(), id: id.into(), style: ActionStyle::Primary }
}
```

**Enhanced**:
```rust
impl NotificationAction {
    pub fn new(label: impl Into<String>, id: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            id: id.into(),
            style: ActionStyle::Default,
        }
    }

    pub fn with_style(mut self, style: ActionStyle) -> Self {
        self.style = style;
        self
    }

    pub fn primary(label: impl Into<String>, id: impl Into<String>) -> Self {
        Self::new(label, id).with_style(ActionStyle::Primary)
    }
}

// Usage
NotificationAction::new("Delete", "delete").with_style(ActionStyle::Destructive)
```

#### 3. Use Type Aliases for Clarity
**Opportunity** (hotkeys.rs):
```rust
pub type HotkeyId = u32;
pub type HotkeyHandler = Arc<dyn Fn() + Send + Sync>;

// Current (verbose)
let hotkey_id: u32 = new_hotkey.id();

// Better (clear intent)
let hotkey_id: HotkeyId = new_hotkey.id();
```

### 10.2 Medium Priority

#### 1. Leverage Iterator Combinators
**Current Pattern** (snippet.rs):
```rust
let mut tabstop_map: BTreeMap<usize, TabstopInfo> = BTreeMap::new();

for part in parts {
    if let SnippetPart::Tabstop { index, placeholder, choices, range } = part {
        tabstop_map.entry(*index)
            .and_modify(|info| { /* ... */ })
            .or_insert_with(|| { /* ... */ });
    }
}
```

**More Idiomatic Option**:
```rust
let tabstop_map: BTreeMap<usize, TabstopInfo> = parts
    .iter()
    .filter_map(|part| {
        if let SnippetPart::Tabstop { index, placeholder, choices, range } = part {
            Some((*index, TabstopInfo { /* ... */ }))
        } else {
            None
        }
    })
    .fold(BTreeMap::new(), |mut map, (idx, info)| {
        map.entry(idx)
            .and_modify(|existing| existing.ranges.push(*range))
            .or_insert(info);
        map
    });
```

*Note*: Current approach is acceptable; functional style is not always clearer.

#### 2. Use `if let` for Single Pattern Matches
**Opportunity**: When matching a single pattern

```rust
// Current
match self.path_to_id.remove(path) {
    Some(hotkey_id) => { /* ... */ }
    None => {}
}

// More idiomatic
if let Some(hotkey_id) = self.path_to_id.remove(path) {
    // ...
}
```

#### 3. Document Type-State Intentions
**Opportunity** (hotkeys.rs):
```rust
/// Global routing table - protected by RwLock for fast reads
/// Read operations (event dispatch) use `.read()`
/// Write operations (registration) use `.write()`
static HOTKEY_ROUTES: OnceLock<RwLock<HotkeyRoutes>> = OnceLock::new();
```

### 10.3 Low Priority (Code Style)

#### 1. Consistent Destructuring
**Pattern**: Mix of different destructuring styles

```rust
// Sometimes explicit
if let Some(handler) = handler {
    // ...
}

// Sometimes using ?
let Some(sender) = sender else { return false; };
```

**Recommendation**: Document preference in style guide (current approach is acceptable).

#### 2. Error Context Propagation
**Opportunity**: Use `context()` from `anyhow` for richer errors

```rust
// Current
Err(anyhow::anyhow!("Failed to parse shortcut: {}", shortcut))

// With context
shortcut_str.parse().context("Failed to parse shortcut")
```

---

## 11. Idiom Compliance Summary

| Category | Score | Status |
|----------|-------|--------|
| Error Handling | 9/10 | Excellent - Uses thiserror, extension traits, proper context |
| Builder Patterns | 8/10 | Good - Fluent API used well; some duplication in factory methods |
| Match Expressions | 9/10 | Excellent - Exhaustive matching, guard clauses, enum dispatch |
| Lifetimes & Borrowing | 9/10 | Excellent - Explicit lifetimes, proper 'static usage |
| Type-State Pattern | 6/10 | N/A - Not utilized, but not necessary for current use cases |
| Trait Implementations | 9/10 | Excellent - Proper derives, blanket impls, bounds usage |
| Clippy Compliance | 9/10 | Excellent - Enforced via CI, targeted allow attributes |
| Global Singletons | 9/10 | Excellent - Modern OnceLock usage, proper thread-safety |
| Code Organization | 8/10 | Good - Clear module structure, some repetition opportunities |
| Documentation | 8/10 | Good - Doc comments on public API, rationale in code |

**Overall Rust Idiom Score: 8.5/10**

---

## 12. Best Practices to Standardize

### 12.1 Established Patterns (Enforce)

1. **Error Handling**
   - Use `thiserror::Error` for domain errors
   - Implement `ResultExt` trait for automatic logging
   - Provide `#[source]` for error chains

2. **Builder Pattern**
   - Use `impl Into<T>` for string parameters
   - Implement fluent `.with_*()` methods
   - Provide semantic factory methods (`.primary()`, `.destructive()`)

3. **Global State**
   - Use `OnceLock` for lazy initialization
   - Combine with `RwLock` for read-heavy patterns
   - Use `Mutex` for exclusive access
   - Provide accessor functions instead of direct static access

4. **Match Expressions**
   - Leverage exhaustive matching for safety
   - Use guard clauses (`if <condition>`)
   - Document wildcard patterns that intentionally ignore values

### 12.2 Recommended Conventions

1. **Naming**
   - Trait methods: `log_err()`, `warn_on_err()` for error handling
   - Factory methods: `new()`, `with_*()`, `primary()`, `destructive()`
   - Module functions: `get_*()`, `set_*()`, `is_*()`

2. **Error Messages**
   - Capitalized descriptions: "Cannot reveal scriptlets in Finder"
   - Include context: "Failed to register hotkey 'cmd+k' (already in use)"
   - Actionable when possible: "Try a different shortcut or close the conflicting app"

3. **Documentation**
   - Document lifetime intentions explicitly
   - Explain `#[allow]` attributes when suppressing lints
   - Document safety invariants in unsafe blocks

---

## 13. Conclusion

The Script Kit GPUI codebase demonstrates **mature Rust practices** with strong adherence to idiomatic patterns. The team has successfully:

- ✅ Implemented comprehensive error handling with domain-specific types
- ✅ Used builder patterns for ergonomic API design
- ✅ Leveraged Rust's type system for compile-time safety
- ✅ Applied modern std library patterns (OnceLock, etc.)
- ✅ Maintained high code quality with Clippy enforcement

The primary areas for enhancement are:
- Reducing match arm duplication
- Standardizing builder method chaining
- Documenting established patterns as team conventions

The codebase is well-positioned for maintenance and evolution, with clear examples of idiomatic Rust throughout.

---

## References

- **Rust Book**: https://doc.rust-lang.org/book/
- **Zed Architecture**: https://github.com/zed-industries/zed
- **GPUI Docs**: https://docs.rs/gpui/latest/gpui/
- **thiserror Crate**: https://docs.rs/thiserror/latest/thiserror/
- **Clippy Lints**: https://rust-lang.github.io/rust-clippy/

---

**Document Generated**: 2026-01-30
**Analysis Tool**: Claude Code Agent
**Analysis Scope**: 304 Rust source files across Script Kit GPUI codebase
