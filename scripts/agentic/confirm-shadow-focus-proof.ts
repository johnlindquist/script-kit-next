#!/usr/bin/env bun
import { existsSync, mkdirSync } from "node:fs";
import { basename, join, resolve } from "node:path";
import { Driver, type Json } from "../devtools/driver";

const repoRoot = resolve(import.meta.dir, "../..");
const binary =
  process.env.SCRIPT_KIT_GPUI_BINARY ??
  "target-agent/artifacts/confirm-shadow-layer/script-kit-gpui";
const screenshotDir = join(repoRoot, ".test-screenshots", "confirm-shadow-layer");
const sessionName = "confirm-shadow-layer-proof";

function asArray(value: unknown): Json[] {
  return Array.isArray(value) ? (value as Json[]) : [];
}

function popupFromList(list: Json): Json | null {
  return (
    asArray(list.windows).find(
      (window) =>
        window.id === "confirm-popup" ||
        (window.kind === "promptPopup" && window.semanticSurface === "confirmDialog"),
    ) ?? null
  );
}

async function captureScreen(name: string): Promise<Json> {
  mkdirSync(screenshotDir, { recursive: true });
  const path = join(screenshotDir, `${name}.png`);
  const proc = Bun.spawn(["screencapture", "-x", path], {
    cwd: repoRoot,
    stdout: "pipe",
    stderr: "pipe",
  });
  const [stdout, stderr, code] = await Promise.all([
    new Response(proc.stdout).text(),
    new Response(proc.stderr).text(),
    proc.exited,
  ]);
  return {
    path,
    basename: basename(path),
    exists: existsSync(path),
    exitCode: code,
    stdout: stdout.trim(),
    stderr: stderr.trim(),
  };
}

async function captureTarget(driver: Driver, name: string, target: Json): Promise<Json> {
  const path = join(screenshotDir, `${name}.png`);
  const result = await driver.captureScreenshot({ target, savePath: path, timeoutMs: 10_000 });
  return {
    path,
    basename: basename(path),
    exists: existsSync(path),
    width: result.width ?? null,
    height: result.height ?? null,
    error: result.error ?? null,
  };
}

async function main() {
  const receipt: Json = {
    schemaVersion: 1,
    verifier: "confirm-shadow-focus-proof",
    binary,
    sessionName,
    route: "main.quit.confirm",
    status: "fail",
    screenshots: {},
    samples: [],
  };

  const driver = await Driver.launch({
    sessionName,
    sandboxHome: true,
    binary,
    readyTimeoutMs: 15_000,
    defaultTimeoutMs: 8_000,
  });

  let driverClosed = false;
  try {
    driver.send({ type: "show", requestId: "confirm-shadow-show" });
    await driver.setFilterAndWait("quit", { timeoutMs: 8_000 });
    await Bun.sleep(300);

    const beforeState = await driver.getState({ timeoutMs: 8_000 });
    const beforeWindows = await driver.listAutomationWindows({ timeoutMs: 8_000 });
    receipt.before = {
      selectedValue: beforeState.selectedValue ?? null,
      focusedWindowId: beforeWindows.focusedWindowId ?? null,
      windows: asArray(beforeWindows.windows).map((window) => ({
        id: window.id,
        kind: window.kind,
        focused: window.focused,
        visible: window.visible,
        semanticSurface: window.semanticSurface,
        parentWindowId: window.parentWindowId ?? null,
      })),
    };
    receipt.screenshots.beforeScreen = await captureScreen("before-screen");
    receipt.screenshots.beforeMain = await captureTarget(driver, "before-main", {
      type: "id",
      id: "main",
    });

    driver.simulateKey("enter", []);
    let popup: Json | null = null;
    let popupWindows: Json | null = null;
    for (let attempt = 0; attempt < 20; attempt += 1) {
      popupWindows = await driver.listAutomationWindows({ timeoutMs: 8_000 });
      popup = popupFromList(popupWindows);
      receipt.samples.push({
        attempt,
        focusedWindowId: popupWindows.focusedWindowId ?? null,
        main: asArray(popupWindows.windows).find((window) => window.id === "main") ?? null,
        popup,
      });
      if (popup) break;
      await Bun.sleep(50);
    }

    if (!popup || !popupWindows) {
      throw new Error("confirm-popup did not open from main Quit route");
    }

    await Bun.sleep(150);
    const popupElements = await driver.getElements(
      { target: { type: "id", id: "confirm-popup" }, limit: 40 },
      { timeoutMs: 8_000 },
    );
    const openWindows = await driver.listAutomationWindows({ timeoutMs: 8_000 });
    receipt.open = {
      focusedWindowId: openWindows.focusedWindowId ?? null,
      popup,
      popupElements: {
        focusedSemanticId: popupElements.focusedSemanticId ?? null,
        buttons: asArray(popupElements.elements).filter((element) => element.type === "button"),
      },
    };
    receipt.screenshots.openScreen = await captureScreen("open-screen");
    receipt.screenshots.openMain = await captureTarget(driver, "open-main", {
      type: "id",
      id: "main",
    });
    receipt.screenshots.openPopup = await captureTarget(driver, "open-popup", {
      type: "id",
      id: "confirm-popup",
    });

    await Bun.sleep(1_000);
    const delayedWindows = await driver.listAutomationWindows({ timeoutMs: 8_000 });
    const delayedPopup = popupFromList(delayedWindows);
    const delayedPopupElements = delayedPopup
      ? await driver.getElements(
          { target: { type: "id", id: "confirm-popup" }, limit: 40 },
          { timeoutMs: 8_000 },
        )
      : null;
    receipt.delayedOpen = {
      focusedWindowId: delayedWindows.focusedWindowId ?? null,
      popup: delayedPopup,
      popupElements: delayedPopupElements
        ? {
            focusedSemanticId: delayedPopupElements.focusedSemanticId ?? null,
            buttons: asArray(delayedPopupElements.elements).filter(
              (element) => element.type === "button",
            ),
          }
        : null,
    };
    receipt.screenshots.delayedScreen = await captureScreen("delayed-screen");
    receipt.screenshots.delayedPopup = delayedPopup
      ? await captureTarget(driver, "delayed-popup", { type: "id", id: "confirm-popup" })
      : { exists: false, error: "confirm-popup missing after delayed visibility sample" };

    driver.simulateKey("tab", []);
    await Bun.sleep(150);
    const tabElements = await driver.getElements(
      { target: { type: "id", id: "confirm-popup" }, limit: 40 },
      { timeoutMs: 8_000 },
    );
    receipt.afterTab = {
      focusedSemanticId: tabElements.focusedSemanticId ?? null,
      buttons: asArray(tabElements.elements).filter((element) => element.type === "button"),
    };

    driver.simulateKey("escape", []);
    await Bun.sleep(250);
    const afterWindows = await driver.listAutomationWindows({ timeoutMs: 8_000 });
    const popupClosed = popupFromList(afterWindows) == null;
    receipt.after = {
      focusedWindowId: afterWindows.focusedWindowId ?? null,
      popupClosed,
      windows: asArray(afterWindows.windows).map((window) => ({
        id: window.id,
        kind: window.kind,
        focused: window.focused,
        visible: window.visible,
        semanticSurface: window.semanticSurface,
        parentWindowId: window.parentWindowId ?? null,
      })),
    };
    receipt.screenshots.afterScreen = await captureScreen("after-screen");

    const mainStayedFocused = receipt.samples.every(
      (sample: Json) => sample.focusedWindowId === "main",
    ) && receipt.delayedOpen.focusedWindowId === "main";
    const attachedToMain = popup.parentWindowId === "main";
    const popupIsConfirm = popup.semanticSurface === "confirmDialog";
    const popupStillVisibleAfterDelay =
      delayedPopup?.visible === true && delayedPopup?.parentWindowId === "main";
    const keyboardTabMovedFocus =
      receipt.open.popupElements.focusedSemanticId !== receipt.afterTab.focusedSemanticId;
    const keyboardEscapeClosed = popupClosed === true;
    const screenshotsExist = Object.values(receipt.screenshots).every(
      (shot) => Boolean((shot as Json).exists) && !(shot as Json).error,
    );

    receipt.status =
      mainStayedFocused &&
      attachedToMain &&
      popupIsConfirm &&
      popupStillVisibleAfterDelay &&
      keyboardTabMovedFocus &&
      keyboardEscapeClosed &&
      screenshotsExist
        ? "pass"
        : "fail";
    receipt.assertions = {
      mainStayedFocused,
      attachedToMain,
      popupIsConfirm,
      popupStillVisibleAfterDelay,
      keyboardTabMovedFocus,
      keyboardEscapeClosed,
      screenshotsExist,
      shadowDemotionObserved: "not-observed-in-delayed-sampled-screenshots",
    };
  } finally {
    await driver.close();
    driverClosed = true;
    receipt.cleanup = { driverClosed };
  }

  console.log(JSON.stringify(receipt, null, 2));
  if (receipt.status !== "pass") {
    process.exit(1);
  }
}

main().catch((error) => {
  console.error(error instanceof Error ? error.stack : String(error));
  process.exit(1);
});
