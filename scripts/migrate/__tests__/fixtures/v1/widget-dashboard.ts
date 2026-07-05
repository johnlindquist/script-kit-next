// Name: Widget Dashboard
// Description: Always-on stats widget with typing automation

const w = await widget(`<h1>Stats</h1><div id="count">0</div>`);
w.onClick(async () => {
  await keyboard.type("refreshed");
});
await registerShortcut("cmd shift 9", async () => {
  w.close();
});
