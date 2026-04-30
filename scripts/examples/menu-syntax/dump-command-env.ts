import { mkdir, appendFile } from "node:fs/promises";
import { join } from "node:path";

export const metadata = {
  name: "Power Syntax Command Env Dump",
  description: "Dump ! command fields, tags, and argv to a local JSONL file",
  icon: "terminal",
  alias: "ps-env",
  tags: ["menu-syntax", "demo", "command"],
  category: "menu-syntax-demo",
  domain: {
    kind: "command",
    localFirst: true,
  },
};

function parseJsonEnv<T>(name: string, fallback: T): T {
  try {
    return JSON.parse(process.env[name] || "") as T;
  } catch {
    return fallback;
  }
}

const skPath = process.env.SK_PATH || join(process.env.HOME || ".", ".scriptkit");
const dir = join(skPath, "menu-syntax", "commands");

await mkdir(dir, { recursive: true });

const fieldPairs = parseJsonEnv<[string, string][]>(
  "KIT_MENU_SYNTAX_COMMAND_FIELDS",
  []
);
const tags = parseJsonEnv<string[]>("KIT_MENU_SYNTAX_COMMAND_TAGS", []);

await appendFile(
  join(dir, "command-env.jsonl"),
  JSON.stringify({
    head: process.env.KIT_MENU_SYNTAX_COMMAND_HEAD ?? null,
    family: process.env.KIT_MENU_SYNTAX_FAMILY ?? null,
    fields: Object.fromEntries(fieldPairs),
    fieldPairs,
    tags,
    argv: process.argv.slice(2),
    createdAt: new Date().toISOString(),
  }) + "\n"
);
