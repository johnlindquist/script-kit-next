import { createHash, randomUUID } from "node:crypto";
import { spawnSync } from "node:child_process";
import {
  closeSync,
  constants as F,
  createReadStream,
  fstatSync,
  fsyncSync,
  ftruncateSync,
  lstatSync,
  mkdirSync,
  mkdtempSync,
  openSync,
  readFileSync,
  readSync,
  realpathSync,
  statSync,
  writeFileSync,
  writeSync,
} from "node:fs";
import { release, userInfo } from "node:os";
import { dirname, join } from "node:path";
import { StringDecoder } from "node:string_decoder";
import { spawnOwnedProcess } from "../../agentic/owned-process";
import { challengeSettingsSdk, sdkEntry } from "./installed-release-settings-sdk";
import { challengeSettings, confirmJourneyBytes, releasedSettingsSource, releasedWorkspaceSource, settingsOwners, type SettingsInvocation, type SettingsSources } from "./installed-release-settings";
import { challengeCommand, releasedBunPin } from "./installed-release-command";
import { assertNoninteractiveVisualProbe } from "./operator-safety";
import {
  APP,
  ARCHIVE,
  authorizeInstalledCommand,
  STABILITY_STARTER,
  check,
  correlated,
  EXE,
  ID,
  type Identity,
  intent,
  type Json,
  previousTag,
  REPO,
  sameProcess,
  SmokeFault,
  TEAM,
  verifyPublicJob,
} from "./installed-release-contract";
import { assetBytes, fetchPublicRelease, publicJson } from "./installed-release-distribution";

export interface InstalledKit {
  kitPath: string;
  scratchPath: string;
  freshStarter?: { sha256: string; semanticId: string };
  verifyStdinNoteReopen?: boolean;
  originalData?: {date:string; noteSha256:string; configSha256:string; editorFontSize:number};
  pid: number;
  send(command: Record<string, any>): Promise<void>;
  request(command: Record<string, any>, expect: string | string[], timeoutMs?: number): Promise<any>;
}
const HOME = "/Users/runner", KIT = `${HOME}/.scriptkit`, TEMP = `${HOME}/work/_temp`;
const PROJECT = `${HOME}/work/script-kit-next/script-kit-next`;
const lib = import.meta.dir;
type Release = Awaited<ReturnType<typeof fetchPublicRelease>>;
const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));
const noEntry = (path: string) => lstatSync(path, { throwIfNoEntry: false }) === undefined;
const missing = (path: string) => check(noEntry(path), "pre_existing_target", "blocked");
const dirIdentity = (path: string) => {
  const s = lstatSync(path);
  check(s.isDirectory() && realpathSync(path) === path, "unexpected_directory");
  return `${s.dev}:${s.ino}`;
};
const reporting = (error: unknown) =>
  error instanceof SmokeFault
    ? { status: error.status, code: error.code }
    : { status: "fail", code: "unexpected_operation_error" };

// No signing/repo/API token, credential helper, SK_PATH, home override to a fixture,
// safety override, diagnostic flag or owned-evaluation switch is passed to the app.
function cleanEnv(): NodeJS.ProcessEnv {
  return {
    HOME,
    USER: "runner",
    LOGNAME: "runner",
    LANG: "en_US.UTF-8",
    PATH: `${dirname(process.execPath)}:/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin`,
  };
}
function command(code: string, exe: string, args: string[], timeout = 20_000, env = cleanEnv()) {
  assertNoninteractiveVisualProbe("installed-release-host-subprocess");
  const p = spawnSync(exe, args, {
    cwd: HOME,
    env,
    timeout,
    killSignal: "SIGKILL",
    maxBuffer: 1024 * 1024,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
  });
  // Never leak stdout/stderr of app-adjacent tools or a redirected asset URL to CI logs.
  check(!p.error && p.status === 0, code);
  return `${p.stdout ?? ""}${p.stderr ?? ""}`.trim();
}
async function attestHost() {
  // First effect in every live entry. Refusal is intentionally BEFORE disk writes, child processes and public API calls.
  assertNoninteractiveVisualProbe("installed-release-smoke");
  const env = process.env, plan = intent(env);
  check(
    process.platform === "darwin" && process.arch === "arm64" && release().split(".")[0] === "24" &&
      process.getuid!() === process.geteuid!() && process.getuid!() > 0 && userInfo().username === "runner" &&
      userInfo().homedir === HOME &&
      env.HOME === HOME && env.RUNNER_TEMP === TEMP && env.GITHUB_WORKSPACE === PROJECT &&
      realpathSync(process.cwd()) === PROJECT && realpathSync(TEMP) === TEMP &&
      realpathSync(PROJECT) === PROJECT &&
      statSync(TEMP).uid === process.getuid!() && lstatSync(PROJECT).uid === process.getuid!(),
    "not_dedicated_host",
    "blocked",
  );
  const path = `actions/runs/${env.GITHUB_RUN_ID}/attempts/${env.GITHUB_RUN_ATTEMPT}`;
  const [run, jobs] = await Promise.all([publicJson(path), publicJson(`${path}/jobs?per_page=100`)]);
  const verified = verifyPublicJob(run, jobs, env);
  // A copied event/env (even from a real public run) is not a runner process.
  let ancestor = process.ppid, foundWorker = false;
  for (let i = 0; i < 12 && ancestor > 1; i++) {
    const line = command("cannot_inspect_runner_parent", "/bin/ps", [
      "-p",
      String(ancestor),
      "-o",
      "ppid=",
      "-o",
      "comm=",
    ]);
    const m = /^(\d+)\s+(.+)$/.exec(line);
    check(m, "unknown_process_ancestry", "blocked");
    if (/^\/.*\/bin\/Runner\.Worker$/.test(m[2]!)) {
      foundWorker = true;
      break;
    }
    ancestor = Number(m[1]);
  }
  check(foundWorker, "no_actions_worker_ancestor", "blocked");
  check(
    command("cannot_verify_checkout", "/usr/bin/git", ["-C", PROJECT, "rev-parse", "HEAD"]) ===
        plan.verificationSha &&
      command("cannot_verify_checkout", "/usr/bin/git", [
          "-C",
          PROJECT,
          "status",
          "--porcelain=v1",
          "--untracked-files=all",
        ]) === "",
    "dirty_or_wrong_checkout",
  );
  return { ...plan, ...verified };
}
async function fileHash(path: string) {
  const h = createHash("sha256");
  for await (const chunk of createReadStream(path)) h.update(chunk);
  return h.digest("hex");
}
function quarantined(path: string) {
  // Presence only. Never synthesize, strip, or record the contents of quarantine metadata.
  return command("unreadable_xattrs", "/usr/bin/xattr", [path]).split("\n").includes("com.apple.quarantine");
}
async function verifyBundle(path: string, release: Release) {
  const plist = (key: string) =>
    command("bad_bundle_plist", "/usr/bin/plutil", [
      "-extract",
      key,
      "raw",
      "-expect",
      "string",
      "-o",
      "-",
      `${path}/Contents/Info.plist`,
    ]);
  check(
    plist("CFBundleIdentifier") === ID && plist("CFBundleExecutable") === "script-kit-gpui" &&
      plist("CFBundleShortVersionString") === release.tag.slice(1) &&
      plist("NSMicrophoneUsageDescription").length > 0,
    "installed_bundle_metadata_mismatch",
  );
  check(
    await fileHash(`${path}/${EXE}`) === release.executableSha256 &&
      await fileHash(`${path}/Contents/MacOS/goose`) === release.sidecarSha256,
    "bundle_digest_mismatch",
  );
  const requirement =
    `anchor apple generic and identifier "${ID}" and certificate leaf[subject.OU] = "${TEAM}" and certificate leaf[field.1.2.840.113635.100.6.1.13] exists`;
  command("invalid_bundle_signature", "/usr/bin/codesign", [
    "--verify",
    "--deep",
    "--strict",
    "--verbose=2",
    path,
  ], 60_000);
  command("wrong_distribution_identity", "/usr/bin/codesign", [
    "--verify",
    "--strict",
    `-R=${requirement}`,
    path,
  ], 60_000);
  const signature = command("cannot_inspect_signature", "/usr/bin/codesign", [
    "-d",
    "--arch",
    "arm64",
    "--verbose=4",
    path,
  ]);
  const cdhash = /^CDHash=([a-f0-9]{40})$/m.exec(signature)?.[1];
  check(
    cdhash && signature.split("\n").includes(`TeamIdentifier=${TEAM}`) &&
      signature.includes("Authority=Developer ID Application:") &&
      /flags=0x[\da-f]+\([^)]*runtime/.test(signature),
    "wrong_distribution_identity",
  );
  command("notarization_ticket_invalid", "/bin/bash", [join(PROJECT, "scripts/release-stapler.sh"), "validate", path], 120_000, { ...cleanEnv(), PATH: "/usr/bin:/bin:/usr/sbin:/sbin" });
  const assessment = command("gatekeeper_rejected", "/usr/sbin/spctl", [
    "--assess",
    "--type",
    "execute",
    "--verbose=4",
    path,
  ], 60_000);
  check(assessment.includes("source=Notarized Developer ID"), "wrong_gatekeeper_source");
  return {
    cdhash,
    team: TEAM,
    signature: "Developer ID Application",
    stapled: true,
    gatekeeper: "Notarized Developer ID",
    quarantined: quarantined(path),
  };
}
// Atomic and no-clobber, including a dangling symlink or an empty target directory.
function moveExclusive(from: string, to: string) {
  missing(to);
  command("exclusive_move_refused", "/usr/bin/python3", [
    "-I",
    "-B",
    "-c",
    "import ctypes,os,sys; f=ctypes.CDLL(None,use_errno=True).renamex_np; sys.exit(0 if f(os.fsencode(sys.argv[1]),os.fsencode(sys.argv[2]),4)==0 else 1)",
    from,
    to,
  ]);
}
async function install(tag: string, time: string, scratch: string, record: Json, checkpoint: () => void) {
  const dir = mkdtempSync(`${scratch}/release-`);
  record.stage = "download";
  checkpoint();
  const release = await fetchPublicRelease(tag, time);
  record.release = release;
  checkpoint();
  const zip = join(dir, ARCHIVE),
    extracted = join(dir, "extracted"),
    bundle = join(extracted, "Script Kit.app");
  const fd = openSync(zip, "wx", 0o600);
  try {
    for await (const bytes of assetBytes(release.archive)) {
      let offset = 0;
      while (offset < bytes.length) {
        const n = writeSync(fd, bytes, offset, bytes.length - offset);
        check(n > 0, "asset_write_failed");
        offset += n;
      }
    }
  } finally {
    closeSync(fd);
  }
  record.archiveQuarantined = quarantined(zip);
  record.stage = "verify-distribution";
  checkpoint();
  command("unsafe_or_unsupported_release_zip", "/usr/bin/python3", [
    "-I",
    "-B",
    join(lib, "installed-release-archive.py"),
    zip,
  ], 120_000);
  mkdirSync(extracted, { mode: 0o700 });
  command("extraction_failed", "/usr/bin/ditto", ["-x", "-k", zip, extracted], 120_000);
  record.staged = await verifyBundle(bundle, release);
  record.stage = "install";
  checkpoint();
  // Some runner images mount the work volume separately. Never fall back to a
  // recursive copy into the final path. Prepare a unique private destination on
  // the Applications volume and re-verify it before the atomic exclusive rename.
  let movable = bundle, localRoot = dir;
  if (statSync(bundle).dev !== statSync(dirname(APP)).dev) {
    localRoot = mkdtempSync("/Applications/.script-kit-smoke-");
    movable = join(localRoot, "Script Kit.app");
    command("same_volume_staging_failed", "/usr/bin/ditto", [bundle, movable], 120_000);
    const copy = await verifyBundle(movable, release);
    check(
      copy.cdhash === record.staged.cdhash && copy.quarantined === record.staged.quarantined,
      "staging_copy_mismatch",
    );
  }
  moveExclusive(movable, APP);
  const inode = dirIdentity(APP);
  record.installed = await verifyBundle(APP, release);
  record.path = APP;
  check(
    record.staged.cdhash === record.installed.cdhash &&
      (!record.staged.quarantined || record.installed.quarantined) &&
      (!record.archiveQuarantined || record.installed.quarantined),
    "installed_verification_changed",
  );
  record.stage = "launch";
  checkpoint();
  return { release, inode, localRoot, cdhash: record.installed.cdhash as string };
}

function inspectors(scratch: string) {
  const helper = join(scratch, "inspect-process"), cap = randomUUID();
  command("cannot_compile_passive_inspector", "/usr/bin/xcrun", [
    "clang",
    "-fobjc-arc",
    `-DSMOKE_CAPABILITY="${cap}"`,
    join(lib, "installed-release-process.m"),
    "-framework",
    "AppKit",
    "-framework",
    "Security",
    "-framework",
    "CoreGraphics",
    "-lproc",
    "-o",
    helper,
  ], 90_000);
  return (code: string, args: string[]) =>
    command(code, helper, args, 5000, { ...cleanEnv(), SMOKE_CAP: cap });
}
type Inspect = ReturnType<typeof inspectors>;
async function withInstance<T>(
  inspect: Inspect,
  scratch: string,
  cdhash: string,
  record: Json,
  checkpoint: () => void,
  verifyStarter: boolean,
  body: (kit: InstalledKit) => Promise<T>,
  observePaneSelection = false,
): Promise<T> {
  const io = mkdtempSync(`${scratch}/instance-`), fifo = join(io, "stdin"), out = join(io, "stdout");
  command("cannot_create_private_fifo", "/usr/bin/mkfifo", ["-m", "600", fifo]);
  const stdin = openSync(fifo, F.O_RDWR | F.O_NONBLOCK), stdout = openSync(out, "wx+", 0o600);
  const args = [fifo, out, cdhash];
  type Live = Identity & { onscreen: boolean };
  const current = (): Live | null =>
    JSON.parse(inspect("cannot_verify_running_instance", ["inspect", ...args]));
  let owner: Live | null = null, launched = false;
  let cursor = 0, buffer = "";
  const decoder = new StringDecoder("utf8");
  function response(id: string, types: string[]) {
    const size = fstatSync(stdout).size;
    check(size >= cursor && size <= 128 * 1024 ** 2, "unexpected_protocol_spool");
    while (true) {
      const newline = buffer.indexOf("\n");
      if (newline !== -1) {
        const line = buffer.slice(0, newline);
        buffer = buffer.slice(newline + 1);
        const match = correlated(line, id, types);
        if (match) return match;
        continue;
      }
      check(buffer.length < 16 * 1024 ** 2, "protocol_line_unbounded");
      if (cursor >= size) return null;
      const bytes = Buffer.allocUnsafe(Math.min(size - cursor, 65536));
      const n = readSync(stdout, bytes, 0, bytes.length, cursor);
      check(n > 0, "protocol_spool_truncated");
      cursor += n;
      buffer += decoder.write(bytes.subarray(0, n));
    }
  }
  let cancelled = false;
  const cancel = () => {
    cancelled = true;
  };
  const active = () => check(!cancelled, "runner_cancelled", "blocked");
  process.once("SIGINT", cancel);
  process.once("SIGTERM", cancel);
  try {
    inspect("app_already_running", ["absent"]);
    // open receives only the minimal environment. No -F, --args, harness/evaluator, or fake grants.
    launched = true;
    command("launchservices_open_failed", "/usr/bin/open", [
      "-n",
      "-a",
      APP,
      "--stdin",
      fifo,
      "--stdout",
      out,
      "--stderr",
      "/dev/null",
    ], 30_000);
    const until = Date.now() + 45_000;
    do {
      active();
      owner = current();
      if (owner) break;
      await sleep(150);
    } while (Date.now() < until);
    check(owner, "installed_launch_not_observed", "blocked");
    record.owner = { pid: owner.pid, generation: owner.generation, cdhash: owner.cdhash };
    checkpoint();
    const owned = owner, ids = new Set<string>();
    const paneRows = new Set<string>();
    let paneGeneration: number | undefined;
    let busy = false;
    const assertOwner = () => check(sameProcess(current(), owned), "installed_instance_changed");
    function write(command: Json, request: boolean) {
      active();
      assertNoninteractiveVisualProbe("installed-release-send");
      authorizeInstalledCommand(command, io, verifyStarter,
        observePaneSelection && command.target?.generation === paneGeneration ? paneRows : undefined);
      if(command.type!=="getState"){paneRows.clear();paneGeneration=undefined;}
      if (command.type === "run") {
        check(
          realpathSync(command.path) === command.path && lstatSync(command.path).isFile(),
          "unreviewed_script_path",
        );
      }
      assertOwner();
      const id = command.requestId ?? randomUUID();
      check(
        typeof id === "string" && /^[A-Za-z0-9.:/_-]{1,192}$/.test(id) && !ids.has(id) && ids.size < 10000 &&
          (command.protocolVersion === undefined || command.protocolVersion === 2),
        "invalid_or_reused_request_id",
      );
      ids.add(id);
      const bytes = Buffer.from(
        JSON.stringify({ ...command, ...(request ? { requestId: id } : {}), protocolVersion: 2 }) + "\n",
      );
      check(
        bytes.length <= 32 * 1024 && writeSync(stdin, bytes) === bytes.length,
        "incomplete_protocol_write",
      );
      return id;
    }
    const kit: InstalledKit = {
      kitPath: KIT,
      scratchPath: io,
      freshStarter: verifyStarter ? STABILITY_STARTER : undefined,
      pid: owner.pid,
      send: async (cmd) => {
        check(!busy, "overlapping_protocol_operation");
        write(cmd, false);
      },
      request: async (cmd, expect, timeoutMs = 10_000) => {
        const types = Array.isArray(expect) ? expect : [expect];
        check(
          !busy && types.length > 0 && Number.isSafeInteger(timeoutMs) && timeoutMs > 0 &&
            timeoutMs <= 90_000,
          "invalid_protocol_operation",
        );
        busy = true;
        try {
          const deadline = Date.now() + timeoutMs, id = write(cmd, true);
          do {
            active();
            const result = response(id, types);
            if (result) {
              assertOwner();
              if(observePaneSelection && cmd.type==="getElements" && result.type==="elementsResult" && cmd.target?.type==="instance" && cmd.target.id==="main" && Number.isSafeInteger(cmd.target.generation) && cmd.target.generation>0){
                paneGeneration=cmd.target.generation;
                for(const e of result.elements??[])if(e.role==="row"&&e.selectable===true&&e.actionDisabled==="personal_list_read_only"&&/^personal-item:[0-9a-f-]{36}:h-[0-9a-f]{64}$/.test(e.semanticId))paneRows.add(e.semanticId);
              }
              return result;
            }
            await sleep(40);
          } while (Date.now() < deadline);
          throw new SmokeFault("protocol_response_timeout");
        } finally {
          busy = false;
        }
      },
    };
    const initial = await kit.request({ type: "getState" }, "stateResult", 45_000);
    check(
      typeof initial.windowVisible === "boolean" && typeof initial.promptType === "string" &&
        initial.promptType.length,
      "invalid_initial_app_state",
    );
    record.visibleAtLaunch = initial.windowVisible;
    record.initialSurface = /^[A-Za-z][A-Za-z0-9_-]{0,64}$/.test(initial.promptType)
      ? initial.promptType
      : "unrecognized";
    if (!initial.windowVisible) {
      const ack = await kit.request({ type: "show" }, "windowVisibilityAck");
      check(ack.windowVisible === true, "show_not_acknowledged");
    }
    const visibleUntil = Date.now() + 10_000;
    let visible = false;
    do {
      const state = await kit.request({ type: "getState" }, "stateResult");
      const os = current();
      check(sameProcess(os, owned), "installed_instance_changed");
      visible = state.windowVisible === true && os!.onscreen;
      if (!visible) await sleep(100);
    } while (!visible && Date.now() < visibleUntil);
    check(visible, "no_verified_onscreen_window", "blocked");
    record.visibleOwnedWindow = true;
    record.stage = "journeys";
    checkpoint();
    const result = await body(kit);
    active();
    assertOwner();
    return result;
  } finally {
    try {
      if (launched) {
        // Diagnose an ambiguous open exactly once, never re-open or kill a name/PID.
        const actual = current();
        if (actual) {
          check(!owner || sameProcess(actual, owner), "refused_cleanup_identity_changed");
          inspect("exact_termination_refused", ["terminate", ...args, String(actual.pid), actual.generation]);
          const until = Date.now() + 10_000;
          let gone = false;
          // Descriptors may close before process exit; poll the kernel generation,
          // not those disappearing descriptors. This operation cannot signal anyone.
          do {
            await sleep(150);
            gone =
              inspect("cannot_verify_owned_exit", ["departed", String(actual.pid), actual.generation]) ===
                "true";
          } while (!gone && Date.now() < until);
          check(gone, "owned_process_did_not_exit");
          record.cleanup = "exact_instance_terminated";
        } else record.cleanup = owner ? "owned_process_already_exited" : "no_owned_process_observed";
      }
    } catch (error) {
      record.cleanup = "identity_or_termination_unverified";
      throw error;
    } finally {
      process.removeListener("SIGINT", cancel);
      process.removeListener("SIGTERM", cancel);
      closeSync(stdin);
      closeSync(stdout);
      checkpoint();
    }
  }
}

export async function runOnHostedMac(journeys: (kit: InstalledKit) => Promise<Json>) {
  const env = process.env, plan = await attestHost();
  const fd = openSync(join(TEMP, "installed-release-receipt.json"), "wx", 0o600);
  const receipt: Json = {
    schemaVersion: 1,
    status: "incomplete",
    repo: REPO,
    ...plan,
    runId: env.GITHUB_RUN_ID,
    runAttempt: env.GITHUB_RUN_ATTEMPT,
    launch: "LaunchServices open",
    os: "macos-15-arm64",
    verifiedInstalls: [],
    statusScope: "automated-installed-release-journeys-only",
    consentAndPhysicalCoverage: "requires-human-evidence",
    limitations: [
      "Quarantine presence and Gatekeeper assessment do not establish a visible first-open or TCC dialog.",
      "No OS dialogs are driven, and no real device or consent grant is simulated.",
    ],
  };
  const checkpoint = () => {
    const bytes = Buffer.from(JSON.stringify(receipt, null, 2) + "\n");
    check(bytes.length < 4 * 1024 ** 2, "receipt_budget_exceeded");
    ftruncateSync(fd, 0);
    check(writeSync(fd, bytes, 0, bytes.length, 0) === bytes.length, "receipt_write_failed");
    fsyncSync(fd);
  };
  try {
    checkpoint();
    const msg = env.GITHUB_EVENT_NAME === "push"
      ? command("missing_push_tip", "/usr/bin/git", ["-C", PROJECT, "log", "-1", "--format=%B"])
      : "";
    const previous = previousTag(plan.tag, env.INSTALLED_PREVIOUS_INPUT ?? "", msg);
    check(plan.mode !== "update" || previous, "previous_tag_required");
    receipt.requestedUpdateFrom = previous;
    receipt.verifiedUpdate = false;
    // Clean disposable account, not a cleaned-out or redirected old workspace. No removals.
    for (
      const path of [
        APP,
        KIT,
        `${HOME}/.kit`,
        `${HOME}/.kenv`,
        `${HOME}/.config/gh/hosts.yml`,
        `${HOME}/.git-credentials`,
        `${HOME}/.codex/auth.json`,
        `${HOME}/.aws/credentials`,
      ]
    ) missing(path);
    const scratch = mkdtempSync(`${TEMP}/installed-smoke-`), inspect = inspectors(scratch);
    try {
      inspect("no_unlocked_aqua_console", ["aqua"]);
    } catch {
      throw new SmokeFault("no_unlocked_aqua_console", "blocked");
    }
    receipt.aqua = true;
    type UpgradeMarker = {path:string; value:string; workspace:string; day:string; dayHash:string; configHash:string; editorFontSize:number; initialGrants:string[]};
    const update:{marker:UpgradeMarker|null}={marker:null};
    const personalHash=async (relative:string) => {
      check(relative==="config.ts" || /^brain\/days\/\d{4}-\d{2}-\d{2}\.md$/.test(relative),"unexpected_upgrade_file");
      const path=join(KIT,relative), st=lstatSync(path);
      check(st.isFile() && st.nlink===1 && st.size<=1024*1024 && realpathSync(path)===path,"unverified_upgrade_file");
      return fileHash(path);
    };
    for (const tag of plan.mode === "update" ? [previous!, plan.tag] : [plan.tag]) {
      const r: Json = { tag };
      receipt.verifiedInstalls.push(r);
      const installed = await install(tag, plan.runCreatedAt, scratch, r, checkpoint);
      const personalSource = "03c680dc872b5919d0d80995fa5c244093b933ec";
      const starterSource = tag === "v0.1.33" ? STABILITY_STARTER.source : tag === "v0.1.34" ? personalSource : null;
      const verifyStarter = starterSource !== null && update.marker === null;
      if (verifyStarter) check(installed.release.sourceSha === starterSource, "unknown_starter_source");
      await withInstance(inspect, scratch, installed.cdhash, r, checkpoint, verifyStarter, async (kit) => {
        let marker=update.marker;
        const workspace = dirIdentity(KIT);
        check(lstatSync(KIT).uid === process.getuid!(), "wrong_workspace_owner");
        if (tag !== plan.tag) {
          // Seed the prior version by exercising its actual SDK, form and Notes editor,
          // not by fabricating their output files. Keep readbacks before the second app writes.
          const prior = await journeys(kit);
          check(prior?.status==="pass" && prior.evidence?.notes?.savedAndReopened === true && prior.evidence?.settings?.revisionMatchedDisk === true,"prior_installed_journeys_not_verified");
          const day = "brain/days/"+prior.evidence.notes.date+".md", dayHash=await personalHash(day);
          check(dayHash===prior.evidence.notes.sha256,"prior_app_note_readback_mismatch");
          const configHash=await personalHash("config.ts");
          const initialGrants=Object.entries(prior.checkpoints.start.permissions).filter(([,row])=>(row as Json).status==="granted").map(([name])=>name);
          r.journeys=prior;
          const scripts = join(KIT, "plugins/main/scripts");
          dirIdentity(scripts);
          marker = {
            path: join(scripts, `installed-upgrade-${randomUUID()}.ts`),
            value: `// Synthetic user script preserved across upgrade: ${randomUUID()}\nexport {};\n`,
            workspace, day,dayHash,configHash,editorFontSize:prior.evidence.settings.value,initialGrants,
          };
          writeFileSync(marker.path, marker.value, { flag: "wx", mode: 0o600 });
          update.marker=marker;
          r.priorReady = true;
          return;
        }
        if (plan.mode === "update") {
          check(
            marker && workspace === marker.workspace && lstatSync(marker.path).isFile() &&
              readFileSync(marker.path, "utf8") === marker.value,
            "workspace_not_preserved",
          );
          check(await personalHash(marker.day)===marker.dayHash && await personalHash("config.ts")===marker.configHash,"prior_editor_note_or_settings_changed");
          r.sameWorkspaceAndMarker = true;
          r.previousNoteAndSettingsBytesPreserved = true;
        }
        const result = await journeys(marker && plan.mode === "update" ? {
          ...kit,originalData:{date:marker.day.slice("brain/days/".length,-3),noteSha256:marker.dayHash,configSha256:marker.configHash,editorFontSize:marker.editorFontSize},
        } : {...kit, verifyStdinNoteReopen: tag === "v0.1.34" && installed.release.sourceSha === personalSource});
        check(
          result && ["pass", "fail", "blocked"].includes(result.status) &&
            Buffer.byteLength(JSON.stringify(result)) <= 1024 ** 2,
          "invalid_redacted_journey_receipt",
        );
        r.journeys = result;
        if(marker && plan.mode==="update" && result?.checkpoints?.start?.permissions){
          const after=result.checkpoints.start.permissions;
          r.priorGrants=marker.initialGrants;
          r.preExistingGrantsStillReported=marker.initialGrants.filter(k=>after[k]?.status==="granted");
          r.effectAfterHumanConsent="not_exercised";
        }
        checkpoint();
        check(
          result?.status === "pass",
          "installed_journeys_not_passed",
          result?.status === "blocked" ? "blocked" : "fail",
        );
      });
      const marker=update.marker;
      check(r.cleanup === "exact_instance_terminated", "owned_runtime_exit_not_verified");
      if (tag !== plan.tag) {
        await confirmJourneyBytes(r.journeys, personalHash, "prior_installed_data_changed_at_exit");
        // Never overwrite an existing bundle. Archive ONLY our verified, stopped prior install.
        check(
          installed.inode === dirIdentity(APP) &&
            (await verifyBundle(APP, installed.release)).cdhash === installed.cdhash,
          "prior_install_ownership_changed",
        );
        inspect("prior_instance_still_running", ["absent"]);
        moveExclusive(APP, join(installed.localRoot, "owned-prior.app"));
      }
      if (tag === plan.tag) {
        if (marker && plan.mode === "update") {
          check(r.journeys?.evidence?.notes?.reopenedOriginal === true && r.journeys.evidence.notes.sha256 === marker.dayHash &&
            r.journeys.evidence.settings?.readOnly === true && r.journeys.evidence.settings.value === marker.editorFontSize &&
            r.journeys.evidence.settings.revision === marker.configHash && await personalHash(marker.day) === marker.dayHash &&
            await personalHash("config.ts") === marker.configHash, "upgrade_original_data_not_retained");
          r.priorNoteAndSettingReadByUpgradedApp = true;
        }
        r.stage = "installed-settings-statement-boundary";
        checkpoint();
        check(r.cleanup === "exact_instance_terminated" && dirIdentity(APP) === installed.inode &&
          r.journeys?.status === "pass", "settings_boundary_without_installed_journeys");
        const workspace = dirIdentity(KIT);
        const originalData = async (code: string) => {
          check(dirIdentity(KIT) === workspace, code);
          await confirmJourneyBytes(r.journeys, personalHash, code);
        };
        await originalData("settings_boundary_journey_data_mismatch");
        check(tag === "v0.1.34" && installed.release.sourceSha === personalSource, "unknown_personal_release");
        const {preparePersonal, recoverPersonal, boundaries} = await import("../installed-release-personal");
        const dataChecks: string[] = [];
        const guard = async (where: typeof boundaries[number]) => {
          check(boundaries[dataChecks.length] === where, "personal_data_check_out_of_order");
          await originalData("personal_original_data_changed_"+where);
          if (marker) check(await personalHash(marker.day)===marker.dayHash && await personalHash("config.ts")===marker.configHash, "original_previous_installation_changed");
          dataChecks.push(where); checkpoint();
        };
        r.personal = {originalDataSource: marker?"previousInstallation":"sameInstallation", originalDataChecks:dataChecks, first:{}, relaunched:{}};
        const appAuthor = (q:Json) => {
          assertNoninteractiveVisualProbe("installed-release-offline-author");
          const envelope=JSON.stringify(q); check(Buffer.byteLength(envelope)<=8192,"personal_request_oversize");
          const sandbox='(version 1)(allow default)(deny network* appleevent-send process-fork mach-lookup iokit-open)(deny file-write*)(allow file-write* (subpath '+JSON.stringify(KIT)+'))';
          const p=spawnSync('/usr/bin/sandbox-exec',['-p',sandbox,join(APP,EXE),'--personalize'],{input:envelope,encoding:'utf8',cwd:scratch,env:cleanEnv(),maxBuffer:65536,timeout:8000,killSignal:'SIGKILL'});
          check(!p.error&&!p.signal&&p.status===0,"personal_offline_author_failed");
          const v=JSON.parse(p.stdout);check(v?.ok===true&&v.data,"personal_offline_author_unverified");return v.data;
        };
        await guard('journeysProcessExited');
        const saved=await withInstance(inspect,scratch,installed.cdhash,r.personal.first,checkpoint,false,kit=>preparePersonal(kit,appAuthor,guard));
        check(r.personal.first.cleanup==='exact_instance_terminated',"personal_first_exit_not_verified");
        await guard('beforeRelaunch');
        r.personal.result=await withInstance(inspect,scratch,installed.cdhash,r.personal.relaunched,checkpoint,false,kit=>recoverPersonal(kit,appAuthor,saved,guard));
        check(r.personal.relaunched.cleanup==='exact_instance_terminated'&&!sameProcess(r.personal.first.owner,r.personal.relaunched.owner),"personal_relaunch_not_verified");
        await guard('afterFinalExit');
        r.stage = 'installed-read-only-pane-list';
        const panes=await import('../installed-release-panes');
        const commit = installed.release.sourceSha;
        const pin = releasedBunPin(await publicJson(`contents/.bun-version?ref=${commit}`));
        const provider=panes.checkedPaneFiles(KIT);
        const operatorPane=(args:string[])=>{const result=JSON.parse(command('installed_pane_cli_failed',process.execPath,['--config=/dev/null','--no-env-file','--no-install','--no-macros',join(KIT,'bin/scriptkit'),'panes',...args],8000));check(result.success===true&&result.data,'installed_pane_cli_unverified');return result.data;};
        r.paneView={instance:{},source:'synthetic Herdr host; real released mod, pairing and installed app'};
        r.paneView.result=await panes.withPairedPaneFixture(scratch,provider,operatorPane,appAuthor,async scene=>{
          await originalData('pane_setup_changed_original_data');
          return await withInstance(inspect,scratch,installed.cdhash,r.paneView.instance,checkpoint,false,kit=>panes.readInstalledPaneView(kit,scene),true);
        });
        check(r.paneView.instance.cleanup==='exact_instance_terminated'&&operatorPane(['status']).state==='notPaired','pane_disposal_unverified');
        await originalData('pane_read_changed_original_data');r.paneView.pairingRemoved=true;
        if (marker && plan.mode==='update') receipt.verifiedUpdate=true;
        r.stage = "installed-settings-statement-boundary";
        const sources = Object.fromEntries(await Promise.all(settingsOwners.map(async name => [name,
          releasedSettingsSource(name, await publicJson(`contents/scripts/${name}?ref=${commit}`)),
        ]))) as SettingsSources;
        const runHelper = ({args, cwd, env}: SettingsInvocation) => {
          assertNoninteractiveVisualProbe("installed-release-workspace-child");
          const p = spawnSync("/usr/bin/sandbox-exec", args, { cwd, env, timeout: 8_000, killSignal: "SIGKILL",
            maxBuffer: 65536, stdio: ["ignore", "pipe", "pipe"] });
          check(!p.error && !p.signal, "installed_workspace_challenge_process_failed");
          return { status: p.status, stdout: p.stdout, stderr: p.stderr };
        };
        let result;
        try {
          result = challengeSettings(join(KIT, "sdk"), scratch, sources, runHelper);
        } finally { await originalData("settings_challenge_changed_original_data"); }
        r.settingsStatementBoundary = { ...result, sourceCommit: commit, journeyNoteAndSettingsUnchangedDuringCheck: true };
        r.stage = "installed-command-link";
        checkpoint();
        const cli = releasedWorkspaceSource("mcp-cli.ts", await publicJson(`contents/scripts/mcp-cli.ts?ref=${commit}`));
        let commandResult;
        try {
          commandResult = challengeCommand(KIT, scratch, cli, pin, runHelper);
        } finally { await originalData("command_challenge_changed_original_data"); }
        r.commandLink = { ...commandResult, sourceCommit: commit, journeyNoteAndSettingsUnchangedDuringCheck: true };
        r.stage = "installed-settings-sdk-scope";
        checkpoint();
        const entry = releasedWorkspaceSource(sdkEntry, await publicJson(`contents/scripts/${sdkEntry}?ref=${commit}`));
        let sdkResult;
        try {
          sdkResult = await challengeSettingsSdk(join(KIT, "sdk"), scratch, { ...sources, [sdkEntry]: entry }, pin,
            async ({ cwd, env, args }) => {
              await originalData("sdk_challenge_changed_original_data");
              assertNoninteractiveVisualProbe("installed-release-settings-sdk-child");
              // The SDK itself starts a CLI child. Wait for the whole bounded group, including on failure.
              const child = await spawnOwnedProcess({ argv: ["/usr/bin/sandbox-exec", ...args], cwd, env,
                timeoutMs: 8_000, maxOutputBytes: 65_536 });
              try {
                child.stdin.end();
                const [status, out, err] = await Promise.all([child.exited, new Response(child.stdout).arrayBuffer(),
                  new Response(child.stderr).arrayBuffer()]);
                return { status, stdout: Buffer.from(out), stderr: Buffer.from(err) };
              } finally {
                const cleanup = await child.close();
                check(cleanup.closed && cleanup.processGroupExited && !cleanup.survivors.length &&
                  !cleanup.failureCodes.length, "sdk_challenge_group_not_closed");
              }
            });
        } finally { await originalData("sdk_challenge_changed_original_data"); }
        r.settingsSdkScope = { ...sdkResult, sourceCommit: commit, journeyNoteAndSettingsUnchangedDuringCheck: true };
      }
      r.stage = "complete";
      checkpoint();
    }
    receipt.status = "pass";
  } catch (error) {
    Object.assign(receipt, reporting(error));
  } finally {
    try {
      checkpoint();
    } finally {
      closeSync(fd);
    }
  }
  return receipt;
}
