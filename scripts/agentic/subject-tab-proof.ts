// Exercise the production launcher with an owned, hidden GPUI window and a process-local clipboard.
// bun scripts/agentic/subject-tab-proof.ts <immutable app reference> <fresh output under .test-output>
import assert from 'node:assert/strict';
import { readdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { OwnedEvaluationClient } from '../devtools/lib/owned-evaluation';
import { readArtifactReference } from '../devtools/design';
import { claimOutput, validateOutputTarget, beginManagedTask, finalizeManagedTask, emptyOwnedCleanup } from './artifact-lifecycle';
import { DriverCommandRefused, DriverLifecycleError } from '../devtools/driver';
import { paintNodes } from './one-window-frames';

const [refPath, destination] = process.argv.slice(2);
if (!refPath || !destination) throw Error('expected artifact reference and a new evidence directory');
const ref = readArtifactReference(resolve(refPath)), out = resolve(destination), fixture='main.subject-tabs';
const claim=claimOutput(validateOutputTarget({repoRoot:process.cwd(),candidate:out,kind:'directory',probeId:'subject-tabs'}));
const task=beginManagedTask(claim,'evidence-run',[ref]);
const client=await OwnedEvaluationClient.launch(process.cwd(),ref,claim,[fixture],undefined,{nativeGlass:'disabled'}).catch(error=>{
  // Artifact verification can refuse before a driver exists; never strand the evidence reference.
  finalizeManagedTask(task,error instanceof DriverLifecycleError ? error.cleanup : emptyOwnedCleanup());
  throw error;
});
const session=client.driver.sessionDir, receipts: Record<string,unknown>={};
try {
  const target=await client.mount(fixture); await client.frame(target);
  const key=(key:string,modifiers:string[]=[])=>client.act(target,{type:'key',key,modifiers});
  const state=()=>client.inspect(target);
  const page=async()=>(await state()).oneWindow.subjectTabs.find((t:any)=>t.selected)?.text;
  const frame=async(name:string)=>{
    const f=await client.captureFrame(target,true);writeFileSync(join(out,name+'.png'),Buffer.from(f.snapshot.capture.pngBase64,'base64'));return f;
  };
  const opened=await frame('opening');const tabs=opened.state.oneWindow.subjectTabs;
  assert.deepEqual(tabs.map((t:any)=>t.text),['Selected text','Copied link','Ghostty','Last run']);
  assert.deepEqual(tabs.map((t:any)=>t.kind),['text-cursor','link','terminal','history']);
  const first=opened.elements.elements.find((e:any)=>e.role==='row'&&e.selected);assert.equal(first.text,'Explain this error');
  const box=paintNodes(opened.layout).find(n=>n.name==='list-row:'+first.semanticId&&n.visible)?.bounds;
  assert.ok(box && box.y>60 && box.y<150 && box.h===44,'first action must actually be in the panel');
  assert.equal(await page(),'Selected text');await key('tab');assert.equal(await page(),'Copied link');
  await key('tab',['shift']);assert.equal(await page(),'Selected text');
  await client.act(target,{type:'select',semanticId:tabs[1].semanticId,submit:false});assert.equal(await page(),'Copied link');
  assert.equal((await state()).oneWindow.enter.label,'Open');
  await key('space',[]);assert.equal((await state()).oneWindow.staged,'https://github.com/script-kit/app/pull/412');
  await key('escape');await key('up');assert.equal(await page(),'Last run');await key('escape');assert.equal(await page(),'Selected text');
  const theme=(await state()).targetIdentity.themeRevision;
  await client.applyTheme(theme,[{tokenId:'theme.colors.background.main',value:0xf9f9f9},{tokenId:'theme.colors.background.searchBox',value:0xffffff}]);
  await client.frame(target);await frame('light');
  // Read the complete selection, then copy it via Return. The clipboard is the evaluator's local sink.
  await key('down');await key('down');await key('enter');await Bun.sleep(450);await key('enter');
  const copied=await state();assert.equal(copied.copySink?.text,'TypeError: Cannot read properties of undefined');assert.equal(copied.copySink?.receipt.destination,'ownedProcessLocal');receipts.copy=copied.copySink;
  // Return to the launcher before changing the tab. ⌘V must select, not stage, the duplicate selection.
  if ((await state()).oneWindow.view) await key('escape');
  await client.act(target,{type:'select',semanticId:tabs[2].semanticId,submit:false});await key('v',['cmd']);
  assert.equal(await page(),'Selected text');assert.ok(!(await state()).oneWindow.staged);
  await key('down');
  for(let i=0;i<30 && (await state()).oneWindow.enterArmed;i++) await Bun.sleep(40);
  const ready=(await state()).oneWindow;assert.equal(ready.enter.label,'Add to note');assert.ok(!ready.enterArmed);
  const selected=(await client.query(target,'elements')).elements.find((e:any)=>e.role==='row'&&e.selected);
  assert.equal(selected.text,'Add selection to today’s note');
  // Ages tick in the real panel. Re-inspect only after a proven pre-dispatch refusal;
  // a changed selection, source generation, view, theme or an uncertain submit is terminal.
  const baseline=(await state()).targetIdentity;
  for(let attempt=0;;attempt++){
    const now=await state(), bound=now.targetIdentity;
    for(const k of ['windowId','windowGeneration','appViewVariant','targetGeneration','surfaceGeneration','dataGeneration','themeRevision']) assert.equal(bound[k],baseline[k],k);
    assert.equal(now.oneWindow.enter.label,'Add to note');assert.ok(!now.oneWindow.enterArmed);
    const row=(await client.query(target,'elements')).elements.find((e:any)=>e.role==='row'&&e.selected);
    assert.equal(row?.semanticId,selected.semanticId);
    try {await client.act(target,{type:'select',semanticId:selected.semanticId,submit:true},bound); receipts.presentationRefusals=attempt;break;}
    catch(err){if(attempt>=3 || !(err instanceof DriverCommandRefused) || err.code!=='stale_target_identity')throw err;}
  }
  await Bun.sleep(650);receipts.saveStatus=(await state()).dayPage?.saveStatus;
  const days=join(session,'notes/brain/days');const saved=readdirSync(days).filter(f=>f.endsWith('.md')).map(f=>readFileSync(join(days,f),'utf8')).find(s=>s.includes('TypeError: Cannot read properties of undefined'));
  assert.ok(saved,'the selected text must be in the saved day page');assert.equal(saved.split('TypeError: Cannot read properties of undefined').length-1,1,'submit must not be replayed');receipts.savedNote=saved;await frame('saved-note');
} finally {
  const closed=finalizeManagedTask(task,await client.close()).cleanup;receipts.closed=closed.closed;
  if(closed.closed&&session.startsWith(out+'/'))rmSync(session,{recursive:true,force:true});
  writeFileSync(join(out,'outcomes.json'),JSON.stringify(receipts,null,2));assert.ok(closed.closed);
}
console.log('Subject tabs: pixels, semantic selection, keyboard, copy and saved note verified');
