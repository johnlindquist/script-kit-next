import { expect, test } from "bun:test";
import { spawnSync } from "node:child_process";
import {
  mkdirSync,
  mkdtempSync,
  readdirSync,
  readFileSync,
  realpathSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { releasedBunPin } from "./lib/installed-release-command";
import { challengeSettingsSdk, sdkEntry, type SdkSources } from "./lib/installed-release-settings-sdk";
import {
  confirmJourneyBytes,
  identity,
  releasedWorkspaceSource,
  type SettingsInvocation,
  settingsOwners,
} from "./lib/installed-release-settings";

const mac = test.skipIf(process.platform !== "darwin");
const modules = [...settingsOwners, sdkEntry] as const;
const contents = (path: string, bytes: Buffer) => ({
  path,
  type: "file",
  encoding: "base64",
  size: bytes.length,
  sha: identity(bytes).gitBlob,
  content: bytes.toString("base64"),
});
// A synthetic distribution, not an installation. Only the per-case directory may be written by the SDK or its CLI child.
async function fixture(
  fn: (
    f: {
      sdk: string;
      scratch: string;
      note: string;
      original: string;
      sources: SdkSources;
      pin: ReturnType<typeof releasedBunPin>;
    },
  ) => Promise<void>,
) {
  const root = realpathSync.native(
      mkdtempSync(join(tmpdir(), "kit-sdk-scope-")),
    ),
    sdk = join(root, "Kit #1 資料/sdk"),
    scratch = join(root, "checks"),
    note = join(sdk, "../day.md"),
    original = join(sdk, "../config.ts");
  try {
    mkdirSync(sdk, { recursive: true, mode: 0o700 });
    mkdirSync(scratch, { mode: 0o700 });
    const paths = [original, original + ".bak", note],
      saved = ["original setting", "original backup", "dated note"];
    paths.forEach((p, i) => writeFileSync(p, saved[i]!, { mode: 0o600 }));
    const sources = Object.fromEntries(
      modules.map((name) => {
        const b = readFileSync(new URL("../" + name, import.meta.url));
        writeFileSync(join(sdk, name), b, { mode: 0o600 });
        return [
          name,
          releasedWorkspaceSource(name, contents("scripts/" + name, b)),
        ];
      }),
    ) as SdkSources;
    const pin = releasedBunPin(
      contents(
        ".bun-version",
        readFileSync(new URL("../../.bun-version", import.meta.url)),
      ),
    );
    try {
      await fn({ sdk, scratch, note, original, sources, pin });
    } finally {
      expect(paths.map((p) => readFileSync(p, "utf8"))).toEqual(saved);
    }
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
}
function child({ args, cwd, env }: SettingsInvocation) {
  const p = spawnSync("/usr/bin/sandbox-exec", args, {
    cwd,
    env,
    timeout: 8000,
    killSignal: "SIGKILL",
    maxBuffer: 65536,
    stdio: ["ignore", "pipe", "pipe"],
  });
  if (p.error || p.signal) throw Error("fixture_child_not_verified");
  return { status: p.status, stdout: p.stdout, stderr: p.stderr };
}
mac(
  "the real settings SDK keeps missing arguments out of both the save and reset paths",
  async () => {
    await fixture(async (f) => {
      const r = await challengeSettingsSdk(
        f.sdk,
        f.scratch,
        f.sources,
        f.pin,
        async (input) => child(input),
      );
      expect(r).toMatchObject({
        status: "pass",
        modules: f.sources,
        runtimePinGitBlob: f.pin.gitBlob,
        competingEditReadBack: true,
        staleEditRefused: true,
        missingRevisionKeptNewerValue: true,
        missingKeyKeptBothSettings: true,
        keyedResetKeptNeighbor: true,
        staleResetAllRefused: true,
        explicitResetAllReadBack: true,
        originalScratchBackupUnchanged: true,
      });
      expect(JSON.stringify(r)).not.toContain(f.scratch);
    });
  },
);
test("a substituted SDK entry, any substituted dependency, or a wrong Bun pin stops before a case or child exists", async () => {
  await fixture(async (f) => {
    const never = async () => {
      throw Error("unexpected child");
    };
    for (const n of modules) {
      await expect(
        challengeSettingsSdk(
          f.sdk,
          f.scratch,
          { ...f.sources, [n]: { ...f.sources[n], sha256: "0".repeat(64) } },
          f.pin,
          never,
        ),
      ).rejects.toThrow("installed_settings_source_mismatch");
    }
    await expect(
      challengeSettingsSdk(f.sdk, f.scratch, f.sources, {
        ...f.pin,
        version: "0.0.1",
      }, never),
    ).rejects.toThrow("sdk_runtime_pin_mismatch");
    expect(readdirSync(f.scratch)).toEqual([]);
  });
});
mac(
  "a generic error or an altered backup is not evidence that the SDK protected the settings",
  async () => {
    for (const fault of ["refusal", "backup"]) {
      await fixture(async (f) => {
        let touched = false;
        await expect(
          challengeSettingsSdk(
            f.sdk,
            f.scratch,
            f.sources,
            f.pin,
            async (input) => {
              const r = child(input);
              if (
                input.args.at(-1)!.includes(
                  'await config.set("editorFontSize", 27, missing.revision)',
                )
              ) {
                touched = true;
                if (fault === "backup") {
                  writeFileSync(
                    input.env.SCRIPT_KIT_CONFIG_PATH + ".bak",
                    "replaced",
                  );
                } else {r.stdout = Buffer.from(
                    JSON.stringify({
                      outcome: "refused",
                      reason: "unexpected",
                      bun: f.pin.version,
                    }),
                  );}
              }
              return r;
            },
          ),
        ).rejects.toThrow(
          fault === "backup" ? "sdk_backup_or_lock_changed" : "sdk_missingRevision_not_refused",
        );
        expect(touched).toBe(true);
      });
    }
  },
);
mac(
  "even an attempted SDK child write cannot reach the original note, settings or backup",
  async () => {
    await fixture(async (f) => {
      await expect(
        challengeSettingsSdk(
          f.sdk,
          f.scratch,
          f.sources,
          f.pin,
          async (input) => {
            const targets = [f.note, f.original, f.original + ".bak"];
            const r = child({
              ...input,
              args: [
                ...input.args.slice(0, -1),
                `import {writeFileSync} from 'node:fs';
        console.log(JSON.stringify(${
                  JSON.stringify(targets)
                }.map(p=>{try{writeFileSync(p,'wrong');return 'changed'}catch(e){return e.code}})));`,
              ],
            });
            expect(r.status).toBe(0);
            expect(JSON.parse(r.stdout.toString())).toEqual([
              "EPERM",
              "EPERM",
              "EPERM",
            ]);
            throw Error("end of containment fixture");
          },
        ),
      ).rejects.toThrow("end of containment fixture");
    });
  },
);
test("the post-exit comparison never takes a replacement config as its baseline", async () => {
  await fixture(async (f) => {
    const note = identity(readFileSync(f.note)).sha256,
      revision = identity(readFileSync(f.original)).sha256;
    const evidence = {
      notes: { date: "2030-01-03", sha256: note },
      settings: { key: "editorFontSize", revisionMatchedDisk: true, revision },
    };
    const hash = async (p: string) => identity(readFileSync(p === "config.ts" ? f.original : f.note)).sha256;
    await confirmJourneyBytes(
      { status: "pass", evidence },
      hash,
      "journey_changed",
    );
    for (const invalid of [undefined, "", "0".repeat(64)]) {
      await expect(
        confirmJourneyBytes(
          {
            status: "pass",
            evidence: {
              ...evidence,
              settings: { ...evidence.settings, revision: invalid },
            },
          },
          hash,
          "journey_changed",
        ),
      ).rejects.toThrow("journey_changed");
    }
    const disposable = join(f.scratch, "changed-config.ts");
    writeFileSync(disposable, "data replaced during quit");
    await expect(
      confirmJourneyBytes(
        { status: "pass", evidence },
        async (p) => p === "config.ts" ? identity(readFileSync(disposable)).sha256 : hash(p),
        "journey_changed",
      ),
    ).rejects.toThrow("journey_changed");
  });
});
