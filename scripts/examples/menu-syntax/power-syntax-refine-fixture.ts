import { mkdir, appendFile, readFile } from "node:fs/promises";
import { join } from "node:path";

export const metadata = {
  name: "Power Syntax Refine Fixture",
  description:
    "Metadata-rich fixture for : tags, alias, shortcut, has, and meta path demos",
  icon: "filter",
  alias: "ps-refine",
  shortcut: "cmd+shift+;",
  tags: ["menu-syntax", "demo", "power-syntax", "script-kit"],
  category: "menu-syntax-demo",
  domain: {
    kind: "fixture",
    team: "launcher",
    localFirst: true,
  },
  menuSyntax: [
    {
      family: "capture.v1",
      targets: ["fixture"],
      accepts: ["tags", "kv"],
      label: "Capture fixture row",
      payloadSchema: "kit://schema/menu-syntax/payload-v1",
      defaultHandler: true,
    },
  ],
};

const skPath = process.env.SK_PATH || join(process.env.HOME || ".", ".scriptkit");
const dir = join(skPath, "menu-syntax");

await mkdir(dir, { recursive: true });

if (process.env.KIT_MENU_SYNTAX_PAYLOAD_PATH) {
  const payloadPath = process.env.KIT_MENU_SYNTAX_PAYLOAD_PATH;
  const payload = JSON.parse(await readFile(payloadPath, "utf8"));
  await appendFile(
    join(dir, "fixture-captures.jsonl"),
    JSON.stringify({
      mode: "capture",
      target: payload.target,
      body: payload.body,
      tags: payload.tags ?? [],
      kv: payload.kv ?? {},
      raw: payload.raw,
      payloadPath,
      createdAt: new Date().toISOString(),
    }) + "\n"
  );
} else {
  await appendFile(
    join(dir, "refine-fixture-runs.jsonl"),
    JSON.stringify({
      mode: process.env.KIT_MENU_SYNTAX_COMMAND_HEAD ? "command" : "direct",
      commandHead: process.env.KIT_MENU_SYNTAX_COMMAND_HEAD ?? null,
      createdAt: new Date().toISOString(),
    }) + "\n"
  );
}
