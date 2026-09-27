#!/usr/bin/env bun
/**
 * record-jev-answers.ts — asks Jev the evaluation sentences' questions once and keeps its answers.
 *
 * `agent-cargo.sh test --lib intent::replay_tests::evaluation` writes every round it has no answers
 * for to $TMPDIR/intent-evaluation/<slug>.json ({ key, ctx, request, state, questions }), exactly as
 * the engine would send it. This posts each one to Jev and stores the answers, trimmed the way
 * replay.json keeps them, in src/intent/fixtures/evaluation.json under the round's key. Run the
 * test again: a compound sentence's clause rounds only appear once its first round is answered.
 *
 * The key is read from TYPESAFE_API_KEY or ~/dev/jev-demos/.env and never printed.
 */
import { existsSync, readdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { homedir, tmpdir } from "node:os";
import { join } from "node:path";

const ENDPOINT = "https://api.typesafe.ai/v1/systemone";
const repo = join(import.meta.dir, "../../..");
const fixturePath = join(repo, "src/intent/fixtures/evaluation.json");
const pending = join(process.env.TMPDIR ?? tmpdir(), "intent-evaluation");

function demosEnv(): Record<string, string> {
  const path = join(homedir(), "dev/jev-demos/.env");
  if (!existsSync(path)) return {};
  const out: Record<string, string> = {};
  for (const line of readFileSync(path, "utf8").split("\n")) {
    const m = line.trim().match(/^([A-Z_]+)\s*=\s*(.*)$/);
    if (m && m[2].trim()) out[m[1]] = m[2].trim().replace(/^"|"$/g, "");
  }
  return out;
}

const env = demosEnv();
const apiKey = process.env.TYPESAFE_API_KEY?.trim() || env.TYPESAFE_API_KEY;
const model = process.env.TYPESAFE_MODEL?.trim() || env.TYPESAFE_MODEL || "jev-latest";
if (!apiKey) {
  console.error("no TYPESAFE_API_KEY in the environment or ~/dev/jev-demos/.env");
  process.exit(2);
}
if (!existsSync(pending)) {
  console.log(`nothing to record: ${pending} doesn't exist (run the evaluation test first)`);
  process.exit(0);
}

type Answer = { choice?: string; noul?: number; score?: number; probabilities?: Record<string, number> };
function compact(a: Answer): unknown {
  if (typeof a.noul === "number") return a.noul;
  const probs = Object.fromEntries(
    Object.entries(a.probabilities ?? {})
      .filter(([, p]) => p >= 0.005)
      .sort((x, y) => y[1] - x[1])
      .slice(0, 6),
  );
  if (typeof a.score === "number") return [a.score, probs];
  return [a.choice ?? "none", probs];
}

const fixture = existsSync(fixturePath)
  ? JSON.parse(readFileSync(fixturePath, "utf8"))
  : { model, sentences: [], answers: {} };
let recorded = 0;
for (const name of readdirSync(pending).filter((n) => n.endsWith(".json")).sort()) {
  const round = JSON.parse(readFileSync(join(pending, name), "utf8"));
  const started = performance.now();
  const res = await fetch(ENDPOINT, {
    method: "POST",
    headers: { Authorization: `Bearer ${apiKey}`, Accept: "application/json", "Content-Type": "application/json" },
    body: JSON.stringify({ model, state: round.state, questions: round.questions }),
  });
  const text = await res.text();
  if (!res.ok) {
    console.error(`${round.key}: HTTP ${res.status} ${text.slice(0, 300)}`);
    process.exitCode = 1;
    continue;
  }
  const body = JSON.parse(text);
  fixture.answers[round.key] = Object.fromEntries(
    Object.entries(body.answers as Record<string, Answer>).map(([q, a]) => [q, compact(a)]),
  );
  fixture.model = body.model || model;
  rmSync(join(pending, name));
  recorded += 1;
  console.log(`${round.key} (${round.ctx}): ${Object.keys(body.answers).length} answers in ${Math.round(performance.now() - started)} ms`);
}
writeFileSync(fixturePath, `${JSON.stringify(fixture, null, 1)}\n`);
console.log(`recorded ${recorded} round(s) into ${fixturePath}`);
