import "@scriptkit/sdk";

export const metadata = {
  name: "Path Picker",
  description: "Pick a file and perform an action on it",
};

const file = await path("Choose a file");

const action = await arg("What to do with it?", [
  { name: "Copy Path", description: "Copy the full path to clipboard", value: "copy-path" },
  { name: "Read Contents", description: "Display the file contents", value: "read" },
  { name: "Open", description: "Open in default application", value: "open" },
  { name: "File Info", description: "Show file size and type", value: "info" },
]);

switch (action) {
  case "copy-path":
    await copy(file);
    await notify(`Path copied: ${file}`);
    break;

  case "read": {
    const content = await Bun.file(file).text();
    await div(`<pre class="p-4 text-sm overflow-auto h-full whitespace-pre-wrap">${content}</pre>`);
    break;
  }

  case "open":
    await open(file);
    break;

  case "info": {
    const bunFile = Bun.file(file);
    const size = bunFile.size;
    const type = bunFile.type;
    const units = ["B", "KB", "MB", "GB"];
    let s = size;
    let u = 0;
    while (s >= 1024 && u < units.length - 1) {
      s /= 1024;
      u++;
    }
    await div(`
      <div class="p-8 space-y-2">
        <h2 class="text-xl font-bold text-yellow-400">${file.split("/").pop()}</h2>
        <p class="text-gray-300">Path: ${file}</p>
        <p class="text-gray-300">Size: ${s.toFixed(1)} ${units[u]}</p>
        <p class="text-gray-300">Type: ${type}</p>
      </div>
    `);
    break;
  }
}
