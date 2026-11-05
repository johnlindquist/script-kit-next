import { expect, test } from "bun:test";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  mkdirSync,
  mkdtempSync,
  readdirSync,
  readFileSync,
  realpathSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";
import {
  challengeSettings,
  releasedSettingsSource,
  type SettingsInvocation,
  settingsOwners,
  type SettingsSources,
} from "./lib/installed-release-settings";

// A copied source layout and a real constrained Bun process, NOT an installation or the hosted release API.
function fixture<T>(
  fn: (f: { sdk: string; scratch: string; original: string; note: string; sources: SettingsSources }) => T,
) {
  const root = realpathSync.native(mkdtempSync(join(tmpdir(), "kit-installed-settings-test-")));
  try {
    const sdk = join(root, "workspace/sdk"),
      scratch = join(root, "scratch"),
      original = join(root, "workspace/config.ts"),
      note = join(root, "workspace/day.md");
    mkdirSync(sdk, { recursive: true, mode: 0o700 });
    mkdirSync(scratch, { mode: 0o700 });
    writeFileSync(original, "original setting\n", { mode: 0o600 });
    writeFileSync(note, "original day\n", { mode: 0o600 });
    const sources = Object.fromEntries(settingsOwners.map((name) => {
      const bytes = readFileSync(join(import.meta.dir, "..", name));
      writeFileSync(join(sdk, name), bytes, { mode: 0o600 });
      return [name, releasedSettingsSource(name, apiRow(name, bytes))];
    })) as SettingsSources;
    const result = fn({ sdk, scratch, original, note, sources });
    expect(readFileSync(original, "utf8")).toBe("original setting\n");
    expect(readFileSync(note, "utf8")).toBe("original day\n");
    return result;
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
}
function apiRow(name: string, bytes: Buffer) {
  return {
    type: "file",
    path: `scripts/${name}`,
    encoding: "base64",
    size: bytes.length,
    content: bytes.toString("base64"),
    sha: createHash("sha1").update(`blob ${bytes.length}\0`).update(bytes).digest("hex"),
  };
}
function child(input: SettingsInvocation) {
  const p = spawnSync("/usr/bin/sandbox-exec", input.args, {
    cwd: input.cwd,
    env: input.env,
    timeout: 8_000,
    killSignal: "SIGKILL",
    maxBuffer: 65536,
    stdio: ["ignore", "pipe", "pipe"],
  });
  if (p.error || p.signal) throw Error("fixture_child_failed");
  return { status: p.status, stdout: p.stdout, stderr: p.stderr };
}
const mac = test.skipIf(process.platform !== "darwin");
mac(
  "actual CLI: matched edits and resets leave invalid originals alone; a continued type saves and reads back",
  () =>
    fixture((f) => {
      let calls = 0;
      const r = challengeSettings(f.sdk, f.scratch, f.sources, (input) => {
        calls++;
        return child(input);
      });
      expect(calls).toBe(10);
      expect(r.checks).toEqual(["as", "satisfies"].flatMap((operator) =>
        ["save", "fullReset", "validSave", "separateStatementsReset"].map((operation) => ({
          operator,
          operation,
          outcome: operation === "validSave" ? "savedAndReadBack" : "refusedWithoutChangingFiles",
        }))
      ));
      expect(r.modules).toEqual(f.sources);
      expect(JSON.stringify(r)).not.toContain(f.scratch);
    }),
);

test("source substitution or a bad upstream blob stops before the first case or subprocess", () =>
  fixture((f) => {
    const row = apiRow("config-cli.ts", Buffer.from("not the release"));
    expect(() => releasedSettingsSource("config-cli.ts", { ...row, sha: "0".repeat(40) })).toThrow(
      "settings_source_blob_mismatch",
    );
    expect(() => releasedSettingsSource("config-document.ts", row)).toThrow("settings_source_unavailable");
    writeFileSync(join(f.sdk, "config-cli.ts"), "console.log('substituted')");
    let called = false;
    expect(() =>
      challengeSettings(f.sdk, f.scratch, f.sources, () => {
        called = true;
        throw Error("unexpected");
      })
    ).toThrow("installed_settings_source_mismatch");
    expect(called).toBe(false);
    expect(readdirSync(f.scratch)).toEqual([]);
  }));
mac(
  "a genuine stale-revision refusal is not an annotation refusal; the verifier stops",
  () =>
    fixture((f) => {
      let calls = 0;
      expect(() =>
        challengeSettings(f.sdk, f.scratch, f.sources, (input) => {
          calls++;
          const actual = child({ ...input, args: [...input.args.slice(0, -1), "0".repeat(64)] });
          expect(JSON.parse(actual.stdout.toString()).errorCode).toBe("stale_revision");
          return actual;
        })
      ).toThrow("settings_annotation_refusal_unverified");
      expect(calls).toBe(1);
    }),
);
mac("a correct refusal cannot hide side effects, and a claimed save cannot stand in for the bytes", () => {
  for (const fault of ["backup", "lock", "lyingSave", "wrongReadback"]) {
    fixture((f) => {
      let calls = 0;
      expect(() =>
        challengeSettings(f.sdk, f.scratch, f.sources, (input) => {
          calls++;
          if (fault === "lyingSave" && calls === 3) {
            return {
              status: 0,
              stdout: Buffer.from('{"success":true,"data":{"value":27}}'),
              stderr: Buffer.alloc(0),
            };
          }
          const result = child(input);
          if (fault === "backup") writeFileSync(input.env.SCRIPT_KIT_CONFIG_PATH + ".bak", "changed");
          if (fault === "lock") mkdirSync(input.env.SCRIPT_KIT_CONFIG_PATH + ".settings-lock");
          if (fault === "wrongReadback" && calls === 4) {
            const v = JSON.parse(result.stdout.toString());
            v.data.path = f.original;
            result.stdout = Buffer.from(JSON.stringify(v));
          }
          return result;
        })
      ).toThrow(
        fault === "wrongReadback"
          ? "settings_annotation_readback_unverified"
          : "settings_challenge_disk_mismatch",
      );
      expect(calls).toBe(fault === "wrongReadback" ? 4 : fault === "lyingSave" ? 3 : 1);
    });
  }
});
mac(
  "even a hostile fixture helper cannot open the original note or settings for writing",
  () =>
    fixture((f) => {
      // Own synthetic files only. The caller explicitly enrolls this fixture code, not production Kit.
      const name = "config-cli.ts",
        bytes = Buffer.from(
          `import fs from 'node:fs';\nconst out=[];for(const path of ${
            JSON.stringify([f.original, f.note])
          }) {try{const fd=fs.openSync(path,'r+');fs.closeSync(fd);out.push('opened')}catch(e){out.push(e.code)}}console.log(JSON.stringify(out));`,
        );
      writeFileSync(join(f.sdk, name), bytes);
      f.sources[name] = releasedSettingsSource(name, apiRow(name, bytes));
      let observed: string[] = [];
      expect(() =>
        challengeSettings(f.sdk, f.scratch, f.sources, (input) => {
          const r = child(input);
          observed = JSON.parse(r.stdout.toString());
          return r;
        })
      ).toThrow("settings_challenge_unverified_reply");
      expect(observed).toEqual(["EPERM", "EPERM"]);
    }),
);
