import {expect, test} from 'bun:test';
import {readFileSync} from 'node:fs';
import {runInNewContext} from 'node:vm';

// Run the actual macOS adapter body against a fixture object model. Never launch a browser.
const focus = runInNewContext(readFileSync(new URL('../../src/browser_tabs/focus_named.js',import.meta.url),'utf8'));
function browser(titles:string[][], safari=false) {
  let activated=0;
  const wins:any[]=[];
  for(const list of titles){
    const tabs:any=list.map((title,index)=>({title:()=>title,name:()=>title,url:()=>`https://${title.toLowerCase().replaceAll(' ','-')}.example/${index}`}));
    tabs.url=()=>tabs.map((t:any)=>t.url());tabs.title=()=>list;tabs.name=()=>list;
    let pos=0;
    const win:any={tabs};
    Object.defineProperties(win,{
      activeTabIndex:{get:()=>()=>pos+1,set:(i)=>{if(safari)throw Error('wrong adapter');pos=i-1;}},
      currentTab:{get:()=>()=>tabs[pos],set:t=>{if(!safari)throw Error('wrong adapter');pos=tabs.indexOf(t);}},
      index:{set:i=>{expect(i).toBe(1);wins.splice(wins.indexOf(win),1);wins.unshift(win);}},
    });wins.push(win);
  }
  return {running:()=>true,windows:()=>wins,activate:()=>activated++,get activated(){return activated;}};
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
  const stale=browser([['Home','Herdr']]);stale.windows()[0].tabs[1].title=()=> 'Different tab';
  expect(focus(stale,'herdr',false)).toBe('changed');expect(stale.activated).toBe(0);
});
test('not running is not permission to open a browser',()=>{
  expect(focus({running:()=>false},'herdr',false)).toBe('not-running');
});
