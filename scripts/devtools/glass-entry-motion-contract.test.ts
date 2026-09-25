import { describe, expect, test } from "bun:test";
import {
  ACTIONS_GLASS_ENTRY_EXPECTATION,
  MAIN_GLASS_ENTRY_EXPECTATION,
  analyzeEntryFadeLog,
  analyzeEntryMotionEnvelope,
} from "./glass-entry-motion-contract.ts";

type Bounds = [[number, number], [number, number]];

const settled: Bounds = [[381, 166], [750, 480]];

function frame(sequence: number, alpha: number, bounds: Bounds = settled) {
  return {
    sequence,
    displayTimeNs: 1_000_000_000 + sequence * 16_666_667,
    windowAlpha: alpha,
    windowBounds: bounds,
  };
}

/** A healthy 90ms sharp_presence capture at 60Hz: 0.99 → 1.0, frame fixed. */
const fadeFrames = [
  frame(0, 0.99),
  frame(1, 0.994),
  frame(2, 0.997),
  frame(3, 0.999),
  frame(4, 1),
  frame(5, 1),
];

const mainLog =
  "event=glass_morph window=Main window variant=fade phase=enter surface_profile=main preset=sharp_presence direction=fixed_frame duration_ns=90000000 width_scale=1 height_scale=1 window_start_alpha=0.9900 material_start_alpha=0.8000 content_start_alpha=0.8500 entry_blur_radius=0.00 foreground_blur_radius=0.00 foreground_blurred_root_count=0 foreground_root_count=1 footer_capsule_count=2 footer_blurred_capsule_count=0 footer_material_ramp_count=2 start_alpha=0.9900 start_alpha_bits=3fefae147ae147ae settle_duration_ns=90000000 configured_at_host_time_ns=100000000 expected_settle_deadline_ns=190000000 final_width_pt=750.00 final_height_pt=480.00 final_alpha=1 curve=easeOut";

describe("fixed-frame glass entry envelope", () => {
  test("accepts a fade that keeps the settled frame", () => {
    const result = analyzeEntryMotionEnvelope(
      fadeFrames,
      settled,
      MAIN_GLASS_ENTRY_EXPECTATION,
    );
    expect(result.errors).toEqual([]);
    expect(result.pass).toBe(true);
    expect(result.distinctWidths).toBe(1);
    expect(result.firstVisible?.alpha).toBeCloseTo(0.99, 6);
  });

  test("the Actions expectation judges rendered frames the same way", () => {
    const result = analyzeEntryMotionEnvelope(
      fadeFrames,
      settled,
      ACTIONS_GLASS_ENTRY_EXPECTATION,
    );
    expect(result.pass).toBe(true);
  });

  test("rejects the retired 101.2% shrink-in start", () => {
    const frames = [
      frame(0, 0.85, [[376.5, 166], [759, 480]]),
      ...fadeFrames.slice(1),
    ];
    const result = analyzeEntryMotionEnvelope(
      frames,
      settled,
      MAIN_GLASS_ENTRY_EXPECTATION,
    );
    expect(result.pass).toBe(false);
    expect(result.errors.join("\n")).toContain("must keep the settled frame");
  });

  test("rejects the retired 98.8% grow-in start", () => {
    const frames = [
      frame(0, 0.99, [[385.5, 166], [741, 480]]),
      ...fadeFrames.slice(1),
    ];
    const result = analyzeEntryMotionEnvelope(
      frames,
      settled,
      ACTIONS_GLASS_ENTRY_EXPECTATION,
    );
    expect(result.pass).toBe(false);
  });

  test("rejects height movement", () => {
    const frames = fadeFrames.map((entry, index) =>
      index === 2 ? frame(2, entry.windowAlpha, [[381, 166], [750, 486]]) : entry
    );
    const result = analyzeEntryMotionEnvelope(
      frames,
      settled,
      MAIN_GLASS_ENTRY_EXPECTATION,
    );
    expect(result.pass).toBe(false);
  });

  test("tolerates one point of backing-scale rounding", () => {
    const frames = fadeFrames.map((entry, index) =>
      index === 1 ? frame(1, entry.windowAlpha, [[381, 166], [750.5, 480]]) : entry
    );
    const result = analyzeEntryMotionEnvelope(
      frames,
      settled,
      MAIN_GLASS_ENTRY_EXPECTATION,
    );
    expect(result.pass).toBe(true);
  });

  test("rejects a first visible frame below the alpha floor", () => {
    const frames = [frame(0, 0.6), ...fadeFrames.slice(1)];
    const result = analyzeEntryMotionEnvelope(
      frames,
      settled,
      MAIN_GLASS_ENTRY_EXPECTATION,
    );
    expect(result.pass).toBe(false);
    expect(result.errors.join("\n")).toContain("below the 0.85 floor");
  });

  test("rejects alpha stepping backward mid-fade", () => {
    const frames = [
      frame(0, 0.99),
      frame(1, 1),
      frame(2, 0.9),
      frame(3, 1),
    ];
    const result = analyzeEntryMotionEnvelope(
      frames,
      settled,
      MAIN_GLASS_ENTRY_EXPECTATION,
    );
    expect(result.pass).toBe(false);
    expect(result.errors.join("\n")).toContain("stepped backward");
  });

  test("flags under-resolved captures instead of passing them", () => {
    const result = analyzeEntryMotionEnvelope(
      fadeFrames.slice(0, 2),
      settled,
      MAIN_GLASS_ENTRY_EXPECTATION,
    );
    expect(result.underResolved).toBe(true);
    expect(result.pass).toBe(false);
  });

  test("missing settled bounds fail closed", () => {
    const result = analyzeEntryMotionEnvelope(
      fadeFrames,
      null,
      MAIN_GLASS_ENTRY_EXPECTATION,
    );
    expect(result.pass).toBe(false);
    expect(result.underResolved).toBe(false);
  });
});

describe("logged entry fade", () => {
  test("accepts the main sharp_presence fade line", () => {
    const result = analyzeEntryFadeLog([mainLog], MAIN_GLASS_ENTRY_EXPECTATION);
    expect(result.errors).toEqual([]);
    expect(result.pass).toBe(true);
  });

  test("uses the most recent entry line", () => {
    const stale = mainLog.replace("duration_ns=90000000", "duration_ns=235000000");
    const result = analyzeEntryFadeLog([stale, mainLog], MAIN_GLASS_ENTRY_EXPECTATION);
    expect(result.pass).toBe(true);
  });

  test("a secondary line must say surface_profile=secondary", () => {
    const secondary = mainLog
      .replace("window=Main window", "window=Actions popup")
      .replace("surface_profile=main", "surface_profile=secondary");
    expect(analyzeEntryFadeLog([secondary], ACTIONS_GLASS_ENTRY_EXPECTATION).pass).toBe(true);
    expect(analyzeEntryFadeLog([secondary], MAIN_GLASS_ENTRY_EXPECTATION).pass).toBe(false);
  });

  test("an old width-travel line fails closed", () => {
    const old =
      "event=glass_morph window=Main window variant=main_preset phase=enter surface_profile=main preset=sharp_presence direction=shrink_in duration_ns=235000000 compression_ns=115000000 start_width_scale=1.012 squish_width_scale=0.987";
    const result = analyzeEntryFadeLog([old], MAIN_GLASS_ENTRY_EXPECTATION);
    expect(result.pass).toBe(false);
    expect(result.errors).toContain("variant is main_preset, expected fade");
    expect(result.errors).toContain("width_scale is missing, expected 1");
  });

  test("a defocus larger than the preset's fails", () => {
    const blurred = mainLog.replace("entry_blur_radius=0.00", "entry_blur_radius=8.00");
    expect(analyzeEntryFadeLog([blurred], MAIN_GLASS_ENTRY_EXPECTATION).pass).toBe(false);
  });

  test("no entry line fails closed", () => {
    const result = analyzeEntryFadeLog(
      ["event=glass_morph_skip window=Main window phase=enter reason=no_glass_view"],
      MAIN_GLASS_ENTRY_EXPECTATION,
    );
    expect(result.pass).toBe(false);
    expect(result.errors).toEqual(["no entry fade log line"]);
  });
});
