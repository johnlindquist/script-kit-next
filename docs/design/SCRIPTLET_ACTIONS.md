# Scriptlet Actions Design

## Overview

Extension scriptlets will support H3 headers with tool codefences to define **custom actions** that appear in the Actions Menu when the scriptlet is focused in the main menu.

## User-Facing Format

### Markdown Structure

```markdown
---
name: My Tools Bundle
---

# Development Tools

## Open GitHub Repo
<!-- shortcut: cmd+g -->

Opens the current repo in GitHub.

```open
https://github.com/{{repo}}
```

### Copy SSH URL
<!-- shortcut: cmd+shift+c -->
```bash
echo "git@github.com:{{repo}}.git" | pbcopy
```

### Open in VSCode
```bash
code ~/projects/{{repo}}
```

### View README
```open
https://github.com/{{repo}}/blob/main/README.md
```
```

### Header Hierarchy

| Header | Purpose | Example |
|--------|---------|---------|
| `# H1` | Group name (optional global prepend) | `# Development Tools` |
| `## H2` | Scriptlet definition (main code) | `## Open GitHub Repo` |
| `### H3` | Scriptlet action (appears in Actions Menu) | `### Copy SSH URL` |

### Action Metadata (Optional)

H3 actions can include metadata via HTML comments:

```markdown
### My Action
<!-- shortcut: cmd+shift+a -->
<!-- description: Does something useful -->
```bash
echo "action code"
```
```

Supported metadata fields:
- `shortcut` - Keyboard shortcut hint (e.g., `cmd+shift+a`)
- `description` - Description shown in Actions Menu

---

## Technical Design

### 1. New Types (`src/scriptlets.rs`)

```rust
/// An action defined within a scriptlet via H3 + codefence
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ScriptletAction {
    /// Name from H3 header (e.g., "Copy SSH URL")
    pub name: String,
    
    /// Slugified command identifier
    pub command: String,
    
    /// Tool type from codefence (e.g., "bash", "open", "transform")
    pub tool: String,
    
    /// Code content from codefence
    pub code: String,
    
    /// Named input placeholders (e.g., ["repo", "branch"])
    pub inputs: Vec<String>,
    
    /// Optional keyboard shortcut hint
    pub shortcut: Option<String>,
    
    /// Optional description
    pub description: Option<String>,
}

impl ScriptletAction {
    /// Create action ID for the Actions Menu (prefixed to avoid collisions)
    pub fn action_id(&self) -> String {
        format!("scriptlet_action:{}", self.command)
    }
}
```

### 2. Extended Scriptlet Struct

```rust
pub struct Scriptlet {
    // ... existing fields ...
    
    /// Actions defined via H3 headers within this scriptlet
    pub actions: Vec<ScriptletAction>,
}
```

### 3. Parser Changes (`src/scriptlets.rs`)

The parser needs to:
1. Continue splitting on H1/H2 as before
2. Within each H2 section, scan for H3 headers
3. Extract H3 name + metadata + codefence as `ScriptletAction`

```rust
/// Extract H3 actions from a scriptlet section
fn extract_h3_actions(section_text: &str) -> Vec<ScriptletAction> {
    let mut actions = Vec::new();
    let mut current_h3_name: Option<String> = None;
    let mut current_h3_content = String::new();
    let mut in_h3_section = false;
    
    for line in section_text.lines() {
        let trimmed = line.trim_start();
        
        if trimmed.starts_with("### ") {
            // Save previous H3 if exists
            if let Some(name) = current_h3_name.take() {
                if let Some(action) = parse_h3_action(&name, &current_h3_content) {
                    actions.push(action);
                }
            }
            
            // Start new H3 section
            current_h3_name = Some(trimmed.strip_prefix("### ").unwrap().trim().to_string());
            current_h3_content.clear();
            in_h3_section = true;
        } else if in_h3_section {
            // Accumulate content for current H3
            current_h3_content.push_str(line);
            current_h3_content.push('\n');
        }
    }
    
    // Don't forget last H3
    if let Some(name) = current_h3_name {
        if let Some(action) = parse_h3_action(&name, &current_h3_content) {
            actions.push(action);
        }
    }
    
    actions
}

/// Parse a single H3 action from its content
fn parse_h3_action(name: &str, content: &str) -> Option<ScriptletAction> {
    // Extract metadata from HTML comments
    let metadata = parse_html_comment_metadata(content);
    
    // Extract code block
    let (tool, code) = extract_code_block_nested(content)?;
    
    // Only create action if tool is valid
    if !VALID_TOOLS.contains(&tool.as_str()) && !tool.is_empty() {
        return None;
    }
    
    let tool = if tool.is_empty() { "bash".to_string() } else { tool };
    let inputs = extract_named_inputs(&code);
    let command = slugify(name);
    
    Some(ScriptletAction {
        name: name.to_string(),
        command,
        tool,
        code,
        inputs,
        shortcut: metadata.shortcut,
        description: metadata.description,
    })
}
```

### 4. Actions Integration (`src/actions/builders.rs`)

Add function to convert scriptlet actions to UI actions:

```rust
/// Convert scriptlet-defined actions to Action structs for the UI
pub fn get_scriptlet_defined_actions(scriptlet: &Scriptlet) -> Vec<Action> {
    scriptlet.actions.iter().map(|sa| {
        let mut action = Action::new(
            sa.action_id(),
            &sa.name,
            sa.description.clone(),
            ActionCategory::ScriptContext,
        );
        
        if let Some(ref shortcut) = sa.shortcut {
            action = action.with_shortcut(format_shortcut_hint(shortcut));
        }
        
        // Mark as scriptlet action for routing
        action.has_action = true;  // Will be handled specially
        action.value = Some(sa.command.clone());
        
        action
    }).collect()
}

/// Merge scriptlet-defined actions with built-in actions
pub fn get_scriptlet_context_actions_with_custom(
    script: &ScriptInfo,
    scriptlet: Option<&Scriptlet>,
) -> Vec<Action> {
    let mut actions = Vec::new();
    
    // 1. Primary action (Run)
    actions.push(/* run action */);
    
    // 2. Scriptlet-defined actions (from H3s)
    if let Some(scriptlet) = scriptlet {
        actions.extend(get_scriptlet_defined_actions(scriptlet));
    }
    
    // 3. Built-in scriptlet actions (Edit, Reveal, Copy Path)
    actions.extend(get_built_in_scriptlet_actions(script));
    
    // 4. Universal actions (Shortcut, Alias, Deeplink)
    actions.extend(get_universal_actions(script));
    
    actions
}
```

### 5. Action Execution

When a scriptlet action is triggered:

```rust
// In actions/handlers.rs or similar
fn handle_scriptlet_action(
    scriptlet: &Scriptlet,
    action_command: &str,
) -> anyhow::Result<()> {
    // Find the matching action
    let action = scriptlet.actions.iter()
        .find(|a| a.command == action_command)
        .ok_or_else(|| anyhow::anyhow!("Action not found: {}", action_command))?;
    
    // Execute using existing scriptlet execution infrastructure
    execute_scriptlet_code(&action.tool, &action.code, &action.inputs)
}
```

---

## Data Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                        Markdown File                             │
│  ## My Scriptlet                                                 │
│  ```bash                                                         │
│  main code                                                       │
│  ```                                                             │
│  ### Action One          ### Action Two                          │
│  ```bash                 ```open                                 │
│  action 1 code           https://...                             │
│  ```                     ```                                     │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      parse_markdown_as_scriptlets()              │
│  - Split by H1/H2 (existing)                                     │
│  - Within H2 section, extract H3 actions (new)                   │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                        Scriptlet struct                          │
│  {                                                               │
│    name: "My Scriptlet",                                         │
│    tool: "bash",                                                 │
│    scriptlet_content: "main code",                               │
│    actions: [                                                    │
│      { name: "Action One", tool: "bash", code: "action 1" },     │
│      { name: "Action Two", tool: "open", code: "https://..." }   │
│    ]                                                             │
│  }                                                               │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                     Actions Menu (when focused)                  │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │ Run "My Scriptlet"                              ↵       │    │
│  ├─────────────────────────────────────────────────────────┤    │
│  │ Action One                                     ⌘⇧1      │    │  ← From H3
│  │ Action Two                                     ⌘⇧2      │    │  ← From H3
│  ├─────────────────────────────────────────────────────────┤    │
│  │ Edit Scriptlet                                 ⌘E       │    │
│  │ Reveal in Finder                               ⌘⇧F      │    │
│  │ Copy Path                                      ⌘⇧C      │    │
│  └─────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────┘
```

---

## UI/UX Considerations

### Action Ordering

1. **Primary Action** - Run the scriptlet (always first)
2. **Custom Actions** - H3-defined actions (in document order)
3. **Built-in Actions** - Edit, Reveal, Copy Path
4. **Universal Actions** - Shortcut, Alias, Deeplink, etc.

### Visual Grouping (Optional Enhancement)

Consider adding a visual separator between custom and built-in actions:

```
Run "My Scriptlet"                              ↵
────────────────────────────────────────────────
Copy SSH URL                                   ⌘⇧C
Open in VSCode                                 ⌘⇧V
────────────────────────────────────────────────
Edit Scriptlet                                 ⌘E
Reveal in Finder                               ⌘⇧F
```

### Empty State

If a scriptlet has no H3 actions, the Actions Menu shows only built-in actions (current behavior).

---

## Implementation Plan

### Phase 1: Parser Changes
1. Add `ScriptletAction` struct to `scriptlets.rs`
2. Add `actions: Vec<ScriptletAction>` field to `Scriptlet`
3. Implement `extract_h3_actions()` function
4. Integrate into `parse_markdown_as_scriptlets()`
5. Add unit tests for H3 parsing

### Phase 2: Actions Integration
1. Add `get_scriptlet_defined_actions()` to `builders.rs`
2. Modify `get_script_context_actions()` to accept optional `Scriptlet`
3. Update `ActionsDialog` to pass scriptlet when available
4. Wire up action routing for `scriptlet_action:*` IDs

### Phase 3: Execution
1. Add handler for scriptlet action execution
2. Reuse existing scriptlet execution code
3. Handle variable substitution for action inputs

### Phase 4: Testing & Polish
1. E2E test with visual verification
2. Test edge cases (no actions, invalid codefences, nested fences)
3. Documentation updates

---

## Edge Cases

### No Codefence in H3
H3 without a valid tool codefence is ignored (not an action).

### Invalid Tool Type
H3 with unrecognized tool type is ignored with a debug log.

### Nested Code Fences
The existing fence detection handles nested fences (`~~~` inside ``` and vice versa).

### H3 Before First H2
H3 headers before any H2 are part of the document preamble and ignored.

### Variables in Actions
Actions inherit the variable namespace from their parent scriptlet:
- `{{selection}}` - Current selection
- `{{clipboard}}` - Clipboard content
- Custom inputs defined in the scriptlet

---

## Example: Full Scriptlet Bundle

```markdown
---
name: Developer Tools
author: Script Kit Team
icon: code
---

# Git Tools

## Git Status
<!-- shortcut: cmd+shift+s -->

Quick git status for the current project.

```bash
cd "{{projectPath}}" && git status
```

### Commit All
<!-- shortcut: cmd+shift+c -->
<!-- description: Stage and commit all changes -->
```bash
cd "{{projectPath}}" && git add -A && git commit -m "{{message}}"
```

### Push
<!-- shortcut: cmd+shift+p -->
```bash
cd "{{projectPath}}" && git push
```

### Pull
```bash
cd "{{projectPath}}" && git pull
```

## Open Repo in Browser
<!-- trigger: repo -->

```open
https://github.com/{{owner}}/{{repo}}
```

### Open Issues
```open
https://github.com/{{owner}}/{{repo}}/issues
```

### Open PRs
```open
https://github.com/{{owner}}/{{repo}}/pulls
```

### Copy Clone URL
```bash
echo "https://github.com/{{owner}}/{{repo}}.git" | pbcopy
```
```

---

## Related Files

- `src/scriptlets.rs` - Parser and data structures
- `src/actions/types.rs` - Action types
- `src/actions/builders.rs` - Action factory functions
- `src/actions/dialog.rs` - Actions dialog UI
- `src/executor/scriptlet.rs` - Scriptlet execution

---

## Acceptance Criteria

- [ ] H3 headers with valid tool codefences are parsed as actions
- [ ] Actions appear in Actions Menu when scriptlet is focused
- [ ] Actions execute their tool+code when triggered
- [ ] Keyboard shortcuts work when specified
- [ ] Variables are substituted in action code
- [ ] Invalid H3s (no codefence, invalid tool) are gracefully ignored
- [ ] Existing scriptlet functionality is unchanged
- [ ] Unit tests cover parsing edge cases
- [ ] E2E test verifies visual appearance
