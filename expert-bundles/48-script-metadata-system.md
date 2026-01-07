# Script Kit GPUI - Expert Bundle 48: Script Metadata System

## Project Context

Script Kit GPUI is a **Rust desktop app** built with GPUI (Zed's UI framework) that serves as a command launcher and script runner.

**Script Metadata Sources:**
- Comment-based metadata (`// Name: My Script`)
- Export-based metadata (`export const metadata = { ... }`)
- File system structure (folders, naming conventions)
- Scriptlet codefence metadata

---

## Goal

Create a **unified metadata system** that:
1. Supports both legacy comment syntax and new export syntax
2. Provides type-safe parsing and validation
3. Enables rich script configuration
4. Supports scriptlet-specific metadata
5. Offers IDE tooling support (autocomplete, validation)

---

## Current State

### Supported Metadata Fields

| Field | Comment Syntax | Export Syntax | Used In |
|-------|---------------|---------------|---------|
| Name | `// Name: X` | `name: "X"` | Script list |
| Description | `// Description: X` | `description: "X"` | Preview |
| Shortcut | `// Shortcut: cmd+g` | `shortcut: "cmd+g"` | Global hotkey |
| Alias | `// Alias: g` | `alias: "g"` | Quick launch |
| Schedule | `// Schedule: every 5 minutes` | `schedule: "..."` | Scheduler |
| Cron | `// Cron: 0 9 * * *` | `cron: "..."` | Scheduler |
| Author | `// Author: John` | `author: "John"` | Preview |
| Icon | (not supported) | (not supported) | (missing) |
| Group | (not supported) | (not supported) | (missing) |
| Tags | (not supported) | (not supported) | (missing) |

### Current Parsing

```rust
// src/scripts.rs - Comment-based parsing
fn parse_script_metadata(content: &str) -> ScriptMetadata {
    let mut metadata = ScriptMetadata::default();
    
    for line in content.lines() {
        if !line.starts_with("//") { break; }
        
        if let Some(name) = line.strip_prefix("// Name:") {
            metadata.name = Some(name.trim().to_string());
        } else if let Some(desc) = line.strip_prefix("// Description:") {
            metadata.description = Some(desc.trim().to_string());
        }
        // ... more fields
    }
    
    metadata
}

// src/metadata_parser.rs - Export-based parsing
fn parse_export_metadata(content: &str) -> Option<ScriptMetadata> {
    // Regex to find: export const metadata = { ... }
    // Parse as JSON5 (allows trailing commas, comments)
}
```

### Problems

1. **Two Parsers** - Comment and export parsing are separate
2. **No Validation** - Invalid metadata silently ignored
3. **Limited Fields** - Missing icon, group, tags, permissions
4. **No Type Safety** - TypeScript types not enforced at runtime
5. **No IDE Support** - No autocomplete for metadata fields
6. **Scriptlet Inconsistent** - Different parsing for codefence metadata

---

## Proposed Architecture

### 1. Unified Metadata Schema

```rust
/// Complete script metadata schema
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScriptMetadata {
    // Identity
    pub name: Option<String>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub version: Option<String>,
    
    // Organization
    pub group: Option<String>,
    pub tags: Option<Vec<String>>,
    pub icon: Option<String>,        // Icon reference (see Bundle 45)
    pub color: Option<String>,       // Accent color for this script
    
    // Activation
    pub shortcut: Option<String>,
    pub alias: Option<String>,
    pub trigger: Option<String>,     // Text expansion trigger
    
    // Scheduling
    pub schedule: Option<String>,    // Natural language schedule
    pub cron: Option<String>,        // Cron expression
    
    // Behavior
    pub background: Option<bool>,    // Run without UI
    pub timeout: Option<u64>,        // Max execution time (ms)
    pub log: Option<LogLevel>,       // Logging level
    
    // Permissions
    pub permissions: Option<Permissions>,
    
    // UI Hints
    pub preview: Option<PreviewConfig>,
    pub actions: Option<Vec<ActionDefinition>>,
    
    // Source info (set by parser)
    #[serde(skip)]
    pub source_file: Option<PathBuf>,
    #[serde(skip)]
    pub source_format: MetadataFormat,
}

#[derive(Debug, Clone, Default)]
pub enum MetadataFormat {
    #[default]
    Comment,      // // Name: X
    Export,       // export const metadata = {}
    Frontmatter,  // ---\nname: X\n---
    Codefence,    // ```metadata\n{}\n```
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Permissions {
    pub clipboard: Option<bool>,
    pub notifications: Option<bool>,
    pub filesystem: Option<FilesystemPermission>,
    pub network: Option<NetworkPermission>,
    pub accessibility: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilesystemPermission {
    None,
    ReadOnly,
    ReadWrite,
    Paths(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkPermission {
    None,
    All,
    Hosts(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewConfig {
    pub mode: PreviewMode,
    pub height: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PreviewMode {
    Code,       // Show script source
    Output,     // Show last output
    Custom,     // Custom preview HTML
    None,       // No preview
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionDefinition {
    pub name: String,
    pub shortcut: Option<String>,
    pub handler: String,  // Function name to call
}
```

### 2. Unified Parser

```rust
/// Unified metadata parser supporting all formats
pub struct MetadataParser {
    /// Cache of parsed metadata
    cache: HashMap<PathBuf, (SystemTime, ScriptMetadata)>,
}

impl MetadataParser {
    /// Parse metadata from script content
    pub fn parse(&mut self, path: &Path, content: &str) -> Result<ScriptMetadata> {
        // Check cache first
        if let Some((mtime, cached)) = self.cache.get(path) {
            if path.metadata()?.modified()? == *mtime {
                return Ok(cached.clone());
            }
        }
        
        // Detect format and parse
        let metadata = if content.contains("export const metadata") {
            self.parse_export(content)?
        } else if content.starts_with("---\n") {
            self.parse_frontmatter(content)?
        } else if content.contains("```metadata") {
            self.parse_codefence(content)?
        } else {
            self.parse_comments(content)?
        };
        
        // Validate
        self.validate(&metadata)?;
        
        // Update cache
        let mtime = path.metadata()?.modified()?;
        self.cache.insert(path.to_path_buf(), (mtime, metadata.clone()));
        
        Ok(metadata)
    }
    
    fn parse_comments(&self, content: &str) -> Result<ScriptMetadata> {
        let mut metadata = ScriptMetadata::default();
        metadata.source_format = MetadataFormat::Comment;
        
        for line in content.lines() {
            let line = line.trim();
            if !line.starts_with("//") {
                if !line.is_empty() && !line.starts_with("import") {
                    break; // End of metadata section
                }
                continue;
            }
            
            let line = line.trim_start_matches("//").trim();
            
            // Parse key: value
            if let Some((key, value)) = line.split_once(':') {
                let key = key.trim().to_lowercase();
                let value = value.trim();
                
                match key.as_str() {
                    "name" => metadata.name = Some(value.to_string()),
                    "description" | "desc" => metadata.description = Some(value.to_string()),
                    "shortcut" => metadata.shortcut = Some(value.to_string()),
                    "alias" => metadata.alias = Some(value.to_string()),
                    "author" => metadata.author = Some(value.to_string()),
                    "schedule" => metadata.schedule = Some(value.to_string()),
                    "cron" => metadata.cron = Some(value.to_string()),
                    "group" => metadata.group = Some(value.to_string()),
                    "tags" => metadata.tags = Some(
                        value.split(',').map(|s| s.trim().to_string()).collect()
                    ),
                    "icon" => metadata.icon = Some(value.to_string()),
                    "background" => metadata.background = Some(value == "true"),
                    "timeout" => metadata.timeout = value.parse().ok(),
                    "trigger" | "expand" => metadata.trigger = Some(value.to_string()),
                    _ => {} // Unknown fields ignored with warning
                }
            }
        }
        
        Ok(metadata)
    }
    
    fn parse_export(&self, content: &str) -> Result<ScriptMetadata> {
        // Find export const metadata = { ... }
        let re = Regex::new(r"export\s+const\s+metadata\s*=\s*(\{[\s\S]*?\n\})")?;
        
        if let Some(captures) = re.captures(content) {
            let json_str = &captures[1];
            // Use json5 for lenient parsing (trailing commas, comments)
            let mut metadata: ScriptMetadata = json5::from_str(json_str)?;
            metadata.source_format = MetadataFormat::Export;
            Ok(metadata)
        } else {
            Ok(ScriptMetadata::default())
        }
    }
    
    fn validate(&self, metadata: &ScriptMetadata) -> Result<()> {
        // Validate shortcut format
        if let Some(ref shortcut) = metadata.shortcut {
            Shortcut::parse(shortcut)
                .ok_or_else(|| anyhow!("Invalid shortcut format: {}", shortcut))?;
        }
        
        // Validate cron expression
        if let Some(ref cron) = metadata.cron {
            cron_parser::parse(cron)
                .map_err(|e| anyhow!("Invalid cron expression: {} - {}", cron, e))?;
        }
        
        // Validate schedule
        if let Some(ref schedule) = metadata.schedule {
            natural_schedule::parse(schedule)
                .map_err(|e| anyhow!("Invalid schedule: {} - {}", schedule, e))?;
        }
        
        Ok(())
    }
}
```

### 3. TypeScript SDK Types

```typescript
// In kit-sdk.d.ts

/**
 * Script metadata configuration
 * Use `export const metadata = { ... }` at the top of your script
 */
export interface ScriptMetadata {
  /** Display name (defaults to filename) */
  name?: string;
  
  /** Description shown in preview */
  description?: string;
  
  /** Script author */
  author?: string;
  
  /** Script version (semver) */
  version?: string;
  
  /** Group/folder for organization */
  group?: string;
  
  /** Tags for filtering */
  tags?: string[];
  
  /** Icon (lucide:name, sf:name, app:bundle.id, or URL) */
  icon?: string;
  
  /** Accent color (hex) */
  color?: string;
  
  /** Global keyboard shortcut */
  shortcut?: string;
  
  /** Quick alias (single character or short string) */
  alias?: string;
  
  /** Text expansion trigger */
  trigger?: string;
  
  /** Natural language schedule */
  schedule?: string;
  
  /** Cron expression */
  cron?: string;
  
  /** Run in background (no UI) */
  background?: boolean;
  
  /** Max execution time in milliseconds */
  timeout?: number;
  
  /** Required permissions */
  permissions?: {
    clipboard?: boolean;
    notifications?: boolean;
    filesystem?: 'none' | 'read' | 'write' | string[];
    network?: 'none' | 'all' | string[];
    accessibility?: boolean;
  };
  
  /** Preview configuration */
  preview?: {
    mode: 'code' | 'output' | 'custom' | 'none';
    height?: number;
  };
  
  /** Custom actions for this script */
  actions?: Array<{
    name: string;
    shortcut?: string;
    handler: string;
  }>;
}

// Usage:
export const metadata: ScriptMetadata = {
  name: "Git Commit",
  description: "Quick commit with message",
  shortcut: "cmd+shift+g",
  group: "Git",
  tags: ["git", "vcs", "commit"],
  icon: "lucide:git-commit",
  permissions: {
    filesystem: 'read',
  }
};
```

### 4. Metadata Validation Errors

```rust
#[derive(Debug)]
pub enum MetadataError {
    /// Invalid shortcut format
    InvalidShortcut { value: String, reason: String },
    /// Invalid cron expression
    InvalidCron { value: String, reason: String },
    /// Invalid schedule
    InvalidSchedule { value: String, reason: String },
    /// Unknown permission
    UnknownPermission { name: String },
    /// Invalid icon reference
    InvalidIcon { value: String },
    /// JSON parsing error
    ParseError { format: MetadataFormat, error: String },
}

impl MetadataError {
    /// User-friendly error message
    pub fn message(&self) -> String {
        match self {
            Self::InvalidShortcut { value, reason } => 
                format!("Invalid shortcut '{}': {}", value, reason),
            Self::InvalidCron { value, reason } =>
                format!("Invalid cron '{}': {}", value, reason),
            // ...
        }
    }
    
    /// Suggested fix
    pub fn suggestion(&self) -> Option<String> {
        match self {
            Self::InvalidShortcut { .. } => Some(
                "Use format: cmd+shift+k, ctrl+alt+1".to_string()
            ),
            Self::InvalidCron { .. } => Some(
                "Use format: minute hour day month weekday (e.g., '0 9 * * 1-5')".to_string()
            ),
            // ...
            _ => None,
        }
    }
}
```

---

## Implementation Checklist

### Phase 1: Schema Definition
- [ ] Define complete `ScriptMetadata` struct
- [ ] Add new fields (icon, group, tags, permissions)
- [ ] Create TypeScript SDK types
- [ ] Document all fields

### Phase 2: Unified Parser
- [ ] Create `MetadataParser` with caching
- [ ] Support comment format
- [ ] Support export format
- [ ] Support frontmatter format
- [ ] Support codefence format

### Phase 3: Validation
- [ ] Validate shortcuts
- [ ] Validate cron expressions
- [ ] Validate schedules
- [ ] Validate permissions
- [ ] Return helpful error messages

### Phase 4: Integration
- [ ] Update script loading to use new parser
- [ ] Add metadata to protocol
- [ ] Show validation errors in UI
- [ ] Cache parsed metadata

### Phase 5: Tooling
- [ ] Add metadata snippet for IDE
- [ ] Create metadata generator script
- [ ] Add metadata preview in actions
- [ ] Document migration from comments to export

---

## Key Questions

1. Should comment and export metadata be combinable (export overrides comments)?
2. How to handle metadata for scriptlets (different format)?
3. Should there be a JSON Schema for IDE validation?
4. How to version metadata schema for future changes?
5. Should permissions be enforced or just informational?

---

## Related Bundles

- Bundle 13: Script Loading - uses metadata parser
- Bundle 46: Keyboard Shortcuts - uses shortcut field
- Bundle 45: Icon Library - uses icon field
