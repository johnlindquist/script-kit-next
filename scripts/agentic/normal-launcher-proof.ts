// NORMAL.md: real user actions, assertions and correlated GPU captures. No native input,
// desktop capture, installed applications, TCC mutations, clipboard reads or live model calls.
// bun scripts/agentic/normal-launcher-proof.ts list
// bun scripts/agentic/normal-launcher-proof.ts <group> <immutable reference> <fresh .test-output dir>
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import {
  existsSync,
  readFileSync,
  readdirSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { join, resolve } from "node:path";
import { readArtifactReference } from "../devtools/design";
import { DriverCommandRefused, DriverLifecycleError } from "../devtools/driver";
import {
  OwnedEvaluationClient,
  type OwnedAction,
} from "../devtools/lib/owned-evaluation";
import {
  beginManagedTask,
  claimOutput,
  emptyOwnedCleanup,
  finalizeManagedTask,
  validateOutputTarget,
} from "./artifact-lifecycle";
import {
  compositeOver,
  decodePngRgba,
  encodePngRgba,
  paintNodes,
} from "./one-window-frames";

const groups = {
  names: "main.launcher-ready",
  rapid: "main.launcher-ready",
  wording: "main.launcher-ready",
  selected: "main.launcher-ready",
  copied: "main.launcher-ready",
  clean: "main.launcher-ready",
  permissions: "main.launcher-basics",
  settings: "main.launcher-basics",
  allowed: "main.launcher-ready",
  home: "main.script-list",
  navigation: "main.launcher-ready",
  hold: "main.script-list",
  paste: "main.launcher-ready",
  window: "main.launcher-ready",
  shortcuts: "main.script-list",
  personal: "main.launcher-basics",
  learned: "main.script-list",
  syntax: "main.script-list",
  watch: "main.launcher-watch",
  late: "main.launcher-reading",
  abandoned: "main.launcher-reading",
  permissionEntry: "main.launcher-basics",
  waiting: "main.launcher-reading",
  failure: "main.launcher-ready",
  stuck: "main.launcher-basics",
  script: "main.launcher-ready",
  cold: "main.launcher-cold",
  moved: "main.launcher-cold",
  expired: "main.launcher-cold",
  changes: "main.launcher-basics",
  restoreHome: "main.launcher-ready",
  history: "main.subject-tabs",
  pointer: "main.launcher-ready",
  today: "main.launcher-ready",
  trash: "main.launcher-ready",
  clips: "main.clipboard-history",
} as const;
const basics = [
  "names",
  "rapid",
  "selected",
  "permissions",
  "restoreHome",
] as const;
const [mode, referencePath, destination] = process.argv.slice(2);
if (mode === "list") {
  console.log(
    JSON.stringify({ groups, basics, allOwned: Object.keys(groups) }),
  );
  process.exit(0);
}
const pack =
  mode === "basics"
    ? basics
    : mode === "all-owned"
      ? (Object.keys(groups) as (keyof typeof groups)[])
      : null;
if ((!pack && !Object.hasOwn(groups, mode)) || !referencePath || !destination)
  throw Error(
    "expected basics, all-owned or group; immutable reference; fresh output directory. Use list to discover groups.",
  );
const ref = readArtifactReference(resolve(referencePath));
const sha = (s: string | Buffer) =>
  createHash("sha256").update(s).digest("hex");
async function runGroup(command: keyof typeof groups, output: string) {
  const fixture = groups[command];
  const claim = claimOutput(
    validateOutputTarget({
      repoRoot: process.cwd(),
      candidate: resolve(output),
      kind: "directory",
      probeId: "normal-launcher",
    }),
  );
  const task = beginManagedTask(claim, "evidence-run", [ref]);
  let client: OwnedEvaluationClient | undefined;
  let cleanup = emptyOwnedCleanup();
  const observations: Record<string, unknown>[] = [];
  let failure: string | undefined;
  try {
    client = await OwnedEvaluationClient.launch(
      process.cwd(),
      ref,
      claim,
      [fixture],
      undefined,
      { nativeGlass: "disabled" },
    );
    const c = client;
    const defs = await c.discover();
    assert.ok(defs.fixtures.some((f) => f.id === fixture));
    const target = await c.mount(fixture);
    await c.frame(target);
    const state = () => c.inspect(target);
    type J = Record<string, any>;
    const rows = (f: J) =>
      (f.elements.elements as J[]).filter((n) => n.role === "row");
    const title = (r?: J) => r?.text ?? r?.label ?? r?.name;
    const w = (f: J) => f.state.oneWindow;
    const titles = (f: J) => rows(f).map(title);
    const sel = (f: J) => title(rows(f).find((r) => r.selected));
    const act = async (a: OwnedAction) => {
      // Only retry an explicit pre-dispatch refusal, and never after a submit. Recheck
      // the input, view, selection and Enter target before retrying a character.
      const before = await state();
      for (let n = 0; ; n++) {
        const s = n ? await state() : before;
        if (n) {
          assert.deepEqual(
            [s.inputValue, s.oneWindow.view, s.appViewVariant],
            [before.inputValue, before.oneWindow.view, before.appViewVariant],
          );
          if (!(a.type === "key" && a.text))
            assert.deepEqual(
              [s.oneWindow.enter.id, s.oneWindow.selected],
              [before.oneWindow.enter.id, before.oneWindow.selected],
            );
        }
        try {
          return await c.act(target, a, s.targetIdentity);
        } catch (e) {
          if (
            !(e instanceof DriverCommandRefused) ||
            e.code !== "stale_target_identity" ||
            n >= 4
          ) {
            throw Error(
              `${JSON.stringify(a)}: ${e}; observed input: ${JSON.stringify((await state()).inputValue)}`,
            );
          }
          await Bun.sleep(35);
        }
      }
    };
    const key = (key: string, modifiers: string[] = []) =>
      act({ type: "key", key, modifiers });
    const chars = async (text: string) => {
      for (const ch of text)
        await act({
          type: "key",
          key: ch === " " ? "space" : ch.toLowerCase(),
          text: ch,
          ...(ch !== ch.toLowerCase() ? { modifiers: ["shift"] } : {}),
        });
    };
    const type = async (s: string) => {
      if ((await state()).inputValue) await key("a", ["cmd"]);
      await chars(s);
    };
    const until = async (pred: (s: J) => boolean, label: string, ms = 3500) => {
      let s: J = {};
      const end = performance.now() + ms;
      do {
        await c.frame(target);
        s = await state();
        if (pred(s)) return s;
        await Bun.sleep(30);
      } while (performance.now() < end);
      throw Error(
        `${label}: got ${JSON.stringify({ input: s.inputValue, enter: s.oneWindow?.enter, view: s.oneWindow?.view, lastRun: s.oneWindow?.lastRun, delivery: s.oneWindow?.ownedChatDelivery })}`,
      );
    };
    const ctl = async (
      control: import("../devtools/lib/owned-evaluation").FixtureControl,
    ) => {
      const before = await state();
      for (let i = 0; ; i++) {
        const s = i ? await state() : before;
        assert.deepEqual(
          [s.inputValue, s.oneWindow.view, s.oneWindow.enter.id],
          [before.inputValue, before.oneWindow.view, before.oneWindow.enter.id],
        );
        try {
          const r = await c.design({
            operation: "fixtureControl",
            target,
            expected: s.targetIdentity,
            control,
          });
          assert.ok(r.ok);
          return;
        } catch (e) {
          if (
            i >= 3 ||
            !(e instanceof DriverCommandRefused) ||
            e.code !== "stale_target_identity"
          )
            throw e;
          await Bun.sleep(45);
        }
      }
    };
    const enter = async (label?: string) => {
      await until(
        (s) =>
          !s.oneWindow.enterArmed &&
          (!label || s.oneWindow.enter.label === label),
        `ready Return: ${label}`,
      );
      await key("enter");
    };
    const shot = async (
      ids: string[],
      name: string,
      test: (f: J) => void,
      boundary?: string,
    ) => {
      // The text field updates before the 16ms search commit. Observe both revisions,
      // not an arbitrary sleep and not a wait for the *desired* winner. Rapid Return
      // has already been dispatched by this point and is never delayed by this check.
      const s0 = await state();
      if (
        s0.targetIdentity.appViewVariant === "ScriptList" &&
        s0.oneWindow.owned &&
        !s0.oneWindow.view
      )
        await until(
          (s) =>
            s.inputValue === s0.inputValue &&
            s.oneWindow.text === s0.inputValue &&
            s.search?.pending !== true,
          "current query committed",
        );
      const f = await c.captureFrame(target, true);
      const png = Buffer.from(f.snapshot.capture!.pngBase64, "base64");
      assert.equal(sha(png), f.snapshot.capture!.sha256);
      const data = encodePngRgba(compositeOver(decodePngRgba(png)));
      const filename = name + ".png";
      writeFileSync(join(claim.root, filename), data, { flag: "wx" });
      const row: J = {
        ids,
        name,
        fixture,
        filename,
        sha256: sha(data),
        frame: f.frame.target.frameGeneration,
        sourceSha256: sha(png),
        input: f.state.inputValue,
        enter: w(f).enter,
        selected: sel(f),
        rows: titles(f),
        kinds: rows(f).map((r) => r.kind),
        view: w(f).view ?? null,
        lastRun: w(f).lastRun ?? null,
        ...(boundary ? { boundary } : {}),
        pass: false,
      };
      observations.push(row);
      test(f);
      row.pass = true;
    };
    const opens = (f: J, name: string) => {
      assert.ok(rows(f).find((r) => r.selected));
      assert.match(w(f).enter.id, /^app\//);
      assert.equal(w(f).enter.label, `Open ${name}`);
      assert.equal(w(f).view, null);
    };
    const subjectAction = (f: J, words: string) => {
      assert.equal(f.state.inputValue, words);
      assert.equal(rows(f)[0]?.selected, true);
      assert.match(w(f).enter.id, /^subject\/.+\/ask-words$/);
      assert.equal(w(f).enter.label, "Ask the model");
    };
    const targetOnly =
      "Search and dispatch target only. The owned app cannot activate another macOS application.";
    const delivery = async (words: string, subject: string, source: string) => {
      await enter("Ask the model");
      const s = await until(
        (s) =>
          s.oneWindow.view === "chat" &&
          s.oneWindow.ownedChatDelivery?.turnCount === 1,
        "one turn received by the mock connection",
      );
      const hashes = s.oneWindow.ownedChatDelivery.lastTextBlocksSha256;
      assert.ok(
        hashes.includes(sha(`--- USER REQUEST ---\n${words}`)),
        "request words missing at adapter",
      );
      assert.ok(
        hashes.includes(
          sha(
            `<context source="${source}" mimeType="text/plain" label="${subject}">\n${subject}\n</context>`,
          ),
        ),
        "exact subject missing at adapter",
      );
      assert.equal(s.oneWindow.chatStagedPartCount, 0);
    };
    switch (command) {
      case "names": {
        await type("super");
        await shot(["1.4"], "prefix", (f) => opens(f, "Superhuman"));
        await chars("whisper");
        await shot(
          ["1.1", "1.4", "3.1"],
          "complete-name",
          (f) => opens(f, "superwhisper"),
          targetOnly,
        );
        await type("SUPERWHISPER");
        await shot(["1.6"], "upper-case", (f) => opens(f, "superwhisper"));
        await type("SuperWhisper");
        await shot(["1.6"], "mixed-case", (f) => opens(f, "superwhisper"));
        await chars("x");
        await key("backspace");
        await shot(["1.7"], "backspace", (f) => opens(f, "superwhisper"));
        await type("chr");
        await shot(["1.11"], "short-prefix", (f) => {
          opens(f, "Google Chrome");
          assert.equal(rows(f)[0]?.selected, true);
        });
        break;
      }
      case "rapid": {
        await type("superwhispe");
        await chars("r");
        const t = performance.now();
        await key("enter");
        const dispatchMs = performance.now() - t;
        await shot(
          ["1.3"],
          "immediate-return",
          (f) => {
            assert.equal(w(f).lastRun?.title, "superwhisper");
            assert.notEqual(w(f).view, "chat");
          },
          targetOnly,
        );
        observations.push({ name: "last-key-to-return-roundtrip", dispatchMs });
        break;
      }
      case "wording": {
        for (const words of [
          "translate this to French",
          "summarize",
          "please fix this",
          "proofread",
          "make this shorter",
          "fix grammar",
          "turn this into a list",
        ]) {
          await type(words);
          await shot(
            [
              [
                "make this shorter",
                "fix grammar",
                "turn this into a list",
              ].includes(words)
                ? "2.5"
                : "2.2",
            ],
            words.replaceAll(" ", "-"),
            (f) => subjectAction(f, words),
          );
        }
        break;
      }
      case "selected": {
        await type("Translate to French");
        await shot(["2.1"], "selected-instruction", (f) =>
          subjectAction(f, "Translate to French"),
        );
        await delivery("Translate to French", "how are you?", "Selected text");
        await shot(
          ["2.1"],
          "selected-delivered",
          (f) => assert.equal(w(f).ownedChatDelivery.turnCount, 1),
          "The actual first turn reached a provider-free adapter; the reply is fixture text.",
        );
        break;
      }
      case "copied": {
        await key("tab");
        await type("translate to Spanish");
        await shot(["2.6"], "copied-instruction", (f) =>
          subjectAction(f, "translate to Spanish"),
        );
        await delivery("translate to Spanish", "Good morning", "Copied text");
        await shot(["2.6"], "copied-delivered", (f) =>
          assert.equal(w(f).ownedChatDelivery.turnCount, 1),
        );
        break;
      }
      case "clean": {
        await type("translate hello to French");
        await shot(["2.3"], "instruction-with-object", (f) => {
          assert.ok(!titles(f).some((t) => t?.endsWith("· Selected text")));
          assert.ok(!w(f).ownedChatDelivery?.turnCount);
        });
        await type("translate");
        await shot(["2.11"], "translate-name-and-selection", (f) =>
          subjectAction(f, "translate"),
        );
        await key("down");
        await shot(["2.11"], "translate-app", (f) => opens(f, "Translate"));
        await type("Summarize Page");
        await shot(["2.12"], "verb-script-name", (f) => {
          assert.ok(rows(f)[0]?.selected);
          assert.match(w(f).enter.id, /script/i);
          assert.notEqual(w(f).enter.label, "Ask the model");
        });
        await type("Slack");
        await shot(["1.8", "2.4"], "namesake-app", (f) => {
          opens(f, "Slack");
          assert.equal(rows(f)[0]?.selected, true);
        });
        await key("down");
        await shot(["1.8"], "namesake-script", (f) => {
          assert.equal(rows(f)[1]?.selected, true);
          assert.match(w(f).enter.id, /script/i);
          assert.notEqual(w(f).enter.label, "Open Slack");
        });
        await key("escape");
        await key("enter", ["cmd"]);
        await shot(["2.8"], "quick-question", (f) => {
          assert.equal(w(f).view, "chat");
          assert.equal(w(f).chatStagedPartCount, 0);
          assert.ok(!w(f).ownedChatDelivery?.turnCount);
        });
        break;
      }
      case "permissions": {
        await shot(["4.1", "4.6"], "first-open", (f) => {
          assert.equal(w(f).header, "Needs setup");
          assert.deepEqual(titles(f).slice(0, 3), [
            "Accessibility",
            "Screen Recording",
            "Continue without them",
          ]);
          assert.ok(w(f).subjectTabs.length);
        });
        await key("down");
        await key("down");
        await enter("Continue without them");
        await shot(["4.4"], "declined", (f) =>
          assert.ok(!titles(f).includes("Accessibility")),
        );
        await key("z", ["cmd"]);
        await shot(["4.4"], "undo-decline", (f) =>
          assert.deepEqual(titles(f).slice(0, 3), [
            "Accessibility",
            "Screen Recording",
            "Continue without them",
          ]),
        );
        break;
      }
      case "settings":
      case "allowed": {
        await type("Set Up Permissions");
        await shot(["4.8", "4.9"], "permission-command", (f) =>
          assert.equal(sel(f), "Set Up Permissions"),
        );
        await enter();
        if (command === "settings")
          await shot(["4.9"], "all-permission-rows", (f) =>
            assert.deepEqual(titles(f).slice(0, 6), [
              "Accessibility",
              "Screen Recording",
              "Event Synthesizing",
              "Input Monitoring",
              "Microphone",
              "Continue without them",
            ]),
          );
        else
          await shot(["4.8"], "all-granted", (f) =>
            assert.match(
              JSON.stringify([w(f).line, w(f).recent]),
              /Every permission is on/,
            ),
          );
        break;
      }
      case "watch": {
        const name = "Lantern Inspector";
        await type(name);
        await shot(["6.2"], "before-script-save", (f) =>
          assert.ok(!rows(f).some((r) => r.kind === "script")),
        );
        const p = join(
          c.driver.sessionDir,
          "normal-kit/plugins/fixture/scripts/lantern-inspector.ts",
        );
        writeFileSync(p, "// Name: Lantern Inspector\nexport {};\n", {
          flag: "wx",
          mode: 0o600,
        });
        const end = performance.now() + 15500;
        let found = false;
        do {
          await c.frame(target);
          const now = await c.query(target, "elements");
          found = now.elements.some(
            (r: J) => r.role === "row" && r.kind === "script",
          );
          if (!found) await Bun.sleep(80);
        } while (!found && performance.now() < end);
        assert.ok(found, "real watcher/loader did not publish the new script");
        // The scenario is a fresh search after saving, in the same application process.
        await key("escape");
        await type(name);
        await until(
          (s) =>
            s.oneWindow.text === name &&
            s.oneWindow.enter.id.startsWith("script/"),
          "new script selected by name",
        );
        await shot(
          ["6.2"],
          "after-script-save",
          (f) => {
            assert.equal(rows(f)[0].kind, "script");
            assert.equal(rows(f)[0].selected, true);
          },
          "A real file save, watcher notification and script parser in an isolated plugin directory.",
        );
        break;
      }
      case "late": {
        // A 6.5s response from the owned service contains a duplicate app launch
        // plus a distinct question. The normal host validates and projects it.
        await act({ type: "setInput", text: "superwhisper" });
        await until(
          (s) =>
            s.oneWindow.text === "superwhisper" &&
            s.oneWindow.jevRequestCount > 0 &&
            !s.oneWindow.jevSettled,
          "Jev still reading",
        );
        await shot(["1.2"], "app-before-reading", (f) =>
          opens(f, "superwhisper"),
        );
        await until(
          (s) => s.oneWindow.jevRows.some((r: J) => r.key === "down:model"),
          "five-second fallback deadline",
          6500,
        );
        await shot(["1.2"], "app-during-timeout", (f) =>
          opens(f, "superwhisper"),
        );
        const arrival = await until(
          (s) =>
            s.oneWindow.jevRows.some(
              (r: J) => r.title === "Ask about superwhisper",
            ),
          "success after deadline",
          3500,
        );
        await shot(
          ["1.2"],
          "app-after-reading",
          (f) => {
            opens(f, "superwhisper");
            const words = w(f).jevRows.map((r: J) => r.title);
            assert.ok(words.includes("Ask about superwhisper"));
            assert.ok(!words.includes("Open superwhisper"));
          },
          "A delayed fixture response, not a live model request.",
        );
        for (
          let i = 0;
          i < 12 &&
          (await state()).oneWindow.enter.label !== "Ask about superwhisper";
          i++
        )
          await key("down");
        await shot(["3.4"], "reading-selected", (f) =>
          assert.equal(w(f).enter.label, "Ask about superwhisper"),
        );
        await until(
          (s) =>
            s.oneWindow.jevResponseCount > arrival.oneWindow.jevResponseCount,
          "second answer received",
        );
        await shot(["3.4"], "second-answer-ignored", (f) => {
          assert.equal(w(f).enter.label, "Ask about superwhisper");
          assert.ok(
            !w(f).jevRows.some((r: J) => r.title === "Obsolete second answer"),
          );
        });
        break;
      }
      case "abandoned": {
        await act({ type: "setInput", text: "superwhisper" });
        await until(
          (s) => s.oneWindow.jevRows.some((r: J) => r.key === "down:model"),
          "fallback deadline",
          6500,
        );
        const count = (await state()).oneWindow.jevResponseCount;
        await key("escape");
        await until((s) => s.inputValue === "", "abandoned sentence");
        await until(
          (s) => s.oneWindow.jevResponseCount >= count + 2,
          "both abandoned replies observed",
          4000,
        );
        await shot(["1.2"], "late-answer-cancelled", (f) => {
          assert.equal(f.state.inputValue, "");
          assert.equal(w(f).view, null);
          assert.ok(!w(f).lastRun);
        });
        break;
      }
      case "permissionEntry": {
        await enter("Allow Accessibility");
        await shot(
          ["4.2"],
          "scoped-grant-refusal",
          (f) => {
            assert.equal(w(f).lastRun?.key, "setup:permission/accessibility");
            assert.match(
              JSON.stringify([w(f).line, w(f).recent]),
              /Couldn't check permissions here/,
            );
          },
          "The grant dispatch reaches the native permission boundary and is refused before macOS is asked.",
        );
        break;
      }
      case "waiting": {
        const words = "what describes the lantern paradox";
        await act({ type: "setInput", text: words });
        await until(
          (s) =>
            s.oneWindow.text === words &&
            s.oneWindow.jevRequestCount > 0 &&
            !s.oneWindow.enterArmed &&
            !s.oneWindow.jevSettled,
          "unguarded pending reading",
        );
        await key("enter");
        await shot(["3.3"], "after-early-return", (f) => {
          assert.ok(!w(f).view && !w(f).ownedChatDelivery);
          assert.ok(!w(f).jevSettled);
        });
        await until(
          (s) =>
            s.oneWindow.view === "chat" &&
            s.oneWindow.ownedChatDelivery?.turnCount === 1,
          "held Return dispatched once",
          6000,
        );
        await shot(
          ["3.3"],
          "one-delayed-turn",
          (f) => assert.equal(w(f).ownedChatDelivery.turnCount, 1),
          "One definite reading; the close-alternatives confirmation also has a reducer test.",
        );
        break;
      }
      case "failure": {
        await ctl({ family: "oneWindow", operation: "reportScriptFailure" });
        await until(
          (s) =>
            s.oneWindow.view === "chat" &&
            s.oneWindow.chatStagedPartCount === 2,
          "script report in chat",
        );
        await shot(
          ["6.3"],
          "failure-context-not-sent",
          (f) => {
            const x = w(f);
            assert.ok(!x.ownedChatDelivery?.turnCount);
            assert.match(
              JSON.stringify([x.line, x.recent]),
              /stopped with an error/,
            );
          },
          "ScriptError report and files from an owned fixture; a script process did not execute.",
        );
        break;
      }
      case "script": {
        await type("Summarize Page");
        await shot(
          ["6.1"],
          "script-launch-target",
          (f) => {
            assert.equal(rows(f)[0]?.kind, "script");
            assert.match(w(f).enter.id, /script/i);
          },
          "Script discovery/selection only. Script execution is forbidden in the owned renderer.",
        );
        break;
      }
      case "stuck": {
        await key("down");
        await ctl({
          family: "oneWindow",
          operation: "observeUnchangedPermissionRequest",
        });
        const end = performance.now() + 20000;
        let sub = "";
        do {
          await Bun.sleep(250);
          await c.frame(target);
          const f = await c.query(target, "elements");
          sub = JSON.stringify(
            f.elements.find(
              (r: J) => r.role === "row" && title(r) === "Screen Recording",
            ),
          );
          if (sub.includes("restart Script Kit")) break;
        } while (performance.now() < end);
        await shot(
          ["4.11"],
          "unchanged-grant",
          (f) => {
            assert.ok(
              sub.includes("restart Script Kit") ||
                JSON.stringify(f.elements).includes("restart Script Kit"),
            );
          },
          "Owned request history and a detector remaining off. No TCC change, restart, or reset.",
        );
        break;
      }
      case "cold":
      case "moved": {
        await type("superwhisper");
        await until(
          (s) => s.oneWindow.text === "superwhisper",
          "catalogue query",
        );
        if (command === "moved") await key("down");
        let earlier = "";
        await shot(
          [command === "moved" ? "1.10" : "1.9"],
          "before-catalogue",
          (f) => {
            earlier = w(f).enter.id;
            assert.ok(!earlier.startsWith("app/"));
          },
        );
        await ctl({ family: "oneWindow", operation: "finishAppIndex" });
        await shot(
          [command === "moved" ? "1.10" : "1.9"],
          "after-catalogue",
          (f) => {
            if (command === "moved") assert.equal(w(f).enter.id, earlier);
            else opens(f, "superwhisper");
          },
          "Delayed catalogue supplied to the production publisher; no native app discovery or activation.",
        );
        break;
      }
      case "expired": {
        await shot(["2.7"], "expired-copy", (f) =>
          assert.ok(
            !w(f).subjectTabs.some((tab: J) =>
              /copied|selected text/i.test(tab.text),
            ),
          ),
        );
        await type("translate to Spanish");
        await shot(["2.7", "2.9"], "no-stale-subject", (f) =>
          assert.ok(
            !rows(f).some(
              (r) =>
                title(r)?.includes(" · Copied") ||
                title(r)?.includes(" · Selected text"),
            ),
          ),
        );
        break;
      }
      case "changes": {
        await ctl({
          family: "oneWindow",
          operation: "observeAccessibility",
          granted: true,
        });
        await until(
          (s) => s.oneWindow.enter.label === "Allow Screen Recording",
          "status poll removed granted row",
        );
        await shot(
          ["4.3"],
          "accessibility-now-granted",
          (f) => {
            assert.deepEqual(titles(f).slice(0, 2), [
              "Screen Recording",
              "Continue without them",
            ]);
            assert.ok(!titles(f).includes("Accessibility"));
          },
          "Detector change, not a grant in System Settings.",
        );
        await ctl({
          family: "oneWindow",
          operation: "observeAccessibility",
          granted: false,
        });
        await until(
          (s) => s.oneWindow.enter.label === "Allow Accessibility",
          "status poll saw revocation",
        );
        await shot(
          ["4.5"],
          "accessibility-revoked",
          (f) => assert.equal(titles(f)[0], "Accessibility"),
          "Detector change.",
        );
        break;
      }
      case "restoreHome": {
        await key("enter", ["cmd"]);
        await shot(["5.1"], "in-chat", (f) => assert.equal(w(f).view, "chat"));
        await ctl({ family: "oneWindow", operation: "shortcutOutside" });
        await shot(
          ["5.1"],
          "back-to-launcher",
          (f) => {
            assert.equal(f.state.targetIdentity.appViewVariant, "ScriptList");
            assert.equal(w(f).view, null);
            assert.equal(f.state.inputValue, "");
          },
          "Production shortcut handler, not global delivery or a visible first frame.",
        );
        await type("superwhisper");
        await enter("Open superwhisper");
        await shot(
          ["5.1"],
          "latest-app-target",
          (f) => assert.equal(w(f).lastRun?.title, "superwhisper"),
          "Launch target only; activation blocked by the owned policy.",
        );
        break;
      }
      case "history": {
        await key("up");
        await shot(["5.7"], "last-run-history", (f) =>
          assert.ok(
            w(f).subjectTabs.some(
              (t: J) => t.selected && t.text === "Last run",
            ),
          ),
        );
        break;
      }
      case "pointer": {
        await type("super");
        await until(
          (s) => s.oneWindow.text === "super" && !s.oneWindow.enterArmed,
          "query ready",
        );
        const second = (await c.query(target, "elements")).elements.filter(
          (e: J) => e.role === "row",
        )[1].semanticId;
        const click = async () => {
          const old = await state();
          for (let i = 0; ; i++) {
            const f = await c.captureFrame(target, false);
            assert.equal(w(f).text, "super");
            assert.deepEqual(w(f).lastRun, old.oneWindow.lastRun);
            assert.equal(w(f).enter.id, old.oneWindow.enter.id);
            const n = paintNodes(f.layout).find(
              (n) => n.name === "list-row:" + second && n.visible,
            );
            assert.ok(n);
            try {
              await c.act(
                target,
                {
                  type: "gpuiEvent",
                  frame: f.frame,
                  event: {
                    type: "mouseClick",
                    button: "left",
                    x: n.bounds.x + n.bounds.w / 2,
                    y: n.bounds.y + n.bounds.h / 2,
                  },
                },
                f.frame.target,
              );
              return;
            } catch (e) {
              if (
                i >= 3 ||
                !(e instanceof DriverCommandRefused) ||
                e.code !== "stale_target_identity"
              )
                throw e;
              await Bun.sleep(45);
            }
          }
        };
        await click();
        await shot(["5.5"], "click-selects", (f) => {
          opens(f, "superwhisper");
          assert.ok(!w(f).lastRun);
        });
        await click();
        await shot(
          ["5.5"],
          "second-click-activates",
          (f) => assert.equal(w(f).lastRun?.title, "superwhisper"),
          "Two scoped GPUI clicks; not an OS double-click cadence measurement.",
        );
        break;
      }
      case "today": {
        await key("t", ["cmd"]);
        await until(
          (s) => s.targetIdentity.appViewVariant === "DayPage",
          "Today editor",
        );
        await shot(
          ["6.7"],
          "today-in-place",
          (f) => assert.equal(f.state.targetIdentity.appViewVariant, "DayPage"),
          "Today opens in the main window; not the global Notes hotkey.",
        );
        break;
      }
      case "trash": {
        const notes = join(c.driver.sessionDir, "notes/brain/notes");
        const note = readdirSync(notes).find(
          (n) =>
            n.endsWith(".md") &&
            readFileSync(join(notes, n), "utf8").includes(
              "Second document for search and switcher.",
            ),
        );
        assert.ok(note);
        const path = join(notes, note);
        await key("t", ["cmd"]);
        await until(
          (s) => s.targetIdentity.appViewVariant === "DayPage",
          "Today editor",
        );
        await key("p", ["cmd"]);
        await chars("Fixture Beta");
        await key("enter");
        await until(
          (s) =>
            s.targetIdentity.appViewVariant === "DayPage" &&
            String(s.inputValue).includes(
              "Second document for search and switcher",
            ),
          "named note editor",
        );
        await key("k", ["cmd"]);
        await chars("Move to Trash");
        await key("enter");
        await shot(["6.6"], "note-delete-hold", () =>
          assert.ok(existsSync(path)),
        );
        await until(() => !existsSync(path), "owned note moved", 11500);
        await shot(["6.6"], "note-in-trash", (f) =>
          assert.match(
            JSON.stringify([w(f).line, w(f).recent]),
            /Moved|trash/i,
          ),
        );
        // The editor owns its text undo. NORMAL 6.6 uses the empty launcher input.
        if ((await state()).targetIdentity.appViewVariant === "DayPage")
          await key("escape");
        await until(
          (s) =>
            s.targetIdentity.appViewVariant === "ScriptList" && !s.inputValue,
          "empty launcher for undo",
        );
        await key("z", ["cmd"]);
        await until(() => existsSync(path), "owned note restored");
        await shot(["6.6"], "note-restored", () =>
          assert.match(
            readFileSync(path, "utf8"),
            /Second document for search and switcher/,
          ),
        );
        break;
      }
      case "clips": {
        await shot(
          ["6.8"],
          "clipboard-history",
          (f) => {
            assert.equal(
              f.state.targetIdentity.appViewVariant,
              "ClipboardHistoryView",
            );
            const choices = f.elements.elements.filter((e: J) =>
              e.semanticId?.startsWith("choice:"),
            );
            assert.equal(choices.length, 3);
          },
          "Seeded clipboard history only.",
        );
        await chars("release notes");
        await shot(
          ["6.8"],
          "filtered-clipboard",
          (f) => {
            assert.equal(f.state.inputValue, "release notes");
            // Private choice IDs are opaque. Check the three-to-one reduction
            // here and review the resulting selected text on the captured frame.
            const choices = f.elements.elements.filter((e: J) =>
              e.semanticId?.startsWith("choice:"),
            );
            assert.equal(choices.length, 1);
            assert.equal(choices[0].selected, true);
          },
          "A real matching entry in the owned clipboard store; no native paste destination.",
        );
        await type("nothing-matches-this-fixture");
        await shot(["6.8"], "no-clipboard-match", (f) =>
          assert.ok(
            !f.elements.elements.some((e: J) =>
              e.semanticId?.startsWith("choice:"),
            ),
          ),
        );
        break;
      }
      case "paste": {
        // Seed only the evaluator's process-local clipboard through the same input's Copy command.
        // A nonempty replacement selection gives paste to the input, not the empty launcher's subject shortcut.
        await type("superwhisper");
        await key("a", ["cmd"]);
        await key("c", ["cmd"]);
        await type("replace");
        await key("a", ["cmd"]);
        await key("v", ["cmd"]);
        await shot(
          ["1.5"],
          "pasted-name",
          (f) => {
            assert.equal(f.state.inputValue, "superwhisper");
            opens(f, "superwhisper");
          },
          "Paste into the search field from a process-local clipboard.",
        );
        break;
      }
      case "window": {
        await key("tab");
        await key("tab");
        await type("translate to French");
        await shot(["2.9"], "window-not-attached", (f) => {
          assert.ok(
            !titles(f).some(
              (t) =>
                t?.includes(" · Fixture Editor") ||
                t?.includes(" · Selected text"),
            ),
          );
          assert.ok(!w(f).ownedChatDelivery?.turnCount);
        });
        break;
      }
      case "shortcuts": {
        await type("a blue kite at noon");
        await until((s) => s.oneWindow.jevSettled, "stand-in rows");
        await shot(["3.2", "5.6"], "offline-actions", (f) =>
          assert.deepEqual(
            w(f).jevRows.map((r: J) => r.key),
            ["down:model", "down:retry", "down:note"],
          ),
        );
        await key("3", ["cmd"]);
        await shot(["5.6"], "shortcut-opens-note", (f) => {
          assert.equal(f.state.targetIdentity.appViewVariant, "DayPage");
          assert.equal(w(f).lastRun?.key, "down:note");
          assert.match(f.state.inputValue, /a blue kite at noon/);
        });
        break;
      }
      case "learned": {
        await type("put the pr on my second screen");
        await shot(
          ["1.12"],
          "learned-sentence",
          (f) =>
            assert.equal(
              w(f).enter.id,
              "learned:Move Google Chrome to Next Display",
            ),
          "Pre-existing lesson; the same-titled application collision is checked in the reducer.",
        );
        break;
      }
      case "personal": {
        await type("show only scripts when I open Kit");
        await shot(["6.9"], "home-rule", (f) =>
          assert.equal(sel(f), "Show just scripts on the launcher home"),
        );
        await enter();
        await until(
          (s) =>
            JSON.stringify(s.oneWindow.personalization)
              .toLowerCase()
              .includes("minimal"),
          "scripts-only home rule applied",
        );
        await shot(["4.7", "6.9"], "minimal-home", (f) => {
          assert.equal(w(f).header, "Needs setup");
          assert.ok(w(f).subjectTabs.length === 0);
        });
        await key("z", ["cmd"]);
        await shot(["6.9"], "undo-home-rule", (f) =>
          assert.ok(w(f).subjectTabs.length),
        );
        break;
      }
      case "syntax": {
        for (const [name, text] of [
          ["at", "@"],
          ["slash", "/"],
          ["capture", ";todo"],
          ["postfix", "todo; buy milk"],
        ]) {
          const n = (await state()).oneWindow.jevRequestCount;
          await act({ type: "setInput", text });
          await Bun.sleep(300);
          await shot(
            ["3.5"],
            name,
            (f) => {
              assert.equal(f.state.inputValue, text);
              assert.equal(w(f).jevRequestCount, n);
            },
            "Complete syntax inserted as a unit. The host request count stayed unchanged across the debounce interval.",
          );
        }
        break;
      }
      case "home": {
        await shot(["4.10"], "missing-bun", (f) =>
          assert.equal(titles(f)[0], "bun is not installed"),
        );
        break;
      }
      case "navigation": {
        const geom = async () => {
          const f = await c.captureFrame(target, false);
          return {
            input: paintNodes(f.layout).find(
              (n) => n.name.includes("input") && n.visible,
            )?.bounds.y,
            height: w(f).heightPx,
          };
        };
        const baseline = await geom();
        assert.equal(typeof baseline.input, "number");
        assert.ok(baseline.height > 0);
        await type("s");
        await shot(["5.4"], "first-letter", () => {});
        await chars("uperwhisper");
        assert.deepEqual(await geom(), baseline);
        await shot(["5.4"], "unchanged-geometry", (f) =>
          opens(f, "superwhisper"),
        );
        await key("escape");
        await shot(["5.3"], "cleared", (f) =>
          assert.equal(f.state.inputValue, ""),
        );
        break;
      }
      case "hold": {
        await type("Quit Script Kit");
        await shot(["6.5"], "quit-command", (f) =>
          assert.equal(sel(f), "Quit Script Kit"),
        );
        await enter();
        await shot(["6.5"], "quit-wait", (f) =>
          assert.match(JSON.stringify(w(f).line), /Runs in|Quit Script Kit/i),
        );
        await key("z", ["cmd"]);
        await shot(["6.5"], "quit-cancelled", (f) =>
          assert.match(
            JSON.stringify([w(f).line, w(f).recent]),
            /Didn't run|Didn’t run/,
          ),
        );
        break;
      }
    }
  } catch (error) {
    if (error instanceof DriverLifecycleError) cleanup = error.cleanup;
    failure = String(error);
  } finally {
    if (client) {
      cleanup = await client.close();
      if (failure && existsSync(client.driver.logPath)) {
        const lines = readFileSync(client.driver.logPath, "utf8")
          .split("\n")
          .filter(
            (s) =>
              s.length < 1500 &&
              !s.startsWith("{") &&
              /enter|submit|reading|picked|key_down|fixture|focus|refus/i.test(
                s,
              ),
          );
        writeFileSync(
          join(claim.root, "failure-local.log"),
          lines.slice(-90).join("\n"),
        );
      }
    }
    cleanup = finalizeManagedTask(task, cleanup).cleanup;
    if (
      cleanup.closed &&
      client &&
      client.driver.sessionDir.startsWith(claim.root + "/")
    )
      rmSync(client.driver.sessionDir, { recursive: true, force: true });
    const receipt = {
      schemaVersion: 1,
      group: command,
      fixture,
      artifact: ref,
      observations,
      closed: cleanup.closed,
      failure: failure ?? null,
    };
    writeFileSync(
      join(claim.root, "normal.json"),
      JSON.stringify(receipt, null, 2),
      { flag: "wx" },
    );
    console.log(
      JSON.stringify({
        group: command,
        captured: observations.filter((o) => o.filename).length,
        closed: cleanup.closed,
        failure: failure ?? null,
      }),
    );
    if (!cleanup.closed || failure) process.exitCode = 1;
    if (!cleanup.closed)
      throw Error("Owned runtime cleanup incomplete; stopping the pack");
  }
  return !failure;
}
if (!pack) await runGroup(mode as keyof typeof groups, destination);
else {
  const parent = claimOutput(
    validateOutputTarget({
      repoRoot: process.cwd(),
      candidate: resolve(destination),
      kind: "directory",
      probeId: "normal-launcher-pack",
    }),
  );
  const task = beginManagedTask(parent, "evidence-run", [ref]);
  const outcomes: Record<string, boolean> = {};
  try {
    for (const group of pack)
      outcomes[group] = await runGroup(group, join(parent.root, group));
  } finally {
    const closed = finalizeManagedTask(task, emptyOwnedCleanup()).cleanup
      .closed;
    writeFileSync(
      join(parent.root, "pack.json"),
      JSON.stringify(
        {
          mode,
          artifact: ref,
          outcomes,
          complete: closed && pack.every((g) => outcomes[g] === true),
        },
        null,
        2,
      ),
      { flag: "wx" },
    );
  }
}
