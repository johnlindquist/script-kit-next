# Scriptlet System - Expert Bundle

## Overview

Scriptlets are embedded scripts in markdown files with frontmatter metadata. They enable quick text expansion, template insertion, and mini-scripts without separate files.

## Scriptlet Bundle Structure

### File Location

```
~/.scriptkit/snippets/*.md
```

### Bundle Format

```markdown
---
name: My API Tools
description: Collection of API utilities
author: John Doe
icon: api
---

## Get User {{username}}

\`\`\`template
Hello {{username}}, your API key is: {{API_KEY}}
\`\`\`

## Shell: List Files

\`\`\`shell
ls -la {{directory}}
\`\`\`

## TypeScript: Fetch Data

\`\`\`typescript
const response = await fetch('{{url}}');
const data = await response.json();
console.log(data);
\`\`\`
```

## Frontmatter Parsing (src/scriptlets.rs)

```rust
#[derive(Debug, Clone, Default)]
pub struct BundleFrontmatter {
    pub name: Option<String>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub icon: Option<String>,
}

pub fn parse_frontmatter(content: &str) -> (Option<BundleFrontmatter>, &str) {
    let content = content.trim_start();
    
    // Must start with ---
    if !content.starts_with("---") {
        return (None, content);
    }
    
    // Find closing ---
    let after_start = &content[3..];
    if let Some(end_pos) = after_start.find("\n---") {
        let yaml_content = &after_start[..end_pos].trim();
        let remaining = &after_start[end_pos + 4..].trim_start();
        
        match serde_yaml::from_str::<BundleFrontmatter>(yaml_content) {
            Ok(fm) => (Some(fm), remaining),
            Err(e) => {
                logging::log_warn("SCRIPTLET", &format!(
                    "Invalid YAML in frontmatter: {}", e
                ));
                (None, content)
            }
        }
    } else {
        logging::log_warn("SCRIPTLET", "Unclosed frontmatter (missing ---)");
        (None, content)
    }
}
```

## Scriptlet Types

### Tool Types

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum ScriptletType {
    Template,    // Text with {{variables}}
    Shell,       // Shell commands
    TypeScript,  // TypeScript code
    AppleScript, // macOS automation
    Python,      // Python scripts
    Transform,   // Text transformations
}

impl ScriptletType {
    pub fn from_code_block(lang: &str) -> Option<Self> {
        match lang.to_lowercase().as_str() {
            "template" => Some(Self::Template),
            "shell" | "bash" | "sh" | "zsh" => Some(Self::Shell),
            "typescript" | "ts" => Some(Self::TypeScript),
            "applescript" => Some(Self::AppleScript),
            "python" | "py" => Some(Self::Python),
            "transform" => Some(Self::Transform),
            _ => None,
        }
    }

    pub fn default_icon(&self) -> &'static str {
        match self {
            Self::Template => "text-cursor-input",
            Self::Shell => "terminal",
            Self::TypeScript => "code",
            Self::AppleScript => "apple",
            Self::Python => "code",
            Self::Transform => "wand",
        }
    }
}
```

## Scriptlet Parsing

### Parse Scriptlets from Bundle

```rust
#[derive(Debug, Clone)]
pub struct Scriptlet {
    pub name: String,
    pub content: String,
    pub scriptlet_type: ScriptletType,
    pub variables: Vec<String>,
    pub shortcut: Option<String>,
    pub icon: Option<String>,
    pub file_path: Option<String>,
    pub bundle_name: Option<String>,
}

pub fn parse_scriptlets(content: &str, bundle_path: &Path) -> Vec<Scriptlet> {
    let (frontmatter, body) = parse_frontmatter(content);
    let bundle_name = frontmatter.as_ref()
        .and_then(|fm| fm.name.clone())
        .unwrap_or_else(|| {
            bundle_path.file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default()
        });
    
    let mut scriptlets = Vec::new();
    let mut current_name = String::new();
    let mut in_code_block = false;
    let mut code_lang = String::new();
    let mut code_content = String::new();
    
    for line in body.lines() {
        if line.starts_with("## ") {
            // New scriptlet heading
            current_name = line[3..].trim().to_string();
        } else if line.starts_with("```") && !in_code_block {
            // Start code block
            in_code_block = true;
            code_lang = line[3..].trim().to_string();
            code_content.clear();
        } else if line == "```" && in_code_block {
            // End code block
            in_code_block = false;
            
            if let Some(scriptlet_type) = ScriptletType::from_code_block(&code_lang) {
                let variables = extract_variables(&code_content);
                
                scriptlets.push(Scriptlet {
                    name: current_name.clone(),
                    content: code_content.clone(),
                    scriptlet_type,
                    variables,
                    shortcut: None,
                    icon: frontmatter.as_ref().and_then(|fm| fm.icon.clone()),
                    file_path: Some(bundle_path.to_string_lossy().to_string()),
                    bundle_name: Some(bundle_name.clone()),
                });
            }
        } else if in_code_block {
            code_content.push_str(line);
            code_content.push('\n');
        }
    }
    
    scriptlets
}
```

## Variable Extraction

```rust
use regex::Regex;

pub fn extract_variables(content: &str) -> Vec<String> {
    let re = Regex::new(r"\{\{(\w+)\}\}").unwrap();
    
    let mut vars: Vec<String> = re.captures_iter(content)
        .map(|cap| cap[1].to_string())
        .collect();
    
    // Deduplicate while preserving order
    let mut seen = std::collections::HashSet::new();
    vars.retain(|v| seen.insert(v.clone()));
    
    vars
}
```

## Scriptlet Execution (src/executor/scriptlet.rs)

### Template Expansion

```rust
pub fn expand_template(content: &str, variables: &HashMap<String, String>) -> String {
    let mut result = content.to_string();
    
    for (key, value) in variables {
        let pattern = format!("{{{{{}}}}}", key);
        result = result.replace(&pattern, value);
    }
    
    result
}
```

### Shell Execution

```rust
pub async fn execute_shell_scriptlet(
    content: &str,
    variables: &HashMap<String, String>,
) -> Result<ScriptletResult> {
    let expanded = expand_template(content, variables);
    
    let output = Command::new("sh")
        .arg("-c")
        .arg(&expanded)
        .output()
        .await?;
    
    if output.status.success() {
        Ok(ScriptletResult::Output(
            String::from_utf8_lossy(&output.stdout).to_string()
        ))
    } else {
        Err(anyhow!(
            "Shell command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}
```

### TypeScript Execution

```rust
pub async fn execute_typescript(
    content: &str,
    variables: &HashMap<String, String>,
) -> Result<ScriptletResult> {
    let expanded = expand_template(content, variables);
    
    // Create temp file
    let temp_dir = tempfile::tempdir()?;
    let script_path = temp_dir.path().join("scriptlet.ts");
    fs::write(&script_path, &expanded)?;
    
    // Execute with bun
    let output = Command::new("bun")
        .arg("run")
        .arg(&script_path)
        .output()
        .await?;
    
    if output.status.success() {
        Ok(ScriptletResult::Output(
            String::from_utf8_lossy(&output.stdout).to_string()
        ))
    } else {
        Err(anyhow!(
            "TypeScript execution failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}
```

## Transform Types

```rust
pub enum TransformType {
    Uppercase,
    Lowercase,
    TitleCase,
    SnakeCase,
    CamelCase,
    KebabCase,
    Trim,
    Base64Encode,
    Base64Decode,
    UrlEncode,
    UrlDecode,
    JsonFormat,
    JsonMinify,
}

pub fn apply_transform(input: &str, transform: TransformType) -> String {
    match transform {
        TransformType::Uppercase => input.to_uppercase(),
        TransformType::Lowercase => input.to_lowercase(),
        TransformType::TitleCase => to_title_case(input),
        TransformType::SnakeCase => to_snake_case(input),
        TransformType::CamelCase => to_camel_case(input),
        TransformType::KebabCase => to_kebab_case(input),
        TransformType::Trim => input.trim().to_string(),
        TransformType::Base64Encode => base64::encode(input),
        TransformType::Base64Decode => {
            base64::decode(input)
                .map(|b| String::from_utf8_lossy(&b).to_string())
                .unwrap_or_else(|_| input.to_string())
        }
        TransformType::UrlEncode => urlencoding::encode(input).to_string(),
        TransformType::UrlDecode => urlencoding::decode(input)
            .unwrap_or_else(|_| input.into()),
        TransformType::JsonFormat => {
            serde_json::from_str::<serde_json::Value>(input)
                .map(|v| serde_json::to_string_pretty(&v).unwrap_or_default())
                .unwrap_or_else(|_| input.to_string())
        }
        TransformType::JsonMinify => {
            serde_json::from_str::<serde_json::Value>(input)
                .map(|v| serde_json::to_string(&v).unwrap_or_default())
                .unwrap_or_else(|_| input.to_string())
        }
    }
}
```

## Text Injection

```rust
pub async fn inject_text(text: &str) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        // Use CGEventCreateKeyboardEvent for typing
        use core_graphics::event::{CGEvent, CGEventTapLocation};
        
        // Copy to clipboard
        let mut ctx = ClipboardContext::new()?;
        ctx.set_contents(text.to_string())?;
        
        // Paste via Cmd+V
        simulate_paste()?;
    }
    
    Ok(())
}
```

## Icon Resolution

```rust
pub fn resolve_scriptlet_icon(
    frontmatter_icon: Option<&str>,
    tool_type: Option<&ScriptletType>,
) -> &'static str {
    if let Some(icon) = frontmatter_icon {
        return match icon {
            "api" => "globe",
            "code" => "code",
            "terminal" => "terminal",
            "text" => "text-cursor-input",
            _ => icon,
        };
    }
    
    tool_type
        .map(|t| t.default_icon())
        .unwrap_or("file-text")
}
```

## Loading Scriptlets

```rust
pub fn load_all_scriptlets() -> Vec<Scriptlet> {
    let snippets_dir = dirs::home_dir()
        .unwrap()
        .join(".scriptkit/snippets");
    
    let mut all_scriptlets = Vec::new();
    
    if let Ok(entries) = fs::read_dir(&snippets_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "md").unwrap_or(false) {
                if let Ok(content) = fs::read_to_string(&path) {
                    let scriptlets = parse_scriptlets(&content, &path);
                    all_scriptlets.extend(scriptlets);
                }
            }
        }
    }
    
    all_scriptlets
}
```

## Best Practices

1. **Use descriptive headings** - `## Send Email to {{recipient}}`
2. **Group related scriptlets** - One bundle per topic
3. **Add frontmatter** - Name and icon help with discovery
4. **Use meaningful variable names** - `{{api_key}}` not `{{x}}`
5. **Validate variables exist** - Check before execution

## Summary

| Type | Block | Use Case |
|------|-------|----------|
| Template | `\`\`\`template` | Text expansion |
| Shell | `\`\`\`shell` | Command execution |
| TypeScript | `\`\`\`typescript` | Complex logic |
| AppleScript | `\`\`\`applescript` | macOS automation |
| Transform | `\`\`\`transform` | Text manipulation |
