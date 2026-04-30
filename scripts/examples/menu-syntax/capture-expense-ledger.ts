import { mkdir, appendFile, readFile } from "node:fs/promises";
import { join } from "node:path";

export const metadata = {
  name: "Capture Expense Ledger",
  description:
    "Capture a local expense record from ;expense / expense: menu syntax",
  icon: "receipt",
  alias: "expense-ledger",
  tags: ["menu-syntax", "demo", "finance"],
  category: "menu-syntax-demo",
  domain: {
    kind: "ledger",
    target: "expense",
    localFirst: true,
  },
  menuSyntax: [
    {
      family: "capture.v1",
      targets: ["expense"],
      accepts: ["tags", "date", "kv"],
      label: "Capture local expense",
      payloadSchema: "kit://schema/menu-syntax/payload-v1",
      defaultHandler: true,
      // Dynamic schema (Run 11 Pass 21): the launcher parses these tokens
      // into a CaptureFieldSchema via dynamic_capture_schema_from_spec.
      // Vocabulary: "body" | "url" | "priority" | "duration" | "tag" |
      // "date" | "date:start|end|due|at|any" | "kv:KEY".
      required: ["body", "kv:amount"],
      optional: ["tag", "date", "kv:vendor"],
      forbidden: ["url"],
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

const amountText =
  kv.amount ??
  String(payload.body ?? "").match(/(?:^|\s)\$?(\d+(?:\.\d{1,2})?)/)?.[1] ??
  null;
const amount = amountText
  ? Number(String(amountText).replace(/[^\d.]/g, ""))
  : null;
const date = payload.dates?.[0]?.iso ?? new Date().toISOString();

await appendFile(
  join(dir, "expenses.jsonl"),
  JSON.stringify({
    source: "expense-ledger",
    merchant: kv.vendor || kv.merchant || payload.body || "Unknown merchant",
    amount,
    currency: kv.currency || "USD",
    project: kv.project ?? null,
    reimbursable: kv.reimbursable === "true" || kv.reimbursable === "yes",
    tags: payload.tags ?? [],
    date,
    raw: payload.raw,
    payloadPath,
    createdAt: new Date().toISOString(),
  }) + "\n"
);
