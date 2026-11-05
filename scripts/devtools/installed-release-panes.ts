// Normal installed Kit is the consumer. Only Herdr's host API supplies synthetic terminals.
import {createHash,randomUUID} from 'node:crypto';
import {lstatSync,readFileSync,realpathSync} from 'node:fs';
import {join} from 'node:path';
import {check,type Json} from './lib/installed-release-contract';
import type {InstalledKit} from './lib/installed-release-host';
import {personalControls,type Author} from './installed-release-personal';
import {createPaneFixture,paneCanary,paneFixtureSpace,type PanePhase} from './lib/installed-release-pane-fixture';
const privateTitleMarker='ploverhill';
const view={id:'qa-shared-panes',title:'Ploverhill metadata list',limit:2,list:{source:'herdr.paneMetadata',filter:'needsAttention',sort:'attentionFirst',detail:'status',enter:'none'}};
const hash=(data:string|Buffer)=>createHash('sha256').update(data).digest('hex');
const checkedModule=(root:string,relative:string,expected:string)=>{const name=join(root,relative),st=lstatSync(name);check(realpathSync(name)===name&&st.isFile()&&st.nlink===1&&st.uid===process.getuid!()&&st.size<512000,'unexpected_installed_pane_file');const b=readFileSync(name);check(hash(b)===expected,'installed_pane_module_mismatch');return b;};
// These are the release's embedded files, including the dynamically imported reader. Do not
// run edited workspace code. The app bundle/manifest and Bun are verified by the outer host.
export function checkedPaneFiles(root:string){
  checkedModule(root,'bin/scriptkit','6286a9746e5f76771cc139ea3cb659e4d61228afe624e0ebdee6685affed2919');
  checkedModule(root,'bin/herdr-panes.mjs','6071ad841e4db42a38d97fe9704ccf150fd2f8f7aa3a801592b047259a9dc071');
  checkedModule(root,'bin/herdr-pane-read.mjs','0df3990539ec91dccbeeee4175b7f5d973150dd67263ea83797b8ce12035c00e');
  return checkedModule(root,'share/herdr-pane-metadata/main.js','a5ed804c0b29f2f4727386d21177ba38b47dab22f34111d88e94d5780c8519c3');
}
async function until(f:()=>Promise<boolean>,why:string){const stop=performance.now()+13000;do{if(await f())return;await Bun.sleep(100);}while(performance.now()<stop);check(false,why);}
export type Scene={revision:string;pair:string;phase:(p:PanePhase)=>void;withdraw:()=>void};
export async function readInstalledPaneView(kit:InstalledKit,scene:Scene){
  const ui=await personalControls(kit);await until(async()=>(await ui.state()).oneWindow?.personalization?.appliedRevision===scene.revision,'pane_recipe_not_adopted');
  const words='view '+view.id,openId='personal/view/'+scene.revision+'/'+view.id;
  await ui.filter(words);await until(async()=>{const s=await ui.state();return s.inputValue===words&&s.oneWindow?.enter?.id===openId&&s.oneWindow.enterArmed===false;},'declared_pane_view_not_current');
  const menu=(await ui.elements()).elements as Json[];
  check(!JSON.stringify(menu).toLowerCase().includes(privateTitleMarker) && menu.some(e=>e.role==='row'&&e.selected===true&&e.selectable===true&&e.text==null&&e.content?.text?.rawContentReturned===false),'private_view_title_or_identity_exposed');
  const beforeOpen=await ui.state();check(beforeOpen.inputValue===words&&beforeOpen.oneWindow?.enter?.id===openId&&!beforeOpen.oneWindow.enterArmed,'pane_view_selection_changed');
  await ui.key('enter');
  let mount:string|undefined,selected:string|undefined;
  const suffix=(n:number,replaced=false)=>'h-'+hash([scene.pair,'tab'+n,'p'+n,'t'+n,4100+n+(replaced&&n===9?1000:0),99000+n+(replaced&&n===9?1000:0)].join(':'));
  async function frame(order:number[],kind:'ready'|'empty'|'refused',selectedN?:number,replaced=false){
    return until(async()=>{
      const s=await ui.state(),w=s.oneWindow,p=w?.personalList;
      if(!s.windowVisible||s.promptType!=='none'||w?.view!=='personal-list'||p?.source!=='herdr.paneMetadata'||!p.readOnly||p.limit!==2||w.enter.label!=='No action')return false;
      if(kind==='refused'?!['stale','unavailable','unverified'].includes(p.state):p.state!==kind)return false;
      if(p.visibleCount!==order.length||p.sourceCount!==order.length||p.truncated!==(kind==='ready'))return false;
      const elements=(await ui.elements()).elements as Json[];check(!JSON.stringify(elements).includes(paneCanary)&&!JSON.stringify(elements).toLowerCase().includes(privateTitleMarker),'pane_private_data_disclosed');
      const rows=elements.filter(e=>e.semanticId?.startsWith('personal-item:'));if(rows.length!==order.length)return false;
      if(kind!=='ready')return p.selectedElementId===null&&elements.some(e=>e.semanticId==='personal-list:status'&&!e.selectable&&e.statusKind===p.state&&e.actionDisabled==='personal_list_no_target');
      const parts=rows.map(r=>String(r.semanticId).split(':'));if(!parts.every((v,i)=>v.length===3&&v[0]==='personal-item'&&v[1]===parts[0][1]&&v[2]===suffix(order[i],replaced)))return false;
      if(!mount)mount=parts[0][1];check(mount===parts[0][1],'pane_view_remounted');
      if(!rows.every((r,i)=>r.selectable===true&&r.actionDisabled==='personal_list_read_only'&&r.text==null&&r.value==null&&r.content?.text?.rawContentReturned===false&&r.content.text.charLength===('Pane '+order[i]).length))return false;
      const chosen=rows.filter(r=>r.selected);if(selectedN===undefined)return chosen.length===0&&p.selectedElementId===null;
      if(chosen.length!==1||p.selectedElementId!==chosen[0].semanticId||!p.selectedElementId.endsWith(':'+suffix(selectedN,replaced)))return false;
      selected=p.selectedElementId;return true;
    },'installed_pane_state_'+kind);
  }
  await frame([9,2],'ready',9);const before=selected!;await ui.selectPane(before);await ui.key('enter');await frame([9,2],'ready',9);
  scene.phase('reordered');await frame([2,9],'ready',9);check(selected===before,'pane_refresh_retargeted_selection');
  scene.phase('replaced');await frame([9,2],'ready',undefined,true);
  scene.phase('empty');await frame([],'empty');scene.phase('initial');await frame([9,2],'ready');
  scene.withdraw();await frame([],'refused');await ui.key('escape');await until(async()=>(await ui.state()).inputValue===words,'pane_filter_not_restored');
  return {consumer:'normalInstalledKit',source:'released metadata mod with a synthetic isolated Herdr host',sameMountedView:true,limitedTo:2,privateListTitleProtected:true,selectionFollowedIdentity:true,reusedAddressLostSelection:true,readOnlyReturn:true,verifiedEmpty:true,withdrawalClearedRows:true,realHerdrDesktop:false,focusOrZoomTested:false};
}
export async function withPairedPaneFixture(scratch:string,bytes:Buffer,call:(args:string[])=>Json,author:Author,run:(s:Scene)=>Promise<Json>){
  check(!['SCRIPT_KIT_AGENT_CREDENTIAL','SCRIPT_KIT_MCP_TOKEN','SCRIPT_KIT_MCP_ENDPOINT','SCRIPT_KIT_MCP_SERVER_JSON'].some(k=>Object.hasOwn(process.env,k)),'operator_only_pane_fixture');
  check(call(['status']).state==='notPaired','prior_pane_grant_exists');const initial=author({op:'inspect'}).state;
  const initialViews=initial.views??[]; // the real store omits an empty views array
  check(initial.enabled===true&&initial.undo===null&&Array.isArray(initialViews)&&!initialViews.some((x:Json)=>x.id===view.id),'pane_fixture_prior_preference');
  const peer=createPaneFixture(scratch,bytes);let offer:Json|undefined,pair:Json|undefined, saved:Json|undefined;const requestId=randomUUID();
  const withdraw=()=>{check(offer?.state==='offered'&&call(['withdraw',peer.home,offer.offerId]).state==='withdrawn','pane_withdrawal_unverified');};
  try{
    offer=call(['offer',peer.home,paneFixtureSpace]);check(offer.state==='offered'&&offer.workspace===initial.workspace,'pane_offer_not_bound');
    const marker=await peer.listen();pair=call(['pair',marker,paneFixtureSpace]);check(pair.state==='paired'&&pair.workspace===initial.workspace&&/^[0-9a-f]{32}$/.test(pair.pairId),'pane_pairing_unverified');
    const inbox=join(peer.home,'kit-pane-share-v1/in/request.json'),prior=hash(readFileSync(inbox));
    saved=author({op:'apply',request:{workspace:initial.workspace,expectedRevision:initial.revision,requestId,change:{op:'putView',view}}}).result;
    check(saved && saved.requestId===requestId && /^[a-f0-9]{64}$/.test(saved.appliedRevision) && saved.state?.undo===requestId && !saved.replayed && hash(readFileSync(inbox))===prior,'saving_recipe_read_the_source');
    return await run({revision:saved.appliedRevision,pair:pair.pairId,phase:peer.phase,withdraw});
  }finally{
    // Never undo somebody else's newer write or unpair a replacement generation. Always
    // release our private process and temp files, even when another cleanup check fails.
    try{if(saved){const s=author({op:'inspect'}).state;check(s.workspace===initial.workspace&&s.revision===saved.appliedRevision&&s.undo===requestId,'pane_recipe_cleanup_identity_changed');const restored=author({op:'apply',request:{workspace:s.workspace,expectedRevision:s.revision,requestId:randomUUID(),change:{op:'undo',of:requestId}}}).result.state;check(JSON.stringify(restored.views??[])===JSON.stringify(initialViews)&&restored.undo===null,'pane_recipe_cleanup_unverified');}}
    finally{try{if(pair?.state==='paired')check(call(['unpair',pair.pairId]).state==='unpaired','pane_unpair_unverified');}finally{try{if(offer?.state==='offered')withdraw();}finally{await peer.close();}}}
  }
}
