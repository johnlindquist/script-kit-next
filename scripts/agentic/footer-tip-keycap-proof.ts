#!/usr/bin/env bun

import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { Driver } from "../devtools/driver";

const projectRoot = resolve(import.meta.dir, "../..");
const binary = join(
  projectRoot,
  "target-agent/artifacts/footer-tip-proof/script-kit-gpui",
);
const outDir = "/tmp/sk-footer-tip-proof";
const target = { type: "id", id: "footer-overlay" };
const windowQuery = join(projectRoot, "scripts/agentic/macos-window-query.swift");
const desired = new Map([
  ["opens Today", "space"],
  ["twice for Quick AI", "quick-ai"],
]);

rmSync(outDir, { recursive: true, force: true });
mkdirSync(outDir, { recursive: true });

const receipt: Record<string, unknown> = {
  binary,
  binarySha256: createHash("sha256").update(readFileSync(binary)).digest("hex"),
  outDir,
  captures: {},
  rotations: [],
};
const captures = receipt.captures as Record<string, unknown>;

async function runCommand(args: string[]) {
  const process = Bun.spawn(args, { stdout: "pipe", stderr: "pipe" });
  const [stdout, stderr, exitCode] = await Promise.all([
    new Response(process.stdout).text(),
    new Response(process.stderr).text(),
    process.exited,
  ]);
  return { stdout, stderr, exitCode };
}

async function pngDimensions(path: string) {
  const result = await runCommand([
    "sips",
    "-g",
    "pixelWidth",
    "-g",
    "pixelHeight",
    path,
  ]);
  const width = Number(result.stdout.match(/pixelWidth: (\d+)/)?.[1] ?? 0);
  const height = Number(result.stdout.match(/pixelHeight: (\d+)/)?.[1] ?? 0);
  return { width, height };
}

async function captureExactFooterWindow(pid: number, savePath: string) {
  const query = await runCommand([
    "swift",
    windowQuery,
    "--pid",
    String(pid),
  ]);
  if (query.exitCode !== 0) {
    throw new Error(`window query failed: ${query.stderr.trim()}`);
  }
  const windows = JSON.parse(query.stdout)?.windows ?? [];
  const footer = windows.find((window: any) =>
    window.onscreen === true &&
    typeof window.title === "string" &&
    window.title.toLowerCase().includes("footer overlay")
  );
  if (!footer?.windowId) {
    throw new Error(`no on-screen footer overlay for pid ${pid}`);
  }

  const shot = await runCommand([
    "screencapture",
    "-x",
    "-o",
    "-l",
    String(footer.windowId),
    savePath,
  ]);
  if (shot.exitCode !== 0 || !existsSync(savePath)) {
    throw new Error(
      `exact-window screencapture failed: exit=${shot.exitCode} ${shot.stderr.trim()}`,
    );
  }
  return {
    captureMethod: "external-screencapture",
    nativeWindow: footer,
    ...(await pngDimensions(savePath)),
  };
}

const driver = await Driver.launch({
  binary,
  sessionName: `footer-tip-proof-${process.pid}`,
  sandboxHome: true,
});
receipt.sessionDir = driver.sessionDir;

try {
  driver.send({ type: "show" });
  await driver.waitForState({ windowVisible: true }, { timeoutMs: 10_000 });
  await driver.waitForSettle();

  const sandboxTips = join(driver.sessionDir, "home", ".scriptkit", "tips.json");
  receipt.sandboxTips = {
    path: sandboxTips,
    exists: existsSync(sandboxTips),
    sha256: existsSync(sandboxTips)
      ? createHash("sha256").update(readFileSync(sandboxTips)).digest("hex")
      : null,
  };

  for (let index = 0; index < 12 && Object.keys(captures).length < desired.size; index += 1) {
    const state: any = await driver.getState();
    const leftInfo = state?.activeFooter?.leftInfo ?? state?.state?.activeFooter?.leftInfo ?? null;
    const modelName = leftInfo?.modelName ?? null;
    const keycap = leftInfo?.keycap ?? null;
    (receipt.rotations as unknown[]).push({ index, modelName, keycap });

    const captureName = typeof modelName === "string" ? desired.get(modelName) : undefined;
    if (captureName && captures[captureName] === undefined) {
      let footerWindow: any = null;
      const deadline = Date.now() + 10_000;
      while (!footerWindow && Date.now() < deadline) {
        const windows: any = await driver.listAutomationWindows();
        footerWindow = (windows?.windows ?? []).find((window: any) => window.id === "footer-overlay");
        if (!footerWindow) await Bun.sleep(25);
      }
      if (!footerWindow) throw new Error("footer-overlay automation window did not appear");

      const savePath = join(outDir, `${captureName}.png`);
      const [driverScreenshot, elements, layout] = await Promise.all([
        driver.captureScreenshot({ target, hiDpi: true, savePath, timeoutMs: 15_000 }),
        driver.getElements({ target }, { timeoutMs: 10_000 }),
        driver.getLayoutInfo({ target }, { timeoutMs: 10_000 }),
      ]);
      const screenshotReceipt: any = driverScreenshot;
      let screenshotError = screenshotReceipt?.error ?? null;
      let screenshotExists = existsSync(savePath);
      let captureMethod = "driver-captureScreenshot";
      let nativeWindow = null;
      let screenshotWidth = screenshotReceipt?.width ?? null;
      let screenshotHeight = screenshotReceipt?.height ?? null;

      if (screenshotError != null || !screenshotExists) {
        try {
          const exactCapture = await captureExactFooterWindow(footerWindow.pid, savePath);
          screenshotError = null;
          screenshotExists = true;
          captureMethod = exactCapture.captureMethod;
          nativeWindow = exactCapture.nativeWindow;
          screenshotWidth = exactCapture.width;
          screenshotHeight = exactCapture.height;
        } catch (error) {
          screenshotError = `${String(screenshotError)}; fallback=${String(error)}`;
        }
      }
      captures[captureName] = {
        modelName,
        keycap,
        footerWindow,
        screenshot: {
          savePath,
          width: screenshotWidth,
          height: screenshotHeight,
          error: screenshotError,
          exists: screenshotExists,
          captureMethod,
          nativeWindow,
        },
        elements,
        layout,
      };
    }

    if (Object.keys(captures).length >= desired.size) break;
    driver.send({ type: "hide" });
    await driver.waitForState({ windowVisible: false }, { timeoutMs: 10_000 });
    driver.send({ type: "show" });
    await driver.waitForState({ windowVisible: true }, { timeoutMs: 10_000 });
    await driver.waitForSettle();
  }

  if (Object.keys(captures).length !== desired.size) {
    throw new Error(`captured ${Object.keys(captures).length}/${desired.size} desired tips`);
  }

  // Hover proof: the action-bearing left-info tip renders as a real footer
  // button (canonical hover pill + text/glyph brightening). Dispatch a real
  // mouseMove over its bounds and require the hover frame to paint
  // differently from the rest frame. Screenshot failures are classified
  // honestly instead of silently passing.
  {
    driver.send({ type: "show" });
    await driver.waitForState({ windowVisible: true }, { timeoutMs: 10_000 });
    await driver.waitForSettle();

    const layout: any = await driver.getLayoutInfo({ target }, { timeoutMs: 10_000 });
    let tipBounds: { x: number; y: number; width: number; height: number } | null = null;
    let tipBoundsSource = "layout";
    const stack: any[] = [layout];
    while (stack.length > 0) {
      const node = stack.pop();
      if (node == null || typeof node !== "object") continue;
      if (Array.isArray(node)) {
        stack.push(...node);
        continue;
      }
      const label = `${node.id ?? ""} ${node.debugSelector ?? ""} ${node.selector ?? ""} ${node.name ?? ""}`;
      const frame = node.bounds ?? node.frame ?? null;
      if (
        /profile|left-info/i.test(label) &&
        frame &&
        Number(frame.width ?? 0) > 0 &&
        Number(frame.height ?? 0) > 0
      ) {
        tipBounds = {
          x: Number(frame.x ?? frame.originX ?? 0),
          y: Number(frame.y ?? frame.originY ?? 0),
          width: Number(frame.width),
          height: Number(frame.height),
        };
        break;
      }
      stack.push(...Object.values(node));
    }
    const hoverY = tipBounds
      ? tipBounds.y + tipBounds.height / 2
      : 16; // overlay is 32pt tall; the tip row is vertically centered
    const sweepXs = tipBounds
      ? [tipBounds.x + Math.min(tipBounds.width / 2, 60)]
      : [20, 32, 48, 64, 84, 110, 140];
    if (!tipBounds) tipBoundsSource = "left-lane-sweep";

    const restPath = join(outDir, "tip-rest.png");
    const hoverPath = join(outDir, "tip-hover.png");
    // Rest frame: pointer parked in the empty flex lane between the tip and
    // the centered "Do in Current App" button (x=360 lands on that button's
    // own hover pill and would contaminate the rest frame).
    await driver.simulateGpuiEvent(
      { type: "mouseMove", x: 250, y: hoverY },
      { target, timeoutMs: 10_000 },
    );
    await driver.waitForSettle();
    const rest: any = await driver.captureScreenshot({
      target,
      hiDpi: true,
      savePath: restPath,
      timeoutMs: 15_000,
    });
    const restSha = existsSync(restPath)
      ? createHash("sha256").update(readFileSync(restPath)).digest("hex")
      : null;

    let hover: any = null;
    let hoverSha: string | null = null;
    let hoverPoint: { x: number; y: number } | null = null;
    for (const x of sweepXs) {
      await driver.simulateGpuiEvent(
        { type: "mouseMove", x, y: hoverY },
        { target, timeoutMs: 10_000 },
      );
      await driver.waitForSettle();
      hover = await driver.captureScreenshot({
        target,
        hiDpi: true,
        savePath: hoverPath,
        timeoutMs: 15_000,
      });
      hoverSha = existsSync(hoverPath)
        ? createHash("sha256").update(readFileSync(hoverPath)).digest("hex")
        : null;
      hoverPoint = { x, y: hoverY };
      if (hover?.error == null && hoverSha != null && restSha != null && hoverSha !== restSha) {
        break;
      }
    }

    const capturedBoth =
      rest?.error == null && hover?.error == null && restSha != null && hoverSha != null;
    receipt.hoverProof = {
      tipBounds,
      tipBoundsSource,
      hoverPoint,
      sweepXs,
      restPath,
      hoverPath,
      restSha256: restSha,
      hoverSha256: hoverSha,
      restError: rest?.error ?? null,
      hoverError: hover?.error ?? null,
      classification: capturedBoth ? "captured" : "screenshot-blocked",
      framesDiffer: capturedBoth ? hoverSha !== restSha : null,
    };
    if (capturedBoth && hoverSha === restSha) {
      throw new Error(
        "hover frames are pixel-identical to the rest frame across the sweep: the tip's hover pill did not paint",
      );
    }
  }

  const spaceCapture: any = captures.space;
  const quickAiCapture: any = captures["quick-ai"];
  receipt.protocolPass =
    spaceCapture?.keycap === "Space" &&
    quickAiCapture?.keycap === "⌘;" &&
    spaceCapture?.modelName === "opens Today" &&
    quickAiCapture?.modelName === "twice for Quick AI";
  receipt.visualPass = Object.values(captures).every((capture: any) =>
    capture?.screenshot?.error == null &&
    capture?.screenshot?.exists === true &&
    Number(capture?.screenshot?.width ?? 0) > 0 &&
    Number(capture?.screenshot?.height ?? 0) > 0
  );
  const hoverProof: any = receipt.hoverProof;
  receipt.hoverPass =
    hoverProof?.classification === "captured"
      ? hoverProof?.framesDiffer === true
      : "screenshot-blocked";
  receipt.pass =
    receipt.protocolPass === true
      && receipt.visualPass === true
      && receipt.hoverPass !== false;
  if (receipt.pass !== true) {
    throw new Error(
      `footer proof incomplete: protocolPass=${String(receipt.protocolPass)} visualPass=${String(receipt.visualPass)} hoverPass=${String(receipt.hoverPass)}`,
    );
  }
} catch (error) {
  receipt.pass = false;
  receipt.error = String(error);
  process.exitCode = 1;
} finally {
  let cleanupState: any = null;
  try {
    driver.send({ type: "hide" });
    await driver.waitForState({ windowVisible: false }, { timeoutMs: 10_000 });
    cleanupState = await driver.getState();
  } catch {
    cleanupState = null;
  }
  receipt.cleanup = {
    windowVisible: cleanupState?.windowVisible ?? null,
  };
  await driver.close();
  receipt.cleanedUp = true;
}

const receiptPath = join(outDir, "receipt.json");
writeFileSync(receiptPath, `${JSON.stringify(receipt, null, 2)}\n`);
console.log(JSON.stringify({
  receiptPath,
  binary: receipt.binary,
  binarySha256: receipt.binarySha256,
  sessionDir: receipt.sessionDir,
  sandboxTips: receipt.sandboxTips,
  rotations: receipt.rotations,
  captures: Object.fromEntries(Object.entries(captures).map(([name, capture]: [string, any]) => [
    name,
    {
      modelName: capture.modelName,
      keycap: capture.keycap,
      footerWindow: capture.footerWindow,
      screenshot: capture.screenshot,
    },
  ])),
  protocolPass: receipt.protocolPass,
  visualPass: receipt.visualPass,
  hoverProof: receipt.hoverProof,
  hoverPass: receipt.hoverPass,
  pass: receipt.pass,
  error: receipt.error,
  cleanup: receipt.cleanup,
  cleanedUp: receipt.cleanedUp,
}, null, 2));
