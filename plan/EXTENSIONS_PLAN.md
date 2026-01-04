# Extensions System Plan

> Rename "Scriptlets" → "Extensions" with Raycast-compatible manifest fields

## Overview

This plan covers:
1. **Terminology refactor**: Rename all "scriptlet" references to "extension"
2. **Manifest alignment**: Add Raycast-compatible fields for easy porting
3. **Example extensions**: CleanShot and Chrome extensions as proof-of-concept

## Current State

### Existing Implementation

| Component | Location | Status |
|-----------|----------|--------|
| Scriptlet parsing | `src/scriptlets.rs` | ✅ Complete |
| Codefence metadata | `src/scriptlet_metadata.rs` | ✅ Complete |
| Bundle frontmatter | `src/scriptlets.rs:66-78` | ✅ Basic (name, description, author, icon) |
| Typed metadata | `src/metadata_parser.rs` | ✅ Complete |
| Schema parsing | `src/schema_parser.rs` | ✅ Complete |
| Cache layer | `src/scriptlet_cache.rs` | ✅ Complete |
| Test fixtures | `tests/fixtures/test-scriptlets.md` | ✅ Complete |

### Current Frontmatter Fields (BundleFrontmatter)

```rust
pub struct BundleFrontmatter {
    pub name: Option<String>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub icon: Option<String>,
    pub extra: HashMap<String, serde_yaml::Value>,  // Catch-all
}
```

### Current TypedMetadata Fields

```rust
pub struct TypedMetadata {
    pub name: Option<String>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub enter: Option<String>,
    pub alias: Option<String>,
    pub icon: Option<String>,
    pub shortcut: Option<String>,
    pub tags: Vec<String>,
    pub hidden: bool,
    pub placeholder: Option<String>,
    pub cron: Option<String>,
    pub schedule: Option<String>,
    pub watch: Vec<String>,
    pub background: bool,
    pub system: bool,
    pub extra: HashMap<String, serde_json::Value>,
}
```

---

## Phase 1: Raycast Manifest Alignment

### Required Fields for Raycast Parity

Based on [Raycast manifest documentation](https://developers.raycast.com/information/manifest):

#### Extension-Level (Bundle Frontmatter)

| Raycast Field | Type | Required | Script Kit Mapping | Status |
|---------------|------|----------|-------------------|--------|
| `name` | string | Yes | `name` | ✅ Have |
| `title` | string | Yes | `title` (display name) | ❌ Need |
| `description` | string | Yes | `description` | ✅ Have |
| `icon` | string | Yes | `icon` | ✅ Have |
| `author` | string | Yes | `author` | ✅ Have |
| `license` | string | Yes | `license` | ❌ Need |
| `categories` | string[] | Yes | `categories` | ❌ Need |
| `keywords` | string[] | No | `keywords` | ❌ Need |
| `contributors` | string[] | No | `contributors` | ❌ Need |

#### Command-Level (Individual Extensions in Bundle)

| Raycast Field | Type | Required | Script Kit Mapping | Status |
|---------------|------|----------|-------------------|--------|
| `name` | string | Yes | `command` (slug) | ✅ Have |
| `title` | string | Yes | `name` (display) | ✅ Have |
| `description` | string | Yes | `description` | ✅ Have (in metadata) |
| `mode` | enum | Yes | Inferred from tool | ✅ Implicit |
| `icon` | string | No | `icon` | ✅ Have |
| `subtitle` | string | No | `subtitle` | ❌ Need |
| `keywords` | string[] | No | `keywords` | ❌ Need |
| `interval` | string | No | `cron`/`schedule` | ✅ Have |

#### Preferences (Extension Settings)

| Raycast Field | Type | Required | Script Kit Mapping | Status |
|---------------|------|----------|-------------------|--------|
| `preferences` | array | No | `preferences` | ❌ Need |
| `arguments` | array | No | `inputs` (from `{{var}}`) | ✅ Have |

### New BundleFrontmatter Structure

```rust
/// Extension bundle metadata (YAML frontmatter)
/// Compatible with Raycast manifest for easy porting
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionManifest {
    // === Required for publishing ===
    /// Unique URL-safe identifier (e.g., "cleanshot")
    pub name: String,
    /// Display name shown in UI (e.g., "CleanShot X")
    pub title: String,
    /// Full description
    pub description: String,
    /// 512x512 PNG icon path or icon name
    pub icon: String,
    /// Author's handle/username
    pub author: String,
    /// License identifier (e.g., "MIT")
    #[serde(default = "default_license")]
    pub license: String,
    /// Categories for discovery
    #[serde(default)]
    pub categories: Vec<String>,
    
    // === Optional ===
    /// Additional search keywords
    #[serde(default)]
    pub keywords: Vec<String>,
    /// Active contributors
    #[serde(default)]
    pub contributors: Vec<String>,
    /// Extension version
    pub version: Option<String>,
    /// Repository URL
    pub repository: Option<String>,
    /// Homepage URL  
    pub homepage: Option<String>,
    /// Extension-wide preferences
    #[serde(default)]
    pub preferences: Vec<Preference>,
    
    // === Script Kit specific ===
    /// Required permissions (clipboard, accessibility, etc.)
    #[serde(default)]
    pub permissions: Vec<String>,
    /// Minimum Script Kit version
    pub min_version: Option<String>,
    
    /// Catch-all for unknown fields
    #[serde(flatten)]
    pub extra: HashMap<String, serde_yaml::Value>,
}

fn default_license() -> String {
    "MIT".to_string()
}

/// User preference definition
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Preference {
    pub name: String,
    pub title: String,
    pub description: String,
    #[serde(rename = "type")]
    pub pref_type: PreferenceType,
    pub required: bool,
    #[serde(default)]
    pub default: Option<serde_json::Value>,
    pub placeholder: Option<String>,
    /// For dropdown type
    #[serde(default)]
    pub data: Vec<PreferenceOption>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PreferenceType {
    Textfield,
    Password,
    Checkbox,
    Dropdown,
    AppPicker,
    File,
    Directory,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PreferenceOption {
    pub title: String,
    pub value: String,
}
```

### Valid Categories

Match Raycast's categories for compatibility:

```rust
pub const VALID_CATEGORIES: &[&str] = &[
    "Applications",
    "Communication", 
    "Data",
    "Design Tools",
    "Developer Tools",
    "Documentation",
    "Finance",
    "Fun",
    "Media",
    "News",
    "Productivity",
    "Security",
    "System",
    "Web",
    "Other",
];
```

---

## Phase 2: Terminology Refactor

### File Renames

| Current | New |
|---------|-----|
| `src/scriptlets.rs` | `src/extensions.rs` |
| `src/scriptlet_metadata.rs` | `src/extension_metadata.rs` |
| `src/scriptlet_cache.rs` | `src/extension_cache.rs` |
| `src/scriptlet_tests.rs` | `src/extension_tests.rs` |
| `tests/fixtures/test-scriptlets.md` | `tests/fixtures/test-extensions.md` |
| `tests/smoke/test-scriptlet-*.ts` | `tests/smoke/test-extension-*.ts` |
| `docs/SCRIPTLET_TOOL_MAP.md` | `docs/EXTENSION_TOOL_MAP.md` |

### Directory Renames

| Current | New |
|---------|-----|
| `~/.sk/kit/snippets/` | `~/.sk/kit/extensions/` |
| (future) `~/.sk/kit/scriptlets/` | `~/.sk/kit/extensions/` |

### Struct/Type Renames

| Current | New |
|---------|-----|
| `Scriptlet` | `Extension` |
| `ScriptletMetadata` | `ExtensionMetadata` |
| `ScriptletMatch` | `ExtensionMatch` |
| `BundleFrontmatter` | `ExtensionManifest` |
| `ScriptletValidationError` | `ExtensionValidationError` |
| `ScriptletParseResult` | `ExtensionParseResult` |

### Function Renames

| Current | New |
|---------|-----|
| `load_scriptlets()` | `load_extensions()` |
| `parse_scriptlet_section()` | `parse_extension_section()` |
| `fuzzy_search_scriptlets()` | `fuzzy_search_extensions()` |
| `read_scriptlets_from_file()` | `read_extensions_from_file()` |
| `parse_markdown_as_scriptlets()` | `parse_markdown_as_extensions()` |
| `resolve_scriptlet_icon()` | `resolve_extension_icon()` |

### Backward Compatibility

Keep type aliases for transition period:

```rust
// Deprecated aliases for backward compatibility
#[deprecated(since = "2.0.0", note = "Use Extension instead")]
pub type Scriptlet = Extension;

#[deprecated(since = "2.0.0", note = "Use ExtensionMetadata instead")]  
pub type ScriptletMetadata = ExtensionMetadata;
```

---

## Phase 3: Example Extensions

### Location

```
~/.sk/kit/examples/extensions/
├── cleanshot.md        # CleanShot X integration
├── chrome.md           # Chrome browser integration
└── README.md           # Examples documentation
```

### 3.1 CleanShot Extension

**Purpose**: Integrate with CleanShot X app for screenshots and recordings.

**Manifest**:

```yaml
---
name: cleanshot
title: CleanShot X
description: Capture screenshots, recordings, and annotations with CleanShot X
icon: camera
author: scriptkit
license: MIT
categories:
  - Productivity
  - Media
keywords:
  - screenshot
  - screen recording
  - annotation
  - capture
permissions:
  - accessibility
preferences:
  - name: outputFolder
    title: Output Folder
    description: Where to save captures
    type: directory
    required: false
    default: ~/Desktop
---
```

**Commands**:

| Command | Raycast Equivalent | Implementation |
|---------|-------------------|----------------|
| Capture Area | ✅ | `open cleanshot://capture-area` |
| Capture Fullscreen | ✅ | `open cleanshot://capture-fullscreen` |
| Capture Window | ✅ | `open cleanshot://capture-window` |
| Capture Previous Area | ✅ | `open cleanshot://capture-previous-area` |
| Record Screen | ✅ | `open cleanshot://record-screen` |
| Record GIF | ✅ | `open cleanshot://record-gif` |
| Self Timer | ✅ | `open cleanshot://self-timer` |
| Scrolling Capture | ✅ | `open cleanshot://scrolling-capture` |
| Capture Text (OCR) | ✅ | `open cleanshot://capture-text` |
| Toggle Desktop Icons | ✅ | `open cleanshot://toggle-desktop-icons` |
| Open from Clipboard | ✅ | `open cleanshot://open-from-clipboard` |
| All-In-One | ✅ | `open cleanshot://all-in-one` |
| Pin Screenshot | ✅ | `open cleanshot://pin-screenshot` |
| Restore Recently Closed | ✅ | `open cleanshot://restore-recently-closed` |
| Open History | ✅ | `open cleanshot://open-history` |

**Example Extension File**:

```markdown
---
name: cleanshot
title: CleanShot X
description: Capture screenshots, recordings, and annotations with CleanShot X
icon: camera
author: scriptkit
license: MIT
categories:
  - Productivity
  - Media
keywords:
  - screenshot
  - capture
  - recording
---

# CleanShot X

## Capture Area
<!-- Description: Capture a selected area of the screen -->
<!-- Shortcut: cmd shift 4 -->

```open
cleanshot://capture-area
```

## Capture Fullscreen
<!-- Description: Capture the entire screen -->

```open
cleanshot://capture-fullscreen
```

## Capture Window
<!-- Description: Capture a specific window -->

```open
cleanshot://capture-window
```

## Record Screen
<!-- Description: Start a screen recording -->
<!-- Shortcut: cmd shift 5 -->

```open
cleanshot://record-screen
```

## Record GIF
<!-- Description: Record screen as animated GIF -->

```open
cleanshot://record-gif
```

## Scrolling Capture
<!-- Description: Capture scrolling content -->

```open
cleanshot://scrolling-capture
```

## Capture Text (OCR)
<!-- Description: Capture and extract text from screen -->

```open
cleanshot://capture-text
```

## Toggle Desktop Icons
<!-- Description: Show or hide desktop icons -->

```open
cleanshot://toggle-desktop-icons
```

## Open from Clipboard
<!-- Description: Open image from clipboard in editor -->

```open
cleanshot://open-from-clipboard
```

## All-In-One
<!-- Description: Open all capture options overlay -->

```open
cleanshot://all-in-one
```

## Pin Screenshot
<!-- Description: Pin a screenshot to screen -->

```open
cleanshot://pin-screenshot
```

## Open History
<!-- Description: Open CleanShot capture history -->

```open
cleanshot://open-history
```

## Restore Recently Closed
<!-- Description: Restore last closed capture -->

```open
cleanshot://restore-recently-closed
```

## Self Timer
<!-- Description: Capture with countdown timer -->

```open
cleanshot://self-timer
```

## Capture Previous Area
<!-- Description: Capture the same area as last time -->

```open
cleanshot://capture-previous-area
```
```

### 3.2 Chrome Extension

**Purpose**: Search and interact with Google Chrome browser.

**Manifest**:

```yaml
---
name: chrome
title: Google Chrome
description: Search bookmarks, history, tabs, and control Chrome
icon: chrome
author: scriptkit
license: MIT
categories:
  - Applications
  - Productivity
  - Web
keywords:
  - browser
  - bookmarks
  - history
  - tabs
permissions:
  - accessibility
---
```

**Commands**:

| Command | Raycast Equivalent | Implementation |
|---------|-------------------|----------------|
| Search Bookmarks | ✅ | SQLite query + fuzzy search |
| Search History | ✅ | SQLite query + fuzzy search |
| Search Tabs | ✅ | AppleScript/JXA |
| Open New Tab | ✅ | AppleScript |
| New Incognito Window | ✅ | AppleScript |
| Search Downloads | ✅ | SQLite query |

**Technical Details**:

Chrome stores data in SQLite databases:
- Bookmarks: `~/Library/Application Support/Google/Chrome/Default/Bookmarks` (JSON)
- History: `~/Library/Application Support/Google/Chrome/Default/History` (SQLite)
- Downloads: Same History DB, `downloads` table

**Example Extension File**:

```markdown
---
name: chrome
title: Google Chrome
description: Search bookmarks, history, tabs, and control Chrome
icon: chrome
author: scriptkit
license: MIT
categories:
  - Applications
  - Web
keywords:
  - browser
  - bookmarks
  - history
---

# Google Chrome

## Search Bookmarks
<!-- Description: Search and open Chrome bookmarks -->
<!-- Shortcut: cmd shift b -->

```ts
import { readFileSync, existsSync } from 'fs';
import { homedir } from 'os';
import { join } from 'path';

const bookmarksPath = join(
  homedir(),
  'Library/Application Support/Google/Chrome/Default/Bookmarks'
);

interface BookmarkNode {
  name: string;
  url?: string;
  children?: BookmarkNode[];
  type: 'url' | 'folder';
}

interface BookmarksFile {
  roots: {
    bookmark_bar: BookmarkNode;
    other: BookmarkNode;
    synced: BookmarkNode;
  };
}

function flattenBookmarks(node: BookmarkNode, path: string[] = []): { name: string; url: string; path: string }[] {
  const results: { name: string; url: string; path: string }[] = [];
  
  if (node.type === 'url' && node.url) {
    results.push({
      name: node.name,
      url: node.url,
      path: path.join(' > ')
    });
  }
  
  if (node.children) {
    for (const child of node.children) {
      results.push(...flattenBookmarks(child, [...path, node.name]));
    }
  }
  
  return results;
}

if (!existsSync(bookmarksPath)) {
  await div(`<div class="p-4 text-red-500">Chrome bookmarks not found. Is Chrome installed?</div>`);
  process.exit(1);
}

const bookmarksData: BookmarksFile = JSON.parse(readFileSync(bookmarksPath, 'utf-8'));
const allBookmarks = [
  ...flattenBookmarks(bookmarksData.roots.bookmark_bar),
  ...flattenBookmarks(bookmarksData.roots.other),
  ...flattenBookmarks(bookmarksData.roots.synced),
];

const selected = await arg('Search bookmarks', allBookmarks.map(b => ({
  name: b.name,
  description: b.path,
  value: b.url,
  preview: `<div class="p-4">
    <div class="font-bold">${b.name}</div>
    <div class="text-sm text-gray-500">${b.path}</div>
    <div class="text-xs text-blue-500 mt-2 break-all">${b.url}</div>
  </div>`
})));

await open(selected);
```

## Search History
<!-- Description: Search Chrome browsing history -->

```ts
import { homedir } from 'os';
import { join } from 'path';
import { execSync, spawn } from 'child_process';
import { copyFileSync, unlinkSync, existsSync } from 'fs';
import Database from 'bun:sqlite';

const historyPath = join(
  homedir(),
  'Library/Application Support/Google/Chrome/Default/History'
);

if (!existsSync(historyPath)) {
  await div(`<div class="p-4 text-red-500">Chrome history not found. Is Chrome installed?</div>`);
  process.exit(1);
}

// Chrome locks the database, so we need to copy it
const tempPath = '/tmp/chrome-history-copy.db';
copyFileSync(historyPath, tempPath);

const db = new Database(tempPath, { readonly: true });

const rows = db.query(`
  SELECT title, url, last_visit_time, visit_count
  FROM urls
  ORDER BY last_visit_time DESC
  LIMIT 1000
`).all() as { title: string; url: string; last_visit_time: number; visit_count: number }[];

db.close();
unlinkSync(tempPath);

// Chrome timestamps are microseconds since 1601-01-01
const chromeEpoch = 11644473600000000n;

const history = rows.map(row => {
  const timestamp = Number((BigInt(row.last_visit_time) - chromeEpoch) / 1000n);
  const date = new Date(timestamp);
  return {
    title: row.title || row.url,
    url: row.url,
    date: date.toLocaleDateString(),
    time: date.toLocaleTimeString(),
    visits: row.visit_count
  };
});

const selected = await arg('Search history', history.map(h => ({
  name: h.title,
  description: `${h.date} ${h.time} • ${h.visits} visits`,
  value: h.url
})));

await open(selected);
```

## Search Open Tabs
<!-- Description: Search and switch to open Chrome tabs -->

```ts
const script = `
tell application "Google Chrome"
  set tabList to {}
  set windowIndex to 1
  repeat with w in windows
    set tabIndex to 1
    repeat with t in tabs of w
      set end of tabList to {title of t, URL of t, windowIndex, tabIndex}
      set tabIndex to tabIndex + 1
    end repeat
    set windowIndex to windowIndex + 1
  end repeat
  return tabList
end tell
`;

const result = await applescript(script);

// Parse AppleScript list result
const tabs = (result as string[][]).map(([title, url, windowIdx, tabIdx]) => ({
  title,
  url,
  windowIndex: parseInt(windowIdx),
  tabIndex: parseInt(tabIdx)
}));

const selected = await arg('Search tabs', tabs.map(t => ({
  name: t.title,
  description: t.url,
  value: t
})));

// Switch to selected tab
await applescript(`
tell application "Google Chrome"
  set active tab index of window ${selected.windowIndex} to ${selected.tabIndex}
  set index of window ${selected.windowIndex} to 1
  activate
end tell
`);
```

## New Tab
<!-- Description: Open a new Chrome tab -->

```applescript
tell application "Google Chrome"
  activate
  tell front window
    make new tab
  end tell
end tell
```

## New Incognito Window
<!-- Description: Open a new Chrome incognito window -->

```applescript
tell application "Google Chrome"
  activate
  make new window with properties {mode:"incognito"}
end tell
```

## Close Current Tab
<!-- Description: Close the current Chrome tab -->

```applescript
tell application "Google Chrome"
  tell front window
    close active tab
  end tell
end tell
```
```

---

## Phase 4: Implementation Tasks

### 4.1 Manifest Updates (Priority: High)

- [ ] Add new fields to `BundleFrontmatter` / `ExtensionManifest`
- [ ] Add `Preference` and `PreferenceType` structs
- [ ] Add `VALID_CATEGORIES` constant
- [ ] Update frontmatter parser to handle new fields
- [ ] Add validation for required fields
- [ ] Update tests

### 4.2 Terminology Refactor (Priority: High)

- [ ] Rename files (see File Renames table)
- [ ] Update `lib.rs` module declarations
- [ ] Rename structs and types
- [ ] Rename functions
- [ ] Update all imports across codebase
- [ ] Add deprecated type aliases
- [ ] Update documentation
- [ ] Update test files
- [ ] Run full test suite

### 4.3 Directory Migration (Priority: Medium)

- [ ] Add support for `~/.sk/kit/extensions/` path
- [ ] Add migration logic for `snippets/` → `extensions/`
- [ ] Update file watchers
- [ ] Update cache paths
- [ ] Update documentation

### 4.4 Example Extensions (Priority: Medium)

- [ ] Create `~/.sk/kit/examples/extensions/` directory
- [ ] Implement `cleanshot.md`
- [ ] Implement `chrome.md`
- [ ] Add `README.md` with usage instructions
- [ ] Test all commands

### 4.5 Preference System (Priority: Low - Future)

- [ ] Design preference storage (`~/.sk/kit/preferences/<extension-name>.json`)
- [ ] Implement `getPreference(name)` SDK function
- [ ] Add preferences UI in settings
- [ ] Add preference validation

---

## Testing Checklist

### Unit Tests

- [ ] `ExtensionManifest` parsing with all fields
- [ ] Category validation
- [ ] Preference parsing
- [ ] Backward compatibility with old frontmatter

### Integration Tests

- [ ] Load extensions from `~/.sk/kit/extensions/`
- [ ] CleanShot commands execute correctly
- [ ] Chrome commands work (bookmarks, history, tabs)
- [ ] Migration from `snippets/` works

### Smoke Tests

- [ ] `test-extension-basic.ts`
- [ ] `test-extension-typescript.ts`
- [ ] `test-extension-bundles.ts`

---

## Migration Guide

### For Users

```bash
# Extensions will be auto-migrated from ~/.sk/kit/snippets/ to ~/.sk/kit/extensions/
# No action required - both paths work during transition
```

### For Extension Authors

1. **Rename frontmatter**: Add new required fields

```yaml
# Before
---
name: My Bundle
description: Some tools
---

# After  
---
name: my-bundle
title: My Bundle
description: Some tools
icon: wrench
author: yourname
license: MIT
categories:
  - Productivity
---
```

2. **File location**: Move from `snippets/` to `extensions/`

3. **No code changes required**: Extension code remains the same

---

## Open Questions

1. **Preference storage format**: JSON file per extension vs single SQLite DB?
2. **Icon format**: Support both icon names and file paths?
3. **Version checking**: How to handle `min_version` requirement?
4. **Extension registry**: Future plans for community extensions?

---

## References

- [Raycast Manifest Documentation](https://developers.raycast.com/information/manifest)
- [Raycast CleanShot Extension](https://www.raycast.com/Aayush9029/cleanshotx)
- [Raycast Chrome Extension](https://www.raycast.com/nicholasess/google-chrome)
- [CleanShot X URL Schemes](https://cleanshot.com/docs/api)
- [Chrome User Data Directory](https://chromium.googlesource.com/chromium/src/+/master/docs/user_data_dir.md)
