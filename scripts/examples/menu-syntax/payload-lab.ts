import {
  mkdir,
  appendFile,
  readFile,
  readdir,
  stat,
  writeFile,
} from "node:fs/promises";
import { join } from "node:path";

export const metadata = {
  name: "Power Syntax Payload Lab",
  description: "Inspect or replay the latest capture payload written by menu syntax",
  icon: "file-json",
  alias: "ps-payload",
  tags: ["menu-syntax", "demo", "payload"],
  category: "menu-syntax-demo",
  domain: {
    kind: "payload",
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

async function latestPayloadPath(payloadDir: string): Promise<string> {
  const files = (await readdir(payloadDir).catch(() => []))
    .filter((file) => file.startsWith("capture_v1-") && file.endsWith(".json"));
  const ranked = await Promise.all(
    files.map(async (file) => {
      const fullPath = join(payloadDir, file);
      const entry = await stat(fullPath);
      return { fullPath, mtimeMs: entry.mtimeMs };
    })
  );

  ranked.sort(
    (a, b) => b.mtimeMs - a.mtimeMs || a.fullPath.localeCompare(b.fullPath)
  );

  if (!ranked[0]) {
    throw new Error(`No capture_v1-*.json payloads found in ${payloadDir}`);
  }

  return ranked[0].fullPath;
}

const skPath = process.env.SK_PATH || join(process.env.HOME || ".", ".scriptkit");
const payloadDir = join(skPath, "menu-syntax", "payloads");
const outDir = join(skPath, "menu-syntax", "payload-lab");

await mkdir(outDir, { recursive: true });

const fieldPairs = parseJsonEnv<[string, string][]>(
  "KIT_MENU_SYNTAX_COMMAND_FIELDS",
  []
);
const fields = Object.fromEntries(fieldPairs);
const argv = process.argv.slice(2);
const mode = fields.mode || "inspect";
const chosenPayloadPath =
  fields.path || argv[0] || (await latestPayloadPath(payloadDir));
const payload = JSON.parse(await readFile(chosenPayloadPath, "utf8"));

if (mode === "replay") {
  await appendFile(
    join(outDir, "replayed-payloads.jsonl"),
    JSON.stringify({
      replayedAt: new Date().toISOString(),
      payloadPath: chosenPayloadPath,
      target: payload.target,
      body: payload.body,
      tags: payload.tags ?? [],
      priority: payload.priority ?? null,
      url: payload.url ?? null,
      kv: payload.kv ?? {},
      dates: payload.dates ?? [],
      handler: payload.handler ?? null,
      raw: payload.raw,
    }) + "\n"
  );
} else {
  const json = JSON.stringify(payload, null, 2).replaceAll("```", "`\u200b``");
  await writeFile(
    join(outDir, "last-payload.md"),
    [
      "# Last Menu Syntax Payload",
      "",
      `Payload path: ${chosenPayloadPath}`,
      "",
      `Target: \`${payload.target}\``,
      `Handler: \`${payload.handler?.name ?? "unknown"}\``,
      "",
      "```json",
      json,
      "```",
      "",
    ].join("\n")
  );
}
