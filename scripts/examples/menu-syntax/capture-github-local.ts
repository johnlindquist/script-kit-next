import "@scriptkit/sdk";
import { appendFile, mkdir, readFile } from "node:fs/promises";
import { homedir } from "node:os";
import { join } from "node:path";

type GitHubPayload = {
  body?: string;
  raw?: string;
  tags?: string[];
  priority?: string;
  url?: string;
  dates?: { role?: string; iso?: string }[];
  kv?: Record<string, string | undefined>;
};

type SavedToken = {
  token?: { access_token?: string };
};

type GitHubIssueResponse = {
  number: number;
  html_url: string;
  title: string;
};

export const metadata = {
  name: "Create GitHub Issue",
  description:
    "Create a GitHub issue from ;github menu syntax using a saved GitHub device-flow token",
  icon: "github",
  alias: "gh-issue",
  tags: ["menu-syntax", "demo", "power-syntax", "github", "api"],
  category: "menu-syntax-demo",
  domain: {
    kind: "capture",
    target: "github",
    localFirst: false,
  },
  menuSyntax: [
    {
      family: "capture.v1",
      targets: ["github"],
      accepts: ["tags", "date", "priority", "url", "kv"],
      label: "Create GitHub issue",
      payloadSchema: "kit://schema/menu-syntax/payload-v1",
      defaultHandler: true,
      required: ["body"],
      optional: ["kv:repo", "kv:title", "kv:assignee", "kv:milestone", "url", "priority", "tag", "date"],
    },
  ],
};

const payloadPath = process.env.KIT_MENU_SYNTAX_PAYLOAD_PATH;
if (!payloadPath) throw new Error("KIT_MENU_SYNTAX_PAYLOAD_PATH is required");

const payload = JSON.parse(await readFile(payloadPath, "utf8")) as GitHubPayload;
const kv = payload.kv ?? {};
const skPath = process.env.SK_PATH || join(homedir(), ".scriptkit");
const token = await loadToken(skPath);
const repoAndTitle = resolveRepoAndTitle(payload, kv);
const issue = await createIssue(token, repoAndTitle.repo, {
  title: repoAndTitle.title,
  body: buildIssueBody(payload, kv),
});

const auditDir = join(skPath, "menu-syntax");
await mkdir(auditDir, { recursive: true });
await appendFile(
  join(auditDir, "github-issues.jsonl"),
  JSON.stringify({
    source: "github-api",
    repo: repoAndTitle.repo,
    issue: issue.number,
    title: issue.title,
    url: issue.html_url,
    raw: payload.raw,
    payloadPath,
    createdAt: new Date().toISOString(),
  }) + "\n",
);

const action = await arg(`Created ${repoAndTitle.repo}#${issue.number}`, [
  { name: "Open Issue", description: issue.html_url, value: "issue" },
  { name: "Open Repo", description: repoAndTitle.repo, value: "repo" },
  { name: "Done", description: issue.title, value: "done" },
]);

if (action === "issue") {
  await open(issue.html_url);
} else if (action === "repo") {
  await open(`https://github.com/${repoAndTitle.repo}`);
}

async function loadToken(root: string): Promise<string> {
  const path = join(root, "secrets", "device-auth", "github.json");
  let saved: SavedToken;
  try {
    saved = JSON.parse(await readFile(path, "utf8")) as SavedToken;
  } catch (error) {
    throw new Error(
      `No saved GitHub token at ${path}. Run the GitHub Device Login example first.`,
      { cause: error },
    );
  }

  const token = saved.token?.access_token;
  if (!token) throw new Error(`Saved GitHub token at ${path} is missing access_token`);
  return token;
}

async function createIssue(
  token: string,
  repo: string,
  input: { title: string; body: string },
): Promise<GitHubIssueResponse> {
  const response = await fetch(`https://api.github.com/repos/${repo}/issues`, {
    method: "POST",
    headers: {
      Accept: "application/vnd.github+json",
      Authorization: `Bearer ${token}`,
      "Content-Type": "application/json",
      "X-GitHub-Api-Version": "2022-11-28",
    },
    body: JSON.stringify(input),
  });
  const body = await response.json();
  if (!response.ok) {
    throw new Error(`GitHub issue create failed (${response.status}): ${JSON.stringify(body)}`);
  }
  return body as GitHubIssueResponse;
}

function buildIssueBody(
  payload: GitHubPayload,
  kv: Record<string, string | undefined>,
): string {
  const due =
    payload.dates?.find((date) => date.role === "due")?.iso ??
    payload.dates?.[0]?.iso ??
    undefined;
  return [
    kv.body || kv.description || "",
    payload.url ? `URL: ${payload.url}` : "",
    payload.priority ? `Priority: ${payload.priority}` : "",
    due ? `Due: ${due}` : "",
    payload.tags?.length ? `Tags: ${payload.tags.map((tag) => `#${tag}`).join(" ")}` : "",
    kv.assignee ? `Assignee hint: ${kv.assignee}` : "",
    kv.milestone ? `Milestone hint: ${kv.milestone}` : "",
    payload.raw ? `Captured from: ${payload.raw}` : "",
  ]
    .filter(Boolean)
    .join("\n\n");
}

function resolveRepoAndTitle(
  payload: { body?: string; url?: string },
  kv: Record<string, string | undefined>,
): { repo: string; title: string } {
  const body = String(payload.body || "").trim();
  const explicitRepo = normalizeRepo(kv.repo);
  if (explicitRepo) {
    return {
      repo: explicitRepo,
      title: (kv.title || body || "Untitled issue").trim(),
    };
  }

  const urlRepo = repoFromUrl(payload.url);
  if (urlRepo) {
    return {
      repo: urlRepo,
      title: (kv.title || body || "Untitled issue").trim(),
    };
  }

  const firstTokenMatch = body.match(/^([A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+)(?:\s+(.+))?$/);
  const inferredRepo = normalizeRepo(firstTokenMatch?.[1]);
  if (inferredRepo) {
    return {
      repo: inferredRepo,
      title: (kv.title || firstTokenMatch?.[2] || "Untitled issue").trim(),
    };
  }

  throw new Error(
    "GitHub issue creation requires a repo. Use `repo=owner/name` or start the body with `owner/name`, e.g. `;github johnlindquist/kit Fix launcher focus`.",
  );
}

function normalizeRepo(value?: string): string | null {
  const repo = String(value || "").trim();
  return /^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/.test(repo) ? repo : null;
}

function repoFromUrl(value?: string): string | null {
  if (!value) return null;
  try {
    const url = new URL(value);
    if (url.hostname !== "github.com") return null;
    const [, owner, repo] = url.pathname.split("/");
    return normalizeRepo(owner && repo ? `${owner}/${repo}` : undefined);
  } catch {
    return null;
  }
}
