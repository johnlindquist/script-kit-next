import path from "node:path";
import type { HitlChoiceJob } from "./types";

type Args = {
  file?: string;
  url: string;
  token: string;
};

function usage(): never {
  console.error("Usage: bun scripts/hitl-choice/push-job.ts <job.json> [--url http://127.0.0.1:8877] [--token <token>]");
  process.exit(2);
}

function parseArgs(argv: string[]): Args {
  const args: Args = {
    url: process.env.HITL_URL ?? "http://127.0.0.1:8877",
    token: process.env.HITL_TOKEN ?? "",
  };

  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === "--url") args.url = argv[++index] ?? usage();
    else if (arg === "--token") args.token = argv[++index] ?? usage();
    else if (arg === "--help" || arg === "-h") usage();
    else if (!args.file) args.file = arg;
    else usage();
  }

  if (!args.file) usage();
  if (!args.token) {
    console.error("Missing token. Pass --token or set HITL_TOKEN.");
    process.exit(2);
  }

  return args;
}

function validateJob(job: HitlChoiceJob): void {
  if (!job.jobId || !job.title || !Array.isArray(job.options) || job.options.length === 0) {
    throw new Error("job must include jobId, title, and at least one option");
  }
  const ids = new Set<string>();
  for (const option of job.options) {
    if (!option.id || !option.title || !option.description) {
      throw new Error("each option must include id, title, and description");
    }
    if (ids.has(option.id)) throw new Error(`duplicate option id: ${option.id}`);
    ids.add(option.id);
  }
}

const args = parseArgs(Bun.argv.slice(2));
const jobPath = path.resolve(args.file);
const job = await Bun.file(jobPath).json() as HitlChoiceJob;
validateJob(job);

const endpoint = new URL("/api/jobs", args.url);
endpoint.searchParams.set("token", args.token);

const response = await fetch(endpoint, {
  method: "POST",
  headers: {
    "content-type": "application/json",
    "x-hitl-token": args.token,
  },
  body: JSON.stringify(job),
});

const payload = await response.json();
if (!response.ok) {
  console.error(JSON.stringify(payload, null, 2));
  process.exit(1);
}

console.log(JSON.stringify({
  jobId: payload.jobId,
  title: payload.title,
  optionCount: payload.options.length,
  url: `${args.url}/?token=${encodeURIComponent(args.token)}`,
}, null, 2));
