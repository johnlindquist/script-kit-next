import { mkdir, appendFile } from "node:fs/promises";
import { join } from "node:path";

export const metadata = {
  name: "Power Syntax Duplicate Command Script",
  description: "Intentionally collides with a scriptlet command named ps-dupe",
  icon: "triangle-alert",
  alias: "ps-dupe",
  tags: ["menu-syntax", "demo", "duplicate"],
  category: "menu-syntax-demo",
  domain: {
    kind: "command-duplicate",
    localFirst: true,
  },
};

const skPath = process.env.SK_PATH || join(process.env.HOME || ".", ".scriptkit");
const dir = join(skPath, "menu-syntax", "commands");

await mkdir(dir, { recursive: true });
await appendFile(
  join(dir, "ps-dupe-script-ran.jsonl"),
  JSON.stringify({
    warning: "This should only appear if the duplicate scriptlet is not installed.",
    createdAt: new Date().toISOString(),
  }) + "\n"
);
