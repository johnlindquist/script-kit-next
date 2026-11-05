// Drives only the host's verified installed instance and the matching app's offline author.
// No global input, OS clipboard, screenshots, provider, or access to a different workspace.
import {randomUUID} from 'node:crypto';
import {check, SmokeFault, type Json} from './lib/installed-release-contract';
import type {InstalledKit} from './lib/installed-release-host';
export type Author = (q:Json)=>Json;
export const boundaries=['journeysProcessExited','beforeLauncherEdit','afterLauncherEdit','afterExternalEdit','beforeRelaunch','afterRelaunch','afterSelectionUndo','afterPinnedLabelUndo','afterFinalExit'] as const;
export type Boundary=(where:typeof boundaries[number])=>Promise<void>;
const validRev=(r:unknown)=>typeof r==='string' && /^[0-9a-f]{64}$/.test(r);
const stock=(s:Json)=>s.pinnedText==='muted' && s.pinnedWeight==='regular' && s.selectionFirst===null && s.clipboardFirst===null && s.homeMode==='standard';
async function wait(f:()=>Promise<boolean>,code:string){const until=performance.now()+10_000;do{if(await f())return;await Bun.sleep(100);}while(performance.now()<until);check(false,code);}
export async function personalControls(kit:InstalledKit){
  const req=async (q:Json,want:string)=>{const requestId=randomUUID(),r=await kit.request({...q,requestId},want,6000);check(r.requestId===requestId && r.type===want,'personal_reply_unbound');return r;};
  const windows=await req({type:'listAutomationWindows'},'automationWindowListResult');
  check(windows.windows?.length===1,'personal_unique_window_missing');const w=windows.windows[0];check(w.id==='main'&&w.pid===kit.pid&&Number.isSafeInteger(w.generation)&&w.generation>0,'personal_wrong_window');
  const target={type:'instance',id:'main',generation:w.generation};
  const state=async()=>{const s=await req({type:'getState',target},'stateResult');check(s.promptType!=='target_resolution_failed'&&s.permissions?.bundleIdentifier==='com.scriptkit.app','personal_wrong_state');return s;};
  if((await state()).promptType==='permissionsWizard'){
    const e=await req({type:'getElements',target,limit:100},'elementsResult');check(e.semanticSurface==='permissionsWizard'&&e.elements.some((x:Json)=>x.semanticId==='panel:permissions-wizard'),'personal_unrecognized_startup');
    const k=await req({type:'simulateKey',target,key:'escape',modifiers:[]},'externalCommandResult');check(k.ok&&k.command==='simulateKey','personal_wizard_escape_refused');
    await wait(async()=>(await state()).promptType==='none','personal_wizard_not_closed');
    if(!(await state()).windowVisible){const shown=await req({type:'show'},'windowVisibilityAck');check(shown.windowVisible,'personal_launcher_not_visible');}
  }
  const filter=async(text:string)=>{const b=await req({type:'batch',target,commands:[{type:'setInput',text}]},'batchResult');check(b.success && b.results?.length===1 && b.results[0].success && b.results[0].command==='setInput','personal_input_refused');};
  const key=async(value:'enter'|'escape')=>{const k=await req({type:'simulateKey',target,key:value,modifiers:[]},'externalCommandResult');check(k.ok && k.command==='simulateKey','personal_key_refused');};
  const elements=()=>req({type:'getElements',target,limit:100},'elementsResult');
  const applyText=async(words:string,label:string,revision:string)=>{
    await filter(words);
    const ready=async()=>{const s=await state();return s.windowVisible===true && s.inputValue===words && s.oneWindow.personalization?.appliedRevision===revision && s.oneWindow.enterArmed===false && s.oneWindow.enter?.id?.startsWith('personal/launcher/'+revision+'/') && s.oneWindow.enter.label===label;};
    try { await wait(ready,'personal_row_not_ready'); }
    catch (failure) {
      if(!(failure instanceof SmokeFault) || failure.code!=='personal_row_not_ready') throw failure;
      const s=await state(), w=s.oneWindow??{}, e=await elements();
      const action=label==='Apply'?'Use brighter text for Stays open':'Undo my last launcher change';
      const rows=(e.elements??[]).filter((r:Json)=>r.role==='row' && r.text===action && r.selectable && !r.actionDisabled);
      // Flags only: no launcher text, personal row IDs, private titles or paths in the receipt.
      const flags=[s.windowVisible===true,s.inputValue===words,w.owned===true,w.text===words,w.personalization?.appliedRevision===revision,
        w.enterArmed===true,String(w.enter?.id??'').startsWith('personal/launcher/'+revision+'/'),w.enter?.label===label,
        rows.length===1,rows.length===1&&rows[0].selected===true];
      throw new SmokeFault('personal_not_ready_visible_input_owns_text_rev_armed_target_label_row_sel_'+flags.map(Number).join(''));
    }
    const e=await elements();
    const selected=e.elements?.filter((r:Json)=>r.role==='row'&&r.selected===true&&!r.actionDisabled&&r.selectable);check(selected?.length===1 && (label==='Apply'?selected[0].text==='Use brighter text for Stays open':selected[0].text==='Undo my last launcher change'),'personal_wrong_selected_row');
    check(await ready(),'personal_selection_changed');await key('enter');
  };
  const adopted=async(rev:string,matches:(s:Json)=>boolean)=>wait(async()=>{const s=await state();const a=s.oneWindow?.personalization;return s.inputValue==='' && a?.scope==='runningWindow' && a.appliedRevision===rev && matches(a.effective);},'personal_live_revision_not_adopted');
  const selectPane=async(semanticId:string)=>{const b=await req({type:'batch',target,commands:[{type:'selectBySemanticId',semanticId,submit:false}]},'batchResult');check(b.success && b.results?.length===1 && b.results[0].command==='selectBySemanticId' && b.results[0].success,'pane_selection_refused');};
  return {applyText,adopted,state,filter,key,elements,selectPane};
}
const inspect=(a:Author)=>{const r=a({op:'inspect'});check(r.scope==='stored'&&r.safeMode===false&&r.state?.enabled===true,'personal_stored_state_missing');return r.state;};
export type PersonalCheckpoint={workspace:string; pinId:string; selectionId:string; selectionRevision:string};
export async function preparePersonal(kit:InstalledKit,author:Author,boundary:Boundary):Promise<PersonalCheckpoint>{
  const ui=await personalControls(kit);const s0=inspect(author);check(s0.revision==='stock'&&s0.undo===null&&stock(s0.effective),'personal_initial_not_stock');await ui.adopted('stock',stock);await boundary('beforeLauncherEdit');
  await ui.applyText('Make the pinned label brighter','Apply','stock');
  await wait(async()=>{const p=(await ui.state()).oneWindow.personalization;return validRev(p?.appliedRevision)&&p.effective.pinnedText==='primary';},'personal_pinned_label_not_saved');
  const pin=inspect(author);check(validRev(pin.revision)&&pin.workspace===s0.workspace&&typeof pin.undo==='string'&&pin.effective.pinnedText==='primary','personal_pinned_write_unverified');
  const why=author({op:'why',target:'launcher.header.pinned-label.textRole'});check(why.origin==='launcher'&&why.requestId===pin.undo,'personal_launcher_provenance_mismatch');await boundary('afterLauncherEdit');
  const edit={op:'edit',edit:{target:'launcher.subject.selection.firstAction',value:'note'}};const preview=author({op:'preview',change:edit});check(preview.scope==='preview'&&preview.wouldStore?.effective.selectionFirst==='note'&&inspect(author).revision===pin.revision,'personal_preview_wrote_or_misbound');
  const id=randomUUID();const saved=author({op:'apply',request:{workspace:pin.workspace,expectedRevision:pin.revision,requestId:id,change:edit}});
  check(saved.scope==='stored'&&saved.result?.requestId===id&&!saved.result.replayed&&validRev(saved.result.appliedRevision)&&saved.result.state?.revision===saved.result.appliedRevision&&saved.result.state.undo===id,'personal_external_write_unverified');
  await ui.adopted(saved.result.appliedRevision,s=>s.pinnedText==='primary'&&s.selectionFirst==='note');await boundary('afterExternalEdit');
  return {workspace:pin.workspace,pinId:pin.undo,selectionId:id,selectionRevision:saved.result.appliedRevision};
}
export async function recoverPersonal(kit:InstalledKit,author:Author,prior:PersonalCheckpoint,boundary:Boundary){
  const ui=await personalControls(kit);let disk=inspect(author);check(disk.workspace===prior.workspace && disk.revision===prior.selectionRevision && disk.undo===prior.selectionId,'personal_restart_changed_workspace');
  await ui.adopted(disk.revision,s=>s.pinnedText==='primary'&&s.selectionFirst==='note');await boundary('afterRelaunch');
  for(const [expectedUndo,pin,point] of [[prior.pinId,'primary','afterSelectionUndo'],[null,'muted','afterPinnedLabelUndo']] as const){
    await ui.applyText('Undo my last launcher change','Undo',disk.revision);
    await wait(async()=>{const a=(await ui.state()).oneWindow.personalization;return validRev(a?.appliedRevision)&&a.appliedRevision!==disk.revision&&a.effective.pinnedText===pin&&a.effective.selectionFirst===null;},'personal_undo_not_visible');
    disk=inspect(author);check(disk.workspace===prior.workspace && disk.undo===expectedUndo && disk.effective.selectionFirst===null && disk.effective.pinnedText===pin,'personal_wrong_undo_tip');await ui.adopted(disk.revision,s=>s.selectionFirst===null&&s.pinnedText===pin);await boundary(point);
  }
  check(stock(disk.effective),'personal_stock_not_restored');return {sameWindowExternalAdoption:true,stockRestored:true,normalRelaunch:true,undoCount:2};
}
