# PopClip and Text Selection Utilities for Quick Notes - Research

This document covers how PopClip and similar text utility apps capture selected text and enable quick note-taking workflows on macOS.

---

## 1. PopClip Overview

### What is PopClip?

PopClip is a macOS utility that appears when you select text with the mouse. It displays a contextual popup bar with actions you can perform on the selected text, similar to the iOS text selection interface but for macOS.

**Key Features:**
- Appears automatically when text is selected via mouse
- Shows context-aware action buttons based on the selected text
- Extensible through a plugin/extension system
- Works system-wide across most macOS applications

**How PopClip Works:**
1. PopClip monitors text selection events system-wide using macOS Accessibility APIs
2. When text is selected via mouse, it reads the selected text
3. It displays a floating popup near the selection with relevant actions
4. Actions can copy, search, transform, or send the text to other apps

### PopClip Extension System

PopClip extensions are defined using a simple format that specifies:
- **Name**: Display name for the action
- **Icon**: Symbol or image for the button
- **Action Types**:
  - `copy` - Copy text to clipboard
  - `paste` - Paste transformed text
  - `url` - Open a URL with the text
  - `service` - Call a macOS Service
  - `script` - Run a shell script, AppleScript, or JavaScript
  - `keypress` - Simulate keypresses
  - `app` - Open an application

### Notes-Related PopClip Extensions

Several extensions enable sending selected text to notes apps:

| Extension | Target App | Action |
|-----------|------------|--------|
| Obsidian | Obsidian | Append/prepend text to daily note or specific note |
| Bear | Bear | Create new note or append to existing |
| Drafts | Drafts | Create new draft with selected text |
| Evernote | Evernote | Create new note or append to notebook |
| DEVONthink | DEVONthink | Capture text with metadata |
| OmniFocus | OmniFocus | Create task from selected text |
| Things | Things | Create to-do item |
| Apple Notes | Apple Notes | Create new note (via URL scheme or AppleScript) |
| Notion | Notion | Append to page via API |

**Example: Obsidian Extension**
```yaml
name: Obsidian
icon: symbol:doc.text
requirements: [text]
actions:
  - title: Send to Daily Note
    script: |
      const text = popclip.input.text
      const url = `obsidian://new?vault=MyVault&file=DailyNotes&content=${encodeURIComponent(text)}&append=true`
      popclip.openUrl(url)
```

---

## 2. Similar Text Selection Utilities

### Alfred (with Universal Actions)

**Universal Actions** in Alfred 4+ allow acting on selected text:
- Trigger with configurable hotkey (default: Cmd+Opt+\)
- Works on selected text, files, or URLs
- Supports custom workflows for processing text

**Key Differences from PopClip:**
- Requires manual hotkey trigger (not automatic on selection)
- More powerful workflow engine for complex automation
- Works with files and other content types, not just text

### Raycast

Raycast offers similar functionality through:
- **Quicklinks**: Transform and search selected text
- **Extensions**: Custom actions on clipboard/selection
- **Snippets**: Text expansion with dynamic content

**Note-taking integrations:**
- Raycast Notes (built-in floating notes)
- Apple Notes extension
- Notion, Obsidian, and Bear extensions

### Keyboard Maestro

A powerful automation tool that can:
- Trigger macros on selected text via hotkey
- Read selected text using system clipboard or Accessibility APIs
- Send text to any application via AppleScript, shell scripts, or UI automation

### Hammerspoon

Open-source automation tool for macOS:
- Lua scripting for custom behaviors
- Can monitor clipboard and simulate selection reading
- Extensible hotkey system for text actions

### macOS Services Menu

Built-in macOS feature:
- Apps register Services that accept text input
- Access via right-click context menu or Keyboard > Shortcuts > Services
- Limited UI discoverability but native integration

**Creating a Service for Notes:**
1. Use Automator to create a "Service" (Quick Action)
2. Set input to "text"
3. Add action: "New Note" or script to process text
4. Service appears in app context menus

### Quick Note (macOS Monterey+)

Apple's built-in Quick Note feature:
- Access via hot corner, keyboard shortcut, or Share menu
- "Add to Quick Note" action available in Share sheet
- Captures selected text with app context/links
- Syncs across devices via iCloud

---

## 3. Technical Implementation Approaches

### Method 1: Accessibility API (AXUIElement)

The most reliable method for reading selected text system-wide.

**Key APIs:**
- `AXUIElementCopyAttributeValue` with `kAXSelectedTextAttribute`
- `AXUIElementCopyAttributeValue` with `kAXFocusedUIElementAttribute`

**Rust Implementation (via accessibility crate or raw bindings):**
```rust
use accessibility::{AXUIElement, AXAttribute};
use core_foundation::string::CFString;

fn get_selected_text() -> Option<String> {
    let system_wide = AXUIElement::system_wide();
    
    // Get focused element
    let focused: AXUIElement = system_wide
        .attribute(&AXAttribute::new(&CFString::new("AXFocusedUIElement")))?
        .downcast()?;
    
    // Get selected text
    let selected: CFString = focused
        .attribute(&AXAttribute::new(&CFString::new("AXSelectedText")))?
        .downcast()?;
    
    Some(selected.to_string())
}
```

**Requirements:**
- App must have Accessibility permission (System Preferences > Privacy > Accessibility)
- Some apps may not expose selected text via Accessibility

### Method 2: Clipboard-Based Capture

Fallback method when Accessibility API is unavailable.

**Approach:**
1. Save current clipboard contents
2. Simulate Cmd+C keypress
3. Read clipboard
4. Restore original clipboard contents

**Rust Implementation:**
```rust
use arboard::Clipboard;
use enigo::{Enigo, Key, KeyboardControllable};
use std::thread::sleep;
use std::time::Duration;

fn capture_via_clipboard() -> Option<String> {
    let mut clipboard = Clipboard::new().ok()?;
    
    // Save current clipboard
    let original = clipboard.get_text().ok();
    
    // Simulate Cmd+C
    let mut enigo = Enigo::new();
    enigo.key_down(Key::Meta);
    enigo.key_click(Key::Layout('c'));
    enigo.key_up(Key::Meta);
    
    // Wait for clipboard update
    sleep(Duration::from_millis(50));
    
    // Get selected text
    let selected = clipboard.get_text().ok();
    
    // Restore original clipboard
    if let Some(orig) = original {
        let _ = clipboard.set_text(orig);
    }
    
    selected
}
```

**Pros:**
- Works in most applications
- No Accessibility permission needed for clipboard read

**Cons:**
- Destructive to clipboard contents (requires save/restore)
- May trigger app-specific copy behaviors
- Slightly slower due to keypress simulation

### Method 3: macOS Services

Register as a Service provider to receive selected text.

**Info.plist Configuration:**
```xml
<key>NSServices</key>
<array>
  <dict>
    <key>NSMenuItem</key>
    <dict>
      <key>default</key>
      <string>Send to Notes</string>
    </dict>
    <key>NSMessage</key>
    <string>handleSelection</string>
    <key>NSPortName</key>
    <string>MyApp</string>
    <key>NSSendTypes</key>
    <array>
      <string>public.utf8-plain-text</string>
    </array>
  </dict>
</array>
```

**Handling the Service (Swift example):**
```swift
@objc func handleSelection(_ pboard: NSPasteboard, userData: String, error: AutoreleasingUnsafeMutablePointer<NSString>) {
    guard let text = pboard.string(forType: .string) else { return }
    // Process selected text
    createNote(with: text)
}
```

### Method 4: Share Extension

Create a Share Extension that accepts text.

**Capabilities:**
- Appears in Share menu for selected text
- Receives text via `NSExtensionItem`
- Can process and save to notes database

### Method 5: URL Schemes

Many notes apps support URL schemes for receiving text:

| App | URL Scheme Example |
|-----|-------------------|
| Apple Notes | `mobilenotes://` (limited) |
| Obsidian | `obsidian://new?content=TEXT` |
| Bear | `bear://x-callback-url/create?text=TEXT` |
| Drafts | `drafts://x-callback-url/create?text=TEXT` |
| Notion | Via API only |
| DEVONthink | `x-devonthink://createText?text=TEXT` |

---

## 4. macOS APIs and Permissions

### Accessibility API

**Key Headers:**
- `<ApplicationServices/ApplicationServices.h>`
- `AXUIElement.h`, `AXValue.h`, `AXError.h`

**Important Attributes:**
| Attribute | Description |
|-----------|-------------|
| `kAXFocusedUIElementAttribute` | Currently focused UI element |
| `kAXSelectedTextAttribute` | Currently selected text |
| `kAXSelectedTextRangeAttribute` | Range of selected text |
| `kAXValueAttribute` | Full text value of element |

**Permission Requirements:**
- App must be added to System Preferences > Security & Privacy > Privacy > Accessibility
- Can check via `AXIsProcessTrusted()` or `AXIsProcessTrustedWithOptions()`

**Requesting Permission:**
```rust
use core_foundation::dictionary::CFDictionary;
use core_foundation::string::CFString;
use core_foundation::boolean::CFBoolean;

fn request_accessibility() -> bool {
    let key = CFString::new("AXTrustedCheckOptionPrompt");
    let value = CFBoolean::true_value();
    let options = CFDictionary::from_CFType_pairs(&[(key, value)]);
    
    unsafe {
        AXIsProcessTrustedWithOptions(options.as_concrete_TypeRef())
    }
}
```

### NSPasteboard (Clipboard)

**General Pasteboard:**
```rust
// Via objc crate or cocoa bindings
let pasteboard = NSPasteboard::generalPasteboard();
let text = pasteboard.stringForType(NSPasteboardTypeString);
```

**Pasteboard Types:**
- `public.utf8-plain-text` - Plain text
- `public.rtf` - Rich text
- `public.html` - HTML content

### NSServices

**Input Types:**
- `NSSendTypes` - Types app can receive
- `NSReturnTypes` - Types app can provide

**Registering Services:**
Services are registered via Info.plist and automatically appear in:
- Application menu > Services
- Right-click context menu > Services

---

## 5. Comparison Matrix

| Feature | PopClip | Alfred | Raycast | Services | Quick Note |
|---------|---------|--------|---------|----------|------------|
| Auto-show on selection | Yes | No | No | No | No |
| Hotkey trigger | Optional | Yes | Yes | Yes | Yes |
| Custom extensions | Yes | Yes (workflows) | Yes | Yes (Automator) | No |
| System-wide | Yes | Yes | Yes | Yes | Yes |
| Accessibility required | Yes | Varies | Varies | No | No |
| Free | No ($15) | Freemium | Free | Yes | Yes |

---

## 6. Recommendations for Script Kit

### Primary Approach: Accessibility API

1. Request Accessibility permission on first launch
2. Use `kAXSelectedTextAttribute` to read selected text
3. Fallback to clipboard method if Accessibility fails

### Trigger Mechanisms

1. **Global Hotkey**: User presses hotkey to capture selection
2. **Menu Bar**: Click menu bar icon to capture and process
3. **Services**: Register as a Service for context menu access

### Integration with Notes Window

1. Capture selected text via hotkey (e.g., Cmd+Shift+N)
2. Open floating notes window with captured text
3. Allow editing before saving
4. Support multiple note destinations (file, clipboard, external app)

### Sample Workflow

```
User Flow:
1. User selects text in any app
2. Presses Cmd+Shift+N (Script Kit hotkey)
3. Script Kit reads selection via Accessibility API
4. Opens Notes window with text pre-filled
5. User can edit, add tags, choose destination
6. Press Enter to save to configured location
```

---

## 7. References

### Official Documentation
- Apple Accessibility Programming Guide: https://developer.apple.com/documentation/accessibility
- NSPasteboard: https://developer.apple.com/documentation/appkit/nspasteboard
- Services Implementation Guide: https://developer.apple.com/library/archive/documentation/Cocoa/Conceptual/SysServices/

### PopClip Resources
- PopClip Website: https://www.popclip.app
- PopClip Extensions: https://www.popclip.app/extensions/
- PopClip Developer Docs: https://github.com/pilotmoon/PopClip-Extensions

### Community Resources
- Hammerspoon Docs: https://www.hammerspoon.org/docs/
- Alfred Workflows: https://www.alfredapp.com/workflows/
- Raycast Extensions: https://www.raycast.com/store

---

*Document created: 2025-01-31*
*Research scope: PopClip, text selection utilities, macOS APIs for capturing selected text*
