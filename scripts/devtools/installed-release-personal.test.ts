// Verifier behavior only. This synthetic transport is not evidence of installation or live adoption.
import {describe,it,expect} from 'bun:test';
import {preparePersonal,recoverPersonal,boundaries} from './installed-release-personal';
import type {InstalledKit} from './lib/installed-release-host';
import {personalChoiceTarget} from './lib/installed-release-contract';
type J=Record<string,any>;
const stock={pinnedText:'muted',pinnedWeight:'regular',selectionFirst:null,clipboardFirst:null,homeMode:'standard'};
function fixture(){
  let rev='stock',undo:string|null=null,workspace='1'.repeat(64),words='',pinId='',seq=0,enters=0;
  let rules:J={...stock};const next=()=>rev=(++seq).toString(16).padStart(64,'0');
  const state=()=>({workspace,revision:rev,enabled:true,undo,effective:{...rules},declared:{...rules}});
  const author=(q:J):J=>{
    if(q.op==='inspect')return {scope:'stored',safeMode:false,state:state()};
    if(q.op==='why')return {origin:'launcher',requestId:pinId};
    if(q.op==='preview')return {scope:'preview',wouldStore:{...state(),effective:{...rules,selectionFirst:'note'}}};
    expect(q.request.workspace).toBe(workspace);expect(q.request.expectedRevision).toBe(rev);
    undo=q.request.requestId;rules.selectionFirst='note';next();return {scope:'stored',result:{requestId:undo,replayed:false,state:state(),appliedRevision:rev}};
  };
  const instance=(pid:number,generation:number):InstalledKit=>{let selected='';return {pid,kitPath:'unused',scratchPath:'unused',async send(){throw Error('unreviewed fixture action')},async request(q:J){
    const r=(type:string,x:J)=>({type,...x,requestId:q.requestId});
    if(q.type==='listAutomationWindows')return r('automationWindowListResult',{windows:[{id:'main',pid,generation}]});
    expect(q.target).toEqual({type:'instance',id:'main',generation});
    const un=words==='Undo my last launcher change', target=personalChoiceTarget(un?'undo':'pin',rev), focused=!!target&&selected===target.id;
    if(q.type==='getState')return r('stateResult',{permissions:{bundleIdentifier:'com.scriptkit.app'},promptType:'none',inputValue:words,windowVisible:true,oneWindow:{personalization:{scope:'runningWindow',appliedRevision:rev,effective:rules},enterArmed:false,enter:focused?{id:target!.key,label:un?'Undo':'Apply'}:{id:'some-neighbor',label:'Open'}}});
    if(q.type==='batch'){const cmd=q.commands[0];if(cmd.type==='setInput'){words=cmd.text;selected='';}else{expect(cmd).toEqual({type:'selectBySemanticId',semanticId:target!.id,submit:false});selected=cmd.semanticId;}return r('batchResult',{success:true,results:[{command:cmd.type,success:true}]})}
    if(q.type==='getElements')return r('elementsResult',{semanticSurface:'scriptList',elements:[{role:'row',selected:!focused,selectable:true,text:'Neighbor'},{role:'row',semanticId:target!.id,selected:focused,selectable:true,text:target!.label}]});
    expect(focused).toBe(true);expect(q.key).toBe('enter');enters++;if(un && rules.selectionFirst){rules.selectionFirst=null;undo=pinId}else if(un){rules.pinnedText='muted';undo=null}else{rules.pinnedText='primary';undo=pinId='00000000-0000-4000-8000-000000000001'}next();words='';return r('externalCommandResult',{ok:true,command:'simulateKey'});
  }};};
  return {author,instance,enters:()=>enters};
}
describe('installed personalization verifier — controlled requests and responses only',()=>{
  it('checks originals across two processes and two separate Undos',async()=>{
    const x=fixture(), seen:string[]=[];const check=async(where:typeof boundaries[number])=>{seen.push(where)};
    await check('journeysProcessExited');const saved=await preparePersonal(x.instance(31,1),x.author,check);await check('beforeRelaunch');
    const r=await recoverPersonal(x.instance(32,2),x.author,saved,check);await check('afterFinalExit');expect(seen).toEqual([...boundaries]);expect(r.stockRestored).toBe(true);expect(x.enters()).toBe(3);
  });
  it('does not send another Undo after original data fails its intermediate readback',async()=>{
    const x=fixture();const saved=await preparePersonal(x.instance(31,1),x.author,async()=>{});
    await expect(recoverPersonal(x.instance(32,2),x.author,saved,async(where)=>{if(where==='afterSelectionUndo')throw Error('original changed')})).rejects.toThrow('original changed');expect(x.enters()).toBe(2);
  });
});
