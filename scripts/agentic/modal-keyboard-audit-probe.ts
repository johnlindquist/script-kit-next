#!/usr/bin/env bun
/**
 * Modal audit — runtime proof, keyboard only.
 *
 * Every key goes through GPUI's real input pipeline (`simulateGpuiEvent`
 * keyDown targeted at the window macOS would deliver it to: the modal's
 * parent, because the shared confirm popup never becomes the key window).
 * That path runs keystroke interceptors, key bindings, element listeners,
 * and text input exactly as a physical key press does — unlike the legacy
 * `simulateKey` route, which calls surface methods directly and hid every
 * defect this probe exists to catch.
 *
 * For each modal it proves:
 *   - its risk reads right: a destructive or irreversible primary is marked
 *     danger (red), and an irreversible modal opens with Cancel focused so
 *     Enter never destroys anything unasked;
 *   - it is a real modal: a popup over its window, never a state that
 *     replaces the window's content;
 *   - it fits on screen: fully inside one display's visible area (menu bar
 *     and Dock excluded), over its parent;
 *   - Tab / ⇧Tab / ← / → move focus between the buttons (in a modal with a
 *     text field, ←/→ move the caret instead);
 *   - Enter activates the focused button, and only a bare Enter does
 *     (⌘↵ must never run a destructive primary);
 *   - Esc and ⌘W dismiss the modal and nothing else;
 *   - no key leaks to the window underneath (filter text, list selection,
 *     note text, composer text, Actions, Agent Chat);
 *   - after dismissal the keyboard is back where it was;
 *   - a modal closes with its parent window;
 *   - a second modal waits for the first instead of replacing it.
 *
 * Every modal is also measured from its painted layout: equal left/right
 * insets, equal top/bottom insets, parts inside the modal, one button row of
 * equal-height, non-overlapping, evenly spaced buttons at least 28pt tall,
 * and the same header/body/action gaps. A final `layout-consistency` entry
 * proves every measured modal uses the same insets, gaps, and button height.
 *
 * Reversible actions have no modal: deleting a script or a note happens at
 * once and ⌘Z undoes it; typing withdraws the offer.
 *
 * Surfaces: launcher confirm (Clear Suggested Items), launcher Delete Script
 * with Undo, Agent Chat sign-in consent, Notes delete with Undo and permanent
 * delete from the Trash, detached Agent Chat "Delete All History?", the
 * confirm fixture over the main window (and a queued second one), SDK
 * `confirm()` with a title and `irreversible`, the Add Alias editor, and the
 * main hotkey hiding a window with a modal open.
 * Screenshots (2x) of each modal land in .test-screenshots/modal-audit/.
 *
 * Run: bun scripts/agentic/modal-keyboard-audit-probe.ts [--receipt <path>]
 *      [--keep-sessions] [scenario ...]
 * Binary: SCRIPT_KIT_GPUI_BINARY, else target-agent/pools/agent-debug.
 */
import { Driver } from "../devtools/driver.ts";
import type { Json } from "../devtools/driver.ts";
import { assertNoninteractiveVisualProbe } from "../devtools/lib/operator-safety.ts";
import { existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";

assertNoninteractiveVisualProbe("modal-keyboard-audit");

const ROOT = resolve(import.meta.dir, "../..");
const args = process.argv.slice(2);
const receiptArg = args.indexOf("--receipt");
const receiptPath = receiptArg >= 0 ? args[receiptArg + 1] : undefined;
const keepSessions = args.includes("--keep-sessions");
const SCENARIOS = [
	"launcher-confirm",
	"launcher-undo",
	"agent-chat-consent",
	"notes-delete",
	"detached-chat-confirm",
	"fixture-confirm",
	"modal-variants",
	"sdk-confirm",
	"alias-editor",
	"hotkey-hides-modal",
] as const;
type Scenario = (typeof SCENARIOS)[number];
const requested = args.filter((arg): arg is Scenario => (SCENARIOS as readonly string[]).includes(arg));
const scenarios: readonly Scenario[] = requested.length ? requested : SCENARIOS;
const binary =
	process.env.SCRIPT_KIT_GPUI_BINARY ?? join(ROOT, "target-agent/pools/agent-debug/debug/script-kit-gpui");
const shotDir = join(ROOT, ".test-screenshots/modal-audit");
mkdirSync(shotDir, { recursive: true });

const MAIN = { type: "id", id: "main" } as const;
const NOTES = { type: "id", id: "notes" } as const;
const CONFIRM = { type: "id", id: "confirm-popup" } as const;

type Checks = Record<string, boolean>;
const receipt: Json = { probe: "modal-keyboard-audit", binary, scenarios: {} };
const sessionDirs: string[] = [];

interface Rect {
	x: number;
	y: number;
	width: number;
	height: number;
}

/**
 * Each display's visible area (menu bar and Dock excluded) in the app's
 * global coordinates: origin at the primary display's top-left, y down.
 */
function visibleScreens(): Rect[] {
	const script = `ObjC.import("AppKit");
const screens = $.NSScreen.screens; const out = [];
for (let i = 0; i < screens.count; i++) {
  const f = screens.objectAtIndex(i).visibleFrame; const full = screens.objectAtIndex(i).frame;
  out.push({ x: f.origin.x, y: f.origin.y, width: f.size.width, height: f.size.height, fullHeight: full.size.height });
}
JSON.stringify(out);`;
	const result = Bun.spawnSync(["osascript", "-l", "JavaScript", "-e", script]);
	const frames = JSON.parse(result.stdout.toString()) as Array<Rect & { fullHeight: number }>;
	const primaryHeight = frames[0]?.fullHeight ?? 0;
	return frames.map((frame) => ({
		x: frame.x,
		y: primaryHeight - (frame.y + frame.height),
		width: frame.width,
		height: frame.height,
	}));
}
const SCREENS = visibleScreens();
receipt.screens = SCREENS;

function contains(outer: Rect, inner: Rect): boolean {
	const slack = 0.5;
	return (
		inner.x >= outer.x - slack &&
		inner.y >= outer.y - slack &&
		inner.x + inner.width <= outer.x + outer.width + slack &&
		inner.y + inner.height <= outer.y + outer.height + slack
	);
}

function overlaps(a: Rect, b: Rect): boolean {
	return a.x < b.x + b.width && b.x < a.x + a.width && a.y < b.y + b.height && b.y < a.y + a.height;
}

async function launch(name: string): Promise<{ driver: Driver; kit: string }> {
	const driver = await Driver.launch({
		sessionName: `modal-audit-${name}`,
		binary,
		sandboxHome: true,
		sharedModels: false,
		env: {
			// Deterministic goose ACP shim that asks for a ChatGPT sign-in and
			// never approves it, so the consent modal is reachable offline.
			SCRIPT_KIT_GOOSE_BINARY: join(ROOT, "scripts/agentic/mock-goose-acp.ts"),
			MOCK_GOOSE_SIGN_IN: "1",
			MOCK_GOOSE_SIGN_IN_APPROVE_AFTER_MS: "600000",
			SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1",
		},
		readyTimeoutMs: 30_000,
		defaultTimeoutMs: 15_000,
	});
	sessionDirs.push(driver.sessionDir);
	const kit = join(driver.sessionDir, "home", ".scriptkit");
	// One-time first-run hint toasts would cover the surfaces under test.
	for (const marker of ["gesture-hint-tap-dismiss.json", "sediment-disclosure.json"]) {
		writeFileSync(join(kit, marker), "{}");
	}
	await driver.request({ type: "show" });
	await driver.waitForSettle();
	return { driver, kit };
}

async function until<T>(probe: () => Promise<T | null | undefined | false>, timeoutMs = 5_000): Promise<T | null> {
	const deadline = Date.now() + timeoutMs;
	while (Date.now() < deadline) {
		const value = await probe().catch(() => null);
		if (value) return value as T;
		await Bun.sleep(100);
	}
	return null;
}

/** The open confirm popup as the user sees it. */
interface ModalSnapshot {
	title: string;
	parent: string;
	bounds: Rect | null;
	/** "confirm", "secondary", or "cancel". */
	focused: string | null;
	/** Text of the modal's field, when it has one. */
	field: string | null;
	/** The field's problem, when its text cannot be submitted. */
	fieldProblem: string | null;
	primaryDisabled: boolean;
	/** The primary is painted as dangerous (red). */
	primaryDanger: boolean;
	/** "safe", "destructive", or "irreversible". */
	risk: string | null;
	/** Modals waiting behind this one. */
	waiting: number;
	buttons: string[];
}

interface MainSnapshot {
	view: string;
	input: string;
	selected: string | undefined;
	visible: boolean;
}

interface ModalHelpers {
	/** One real key press through GPUI's input pipeline on `target`. */
	key(name: string, modifiers?: string[], target?: Json, text?: string): Promise<void>;
	type(text: string, target?: Json): Promise<void>;
	windows(): Promise<Json[]>;
	modal(): Promise<ModalSnapshot | null>;
	main(): Promise<MainSnapshot>;
	settle(): Promise<void>;
}

function makeHelpers(driver: Driver): ModalHelpers {
	const key = async (name: string, modifiers: string[] = [], target: Json = MAIN, text?: string) => {
		await driver.simulateGpuiEvent(
			{ type: "keyDown", key: name, ...(modifiers.length ? { modifiers } : {}), ...(text ? { text } : {}) },
			{ target },
		);
		await Bun.sleep(120);
	};
	const type = async (text: string, target: Json = MAIN) => {
		for (const ch of text) await key(ch === " " ? "space" : ch, [], target, ch);
	};
	const windows = async (): Promise<Json[]> =>
		((await driver.listAutomationWindows({ timeoutMs: 5_000 })) as Json).windows ?? [];
	const modal = async (): Promise<ModalSnapshot | null> => {
		const popup = (await windows()).find((window) => window.id === "confirm-popup");
		if (!popup) return null;
		const snapshot = (await driver.getElements({ target: CONFIRM, limit: 40 }, { timeoutMs: 5_000 })) as Json;
		const elements = (snapshot.elements ?? []) as Json[];
		const field = elements.find((element) => element.semanticId === "input:confirm-field");
		const primary = elements.find((element) => element.semanticId === "button:0:confirm");
		const panel = elements.find((element) => element.semanticId === "panel:confirm-dialog");
		const focusedId: string = snapshot.focusedSemanticId ?? "";
		return {
			title: popup.title,
			parent: popup.parentWindowId,
			bounds: popup.bounds ?? null,
			focused: focusedId.split(":").at(-1) ?? null,
			field: field ? String(field.value ?? "") : null,
			fieldProblem: field?.actionDisabled ?? null,
			primaryDisabled: Boolean(primary?.actionDisabled),
			primaryDanger: primary?.statusKind === "danger",
			risk: panel?.kind ?? null,
			waiting: Number.parseInt(String(panel?.statusKind ?? "0"), 10) || 0,
			buttons: elements
				.filter((element) => String(element.semanticId).startsWith("button:"))
				.map((element) => String(element.text ?? "")),
		};
	};
	const main = async (): Promise<MainSnapshot> => {
		const state = (await driver.getState()) as Json;
		return {
			view: state.promptType,
			input: state.inputValue,
			selected: state.selectedValue,
			visible: state.windowVisible === true,
		};
	};
	const settle = () => Bun.sleep(300);
	return { key, type, windows, modal, main, settle };
}

/**
 * On-screen frames of `pid`'s windows from the macOS window server, in global
 * coordinates (origin at the primary display's top-left, y down). GPUI
 * reports window bounds relative to the window's own display, so the window
 * server is the ground truth for "is this on screen".
 */
async function windowServerFrames(pid: number): Promise<Rect[]> {
	const script = `ObjC.import("CoreGraphics"); ObjC.import("Foundation");
const list = ObjC.castRefToObject($.CGWindowListCopyWindowInfo($.kCGWindowListOptionOnScreenOnly, 0));
const out = [];
for (let i = 0; i < list.count; i++) {
  const w = list.objectAtIndex(i);
  if (ObjC.unwrap(w.objectForKey("kCGWindowOwnerPID")) !== ${pid}) continue;
  const b = w.objectForKey("kCGWindowBounds");
  out.push({ x: ObjC.unwrap(b.objectForKey("X")), y: ObjC.unwrap(b.objectForKey("Y")), width: ObjC.unwrap(b.objectForKey("Width")), height: ObjC.unwrap(b.objectForKey("Height")) });
}
JSON.stringify(out);`;
	const proc = Bun.spawn(["osascript", "-l", "JavaScript", "-e", script], { stdout: "pipe", stderr: "ignore" });
	const text = await new Response(proc.stdout).text();
	await proc.exited;
	return JSON.parse(text || "[]") as Rect[];
}

/** The window-server frame of a window with the given size. */
function frameOfSize(frames: Rect[], size: Rect | null): Rect | null {
	if (!size) return null;
	return (
		frames.find(
			(frame) => Math.abs(frame.width - size.width) <= 1 && Math.abs(frame.height - size.height) <= 1,
		) ?? null
	);
}

/**
 * A modal is a popup over its parent and fits on screen: fully inside one
 * display's visible area (menu bar and Dock excluded), overlapping its parent.
 */
async function checkPlacement(h: ModalHelpers, checks: Checks, prefix: string, receiptOut: Json) {
	const windows = await h.windows();
	const popup = windows.find((window) => window.id === "confirm-popup");
	const parent = windows.find((window) => window.id === popup?.parentWindowId);
	const frames = popup?.pid ? await windowServerFrames(popup.pid) : [];
	const modalFrame = frameOfSize(frames, popup?.bounds ?? null);
	const parentFrame = frameOfSize(frames, parent?.bounds ?? null);
	receiptOut[`${prefix}.frames`] = { modal: modalFrame, parent: parentFrame };
	checks[`${prefix}.fitsOnScreen`] = Boolean(
		modalFrame && SCREENS.some((screen) => contains(screen, modalFrame)),
	);
	checks[`${prefix}.overParent`] = Boolean(modalFrame && parentFrame && overlaps(modalFrame, parentFrame));
}

/**
 * Sample every one of `pid`'s on-screen window frames while a modal appears
 * (its entry animation runs ~300ms) and report any that leave the displays.
 */
async function sampleFramesDuringEntry(pid: number, durationMs = 700): Promise<{ samples: number; offScreen: Rect[] }> {
	const deadline = Date.now() + durationMs;
	let samples = 0;
	const offScreen: Rect[] = [];
	while (Date.now() < deadline) {
		for (const frame of await windowServerFrames(pid)) {
			samples += 1;
			if (!SCREENS.some((screen) => contains(screen, frame))) offScreen.push(frame);
		}
		await Bun.sleep(10);
	}
	return { samples, offScreen };
}

async function appPid(h: ModalHelpers): Promise<number> {
	return Number((await h.windows()).find((window) => window.pid)?.pid ?? 0);
}

async function shot(driver: Driver, scenario: string, name: string, target: Json, receiptOut: Json) {
	const savePath = join(shotDir, `${scenario}-${name}.png`);
	const result = (await driver
		.captureScreenshot({ target, savePath, hiDpi: true, timeoutMs: 10_000 })
		.catch((error: unknown) => ({ error: String(error) }))) as Json;
	receiptOut[`shot:${name}`] = result?.error ?? savePath;
}

/**
 * ⌘K, type to filter the Actions menu, ↵ — the keyboard path to an action.
 * Actions ignores a ⌘K that lands right after it closed (toggle debounce), so
 * wait for the menu before typing; typed text would otherwise hit the filter.
 */
async function openLauncherAction(h: ModalHelpers, action: string) {
	const actionsOpen = async () => (await h.windows()).some((window) => window.kind === "actionsDialog");
	await h.key("k", ["cmd"]);
	if (!(await until(actionsOpen, 1_500))) {
		await h.key("k", ["cmd"]);
		if (!(await until(actionsOpen, 4_000))) throw new Error("Actions menu did not open");
	}
	await h.type(action);
	await Bun.sleep(250);
	await h.key("enter");
}

async function addFixtureScript(driver: Driver, kit: string, name: string, body = 'await div("modal audit")'): Promise<string> {
	const file = join(kit, "plugins", "main", "scripts", `${name.toLowerCase().replaceAll(" ", "-")}.ts`);
	writeFileSync(file, `// Name: ${name}\n${body}\n`);
	// The launcher re-filters only when the query changes, so clear it before
	// each attempt while the script watcher picks the new file up.
	const selected = await until(async () => {
		await Bun.sleep(400);
		await driver.setFilterAndWait("", { timeoutMs: 8_000 });
		await driver.setFilterAndWait(name, { timeoutMs: 8_000 });
		const state = (await driver.getState()) as Json;
		return state.selectedValue === name;
	}, 20_000);
	if (!selected) throw new Error(`fixture script ${name} never became the launcher selection`);
	return file;
}

/** Keys that must do nothing but stay inside the modal. */
async function leakKeys(h: ModalHelpers, target: Json) {
	await h.type("zq", target);
	await h.key("backspace", [], target);
	await h.key("down", [], target);
	await h.key("up", [], target);
	await h.key("k", ["cmd"], target);
	await h.key("n", ["cmd"], target);
	await h.key("enter", ["cmd"], target);
	await h.key("enter", ["cmd", "shift"], target);
	await h.settle();
}

/**
 * Tab / ⇧Tab / arrows move focus in a two-button modal. Focus starts on the
 * primary, or on Cancel for an irreversible modal; it ends on the primary.
 */
async function focusCycles(h: ModalHelpers, target: Json, checks: Checks, prefix: string) {
	const opened = await h.modal();
	const start = opened?.risk === "irreversible" ? "cancel" : "confirm";
	const other = start === "confirm" ? "cancel" : "confirm";
	checks[`${prefix}.defaultFocus.${start}`] = opened?.focused === start;
	await h.key("tab", [], target);
	checks[`${prefix}.tabMoves`] = (await h.modal())?.focused === other;
	await h.key("tab", [], target);
	checks[`${prefix}.tabWraps`] = (await h.modal())?.focused === start;
	await h.key("tab", ["shift"], target);
	checks[`${prefix}.shiftTabMovesBack`] = (await h.modal())?.focused === other;
	await h.key("right", [], target);
	checks[`${prefix}.arrowsMoveFocus`] = (await h.modal())?.focused === start;
	if ((await h.modal())?.focused !== "confirm") await h.key("tab", [], target);
}

async function elementsOf(driver: Driver, target: Json): Promise<Json[]> {
	const snapshot = (await driver.getElements({ target, limit: 400 }, { timeoutMs: 5_000 })) as Json;
	return (snapshot.elements ?? []) as Json[];
}

/** The pending Undo offer projected on `target`, if any. */
async function undoOffer(driver: Driver, target: Json): Promise<Json | null> {
	return (await elementsOf(driver, target)).find((element) => element.semanticId === "button:undo") ?? null;
}

/** A modal's measured spacing, compared across every modal the probe opens. */
interface ModalGeometry {
	insetLeft: number;
	insetRight: number;
	insetTop: number;
	insetBottom: number;
	headerHeight: number;
	/** Header to the first part under it. */
	headerGap: number;
	/** Body (or field hint) to the button row, when there is a body. */
	bodyGap: number | null;
	buttonHeight: number;
	/** Space between neighbouring buttons, when there are two or more. */
	buttonGap: number | null;
}
const measuredModals: Array<{ modal: string; geometry: ModalGeometry }> = [];
const GEOMETRY_SLACK = 0.5;
const MIN_HIT_HEIGHT = 28;

function rectOf(value: Json): Rect {
	return { x: Number(value?.x ?? 0), y: Number(value?.y ?? 0), width: Number(value?.width ?? 0), height: Number(value?.height ?? 0) };
}
const right = (rect: Rect) => rect.x + rect.width;
const bottom = (rect: Rect) => rect.y + rect.height;
const near = (a: number, b: number) => Math.abs(a - b) <= GEOMETRY_SLACK;

/**
 * Measure the open confirm popup from its painted layout and check that its
 * spacing is symmetric and its parts line up.
 */
async function measureModal(driver: Driver, modal: string, checks: Checks, receiptOut: Json) {
	const layout = (await driver.request(
		{ type: "getLayoutInfo", target: CONFIRM },
		{ expect: "layoutInfoResult", timeoutMs: 10_000 },
	)) as Json;
	const parts = new Map<string, Rect>(
		((layout.components ?? []) as Json[]).map((component) => [String(component.name), rectOf(component.bounds)]),
	);
	const root = parts.get("confirm-popup-root");
	const stack = parts.get("confirm-modal-stack");
	const header = parts.get("confirm-modal-header");
	const row = parts.get("confirm-modal-action-row");
	const prefix = `${modal}.geometry`;
	checks[`${prefix}.measured`] = Boolean(root && stack && header && row);
	if (!root || !stack || !header || !row) {
		receiptOut[`${modal}.geometry`] = { missing: true, parts: [...parts.keys()] };
		return;
	}
	const field = parts.get("confirm-text-field");
	const body = parts.get("confirm-modal-body");
	const buttons = ["confirm-ok-button", "confirm-secondary-button", "confirm-cancel-button"]
		.map((name) => parts.get(name))
		.filter((rect): rect is Rect => Boolean(rect))
		.sort((a, b) => a.x - b.x);
	const underHeader = [field, body, row].filter((rect): rect is Rect => Boolean(rect)).sort((a, b) => a.y - b.y);
	const buttonGaps = buttons.slice(1).map((button, index) => button.x - right(buttons[index]));
	const geometry: ModalGeometry = {
		insetLeft: stack.x - root.x,
		insetRight: right(root) - right(stack),
		insetTop: stack.y - root.y,
		insetBottom: bottom(root) - bottom(row),
		headerHeight: header.height,
		headerGap: underHeader[0].y - bottom(header),
		bodyGap: body ? row.y - bottom(body) : null,
		buttonHeight: buttons[0]?.height ?? 0,
		buttonGap: buttonGaps[0] ?? null,
	};
	receiptOut[`${modal}.geometry`] = { geometry, window: { width: root.width, height: root.height }, row, buttons };
	measuredModals.push({ modal, geometry });

	checks[`${prefix}.partsInsideModal`] = [...parts.values()].every((rect) => contains(root, rect));
	checks[`${prefix}.leftEqualsRight`] = near(geometry.insetLeft, geometry.insetRight);
	checks[`${prefix}.topEqualsBottom`] = near(geometry.insetTop, geometry.insetBottom);
	checks[`${prefix}.rowEndsTheStack`] = near(bottom(row), bottom(stack));
	checks[`${prefix}.partsSpanTheStack`] = [header, row, ...(body ? [body] : []), ...(field ? [field] : [])].every(
		(rect) => near(rect.x, stack.x) && near(right(rect), right(stack)),
	);
	checks[`${prefix}.partsInOrder`] = underHeader.every(
		(rect, index) => rect.y >= bottom(index === 0 ? header : underHeader[index - 1]) - GEOMETRY_SLACK,
	);
	checks[`${prefix}.bodyGapEqualsHeaderGap`] = geometry.bodyGap === null || near(geometry.bodyGap, geometry.headerGap);
	checks[`${prefix}.buttonsInRow`] =
		buttons.length > 0 && buttons.every((button) => contains(row, button) && near(button.y, buttons[0].y));
	checks[`${prefix}.buttonsSameHeight`] = buttons.every((button) => near(button.height, geometry.buttonHeight));
	checks[`${prefix}.buttonsMeetHitSize`] = buttons.every((button) => button.height >= MIN_HIT_HEIGHT - GEOMETRY_SLACK);
	checks[`${prefix}.buttonsDoNotOverlap`] = buttonGaps.every((gap) => gap >= -GEOMETRY_SLACK);
	checks[`${prefix}.buttonsEvenlySpaced`] = buttonGaps.every((gap) => near(gap, buttonGaps[0]));
	checks[`${prefix}.buttonsEndAtRowEdge`] = buttons.length > 0 && near(right(buttons.at(-1) as Rect), right(row));
}

/** Every measured modal uses the same insets, gaps, and button height. */
function layoutConsistency(): Json {
	const checks: Checks = {};
	const metrics = Object.keys(measuredModals[0]?.geometry ?? {}) as Array<keyof ModalGeometry>;
	const values: Json = {};
	for (const metric of metrics) {
		const seen = measuredModals
			.map(({ modal, geometry }) => ({ modal, value: geometry[metric] }))
			.filter((entry) => entry.value !== null);
		values[metric] = Object.fromEntries(seen.map(({ modal, value }) => [modal, value]));
		checks[`same.${metric}`] = seen.every(({ value }) => near(Number(value), Number(seen[0].value)));
	}
	checks["measuredSeveralModals"] = measuredModals.length >= 2;
	return { checks, modals: measuredModals.length, values };
}

/**
 * The attached toast popup over `parentBounds`: bottom-centered, inset 12pt,
 * inside the parent. It is not an automation window, so it is found and
 * photographed through the window server.
 */
async function checkToastPopup(
	h: ModalHelpers,
	parentId: string,
	checks: Checks,
	prefix: string,
	scenario: string,
	receiptOut: Json,
) {
	const parent = (await h.windows()).find((window) => window.id === parentId);
	const pid = Number(parent?.pid ?? (await appPid(h)));
	const frames = await windowServerFrames(pid);
	const parentFrame = frameOfSize(frames, parent?.bounds ?? null);
	const toast =
		frames.find(
			(frame) =>
				Math.abs(frame.height - 44) <= 1 &&
				Boolean(parentFrame) &&
				contains(parentFrame as Rect, frame),
		) ?? null;
	receiptOut[`${prefix}.frames`] = { parent: parentFrame, toast };
	checks[`${prefix}.visible`] = Boolean(toast);
	if (!toast || !parentFrame) return;
	const inset = 12;
	checks[`${prefix}.centered`] = Math.abs(toast.x + toast.width / 2 - (parentFrame.x + parentFrame.width / 2)) <= 1;
	checks[`${prefix}.bottomInset`] = Math.abs(bottom(parentFrame) - bottom(toast) - inset) <= 1;
	checks[`${prefix}.sideInsets`] = toast.x - parentFrame.x >= inset - 1 && right(parentFrame) - right(toast) >= inset - 1;
	const savePath = join(shotDir, `${scenario}-${prefix}-composite.png`);
	const region = [parentFrame.x, parentFrame.y, parentFrame.width, parentFrame.height].map(Math.round).join(",");
	const capture = Bun.spawnSync(["screencapture", "-x", "-R", region, savePath]);
	receiptOut[`shot:${prefix}-composite`] = capture.exitCode === 0 ? savePath : capture.stderr.toString();
}

const runners: Record<Scenario, () => Promise<Json>> = {
	async "launcher-confirm"() {
		const { driver } = await launch("launcher-confirm");
		const h = makeHelpers(driver);
		const checks: Checks = {};
		const out: Json = { checks };
		try {
			// Irreversible and harmless in a sandbox: the modal must open on Cancel.
			const openClear = async () => {
				await driver.setFilterAndWait("Clear Suggested Items", { timeoutMs: 8_000 });
				await h.key("enter");
				return until(() => h.modal());
			};
			const pid = await appPid(h);
			const launcherView = (await h.main()).view;
			const [opened, entry] = await Promise.all([openClear(), sampleFramesDuringEntry(pid, 2_500)]);
			const before = await h.main();
			out.entryFrames = entry;
			checks["entry.allFramesOnScreen"] = entry.samples > 0 && entry.offScreen.length === 0;
			checks["opened.byKeyboard"] = Boolean(opened?.title);
			checks["opened.parentIsMain"] = opened?.parent === "main";
			checks["opened.windowNotReplaced"] = before.view === launcherView && before.view !== "confirmPrompt";
			checks["risk.irreversible"] = opened?.risk === "irreversible";
			checks["risk.primaryIsDanger"] = opened?.primaryDanger === true;
			checks["risk.opensOnCancel"] = opened?.focused === "cancel";
			await checkPlacement(h, checks, "layout", out);
			await shot(driver, "launcher-confirm", "modal", CONFIRM, out);
			await measureModal(driver, "launcher-confirm", checks, out);
			await shot(driver, "launcher-confirm", "parent", MAIN, out);

			// focusCycles leaves focus on the irreversible primary, so a leaked
			// ⌘↵ in leakKeys would run it and close the modal.
			await focusCycles(h, MAIN, checks, "focus");
			await shot(driver, "launcher-confirm", "primary-focused", CONFIRM, out);
			await leakKeys(h, MAIN);
			const during = await h.main();
			checks["noLeak.modalStillOpen"] = Boolean(await h.modal());
			checks["noLeak.focusKept"] = (await h.modal())?.focused === "confirm";
			checks["noLeak.filterUnchanged"] = during.input === before.input;
			checks["noLeak.selectionUnchanged"] = during.selected === before.selected;
			checks["noLeak.noActionsPopup"] = !(await h.windows()).some((window) => window.kind === "actionsDialog");
			checks["noLeak.stillLauncher"] = during.view === before.view && during.visible;

			// Enter on Cancel dismisses.
			await h.key("tab");
			checks["enterOnCancel.focusedCancel"] = (await h.modal())?.focused === "cancel";
			await h.key(checks["enterOnCancel.focusedCancel"] ? "enter" : "escape");
			checks["enterOnCancel.closes"] = Boolean(await until(async () => !(await h.modal())));

			// Keyboard is back on the filter.
			const afterClose = await h.main();
			await h.type("x");
			checks["afterDismiss.typingReachesFilter"] = (await h.main()).input === `${afterClose.input}x`;

			// Esc dismisses; ⌘W dismisses only the modal.
			await openClear();
			await h.key("escape");
			checks["escape.closes"] = Boolean(await until(async () => !(await h.modal())));
			checks["escape.keptWindow"] = (await h.main()).visible;
			await openClear();
			await h.key("w", ["cmd"]);
			checks["cmdW.closesModal"] = Boolean(await until(async () => !(await h.modal())));
			checks["cmdW.keptWindow"] = (await h.main()).visible;
			return out;
		} finally {
			await driver.close();
		}
	},

	async "launcher-undo"() {
		const { driver, kit } = await launch("launcher-undo");
		const h = makeHelpers(driver);
		const checks: Checks = {};
		const out: Json = { checks };
		const name = "Modal Audit Delete Target";
		let file = "";
		try {
			file = await addFixtureScript(driver, kit, name);
			await openLauncherAction(h, "delete");
			checks["delete.trashedAtOnce"] = Boolean(await until(async () => !existsSync(file), 8_000));
			checks["delete.noModal"] = !(await h.modal());
			checks["delete.launcherStaysOpen"] = (await h.main()).visible;
			const offer = await until(() => undoOffer(driver, MAIN));
			out.offer = offer;
			checks["undo.offered"] = Boolean(offer);
			await shot(driver, "launcher-undo", "offer", MAIN, out);

			await h.key("z", ["cmd"]);
			checks["undo.restored"] = Boolean(await until(async () => existsSync(file), 8_000));
			checks["undo.offerGone"] = Boolean(await until(async () => !(await undoOffer(driver, MAIN))));
			checks["undo.scriptBackInLauncher"] = Boolean(
				await until(async () => {
					await driver.setFilterAndWait("", { timeoutMs: 8_000 });
					await driver.setFilterAndWait(name, { timeoutMs: 8_000 });
					return (await h.main()).selected === name;
				}, 15_000),
			);
			return out;
		} finally {
			out.fixtureLeftInTrash = Boolean(file) && !existsSync(file);
			await driver.close();
		}
	},

	async "agent-chat-consent"() {
		const { driver } = await launch("agent-chat-consent");
		const h = makeHelpers(driver);
		const checks: Checks = {};
		const out: Json = { checks };
		const chat = async () =>
			(await driver.request(
				{ type: "getAgentChatState", target: MAIN },
				{ expect: "agent_chatStateResult", timeoutMs: 8_000 },
			)) as Json;
		try {
			await driver.setFilterAndWait("What is 2+2?", { timeoutMs: 10_000 });
			await h.key("enter", ["cmd"]);
			const consent = await until(() => h.modal(), 10_000);
			checks["opened.consentModal"] = consent?.title === "Sign in to ChatGPT";
			checks["opened.chatStaysUnderneath"] = (await h.main()).view === "agentChatChat";
			await checkPlacement(h, checks, "layout", out);
			await shot(driver, "agent-chat-consent", "modal", CONFIRM, out);
			await measureModal(driver, "agent-chat-consent", checks, out);
			await h.type("hi");
			await h.key("backspace");
			await h.key("k", ["cmd"]);
			await h.key("up");
			await h.settle();
			const during = await chat();
			checks["noLeak.composerEmpty"] = during.inputText === "";
			checks["noLeak.stillAwaitingConsent"] = during.signIn === "awaitingConsent";
			checks["noLeak.noActionsPopup"] = !(await h.windows()).some((window) => window.kind === "actionsDialog");
			await focusCycles(h, MAIN, checks, "focus");
			// Focus is on Sign In again: Tab to Not Now, Enter declines.
			await h.key("tab");
			await h.key("enter");
			checks["enterOnNotNow.closes"] = Boolean(await until(async () => !(await h.modal())));
			const declined = await until(async () => {
				const state = await chat();
				return state.signIn === "notSignedIn" ? state : null;
			});
			checks["enterOnNotNow.declined"] = Boolean(declined);
			checks["enterOnNotNow.chatStays"] = (await h.main()).view === "agentChatChat";
			await h.type("ok");
			checks["afterDismiss.typingReachesComposer"] = (await chat()).inputText === "ok";
			return out;
		} finally {
			await driver.close();
		}
	},

	async "notes-delete"() {
		const { driver } = await launch("notes-delete");
		const h = makeHelpers(driver);
		const checks: Checks = {};
		const out: Json = { checks };
		const note = async () => {
			const state = (await driver.request({ type: "getState", target: NOTES }, { timeoutMs: 8_000 })) as Json;
			return state.inputValue as string;
		};
		const notesOpen = async () => (await h.windows()).some((window) => window.id === "notes" && window.visible);
		try {
			driver.send({ type: "openNotes", requestId: "modal-audit-open-notes" });
			checks["notes.opened"] = Boolean(await until(notesOpen, 10_000));
			await Bun.sleep(800);
			await h.key("n", ["cmd"], NOTES);
			await Bun.sleep(500);
			await h.type("Audit note", NOTES);
			const text = await note();
			checks["notes.typed"] = text === "Audit note";

			// Outside the Trash, delete has no modal: it trashes and offers Undo.
			const trashNote = async () => {
				await h.key("backspace", ["cmd", "shift"], NOTES);
				return until(async () => ((await note()) !== text ? true : null));
			};
			checks["trash.noteLeft"] = Boolean(await trashNote());
			checks["trash.noModal"] = !(await h.modal());
			const offer = await until(() => undoOffer(driver, NOTES));
			out.offer = offer;
			checks["trash.undoOffered"] = Boolean(offer);
			// Notes toasts are an attached glass window, never an in-window layer.
			const notesElements = await elementsOf(driver, NOTES);
			checks["trash.toastIsPopup"] = notesElements.some((element) => element.semanticId === "toast:undo-offer");
			checks["trash.noInWindowToast"] = !notesElements.some((element) =>
				String(element.semanticId).startsWith("root-notification:"),
			);
			await shot(driver, "notes-delete", "undo-offer", NOTES, out);
			await checkToastPopup(h, "notes", checks, "toast", "notes-delete", out);
			await h.key("z", ["cmd"], NOTES);
			checks["undo.noteBack"] = Boolean(await until(async () => (await note()) === text));
			checks["undo.offerGone"] = Boolean(await until(async () => !(await undoOffer(driver, NOTES))));

			// Typing withdraws the offer, so ⌘Z goes back to meaning text undo.
			await trashNote();
			await until(() => undoOffer(driver, NOTES));
			await h.type("q", NOTES);
			checks["withdraw.typingWithdrawsOffer"] = Boolean(
				await until(async () => !(await undoOffer(driver, NOTES))),
			);
			await h.key("z", ["cmd"], NOTES);
			await h.settle();
			checks["withdraw.cmdZDoesNotRestore"] = (await note()) !== text;

			// In the Trash, delete is permanent: an irreversible modal on Cancel.
			await h.key("t", ["cmd", "shift"], NOTES);
			checks["trashView.showsNote"] = Boolean(await until(async () => (await note()) === text));
			const openDelete = async () => {
				await h.key("backspace", ["cmd", "shift"], NOTES);
				return until(() => h.modal());
			};
			const pid = await appPid(h);
			const [opened, entry] = await Promise.all([openDelete(), sampleFramesDuringEntry(pid, 1_500)]);
			out.entryFrames = entry;
			checks["entry.allFramesOnScreen"] = entry.samples > 0 && entry.offScreen.length === 0;
			checks["opened.parentIsNotes"] = opened?.parent === "notes";
			checks["risk.irreversible"] = opened?.risk === "irreversible";
			checks["risk.primaryIsDanger"] = opened?.primaryDanger === true;
			checks["risk.opensOnCancel"] = opened?.focused === "cancel";
			await checkPlacement(h, checks, "layout", out);
			await shot(driver, "notes-delete", "modal", CONFIRM, out);
			await measureModal(driver, "notes-permanent-delete", checks, out);
			await shot(driver, "notes-delete", "parent", NOTES, out);
			await focusCycles(h, NOTES, checks, "focus");
			// Focus is on Delete: a leaked ⌘↵ or Enter here would delete.
			await h.type("xy", NOTES);
			await h.key("enter", ["cmd"], NOTES);
			await h.key("enter", ["shift"], NOTES);
			await h.key("backspace", [], NOTES);
			await h.settle();
			checks["noLeak.noteUnchanged"] = (await note()) === text;
			checks["noLeak.modalStillOpen"] = Boolean(await h.modal());
			checks["noLeak.focusKept"] = (await h.modal())?.focused === "confirm";

			await h.key("escape", [], NOTES);
			checks["escape.closesModal"] = Boolean(await until(async () => !(await h.modal())));
			checks["escape.keptNotesOpen"] = await notesOpen();

			await openDelete();
			await h.key("w", ["cmd"], NOTES);
			checks["cmdW.closesModalOnly"] = Boolean(await until(async () => !(await h.modal()))) && (await notesOpen());

			// Enter right away lands on Cancel and keeps the note.
			await openDelete();
			await h.key("enter", [], NOTES);
			checks["enterAtOnce.closesModal"] = Boolean(await until(async () => !(await h.modal())));
			await h.settle();
			checks["enterAtOnce.keptNote"] = (await note()) === text;

			// Tab to Delete, then Enter deletes for good.
			await openDelete();
			await h.key("tab", [], NOTES);
			checks["enterOnDelete.focusedDelete"] = (await h.modal())?.focused === "confirm";
			await h.key("enter", [], NOTES);
			checks["enterOnDelete.closesModal"] = Boolean(await until(async () => !(await h.modal())));
			checks["enterOnDelete.deletedNote"] = Boolean(await until(async () => (await note()) !== text));

			// A modal closes with its parent window.
			await h.key("escape", [], NOTES);
			await h.key("n", ["cmd"], NOTES);
			await Bun.sleep(400);
			await h.type("Second", NOTES);
			await trashNote();
			await h.key("t", ["cmd", "shift"], NOTES);
			await Bun.sleep(400);
			await openDelete();
			checks["parentClosed.modalOpened"] = Boolean(await h.modal());
			driver.send({ type: "openNotes", requestId: "modal-audit-toggle-notes" });
			await until(async () => !(await notesOpen()), 5_000);
			checks["parentClosed.modalClosed"] = Boolean(await until(async () => !(await h.modal()), 3_000));
			return out;
		} finally {
			await driver.close();
		}
	},

	async "detached-chat-confirm"() {
		const { driver } = await launch("detached-chat-confirm");
		const h = makeHelpers(driver);
		const checks: Checks = {};
		const out: Json = { checks };
		try {
			driver.send({ type: "openAgentChatDetachedFixture", requestId: "modal-audit-detached" });
			const detached = await until(
				async () => (await h.windows()).find((window) => window.kind === "agentChatDetached"),
				10_000,
			);
			if (!detached) throw new Error("detached Agent Chat never opened");
			const target = { type: "id", id: detached.id };
			const composer = async () =>
				(
					(await driver.request(
						{ type: "getAgentChatState", target },
						{ expect: "agent_chatStateResult", timeoutMs: 8_000 },
					)) as Json
				).inputText as string;
			await h.type("draft", target);
			await h.key("k", ["cmd"], target);
			await Bun.sleep(400);
			await h.type("delete all", target);
			await h.key("enter", [], target);
			const opened = await until(() => h.modal());
			checks["opened.parentIsDetachedChat"] = opened?.parent === detached.id;
			checks["risk.irreversible"] = opened?.risk === "irreversible";
			checks["risk.opensOnCancel"] = opened?.focused === "cancel";
			await checkPlacement(h, checks, "layout", out);
			await shot(driver, "detached-chat-confirm", "modal", CONFIRM, out);
			await measureModal(driver, "detached-chat-confirm", checks, out);
			await focusCycles(h, target, checks, "focus");
			await h.type("zz", target);
			await h.key("enter", ["cmd"], target);
			await h.settle();
			checks["noLeak.composerUnchanged"] = (await composer()) === "draft";
			checks["noLeak.modalStillOpen"] = Boolean(await h.modal());
			await h.key("escape", [], target);
			checks["escape.closesModal"] = Boolean(await until(async () => !(await h.modal())));
			checks["escape.keptChatOpen"] = (await h.windows()).some((window) => window.id === detached.id);
			await h.type("!", target);
			checks["afterDismiss.typingReachesComposer"] = (await composer()) === "draft!";
			return out;
		} finally {
			await driver.close();
		}
	},

	async "fixture-confirm"() {
		const { driver } = await launch("fixture-confirm");
		const h = makeHelpers(driver);
		const checks: Checks = {};
		const out: Json = { checks };
		const open = async () => {
			driver.send({
				type: "openConfirmPrompt",
				title: "Replace the file?",
				body: "Modal keyboard audit",
				confirmText: "Replace",
				cancelText: "Cancel",
				requestId: `modal-audit-fixture-${Date.now()}`,
			});
			return until(() => h.modal());
		};
		try {
			const before = await h.main();
			const opened = await open();
			checks["opened"] = opened?.title === "Replace the file?";
			checks["opened.windowNotReplaced"] = (await h.main()).view === before.view;
			checks["risk.destructive"] = opened?.risk === "destructive";
			checks["risk.primaryIsDanger"] = opened?.primaryDanger === true;
			await checkPlacement(h, checks, "layout", out);
			await shot(driver, "fixture-confirm", "modal", CONFIRM, out);
			await measureModal(driver, "fixture-confirm", checks, out);
			await shot(driver, "fixture-confirm", "parent", MAIN, out);
			await focusCycles(h, MAIN, checks, "focus");
			await leakKeys(h, MAIN);
			checks["noLeak.modalStillOpen"] = Boolean(await h.modal());
			checks["noLeak.filterUnchanged"] = (await h.main()).input === before.input;
			checks["noLeak.noActionsPopup"] = !(await h.windows()).some((window) => window.kind === "actionsDialog");
			await h.key("escape");
			checks["escape.closes"] = Boolean(await until(async () => !(await h.modal())));
			await open();
			await h.key("w", ["cmd"]);
			checks["cmdW.closesModalOnly"] = Boolean(await until(async () => !(await h.modal()))) && (await h.main()).visible;
			await open();
			await h.key("enter");
			checks["enter.resolves"] = Boolean(await until(async () => !(await h.modal())));
			await h.type("x");
			checks["afterDismiss.typingReachesFilter"] = (await h.main()).input === `${before.input}x`;
			await h.key("backspace");

			// A second modal waits for the first instead of replacing it.
			await open();
			driver.send({
				type: "openConfirmPrompt",
				title: "Overwrite the backup?",
				body: "Queued behind the first modal",
				confirmText: "Overwrite",
				cancelText: "Cancel",
				requestId: `modal-audit-fixture-queued-${Date.now()}`,
			});
			const waiting = await until(async () => {
				const modal = await h.modal();
				return modal && modal.waiting === 1 ? modal : null;
			});
			checks["queue.firstStaysOpen"] = waiting?.title === "Replace the file?";
			checks["queue.secondWaits"] = waiting?.waiting === 1;
			await h.key("escape");
			const next = await until(async () => {
				const modal = await h.modal();
				return modal?.title === "Overwrite the backup?" ? modal : null;
			});
			checks["queue.secondOpensAfterFirst"] = Boolean(next);
			checks["queue.nothingLeftWaiting"] = next?.waiting === 0;
			await shot(driver, "fixture-confirm", "queued", CONFIRM, out);
			await measureModal(driver, "fixture-queued", checks, out);
			await h.key("escape");
			checks["queue.allClosed"] = Boolean(await until(async () => !(await h.modal())));
			return out;
		} finally {
			await driver.close();
		}
	},

	async "modal-variants"() {
		const { driver } = await launch("modal-variants");
		const h = makeHelpers(driver);
		const checks: Checks = {};
		const out: Json = { checks };
		const long =
			"This permanently deletes every conversation, attachment, and saved draft from this Mac. Shared links stop working. This cannot be undone, even from a backup you made earlier today.";
		const variants: Record<string, Json> = {
			"no-body": { title: "Replace the file?", body: "" },
			"one-line": { title: "Replace the file?", body: "Modal keyboard audit" },
			"several-lines": { title: "Delete all history?", body: long },
			"scrolling-body": { title: "Delete all history?", body: `${long} `.repeat(3) },
			"long-labels": { title: "Move everything to the Trash?", body: "Modal keyboard audit", confirmText: "Move to Trash", cancelText: "Keep Everything" },
		};
		try {
			const heights: Record<string, number> = {};
			for (const [name, variant] of Object.entries(variants)) {
				driver.send({
					type: "openConfirmPrompt",
					confirmText: "Replace",
					cancelText: "Cancel",
					...variant,
					requestId: `modal-audit-variant-${name}-${Date.now()}`,
				});
				const opened = await until(async () => {
					const modal = await h.modal();
					return modal?.title === variant.title ? modal : null;
				});
				checks[`${name}.opened`] = Boolean(opened);
				await Bun.sleep(400);
				await shot(driver, "modal-variants", name, CONFIRM, out);
				await measureModal(driver, `variant-${name}`, checks, out);
				heights[name] = opened?.bounds?.height ?? 0;
				await h.key("escape");
				checks[`${name}.closed`] = Boolean(await until(async () => !(await h.modal())));
				await Bun.sleep(300);
			}
			out.heights = heights;
			checks["height.growsWithBody"] =
				heights["no-body"] < heights["one-line"] && heights["one-line"] < heights["several-lines"];
			checks["height.capsLongBody"] = heights["scrolling-body"] <= 240 && heights["scrolling-body"] >= heights["several-lines"];
			return out;
		} finally {
			await driver.close();
		}
	},

	async "sdk-confirm"() {
		const { driver, kit } = await launch("sdk-confirm");
		const h = makeHelpers(driver);
		const checks: Checks = {};
		const out: Json = { checks };
		const resultPath = join(kit, "tmp", "modal-audit-sdk-confirm.json");
		try {
			const name = "Modal Audit SDK Confirm";
			await addFixtureScript(
				driver,
				kit,
				name,
				`const confirmed = await confirm({ title: "Delete the audit log?", message: "This cannot be undone.", confirmText: "Delete", cancelText: "Keep", irreversible: true });\nawait Bun.write(${JSON.stringify(resultPath)}, JSON.stringify({ confirmed }));`,
			);
			await h.key("enter");
			const opened = await until(() => h.modal(), 15_000);
			checks["opened.byScript"] = opened?.buttons.includes("Delete") === true && opened.buttons.includes("Keep");
			checks["opened.titleFromScript"] = opened?.title === "Delete the audit log?";
			checks["risk.irreversible"] = opened?.risk === "irreversible";
			checks["risk.primaryIsDanger"] = opened?.primaryDanger === true;
			checks["risk.opensOnKeep"] = opened?.focused === "cancel";
			checks["opened.parentIsMain"] = opened?.parent === "main";
			checks["opened.windowNotReplaced"] = (await h.main()).view !== "confirmPrompt";
			out.viewUnderModal = (await h.main()).view;
			await checkPlacement(h, checks, "layout", out);
			await shot(driver, "sdk-confirm", "modal", CONFIRM, out);
			await measureModal(driver, "sdk-confirm", checks, out);
			// Enter right away answers Keep.
			await h.key("enter");
			checks["enterAtOnce.closes"] = Boolean(await until(async () => !(await h.modal())));
			const result = await until(async () => {
				if (!existsSync(resultPath)) return null;
				return JSON.parse(readFileSync(resultPath, "utf8")) as Json;
			}, 8_000);
			checks["enterAtOnce.scriptGotFalse"] = result?.confirmed === false;
			return out;
		} finally {
			await driver.close();
		}
	},

	async "alias-editor"() {
		const { driver, kit } = await launch("alias-editor");
		const h = makeHelpers(driver);
		const checks: Checks = {};
		const out: Json = { checks };
		const aliases = () => {
			try {
				return readFileSync(join(kit, "aliases.json"), "utf8");
			} catch {
				return "";
			}
		};
		try {
			const name = "Modal Audit Alias Target";
			await addFixtureScript(driver, kit, name);
			const before = await h.main();
			const openEditor = async () => {
				await openLauncherAction(h, "alias");
				return until(async () => {
					const modal = await h.modal();
					return modal?.field !== null && modal?.field !== undefined ? modal : null;
				});
			};
			const opened = await openEditor();
			checks["opened.isModalWithField"] = opened?.field === "";
			checks["opened.parentIsMain"] = opened?.parent === "main";
			checks["opened.windowNotReplaced"] = (await h.main()).view === before.view;
			checks["opened.saveDisabledWhenEmpty"] = opened?.primaryDisabled === true;
			await checkPlacement(h, checks, "layout", out);

			// Typing edits the field and nothing underneath.
			await h.type("a b");
			const invalid = await h.modal();
			checks["field.typingEdits"] = invalid?.field === "a b";
			checks["field.invalidExplained"] = Boolean(invalid?.fieldProblem);
			checks["field.saveDisabledWhenInvalid"] = invalid?.primaryDisabled === true;
			await shot(driver, "alias-editor", "invalid", CONFIRM, out);
			await measureModal(driver, "alias-editor", checks, out);
			await h.key("enter");
			checks["field.enterDoesNotSaveInvalid"] = Boolean(await h.modal()) && !aliases().includes('"a b"');
			// Caret after "a b": one ← puts it after the space, ⌫ removes it.
			await h.key("left");
			await h.key("backspace");
			checks["field.arrowsMoveCaret"] = (await h.modal())?.field === "ab";
			await h.key("a", ["cmd"]);
			await h.type("mx");
			checks["field.selectAllReplaces"] = (await h.modal())?.field === "mx";
			await shot(driver, "alias-editor", "valid", CONFIRM, out);
			await h.key("down");
			await h.key("k", ["cmd"]);
			const during = await h.main();
			checks["noLeak.filterUnchanged"] = during.input === before.input;
			checks["noLeak.selectionUnchanged"] = during.selected === before.selected;
			checks["noLeak.noActionsPopup"] = !(await h.windows()).some((window) => window.kind === "actionsDialog");

			await h.key("enter");
			checks["enter.savesAndCloses"] = Boolean(await until(async () => !(await h.modal())));
			checks["enter.aliasSaved"] = Boolean(await until(async () => aliases().includes('"mx"')));
			checks["enter.didNotRunLauncherRow"] = (await h.main()).view === before.view;

			// Reopen: the field holds the alias and Remove is offered.
			const reopened = await openEditor();
			checks["reopen.showsAlias"] = reopened?.field === "mx";
			checks["reopen.offersRemove"] = reopened?.buttons.includes("Remove") === true;
			await shot(driver, "alias-editor", "existing", CONFIRM, out);
			await measureModal(driver, "alias-editor-existing", checks, out);
			await h.key("tab");
			checks["remove.tabReachesRemove"] = (await h.modal())?.focused === "secondary";
			await h.key("tab", ["shift"]);
			// ⌘I runs the secondary button from anywhere in the modal.
			await h.key("i", ["cmd"]);
			checks["remove.closes"] = Boolean(await until(async () => !(await h.modal())));
			checks["remove.aliasRemoved"] = Boolean(await until(async () => !aliases().includes('"mx"')));

			await openEditor();
			await h.key("escape");
			checks["escape.closes"] = Boolean(await until(async () => !(await h.modal())));
			await h.type("q");
			checks["afterDismiss.typingReachesFilter"] = (await h.main()).input === `${before.input}q`;
			return out;
		} finally {
			await driver.close();
		}
	},

	async "hotkey-hides-modal"() {
		const { driver } = await launch("hotkey-hides-modal");
		const h = makeHelpers(driver);
		const checks: Checks = {};
		const out: Json = { checks };
		try {
			await driver.setFilterAndWait("Clear Suggested Items", { timeoutMs: 8_000 });
			await h.key("enter");
			checks["opened"] = Boolean(await until(() => h.modal()));
			for (const phase of ["down", "up"]) {
				await driver.request(
					{ type: "simulateMainHotkeyGesture", phase, requestId: `modal-audit-hotkey-${phase}` },
					{ expect: "externalCommandResult", timeoutMs: 5_000 },
				);
				await Bun.sleep(120);
			}
			checks["hotkey.hidesWindow"] = Boolean(await until(async () => !(await h.main()).visible));
			checks["hotkey.closesModal"] = Boolean(await until(async () => !(await h.modal())));
			return out;
		} finally {
			await driver.close();
		}
	},
};

let pass = true;
for (const scenario of scenarios) {
	let result: Json;
	try {
		result = await runners[scenario]();
	} catch (error) {
		result = { error: error instanceof Error ? (error.stack ?? error.message) : String(error), checks: {} };
	}
	const failed = Object.entries((result.checks ?? {}) as Checks)
		.filter(([, ok]) => !ok)
		.map(([name]) => name);
	result.failedChecks = failed;
	result.pass = !result.error && failed.length === 0;
	pass &&= result.pass;
	receipt.scenarios[scenario] = result;
	console.error(
		`${result.pass ? "PASS" : "FAIL"} ${scenario}${failed.length ? ` — ${failed.join(", ")}` : ""}${result.error ? ` — ${String(result.error).split("\n")[0]}` : ""}`,
	);
}

if (measuredModals.length) {
	const consistency = layoutConsistency();
	const failed = Object.entries(consistency.checks as Checks)
		.filter(([, ok]) => !ok)
		.map(([name]) => name);
	consistency.failedChecks = failed;
	consistency.pass = failed.length === 0;
	pass &&= consistency.pass;
	receipt.scenarios["layout-consistency"] = consistency;
	console.error(`${consistency.pass ? "PASS" : "FAIL"} layout-consistency${failed.length ? ` — ${failed.join(", ")}` : ""}`);
}

if (keepSessions) {
	receipt.sessionDirs = sessionDirs;
} else {
	for (const dir of sessionDirs) rmSync(dir, { recursive: true, force: true });
}
receipt.pass = pass;
const serialized = JSON.stringify(receipt, null, 2);
if (receiptPath) writeFileSync(receiptPath, serialized);
console.log(serialized);
process.exit(pass ? 0 : 1);
