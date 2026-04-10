/**
 * write-storybook-fixture.ts
 *
 * Copies a screenshot PNG into the runtime-fixture directory and writes the
 * companion JSON manifest that the Storybook runtime-fixture host expects.
 *
 * Usage:
 *   bun scripts/agentic/write-storybook-fixture.ts \
 *       <pngSource> <surface> <variantId> <automationKind> <semanticSurface>
 *
 * Example:
 *   bun scripts/agentic/write-storybook-fixture.ts \
 *       test-screenshots/main-menu-current.png \
 *       main-menu current-main-menu main scriptList
 */

import { copyFileSync, mkdirSync, writeFileSync } from "node:fs";
import { join, relative } from "node:path";

type RuntimeFixtureManifest = {
  schemaVersion: 1;
  surface: string;
  variantId: string;
  imagePath: string;
  width: number;
  height: number;
  automationKind: string;
  semanticSurface: string;
};

async function getImageDimensions(
  filePath: string
): Promise<{ width: number; height: number }> {
  const proc = Bun.spawn(
    ["sips", "-g", "pixelWidth", "-g", "pixelHeight", filePath],
    { stdout: "pipe", stderr: "pipe" }
  );
  const out = await new Response(proc.stdout).text();
  await proc.exited;

  const width = Number(out.match(/pixelWidth:\s*(\d+)/)?.[1] ?? 0);
  const height = Number(out.match(/pixelHeight:\s*(\d+)/)?.[1] ?? 0);

  if (width <= 0 || height <= 0) {
    throw new Error(`Unable to read dimensions for ${filePath}`);
  }

  return { width, height };
}

async function main() {
  const [pngSource, surface, variantId, automationKind, semanticSurface] =
    Bun.argv.slice(2);

  if (
    !pngSource ||
    !surface ||
    !variantId ||
    !automationKind ||
    !semanticSurface
  ) {
    throw new Error(
      "Usage: bun scripts/agentic/write-storybook-fixture.ts <pngSource> <surface> <variantId> <automationKind> <semanticSurface>"
    );
  }

  const projectRoot = process.cwd();
  const fixtureDir = join(
    projectRoot,
    "test-screenshots",
    "storybook-fixtures",
    surface
  );
  mkdirSync(fixtureDir, { recursive: true });

  const pngPath = join(fixtureDir, `${variantId}.png`);
  copyFileSync(pngSource, pngPath);

  const { width, height } = await getImageDimensions(pngPath);

  const manifestPath = join(fixtureDir, `${variantId}.json`);
  const manifest: RuntimeFixtureManifest = {
    schemaVersion: 1,
    surface,
    variantId,
    imagePath: relative(projectRoot, pngPath),
    width,
    height,
    automationKind,
    semanticSurface,
  };
  writeFileSync(manifestPath, JSON.stringify(manifest, null, 2) + "\n");

  console.log(
    JSON.stringify(
      {
        event: "storybook_fixture_written",
        ok: true,
        surface,
        variantId,
        pngPath: relative(projectRoot, pngPath),
        manifestPath: relative(projectRoot, manifestPath),
        width,
        height,
      },
      null,
      2
    )
  );
}

main().catch((error) => {
  console.error(
    JSON.stringify(
      {
        event: "storybook_fixture_written",
        ok: false,
        error: String(error),
      },
      null,
      2
    )
  );
  process.exit(1);
});
