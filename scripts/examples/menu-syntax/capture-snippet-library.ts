import { mkdir, appendFile, readFile } from "node:fs/promises";
import { join } from "node:path";

export const metadata = {
  name: "Capture Snippet Library",
  description:
    "Capture a local snippet from ;snippet / snippet: menu syntax",
  icon: "code",
  alias: "snip",
  tags: ["menu-syntax", "demo", "snippets"],
  category: "menu-syntax-demo",
  domain: {
    kind: "library",
    target: "snippet",
    localFirst: true,
  },
  menuSyntax: [
    {
      family: "capture.v1",
      targets: ["snippet"],
      accepts: ["tags", "url", "kv"],
      label: "Capture local snippet",
      payloadSchema: "kit://schema/menu-syntax/payload-v1",
      defaultHandler: true,
    },
  ],
};

const payloadPath = process.env.KIT_MENU_SYNTAX_PAYLOAD_PATH;
if (!payloadPath) throw new Error("KIT_MENU_SYNTAX_PAYLOAD_PATH is required");

const payload = JSON.parse(await readFile(payloadPath, "utf8"));
const kv = payload.kv ?? {};
const skPath = process.env.SK_PATH || join(process.env.HOME || ".", ".scriptkit");
const dir = join(skPath, "menu-syntax");

await mkdir(dir, { recursive: true });

const lang =
  String(kv.lang || "text")
    .replace(/[^a-z0-9#+_-]/gi, "")
    .slice(0, 32) || "text";
const title = kv.title || payload.body?.slice(0, 60) || "Untitled snippet";
const safeBody = String(payload.body || "").replaceAll("```", "`\u200b``");
const tags = payload.tags ?? [];
const record = {
  source: "snippet-library",
  title,
  lang,
  body: payload.body,
  url: payload.url ?? kv.source ?? null,
  tags,
  raw: payload.raw,
  payloadPath,
  createdAt: new Date().toISOString(),
};

await appendFile(join(dir, "snippets.jsonl"), JSON.stringify(record) + "\n");
await appendFile(
  join(dir, "snippets.md"),
  [
    "",
    `## ${new Date().toISOString()} - ${title}`,
    tags.length
      ? `Tags: ${tags.map((tag: string) => `#${tag}`).join(" ")}`
      : "Tags: none",
    payload.url ? `Source: ${payload.url}` : "",
    "",
    "```" + lang,
    safeBody,
    "```",
    "",
  ]
    .filter(Boolean)
    .join("\n")
);
