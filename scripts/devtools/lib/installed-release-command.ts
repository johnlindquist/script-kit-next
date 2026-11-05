// Run only after the installed app exits. Source comes from the peeled public commit;
// the host owns subprocess creation. The workspace and its normal PATH are never test targets.
import {
  lstatSync,
  mkdirSync,
  mkdtempSync,
  readdirSync,
  readlinkSync,
  realpathSync,
  statSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { basename, dirname, join, relative, resolve } from "node:path";
import { check, type Json, SmokeFault } from "./installed-release-contract";
import {
  directBytes,
  directory,
  identity,
  inside,
  type SettingsInvocation,
  type SettingsReply,
  type SourceIdentity,
  workspaceProbePolicy,
} from "./installed-release-settings";

const utf8 = (b: Buffer) => new TextDecoder("utf-8", { fatal: true }).decode(b);
/** The workflow provisions Bun; the independent release pin identifies the version to test. */
export function releasedBunPin(row: Json) {
  check(
    row?.path === ".bun-version" && row.type === "file" &&
      row.encoding === "base64" &&
      Number.isSafeInteger(row.size) && row.size > 0 && row.size <= 32 &&
      typeof row.content === "string" && row.content.length <= 64,
    "command_runtime_pin_missing",
  );
  const bytes = Buffer.from(row.content, "base64"),
    id = identity(bytes),
    pin = utf8(bytes).replace(/\r?\n$/, "");
  check(
    id.gitBlob === row.sha && id.bytes === row.size &&
      /^\d+\.\d+\.\d+$/.test(pin) && pin === process.versions.bun,
    "command_runtime_pin_mismatch",
  );
  return { version: pin, gitBlob: id.gitBlob };
}
const stamp = (path: string) => {
  const s = lstatSync(path, { bigint: true });
  return [s.dev, s.ino, s.mode, s.uid, s.nlink, s.size, s.mtimeNs, s.ctimeNs]
    .join(":");
};
const linkMark = (p: string) => {
  check(
    lstatSync(p, { throwIfNoEntry: false })?.isSymbolicLink(),
    "command_link_missing",
  );
  return stamp(p) + ":" + readlinkSync(p);
};
const sameFile = (a: string, b: string) => {
  const x = statSync(a, { bigint: true }), y = statSync(b, { bigint: true });
  return x.isFile() && y.isFile() && x.dev === y.dev && x.ino === y.ino;
};

/** Build the misleading link entirely under our case directory, even when the app and test
 * directories have no common ancestor except /. No alias or decoy is created in the install. */
function conflictFixture(caseDir: string, source: string) {
  const bin = join(caseDir, "other-bin"), fake = join(caseDir, "decoy");
  mkdirSync(bin, { mode: 0o700 });
  const rel = relative(bin, source), parts = rel.split("/");
  const up = parts.findIndex((part) => part !== "..");
  check(
    up > 0 && up < 64 &&
      parts.slice(up).every((p) => p && p !== "." && p !== ".."),
    "command_fixture_location",
  );
  const tower = join(fake, ...Array<string>(up + 1).fill("down"));
  const inert = join(fake, ...parts.slice(up)),
    through = join(bin, "through"),
    target = join(bin, "scriptkit");
  check(
    inside(fake, inert) && !inside(tower, inert),
    "command_fixture_location",
  );
  mkdirSync(tower, { recursive: true, mode: 0o700 });
  mkdirSync(dirname(inert), { recursive: true, mode: 0o700 });
  const bytes = Buffer.from("#!/bin/sh\nexit 73\n"); // An inert executable; never run it.
  writeFileSync(inert, bytes, { flag: "wx", mode: 0o700 });
  symlinkSync(tower, through);
  symlinkSync("through/../" + rel, target);
  const mark = linkMark(target), via = linkMark(through), file = stamp(inert);
  function unchanged() {
    check(
      resolve(bin, readlinkSync(target)) === source &&
        realpathSync.native(target) === inert &&
        sameFile(target, inert) && !sameFile(target, source) &&
        mark === linkMark(target) &&
        via === linkMark(through) && stamp(inert) === file &&
        directBytes(inert, 128).equals(bytes),
      "command_conflict_fixture_changed",
    );
  }
  unchanged();
  return { target, unchanged };
}

export function challengeCommand(
  workspace: string,
  scratch: string,
  expected: SourceIdentity,
  pin: ReturnType<typeof releasedBunPin>,
  run: (input: SettingsInvocation) => SettingsReply,
) {
  const bin = join(workspace, "bin"), source = join(bin, "scriptkit");
  const wsId = directory(workspace, false),
    scratchId = directory(scratch, true);
  check(
    !inside(workspace, scratch) && !inside(scratch, workspace),
    "command_homes_overlap",
  );
  check(
    lstatSync(bin, { throwIfNoEntry: false }) &&
      lstatSync(source, { throwIfNoEntry: false }),
    "installed_command_missing",
  );
  const binId = directory(bin, false);
  function checkSource() {
    check(
      directory(workspace, false) === wsId && directory(bin, false) === binId &&
        directory(scratch, true) === scratchId,
      "command_location_changed",
    );
    const s = lstatSync(source, { throwIfNoEntry: false });
    check(
      s?.isFile() && s.nlink === 1 && s.uid === process.getuid!() &&
        !(s.mode & 0o022) &&
        (s.mode & 0o100) !== 0 && realpathSync.native(source) === source,
      "installed_command_not_direct_executable",
    );
    const got = identity(directBytes(source));
    check(
      got.gitBlob === expected.gitBlob && got.sha256 === expected.sha256 &&
        got.bytes === expected.bytes,
      "installed_command_source_mismatch",
    );
  }
  checkSource();
  const original = stamp(source), bun = realpathSync.native(process.execPath);
  check(
    !!process.versions.bun && pin.version === process.versions.bun &&
      basename(bun) === "bun" && lstatSync(bun).isFile() &&
      !/[:\r\n]/.test(bun + scratch),
    "command_path_runtime_unavailable",
  );
  const bunStamp = stamp(bun);
  const dir = mkdtempSync(join(scratch, "command-case-")),
    commands = join(dir, "commands");
  mkdirSync(commands, { mode: 0o700 });
  const target = join(commands, "scriptkit"),
    caseId = directory(dir, true),
    cmdDirId = directory(commands, true);
  const verify = () => {
    checkSource();
    check(stamp(bun) === bunStamp, "command_runtime_changed");
    check(
      stamp(source) === original && directory(dir, true) === caseId &&
        directory(commands, true) === cmdDirId,
      "installed_command_changed",
    );
  };
  const call = (args: string[], shell = false, writes: string[] = []) => {
    verify();
    const profile = workspaceProbePolicy(
      dir,
      bun,
      [bin, source],
      shell ? source : undefined,
      writes,
    );
    const reply = run({
      args: [
        "-p",
        profile,
        ...(shell ? ["/bin/sh", "-c"] : [
          bun,
          "--config=/dev/null",
          "--no-env-file",
          "--no-install",
          "--no-macros",
          source,
        ]),
        ...args,
      ],
      cwd: dir,
      env: {
        HOME: dir,
        TMPDIR: dir + "/",
        SK_PATH: join(dir, "unused"),
        PATH: commands + ":" + dirname(bun),
        LANG: "C",
        SCRIPT_KIT_NONINTERACTIVE: "1",
        BUN_RUNTIME_TRANSPILER_CACHE_PATH: "0",
        DO_NOT_TRACK: "1",
      },
    });
    verify();
    check(
      reply.status !== null && reply.stdout.length <= 65536 &&
        reply.stderr.length === 0,
      "command_child_unverified",
    );
    return reply;
  };
  const body = (r: SettingsReply) => {
    try {
      return JSON.parse(utf8(r.stdout));
    } catch {
      throw new SmokeFault("command_reply_not_json");
    }
  };
  function installed(r: SettingsReply) {
    const v = body(r);
    check(
      r.status === 0 && v?.success === true &&
        v.data?.command === "scriptkit" &&
        v.data.target === target && v.data.source === source,
      "command_install_receipt_mismatch",
    );
    const mark = linkMark(target);
    check(
      readdirSync(commands).join() === "scriptkit" &&
        realpathSync.native(target) === source &&
        sameFile(target, source),
      "command_link_wrong_executable",
    );
    return mark;
  }
  const direct = call(["--help"]), lines = utf8(direct.stdout).split("\n");
  check(
    direct.status === 0 && lines[0] === "Script Kit command line" &&
      lines.includes("  scriptkit install-command [target-path]"),
    "installed_command_help_unverified",
  );
  const made = installed(call(["install-command", target], false, [commands]));
  const help = call([
    "command -v bun && bun --version && command -v scriptkit && exec scriptkit --help",
  ], true);
  const prefix = Buffer.from([bun, pin.version, target, ""].join("\n"));
  check(
    help.status === 0 && help.stdout.subarray(0, prefix.length).equals(prefix),
    "command_shell_resolution_mismatch",
  );
  check(
    help.stdout.subarray(prefix.length).equals(direct.stdout),
    "command_path_help_mismatch",
  );
  check(linkMark(target) === made, "command_link_replaced");
  const repeated = installed(
    call(
      ['exec scriptkit install-command "$1"', "check-command", target],
      true,
      [commands],
    ),
  );
  check(repeated === made, "command_link_replaced");
  const other = conflictFixture(dir, source);
  const refused = call(["install-command", other.target], false, [
    dirname(other.target),
    join(dir, "decoy"),
  ]); // Absolute installed source only, never the decoy on PATH.
  other.unchanged();
  check(
    linkMark(target) === made && realpathSync.native(target) === source,
    "command_link_replaced",
  );
  const v = body(refused);
  check(
    refused.status === 1 && v?.success === false && v.error ===
        `Refusing to replace a different command at ${other.target}. Choose an unused target path.`,
    "command_conflict_not_refused",
  );
  return {
    status: "pass",
    scope: "installed-workspace-command-private-path",
    source: expected,
    bunVersion: process.versions.bun,
    runtimePinGitBlob: pin.gitBlob,
    shellResolvedPinnedBun: true,
    pathHelpMatchedInstalledCommand: true,
    repeatKeptOriginalLink: true,
    differentExecutableRefusedWithoutReplacement: true,
  };
}
