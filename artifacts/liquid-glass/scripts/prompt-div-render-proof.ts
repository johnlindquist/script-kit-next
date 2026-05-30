import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";

type JsonObject = Record<string, unknown>;

function arg(name: string, fallback: string) {
  const index = Bun.argv.indexOf(name);
  return index >= 0 ? Bun.argv[index + 1] ?? fallback : fallback;
}

const session = arg("--session", "liquid-glass-promptentity-render");
const label = arg("--label", "window-priority-prompt-div-fixed");
const fixture = resolve(arg("--fixture", "artifacts/liquid-glass/fixtures/prompt-div-liquid-glass.ts"));
const receiptRoot = "artifacts/liquid-glass/receipts";
const screenshotRoot = "artifacts/liquid-glass/screenshots";
const openOut = arg("--open-out", `${receiptRoot}/${label}-open-render.json`);
const waitOut = arg("--wait-out", `${receiptRoot}/${label}-wait-render.json`);
const layoutOut = arg("--layout-out", `${receiptRoot}/${label}-layout-devtools.json`);
const renderOut = arg("--render-out", `${receiptRoot}/${label}-render.json`);
const renderPng = arg("--render-png", `${screenshotRoot}/${label}-render.png`);
const matrixOut = arg("--matrix-out", `${receiptRoot}/liquid-glass-proof-matrix.json`);
const matrixRefreshOut = arg(
  "--matrix-refresh-out",
  `${receiptRoot}/liquid-glass-proof-matrix-refresh-prompt-div-render.json`,
);

mkdirSync(receiptRoot, { recursive: true });
mkdirSync(screenshotRoot, { recursive: true });

async function run(command: string[], allowFailure = false): Promise<JsonObject> {
  const proc = Bun.spawn(command, { stdout: "pipe", stderr: "pipe" });
  const [stdout, stderr, exitCode] = await Promise.all([
    new Response(proc.stdout).text(),
    new Response(proc.stderr).text(),
    proc.exited,
  ]);
  let parsed: JsonObject;
  try {
    parsed = stdout.trim() ? JSON.parse(stdout) as JsonObject : {};
  } catch {
    parsed = { stdout: stdout.trim() };
  }
  const receipt = {
    ...parsed,
    _command: command,
    _exitCode: exitCode,
    _stderr: stderr.trim(),
  };
  if (exitCode !== 0 && !allowFailure) {
    throw new Error(`${command.join(" ")} failed ${exitCode}: ${stderr || stdout}`);
  }
  return receipt;
}

function fixtureHtml(path: string) {
  const source = readFileSync(path, "utf8");
  const match = source.match(/await\s+div\(`([\s\S]*)`\);/);
  if (!match) {
    throw new Error(`Fixture ${path} must contain await div(\`...\`)`);
  }
  return match[1];
}

const openReceipt = await run([
  "bash",
  "scripts/agentic/session.sh",
  "send",
  session,
  JSON.stringify({
    type: "div",
    id: "liquid-glass-promptentity-render",
    html: fixtureHtml(fixture),
    requestId: `${label}-open-render`,
  }),
], true);
writeFileSync(openOut, `${JSON.stringify({
  schemaVersion: 1,
  label,
  command: "prompt-div.open-render-fixture",
  target: {
    surfaceKind: "PromptEntity",
    appViewVariant: "DivPrompt",
  },
  fixture,
  receipt: openReceipt,
}, null, 2)}\n`);

const waitReceipt = await run([
  "bash",
  "scripts/agentic/session.sh",
  "rpc",
  session,
  JSON.stringify({
    type: "waitFor",
    requestId: `${label}-wait-render`,
    condition: {
      type: "stateMatch",
      state: {
        promptType: "div",
        windowVisible: true,
      },
    },
    timeout: 8000,
    pollInterval: 50,
  }),
  "--expect",
  "waitForResult",
  "--timeout",
  "9000",
], true);
writeFileSync(waitOut, `${JSON.stringify({
  schemaVersion: 1,
  label,
  command: "prompt-div.wait-render-fixture",
  target: {
    surfaceKind: "PromptEntity",
    appViewVariant: "DivPrompt",
  },
  receipt: waitReceipt,
}, null, 2)}\n`);

const layoutReceipt = await run([
  "bun",
  "scripts/devtools/layout.ts",
  "measure",
  "--session",
  session,
  "--main",
  "--strict",
  "--surface",
  "PromptEntity",
  "--include",
  "nodes,regions,scroll,anchors,resize,overlaps",
], true);
writeFileSync(layoutOut, `${JSON.stringify({
  schemaVersion: 1,
  label,
  command: "prompt-div.layout-devtools",
  target: {
    surfaceKind: "PromptEntity",
    appViewVariant: "DivPrompt",
  },
  ...layoutReceipt,
}, null, 2)}\n`);

const renderReceipt = await run([
  "bun",
  "scripts/agentic/verify-shot.ts",
  "--session",
  session,
  "--label",
  `${label}-render`,
  "--target-json",
  JSON.stringify({ type: "main" }),
  "--visual-source",
  "render",
  "--skip-state",
  "--skip-probe",
  "--out",
  renderPng,
  "--json",
], true);
writeFileSync(renderOut, `${JSON.stringify({
  schemaVersion: 1,
  label,
  command: "prompt-div.app-render-readback",
  target: {
    surfaceKind: "PromptEntity",
    appViewVariant: "DivPrompt",
  },
  ...renderReceipt,
}, null, 2)}\n`);

const matrixReceipt = await run([
  "bun",
  "scripts/devtools/liquid-glass-proof.ts",
  "--out",
  matrixOut,
]);
writeFileSync(matrixRefreshOut, `${JSON.stringify(matrixReceipt, null, 2)}\n`);

console.log(JSON.stringify({
  schemaVersion: 1,
  status: "ok",
  label,
  session,
  fixture,
  receipts: {
    open: openOut,
    wait: waitOut,
    layout: layoutOut,
    render: renderOut,
    matrix: matrixOut,
    matrixRefresh: matrixRefreshOut,
  },
}, null, 2));
