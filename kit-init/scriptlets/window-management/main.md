---
name: Window Management
description: Window tiling, positioning, and display management for macOS
author: Script Kit
icon: layout-grid
---

# Window Management

Organize and position windows with keyboard shortcuts. All commands operate on the frontmost window of the app you were using before Script Kit appeared.

## Tile {{app}} Left Half

<!--
description: Tile {{app}} to left half of screen
-->

```ts
const win = await getFrontmostWindow();
if (win) {
  await tileWindow(win.windowId, 'left');
}
```

## Tile {{app}} Right Half

<!--
description: Tile {{app}} to right half of screen
-->

```ts
const win = await getFrontmostWindow();
if (win) {
  await tileWindow(win.windowId, 'right');
}
```

## Tile {{app}} Top Half

<!--
description: Tile {{app}} to top half of screen
-->

```ts
const win = await getFrontmostWindow();
if (win) {
  await tileWindow(win.windowId, 'top');
}
```

## Tile {{app}} Bottom Half

<!--
description: Tile {{app}} to bottom half of screen
-->

```ts
const win = await getFrontmostWindow();
if (win) {
  await tileWindow(win.windowId, 'bottom');
}
```

## Tile {{app}} Top Left

<!--
description: Tile {{app}} to top-left quadrant
-->

```ts
const win = await getFrontmostWindow();
if (win) {
  await tileWindow(win.windowId, 'top-left');
}
```

## Tile {{app}} Top Right

<!--
description: Tile {{app}} to top-right quadrant
-->

```ts
const win = await getFrontmostWindow();
if (win) {
  await tileWindow(win.windowId, 'top-right');
}
```

## Tile {{app}} Bottom Left

<!--
description: Tile {{app}} to bottom-left quadrant
-->

```ts
const win = await getFrontmostWindow();
if (win) {
  await tileWindow(win.windowId, 'bottom-left');
}
```

## Tile {{app}} Bottom Right

<!--
description: Tile {{app}} to bottom-right quadrant
-->

```ts
const win = await getFrontmostWindow();
if (win) {
  await tileWindow(win.windowId, 'bottom-right');
}
```

## Tile {{app}} Left Third

<!--
description: Tile {{app}} to left third of screen
-->

```ts
const win = await getFrontmostWindow();
if (win) {
  await tileWindow(win.windowId, 'left-third');
}
```

## Tile {{app}} Center Third

<!--
description: Tile {{app}} to center third of screen
-->

```ts
const win = await getFrontmostWindow();
if (win) {
  await tileWindow(win.windowId, 'center-third');
}
```

## Tile {{app}} Right Third

<!--
description: Tile {{app}} to right third of screen
-->

```ts
const win = await getFrontmostWindow();
if (win) {
  await tileWindow(win.windowId, 'right-third');
}
```

## Tile {{app}} Top Third

<!--
description: Tile {{app}} to top third of screen
-->

```ts
const win = await getFrontmostWindow();
if (win) {
  await tileWindow(win.windowId, 'top-third');
}
```

## Tile {{app}} Middle Third

<!--
description: Tile {{app}} to middle third of screen
-->

```ts
const win = await getFrontmostWindow();
if (win) {
  await tileWindow(win.windowId, 'middle-third');
}
```

## Tile {{app}} Bottom Third

<!--
description: Tile {{app}} to bottom third of screen
-->

```ts
const win = await getFrontmostWindow();
if (win) {
  await tileWindow(win.windowId, 'bottom-third');
}
```

## Tile {{app}} First Two Thirds

<!--
description: Tile {{app}} to first two-thirds of screen (left)
-->

```ts
const win = await getFrontmostWindow();
if (win) {
  await tileWindow(win.windowId, 'first-two-thirds');
}
```

## Tile {{app}} Last Two Thirds

<!--
description: Tile {{app}} to last two-thirds of screen (right)
-->

```ts
const win = await getFrontmostWindow();
if (win) {
  await tileWindow(win.windowId, 'last-two-thirds');
}
```

## Tile {{app}} Top Two Thirds

<!--
description: Tile {{app}} to top two-thirds of screen
-->

```ts
const win = await getFrontmostWindow();
if (win) {
  await tileWindow(win.windowId, 'top-two-thirds');
}
```

## Tile {{app}} Bottom Two Thirds

<!--
description: Tile {{app}} to bottom two-thirds of screen
-->

```ts
const win = await getFrontmostWindow();
if (win) {
  await tileWindow(win.windowId, 'bottom-two-thirds');
}
```

## Center {{app}}

<!--
description: Center {{app}} on screen (60% size)
-->

```ts
const win = await getFrontmostWindow();
if (win) {
  await tileWindow(win.windowId, 'center');
}
```

## Almost Maximize {{app}}

<!--
description: Expand {{app}} to 90% of screen with margins
-->

```ts
const win = await getFrontmostWindow();
if (win) {
  await tileWindow(win.windowId, 'almost-maximize');
}
```

## Maximize {{app}}

<!--
description: Maximize {{app}} to fill screen
-->

```ts
const win = await getFrontmostWindow();
if (win) {
  await tileWindow(win.windowId, 'maximize');
}
```

## Move {{app}} to Next Display

<!--
description: Move {{app}} to next display/monitor
-->

```ts
const win = await getFrontmostWindow();
if (win) {
  await moveToNextDisplay(win.windowId);
}
```

## Move {{app}} to Previous Display

<!--
description: Move {{app}} to previous display/monitor
-->

```ts
const win = await getFrontmostWindow();
if (win) {
  await moveToPreviousDisplay(win.windowId);
}
```

## Minimize {{app}}

<!--
description: Minimize {{app}}
-->

```ts
const win = await getFrontmostWindow();
if (win) {
  await minimizeWindow(win.windowId);
}
```

## Close {{app}} Window

<!--
description: Close the frontmost {{app}} window
-->

```ts
const win = await getFrontmostWindow();
if (win) {
  await closeWindow(win.windowId);
}
```

## Show Display Info

<!--
description: Show information about connected displays
-->

```ts
const displays = await getDisplays();
await div(
  `<pre style="padding: 16px; font-size: 12px;">${JSON.stringify(displays, null, 2)}</pre>`
);
```

## Show {{app}} Window Info

<!--
description: Show information about {{app}}'s frontmost window
-->

```ts
const win = await getFrontmostWindow();
if (win) {
  await div(
    `<pre style="padding: 16px; font-size: 12px;">${JSON.stringify(win, null, 2)}</pre>`
  );
} else {
  await div('<p style="padding: 16px;">No frontmost window found</p>');
}
```
