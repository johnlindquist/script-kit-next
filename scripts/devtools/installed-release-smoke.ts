import { assertNoninteractiveVisualProbe } from "./lib/operator-safety";
import { runOnHostedMac } from "./lib/installed-release-host";
import { SmokeFault } from "./lib/installed-release-contract";

if (import.meta.main) {
  try {
    assertNoninteractiveVisualProbe("installed-release-smoke");
    const r = await runOnHostedMac(async (kit) => {
      const { runInstalledJourneys } = await import("./installed-release-journeys");
      return runInstalledJourneys(kit);
    });
    console.log(
      JSON.stringify({
        status: r.status,
        code: r.code,
        tag: r.tag,
        mode: r.mode,
        receipt: "installed-release-receipt.json",
      }),
    );
    if (r.status !== "pass") process.exitCode = 1;
  } catch (e) {
    // Pre-attestation refusal must not write anything, not even a failure artifact.
    const code = e instanceof SmokeFault
      ? e.code
      : e instanceof Error && e.name === "NoninteractiveSafetyError"
      ? "noninteractive_refusal"
      : "preflight_or_receipt_unavailable";
    console.error(JSON.stringify({ status: "blocked", code }));
    process.exitCode = 1;
  }
}
