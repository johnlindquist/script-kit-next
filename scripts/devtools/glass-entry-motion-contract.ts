/**
 * Glass entry-motion contract — the fixed-frame fade.
 *
 * Every window keeps its settled frame. Entry is one grouped ease-out fade:
 * glass material, content, and NSWindow alpha rise together to 1.0 over the
 * preset's duration. The default preset (`sharp_presence`) runs 90ms from
 * window alpha 0.99, material 0.8, and content 0.85 with no defocus.
 *
 * Rendered frames are judged by `analyzeEntryMotionEnvelope`: every frame at
 * the settled size, the first visible frame at or above the 0.85 alpha floor,
 * and alpha never stepping backward. `analyzeEntryFadeLog` checks the values
 * the entry logged against the same expectation; it is a diagnostic, never a
 * substitute for rendered evidence.
 */

export type NativeWindowBounds =
  | [[number, number], [number, number]]
  | null
  | undefined;

type EntryFrame = {
  sequence?: number;
  displayTimeNs?: number | null;
  windowAlpha?: number | null;
  windowBounds?: NativeWindowBounds;
};

export const MAIN_GLASS_ENTRY_EXPECTATION = {
  presetId: "sharp_presence",
  surfaceProfile: "main",
  direction: "fixed_frame",
  durationMs: 90,
  windowStartAlpha: 0.99,
  materialStartAlpha: 0.8,
  contentStartAlpha: 0.85,
  entryBlurRadius: 0,
  foregroundBlurRadius: 0,
  alphaFloor: 0.85,
  minimumResolvedFrames: 3,
} as const;

export const ACTIONS_GLASS_ENTRY_EXPECTATION = {
  ...MAIN_GLASS_ENTRY_EXPECTATION,
  surfaceProfile: "secondary",
} as const;

export type GlassEntryExpectation =
  | typeof MAIN_GLASS_ENTRY_EXPECTATION
  | typeof ACTIONS_GLASS_ENTRY_EXPECTATION;

function size(bounds: NativeWindowBounds) {
  if (
    !Array.isArray(bounds)
    || !Array.isArray(bounds[1])
    || !Number.isFinite(Number(bounds[1][0]))
    || !Number.isFinite(Number(bounds[1][1]))
  ) {
    return null;
  }
  return {
    width: Number(bounds[1][0]),
    height: Number(bounds[1][1]),
  };
}

/**
 * Filmstrip capture analysis against a named expectation.
 *
 * Returns `underResolved: true` (instead of a product verdict) when the
 * damage-driven capture produced fewer frames than a fade needs to be seen.
 */
export function analyzeEntryMotionEnvelope(
  frames: EntryFrame[],
  settledBounds: NativeWindowBounds,
  expectation: GlassEntryExpectation,
) {
  const settled = size(settledBounds);
  const errors: string[] = [];
  if (!settled || settled.width <= 0 || settled.height <= 0) {
    return {
      pass: false,
      underResolved: false,
      errors: ["settled native bounds are missing"],
      frames: [],
    };
  }

  const measured = frames.flatMap((frame) => {
    const measuredSize = size(frame.windowBounds);
    const alpha = Number(frame.windowAlpha);
    if (!measuredSize || !Number.isFinite(alpha) || alpha < 0.001) return [];
    return [{
      sequence: frame.sequence ?? null,
      displayTimeNs: frame.displayTimeNs ?? null,
      alpha,
      widthScale: measuredSize.width / settled.width,
      heightScale: measuredSize.height / settled.height,
    }];
  });
  const first = measured[0];
  const distinctWidths = new Set(
    measured.map((frame) => frame.widthScale.toFixed(4)),
  ).size;
  const underResolved = measured.length < expectation.minimumResolvedFrames;

  // One point of slack on either axis absorbs backing-scale rounding.
  const widthTolerance = Math.max(0.001, 1 / settled.width);
  const heightTolerance = Math.max(0.001, 1 / settled.height);
  const moved = measured.find((frame) =>
    Math.abs(frame.widthScale - 1) > widthTolerance
    || Math.abs(frame.heightScale - 1) > heightTolerance
  );
  if (moved) {
    errors.push(
      `frame ${moved.sequence ?? "?"} is ${(moved.widthScale * 100).toFixed(2)}% × ${(moved.heightScale * 100).toFixed(2)}% of settled; the entry must keep the settled frame`,
    );
  }
  if (first != null && first.alpha < expectation.alphaFloor - 0.01) {
    errors.push(
      `first visible alpha ${first.alpha.toFixed(3)} is below the ${expectation.alphaFloor} floor`,
    );
  }
  for (let index = 1; index < measured.length; index += 1) {
    const previous = measured[index - 1]!;
    const current = measured[index]!;
    if (current.alpha < previous.alpha - 0.01) {
      errors.push(
        `alpha stepped backward from ${previous.alpha.toFixed(3)} to ${current.alpha.toFixed(3)} at frame ${current.sequence ?? index}`,
      );
      break;
    }
  }

  return {
    expectation,
    settled,
    firstVisible: first ?? null,
    minimumWidthScale: measured.length
      ? Math.min(...measured.map((frame) => frame.widthScale))
      : null,
    maximumWidthScale: measured.length
      ? Math.max(...measured.map((frame) => frame.widthScale))
      : null,
    minimumHeightScale: measured.length
      ? Math.min(...measured.map((frame) => frame.heightScale))
      : null,
    maximumHeightScale: measured.length
      ? Math.max(...measured.map((frame) => frame.heightScale))
      : null,
    distinctWidths,
    measuredFrameCount: measured.length,
    underResolved,
    frames: measured,
    errors,
    pass: errors.length === 0 && !underResolved,
  };
}

function parseExactField(line: string, key: string): number | null {
  const match = line.match(new RegExp(`(?:^| )${key}=([0-9.]+)(?= |$)`));
  return match ? Number(match[1]) : null;
}

function parseTextField(line: string, key: string): string | null {
  const match = line.match(new RegExp(`(?:^| )${key}=([A-Za-z0-9_-]+)(?= |$)`));
  return match ? match[1] : null;
}

/**
 * Check the most recent `event=glass_morph ... phase=enter` line against a
 * named expectation. The logged defocus radius is what was actually
 * installed, so it may be lower than the preset's when the runtime lacks the
 * blur filter; it may never be higher.
 */
export function analyzeEntryFadeLog(
  logLines: string[],
  expectation: GlassEntryExpectation,
) {
  const line = [...logLines]
    .reverse()
    .find((candidate) =>
      candidate.includes("event=glass_morph ")
      && candidate.includes("phase=enter")
    ) ?? null;
  if (line === null) {
    return {
      pass: false,
      errors: ["no entry fade log line"],
      line: null,
      fields: null,
    };
  }
  const fields = {
    variant: parseTextField(line, "variant"),
    direction: parseTextField(line, "direction"),
    surfaceProfile: parseTextField(line, "surface_profile"),
    preset: parseTextField(line, "preset"),
    durationNs: parseExactField(line, "duration_ns"),
    widthScale: parseExactField(line, "width_scale"),
    heightScale: parseExactField(line, "height_scale"),
    windowStartAlpha: parseExactField(line, "window_start_alpha"),
    materialStartAlpha: parseExactField(line, "material_start_alpha"),
    contentStartAlpha: parseExactField(line, "content_start_alpha"),
    entryBlurRadius: parseExactField(line, "entry_blur_radius"),
    foregroundBlurRadius: parseExactField(line, "foreground_blur_radius"),
  };
  const errors: string[] = [];
  const expectText = (
    name: string,
    actual: string | null,
    expected: string,
  ) => {
    if (actual !== expected) errors.push(`${name} is ${actual ?? "missing"}, expected ${expected}`);
  };
  const expectNumber = (
    name: string,
    actual: number | null,
    expected: number,
    tolerance = 1e-4,
  ) => {
    if (actual === null || Math.abs(actual - expected) > tolerance) {
      errors.push(`${name} is ${actual ?? "missing"}, expected ${expected}`);
    }
  };
  expectText("variant", fields.variant, "fade");
  expectText("direction", fields.direction, expectation.direction);
  expectText("surface_profile", fields.surfaceProfile, expectation.surfaceProfile);
  expectText("preset", fields.preset, expectation.presetId);
  expectNumber("duration_ns", fields.durationNs, expectation.durationMs * 1_000_000, 0.5);
  expectNumber("width_scale", fields.widthScale, 1);
  expectNumber("height_scale", fields.heightScale, 1);
  expectNumber("window_start_alpha", fields.windowStartAlpha, expectation.windowStartAlpha);
  expectNumber("material_start_alpha", fields.materialStartAlpha, expectation.materialStartAlpha);
  expectNumber("content_start_alpha", fields.contentStartAlpha, expectation.contentStartAlpha);
  if (
    fields.entryBlurRadius === null
    || fields.entryBlurRadius > expectation.entryBlurRadius + 0.01
  ) {
    errors.push(
      `entry_blur_radius is ${fields.entryBlurRadius ?? "missing"}, expected at most ${expectation.entryBlurRadius}`,
    );
  }
  expectNumber("foreground_blur_radius", fields.foregroundBlurRadius, expectation.foregroundBlurRadius, 0.01);
  return {
    pass: errors.length === 0,
    errors,
    line,
    fields,
  };
}
