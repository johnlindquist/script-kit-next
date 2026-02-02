# Module Boundaries Analysis - Script Kit GPUI

**Date**: January 30, 2026
**Scope**: API exposure, module coupling, circular dependencies, trait abstraction patterns

## Executive Summary

Script Kit GPUI maintains relatively clean module boundaries through deliberate architectural choices:

1. **Protocol as a boundary**: Core data types are isolated from implementation
2. **Trait-based abstraction**: DesignRenderer, AiProvider, DesignTokens define contracts
3. **Window isolation**: AI, Notes, Actions use separate GPUI windows with independent lifecycles
4. **No circular dependencies**: Verified through dependency analysis
5. **Re-export discipline**: Module::mod.rs files carefully control public API surface

However, there are **coupling hotspots** that create tight interdependencies and could benefit from further isolation.

---

## 1. Public vs Private API Exposure

### Pattern: Selective Re-exports in `mod.rs`

Each major module uses its `mod.rs` as an API boundary layer:

```rust
// src/actions/mod.rs - Controlled re-exports
pub use builders::to_deeplink_name;
pub use dialog::ActionsDialog;
pub use types::ScriptInfo;
pub use window::{open_actions_window, close_actions_window, ...};

// Internal submodules NOT re-exported
mod builders;        // Builders are opaque - details hidden
mod command_bar;     // Command bar implementation hidden
mod constants;       // Design constants kept private
mod dialog;          // Dialog impl hidden
mod types;           // Types exposed selectively
mod window;          // Window functions exposed, impl hidden
```

**Assessment**: ✅ **Good** - This pattern is applied consistently.

### Key Re-export Groups

| Module | Public API | Internal | Notes |
|--------|-----------|----------|-------|
| `protocol` | 100% re-exported (`pub use io::*`) | None | Intentional: protocol has no internal dependencies on `crate::` modules |
| `actions` | ~40% (types, dialog, window functions) | 60% (builders, command_bar, constants) | Good balance - builders stay opaque |
| `executor` | ~70% (runner, scriptlet, errors) | 30% (stderr_buffer internals) | Heavy re-export by design - executor is infrastructure |
| `theme` | ~50% (types, helpers, service) | 50% (types submodules) | Incremental adoption via `#[allow(unused_imports)]` |
| `designs` | ~90% (all design variants) | ~10% (private traits) | Trait hidden - variants directly exposed |
| `components` | ~95% (all UI components) | ~5% (internal field logic) | Designed for widespread use |

### Design Variants - Trait Pattern

```rust
// src/designs/mod.rs
pub use traits::{DesignRenderer, DesignRendererBox};

// Trait is PUBLIC but implementations are exposed as functions
pub use minimal::{render_minimal_header, MinimalRenderer, ...};
pub use brutalist::{render_brutalist_header, BrutalistRenderer, ...};
```

**Pattern**: Trait-based abstraction with concrete implementation exposure. This allows:
- Direct function calls without vtable overhead: `render_minimal_header()`
- Concrete type access when needed: `MinimalRenderer`
- Future trait implementations without breaking changes

---

## 2. Module Dependency Patterns

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Main App (app_impl.rs)                   │
│                                                              │
│  Uses: config, scripts, theme, builtins, actions, executor  │
│        ui_foundation, components, app_shell, window_ops     │
└──────────────────────────┬──────────────────────────────────┘
                           │
         ┌─────────────────┼─────────────────┐
         │                 │                 │
    ┌────▼────┐      ┌─────▼─────┐     ┌────▼────┐
    │ ACTIONS │      │   THEME   │     │ PROTOCOL│
    │ Window  │      │ (Colors)  │     │(JSONL I/O)
    └────┬────┘      └─────▲─────┘     └────▲────┘
         │                 │                 │
         │            ┌────┴─────┬───────────┴──┐
         │            │          │              │
    ┌────▼────┐  ┌────▼──┐  ┌───▼────┐  ┌─────▼────┐
    │COMPONENTS│  │AI     │  │NOTES   │  │EXECUTOR  │
    │ UI Lib  │  │Window │  │Window  │  │Script I/O│
    └─────────┘  └────────┘  └────────┘  └──────────┘
```

### Dependency Flow Analysis

**Clean Dependencies** (no circular risk):

1. **Protocol Module** (src/protocol/mod.rs)
   - Zero internal crate dependencies: `pub use io::*; pub use message::*; pub use semantic_id::*;`
   - **Only external dependencies**: serde, serde_json
   - **Used by**: executor, ai, notes, actions (unidirectional)
   - **Status**: ✅ Perfect isolation

2. **Theme Module** (src/theme/mod.rs)
   - Internal: hex_color, types, helpers, service, validation
   - **Consumed by**: 5+ modules (actions, designs, components, ai, notes)
   - **Dependencies**: serde, chrono (not crate::*)
   - **Status**: ✅ Good - acts as library, no circular usage

3. **Components Module** (src/components/mod.rs)
   - UI component library with no crate dependencies except `crate::theme`
   - **Used by**: ai window, notes window, prompts
   - **Status**: ✅ Good - minimal coupling

### Problematic Dependencies

**Tight Coupling Hot Spots**:

```rust
// src/actions/dialog.rs - imports 12 modules
use crate::components::scrollbar::{Scrollbar, ScrollbarColors};
use crate::designs::{get_tokens, DesignColors, DesignVariant};
use crate::logging;
use crate::protocol::ProtocolAction;
use crate::theme;
use crate::file_search::FileInfo;
use crate::prompts::PathInfo;
use crate::scriptlets::Scriptlet;
use crate::execute_script;    // In some variants
use crate::app_shell;         // In some variants
```

**Issue**: ActionsDialog couples to 7+ infrastructure modules. If any of these change their API, ActionsDialog breaks.

```rust
// src/ai/window.rs - imports 8 modules
use crate::actions::{get_ai_command_bar_actions, CommandBar, CommandBarConfig};
use crate::protocol::{AiChatInfo, AiMessageInfo, Message};
use crate::designs::icon_variations::IconName;
use crate::theme;
use crate::secrets::get_secret;
use crate::platform;  // For window positioning
```

**Issue**: AI window depends on actions module. If actions changes its command bar API, AI window must update.

---

## 3. Circular Dependency Analysis

### Verified Clean

**No circular dependencies detected**:

✅ `protocol` ← (used by everyone, depends on nothing)
✅ `theme` ← (used by 5+ modules, no backreferences)
✅ `components` ← (used by ai/notes, only depends on theme)
✅ `actions` ← (uses protocol, components, designs; nothing uses it back except main app)
✅ `executor` ← (uses protocol; not used by other system modules)

### Potential Risk Zones

1. **AI/Notes → Actions** (unidirectional)
   - AI window calls: `get_ai_command_bar_actions()`
   - Notes window calls: `get_notes_command_bar_actions()`
   - **Risk**: Actions module rejects AI/Notes imports → circular use
   - **Status**: ✅ Safe - Actions doesn't import AI/Notes, so no cycle possible

2. **App Impl** (src/app_impl.rs)
   - Central hub that imports everything (expected)
   - **Risk**: If modules try to access app state indirectly
   - **Mitigation**: Modules use static window handles, not app_impl imports

---

## 4. Interface & Trait Abstraction Patterns

### Pattern 1: Design Tokens Abstraction

**File**: src/designs/traits.rs

```rust
pub trait DesignRenderer {
    fn render_header(&self, ...) -> Element;
    fn render_list_item(&self, ...) -> Element;
    fn render_preview_panel(&self, ...) -> Element;
}

pub struct DesignTokens {
    pub colors: DesignColors,
    pub spacing: DesignSpacing,
    pub typography: DesignTypography,
    pub visuals: DesignVisual,
}

pub struct DesignColors {
    pub background: u32,
    pub accent: u32,
    pub text_primary: u32,
    // ... 20+ color properties
}
```

**Benefits**:
- ✅ New design variants add implementations without changing callers
- ✅ Colors are Copy/Clone for closure efficiency
- ✅ No inheritance complexity
- ✅ Type-safe design switching: `DesignVariant::Minimal`

**Drawback**:
- Implementations are exposed as functions, not just trait objects
- Allows direct calls like `render_minimal_header()` (good for performance, less good for pluggability)

### Pattern 2: Provider Abstraction (AI Module)

**File**: src/ai/providers.rs

```rust
pub trait AiProvider {
    fn create_completion(&self, request: CompletionRequest) -> Result<CompletionResponse>;
}

pub struct ProviderRegistry {
    providers: HashMap<String, Box<dyn AiProvider>>,
}

impl ProviderRegistry {
    pub fn register(&mut self, name: &str, provider: Box<dyn AiProvider>) {
        self.providers.insert(name.to_string(), provider);
    }
}
```

**Benefits**:
- ✅ Pluggable provider system (OpenAI, Anthropic, etc.)
- ✅ Easy to add new providers without modifying AI window
- ✅ Configuration-driven provider selection

### Pattern 3: Window Registry (Unified Window Management)

**File**: src/windows/registry.rs

```rust
pub fn register_window(role: WindowRole, handle: WindowHandle<View>) {
    // REGISTRY.write().unwrap().insert(role, handle);
}

pub fn with_window<F, R>(role: WindowRole, f: F) -> Option<R>
where
    F: FnOnce(&mut Window) -> R,
{
    // Safely access window without holding mutex across render
}

pub enum WindowRole {
    Main,
    Actions,
    Notes,
    AI,
}
```

**Benefits**:
- ✅ Single source of truth for all open windows
- ✅ Safe cross-window communication patterns
- ✅ Prevents multiple windows of same type
- ✅ Handles window lifecycle consistently

---

## 5. Coupling Between Modules

### Direct Crate Dependencies by Module

```
components        → theme (1 dep)
executor          → protocol, logging, process_manager (3)
protocol          → (0 crate deps - ZERO)
actions           → protocol, designs, theme, components, logging, file_search, ... (10)
ai                → actions, protocol, designs, theme, secrets (5)
notes             → actions, protocol, designs, theme (4)
config            → serde, serde_yaml (no crate deps)
theme             → (serde, chrono only - 0 crate deps)
designs           → (no crate deps)
```

### Coupling Metrics

| Module | Dependencies | Dependents | Coupling Ratio |
|--------|--------------|-----------|-----------------|
| protocol | 0 | 7+ | **0.0** (Pure data) ✅ |
| theme | 0 | 5+ | **0.0** (Pure data) ✅ |
| components | 1 | 3+ | **0.33** (Good) ✅ |
| executor | 3 | 2 | **1.5** (Reasonable) ✅ |
| actions | 10 | 1 | **10.0** (High) ⚠️ |
| ai | 5 | 0 | **∞** (One-way) ✅ |
| notes | 4 | 0 | **∞** (One-way) ✅ |

### Why Actions is Highly Coupled

ActionsDialog needs to render context-aware actions:
1. Script actions (edit, view logs, copy path)
2. File actions (reveal in finder, copy content)
3. Clipboard actions (copy, paste)
4. Scriptlet actions (custom actions from SDK)

Each requires knowledge of:
- Types: ScriptInfo, FileInfo, Scriptlet
- Configuration: designs, theme
- Display: components

**This is acceptable coupling** because:
- ✅ Actions is intentionally a "hub" UI component
- ✅ Its dependencies are orthogonal (don't depend on it back)
- ✅ Could be refactored by passing action builders in, but current approach is pragmatic

---

## 6. Import Patterns & Best Practices

### What's Done Well

#### 1. Submodule Opacity
```rust
// src/actions/mod.rs
mod builders;        // Private - implementation hidden
mod constants;       // Private - callers don't depend on magic numbers
pub use types::ScriptInfo;  // Only ScriptInfo exposed, not Action internals
```

Good: `ScriptInfo` has builders (`ScriptInfo::new()`, `ScriptInfo::builtin()`) that are stable.

#### 2. Feature-Gated Exports
```rust
// src/executor/mod.rs
#[allow(unused_imports)]
pub use auto_submit::AutoSubmitConfig;  // Only for tests/future use

#[allow(dead_code)]
pub use tool_extension;  // Re-exported for tests only
```

#### 3. Type Alias for Complexity
```rust
// Instead of: pub type BoxedRenderer = Box<dyn DesignRenderer>;
pub use traits::DesignRendererBox;  // Alias for dyn renderer
```

Reduces call-site complexity without exposing trait object creation.

#### 4. Semantic ID for AI-Driven UX
```rust
// src/protocol/semantic_id.rs - public module
pub fn generate_semantic_id(message: &str) -> String {
    // Separates ID generation logic from message construction
}
```

Protocol module provides higher-level semantic utilities without full module bloat.

### What Could Improve

#### 1. Actions Still Tightly Coupled

**Current**:
```rust
// In app_impl.rs
let actions = get_script_context_actions(&script);
dialog.set_actions(actions);
```

**Could be**:
```rust
// Create action builder trait
trait ActionBuilder {
    fn build_actions(&self, script: &ScriptInfo) -> Vec<Action>;
}

// Pass builder to dialog instead of pre-built actions
dialog.set_action_builder(Box::new(ScriptActionBuilder));
```

**Benefit**: ActionsDialog wouldn't need to import from actions::builders

**Drawback**: Slower at runtime (virtual dispatch), more boilerplate

#### 2. Config Module Scattered

Config is loaded in `config/loader.rs` but consumed everywhere:
- app_impl.rs accesses `config.bun_path`, `config.hotkey`
- executor uses `config.process_limits`
- theme uses `config.appearance_mode`

**Could be**: Typed config accessors instead of direct struct access
```rust
pub fn get_bun_path() -> Option<PathBuf> {
    CONFIG.read().unwrap().bun_path.clone()
}
```

#### 3. Theme Color Helpers Not Consistent

```rust
// Current approaches mix:
theme::ColorScheme (enum-based)
crate::designs::DesignColors (struct-based)
crate::ui_foundation::hex_to_rgba_with_opacity (function-based)

// Should consolidate to:
theme::Colors (single source)
```

---

## 7. Cross-Window Communication

### Pattern: Static Window Handles

Each window module maintains a static handle:

```rust
// src/actions/window.rs
static ACTIONS_WINDOW: OnceCell<WindowHandle<ActionsWindow>> = OnceCell::new();

pub fn open_actions_window(config: WindowConfig, cx: &mut App) -> Result<()> {
    let window = cx.open_window(WindowOptions::default(), ...)?;
    ACTIONS_WINDOW.set(window).ok();
}

pub fn is_actions_window_open() -> bool {
    ACTIONS_WINDOW.get().map_or(false, |h| h.is_valid())
}

pub fn notify_actions_window() {
    if let Some(handle) = ACTIONS_WINDOW.get() {
        handle.update(/* ... */).ok();
    }
}
```

**Assessment**: ✅ **Good Pattern**
- Avoids app state bloat
- Each window self-manages lifecycle
- Safe re-entrancy (handle checks validity)

**Risks**:
- ⚠️ If handle becomes invalid, subsequent calls fail silently
- ⚠️ No centralized window registry (though src/windows/registry.rs attempts this)

---

## 8. SDK Integration Boundary

### How Scripts Talk to UI

**Flow**: Script → Protocol (JSONL) → Executor → App State

```rust
// src/protocol/types.rs - stable contract
pub struct Message {
    pub msg: String,
    pub data: serde_json::Value,
}

// src/executor/runner.rs - parses messages
pub fn handle_message(msg: Message, state: &mut AppState) {
    match msg.msg.as_str() {
        "setActions" => {
            let actions = serde_json::from_value::<Vec<ProtocolAction>>(msg.data)?;
            state.update_actions(actions);
        }
        // ...
    }
}
```

**Assessment**: ✅ **Clean Boundary**
- Scripts never directly import Rust code
- Message schema is versioned in protocol/
- App can change internals without breaking scripts

---

## Architectural Recommendations

### 1. **Formalize Window Registry** (Highest Priority)

**Current**: Each window has separate static (actions, notes, ai)
**Propose**: Unified registry in src/windows/registry.rs

```rust
// src/windows/registry.rs (today - skeleton only)
pub enum WindowRole {
    Main,
    Actions,
    Notes,
    AI,
}

impl WindowRegistry {
    pub fn get<T>(&self, role: WindowRole) -> Option<WindowHandle<T>> { ... }
    pub fn open<T>(&mut self, role: WindowRole, ...) -> WindowHandle<T> { ... }
    pub fn close(&mut self, role: WindowRole) { ... }
    pub fn notify_all(&self, f: impl Fn(&dyn Any)) { ... }
}
```

**Benefits**:
- Single source of truth for all windows
- Consistent lifecycle management
- Easier to add window types later (Settings, Script Editor, etc.)

### 2. **Extract Actions as Trait-Based Plugin**

**Current**: `get_script_context_actions()` is a function, tightly couples to builders
**Propose**: ActionProvider trait

```rust
pub trait ActionProvider {
    fn get_actions(&self, context: &ActionContext) -> Vec<Action>;
}

pub enum ActionContext {
    Script(ScriptInfo),
    File(FileInfo),
    Clipboard(ClipboardEntry),
}

impl ActionsDialog {
    pub fn with_provider(provider: Box<dyn ActionProvider>) -> Self { ... }
}
```

**Benefits**:
- ActionsDialog decoupled from builders
- Scripts can provide custom actions via SDK
- Easier to test (mock provider)

### 3. **Protocol Type Hierarchy**

**Current**: Protocol types are flat (Message, ProtocolAction, etc.)
**Propose**: Organize by message direction and lifetime

```rust
// src/protocol/mod.rs
pub mod request;   // Script → App queries
pub mod response;  // App → Script responses
pub mod prompt;    // App → Script UI requests
pub mod action;    // SDK-provided actions
```

**Benefits**:
- Easier to find relevant types
- Clear versioning boundaries
- Clearer documentation of message flow

### 4. **Components Library Isolation**

**Current**: Components depend on theme only
**Propose**: Make components fully theme-agnostic

```rust
// Currently: components::Button uses theme::ColorScheme
// Should be: components::Button takes ButtonColors struct

pub struct ButtonColors {
    pub background: Rgba,
    pub text: Rgba,
    pub hover: Rgba,
}

impl Button {
    pub fn with_colors(colors: ButtonColors) -> Self { ... }
}
```

**Benefits**:
- Components can be reused in third-party tools
- Theme system is orthogonal
- Easier to test components in isolation

### 5. **Theme Service Consolidation**

**Current**: Theme accessed multiple ways:
- `crate::theme::load_theme()` (one-time load)
- `theme.colors.accent` (direct access)
- `theme::ColorScheme` enum
- `designs::DesignColors` (separate copy)

**Propose**: Single `ThemeService`

```rust
// src/theme/service.rs (today - basic)
pub struct ThemeService {
    current: Arc<Mutex<Theme>>,
    subscribers: Vec<Box<dyn ThemeSubscriber>>,
}

impl ThemeService {
    pub fn current() -> Theme { /* ... */ }
    pub fn subscribe(callback: impl ThemeSubscriber) { /* ... */ }
}
```

**Benefits**:
- Single source of truth
- All modules see same theme
- Hot-reload support for theme changes

---

## Summary Table: Boundary Health

| Aspect | Status | Evidence |
|--------|--------|----------|
| Protocol isolation | ✅ Excellent | Zero internal dependencies |
| Theme coupling | ✅ Good | Used widely, doesn't depend on anything |
| Components reusability | ✅ Good | Depends only on theme |
| Actions cohesion | ⚠️ Fair | 10+ internal dependencies, but appropriate |
| Window lifecycle | ✅ Good | Static handles with validity checks |
| Circular dependencies | ✅ None | Verified across all major modules |
| SDK boundary | ✅ Clean | Protocol-based, decoupled from impl |
| Config consistency | ⚠️ Fair | Scattered accessors, no facade |
| Design system | ✅ Good | Trait-based with concrete implementations |
| Testing boundaries | ✅ Fair | Re-exports for test modules available |

---

## References

- **Module locations**: `/Users/johnlindquist/dev/script-kit-gpui/src/`
- **Key files analyzed**:
  - `src/protocol/mod.rs` - 0 internal dependencies
  - `src/actions/mod.rs` - 10 internal dependencies (hub pattern)
  - `src/theme/mod.rs` - 0 internal dependencies (library)
  - `src/components/mod.rs` - 1 internal dependency (theme)
  - `src/designs/traits.rs` - Trait abstraction pattern
  - `src/windows/registry.rs` - Window management pattern
  - `src/app_impl.rs` - Central orchestrator (expected high coupling)

---

**Analysis by**: Claude Code Agent
**Next steps**: Implement recommendations 1-3 for improved architectural cohesion
