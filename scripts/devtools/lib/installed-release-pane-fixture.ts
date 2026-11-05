// A sacrificial peer for exercising the released metadata mod. The workspaces, panes and
// terminals supplied by the Herdr host API are synthetic. Never start Herdr or contact a core.
import {createHash} from 'node:crypto';
import {existsSync, lstatSync, mkdirSync, mkdtempSync, readFileSync, realpathSync, rmSync, writeFileSync} from 'node:fs';
import {dirname,join} from 'node:path';
const reviewedMod='a5ed804c0b29f2f4727386d21177ba38b47dab22f34111d88e94d5780c8519c3';
export const paneFixtureSpace='kit-qa-space';
export const paneCanary='PRIVATE_QA_TERMINAL_CONTENT';
function need(ok:unknown, message:string): asserts ok { if(!ok)throw Error(message); }
export type PanePhase='initial'|'reordered'|'replaced'|'empty';

// Only the host bindings and synthetic terminal data are fixtures. The separately hash-verified JS file
// handles enrollment, challenge validation, redaction, sorting, limits and fresh replies.
const host=String.raw`
import fs from 'node:fs';import path from 'node:path';import {pathToFileURL} from 'node:url';
const base=fs.realpathSync(process.cwd()),root=path.join(base,'herdr'),space='kit-qa-space';
const key='PRIVATE_QA_TERMINAL_CONTENT',scope='kit-pane-share-v1/',perm='fixture authority refused';
const ensure=(ok)=>{if(!ok)throw Error(perm)};
const names=(s)=>s===scope+'grant.json'||s===scope+'in/request.json'||/^kit-pane-share-v1\/out\/[a-f0-9]{32}-(?:listening|[a-f0-9]{32})\.json$/.test(s);
function file(s){ensure(typeof s==='string' && s.startsWith(root+'/') && names(s.slice(root.length+1)));return s}
const readTextFile=async(s)=>{const p=file(s);try{const st=fs.lstatSync(p);ensure(st.isFile()&&st.nlink===1&&st.size<=32768&&fs.realpathSync(p)===p);return fs.readFileSync(p,'utf8')}catch(e){if(e.code==='ENOENT')return null;throw e}};
const writeTextFile=async(s,b,previous)=>{ensure(s.startsWith(root+'/'+scope+'out/')&&previous===null&&typeof b==='string'&&b.length<=32768);fs.writeFileSync(file(s),b,{mode:0o600,flag:'wx'})};
const watchFolder=async(s)=>{ensure(s===root+'/'+scope+'in/');await Bun.sleep(20)};
function data(){const phase=fs.readFileSync(path.join(base,'phase'),'utf8');ensure(['initial','reordered','replaced','empty'].includes(phase));const rows=phase==='empty'?[]:[[9,phase==='reordered'?'done':'blocked'],[2,phase==='reordered'?'blocked':'done'],[12,'done'],[3,'idle']];return [...rows,[99,'blocked']];}
const spaces=()=>[space,'other-space'].map(id=>({id,machine:null,title:key,tabs:data().filter(([n])=>id===space?n!==99:n===99).map(([n])=>({id:'tab'+n}))}));
const panes=()=>data().map(([n,status])=>({pane:'p'+n,tab:'tab'+n,terminal:'t'+n,status,focused:n===9,zoomed:false,title:key,cwd:key}));
const terminals=()=>{const replaced=fs.readFileSync(path.join(base,'phase'),'utf8')==='replaced';return data().map(([n])=>({address:'t'+n,pid:4100+n+(replaced&&n===9?1000:0),created_ms:99000+n+(replaced&&n===9?1000:0),state:'live',command:key,report:key,screen:key}));};
Bun.plugin({name:'qa-fixed-herdr-host',setup(b){b.module('gpui-kit',()=>({loader:'object',exports:{View:class {},div:()=>({child(){return this}})}}));b.module('herdr',()=>({loader:'object',exports:{appHome:()=>root,machines:()=>[{name:null,layout:'ready'}],panes,spaces,terminals,readTextFile,writeTextFile,watchFolder}}));}});
let task;try{const {default:Mod}=await import(pathToFileURL(path.join(base,'provider.mjs')).href);new Mod().init({},{spawn:fn=>{ensure(!task);task=fn()}});ensure(task);await task;}catch{process.exitCode=71}
`;

export function createPaneFixture(parent:string, provider:Buffer){
  need(process.platform==='darwin','pane_fixture_needs_macos');
  need(createHash('sha256').update(provider).digest('hex')===reviewedMod,'pane_provider_source_mismatch');
  need(realpathSync(parent)===parent && lstatSync(parent).isDirectory(),'pane_fixture_parent_indirect');
  const root=mkdtempSync(join(parent,'pane-peer-')),id=lstatSync(root,{bigint:true});const home=join(root,'herdr');
  mkdirSync(home,{mode:0o700});writeFileSync(join(root,'provider.mjs'),provider,{flag:'wx',mode:0o600});writeFileSync(join(root,'phase'),'initial',{flag:'wx',mode:0o600});
  let child:ReturnType<typeof Bun.spawn>|null=null;
  const exchange=join(home,'kit-pane-share-v1'), phase=(value:PanePhase)=>{need(['initial','reordered','replaced','empty'].includes(value),'bad_pane_fixture_phase');writeFileSync(join(root,'phase'),value);};
  return {home,phase,
    async listen(){
      need(!child && realpathSync(exchange)===exchange,'pane_offer_required');const out=join(exchange,'out');need(realpathSync(out)===out,'pane_outbox_indirect');
      const q=JSON.stringify,bun=realpathSync(process.execPath), parents=new Set<string>();
      for(const start of [root,dirname(bun)])for(let d=start;d!=='/';d=dirname(d))parents.add(d);
      // The same bounded macOS runtime files the real nonce-reader needs. No network,
      // child processes, WindowServer or user-home contents are granted to this peer.
      const policy='(version 1)(deny default)(allow sysctl-read)(allow process-info* (target self))'+
        '(allow process-exec (literal '+q(bun)+'))(allow file-map-executable (literal '+q(bun)+') (subpath "/System/Library") (subpath "/usr/lib"))'+
        '(allow file-read* (literal "/") (literal '+q(bun)+') (subpath '+q(root)+') (subpath "/System/Library") (subpath "/usr/lib") (subpath "/usr/share/icu") (literal "/dev/urandom"))'+
        '(allow file-read* file-write-data (literal "/dev/null") (literal "/dev/zero"))'+
        '(allow file-read-metadata '+[...parents].map(p=>'(literal '+q(p)+')').join(' ')+')'+
        '(allow file-write-create file-write-data (subpath '+q(out)+'))';
      child=Bun.spawn(['/usr/bin/sandbox-exec','-p',policy,bun,'--config=/dev/null','--no-env-file','--no-install','--no-macros','-e',host],{cwd:root,env:{HOME:root,TMPDIR:root,PATH:'/usr/bin:/bin',LANG:'C',BUN_RUNTIME_TRANSPILER_CACHE_PATH:'0'},stdin:'ignore',stdout:'ignore',stderr:'ignore'});
      const grant=JSON.parse(readFileSync(join(exchange,'grant.json'),'utf8'));need(/^[a-f0-9]{32}$/.test(grant.source)&&grant.space===paneFixtureSpace,'unexpected_pane_grant');
      const marker=join(out,grant.source+'-listening.json');const end=performance.now()+4500;
      while(!existsSync(marker)&&child.exitCode===null&&child.signalCode===null&&performance.now()<end)await Bun.sleep(20);
      need(existsSync(marker)&&child.exitCode===null&&realpathSync(marker)===marker,'pane_fixture_not_listening');return marker;
    },
    async close(){if(child){if(child.exitCode===null&&child.signalCode===null)child.kill();await Promise.race([child.exited,Bun.sleep(2600)]);need(child.exitCode!==null||child.signalCode!==null,'pane_fixture_cleanup_unconfirmed');}
      const now=lstatSync(root,{bigint:true});need(now.isDirectory()&&now.dev===id.dev&&now.ino===id.ino&&realpathSync(root)===root,'pane_fixture_identity_changed');rmSync(root,{recursive:true,force:false});},
  };
}
