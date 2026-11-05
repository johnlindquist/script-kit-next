import { expect, test } from "bun:test";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  chmodSync,
  mkdirSync,
  mkdtempSync,
  readdirSync,
  readFileSync,
  readlinkSync,
  realpathSync,
  renameSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { challengeCommand, releasedBunPin } from "./lib/installed-release-command";
import {
  releasedWorkspaceSource,
  type SettingsInvocation,
  type SettingsReply,
  type SourceIdentity,
} from "./lib/installed-release-settings";

function contents(path: string, bytes: Buffer) {
  return {
    type: "file",
    path,
    encoding: "base64",
    size: bytes.length,
    content: bytes.toString("base64"),
    sha: createHash("sha1").update("blob " + bytes.length + "\0").update(bytes)
      .digest("hex"),
  };
}
const cliId = (bytes: Buffer) => releasedWorkspaceSource("mcp-cli.ts", contents("scripts/mcp-cli.ts", bytes));
const pinBytes = () => readFileSync(new URL("../../.bun-version", import.meta.url));
type Runner = (input: SettingsInvocation) => SettingsReply;
const child: Runner = ({ args, cwd, env }) => {
  const p = spawnSync("/usr/bin/sandbox-exec", args, {
    cwd,
    env,
    stdio: ["ignore", "pipe", "pipe"],
    timeout: 5000,
    killSignal: "SIGKILL",
    maxBuffer: 65536,
  });
  if (p.error || p.signal) throw Error("fixture_child_failed");
  return { status: p.status, stdout: p.stdout, stderr: p.stderr };
};
// Actual CLI bytes and shell, but ONLY a synthetic distribution layout. No application is launched.
function fixture(
  fn: (
    f: {
      workspace: string;
      scratch: string;
      source: string;
      original: string;
      note: string;
      run: (
        exec: Runner,
        expected?: SourceIdentity,
      ) => ReturnType<typeof challengeCommand>;
    },
  ) => void,
) {
  const root = realpathSync.native(
    mkdtempSync(join(tmpdir(), "kit-command-smoke-")),
  );
  try {
    const workspace = join(root, "Kit #1 資料"),
      bin = join(workspace, "bin"),
      source = join(bin, "scriptkit");
    const scratch = join(root, "runner", "temps", "private"),
      original = join(workspace, "config.ts"),
      note = join(workspace, "day.md");
    mkdirSync(bin, { recursive: true, mode: 0o700 });
    mkdirSync(scratch, { recursive: true, mode: 0o700 });
    const bytes = readFileSync(new URL("../mcp-cli.ts", import.meta.url));
    writeFileSync(source, bytes, { flag: "wx", mode: 0o755 });
    writeFileSync(original, "original settings", { mode: 0o600 });
    writeFileSync(note, "original note", { mode: 0o600 });
    const pin = releasedBunPin(contents(".bun-version", pinBytes()));
    try {
      fn({
        workspace,
        scratch,
        source,
        original,
        note,
        run: (exec, expected = cliId(bytes)) => challengeCommand(workspace, scratch, expected, pin, exec),
      });
    } finally {
      expect(readFileSync(original, "utf8")).toBe("original settings");
      expect(readFileSync(note, "utf8")).toBe("original note");
    }
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
}
const mac = test.skipIf(process.platform !== "darwin");
mac(
  "the workspace command installs and runs by name, but will not certify a different executable",
  () =>
    fixture((f) => {
      const stages: { shell: boolean; status: number | null }[] = [];
      const r = f.run((i) => {
        const out = child(i);
        stages.push({ shell: i.args[2] === "/bin/sh", status: out.status });
        return out;
      });
      expect(stages).toEqual([
        { shell: false, status: 0 },
        { shell: false, status: 0 },
        { shell: true, status: 0 },
        { shell: true, status: 0 },
        { shell: false, status: 1 },
      ]);
      expect(r).toMatchObject({
        status: "pass",
        source: cliId(readFileSync(f.source)),
        bunVersion: process.versions.bun,
        pathHelpMatchedInstalledCommand: true,
        repeatKeptOriginalLink: true,
        differentExecutableRefusedWithoutReplacement: true,
      });
      expect(JSON.stringify(r)).not.toContain(f.workspace);
      expect(JSON.stringify(r)).not.toContain(f.scratch);
    }),
);
mac(
  "missing, substituted, nonexecutable and linked commands cannot start a child or create a link",
  () => {
    for (const fault of ["missing", "substitute", "mode", "linked"]) {
      fixture((f) => {
        if (fault === "missing") rmSync(f.source);
        if (fault === "substitute") {
          writeFileSync(f.source, "#!/bin/sh\nexit 73\n");
        }
        if (fault === "mode") chmodSync(f.source, 0o600);
        if (fault === "linked") {
          renameSync(f.source, f.source + ".elsewhere");
          symlinkSync(f.source + ".elsewhere", f.source);
        }
        let sent = false;
        expect(() =>
          f.run(() => {
            sent = true;
            throw Error("unexpected");
          })
        ).toThrow(
          fault === "missing"
            ? "installed_command_missing"
            : fault === "substitute"
            ? "installed_command_source_mismatch"
            : "installed_command_not_direct_executable",
        );
        expect(sent).toBe(false);
        expect(readdirSync(f.scratch)).toEqual([]);
      });
    }
  },
);
mac(
  "claimed installs, wrong PATH output, replaced links and unrelated refusals are not proof",
  () => {
    const cases = [
      ["noLink", 2, "command_link_missing"],
      ["wrongPath", 3, "command_shell_resolution_mismatch"],
      ["wrongHelp", 3, "command_path_help_mismatch"],
      ["newLink", 4, "command_link_replaced"],
      ["otherRefusal", 5, "command_conflict_not_refused"],
      ["changedDecoy", 5, "command_conflict_fixture_changed"],
    ] as const;
    for (const [fault, step, code] of cases) {
      fixture((f) => {
        let calls = 0;
        expect(() =>
          f.run((i) => {
            calls++;
            const target = join(i.cwd, "commands", "scriptkit");
            if (calls === step && fault === "noLink") {
              return {
                status: 0,
                stderr: Buffer.alloc(0),
                stdout: Buffer.from(
                  JSON.stringify({
                    success: true,
                    data: { command: "scriptkit", target, source: f.source },
                  }),
                ),
              };
            }
            const r = child(i);
            if (calls === step) {
              switch (fault) {
                case "wrongPath":
                  r.stdout = Buffer.concat([Buffer.from("wrong-"), r.stdout]);
                  break;
                case "wrongHelp":
                  r.stdout = Buffer.concat([
                    r.stdout,
                    Buffer.from("wrong command"),
                  ]);
                  break;
                case "newLink": {
                  const p = readlinkSync(target);
                  renameSync(target, join(i.cwd, "first-link"));
                  symlinkSync(p, target);
                  break;
                }
                case "otherRefusal":
                  r.stdout = Buffer.from(
                    '{"success":false,"error":"permission denied"}',
                  );
                  break;
                case "changedDecoy":
                  writeFileSync(
                    realpathSync.native(join(i.cwd, "other-bin", "scriptkit")),
                    "changed",
                  );
                  break;
              }
            }
            return r;
          })
        ).toThrow(code);
        expect(calls).toBe(step);
      });
    }
  },
);
mac(
  "an enrolled hostile fixture cannot write the installed command, original data, or shell startup file",
  () =>
    fixture((f) => {
      const bytes = Buffer.from(
        "#!/usr/bin/env bun\nimport fs from 'node:fs'; const paths=" +
          JSON.stringify([f.original, f.note, f.source]) +
          "; const codes=paths.map(p=>{try{fs.closeSync(fs.openSync(p,'r+'));return 'opened'}catch(e){return e.code}});try{fs.writeFileSync(process.env.HOME+'/bunfig.toml','x');codes.push('wrote')}catch(e){codes.push(e.code)}console.log(JSON.stringify(codes))",
      );
      writeFileSync(f.source, bytes);
      let seen: string[] = [];
      expect(() =>
        f.run((i) => {
          const r = child(i);
          seen = JSON.parse(r.stdout.toString());
          return r;
        }, cliId(bytes))
      ).toThrow("installed_command_help_unverified");
      expect(seen).toEqual(["EPERM", "EPERM", "EPERM", "EPERM"]);
    }),
);
test("tagged command and Bun pin are checked independently of the QA checkout", () => {
  const p = contents(".bun-version", pinBytes());
  expect(releasedBunPin(p).version).toBe(process.versions.bun);
  expect(() => releasedBunPin({ ...p, sha: "0".repeat(40) })).toThrow(
    "command_runtime_pin_mismatch",
  );
  expect(() => releasedBunPin(contents(".bun-version", Buffer.from("0.0.0\n"))))
    .toThrow("command_runtime_pin_mismatch");
  expect(() => releasedBunPin({ ...p, path: "other" })).toThrow(
    "command_runtime_pin_missing",
  );
  expect(() => releasedWorkspaceSource("mcp-cli.ts", p)).toThrow(
    "command_source_unavailable",
  );
  expect(() =>
    releasedWorkspaceSource("mcp-cli.ts", {
      ...contents("scripts/mcp-cli.ts", Buffer.from("x")),
      sha: "0".repeat(40),
    })
  ).toThrow("command_source_blob_mismatch");
});
