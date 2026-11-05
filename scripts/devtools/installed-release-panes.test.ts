// Real packaged mod and real nonce-bound worker; controlled Herdr host API and no Kit window.
import {test,expect} from 'bun:test';
import {mkdtempSync,mkdirSync,readFileSync,realpathSync,rmSync} from 'node:fs';
import {join} from 'node:path';import {tmpdir} from 'node:os';
import {createPaneFixture,paneCanary,paneFixtureSpace} from './lib/installed-release-pane-fixture';
import {paneCommand} from '../herdr-panes.mjs';
import {readInstalledPaneView,withPairedPaneFixture} from './installed-release-panes';
import type {InstalledKit} from './lib/installed-release-host';

test('a pre-existing pairing is never replaced by the pane fixture',async()=>{
  const calls:string[][]=[];let author=false,run=false;
  await expect(withPairedPaneFixture('/unused',Buffer.alloc(0),args=>{calls.push(args);return {state:'paired'}},()=>{author=true;throw Error('unexpected')},async()=>{run=true;throw Error('unexpected')})).rejects.toThrow('prior_pane_grant_exists');
  expect(calls).toEqual([['status']]);expect(author||run).toBe(false);
});

test('a cleartext authored list row refuses the installed journey before Return',async()=>{
  const rev='a'.repeat(64),pid=123;let input='',submitted=false;
  const kit:InstalledKit={pid,scratchPath:'unused',kitPath:'unused',send:async()=>{throw Error('unexpected send')},request:async(q:any)=>{
    const reply=(type:string,x:any)=>({type,requestId:q.requestId,...x});
    switch(q.type){
      case 'listAutomationWindows':return reply('automationWindowListResult',{windows:[{id:'main',pid,generation:1}]});
      case 'getState':return reply('stateResult',{windowVisible:true,promptType:'none',permissions:{bundleIdentifier:'com.scriptkit.app'},inputValue:input,oneWindow:{personalization:{appliedRevision:rev},enterArmed:false,enter:{id:'personal/view/'+rev+'/qa-shared-panes'}}});
      case 'batch':input=q.commands[0].text;return reply('batchResult',{success:true,results:[{command:'setInput',success:true}]});
      case 'getElements':return reply('elementsResult',{elements:[{role:'row',selectable:true,selected:true,text:'Ploverhill metadata list'}]});
      case 'simulateKey':submitted=true;throw Error('unexpected Return');
      default:throw Error('unreviewed fixture request');
    }
  }};
  await expect(readInstalledPaneView(kit,{pair:'b'.repeat(32),revision:rev,withdraw:()=>{},phase:()=>{}})).rejects.toThrow('private_view_title_or_identity_exposed');
  expect(submitted).toBe(false);
});

test.skipIf(process.platform!=='darwin')('paired mod returns fresh ordered metadata only; empty and withdrawn are distinct',async()=>{
  const parent=mkdtempSync(join(realpathSync(tmpdir()),'kit-release-panes-')),kit=join(parent,'kit');mkdirSync(kit,{mode:0o700});mkdirSync(join(kit,'db'),{mode:0o700});
  const fixture=createPaneFixture(parent,readFileSync(new URL('../../kit-init/integrations/herdr-pane-metadata/main.js',import.meta.url)));
  const call=(...args:string[])=>paneCommand(args,kit);let offer:any,paired:any;
  try{
    expect(call('status').state).toBe('notPaired');offer=call('offer',fixture.home,paneFixtureSpace);expect(offer.state).toBe('offered');
    const marker=await fixture.listen();paired=call('pair',marker,paneFixtureSpace);expect(paired.state).toBe('paired');expect(paired.workspace).toBe(offer.workspace);
    const query=()=>call('list','needsAttention','attentionFirst','2');const view=()=>{const r=query();if(r.state!=='ready'||!Array.isArray(r.rows)||typeof r.more!=='boolean'||typeof r.pairId!=='string')throw Error('ready metadata page missing');return {...r,rows:r.rows,more:r.more,pairId:r.pairId}};const first=view();expect(first.state).toBe('ready');expect(first.more).toBe(true);expect(first.rows.map((r:any)=>r.title)).toEqual(['Pane 9','Pane 2']);
    expect(first.rows[0].focused).toBe(true);expect(JSON.stringify(first)).not.toContain(paneCanary);expect(JSON.stringify(first)).not.toContain('other-space');
    fixture.phase('reordered');const next=view();expect(next.pairId).toBe(paired.pairId);expect(next.rows.map((r:any)=>r.title)).toEqual(['Pane 2','Pane 9']);
    fixture.phase('replaced');const replaced=view().rows[0];expect(replaced.terminal).toBe(first.rows[0].terminal);expect(replaced.pid).not.toBe(first.rows[0].pid);expect(replaced.createdMs).not.toBe(first.rows[0].createdMs);
    fixture.phase('empty');expect(view()).toMatchObject({state:'ready',more:false,rows:[]});
    expect(call('withdraw',fixture.home,offer.offerId).state).toBe('withdrawn');const gone=query();expect(gone.state).not.toBe('ready');expect(gone.rows).toBeUndefined();
    expect(call('unpair',paired.pairId).state).toBe('unpaired');expect(call('status').state).toBe('notPaired');
  }finally{try{if(paired?.pairId&&call('status').pairId===paired.pairId)call('unpair',paired.pairId);if(offer?.offerId)call('withdraw',fixture.home,offer.offerId);}finally{await fixture.close();rmSync(parent,{recursive:true,force:false});}}
},15000);
