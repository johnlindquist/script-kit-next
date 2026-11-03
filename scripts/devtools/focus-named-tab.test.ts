import {expect, test} from 'bun:test';
import {readFileSync} from 'node:fs';
import {runInNewContext} from 'node:vm';

// Run the actual macOS adapter body against a fixture object model. Never launch a browser.
const snapshot = runInNewContext(readFileSync(new URL('../../src/browser_tabs/snapshot.js',import.meta.url),'utf8'));
function focus(app:unknown, query:string, safari:boolean, pid=()=>17, frontPid=pid) {
  return snapshot(app,safari?'com.apple.Safari':'com.google.Chrome','Fixture',safari,pid,query,frontPid);
}
function browser(titles:string[][], safari=false) {
  let activated=0;
  const wins:any[]=[];
  let nextId = 0;
  for(const list of titles){
    const tabs:any=list.map((title,index)=>({title:()=>title,name:()=>title,url:()=>`https://${title.toLowerCase().replaceAll(' ','-')}.example/${index}`}));
    for (const t of tabs) { const id = `${++nextId}`; t.id = () => id; }
    tabs.id=()=>tabs.map((t:any)=>t.id());
    tabs.url=()=>tabs.map((t:any)=>t.url());tabs.title=()=>tabs.map((t:any)=>t.title());tabs.name=()=>tabs.map((t:any)=>t.name());
    let pos=0;
    const windowId = wins.length + 101;
    const win:any={tabs, id:()=>windowId};
    Object.defineProperties(win,{
      activeTabIndex:{get:()=>()=>pos+1,set:(i)=>{if(safari)throw Error('wrong adapter');pos=i-1;}},
      currentTab:{get:()=>()=>tabs[pos],set:t=>{if(!safari)throw Error('wrong adapter');pos=tabs.indexOf(t);}},
      index:{set:i=>{expect(i).toBe(1);wins.splice(wins.indexOf(win),1);wins.unshift(win);}},
    });wins.push(win);
  }
  const windows=Object.assign(()=>wins,{byId:(id:number)=>wins.find(w=>w.id()===id)});
  return {running:()=>true,windows,activate:()=>activated++,get activated(){return activated;}};
}
test.each([false,true])('focus existing tab in another window; safari=%p', safari=>{
  const app=browser([['Mail','Calendar'],['News','Herdr workspace']],safari);
  expect(focus(app,'herdr',safari)).toBe('focused');expect(app.activated).toBe(1);
  expect(app.windows()[0].tabs[app.windows()[0].activeTabIndex()-1].title()).toBe('Herdr workspace');
});
test('ambiguous and missing requests never activate an arbitrary tab',()=>{
  const app=browser([['Herdr help'],['Herdr project']]);
  expect(focus(app,'HERDR',false)).toBe('ambiguous');expect(focus(app,'absent',false)).toBe('not-found');expect(app.activated).toBe(0);
});
test('an exact name beats a substring; a reordered tab fails before activation',()=>{
  const app=browser([['Herdr help','Herdr']]);expect(focus(app,'herdr',false)).toBe('focused');
  const stale=browser([['Home','Herdr']]);stale.windows()[0].tabs.title=()=>['Home','Herdr'];stale.windows()[0].tabs[1].title=()=> 'Different tab';
  expect(focus(stale,'herdr',false)).toBe('changed');expect(stale.activated).toBe(0);
});
test('a name, host or full URL selects the same existing tab; blank words select nothing',()=>{
  for (const query of [' Release ', 'RELEASE.EXAMPLE', 'https://release.example/1']) {
    const app=browser([['Mail','Release']]), win=app.windows()[0];
    expect(focus(app,query,false)).toBe('focused'); expect(win.activeTabIndex()).toBe(2);
  }
  const app=browser([['Mail']]); expect(focus(app,'   ',false)).toBe('not-found'); expect(app.activated).toBe(0);
});
test('not running is not permission to open a browser',()=>{
  expect(focus({running:()=>false},'herdr',false)).toBe('not-running');
});

// Bulk browser reads and the later selection are separate Apple events. A tab can
// change in between even when the old title and URL still look right.
function afterUrls(win:any, change:()=>void) {
  const read = win.tabs.url;
  win.tabs.url = () => { const urls = read(); win.tabs.url = read; change(); return urls; };
}
test('a named Chromium tab replaced by a same-looking tab is not selected',()=>{
  const app=browser([['Mail','Release']]); const win=app.windows()[0];
  afterUrls(win,()=>{ win.tabs[1]={...win.tabs[1],id:()=>'replacement'}; });
  const result=focus(app,'Release',false);
  expect({result,activations:app.activated,selected:win.tabs[win.activeTabIndex()-1].title()})
    .toEqual({result:'changed',activations:0,selected:'Mail'});
});
test.each([false,true])('a named request cannot follow a restarted browser (Safari %p)',safari=>{
  const app=browser([['Mail','Release']],safari), win=app.windows()[0]; let owner=17;
  afterUrls(win,()=>{ owner=18; });
  const result=focus(app,'Release',safari,()=>owner);
  expect({result,activations:app.activated,selected:win.tabs[win.activeTabIndex()-1].title()})
    .toEqual({result:'changed',activations:0,selected:'Mail'});
});
test.each([false,true])('a matching page in a different front window is not named-focus success (Safari %p)',safari=>{
  const app=browser([['Mail'],['Release']],safari), [other,wanted]=app.windows();
  app.activate=()=>{ other.tabs[0]={...wanted.tabs[0],id:()=>'different'}; app.windows().reverse(); return 0; };
  const result=focus(app,'Release',safari);
  expect(app.windows()[0]).toBe(other);
  expect(result).toBe('changed');
});
test('an incomplete browser list cannot make a named tab unique',()=>{
  const app=browser([['Calendar'],['Release']]); app.windows()[0].tabs.url=()=>[];
  expect({result:focus(app,'Release',false),activations:app.activated}).toEqual({result:'changed',activations:0});
});
test('a tab that navigates during selection is not brought to the front',()=>{
  const app=browser([['Mail'],['Release']]), [other,wanted]=app.windows(), tab=wanted.tabs[0];
  afterUrls(wanted,()=>{ const read=tab.title; tab.title=()=>{tab.url=()=>'https://different.example/';return read();}; });
  const result=focus(app,'release',false);
  expect({result,activations:app.activated,originalStillInFront:app.windows()[0]===other})
    .toEqual({result:'changed',activations:0,originalStillInFront:true});
});

function captured(safari:boolean) {
  const app=browser([['Mail','Calendar'],['Notes','Release']], safari);
  const bundle=safari?'com.apple.Safari':'com.google.Chrome';
  let pid=17;
  const call=(target:unknown)=>snapshot(app,bundle,'Fixture',safari,()=>pid,target,()=>pid);
  const target=JSON.parse(call(null)).find((r:any)=>r.title==='Release');
  return {app,call,target,restart:()=>pid++};
}
test.each([false,true])('a staged result follows the selected page through both window and tab reorders (Safari %p)',safari=>{
  const {app,call,target}=captured(safari);
  const wanted=app.windows()[1]; const tab=wanted.tabs[1];
  app.windows().reverse(); wanted.tabs.reverse();
  expect(call(target)).toBe('focused');expect(app.windows()[0]).toBe(wanted);
  expect(wanted.tabs[wanted.activeTabIndex()-1]).toBe(tab);expect(app.activated).toBe(1);
});
test.each([false,true])('stale page, restarted process and missing original window refuse without activation (Safari %p)',safari=>{
  for(const change of ['navigate','restart','close-window']) {
    const {app,call,target,restart}=captured(safari);const win=app.windows()[1];
    if(change==='navigate')win.tabs[1].url=()=>'https://example.invalid/other';
    if(change==='restart')restart();
    if(change==='close-window'){app.windows().shift();app.windows().splice(0,1);}
    expect(call(target)).toBe('changed');expect(app.activated).toBe(0);
  }
});
test('indistinguishable Safari pages refuse; a replacement Chrome tab cannot inherit the same URL and title',()=>{
  const s=captured(true);const tabs=s.app.windows()[1].tabs;tabs.unshift(tabs[1]);
  expect(s.call(s.target)).toBe('ambiguous');expect(s.app.activated).toBe(0);
  const duplicateSnapshot=JSON.parse(s.call(null)).filter((r:any)=>r.title==='Release');
  tabs.shift();
  for(const r of duplicateSnapshot){expect(r.identity).toBeNull();expect(s.call(r)).toBe('changed');}
  expect(s.app.activated).toBe(0);
  const c=captured(false);c.app.windows()[1].tabs[1].id=()=>'new-tab';
  expect(c.call(c.target)).toBe('changed');expect(c.app.activated).toBe(0);
});
test.each(['named','staged'])('an activation request is not proof that the browser became frontmost (%s)',route=>{
  const app=browser([['Mail','Release']]), pid=()=>17, frontPid=()=>91;
  const call=(target:unknown)=>snapshot(app,'com.google.Chrome','Fixture',false,pid,target,frontPid);
  const result=route==='named' ? focus(app,'Release',false,pid,frontPid) : call(JSON.parse(call(null))[1]);
  expect({result,activations:app.activated}).toEqual({result:'unconfirmed',activations:1});
});
test('activation requires actual front-window and page readback',()=>{
  const s=captured(false);s.app.activate=()=>{ s.app.windows().reverse();return 0 };
  expect(s.call(s.target)).toBe('changed');
});
