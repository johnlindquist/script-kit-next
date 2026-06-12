/**
 * T11 push-to-talk dictation into Day Page runtime proof:
 * - simulated hold (down ≥ HOLD_MS) → Day Page surface + listening/unavailable chrome
 * - pushDictationResult with dayPage target → timestamped capture in editor
 *
 * Usage:
 *   PROBE_BINARY=target-agent/artifacts/t11-dictation/script-kit-gpui \
 *     bun scripts/agentic/day-page-hold-dictation-probe.ts
 */
import { Driver } from "../devtools/driver";

const BINARY =
  process.env.PROBE_BINARY ??
  "target-agent/artifacts/t11-dictation/script-kit-gpui";

type Json = Record<string, unknown>;
const receipts: Record<string, Json> = {};
const failures: string[] = [];

function check(name: string, ok: boolean, detail: Json) {
  receipts[name] = { ok, ...detail };
  if (!ok) failures.push(name);
}

async function simulateMainHotkeyGesture(
  driver: Driver,
  phase: "down" | "up",
  requestId: string,
): Promise<Json> {
  return driver.request(
    {
      type: "simulateMainHotkeyGesture",
      phase,
      requestId,
    },
    { expect: "externalCommandResult", timeoutMs: 5000 },
  ) as Promise<Json>;
}

async function getEditorText(driver: Driver): Promise<string | null> {
  const elements = (await driver.request(
    { type: "getElements", target: { id: "main" } },
    { timeoutMs: 5000 },
  )) as Json;
  const list = (elements.elements ?? []) as Json[];
  const editor = list.find((el) => el.id === "day-page-editor");
  return (editor?.value as string | undefined) ?? null;
}

async function hasElement(driver: Driver, id: string): Promise<boolean> {
  const elements = (await driver.request(
    { type: "getElements", target: { id: "main" } },
    { timeoutMs: 5000 },
  )) as Json;
  const list = (elements.elements ?? []) as Json[];
  return list.some((el) => el.id === id);
}

let globalDriver: Driver | null = null;

try {
  const driver = await Driver.launch({
    binary: BINARY,
    sandboxHome: true,
    sessionName: "day-page-hold-dictation-probe",
    env: { SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1" },
  });
  globalDriver = driver;

  await simulateMainHotkeyGesture(driver, "down", "hold-down");
  await Bun.sleep(320);
  await simulateMainHotkeyGesture(driver, "up", "hold-up");
  await driver.waitForState({ windowVisible: true }, { timeoutMs: 8000 });
  await Bun.sleep(400);

  const stateAfterHold = (await driver.getState({ timeoutMs: 5000 })) as Json;
  check("hold_opens_day_page_surface", stateAfterHold.semanticSurface === "dayPage", {
    semanticSurface: stateAfterHold.semanticSurface,
  });

  const listening = await hasElement(driver, "day-page-dictation-listening");
  const unavailable = await hasElement(driver, "day-page-dictation-unavailable");
  check(
    "hold_shows_dictation_chrome",
    listening || unavailable,
    { listening, unavailable },
  );

  await driver.request(
    {
      type: "pushDictationResult",
      transcript: "hold thought from probe",
      target: "dayPage",
      requestId: "day-page-dictation-probe",
    },
    { expect: "externalCommandResult", timeoutMs: 8000 },
  );
  await Bun.sleep(500);

  const editorText = await getEditorText(driver);
  check(
    "transcript_lands_in_day_page_editor",
    editorText?.includes("hold thought from probe") === true,
    { editorText },
  );
  check(
    "timestamped_capture_line_present",
    /\d{2}:\d{2}\s+hold thought from probe/.test(editorText ?? ""),
    { editorText },
  );

  console.log(JSON.stringify({ ok: failures.length === 0, failures, receipts }, null, 2));
  await driver.close();
  process.exit(failures.length === 0 ? 0 : 1);
} catch (error) {
  console.error(error);
  if (globalDriver) await globalDriver.close().catch(() => {});
  process.exit(1);
}
