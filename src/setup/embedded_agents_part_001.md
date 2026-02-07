
const repo = await arg("Search repos", async (input) => {
  if (!input) return [];
  const res = await fetch(`https://api.github.com/search/repositories?q=${input}`);
  const data = await res.json();
  return data.items?.map((r: any) => ({
    name: r.full_name,
    value: r.html_url,
    description: r.description || "No description",
  })) || [];
});

await open(repo);
```

### Example 3: JSON Formatter

```typescript
import "@scriptkit/sdk";

export const metadata = {
  name: "Format JSON",
  description: "Pretty-print JSON from clipboard",
};

const clipboard = await paste();
try {
  const formatted = JSON.stringify(JSON.parse(clipboard), null, 2);
  await copy(formatted);
  await div(`<pre class="text-green-400">${formatted}</pre>`);
} catch {
  await div(`<p class="text-red-400">Invalid JSON in clipboard</p>`);
}
```

### Example 4: System Info Widget

```typescript
import "@scriptkit/sdk";
import os from "os";

export const metadata = {
  name: "System Info",
  description: "Show system information",
};

const info = `
  <div class="p-4 space-y-2">
    <p><strong>Platform:</strong> ${os.platform()}</p>
    <p><strong>Arch:</strong> ${os.arch()}</p>
    <p><strong>CPUs:</strong> ${os.cpus().length}</p>
    <p><strong>Memory:</strong> ${Math.round(os.totalmem() / 1024 / 1024 / 1024)}GB</p>
    <p><strong>Uptime:</strong> ${Math.round(os.uptime() / 3600)} hours</p>
  </div>
`;

await div(info);
```

### Example 5: File Search

```typescript
import "@scriptkit/sdk";

export const metadata = {
  name: "Find Files",
  description: "Search for files by name",
  shortcut: "cmd shift f",
};

const query = await arg("Search for files");
const { stdout } = await $`find ~ -name "*${query}*" -type f 2>/dev/null | head -20`;

const files = stdout.trim().split("\n").filter(Boolean);

if (files.length === 0) {
  await div(`<p class="text-yellow-400">No files found</p>`);
} else {
  const selected = await arg("Select file", files.map(f => ({
    name: f.split("/").pop() || f,
    value: f,
    description: f,
  })));
  
  await open(selected);
}
```

---

## Best Practices

1. **Always use `export const metadata`** - Get type safety and IDE support
2. **Import the SDK first** - `import "@scriptkit/sdk"` at the top
3. **Use Tailwind classes** - Built-in support for styling in div()
4. **Handle errors gracefully** - Wrap async operations in try/catch
5. **Keep scripts focused** - One script, one task
6. **Use meaningful names** - Clear metadata.name and description
7. **Add shortcuts sparingly** - Only for frequently used scripts

---

## File Locations

| Path | Purpose |
|------|---------|
| `~/.scriptkit/kit/main/scripts/` | Your scripts |
| `~/.scriptkit/kit/main/extensions/` | Your extensions |
| `~/.scriptkit/kit/main/agents/` | Your AI agent definitions |
| `~/.scriptkit/kit/config.ts` | Configuration |
| `~/.scriptkit/kit/theme.json` | Theme customization |
| `~/.scriptkit/sdk/` | SDK (managed by app) |
| `~/.scriptkit/kit/AGENTS.md` | This guide (for AI agents) |
| `~/.scriptkit/kit/CLAUDE.md` | Claude-specific instructions |
