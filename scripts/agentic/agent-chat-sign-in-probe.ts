#!/usr/bin/env bun
/**
 * Agent Chat provider sign-in — runtime proof, keyboard only.
 *
 * Drives the real app against the deterministic goose ACP shim
 * (`mock-goose-acp.ts` with MOCK_GOOSE_SIGN_IN=1), which behaves like goose's
 * ChatGPT login: a prompt without a stored sign-in waits for the token file,
 * and a simulated user "finishes signing in" by writing it. No real browser
 * or account is involved. Every step is a key press.
 *
 * Scenario `first-use` (default):
 *   1. The launcher marks the model "Not signed in"; Agent Chat says so too.
 *   2. Sending a message opens the "Sign in to ChatGPT" modal (↵ Sign In,
 *      Esc Not Now) — nothing opens the browser yet.
 *   3. ↵ starts the sign-in: the main window steps aside for the browser.
 *   4. Reopening Script Kit mid-sign-in shows the chat and a waiting note.
 *   5. The simulated approval lands: the reply arrives in the same chat and
 *      the launcher marker is gone.
 *
 * Scenario `decline`: Esc on the consent modal cancels the turn without ever
 * starting a sign-in; the chat stays open and still says "Not signed in".
 *
 * Scenario `retry`: the first sign-in times out → the window comes back with
 * the "sign-in didn't finish" modal (no in-window error card or error row,
 * footer offers Send, not Stop) → ↵ Try Again goes straight to the sign-in
 * page without asking again → the second sign-in succeeds → the reply arrives
 * and the user's message was sent once.
 *
 * Run: bun scripts/agentic/agent-chat-sign-in-probe.ts [first-use|decline|retry]
 *      [--receipt <path>]
 */
import { Driver, type Json } from "../devtools/driver.ts";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

const scenario = (["retry", "decline"] as const).find((name) => name === process.argv[2]) ?? "first-use";
const receiptArg = process.argv.indexOf("--receipt");
const receiptPath = receiptArg >= 0 ? process.argv[receiptArg + 1] : undefined;
const binary = process.env.SCRIPT_KIT_GPUI_BINARY;
const mockGoose = resolve(import.meta.dir, "mock-goose-acp.ts");
const shotDir = resolve(".test-screenshots/agent-chat-sign-in");
mkdirSync(shotDir, { recursive: true });
const fixtureDir = mkdtempSync(join(tmpdir(), "agent-chat-sign-in-"));
const recordPath = join(fixtureDir, "goose-record.ndjson");

const signInEnv: Record<string, string> =
	scenario !== "retry"
		? { MOCK_GOOSE_SIGN_IN_APPROVE_AFTER_MS: "8000" }
		: {
				// First attempt never approves and times out; the second is
				// approved well before its own timeout.
				MOCK_GOOSE_SIGN_IN_TIMEOUT_MS: "5000",
				MOCK_GOOSE_SIGN_IN_APPROVE_FROM_ATTEMPT: "2",
				MOCK_GOOSE_SIGN_IN_APPROVE_AFTER_MS: "2000",
			};

const receipt: Json = { probe: "agent-chat-sign-in", scenario, binary: binary ?? null, checks: {} };
const checks: Record<string, boolean> = {};

const driver = await Driver.launch({
	sessionName: `agent-chat-sign-in-${scenario}`,
	binary,
	sandboxHome: true,
	// No local models are needed; do not depend on the operator's models folder.
	sharedModels: false,
	env: {
		SCRIPT_KIT_GOOSE_BINARY: mockGoose,
		MOCK_GOOSE_SIGN_IN: "1",
		MOCK_GOOSE_RECORD_PATH: recordPath,
		SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1",
		...signInEnv,
	},
	readyTimeoutMs: 30_000,
	defaultTimeoutMs: 15_000,
});
receipt.sessionDir = driver.sessionDir;
// The fresh sandbox would show Script Kit's one-time first-run hint toasts on
// the first window show; mark them seen, as they are for anyone past day one,
// so screenshots show the sign-in flow itself.
for (const marker of ["gesture-hint-tap-dismiss.json", "sediment-disclosure.json"]) {
	writeFileSync(join(driver.sessionDir, "home", ".scriptkit", marker), "{}");
}

function gooseRecords(): Json[] {
	try {
		return readFileSync(recordPath, "utf8").trim().split("\n").filter(Boolean).map((line) => JSON.parse(line));
	} catch {
		return [];
	}
}

async function chatState(): Promise<Json> {
	return driver.request(
		{ type: "getAgentChatState", target: { type: "id", id: "main" } },
		{ expect: "agent_chatStateResult", timeoutMs: 8_000 },
	);
}

async function mainVisible(): Promise<boolean> {
	const state = (await driver.getState({ timeoutMs: 8_000 })) as Json;
	return state.windowVisible === true;
}

async function elementsText(): Promise<string> {
	return JSON.stringify(
		await driver.request({ type: "getElements", target: { type: "id", id: "main" }, limit: 400 }, { timeoutMs: 8_000 }),
	);
}

async function until<T>(probe: () => Promise<T | null | undefined | false>, timeoutMs: number): Promise<T | null> {
	const deadline = Date.now() + timeoutMs;
	while (Date.now() < deadline) {
		const value = await probe().catch(() => null);
		if (value) return value as T;
		await Bun.sleep(150);
	}
	return null;
}

/** Native footer capsule host installs so far; a settled screen adds none. */
function footerHostInstalls(): number {
	try {
		const log = readFileSync(join(driver.sessionDir, "home", ".scriptkit", "logs", "latest-session.jsonl"), "utf8");
		return log.split("dictation_glass_button_host_installed").length - 1;
	} catch {
		return -1;
	}
}

/** Buttons of the most recent Agent Chat footer snapshot the app applied. */
function lastFooterSnapshot(): string[] | null {
	try {
		const log = readFileSync(join(driver.sessionDir, "home", ".scriptkit", "logs", "latest-session.jsonl"), "utf8");
		const matches = [...log.matchAll(/"event":"agent_chat_footer_snapshot_changed","buttons":"(\[[^"]*(?:\\"[^"]*)*\])"/g)];
		const last = matches.at(-1)?.[1];
		return last ? (JSON.parse(last.replaceAll('\\"', '"')) as string[]) : null;
	} catch {
		return null;
	}
}

/** A settled window must not rebuild its native footer every frame. */
async function settledFooter(label: string): Promise<boolean> {
	await Bun.sleep(500);
	const before = footerHostInstalls();
	await Bun.sleep(2_000);
	const added = footerHostInstalls() - before;
	receipt[`footerInstalls:${label}`] = added;
	return before >= 0 && added <= 2;
}

async function shot(name: string): Promise<void> {
	const result = (await driver.captureScreenshot({
		target: { type: "id", id: "main" },
		savePath: join(shotDir, `${scenario}-${name}.png`),
		timeoutMs: 10_000,
	})) as Json;
	receipt[`shot:${name}`] = result?.error ?? join(shotDir, `${scenario}-${name}.png`);
}

async function key(keyName: string, modifiers: string[] = []): Promise<void> {
	await driver.simulateGpuiEvent(
		{ type: "keyDown", key: keyName, ...(modifiers.length ? { modifiers } : {}) },
		{ target: { type: "kind", kind: "main" } },
	);
}

/** The shared confirm/action modal, when one is open. */
async function modal(): Promise<Json | null> {
	const listed = (await driver.listAutomationWindows({ timeoutMs: 5_000 })) as Json;
	const windows = (listed.windows ?? []) as Json[];
	const popup = windows.find((window) => window.id === "confirm-popup" || window.semanticSurface === "confirmDialog");
	if (!popup) return null;
	const elements = await driver.getElements({ target: { type: "id", id: "confirm-popup" }, limit: 40 }, { timeoutMs: 5_000 });
	return { ...popup, text: JSON.stringify(elements) };
}

async function modalShot(name: string): Promise<void> {
	const result = (await driver.captureScreenshot({
		target: { type: "id", id: "confirm-popup" },
		savePath: join(shotDir, `${scenario}-${name}.png`),
		timeoutMs: 10_000,
	})) as Json;
	receipt[`shot:${name}`] = result?.error ?? join(shotDir, `${scenario}-${name}.png`);
}

const QUESTION = "What is 17 times 3?";

try {
	await driver.request({ type: "show" });
	await driver.waitForSettle();

	// 1. Launcher marks the default model "Not signed in".
	checks["launcher.marksNotSignedIn"] = (await elementsText()).includes("Not signed in");
	await shot("1-launcher-not-signed-in");

	// 2. ⌘↵ on an empty launcher opens a clean chat that already says the
	// provider is not signed in.
	await key("enter", ["cmd"]);
	const before = await until(async () => {
		const state = await chatState();
		return state.signIn ? state : null;
	}, 10_000);
	receipt.beforeSend = { signIn: before?.signIn ?? null };
	checks["chat.notSignedInBeforeSend"] = before?.signIn === "notSignedIn";
	await Bun.sleep(300);
	await shot("2-chat-not-signed-in");

	// 3. Esc to the launcher, type the question, ⌘↵: the consent modal opens
	// and nothing has started a sign-in yet.
	await key("escape");
	await driver.request({ type: "show" });
	await until(async () => mainVisible(), 5_000);
	await driver.setFilterAndWait(QUESTION, { timeoutMs: 10_000 });
	await key("enter", ["cmd"]);
	const consent = await until(async () => modal(), 10_000);
	receipt.consentModal = consent ? { id: consent.id, parent: consent.parentWindowId ?? null } : null;
	checks["consent.modalOpened"] = Boolean(consent?.text?.includes("Sign in to ChatGPT"));
	checks["consent.keyboardHints"] = Boolean(consent?.text?.includes("Sign In") && consent?.text?.includes("Not Now"));
	checks["consent.noBrowserYet"] = !gooseRecords().some((r) => r.event === "oauth_started");
	checks["consent.stateAwaiting"] = (await chatState()).signIn === "awaitingConsent";
	await Bun.sleep(300);
	await modalShot("3-consent-modal");
	await shot("3-consent-main");

	if (scenario === "decline") {
		driver.simulateKey("escape", []);
		const closed = await until(async () => !(await modal()), 5_000);
		checks["decline.modalClosed"] = closed === true;
		await Bun.sleep(1_000);
		const state = await chatState();
		receipt.declined = { status: state.status, signIn: state.signIn ?? null, phase: state.reliability?.phase ?? null };
		checks["decline.noSignInStarted"] = !gooseRecords().some((r) => r.event === "oauth_started");
		checks["decline.windowStays"] = await mainVisible();
		checks["decline.notSignedInNote"] = state.signIn === "notSignedIn";
		checks["decline.notAnError"] = state.reliability?.phase !== "awaitingRecovery";
		checks["decline.footerSettled"] = await settledFooter("decline");
		await shot("4-declined");
	} else {
		// 4. ↵ Sign In: the sign-in starts and the window steps aside.
		driver.simulateKey("enter", []);
		checks["signIn.startedAfterEnter"] = Boolean(
			await until(async () => gooseRecords().some((r) => r.event === "oauth_started"), 10_000),
		);
		checks["signIn.stateWaiting"] = Boolean(
			await until(async () => (await chatState()).signIn === "waitingForBrowser", 5_000),
		);
		checks["window.hiddenForBrowser"] = Boolean(await until(async () => !(await mainVisible()), 3_000));

		if (scenario === "first-use") {
			// 5. Reopening Script Kit mid-sign-in keeps the chat and the note.
			await driver.request({ type: "show" });
			await until(async () => mainVisible(), 5_000);
			await Bun.sleep(300);
			checks["reopen.keepsChatAndWaiting"] = (await chatState()).signIn === "waitingForBrowser";
			await shot("4-reopened-while-waiting");
		} else {
			// Retry: the first sign-in times out → the window comes back with
			// the sign-in-didn't-finish modal; no card, no error row.
			const failed = await until(async () => {
				const popup = await modal();
				return popup?.text?.includes("didn't finish") ? popup : null;
			}, 15_000);
			checks["failure.modalOpened"] = Boolean(failed);
			checks["failure.windowBack"] = await mainVisible();
			const state = await chatState();
			receipt.failure = state.reliability ?? null;
			receipt.failureStatus = state.status;
			checks["failure.signInState"] = state.signIn === "failed";
			checks["failure.noErrorRow"] = !((state.transcriptScroll?.rowSemanticIds ?? []) as string[]).some((id) =>
				id.includes("-error-"),
			);
			checks["failure.noInWindowCard"] = !(await elementsText()).includes("Sign in required");
			checks["failure.footerOffersNoStop"] = state.status !== "streaming";
			checks["failure.footerSettled"] = await settledFooter("failure");
			await modalShot("4-failed-modal");
			await shot("4-failed-main");
			// ↵ Try Again: straight to the sign-in page, no second consent.
			driver.simulateKey("enter", []);
			checks["retry.signInAgainWithoutAsking"] = Boolean(
				await until(async () => gooseRecords().filter((r) => r.event === "oauth_started").length >= 2, 10_000),
			);
			checks["retry.noSecondConsent"] = !((await modal())?.text ?? "").includes("Sign in to ChatGPT");
			checks["retry.windowStepsAsideAgain"] = Boolean(await until(async () => !(await mainVisible()), 3_000));
		}

		// 6. The simulated approval lands: reply in the same chat, window back.
		const done = await until(async () => {
			const state = await chatState();
			return state.reliability?.phase === "succeeded" ? state : null;
		}, 20_000);
		receipt.done = {
			status: done?.status ?? null,
			phase: done?.reliability?.phase ?? null,
			signIn: done?.signIn ?? null,
			rows: done?.transcriptScroll?.rowSemanticIds ?? [],
		};
		checks["reply.arrived"] = Boolean(done);
		checks["reply.signInCleared"] = !done?.signIn || done.signIn === "signedIn";
		checks["window.backAfterSignIn"] = Boolean(await until(async () => mainVisible(), 5_000));
		const userRows = ((done?.transcriptScroll?.rowSemanticIds ?? []) as string[]).filter((id) => id.includes("-user-"));
		checks["reply.messageSentOnce"] = userRows.length === 1;
		checks["reply.footerSettled"] = await settledFooter("reply");
		// The finished turn's footer offers the next step, never Stop. The
		// native footer draws the app's latest Agent Chat footer snapshot.
		receipt.replyFooterButtons = lastFooterSnapshot();
		checks["reply.footerNotStop"] = Array.isArray(receipt.replyFooterButtons)
			&& !(receipt.replyFooterButtons as string[]).includes("Stop");
		await shot("5-signed-in-reply");

		// The launcher no longer says "Not signed in".
		await key("escape");
		await driver.waitForSettle();
		await key("escape");
		await driver.request({ type: "show" });
		await driver.waitForSettle();
		checks["launcher.markerGone"] = !(await elementsText()).includes("Not signed in");
		await shot("6-launcher-signed-in");
	}
} catch (error) {
	receipt.error = error instanceof Error ? error.stack ?? error.message : String(error);
} finally {
	receipt.gooseRecords = gooseRecords();
	await driver.close();
	rmSync(fixtureDir, { recursive: true, force: true });
}

receipt.checks = checks;
receipt.failedChecks = Object.entries(checks).filter(([, ok]) => !ok).map(([name]) => name);
receipt.pass = !receipt.error && (receipt.failedChecks as string[]).length === 0;
const serialized = JSON.stringify(receipt, null, 2);
if (receiptPath) writeFileSync(receiptPath, serialized);
console.log(serialized);
process.exit(receipt.pass ? 0 : 1);
