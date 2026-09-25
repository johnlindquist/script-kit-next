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
 *   - a modal closes with its parent window.
 *
 * Surfaces: launcher confirm (Delete Script), Agent Chat sign-in consent,
 * Notes delete, detached Agent Chat "Delete All History?", the confirm
 * fixture over the main window, SDK `confirm()` from a running script, the
 * Add Alias editor, and the main hotkey hiding a window with a modal open.
 * Screenshots of each modal land in .test-screenshots/modal-audit/.
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
	"agent-chat-consent",
	"notes-delete",
	"detached-chat-confirm",
	"fixture-confirm",
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
		const focusedId: string = snapshot.focusedSemanticId ?? "";
		return {
			title: popup.title,
			parent: popup.parentWindowId,
			bounds: popup.bounds ?? null,
			focused: focusedId.split(":").at(-1) ?? null,
			field: field ? String(field.value ?? "") : null,
			fieldProblem: field?.actionDisabled ?? null,
			primaryDisabled: Boolean(primary?.actionDisabled),
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
		.captureScreenshot({ target, savePath, timeoutMs: 10_000 })
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

/** Tab / ⇧Tab / arrows move focus in a two-button modal. */
async function focusCycles(h: ModalHelpers, target: Json, checks: Checks, prefix: string) {
	checks[`${prefix}.defaultFocusIsPrimary`] = (await h.modal())?.focused === "confirm";
	await h.key("tab", [], target);
	checks[`${prefix}.tabMovesToCancel`] = (await h.modal())?.focused === "cancel";
	await h.key("tab", [], target);
	checks[`${prefix}.tabWrapsToPrimary`] = (await h.modal())?.focused === "confirm";
	await h.key("tab", ["shift"], target);
	checks[`${prefix}.shiftTabMovesBack`] = (await h.modal())?.focused === "cancel";
	await h.key("right", [], target);
	checks[`${prefix}.arrowsMoveFocus`] = (await h.modal())?.focused === "confirm";
}

const runners: Record<Scenario, () => Promise<Json>> = {
	async "launcher-confirm"() {
		const { driver, kit } = await launch("launcher-confirm");
		const h = makeHelpers(driver);
		const checks: Checks = {};
		const out: Json = { checks };
		try {
			const name = "Modal Audit Delete Target";
			const file = await addFixtureScript(driver, kit, name);
			const openDelete = async () => {
				await openLauncherAction(h, "delete");
				return until(() => h.modal());
			};
			const before = await h.main();
			const pid = await appPid(h);
			// Covers the Actions menu, typing, and the modal's entry animation.
			const [opened, entry] = await Promise.all([openDelete(), sampleFramesDuringEntry(pid, 4_000)]);
			out.entryFrames = entry;
			checks["entry.allFramesOnScreen"] = entry.samples > 0 && entry.offScreen.length === 0;
			checks["opened.byKeyboard"] = Boolean(opened?.title);
			checks["opened.parentIsMain"] = opened?.parent === "main";
			checks["opened.windowNotReplaced"] = (await h.main()).view === before.view;
			await checkPlacement(h, checks, "layout", out);
			await shot(driver, "launcher-confirm", "modal", CONFIRM, out);
			await shot(driver, "launcher-confirm", "parent", MAIN, out);

			// focusCycles leaves focus on the destructive primary, so a leaked
			// ⌘↵ in leakKeys would delete the script.
			await focusCycles(h, MAIN, checks, "focus");
			await leakKeys(h, MAIN);
			const during = await h.main();
			checks["noLeak.modalStillOpen"] = Boolean(await h.modal());
			checks["noLeak.focusKept"] = (await h.modal())?.focused === "confirm";
			checks["noLeak.filterUnchanged"] = during.input === before.input;
			checks["noLeak.selectionUnchanged"] = during.selected === before.selected;
			checks["noLeak.noActionsPopup"] = !(await h.windows()).some((window) => window.kind === "actionsDialog");
			checks["noLeak.stillLauncher"] = during.view === before.view && during.visible;
			checks["cmdEnter.didNotDelete"] = existsSync(file);

			// Enter on Cancel dismisses without deleting.
			await h.key("tab");
			checks["enterOnCancel.focusedCancel"] = (await h.modal())?.focused === "cancel";
			// Never press Enter on the destructive primary: it trashes the fixture.
			await h.key(checks["enterOnCancel.focusedCancel"] ? "enter" : "escape");
			checks["enterOnCancel.closes"] = Boolean(await until(async () => !(await h.modal())));
			checks["enterOnCancel.keptScript"] = existsSync(file);

			// Keyboard is back on the filter.
			await h.type("x");
			checks["afterDismiss.typingReachesFilter"] = (await h.main()).input === `${before.input}x`;
			await h.key("backspace");

			// Esc dismisses; ⌘W dismisses only the modal.
			await openDelete();
			await h.key("escape");
			checks["escape.closes"] = Boolean(await until(async () => !(await h.modal())));
			checks["escape.keptWindow"] = (await h.main()).visible;
			await openDelete();
			await h.key("w", ["cmd"]);
			checks["cmdW.closesModal"] = Boolean(await until(async () => !(await h.modal())));
			checks["cmdW.keptWindow"] = (await h.main()).visible;
			checks["end.scriptKept"] = existsSync(file);
			return out;
		} finally {
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

			const openDelete = async () => {
				await h.key("backspace", ["cmd", "shift"], NOTES);
				return until(() => h.modal());
			};
			const pid = await appPid(h);
			const [opened, entry] = await Promise.all([openDelete(), sampleFramesDuringEntry(pid, 1_500)]);
			out.entryFrames = entry;
			checks["entry.allFramesOnScreen"] = entry.samples > 0 && entry.offScreen.length === 0;
			checks["opened.parentIsNotes"] = opened?.parent === "notes";
			await checkPlacement(h, checks, "layout", out);
			await shot(driver, "notes-delete", "modal", CONFIRM, out);
			await shot(driver, "notes-delete", "parent", NOTES, out);
			await focusCycles(h, NOTES, checks, "focus");
			// Focus is back on Delete: a leaked ⌘↵ or Enter here would delete.
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
			await h.type("!", NOTES);
			checks["afterDismiss.typingReachesEditor"] = (await note()) === `${text}!`;
			await h.key("backspace", [], NOTES);

			await openDelete();
			await h.key("w", ["cmd"], NOTES);
			checks["cmdW.closesModalOnly"] = Boolean(await until(async () => !(await h.modal()))) && (await notesOpen());

			await openDelete();
			await h.key("enter", [], NOTES);
			checks["enter.closesModal"] = Boolean(await until(async () => !(await h.modal())));
			checks["enter.deletedNote"] = Boolean(await until(async () => (await note()) !== text));
			checks["enter.noNewlineInserted"] = !(await note()).startsWith(`${text}\n`);

			// A modal closes with its parent window.
			await h.key("n", ["cmd"], NOTES);
			await Bun.sleep(400);
			await h.type("Second", NOTES);
			await openDelete();
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
			await checkPlacement(h, checks, "layout", out);
			await shot(driver, "detached-chat-confirm", "modal", CONFIRM, out);
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
			await checkPlacement(h, checks, "layout", out);
			await shot(driver, "fixture-confirm", "modal", CONFIRM, out);
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
				`const confirmed = await confirm({ message: "Proceed with the audit?", confirmText: "Proceed", cancelText: "Stop" });\nawait Bun.write(${JSON.stringify(resultPath)}, JSON.stringify({ confirmed }));`,
			);
			await h.key("enter");
			const opened = await until(() => h.modal(), 15_000);
			checks["opened.byScript"] = opened?.buttons.includes("Proceed") === true && opened.buttons.includes("Stop");
			checks["opened.parentIsMain"] = opened?.parent === "main";
			checks["opened.windowNotReplaced"] = (await h.main()).view !== "confirmPrompt";
			out.viewUnderModal = (await h.main()).view;
			await checkPlacement(h, checks, "layout", out);
			await shot(driver, "sdk-confirm", "modal", CONFIRM, out);
			await h.key("tab");
			checks["focus.tabMovesToStop"] = (await h.modal())?.focused === "cancel";
			await h.key("enter");
			checks["enterOnStop.closes"] = Boolean(await until(async () => !(await h.modal())));
			const result = await until(async () => {
				if (!existsSync(resultPath)) return null;
				return JSON.parse(readFileSync(resultPath, "utf8")) as Json;
			}, 8_000);
			checks["enterOnStop.scriptGotFalse"] = result?.confirmed === false;
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

			// Reopen: the field holds the alias and Remove Alias is offered.
			const reopened = await openEditor();
			checks["reopen.showsAlias"] = reopened?.field === "mx";
			checks["reopen.offersRemove"] = reopened?.buttons.includes("Remove Alias") === true;
			await shot(driver, "alias-editor", "existing", CONFIRM, out);
			await h.key("tab");
			checks["remove.tabReachesRemove"] = (await h.modal())?.focused === "secondary";
			await h.key("enter");
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
		const { driver, kit } = await launch("hotkey-hides-modal");
		const h = makeHelpers(driver);
		const checks: Checks = {};
		const out: Json = { checks };
		try {
			const name = "Modal Audit Hotkey Target";
			const file = await addFixtureScript(driver, kit, name);
			await openLauncherAction(h, "delete");
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
			checks["hotkey.keptScript"] = existsSync(file);
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
