// The normal installed app has already exited. Challenge only its installed settings CLI,
// on new disposable files. Process creation belongs to the attested host, not this module.
import { createHash, randomUUID } from "node:crypto";
import {
  closeSync,
  constants as F,
  fstatSync,
  lstatSync,
  mkdtempSync,
  openSync,
  readdirSync,
  readSync,
  realpathSync,
  writeFileSync,
} from "node:fs";
import { dirname, isAbsolute, join, relative } from "node:path";
import { check, type Json, SmokeFault } from "./installed-release-contract";

export const settingsOwners = ["config-cli.ts", "config-document.ts", "config-schema.ts"] as const;
type Owner = (typeof settingsOwners)[number];
type Identity = { gitBlob: string; sha256: string; bytes: number };
export type SettingsSources = Record<Owner, Identity>;
const sha256 = (bytes: Buffer | string) => createHash("sha256").update(bytes).digest("hex");
function identity(bytes: Buffer): Identity {
  return {
    gitBlob: createHash("sha1").update(`blob ${bytes.length}\0`).update(bytes).digest("hex"),
    sha256: sha256(bytes),
    bytes: bytes.length,
  };
}
/** From GitHub contents/scripts/<name>?ref=<peeled release commit>, not the QA checkout. */
export function releasedWorkspaceSource(name: Owner | "kit-sdk-config.ts" | "mcp-cli.ts", row: Json): Identity {
  const kind = name === "mcp-cli.ts" ? "command" : "settings";
  check(
    row?.path === `scripts/${name}` && row.type === "file" && row.encoding === "base64" &&
      typeof row.content === "string" && Number.isSafeInteger(row.size) && row.size > 0 &&
      row.size <= 512 * 1024 && row.content.length <= 768 * 1024,
    `${kind}_source_unavailable`,
  );
  const result = identity(Buffer.from(row.content, "base64"));
  check(row.size === result.bytes && row.sha === result.gitBlob, `${kind}_source_blob_mismatch`);
  return result;
}
export function releasedSettingsSource(name: Owner, row: Json): Identity {
  return releasedWorkspaceSource(name, row);
}
export function verifySettingsModule(
  sdk: string,
  name: Owner | "kit-sdk-config.ts",
  expected?: Identity,
) {
  let actual: Identity;
  try {
    actual = identity(directBytes(join(sdk, name)));
  } catch (e) {
    if (e instanceof SmokeFault) throw e;
    throw new SmokeFault("installed_settings_source_unreadable", "blocked");
  }
  check(
    expected && actual.bytes === expected.bytes &&
      actual.gitBlob === expected.gitBlob &&
      actual.sha256 === expected.sha256,
    "installed_settings_source_mismatch",
  );
}

/** Always compare to what the running app read back. A new baseline taken after quit could conceal loss. */
export async function confirmJourneyBytes(
  journey: Json,
  hashFile: (relative: string) => Promise<string>,
  code: string,
) {
  const n = journey?.evidence?.notes, s = journey?.evidence?.settings;
  const digest = (value: unknown) => typeof value === "string" && /^[a-f0-9]{64}$/.test(value);
  check(
    journey?.status === "pass" && typeof n?.date === "string" &&
      /^\d{4}-\d{2}-\d{2}$/.test(n.date) &&
      digest(n.sha256) && s?.key === "editorFontSize" &&
      s.revisionMatchedDisk === true && digest(s.revision) &&
      await hashFile("brain/days/" + n.date + ".md") === n.sha256 &&
      await hashFile("config.ts") === s.revision,
    code,
  );
}
// Shared with the post-exit command check. These reads never execute a workspace file.
export { directBytes, directory, identity, inside };
export type { Identity as SourceIdentity };
function directBytes(file: string, cap = 512 * 1024): Buffer {
  const fd = openSync(file, F.O_RDONLY | F.O_NONBLOCK | F.O_NOFOLLOW);
  try {
    const s = fstatSync(fd);
    check(
      s.isFile() && s.nlink === 1 && s.uid === process.getuid!() && !(s.mode & 0o022) && s.size <= cap,
      "unverified_settings_file",
    );
    const buffer = Buffer.alloc(s.size + 1);
    let n = 0;
    while (n < buffer.length) {
      const got = readSync(fd, buffer, n, buffer.length - n, n);
      if (!got) break;
      n += got;
    }
    const end = lstatSync(file);
    check(
      n === s.size && end.isFile() && end.dev === s.dev && end.ino === s.ino && end.nlink === 1 &&
        end.uid === s.uid && end.mode === s.mode && end.size === n,
      "settings_file_changed",
    );
    return buffer.subarray(0, n);
  } finally {
    closeSync(fd);
  }
}
const inside = (parent: string, path: string) => {
  const r = relative(parent, path);
  return !r || (r !== ".." && !r.startsWith("../") && !isAbsolute(r));
};
function directory(path: string, privateOnly: boolean): string {
  const s = lstatSync(path);
  check(
    realpathSync.native(path) === path && s.isDirectory() && s.uid === process.getuid!() &&
      !(s.mode & (privateOnly ? 0o077 : 0o022)),
    "unverified_settings_directory",
  );
  return `${s.dev}:${s.ino}`;
}
export type SettingsInvocation = { args: string[]; cwd: string; env: Record<string, string> };
export type SettingsReply = { status: number | null; stdout: Buffer; stderr: Buffer };
// The current note, installed config, SDK and command are never writable by a challenge.
export function workspaceProbePolicy(
  dir: string,
  bun: string,
  reads: string[],
  shellCommand?: string,
  writes = [dir],
): string {
  const q = JSON.stringify,
    literal = (p: string) => `(literal ${q(p)})`,
    subpath = (p: string) => `(subpath ${q(p)})`;
  const parents = new Set<string>();
  for (let p of [dir, dirname(bun), ...reads.map((p) => dirname(p))]) {
    for (;;) {
      parents.add(p);
      const next = dirname(p);
      if (p === next) break;
      p = next;
    }
  }
  const exec = shellCommand ? [bun, "/bin/sh", "/bin/bash", "/usr/bin/env", shellCommand] : [bun];
  return `(version 1)(allow default)(deny network*)(deny mach-lookup)(deny iokit-open)(deny appleevent-send)` +
    `(deny file-read* ${
      [
        "/Users",
        "/Volumes",
        "/private/tmp",
        "/private/var/folders",
        "/private/var/root",
        "/opt",
        "/usr/local",
        "/Library/Keychains",
      ].map(subpath).join(" ")
    })` +
    `(allow file-read-metadata ${Array.from(parents, literal).join(" ")})` +
    `(allow file-read* ${subpath(dir)} ${[bun, ...reads].map(literal).join(" ")})` +
    `(deny file-write*)(allow file-write* ${
      writes.map(subpath).join(" ")
    } (literal "/dev/null"))(deny process-exec)(allow process-exec ${exec.map(literal).join(" ")})`;
}
function invocation(sdk: string, bun: string, dir: string, args: string[]): SettingsInvocation {
  const profile = workspaceProbePolicy(dir, bun, [sdk, ...settingsOwners.map((n) => join(sdk, n))]);
  return {
    args: [
      "-p",
      profile,
      bun,
      "--config=/dev/null",
      "--no-env-file",
      "--no-install",
      join(sdk, settingsOwners[0]),
      ...args,
    ],
    cwd: dir,
    env: {
      HOME: dir,
      TMPDIR: dir + "/",
      PATH: "/usr/bin:/bin",
      LANG: "C",
      SK_PATH: join(dir, "unused"),
      SCRIPT_KIT_CONFIG_PATH: join(dir, "config.ts"),
      SCRIPT_KIT_NONINTERACTIVE: "1",
      BUN_RUNTIME_TRANSPILER_CACHE_PATH: "0",
      DO_NOT_TRACK: "1",
    },
  };
}
const boundary =
  /^Unsupported config\.ts at offset \d+: start 'as' or 'satisfies' on the same line as the exported object's closing brace; edit this file manually$/;
/** The host supplies its private scratch directory. Each invocation group gets a new, empty
 * case directory; neither the runner nor the probe accepts a user's config or backup as a target. */
export function challengeSettings(
  sdk: string,
  scratch: string,
  sources: SettingsSources,
  run: (input: SettingsInvocation) => SettingsReply,
) {
  const workspace = dirname(sdk), sdkId = directory(sdk, false), scratchId = directory(scratch, true);
  check(!inside(workspace, scratch) && !inside(scratch, workspace), "settings_homes_overlap");
  const bun = realpathSync.native(process.execPath);
  check(!!process.versions.bun && lstatSync(bun).isFile(), "settings_runtime_unavailable");
  const verifySources = () => {
    check(
      directory(sdk, false) === sdkId && directory(scratch, true) === scratchId,
      "settings_directory_changed",
    );
    for (const name of settingsOwners) verifySettingsModule(sdk, name, sources[name]);
  };
  verifySources();
  const checks: { operator: string; operation: string; outcome: string }[] = [];
  for (const operator of ["as", "satisfies"]) {
    for (const operation of ["save", "fullReset", "validSave", "separateStatementsReset"]) {
      const dir = mkdtempSync(join(scratch, "settings-case-")), dirId = directory(dir, true);
      const config = join(dir, "config.ts"), backup = config + ".bak", privateComment = randomUUID();
      const valid = operation === "validSave";
      const ending = valid
        ? ` ${operator}\nConfig`
        : operation === "separateStatementsReset"
        ? `\n${operator}\nConfig`
        : `\n${operator} Config`;
      const original = `export default {\n  editorFontSize: 16, // ${privateComment}\n}${ending};\n`;
      const saved = original.replace("editorFontSize: 16,", "editorFontSize: 27,"),
        oldBackup = `// ${randomUUID()}\n`;
      writeFileSync(config, original, { flag: "wx", mode: 0o600 });
      writeFileSync(backup, oldBackup, { flag: "wx", mode: 0o600 });
      const call = (args: string[], expectedBytes: string) => {
        verifySources();
        const answer = run(invocation(sdk, bun, dir, args));
        check(
          directory(dir, true) === dirId && directBytes(config, 8192).equals(Buffer.from(expectedBytes)) &&
            directBytes(backup, 8192).equals(Buffer.from(oldBackup)) &&
            readdirSync(dir).sort().join() === "config.ts,config.ts.bak",
          "settings_challenge_disk_mismatch",
        );
        check(
          answer.status !== null && answer.stderr.length === 0 && answer.stdout.length <= 65536 &&
            !answer.stdout.includes(privateComment),
          "settings_challenge_unverified_reply",
        );
        let body: Json;
        try {
          body = JSON.parse(new TextDecoder("utf-8", { fatal: true }).decode(answer.stdout));
        } catch {
          throw new SmokeFault("settings_challenge_unverified_reply");
        }
        check(
          body && typeof body === "object" && !Array.isArray(body),
          "settings_challenge_unverified_reply",
        );
        return { status: answer.status, body };
      };
      const edit = (operation === "fullReset" || operation === "separateStatementsReset")
        ? ["reset"]
        : ["set-json", "editorFontSize", "27"];
      const result = call([...edit, "--if-revision", sha256(original)], valid ? saved : original);
      if (!valid) {
        check(
          result.status === 1 && result.body.success === false && !Object.hasOwn(result.body, "errorCode") &&
            typeof result.body.error === "string" && boundary.test(result.body.error),
          "settings_annotation_refusal_unverified",
        );
      } else {
        const got = result.body.data;
        check(
          result.status === 0 && result.body.success === true && got?.key === "editorFontSize" &&
            got.value === 27 && got.revision === sha256(saved),
          "settings_annotation_save_unverified",
        );
        const read = call(["get", "editorFontSize"], saved), v = read.body.data;
        check(
          read.status === 0 && read.body.success === true && v?.path === config && v.exists === true &&
            v.key === "editorFontSize" && v.isDefault === false && v.value === 27 &&
            v.revision === sha256(saved),
          "settings_annotation_readback_unverified",
        );
      }
      checks.push({
        operator,
        operation,
        outcome: valid ? "savedAndReadBack" : "refusedWithoutChangingFiles",
      });
    }
  }
  verifySources();
  return {
    status: "pass",
    scope: "installed-workspace-cli-on-separate-synthetic-files",
    bunVersion: process.versions.bun,
    modules: sources,
    checks,
  };
}
