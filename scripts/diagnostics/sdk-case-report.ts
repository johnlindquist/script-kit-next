// Read a real SDK report, emit only reviewed fixture locators and bounded measurements.
// This does not grade the release or retry a test. Never echo errors, result values, or paths.
import { lstatSync, readFileSync, readdirSync } from "node:fs";
import { resolve, join } from "node:path";
import { SDK_SYSTEM_INPUT_TESTS } from "../../tests/sdk/system-input-tests.ts";

const source = resolve(import.meta.dir, "../../tests/sdk");
const reviewed = new Set(readdirSync(source, {withFileTypes:true}).filter(x => x.isFile() &&
  /^test-[a-zA-Z0-9_-]{1,80}\.ts$/.test(x.name) && !SDK_SYSTEM_INPUT_TESTS.some(excluded=>excluded===x.name)).map(x=>x.name));
const bounded = (n: unknown, max: number) => typeof n === "number" && Number.isSafeInteger(n) && n >= 0 && n <= max ? n : null;
const sha = (v: unknown) => typeof v === "string" && /^[0-9a-f]{40}$/.test(v) ? v : null;
const identity = {schemaVersion:1, type:"sdk-case-diagnostic", diagnosticSha:sha(process.env.GITHUB_SHA), tagSource:sha(process.env.DIAGNOSTIC_TAG_SHA)};
function report(path: string) {
  const meta=lstatSync(path);
  if (!meta.isFile() || meta.size===0 || meta.size>8*1024**2) throw Error();
  const raw=JSON.parse(readFileSync(path,"utf8"));
  if (raw.mode!=="parallel" || !Array.isArray(raw.files) || !raw.files.length || raw.files.length>256 ||
    [raw.total_passed,raw.total_failed,raw.total_skipped].some(n=>bounded(n,100000)===null)) throw Error();
  const cases=[],seen=new Set<string>();let unexpectedOwners=0, unlistedCases=0;
  for(const row of raw.files) {
    if(!reviewed.has(row?.file)||seen.has(row.file)) {unexpectedOwners++;continue;}
    seen.add(row.file);
    if(!Array.isArray(row.tests)||row.tests.length>4096) throw Error();
    const file=join(source,row.file), st=lstatSync(file);
    if(!st.isFile()||st.size>256*1024)throw Error();
    const lines=readFileSync(file,"utf8").split("\n");let terminal=0;
    for(const outcome of row.tests) {
      if(!["pass","fail","skip"].includes(outcome?.status)) continue;
      terminal++;if(outcome.status!=="fail")continue;
      if(cases.length>=32){unlistedCases++;continue;}
      const id=typeof outcome.test==="string"&&/^[a-zA-Z0-9_-]{1,140}$/.test(outcome.test)?outcome.test:null;
      const nameLines:number[]=[],messageLines:number[]=[];
      const error=typeof outcome.error==="string"?outcome.error.slice(0,4096):"";
      for(const [i,line] of lines.entries()) {
        if(id&&["\"","'","`"].some(q=>line.includes(q+id+q))&&nameLines.length<5)nameLines.push(i+1);
        // Match a public static prefix; report only the line number, never the prefix or error text.
        for(const m of line.matchAll(/["'`]([^"'`]{10,220})["'`]/g)) {
          const prefix=m[1]!.split("${")[0]!.trim();
          if(prefix.length>=10&&error.includes(prefix)&&messageLines.length<5){messageLines.push(i+1);break;}
        }
      }
      cases.push({file:row.file,terminal,publicCase:nameLines.length?id:null,nameLines,messageLines,
        durationMs:bounded(outcome.duration_ms,120000),fileDurationMs:bounded(row.duration_ms,120000)});
    }
  }
  return {...identity,observation:"recorded",passed:raw.total_passed,failed:raw.total_failed,skipped:raw.total_skipped,
    durationMs:bounded(raw.total_duration_ms,300000),reviewedFiles:seen.size,unexpectedOwners,unlistedCases,cases};
}
try { console.log(JSON.stringify(report(process.argv[2]!),null,2)); }
catch { console.log(JSON.stringify({...identity,observation:"missing-or-invalid"}));process.exitCode=1; }
