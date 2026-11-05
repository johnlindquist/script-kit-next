// Pure contracts; none of these authorize execution on their own.
export const REPO = "johnlindquist/script-kit-next";
export const APP = "/Applications/Script Kit.app";
export const EXE = "Contents/MacOS/script-kit-gpui";
export const ID = "com.scriptkit.app";
export const TEAM = "9822B7V7MD";
export const ARCHIVE = "Script-Kit-macos.zip";
export const WORKFLOW = ".github/workflows/installed-release-smoke.yml";
const tagPattern = /^v(0|[1-9]\d{0,8})\.(0|[1-9]\d{0,8})\.(0|[1-9]\d{0,8})$/;
export const sha = /^[a-f0-9]{40}$/;
const hash = /^[a-f0-9]{64}$/;
export type Json = Record<string, any>;
export class SmokeFault extends Error {
  constructor(readonly code: string, readonly status: "fail" | "blocked" = "fail") {
    super(code);
  }
}
export function check(fact: unknown, code: string, status?: "fail" | "blocked"): asserts fact {
  if (!fact) throw new SmokeFault(code, status);
}
export function intent(env: NodeJS.ProcessEnv) {
  const ref = env.GITHUB_REF ?? "";
  const prefix = "refs/heads/qa/installed-release/";
  const tag = ref.slice(prefix.length);
  check(
    ref.startsWith(prefix) && tagPattern.test(tag) && env.GITHUB_REF_TYPE === "branch" &&
      env.GITHUB_REPOSITORY === REPO && env.GITHUB_SERVER_URL === "https://github.com" &&
      env.GITHUB_API_URL === "https://api.github.com" &&
      env.GITHUB_WORKFLOW_REF === `${REPO}/${WORKFLOW}@${ref}` &&
      sha.test(env.GITHUB_SHA ?? "") && env.GITHUB_WORKFLOW_SHA === env.GITHUB_SHA &&
      ["push", "workflow_dispatch"].includes(env.GITHUB_EVENT_NAME ?? "") && env.GITHUB_JOB === "installed",
    "wrong_workflow",
    "blocked",
  );
  check(
    env.SCRIPT_KIT_NONINTERACTIVE === "0" && env.CI === "true" && env.GITHUB_ACTIONS === "true" &&
      env.INSTALLED_RUNNER_ENVIRONMENT === "github-hosted" && env.RUNNER_OS === "macOS" &&
      env.RUNNER_ARCH === "ARM64" &&
      env.RUNNER_NAME && /^\d+$/.test(env.GITHUB_RUN_ID ?? "") &&
      /^[1-9]\d*$/.test(env.GITHUB_RUN_ATTEMPT ?? "") &&
      !Object.keys(env).some((k) =>
        (k.startsWith("SCRIPT_KIT_") && k !== "SCRIPT_KIT_NONINTERACTIVE") || k === "SK_PATH" ||
        k.startsWith("DYLD_")
      ),
    "wrong_host_intent",
    "blocked",
  );
  check(env.INSTALLED_MODE === "fresh" || env.INSTALLED_MODE === "update", "invalid_mode");
  return { ref, tag, mode: env.INSTALLED_MODE, verificationSha: env.GITHUB_SHA! };
}
export function previousTag(target: string, input: string, commitMessage: string): string | null {
  const lines = commitMessage.split("\n").filter((line) => /Installed-Release-Update-From:/i.test(line));
  check(!input || !lines.length, "ambiguous_update_source");
  check(lines.length <= 1, "ambiguous_update_source");
  const value = input || lines[0]?.replace(/^Installed-Release-Update-From: /, "");
  if (!value) return null;
  check(tagPattern.test(target) && tagPattern.test(value), "invalid_update_tag");
  const a = value.slice(1).split(".").map(Number), b = target.slice(1).split(".").map(Number);
  const different = a.findIndex((n, i) => n !== b[i]);
  check(different >= 0 && a[different]! < b[different]!, "update_not_forward");
  return value;
}
export function verifyPublicJob(run: Json, jobs: Json, env: NodeJS.ProcessEnv) {
  const { mode, ref, verificationSha } = intent(env);
  check(
    run.repository?.full_name === REPO && run.repository?.private === false &&
      run.head_repository?.full_name === REPO &&
      String(run.id) === env.GITHUB_RUN_ID && String(run.run_attempt) === env.GITHUB_RUN_ATTEMPT &&
      run.path === WORKFLOW &&
      run.event === env.GITHUB_EVENT_NAME && run.head_sha === verificationSha &&
      run.head_branch === ref.slice("refs/heads/".length) &&
      run.status === "in_progress" && Number.isFinite(Date.parse(run.created_at)),
    "workflow_run_not_verified",
    "blocked",
  );
  const job = jobs.jobs?.filter((j: Json) => j.name === `Installed release (${mode})`);
  check(
    job?.length === 1 && job[0].status === "in_progress" && String(job[0].run_id) === env.GITHUB_RUN_ID &&
      job[0].head_sha === verificationSha && job[0].labels?.length === 1 && job[0].labels[0] === "macos-15" &&
      job[0].runner_name === env.RUNNER_NAME && Number.isSafeInteger(job[0].id),
    "assigned_runner_not_verified",
    "blocked",
  );
  return { jobId: job[0].id, runCreatedAt: run.created_at as string };
}
export function selectAsset(release: Json, tag: string, name: string, runCreatedAt: string) {
  check(
    tagPattern.test(tag) && release.tag_name === tag && release.draft === false &&
      release.prerelease === false &&
      Number.isSafeInteger(release.id) && Date.parse(release.published_at) <= Date.parse(runCreatedAt) &&
      release.html_url === `https://github.com/${REPO}/releases/tag/${tag}`,
    "not_a_prior_public_release",
  );
  const rows = release.assets?.filter((a: Json) => a.name === name);
  check(rows?.length === 1, "missing_or_ambiguous_asset");
  const a = rows[0];
  check(
    a.state === "uploaded" && Number.isSafeInteger(a.id) && a.id > 0 &&
      /^sha256:[a-f0-9]{64}$/.test(a.digest) &&
      Number.isSafeInteger(a.size) && a.size > 0 &&
      a.size <= (name === ARCHIVE ? 768 * 1024 ** 2 : 8 * 1024 ** 2) &&
      a.browser_download_url === `https://github.com/${REPO}/releases/download/${tag}/${name}`,
    "invalid_asset",
  );
  return {
    id: a.id as number,
    name,
    url: a.browser_download_url as string,
    size: a.size as number,
    sha256: a.digest.slice(7) as string,
  };
}
export function verifyManifest(
  m: Json,
  tag: string,
  sourceSha: string,
  asset: ReturnType<typeof selectAsset>,
) {
  const a = m.artifacts?.filter((row: Json) => row.name === ARCHIVE);
  check(
    sha.test(sourceSha) && m.schema_version === 3 && m.source_sha === sourceSha && m.tag === tag &&
      m.version === tag.slice(1) &&
      m.bundle?.identifier === ID && m.bundle.executable?.path === EXE &&
      hash.test(m.bundle.executable.sha256) &&
      m.bundle.sidecar?.path === "Contents/MacOS/goose" && hash.test(m.bundle.sidecar.sha256) &&
      m.distribution_security?.team_identifier === TEAM && m.verification?.status === "pass" &&
      a?.length === 1 && a[0].platform === "macos" && a[0].sha256 === asset.sha256 &&
      a[0].size_bytes === asset.size,
    "release_manifest_mismatch",
  );
  return {
    executableSha256: m.bundle.executable.sha256 as string,
    sidecarSha256: m.bundle.sidecar.sha256 as string,
  };
}
export interface Identity {
  pid: number;
  generation: string;
  cdhash: string;
}
export function sameProcess(actual: Identity | null, expected: Identity): boolean {
  return !!actual && actual.pid === expected.pid && actual.generation === expected.generation &&
    actual.cdhash === expected.cdhash;
}
// This is a task-specific subset of the app's protocol. No OS event injection,
// clipboard, capture, device/permission requests, settings launch or mock state.
export const STABILITY_STARTER = Object.freeze({
  source: "8ec77ff17a7c8520615f1e74630f55a1581d3652",
  sha256: "2e1dfb2669ec978b50c6080ef18fb9f5a20aa1aefd8dee5c43032df6a97bbc88",
  // Source-owned script/main row for the default workspace on /Users/runner.
  semanticId: "main-list-row:v2:64c6ffc7eed5c8f7f1a9ee78d731368cd314204da62103ee751aa4cbf1e72302",
});
export function authorizeInstalledCommand(cmd: Json, scratch: string, verifyStarter = false, observedPaneRows?: ReadonlySet<string>) {
  if (
    ["listAutomationWindows", "getState", "getElements", "checkAccessibility", "openNotes", "show"].includes(
      cmd.type,
    )
  ) return;
  if (cmd.type === "run") {
    check(
      typeof cmd.path === "string" && cmd.path.startsWith(scratch + "/") &&
        /^installed-journey-[A-Za-z0-9_-]+\/exercise\.ts$/.test(cmd.path.slice(scratch.length + 1)),
      "unreviewed_script_path",
    );
    return;
  }
  check(
    cmd.target?.type === "instance" && cmd.target.id === "main" &&
      Number.isSafeInteger(cmd.target.generation) && cmd.target.generation > 0,
    "unbound_ui_target",
  );
  if (cmd.type === "simulateKey") {
    const mods = cmd.modifiers ?? [];
    check(
      Array.isArray(mods) && ((["enter", "escape"].includes(cmd.key) && !mods.length) ||
        (cmd.key === "s" && mods.length === 1 && mods[0] === "cmd")),
      "unreviewed_in_app_key",
    );
    return;
  }
  check(
    cmd.type === "batch" && cmd.commands?.length === 1 &&
      ["setInput", "selectBySemanticId"].includes(cmd.commands[0].type),
    "unreviewed_command",
  );
  const row = cmd.commands[0];
  if (row.type === "selectBySemanticId") {
    const setup = "main-list-row:v2:059f1a69b8c1e700afbeee9dea2a2de81e231afa798f64e77247d04a2eaa0adb";
    const status = "main-list-row:v2:3485d92f119229f9ce4e08bf509f97bb85457fea64d391b0c1cac4494cd9454e";
    check(
      row.semanticId === setup || (row.semanticId === status && row.submit !== true) ||
        (["main-list-row:v2:16cf1fc9a69fa8aa98d65534af5e7c06f11cdfb262a43a3accda7735a5780c79",
          "main-list-row:v2:6a48e7c4c80e6c41d75cddedcc44324ee352b911c5a2cd89653101723b3d16c8"].includes(row.semanticId) && row.submit === true) ||
        (verifyStarter && row.semanticId === STABILITY_STARTER.semanticId && row.submit === true) ||
        (row.submit === false && typeof row.semanticId === "string" && /^personal-item:[0-9a-f]{8}(?:-[0-9a-f]{4}){3}-[0-9a-f]{12}:h-[0-9a-f]{64}$/.test(row.semanticId) && observedPaneRows?.has(row.semanticId) === true &&
          Object.keys(row).every(k=>["type","semanticId","submit"].includes(k))),
      "unreviewed_menu_selection",
    );
  }
}
export function correlated(line: string, id: string, expect: string[]): Json | null {
  let v: Json;
  try {
    v = JSON.parse(line);
  } catch {
    return null;
  } // ordinary startup text is not protocol evidence
  if (!v || v.requestId !== id) return null;
  check(
    v.protocolVersion === 2 && expect.includes(v.type) && v.ok !== false && v.success !== false,
    "protocol_response_refused",
  );
  return v;
}
