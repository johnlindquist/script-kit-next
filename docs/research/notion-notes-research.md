# Notion Quick Capture + Quick Notes Research

## Scope
Research Notion's quick capture and quick note capabilities and summarize notable UX patterns. Sources are official Notion Help Center pages.

## Feature inventory (what Notion offers)

### 1) Global-ish entry points (desktop)
- **Command Search outside the app**: Notion's desktop app exposes Command Search via a **customizable keyboard shortcut**, the **menu bar on macOS**, or the **task bar on Windows**, letting users search/ask Notion AI without foregrounding the app. This is the closest thing to a global hotkey entry point. [1]
- **Search-first new tab**: New tabs can open directly into a **search window** (on by default) where users can **navigate to pages**, **open a new page**, or **start a Notion AI chat** in one click. [1]
- **Always-available entry**: Command Search is on by default, with toggles for **Use Command Search**, **Show Notion in Menu Bar**, and the **keyboard shortcut** in Preferences. [1]

### 2) Inside-app quick capture
- **New page shortcuts**: `cmd/ctrl + N` creates a new page (desktop). [2]
- **Search/Jump**: `cmd/ctrl + P` or `cmd/ctrl + K` opens search or jumps to a recent page. [2]
- **UI affordance**: New page can also be created via the sidebar "new page" button. [3]

### 3) Web capture
- **Web Clipper (desktop browser extension)**: Clicking the Notion button opens a small window to choose a workspace + destination page/database. You can **create a new links database** or **search for an existing destination**, then **Save page** and optionally **Open in Notion**. [4]
- **Web Clipper on mobile**: Uses the **native share sheet** on iOS/Android. Users share a page -> pick Notion -> choose destination -> save. [4]

### 4) Mobile quick access
- **Mobile widgets**: Widgets can link to any workspace/page and let users **open one page**, **see recent pages**, or **pin favorites** directly from the home screen (iOS/Android). [5]
- **New page on mobile**: Tap the "new page" button in the app's bottom bar. [3]

### 5) Quick Notes page (inbox pattern)
- Notion encourages a dedicated **Quick Notes page** as a catch-all "inbox." You can drag & drop notes into it, add structured content (bullets, checkboxes, images, code), **favorite** it for faster access, and reorganize notes over time. [6]

## UX patterns & takeaways

### Entry points & speed
- **Global surfacing**: Command Search in the menu bar/taskbar + customizable shortcut turns Notion into an "always-on" capture/search surface without focus switching. [1]
- **Search-first capture**: New tabs opening into search reduces navigation friction and makes "create new page" one step away. [1]
- **Keyboard-first**: The presence of `cmd/ctrl + N` (new page) and `cmd/ctrl + P/K` (search/jump) creates a rapid, low-latency capture loop for power users. [2]

### Destination-first capture
- **Explicit destination choice**: Web Clipper asks for the exact workspace/page/database and supports search or creating a new links database. This avoids "lost" captures by forcing immediate routing. [4]
- **Share sheet parity**: Mobile capture mirrors the desktop clipper flow using the OS share sheet, keeping the mental model consistent. [4]

### Inbox + triage pattern
- **Quick Notes page** acts as an **inbox**: capture first, organize later. Drag-and-drop + favorite = fast triage and retrieval. [6]

### Ambient access on mobile
- **Widgets** provide instant entry to frequent pages, recent pages, or favorites, reducing the need to open the full app for quick checks. [5]

## Implications for Script Kit GPUI (design cues)
- **Global entry**: A hotkey + menubar affordance should open a lightweight capture/search window.
- **Search-first UI**: Open into search with "New note" as a primary action.
- **Destination clarity**: Let users specify destination at capture time (workspace/collection/page) or have a default "Quick Notes" target.
- **Inbox page**: Provide a default "Quick Notes" page or section and an easy favorite/shortlink.
- **Mobile-style access**: Mirror "widget-like" quick access via pinned items or recent notes.

## Sources
1. Notion Help Center - Notion for desktop: https://www.notion.com/help/notion-for-desktop
2. Notion Help Center - Keyboard shortcuts: https://www.notion.com/help/keyboard-shortcuts
3. Notion Help Center - Create a page: https://www.notion.com/help/create-your-first-page
4. Notion Help Center - Web Clipper: https://www.notion.com/en-gb/help/web-clipper
5. Notion Help Center - Mobile widgets: https://www.notion.com/help/mobile-widgets
6. Notion Help Center - Build a quick notes page in Notion: https://www.notion.com/help/guides/quick-notes
