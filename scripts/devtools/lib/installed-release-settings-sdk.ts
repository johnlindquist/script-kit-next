// Import the installed public SDK, not the verifier checkout. The host owns child lifetimes.
import { randomUUID } from "node:crypto";
import { lstatSync, mkdirSync, mkdtempSync, readdirSync, realpathSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { pathToFileURL } from "node:url";
import { check, type Json, SmokeFault } from "./installed-release-contract";
import type { releasedBunPin } from "./installed-release-command";
import {
  directBytes,
  directory,
  identity,
  inside,
  type SettingsInvocation,
  settingsOwners,
  type SettingsReply,
  type SettingsSources,
  type SourceIdentity,
  verifySettingsModule,
  workspaceProbePolicy,
} from "./installed-release-settings";

export const sdkEntry = "kit-sdk-config.ts";
const owners = [...settingsOwners, sdkEntry] as const;
export type SdkSources =
  & SettingsSources
  & Record<typeof sdkEntry, SourceIdentity>;
const expressions = {
  readEditor: 'config.get("editorFontSize")',
  readTerminal: 'config.get("terminalFontSize")',
  competingEdit: 'config.set("editorFontSize", 21, revision)',
  staleEdit: 'config.set("editorFontSize", 27, revision)',
  missingRevision: 'config.set("editorFontSize", 27, missing.revision)',
  missingKey: "config.reset(missing.key, revision)",
  resetKey: 'config.reset("editorFontSize", revision)',
  resetAll: "config.resetAll(revision)",
} as const;

export async function challengeSettingsSdk(
  sdk: string,
  scratch: string,
  sources: SdkSources,
  pin: ReturnType<typeof releasedBunPin>,
  run: (invocation: SettingsInvocation) => Promise<SettingsReply>,
) {
  const workspace = dirname(sdk),
    wsId = directory(workspace, false),
    sdkId = directory(sdk, false),
    scratchId = directory(scratch, true),
    bun = realpathSync.native(process.execPath);
  check(
    !inside(workspace, scratch) && !inside(scratch, workspace),
    "settings_homes_overlap",
  );
  check(
    pin.version && pin.version === process.versions.bun &&
      lstatSync(bun).isFile(),
    "sdk_runtime_pin_mismatch",
  );
  function verify() {
    check(
      directory(workspace, false) === wsId && directory(sdk, false) === sdkId &&
        directory(scratch, true) === scratchId,
      "settings_directory_changed",
    );
    for (const name of owners) verifySettingsModule(sdk, name, sources[name]);
  }
  verify();
  const dir = mkdtempSync(join(scratch, "sdk-scope-")),
    dirId = directory(dir, true),
    data = join(dir, "settings");
  mkdirSync(data, { mode: 0o700 });
  const dataId = directory(data, true), file = join(data, "config.ts");
  // The nested CLI need not create a macOS transpiler cache under HOME. Only the settings directory is writable.
  const profile = workspaceProbePolicy(
    dir,
    bun,
    [sdk, ...owners.map((n) => join(sdk, n))],
    undefined,
    [data],
  );
  const env = {
    HOME: dir,
    TMPDIR: dir + "/",
    PATH: "/usr/bin:/bin",
    LANG: "C",
    SK_PATH: data,
    SCRIPT_KIT_CONFIG_PATH: file,
    SCRIPT_KIT_NONINTERACTIVE: "1",
    DO_NOT_TRACK: "1",
    BUN_RUNTIME_TRANSPILER_CACHE_PATH: "0",
  };
  const old = 'export default {"editorFontSize":18,"terminalFontSize":20};\n',
    newer = old.replace(":18", ":21");
  const backup = Buffer.from(`// ${randomUUID()}\n`);
  writeFileSync(file, old, { flag: "wx", mode: 0o600 });
  writeFileSync(file + ".bak", backup, { flag: "wx", mode: 0o600 });
  const text = (b: Buffer) => new TextDecoder("utf-8", { fatal: true }).decode(b);
  function bytes() {
    check(
      directory(dir, true) === dirId && directory(data, true) === dataId &&
        readdirSync(data).sort().join() === "config.ts,config.ts.bak" &&
        directBytes(file + ".bak", 4096).equals(backup),
      "sdk_backup_or_lock_changed",
    );
    return text(directBytes(file, 8192));
  }
  const digest = (source: string) => identity(Buffer.from(source)).sha256;
  async function call(
    step: keyof typeof expressions,
    before: string,
    revision = "",
  ) {
    verify();
    check(bytes() === before, "sdk_case_changed_before_call");
    const script = `import { config, StaleConfigRevision } from ${
      JSON.stringify(pathToFileURL(join(sdk, sdkEntry)).href)
    };
const missing = Object.create(null), revision = ${JSON.stringify(revision)};
try {
  const {key,value,path,exists,revision:rev,isDefault} = await ${expressions[step]};
  console.log(JSON.stringify({outcome:"ok",key,value,path,exists,rev,isDefault,bun:process.versions.bun}));
} catch (e) {
  const reason = e instanceof StaleConfigRevision && e.code === "stale_revision" ? "stale" :
    e instanceof Error && e.message === "Expected a settings revision from get or list" ? "missingRevision" :
    e instanceof Error && e.message === "Expected a nonempty settings key" ? "missingKey" : "unexpected";
  console.log(JSON.stringify({outcome:"refused",reason,bun:process.versions.bun})); process.exitCode=1;
}`;
    const r = await run({
      cwd: dir,
      env,
      args: [
        "-p",
        profile,
        bun,
        "--config=/dev/null",
        "--no-install",
        "--no-env-file",
        "--no-macros",
        "-e",
        script,
      ],
    });
    verify();
    const after = bytes();
    check(
      r.stderr.length === 0 && r.stdout.length <= 4096,
      "sdk_call_unverified",
    );
    let reply: Json;
    try {
      reply = JSON.parse(text(r.stdout));
    } catch {
      throw new SmokeFault("sdk_call_unverified");
    }
    check(
      reply?.bun === pin.version &&
        (r.status === 0 && reply.outcome === "ok" &&
            reply.rev === digest(after) ||
          r.status === 1 && reply.outcome === "refused"),
      "sdk_call_unverified",
    );
    return { after, reply };
  }
  async function read(
    step: "readEditor" | "readTerminal",
    source: string,
    inherits: boolean,
    expected?: number,
  ) {
    const key = step === "readEditor" ? "editorFontSize" : "terminalFontSize";
    const { after, reply } = await call(step, source);
    check(
      after === source && reply.outcome === "ok" && reply.key === key &&
        reply.path === file && reply.exists === true &&
        reply.isDefault === inherits && Number.isFinite(reply.value) &&
        (expected === undefined || reply.value === expected),
      "sdk_readback_mismatch",
    );
    return reply.rev as string;
  }
  async function refuses(
    step: "staleEdit" | "missingRevision" | "missingKey" | "resetAll",
    source: string,
    rev: string,
    reason: string,
  ) {
    const { after, reply } = await call(step, source, rev);
    check(
      after === source && reply.outcome === "refused" &&
        reply.reason === reason,
      `sdk_${step}_not_refused`,
    );
  }
  const first = await read("readEditor", old, false, 18);
  const changed = await call("competingEdit", old, first);
  check(
    changed.after === newer && changed.reply.key === "editorFontSize" &&
      changed.reply.value === 21,
    "sdk_competing_edit_not_saved",
  );
  const current = await read("readEditor", newer, false, 21);
  await refuses("staleEdit", newer, first, "stale");
  await refuses("missingRevision", newer, "", "missingRevision");
  await refuses("missingKey", newer, current, "missingKey");
  const keyed = await call("resetKey", newer, current);
  check(
    keyed.reply.key === "editorFontSize" &&
      !keyed.after.includes("editorFontSize") &&
      keyed.after.includes('"terminalFontSize":20'),
    "sdk_reset_changed_neighbor",
  );
  await read("readEditor", keyed.after, true);
  const keyedRevision = await read("readTerminal", keyed.after, false, 20);
  await refuses("resetAll", keyed.after, current, "stale");
  const all = await call("resetAll", keyed.after, keyedRevision);
  check(
    all.reply.outcome === "ok" && all.after.includes("hotkey") &&
      !/editorFontSize|terminalFontSize/.test(all.after),
    "sdk_explicit_reset_not_observed",
  );
  await read("readEditor", all.after, true);
  await read("readTerminal", all.after, true);
  return {
    status: "pass",
    scope: "installed-settings-sdk-on-sacrificial-config",
    modules: sources,
    bunVersion: pin.version,
    runtimePinGitBlob: pin.gitBlob,
    competingEditReadBack: true,
    staleEditRefused: true,
    missingRevisionKeptNewerValue: true,
    missingKeyKeptBothSettings: true,
    keyedResetKeptNeighbor: true,
    staleResetAllRefused: true,
    explicitResetAllReadBack: true,
    originalScratchBackupUnchanged: true,
  };
}
