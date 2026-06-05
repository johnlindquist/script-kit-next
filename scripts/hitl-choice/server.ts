import { mkdir } from "node:fs/promises";
import { randomBytes, randomUUID } from "node:crypto";
import path from "node:path";
import type { HitlChoiceDraft, HitlChoiceJob, HitlChoiceSubmission, HitlChoiceSubmissionInput } from "./types";

const rootDir = import.meta.dir;
const publicDir = path.join(rootDir, "public");
const seedPath = path.join(rootDir, "data", "script-kit-qa-scenarios.json");
const host = process.env.HITL_HOST ?? "127.0.0.1";
const port = Number(process.env.HITL_PORT ?? "8877");
const token = process.env.HITL_TOKEN ?? randomBytes(18).toString("base64url");
const outDir = path.resolve(process.env.HITL_OUT_DIR ?? ".hitl-choice");
const submissionsDir = path.join(outDir, "submissions");
const draftsDir = path.join(outDir, "drafts");
const currentJobPath = path.join(outDir, "current-job.json");
const jsonlPath = path.join(outDir, "submissions.jsonl");
const maxBodyBytes = 1_000_000;
const maxWaitMs = 60_000;

await mkdir(submissionsDir, { recursive: true });
await mkdir(draftsDir, { recursive: true });

function json(data: unknown, status = 200): Response {
  return new Response(JSON.stringify(data, null, 2), {
    status,
    headers: {
      "content-type": "application/json; charset=utf-8",
      "cache-control": "no-store",
    },
  });
}

function notFound(): Response {
  return json({ error: "not_found" }, 404);
}

function tokenFrom(request: Request, url: URL): string {
  return request.headers.get("x-hitl-token") ?? url.searchParams.get("token") ?? "";
}

function assertAuthorized(request: Request, url: URL): Response | null {
  if (tokenFrom(request, url) === token) return null;
  return json({ error: "unauthorized" }, 401);
}

function logRequest(request: Request, url: URL, note = ""): void {
  const safeUrl = new URL(url);
  if (safeUrl.searchParams.has("token")) safeUrl.searchParams.set("token", "<redacted>");
  console.log(JSON.stringify({
    at: new Date().toISOString(),
    method: request.method,
    path: safeUrl.pathname,
    search: safeUrl.search,
    note,
  }));
}

async function readJsonFile<T>(filePath: string): Promise<T | null> {
  try {
    return await Bun.file(filePath).json();
  } catch {
    return null;
  }
}

function validateJob(value: unknown): HitlChoiceJob {
  const job = value as Partial<HitlChoiceJob>;
  if (!job || typeof job !== "object") throw new Error("job must be an object");
  if (!job.jobId || typeof job.jobId !== "string") throw new Error("jobId is required");
  if (!job.title || typeof job.title !== "string") throw new Error("title is required");
  if (!Array.isArray(job.options) || job.options.length === 0) throw new Error("options are required");

  const seen = new Set<string>();
  for (const option of job.options) {
    if (!option || typeof option !== "object") throw new Error("each option must be an object");
    if (!option.id || typeof option.id !== "string") throw new Error("each option needs an id");
    if (seen.has(option.id)) throw new Error(`duplicate option id: ${option.id}`);
    seen.add(option.id);
    if (!option.title || typeof option.title !== "string") throw new Error(`option ${option.id} needs a title`);
    if (!option.description || typeof option.description !== "string") {
      throw new Error(`option ${option.id} needs a description`);
    }
  }

  return {
    jobId: job.jobId,
    title: job.title,
    description: job.description ?? "",
    createdAt: job.createdAt ?? new Date().toISOString(),
    options: job.options,
  };
}

async function currentJob(): Promise<HitlChoiceJob> {
  const customJob = await readJsonFile<HitlChoiceJob>(currentJobPath);
  if (customJob) return validateJob(customJob);
  const seedJob = await Bun.file(seedPath).json();
  return validateJob(seedJob);
}

async function readBodyJson<T>(request: Request): Promise<T> {
  const length = Number(request.headers.get("content-length") ?? "0");
  if (length > maxBodyBytes) throw new Error("request body too large");
  return await request.json();
}

function safeFilePart(value: string): string {
  return value.replace(/[^a-zA-Z0-9_.-]/g, "_").slice(0, 120);
}

function draftPath(jobId: string, clientId: string): string {
  return path.join(draftsDir, `${safeFilePart(jobId)}__${safeFilePart(clientId)}.json`);
}

async function serveStatic(url: URL): Promise<Response> {
  const pathname = url.pathname === "/" ? "/index.html" : url.pathname;
  const fileName = pathname.replace(/^\//, "");
  const allowList = new Set(["index.html", "app.js", "styles.css"]);
  if (!allowList.has(fileName)) return notFound();

  const filePath = path.join(publicDir, fileName);
  const file = Bun.file(filePath);
  if (!(await file.exists())) return notFound();

  const type = fileName.endsWith(".html")
    ? "text/html; charset=utf-8"
    : fileName.endsWith(".js")
      ? "text/javascript; charset=utf-8"
      : "text/css; charset=utf-8";
  return new Response(file, {
    headers: {
      "content-type": type,
      "cache-control": "no-store",
    },
  });
}

async function latestSubmission(jobId: string): Promise<HitlChoiceSubmission | null> {
  const text = await Bun.file(jsonlPath).text().catch(() => "");
  const lines = text.trim().split("\n").filter(Boolean).reverse();
  for (const line of lines) {
    const submission = JSON.parse(line) as HitlChoiceSubmission;
    if (submission.jobId === jobId) return submission;
  }
  return null;
}

async function submissionStatus(jobId: string): Promise<{
  jobId: string;
  submissionCount: number;
  latestSubmission: HitlChoiceSubmission | null;
  jsonlPath: string;
}> {
  const text = await Bun.file(jsonlPath).text().catch(() => "");
  const submissions = text
    .trim()
    .split("\n")
    .filter(Boolean)
    .map((line) => JSON.parse(line) as HitlChoiceSubmission)
    .filter((submission) => submission.jobId === jobId);

  return {
    jobId,
    submissionCount: submissions.length,
    latestSubmission: submissions.at(-1) ?? null,
    jsonlPath,
  };
}

async function readDraft(jobId: string, clientId: string): Promise<HitlChoiceDraft | null> {
  return await readJsonFile<HitlChoiceDraft>(draftPath(jobId, clientId));
}

async function saveDraft(job: HitlChoiceJob, clientId: string, input: HitlChoiceSubmissionInput): Promise<HitlChoiceDraft> {
  if (input.jobId !== job.jobId) throw new Error("jobId mismatch");
  const selectedIds = Array.isArray(input.selectedOptionIds) ? input.selectedOptionIds : [];
  const selectedSet = new Set(selectedIds);
  const selectedOptions = job.options.filter((option) => selectedSet.has(option.id));
  if (selectedOptions.length !== selectedSet.size) throw new Error("draft contains unknown option ids");

  const draft: HitlChoiceDraft = {
    jobId: job.jobId,
    clientId,
    savedAt: new Date().toISOString(),
    selectedOptionIds: selectedIds,
    selectedOptions,
    optionFeedback: input.optionFeedback ?? {},
    overallFeedback: input.overallFeedback ?? "",
    client: input.client ?? {},
  };

  await Bun.write(draftPath(job.jobId, clientId), `${JSON.stringify(draft, null, 2)}\n`);
  console.log(JSON.stringify({
    at: new Date().toISOString(),
    event: "draft_saved",
    jobId: draft.jobId,
    clientId,
    selectedCount: draft.selectedOptionIds.length,
  }));
  return draft;
}

async function waitForSubmission(jobId: string, sinceLine: number, timeoutMs: number): Promise<{
  jobId: string;
  timedOut: boolean;
  submissionCount: number;
  submission: HitlChoiceSubmission | null;
}> {
  const timeout = Math.max(0, Math.min(timeoutMs, maxWaitMs));
  const startedAt = Date.now();

  while (true) {
    const status = await submissionStatus(jobId);
    const submissions = status.latestSubmission ? await readSubmissions(jobId) : [];
    const submission = submissions.slice(sinceLine).find((item) => item.jobId === jobId) ?? null;
    if (submission) {
      return {
        jobId,
        timedOut: false,
        submissionCount: status.submissionCount,
        submission,
      };
    }

    if (timeout === 0 || Date.now() - startedAt >= timeout) {
      return {
        jobId,
        timedOut: true,
        submissionCount: status.submissionCount,
        submission: null,
      };
    }

    await Bun.sleep(500);
  }
}

async function readSubmissions(jobId: string): Promise<HitlChoiceSubmission[]> {
  const text = await Bun.file(jsonlPath).text().catch(() => "");
  return text
    .trim()
    .split("\n")
    .filter(Boolean)
    .map((line) => JSON.parse(line) as HitlChoiceSubmission)
    .filter((submission) => submission.jobId === jobId);
}

async function saveSubmission(job: HitlChoiceJob, input: HitlChoiceSubmissionInput): Promise<HitlChoiceSubmission> {
  if (input.jobId !== job.jobId) throw new Error("jobId mismatch");
  const selectedIds = Array.isArray(input.selectedOptionIds) ? input.selectedOptionIds : [];
  const selectedSet = new Set(selectedIds);
  const selectedOptions = job.options.filter((option) => selectedSet.has(option.id));
  if (selectedOptions.length !== selectedSet.size) throw new Error("submission contains unknown option ids");

  const submission: HitlChoiceSubmission = {
    submissionId: randomUUID(),
    submittedAt: new Date().toISOString(),
    jobId: job.jobId,
    selectedOptionIds: selectedIds,
    selectedOptions,
    optionFeedback: input.optionFeedback ?? {},
    overallFeedback: input.overallFeedback ?? "",
    client: input.client ?? {},
  };

  const line = `${JSON.stringify(submission)}\n`;
  await Bun.write(path.join(submissionsDir, `${submission.submissionId}.json`), `${JSON.stringify(submission, null, 2)}\n`);
  await Bun.write(jsonlPath, line, { append: true });
  return submission;
}

const server = Bun.serve({
  hostname: host,
  port,
  async fetch(request) {
    const url = new URL(request.url);
    if (request.method !== "GET" || url.pathname.startsWith("/api/")) logRequest(request, url);

    if (request.method === "GET" && url.pathname === "/api/health") {
      return json({ ok: true, jobId: (await currentJob()).jobId, outDir });
    }

    if (url.pathname.startsWith("/api/")) {
      const denied = assertAuthorized(request, url);
      if (denied) return denied;
    }

    try {
      if (request.method === "GET" && url.pathname === "/api/jobs/current") {
        return json(await currentJob());
      }

      if (request.method === "POST" && url.pathname === "/api/jobs") {
        const job = validateJob(await readBodyJson(request));
        await Bun.write(currentJobPath, `${JSON.stringify(job, null, 2)}\n`);
        return json(job, 201);
      }

      const latestMatch = url.pathname.match(/^\/api\/jobs\/([^/]+)\/submissions\/latest$/);
      if (request.method === "GET" && latestMatch) {
        const jobId = decodeURIComponent(latestMatch[1]);
        return json((await latestSubmission(jobId)) ?? { jobId, submission: null });
      }

      const statusMatch = url.pathname.match(/^\/api\/jobs\/([^/]+)\/submissions\/status$/);
      if (request.method === "GET" && statusMatch) {
        return json(await submissionStatus(decodeURIComponent(statusMatch[1])));
      }

      const waitMatch = url.pathname.match(/^\/api\/jobs\/([^/]+)\/submissions\/wait$/);
      if (request.method === "GET" && waitMatch) {
        const sinceLine = Number(url.searchParams.get("sinceLine") ?? "0");
        const timeoutMs = Number(url.searchParams.get("timeoutMs") ?? "30000");
        if (!Number.isFinite(sinceLine) || sinceLine < 0) return json({ error: "invalid sinceLine" }, 400);
        if (!Number.isFinite(timeoutMs) || timeoutMs < 0) return json({ error: "invalid timeoutMs" }, 400);
        return json(await waitForSubmission(decodeURIComponent(waitMatch[1]), sinceLine, timeoutMs));
      }

      const draftMatch = url.pathname.match(/^\/api\/jobs\/([^/]+)\/draft$/);
      if (draftMatch) {
        const pathJobId = decodeURIComponent(draftMatch[1]);
        const clientId = url.searchParams.get("clientId") ?? "default";
        if (request.method === "GET") {
          return json((await readDraft(pathJobId, clientId)) ?? { jobId: pathJobId, clientId, draft: null });
        }
        if (request.method === "PUT") {
          const job = await currentJob();
          if (pathJobId !== job.jobId) return json({ error: "job_not_current", currentJobId: job.jobId }, 409);
          return json(await saveDraft(job, clientId, await readBodyJson(request)));
        }
      }

      const submitMatch = url.pathname.match(/^\/api\/jobs\/([^/]+)\/submissions$/);
      if (request.method === "POST" && submitMatch) {
        const job = await currentJob();
        const pathJobId = decodeURIComponent(submitMatch[1]);
        if (pathJobId !== job.jobId) return json({ error: "job_not_current", currentJobId: job.jobId }, 409);
        const submission = await saveSubmission(job, await readBodyJson(request));
        console.log(JSON.stringify({
          at: new Date().toISOString(),
          event: "submission_saved",
          jobId: submission.jobId,
          submissionId: submission.submissionId,
          selectedCount: submission.selectedOptionIds.length,
        }));
        return json(submission, 201);
      }

      if (request.method === "GET") return await serveStatic(url);
      return notFound();
    } catch (error) {
      console.error(JSON.stringify({
        at: new Date().toISOString(),
        event: "request_error",
        path: url.pathname,
        error: error instanceof Error ? error.message : String(error),
      }));
      return json({ error: error instanceof Error ? error.message : String(error) }, 400);
    }
  },
});

console.log(`HITL choice server: http://${server.hostname}:${server.port}/?token=${token}`);
console.log(`HITL API token: ${token}`);
console.log(`HITL output dir: ${outDir}`);

await new Promise(() => {});
