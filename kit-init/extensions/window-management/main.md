---
name: Window Management
description: Window tiling, positioning, and display management for macOS
author: Script Kit
icon: layout-grid
---

# Window Management

Organize and position windows with keyboard shortcuts. All commands operate on the frontmost window of the app you were using before Script Kit appeared.

## Half Tiling

### Tile Left Half

<!--
description: Tile window to left half of screen
-->

```ts
const win = await getFrontmostWindow();
if (win) {
  await tileWindow(win.windowId, 'left');
}
```

### Tile Right Half

<!--
description: Tile window to right half of screen
-->

```ts
const win = await getFrontmostWindow();
if (win) {
  await tileWindow(win.windowId, 'right');
}
```

### Tile Top Half

<!--
description: Tile window to top half of screen
-->

```ts
const win = await getFrontmostWindow();
if (win) {
  await tileWindow(win.windowId, 'top');
}
```

### Tile Bottom Half

<!--
description: Tile window to bottom half of screen
-->

```ts
const win = await getFrontmostWindow();
if (win) {
  await tileWindow(win.windowId, 'bottom');
}
```

## Quadrant Tiling

### Tile Top Left

<!--
description: Tile window to top-left quadrant
-->

```ts
const win = await getFrontmostWindow();
if (win) {
  await tileWindow(win.windowId, 'top-left');
}
```

### Tile Top Right

<!--
description: Tile window to top-right quadrant
-->

```ts
const win = await getFrontmostWindow();
if (win) {
  await tileWindow(win.windowId, 'top-right');
}
```

### Tile Bottom Left

<!--
description: Tile window to bottom-left quadrant
-->

```ts
const win = await getFrontmostWindow();
if (win) {
  await tileWindow(win.windowId, 'bottom-left');
}
```

### Tile Bottom Right

<!--
description: Tile window to bottom-right quadrant
-->

```ts
const win = await getFrontmostWindow();
if (win) {
  await tileWindow(win.windowId, 'bottom-right');
}
```

## Third Tiling (Horizontal)

### Tile Left Third

<!--
description: Tile window to left third of screen
-->

```ts
const win = await getFrontmostWindow();
if (win) {
  await tileWindow(win.windowId, 'left-third');
}
```

### Tile Center Third

<!--
description: Tile window to center third of screen
-->

```ts
const win = await getFrontmostWindow();
if (win) {
  await tileWindow(win.windowId, 'center-third');
}
```

### Tile Right Third

<!--
description: Tile window to right third of screen
-->

```ts
const win = await getFrontmostWindow();
if (win) {
  await tileWindow(win.windowId, 'right-third');
}
```

## Third Tiling (Vertical)

### Tile Top Third

<!--
description: Tile window to top third of screen
-->

```ts
const win = await getFrontmostWindow();
if (win) {
  await tileWindow(win.windowId, 'top-third');
}
```

### Tile Middle Third

<!--
description: Tile window to middle third of screen
-->

```ts
const win = await getFrontmostWindow();
if (win) {
  await tileWindow(win.windowId, 'middle-third');
}
```

### Tile Bottom Third

<!--
description: Tile window to bottom third of screen
-->

```ts
const win = await getFrontmostWindow();
if (win) {
  await tileWindow(win.windowId, 'bottom-third');
}
```

## Two-Thirds Tiling

### Tile First Two Thirds

<!--
description: Tile window to first two-thirds of screen (left)
-->

```ts
const win = await getFrontmostWindow();
if (win) {
  await tileWindow(win.windowId, 'first-two-thirds');
}
```

### Tile Last Two Thirds

<!--
description: Tile window to last two-thirds of screen (right)
-->

```ts
const win = await getFrontmostWindow();
if (win) {
  await tileWindow(win.windowId, 'last-two-thirds');
}
```

### Tile Top Two Thirds

<!--
description: Tile window to top two-thirds of screen
-->

```ts
const win = await getFrontmostWindow();
if (win) {
  await tileWindow(win.windowId, 'top-two-thirds');
}
```

### Tile Bottom Two Thirds

<!--
description: Tile window to bottom two-thirds of screen
-->

```ts
const win = await getFrontmostWindow();
if (win) {
  await tileWindow(win.windowId, 'bottom-two-thirds');
}
```

## Centered Positions

### Center Window

<!--
description: Center window on screen (60% size)
-->

```ts
const win = await getFrontmostWindow();
if (win) {
  await tileWindow(win.windowId, 'center');
}
```

### Almost Maximize

<!--
description: Maximize window with small margin
-->

```ts
const win = await getFrontmostWindow();
if (win) {
  await tileWindow(win.windowId, 'almost-maximize');
}
```

### Maximize

<!--
description: Maximize window to fill screen
-->

```ts
const win = await getFrontmostWindow();
if (win) {
  await tileWindow(win.windowId, 'maximize');
}
```

## Display Management

### Move to Next Display

<!--
description: Move window to next display/monitor
-->

```ts
const win = await getFrontmostWindow();
if (win) {
  await moveToNextDisplay(win.windowId);
}
```

### Move to Previous Display

<!--
description: Move window to previous display/monitor
-->

```ts
const win = await getFrontmostWindow();
if (win) {
  await moveToPreviousDisplay(win.windowId);
}
```

## Window Actions

### Minimize Window

<!--
description: Minimize the frontmost window
-->

```ts
const win = await getFrontmostWindow();
if (win) {
  await minimizeWindow(win.windowId);
}
```

### Close Window

<!--
description: Close the frontmost window
-->

```ts
const win = await getFrontmostWindow();
if (win) {
  await closeWindow(win.windowId);
}
```

## Information

### Show Display Info

<!--
description: Show information about connected displays
-->

```ts
const displays = await getDisplays();
const displayInfo = displays.map((d, i) =>
  `Display ${i + 1}${d.isPrimary ? ' (Primary)' : ''}: ${d.bounds.width}x${d.bounds.height} at (${d.bounds.x}, ${d.bounds.y})`
).join('\n');

await div(`
<div class="p-4">
  <h2 class="text-lg font-bold mb-2">Connected Displays</h2>
  <pre class="text-sm">${displayInfo}</pre>
</div>
`);
```

### Show Window Info

<!--
description: Show information about the frontmost window
-->

```ts
const win = await getFrontmostWindow();
if (win) {
  const info = `
Window: ${win.title}
App: ${win.appName}
ID: ${win.windowId}
Position: (${win.bounds?.x}, ${win.bounds?.y})
Size: ${win.bounds?.width}x${win.bounds?.height}
  `.trim();

  await div(`
<div class="p-4">
  <h2 class="text-lg font-bold mb-2">Frontmost Window</h2>
  <pre class="text-sm">${info}</pre>
</div>
  `);
} else {
  await div(`
<div class="p-4">
  <p>No frontmost window found</p>
</div>
  `);
}
```
