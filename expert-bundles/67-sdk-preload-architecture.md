# SDK Preload Architecture - Expert Bundle

## Overview

Script Kit uses a preloaded SDK that provides global functions (`arg`, `div`, `editor`, etc.) to scripts. The SDK is embedded at build time and extracted at runtime.

## SDK Deployment Architecture

```
Build Time:
  scripts/kit-sdk.ts  ──(include_str!)──>  Embedded in binary
                      ──(build.rs)──────>  ~/.scriptkit/sdk/kit-sdk.ts (dev)

Runtime:
  Binary  ──(ensure_sdk_extracted)──>  ~/.scriptkit/sdk/kit-sdk.ts
                                              │
                                              ▼
  bun run --preload ~/.scriptkit/sdk/kit-sdk.ts <script.ts>
```

## SDK Embedding (build.rs)

```rust
// build.rs
use std::fs;
use std::path::PathBuf;

fn main() {
    // Copy SDK to ~/.scriptkit/sdk/ during development builds
    println!("cargo:rerun-if-changed=scripts/kit-sdk.ts");
    
    let sdk_source = PathBuf::from("scripts/kit-sdk.ts");
    if sdk_source.exists() {
        if let Some(home) = dirs::home_dir() {
            let sdk_dest = home.join(".scriptkit/sdk/kit-sdk.ts");
            
            if let Some(parent) = sdk_dest.parent() {
                let _ = fs::create_dir_all(parent);
            }
            
            let _ = fs::copy(&sdk_source, &sdk_dest);
        }
    }
}
```

## SDK Extraction (src/executor/runner.rs)

```rust
/// Embedded SDK content
const EMBEDDED_SDK: &str = include_str!("../scripts/kit-sdk.ts");

/// Ensure SDK is extracted to disk
pub fn ensure_sdk_extracted() -> Result<PathBuf> {
    let sdk_path = dirs::home_dir()
        .ok_or_else(|| anyhow!("No home directory"))?
        .join(".scriptkit/sdk/kit-sdk.ts");
    
    // Create directory
    if let Some(parent) = sdk_path.parent() {
        fs::create_dir_all(parent)?;
    }
    
    // Check if needs update
    let needs_write = if sdk_path.exists() {
        let existing = fs::read_to_string(&sdk_path)?;
        existing != EMBEDDED_SDK
    } else {
        true
    };
    
    if needs_write {
        fs::write(&sdk_path, EMBEDDED_SDK)?;
        logging::log("SDK", "Extracted embedded SDK to disk");
    }
    
    Ok(sdk_path)
}

/// Find SDK path (extract if needed)
pub fn find_sdk_path() -> Result<PathBuf> {
    ensure_sdk_extracted()
}
```

## Script Execution with Preload

```rust
pub async fn spawn_script(path: &str) -> Result<Child> {
    let sdk_path = find_sdk_path()?;
    let bun_path = find_bun()?;
    
    let mut cmd = Command::new(&bun_path);
    cmd.arg("run")
       .arg("--preload")
       .arg(&sdk_path)
       .arg(path)
       .stdin(Stdio::piped())
       .stdout(Stdio::piped())
       .stderr(Stdio::piped());
    
    // Set environment
    cmd.env("SCRIPT_KIT_PATH", path);
    
    Ok(cmd.spawn()?)
}
```

## SDK Structure (scripts/kit-sdk.ts)

```typescript
// Global type augmentation
declare global {
    // Prompt functions
    function arg<T = string>(prompt: string, choices?: Choice[]): Promise<T>;
    function div(html: string): Promise<void>;
    function editor(options?: EditorOptions): Promise<string>;
    function form(fields: FormField[]): Promise<Record<string, string>>;
    
    // Utility functions
    function captureScreenshot(): Promise<ScreenshotResult>;
    function getLayoutInfo(): Promise<LayoutInfo>;
    function notify(title: string, body: string): void;
    
    // Script metadata
    const metadata: ScriptMetadata;
}

// JSONL Protocol implementation
const send = (msg: object) => {
    process.stdout.write(JSON.stringify(msg) + '\n');
};

const receive = (): Promise<any> => {
    return new Promise((resolve) => {
        process.stdin.once('data', (data) => {
            resolve(JSON.parse(data.toString()));
        });
    });
};

// Implement global functions
(globalThis as any).arg = async <T>(
    prompt: string,
    choices?: Choice[]
): Promise<T> => {
    send({
        type: 'setPrompt',
        prompt,
        choices: choices || [],
    });
    
    const response = await receive();
    return response.value as T;
};

(globalThis as any).div = async (html: string): Promise<void> => {
    send({
        type: 'setDiv',
        html,
    });
    
    await receive(); // Wait for dismiss
};

(globalThis as any).editor = async (options?: EditorOptions): Promise<string> => {
    send({
        type: 'setEditor',
        ...options,
    });
    
    const response = await receive();
    return response.value;
};

// Keep process alive for stdin
process.stdin.resume();
(process.stdin as any).unref?.(); // Allow exit when script is done
```

## Test SDK Path

```typescript
// tests/smoke/test-arg.ts
import '../../scripts/kit-sdk';  // Direct import from repo

const result = await arg("Pick a fruit", ["Apple", "Banana"]);
console.log(result);
```

```typescript
// Production script: ~/.scriptkit/scripts/hello.ts
// SDK automatically preloaded - no import needed!

const result = await arg("Pick a fruit", ["Apple", "Banana"]);
console.log(result);
```

## tsconfig.json Mapping

```json
{
    "compilerOptions": {
        "paths": {
            "@scriptkit/sdk": ["./sdk/kit-sdk.ts"]
        }
    }
}
```

## SDK Global Types

```typescript
// Type definitions for global functions

interface Choice {
    name: string;
    value?: string;
    description?: string;
    icon?: string;
}

interface EditorOptions {
    value?: string;
    language?: string;
    lineNumbers?: boolean;
}

interface FormField {
    name: string;
    label: string;
    type?: 'text' | 'password' | 'email' | 'number' | 'textarea';
    placeholder?: string;
    required?: boolean;
}

interface ScreenshotResult {
    data: string;  // Base64 encoded PNG
    width: number;
    height: number;
}

interface LayoutInfo {
    windowWidth: number;
    windowHeight: number;
    promptType: string;
    components: LayoutComponentInfo[];
}

interface ScriptMetadata {
    name?: string;
    description?: string;
    shortcut?: string;
    author?: string;
}
```

## SDK Functions Reference

### Prompts

```typescript
// Selection prompt
const fruit = await arg("Pick a fruit", ["Apple", "Banana", "Cherry"]);

// With rich choices
const choice = await arg("Select", [
    { name: "Option A", value: "a", description: "First option" },
    { name: "Option B", value: "b", description: "Second option" },
]);

// Text input (no choices)
const name = await arg("What's your name?");

// Multi-line editor
const code = await editor({
    value: "// Start coding",
    language: "typescript",
});

// Custom HTML
await div(`<div class="p-4">Custom content</div>`);

// Form
const data = await form([
    { name: "email", label: "Email", type: "email", required: true },
    { name: "password", label: "Password", type: "password" },
]);
```

### Utilities

```typescript
// Screenshot (app window only)
const screenshot = await captureScreenshot();
writeFileSync('shot.png', Buffer.from(screenshot.data, 'base64'));

// Layout inspection
const layout = await getLayoutInfo();
console.log(layout.windowWidth, layout.windowHeight);

// Notification
notify("Script Complete", "Your task finished successfully");
```

## SDK vs Test Import

| Context | Import | Preload |
|---------|--------|---------|
| Tests | `import '../../scripts/kit-sdk'` | No |
| Production | None (auto) | `--preload ~/.scriptkit/sdk/kit-sdk.ts` |

## Best Practices

1. **Don't import SDK in production scripts** - it's preloaded
2. **Use relative import in tests** - `../../scripts/kit-sdk`
3. **Check SDK extraction on startup** - `ensure_sdk_extracted()`
4. **Keep SDK minimal** - only essential globals
5. **Version SDK with app** - embedded ensures compatibility

## Summary

- SDK embedded in binary via `include_str!`
- Extracted to `~/.scriptkit/sdk/` at runtime
- Preloaded with `bun run --preload`
- Provides global `arg`, `div`, `editor`, etc.
- Tests import directly, production uses preload
