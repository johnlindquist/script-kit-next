import "../../../scripts/kit-sdk";

import { mkdirSync, writeFileSync } from "fs";
import { join } from "path";

type JsonObject = Record<string, any>;

const root = process.cwd();
const receiptDir = join(root, "artifacts/liquid-glass/receipts");
const screenshotDir = join(root, "artifacts/liquid-glass/screenshots");
mkdirSync(receiptDir, { recursive: true });
mkdirSync(screenshotDir, { recursive: true });

const label = "window-priority-prompt-div-current";
const screenshotPath = join(screenshotDir, `${label}.png`);
const layoutPath = join(receiptDir, `${label}-layout-sdk.json`);
const screenshotReceiptPath = join(receiptDir, `${label}-screenshot-sdk.json`);

function asNumber(value: unknown, fallback = 0) {
  return typeof value === "number" && Number.isFinite(value) ? value : fallback;
}

function visualAudit(components: JsonObject[]) {
  const controlsWithHitFailures: JsonObject[] = [];
  const contentGlassNodes: string[] = [];
  const missingStyleNodeNames: string[] = [];
  const chromeLayers: Record<string, number> = {};

  for (const component of components) {
    const name = String(component.name ?? component.type ?? "unknown");
    const style = component.visualStyle;
    if (!style || typeof style !== "object") {
      missingStyleNodeNames.push(name);
      continue;
    }

    const layer = String(style.chromeLayer ?? "unknown");
    chromeLayers[layer] = (chromeLayers[layer] ?? 0) + 1;

    const material = String(style.materialToken ?? style.material ?? "").toLowerCase();
    if (layer === "content" && (material.includes("glass") || material.includes("liquid"))) {
      contentGlassNodes.push(name);
    }

    const hitBounds = style.hitBounds ?? component.bounds ?? {};
    const isControl = /button|input|footer|action|close|search|field/i.test(name);
    if (isControl) {
      const width = asNumber(hitBounds.width);
      const height = asNumber(hitBounds.height);
      if (width < 28 || height < 28) {
        controlsWithHitFailures.push({ name, width, height, minimum: 28 });
      }
    }
  }

  const styledNodeCount = components.filter((component) => component.visualStyle).length;
  return {
    nodeCount: components.length,
    styledNodeCount,
    unstyledNodeCount: components.length - styledNodeCount,
    controlsWithHitFailures,
    contentGlassNodes,
    missingStyleNodeNames,
    chromeLayers,
  };
}

const promptPromise = div(`
<section class="p-6 space-y-3">
  <h1 class="text-xl font-semibold">Div Prompt Liquid Glass Proof</h1>
  <p>This prompt stays at the standard 750x500 shell for window/container measurement.</p>
  <p>It intentionally uses plain content so the proof focuses on the outer prompt shell, footer, material, and container metrics.</p>
  <button class="px-4 py-2 rounded">Measured Content Button</button>
</section>
`);

setTimeout(async () => {
  try {
    const layout = await getLayoutInfo();
    const components = Array.isArray(layout.components) ? layout.components : [];
    const receipt = {
      schemaVersion: 1,
      status: "ok",
      classification: "sdk-runtime-proof",
      label,
      source: "artifacts/liquid-glass/scripts/prompt-div-proof.ts",
      target: {
        surfaceKind: "PromptEntity",
        appViewVariant: "DivPrompt",
        nativeFooterSurface: "div_prompt",
      },
      window: {
        rect: {
          x: 0,
          y: 0,
          width: layout.windowWidth,
          height: layout.windowHeight,
        },
        promptType: layout.promptType,
      },
      componentCount: components.length,
      visualAudit: visualAudit(components),
      nodes: components,
      rawLayout: layout,
    };

    writeFileSync(layoutPath, `${JSON.stringify(receipt, null, 2)}\n`);

    try {
      const screenshot = await captureScreenshot();
      writeFileSync(screenshotPath, Buffer.from(screenshot.data, "base64"));
      writeFileSync(
        screenshotReceiptPath,
        `${JSON.stringify(
          {
            schemaVersion: 1,
            status: "ok",
            classification: "sdk-runtime-screenshot",
            label,
            screenshot: {
              path: screenshotPath,
              width: screenshot.width,
              height: screenshot.height,
            },
          },
          null,
          2,
        )}\n`,
      );
    } catch (error) {
      writeFileSync(
        screenshotReceiptPath,
        `${JSON.stringify(
          {
            schemaVersion: 1,
            status: "error",
            classification: "sdk-runtime-screenshot-failed",
            label,
            screenshot: {
              path: screenshotPath,
            },
            error: String(error),
          },
          null,
          2,
        )}\n`,
      );
    }

    setTimeout(() => submit(), 20000);
  } catch (error) {
    writeFileSync(
      layoutPath,
      `${JSON.stringify(
        {
          schemaVersion: 1,
          status: "error",
          classification: "sdk-runtime-proof-failed",
          label,
          error: String(error),
        },
        null,
        2,
      )}\n`,
    );
    process.exit(1);
  }
}, 1200);

await promptPromise;
process.exit(0);
