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
 * The key comes from the environment. --score-holdout DIR records a reviewed synthetic batch without changing replay fixtures.
 */
import { existsSync, readdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const ENDPOINT = "https://api.typesafe.ai/v1/systemone";
const repo = join(import.meta.dir, "../../..");
const fixturePath = join(repo, "src/intent/fixtures/evaluation.json");
const pending = join(process.env.TMPDIR ?? tmpdir(), "intent-evaluation");

const apiKey = process.env.SCRIPT_KIT_TYPESAFE_API_KEY?.trim() || process.env.TYPESAFE_API_KEY?.trim();
const model = process.env.TYPESAFE_MODEL?.trim() || "jev-1.14.0";
if (!apiKey) {console.error("Set TYPESAFE_API_KEY in the recorder environment");process.exit(2);}
if (["--score-holdout","--score-typing"].includes(process.argv[2])) {
  const count = process.argv[2] === "--score-typing" ? 3 : 10;
  const dir=process.argv[3]; if (!dir || process.argv.length !== 4) throw Error("Supply the reviewed synthetic holdout directory");
  const rows=JSON.parse(readFileSync(join(dir,"requests.json"),"utf8"));
  if (!Array.isArray(rows) || rows.length!==count || rows.some((r,i)=>r.id!==i || r.payload.model!=="jev-1.14.0" || JSON.stringify(r.payload).length>24000)) throw Error("Invalid or oversized batch");
  // Reserve before transmission: a crash or timeout is not permission to rebill.
  writeFileSync(join(dir,"attempted"),"pinned synthetic holdout\n",{flag:"wx",mode:0o600});
  const samples:{ms:number,tokens:number}[]=[];
  for (const {id,payload} of rows) {
    if (samples.reduce((n,s)=>n+s.tokens,0) + 65536 > 350000) throw Error("Batch token ceiling reached; stopped before billing");
    const start=performance.now();const res=await fetch(ENDPOINT,{method:"POST",redirect:"error",signal:AbortSignal.timeout(12000),headers:{Authorization:`Bearer ${apiKey}`,"Content-Type":"application/json"},body:JSON.stringify(payload)});
    if(!res.ok) throw Error(`case ${id}: HTTP ${res.status}; stopped`);
    const body=await res.json();if(body.model!==payload.model || !body.answers || !Number.isFinite(body.usage?.input_tokens))throw Error("Unverified provider receipt");
    writeFileSync(join(dir,`reply-${id}.json`),JSON.stringify(body),{flag:"wx",mode:0o600}); samples.push({ms:Math.round(performance.now()-start),tokens:body.usage.input_tokens});
  }
  const times=samples.map(s=>s.ms).sort((a,b)=>a-b);const timing={model:"jev-1.14.0",samples,p50Ms:times[Math.ceil(times.length*.50)-1],p95Ms:times[Math.ceil(times.length*.95)-1],inputTokens:samples.reduce((n,s)=>n+s.tokens,0)};
  writeFileSync(join(dir,"timing.json"),JSON.stringify(timing,null,2));console.log(timing);process.exit(0);
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
if (Object.keys(fixture.answers).length && fixture.model !== model) {
  throw Error(`Existing recordings use ${fixture.model}; refusing to mix models. Use that pinned model or a separate holdout.`);
}
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
