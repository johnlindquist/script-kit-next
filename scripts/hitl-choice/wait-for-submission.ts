import path from "node:path";
import type { HitlChoiceSubmission } from "./types";

type Args = {
  jobId?: string;
  outDir: string;
  url?: string;
  token?: string;
  sinceLine: number;
  pollMs: number;
  timeoutMs: number;
};

function usage(): never {
  console.error([
    "Usage: bun scripts/hitl-choice/wait-for-submission.ts --job <jobId> [options]",
    "",
    "Options:",
    "  --out-dir <dir>       Runtime output dir. Default: HITL_OUT_DIR or .hitl-choice",
    "  --url <url>           Wait through the HTTP API instead of local JSONL",
    "  --token <token>       API token for --url. Default: HITL_TOKEN",
    "  --since-line <n>      Ignore existing JSONL lines up to n. Default: 0",
    "  --poll-ms <n>         Poll interval. Default: 1000",
    "  --timeout-ms <n>      Timeout. Default: 0, meaning wait forever",
  ].join("\n"));
  process.exit(2);
}

function parseArgs(argv: string[]): Args {
  const args: Args = {
    outDir: process.env.HITL_OUT_DIR ?? ".hitl-choice",
    url: process.env.HITL_URL,
    token: process.env.HITL_TOKEN,
    sinceLine: 0,
    pollMs: 1000,
    timeoutMs: 0,
  };

  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === "--job") args.jobId = argv[++index] ?? usage();
    else if (arg === "--out-dir") args.outDir = argv[++index] ?? usage();
    else if (arg === "--url") args.url = argv[++index] ?? usage();
    else if (arg === "--token") args.token = argv[++index] ?? usage();
    else if (arg === "--since-line") args.sinceLine = Number(argv[++index] ?? usage());
    else if (arg === "--poll-ms") args.pollMs = Number(argv[++index] ?? usage());
    else if (arg === "--timeout-ms") args.timeoutMs = Number(argv[++index] ?? usage());
    else if (arg === "--help" || arg === "-h") usage();
    else usage();
  }

  if (!args.jobId) usage();
  if (args.url && !args.token) {
    console.error("Missing token for --url. Pass --token or set HITL_TOKEN.");
    process.exit(2);
  }
  if (!Number.isFinite(args.sinceLine) || args.sinceLine < 0) usage();
  if (!Number.isFinite(args.pollMs) || args.pollMs < 100) usage();
  if (!Number.isFinite(args.timeoutMs) || args.timeoutMs < 0) usage();
  return args;
}

async function readSubmissions(filePath: string): Promise<HitlChoiceSubmission[]> {
  const text = await Bun.file(filePath).text().catch(() => "");
  return text
    .split("\n")
    .filter(Boolean)
    .map((line) => JSON.parse(line) as HitlChoiceSubmission);
}

const args = parseArgs(Bun.argv.slice(2));
const jsonlPath = path.join(path.resolve(args.outDir), "submissions.jsonl");
const startedAt = Date.now();

async function waitThroughHttp(): Promise<void> {
  if (!args.url || !args.token || !args.jobId) return;
  const endpoint = new URL(`/api/jobs/${encodeURIComponent(args.jobId)}/submissions/wait`, args.url);
  endpoint.searchParams.set("token", args.token);
  endpoint.searchParams.set("sinceLine", String(args.sinceLine));
  endpoint.searchParams.set("timeoutMs", String(args.timeoutMs || 60_000));

  const response = await fetch(endpoint, {
    headers: { "x-hitl-token": args.token },
  });
  const payload = await response.json();
  if (!response.ok) {
    console.error(JSON.stringify(payload, null, 2));
    process.exit(1);
  }
  if (payload.submission) {
    console.log(JSON.stringify(payload.submission, null, 2));
    process.exit(0);
  }
  console.error(JSON.stringify(payload, null, 2));
  process.exit(1);
}

if (args.url) await waitThroughHttp();

while (true) {
  const submissions = await readSubmissions(jsonlPath);
  const match = submissions
    .slice(args.sinceLine)
    .find((submission) => submission.jobId === args.jobId);

  if (match) {
    console.log(JSON.stringify(match, null, 2));
    process.exit(0);
  }

  if (args.timeoutMs > 0 && Date.now() - startedAt >= args.timeoutMs) {
    console.error(JSON.stringify({
      error: "timeout",
      jobId: args.jobId,
      jsonlPath,
      sinceLine: args.sinceLine,
      elapsedMs: Date.now() - startedAt,
    }, null, 2));
    process.exit(1);
  }

  await Bun.sleep(args.pollMs);
}
