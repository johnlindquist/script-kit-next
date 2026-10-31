#!/usr/bin/env bun
/** One real SDK script, one hidden GPUI window, no native input or external services.
 * bun scripts/devtools/sdk-form-correction.ts <app.reference.json> <new-output-dir> <keyboard|button|cancel>
 */
import assert from 'node:assert/strict';
import { rmSync, writeFileSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { beginManagedTask, claimOutput, finalizeManagedTask, validateOutputTarget } from '../agentic/artifact-lifecycle';
import { paintNodes } from '../agentic/one-window-frames';
import { readArtifactReference } from './design';
import { boundedObservation, DriverLifecycleError, unknownOwnedCleanup, type Json } from './driver';
import { OwnedEvaluationClient, type SdkPromptCommand } from './lib/owned-evaluation';
import { aggregateCleanup } from './lib/story-contract';
import { launchFormCorrectionFixture } from './sdk-journey';

const [referencePath, outPath, route] = process.argv.slice(2);
if (!referencePath || !outPath || (route !== 'keyboard' && route !== 'button' && route !== 'cancel')) throw Error('Expected artifact reference, new output directory, and keyboard|button|cancel');
const root = resolve(import.meta.dir, '../..'), out = resolve(outPath), fixtureId = 'sdk.form-correction.v1';
const reference = readArtifactReference(resolve(referencePath));
const claim = claimOutput(validateOutputTarget({repoRoot:root, candidate:out, kind:'directory', probeId:'sdk-form-correction'}));
const task = beginManagedTask(claim, 'evidence-run', [reference]);
let owned: OwnedEvaluationClient | undefined, failedLaunch = unknownOwnedCleanup(false);
let child: Awaited<ReturnType<typeof launchFormCorrectionFixture>> | undefined;
const steps: Json[] = [];
let passed = false, error: string | null = null;
async function observed<T>(p: Promise<T>) {
  const result = await boundedObservation(p, 8000);
  if (!result.completed) throw Error('SDK observation failed after '+(steps.at(-1)?.label ?? 'launch')+': '+String(result.error));
  return result.value;
}
try {
  const client = owned = await OwnedEvaluationClient.launch(root, reference, claim, ['main.script-list', fixtureId], undefined, {nativeGlass:'disabled'});
  const target = await client.mount('main.script-list');
  const control = async (command: SdkPromptCommand) => {
    const result = await client.design({operation:'sdkPrompt',target,expected:(await client.inspect(target)).targetIdentity,command});
    if (result.operation !== 'sdkPrompt' || !result.ok) throw Error('SDK control refused');
    return result;
  };
  child = await launchFormCorrectionFixture(claim, route);
  const write = async (message: Json) => { child!.process.stdin.write(JSON.stringify(message)+'\n'); await child!.process.stdin.flush(); };
  const prompt = await observed(child.prompt); steps.push({label:'sdk-request',type:prompt.type});
  await control({operation:'begin', fixtureId, message:prompt, channel:'connected'});
  const rpc = await observed(child.rpc), state = await client.inspect(target);
  assert.equal(state.promptId, prompt.id);
  steps.push({label:'form-mounted',id:state.promptId,replyBytes:Buffer.byteLength(JSON.stringify(state))});
  // The SDK fixture uses only these getState fields. Don't copy the evaluator's full frame-history bundle onto script stdin.
  await write({type:state.type,requestId:rpc.requestId,promptId:state.promptId,promptType:state.promptType});
  assert.equal((await observed(child.rpcEvidence)).promptResolved, false);
  const key = (key:string,modifiers:string[]=[],text?:string) => client.act(target,{type:'key',key,modifiers,...(text?{text}:{})});
  const submit = async () => {
    if (route !== 'button') return key('enter',['cmd']);
    const f = await client.captureFrame(target,false);
    const node = paintNodes(f.layout).find(x=>x.name==='footer-action:run' && x.visible);
    assert.ok(node,'Submit must be painted');
    return client.act(target,{type:'gpuiEvent',frame:f.frame,event:{type:'mouseClick',button:'left',x:node.bounds.x+node.bounds.w/2,y:node.bounds.y+node.bounds.h/2}},f.state.targetIdentity);
  };
  const capture = async (label:string) => {
    const frame = await client.captureFrame(target,true);
    writeFileSync(join(out,label+'.png'),Buffer.from(frame.snapshot.capture!.pngBase64!,'base64'));
    const prompt = frame.state.promptObservation;
    const elements = frame.elements.elements as Json[];
    steps.push({label,frame:frame.frame,selectedIndex:prompt.selectedIndex,values:prompt.values,error:prompt.validationError,
      statuses:elements.filter(x=>x.statusKind || x.role==='alert').map(x=>({id:x.semanticId,status:x.statusKind,text:x.text}))});
    return {prompt,elements};
  };
  await client.frame(target);
  await key('tab');
  const initial=await capture('01-notes');
  assert.equal(initial.prompt.selectedIndex,1); assert.equal(initial.prompt.validationError,null);
  await submit();
  const invalid=await capture('02-invalid');
  assert.equal(invalid.prompt.selectedIndex,0);
  assert.equal(invalid.elements.find(x=>x.semanticId==='input:form-email')?.statusKind,'invalid');
  assert.ok(invalid.elements.some(x=>x.semanticId==='panel:form-error-0' && x.role==='alert' && x.text===invalid.prompt.validationError));
  assert.equal((await control({operation:'drain'})).forwarded,0);
  await key('a',['cmd']);
  for (const ch of 'reader@example.org') await key(ch,[],ch);
  const corrected=await capture('03-corrected');
  assert.equal(corrected.prompt.input,'reader@example.org'); assert.equal(corrected.prompt.validationError,null);
  assert.ok(!corrected.elements.some(x=>x.role==='alert'||x.statusKind==='invalid'));
  await key('tab'); await key('enter');
  for (const ch of 'Second line') await key(ch,[],ch);
  const draft=await capture('04-draft');
  const expected={email:'reader@example.org',notes:'First line\nSecond line'};
  assert.deepEqual(Object.fromEntries(draft.prompt.values),expected); assert.equal((await control({operation:'drain'})).forwarded,0);
  if (route==='cancel') await key('escape'); else await submit();
  const delivered=await control({operation:'drain'});
  assert.equal(delivered.forwarded,1); assert.equal(delivered.messages.length,1);
  const message=delivered.messages[0]!; assert.equal(message.id,prompt.id); assert.equal(message.type,'submit');
  if (route==='cancel') assert.equal(message.value,null); else assert.deepEqual(JSON.parse(message.value),expected);
  if (route!=='cancel') await submit();
  const repeat=await control({operation:'drain'});
  assert.equal(repeat.messages.length,0); assert.equal(repeat.forwarded,1);
  await write(message);
  const exit=await observed(child.exitEvidence); assert.equal(await observed(child.process.exited),0); await observed(child.readers);
  assert.equal(exit.received.length,1); assert.equal(exit.promptResolved,route!=='cancel'); assert.deepEqual(exit.value,route==='cancel'?null:expected);
  steps.push({label:'script-exit',delivered:delivered.forwarded,repeatDelivered:repeat.messages.length,resolved:exit.promptResolved,value:exit.value,code:exit.code});
  await control({operation:'close'}); passed=true;
} catch (e) {
  if (!owned && e instanceof DriverLifecycleError) failedLaunch=e.cleanup;
  error=String(e); throw e;
}
finally {
  const [app,sdk] = await Promise.all([owned?.close().catch(()=>owned!.cleanup) ?? failedLaunch,child?.process.close().catch(()=>unknownOwnedCleanup(true))]);
  const cleanup = finalizeManagedTask(task,{...aggregateCleanup(sdk?[app,sdk]:[app]),ownedWindowsClosed:app.ownedWindowsClosed}).cleanup;
  if (app.closed && owned?.driver.sessionDir.startsWith(out+'/')) rmSync(owned.driver.sessionDir,{recursive:true,force:true});
  const sandbox=child?.provenance.sandbox;
  if (sdk?.closed && typeof sandbox==='string' && sandbox.startsWith(out+'/')) rmSync(sandbox,{recursive:true,force:true});
  writeFileSync(join(out,'result.json'),JSON.stringify({route,fixtureId,passed:passed&&cleanup.closed,error,reference,steps,cleanup},null,2));
  if (!cleanup.closed) throw Error('Unconfirmed fixture cleanup');
}
