import { describe, expect, test } from "bun:test";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { verifySourceVersion } from "./lib/installed-release-distribution";
import { mkdtempSync, readdirSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import {
  ARCHIVE,
  authorizeInstalledCommand,
  STABILITY_STARTER,
  correlated,
  EXE,
  ID,
  intent,
  previousTag,
  REPO,
  sameProcess,
  selectAsset,
  TEAM,
  verifyManifest,
  verifyPublicJob,
  WORKFLOW,
} from "./lib/installed-release-contract";

const ref = "refs/heads/qa/installed-release/v0.1.22",
  target = "v0.1.22",
  commit = "a".repeat(40),
  digest = "b".repeat(64);
const env = {
  CI: "true",
  GITHUB_ACTIONS: "true",
  GITHUB_REPOSITORY: REPO,
  GITHUB_SERVER_URL: "https://github.com",
  GITHUB_API_URL: "https://api.github.com",
  GITHUB_EVENT_NAME: "push",
  GITHUB_REF: ref,
  GITHUB_REF_TYPE: "branch",
  GITHUB_SHA: commit,
  GITHUB_WORKFLOW_SHA: commit,
  GITHUB_WORKFLOW_REF: `${REPO}/${WORKFLOW}@${ref}`,
  GITHUB_JOB: "installed",
  GITHUB_RUN_ID: "432",
  GITHUB_RUN_ATTEMPT: "2",
  RUNNER_OS: "macOS",
  RUNNER_ARCH: "ARM64",
  RUNNER_NAME: "host-42",
  INSTALLED_RUNNER_ENVIRONMENT: "github-hosted",
  INSTALLED_MODE: "fresh",
  SCRIPT_KIT_NONINTERACTIVE: "0",
};
const created = "2026-01-02T00:00:00Z";
const run = {
  repository: { full_name: REPO, private: false },
  head_repository: { full_name: REPO },
  id: 432,
  run_attempt: 2,
  path: WORKFLOW,
  event: "push",
  head_sha: commit,
  head_branch: ref.slice(11),
  status: "in_progress",
  created_at: created,
};
const job = {
  name: "Installed release (fresh)",
  status: "in_progress",
  run_id: 432,
  head_sha: commit,
  labels: ["macos-15"],
  runner_name: "host-42",
  id: 111,
};
const asset = {
  id: 42,
  name: ARCHIVE,
  state: "uploaded",
  size: 100,
  digest: `sha256:${digest}`,
  browser_download_url: `https://github.com/${REPO}/releases/download/${target}/${ARCHIVE}`,
};
const release = {
  id: 55,
  draft: false,
  prerelease: false,
  tag_name: target,
  html_url: `https://github.com/${REPO}/releases/tag/${target}`,
  published_at: "2026-01-01T00:00:00Z",
  assets: [asset],
};

describe("installed-release admission and provenance (no host authority)", () => {
  test("local CLI refuses before any output creation, even with hosted-looking env", () => {
    const root = mkdtempSync(join(tmpdir(), "installed-refusal-"));
    try {
      for (const mode of ["1", "0"]) {
        const p = spawnSync(process.execPath, ["./scripts/devtools/installed-release-smoke.ts"], {
          env: {
            ...env,
            BUN_RUNTIME_TRANSPILER_CACHE_PATH: "0",
            SCRIPT_KIT_NONINTERACTIVE: mode,
            RUNNER_TEMP: root,
            HOME: root,
          },
          timeout: 5000,
          encoding: "utf8",
        });
        expect(p.status).toBe(1);
        expect(JSON.parse(p.stderr).status).toBe("blocked");
        expect(JSON.parse(p.stderr).code).toBe(
          mode === "1" ? "noninteractive_refusal" : "not_dedicated_host",
        );
        expect(readdirSync(root)).toEqual([]);
      }
    } finally {
      rmSync(root, { recursive: true });
    }
  });
  test("exact QA ref, mode, run, assigned runner and attempt required", () => {
    expect(intent(env).tag).toBe(target);
    expect(verifyPublicJob(run, { jobs: [job] }, env).jobId).toBe(111);
    for (
      const bad of [
        { GITHUB_REPOSITORY: "fork/repo" },
        { GITHUB_SHA: "b".repeat(40) },
        { GITHUB_EVENT_NAME: "pull_request_target" },
        { GITHUB_REF: "refs/tags/v0.1.22" },
        { INSTALLED_MODE: "other" },
        { INSTALLED_RUNNER_ENVIRONMENT: "self-hosted" },
        { SCRIPT_KIT_ALLOW_NATIVE_INPUT: "1" },
        { SK_PATH: "/tmp/fixture" },
      ]
    ) expect(() => intent({ ...env, ...bad })).toThrow();
    for (
      const bad of [{ labels: ["self-hosted", "macos-15"] }, { runner_name: "different" }, {
        head_sha: "f".repeat(40),
      }, { status: "completed" }]
    ) {
      expect(() => verifyPublicJob(run, { jobs: [{ ...job, ...bad }] }, env)).toThrow();
    }
    expect(() => verifyPublicJob({ ...run, run_attempt: 3 }, { jobs: [job] }, env)).toThrow();
  });
  test("commit-addressed Cargo and bundle versions must both match the public tag", () => {
    const bytes = Buffer.from(
      `[package]\nname = "script-kit-gpui"\nversion = "0.1.22"\n[package.metadata.bundle.bin.script-kit-gpui]\nidentifier = "${ID}"\nversion = "0.1.22"\n`,
    );
    const file = {
      type: "file",
      path: "Cargo.toml",
      encoding: "base64",
      content: bytes.toString("base64"),
      size: bytes.length,
      sha: createHash("sha1").update(`blob ${bytes.length}\0`).update(bytes).digest("hex"),
    };
    expect(verifySourceVersion(file, target)).toBe(file.sha);
    expect(() => verifySourceVersion(file, "v0.1.23")).toThrow();
    expect(() => verifySourceVersion({ ...file, sha: commit }, target)).toThrow();
  });
  test("update request is explicit, unambiguous, forward-only", () => {
    expect(previousTag(target, "", "ordinary message")).toBeNull();
    expect(previousTag(target, "", "message\n\nInstalled-Release-Update-From: v0.1.21")).toBe("v0.1.21");
    expect(previousTag(target, "v0.1.9", "")).toBe("v0.1.9");
    for (const tag of [target, "v0.1.23", "v1.0.0", "old", "v0.1.02", "../../other"]) {
      expect(() => previousTag(target, tag, "")).toThrow();
    }
    expect(() => previousTag(target, "v0.1.9", "Installed-Release-Update-From: v0.1.8")).toThrow();
  });
  test("must be a published canonical release predating run with exact asset digest, source and manifest", () => {
    const a = selectAsset(release, target, ARCHIVE, created);
    const m = {
      schema_version: 3,
      source_sha: commit,
      tag: target,
      version: "0.1.22",
      bundle: {
        identifier: ID,
        executable: { path: EXE, sha256: digest },
        sidecar: { path: "Contents/MacOS/goose", sha256: digest },
      },
      distribution_security: { team_identifier: TEAM },
      verification: { status: "pass" },
      artifacts: [{ name: ARCHIVE, platform: "macos", sha256: digest, size_bytes: 100 }],
    };
    expect(verifyManifest(m, target, commit, a).executableSha256).toBe(digest);
    for (
      const bad of [
        { draft: true },
        { tag_name: "v0.1.21" },
        { published_at: "2026-01-03T00:00:00Z" },
        { assets: [asset, asset] },
        { assets: [{ ...asset, digest: null }] },
        { assets: [{ ...asset, browser_download_url: "https://untrusted.example/zip" }] },
      ]
    ) {
      expect(() => selectAsset({ ...release, ...bad }, target, ARCHIVE, created)).toThrow();
    }
    for (
      const bad of [{ source_sha: "f".repeat(40) }, { version: "0.1.21" }, {
        distribution_security: { team_identifier: "OTHER" },
      }, { artifacts: [{ ...m.artifacts[0], sha256: "0".repeat(64) }] }]
    ) expect(() => verifyManifest({ ...m, ...bad }, target, commit, a)).toThrow();
  });
});

test("exact correlated response and cleanup identity; late, refused or recycled targets do not match", () => {
  const row = { type: "stateResult", requestId: "r1", protocolVersion: 2 };
  expect(correlated(JSON.stringify(row), "r1", ["stateResult"])).toEqual(row);
  expect(correlated(JSON.stringify(row), "r2", ["stateResult"])).toBeNull();
  for (const bad of [{ protocolVersion: 1 }, { type: "mockState" }, { success: false }]) {
    expect(() => correlated(JSON.stringify({ ...row, ...bad }), "r1", ["stateResult"])).toThrow();
  }
  const target = { pid: 22, generation: "111.112233", cdhash: "a".repeat(40) };
  expect(sameProcess(target, target)).toBe(true);
  for (
    const wrong of [null, { ...target, pid: 23 }, { ...target, generation: "111.112234" }, {
      ...target,
      cdhash: "b".repeat(40),
    }]
  ) expect(sameProcess(wrong, target)).toBe(false);
});

test("starter submission requires the verified fresh release and its exact source-owned row", () => {
  const submit = (semanticId: string) => ({ type: "batch", target: { type: "instance", id: "main", generation: 5 }, commands: [{ type: "selectBySemanticId", semanticId, submit: true }] });
  expect(() => authorizeInstalledCommand(submit(STABILITY_STARTER.semanticId), "/tmp/owned")).toThrow();
  expect(() => authorizeInstalledCommand(submit("same-name-other-row"), "/tmp/owned", true)).toThrow();
  expect(() => authorizeInstalledCommand(submit(STABILITY_STARTER.semanticId), "/tmp/owned", true)).not.toThrow();
});

test("installed transport only permits in-app journey actions, never native or grant requests", () => {
  const target = { type: "instance", id: "main", generation: 5 }, scratch = "/tmp/owned";
  expect(() => authorizeInstalledCommand({ type: "simulateKey", key: "enter", target }, scratch)).not
    .toThrow();
  for (
    const cmd of [
      { type: "captureScreenshot" },
      { type: "checkAccessibilityLater" },
      { type: "requestPermission" },
      { type: "simulateKey", key: "v", target, modifiers: ["cmd"] },
      { type: "simulateKey", key: "escape" },
      { type: "run", path: "/tmp/owned/installed-journey-demo/../arbitrary.ts" },
      { type: "batch", target, commands: [{ type: "forceSubmit", value: 1 }] },
    ]
  ) expect(() => authorizeInstalledCommand(cmd, scratch)).toThrow();
});
