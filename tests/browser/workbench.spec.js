import {test,expect} from '@playwright/test';
import {readFile,writeFile,mkdir} from 'node:fs/promises';
const fixtures=JSON.parse(await readFile('.cache/native-references.json','utf8'));
const example=name=>fixtures.find(f=>f.name===name).input;
async function open(page){await page.goto('./');await expect(page.locator('#results')).toBeVisible();}
async function download(page,id){const event=page.waitForEvent('download');await page.locator('#'+id).click();return readFile(await(await event).path(),'utf8');}
async function supply(page,text){await page.locator('#request').fill(text);await page.locator('#run').click();}
test('all native responses match actual WASM exactly',async({page})=>{
 await open(page);const actual=await page.evaluate(async fixtures=>{const module=await import('./pkg/rline_web.js');await module.default();return fixtures.map(f=>module.replay(f.input));},fixtures);for(let i=0;i<fixtures.length;i++)expect(actual[i],fixtures[i].name).toBe(fixtures[i].canonical);
});
test('own graph exports exact input and native boundary',async({page})=>{
 await open(page);const request=JSON.parse(example('graph'));request.nodes[0]='My own node';request.edges[0].weight=4;const raw=JSON.stringify(request,null,2);await supply(page,raw);await expect(page.locator('#distances')).toContainText('My own node');const report=JSON.parse(await download(page,'report-download'));expect(report.distances).toEqual([0,4,7,8,null]);expect(await download(page,'submitted-download')).toBe(raw);expect(await download(page,'boundary-download')).toBe(report.boundary_json);
});
test('blocked edge changes reachable topology and SVG',async({page})=>{
 await open(page);await page.locator('[data-example="blocked"]').click();await page.locator('#run').click();await expect(page.locator('#summary')).toContainText('3 components');await expect(page.locator('#graph .blocked')).toHaveCount(1);const report=JSON.parse(await download(page,'report-download'));expect(report.distances[3]).toBeNull();expect(report.reachable).toEqual([0,1,2]);
});
test('parallel edges prevent bridge while loop leaves boundary unchanged',async({page})=>{
 await open(page);await page.locator('[data-example="parallel"]').click();await page.locator('#run').click();await expect(page.locator('#summary')).toContainText('0 bridge edges');const report=JSON.parse(await download(page,'report-download'));expect(report.boundary).toEqual(JSON.parse(fixtures.find(f=>f.name==='graph').canonical).boundary);
});
test('empty selection exposes zero-volume convention',async({page})=>{
 await open(page);await page.locator('[data-example="empty-selection"]').click();await page.locator('#run').click();await expect(page.locator('#boundary')).toContainText('"conductance": 0.0');await expect(page.locator('#notes')).toContainText('not evidence of a good partition');
});
test('invalid input fails closed with source still downloadable',async({page})=>{
 await open(page);await supply(page,example('negative-weight'));await expect(page.locator('#error')).toContainText('WEIGHT');await expect(page.locator('#results')).toBeHidden();expect(await download(page,'original-download')).toBe(example('negative-weight'));await expect(page.locator('#report-download')).toBeDisabled();
});
test('keyboard graph activation changes source through real Rust',async({page})=>{
 await open(page);await page.getByRole('button',{name:'Use node 3 West as source',exact:true}).focus();await page.keyboard.press('Enter');await expect(page.locator('#results')).toBeVisible();await expect(page.locator('#source-node')).toBeFocused();const report=JSON.parse(await download(page,'report-download'));expect(report.graph.source).toBe(3);expect(report.distances).toEqual([6,4,1,0,null]);await expect(page.locator('#graph .source')).toHaveCount(1);await page.locator('#source-node').selectOption('1');await expect(page.locator('#results')).toBeVisible();expect(JSON.parse(await download(page,'report-download')).distances).toEqual([2,0,3,4,null]);
});
test('CRLF file is submitted and exported byte exactly',async({page})=>{
 await open(page);const raw=example('original-crlf');await page.locator('#request-file').setInputFiles({name:'own.json',mimeType:'application/json',buffer:Buffer.from(raw)});expect(await download(page,'original-download')).toBe(raw);await page.locator('#run').click();await expect(page.locator('#results')).toBeVisible();expect(await download(page,'submitted-download')).toBe(raw);
});
test('invalid UTF-8 and oversized files retain draft',async({page})=>{
 await open(page);const before=await page.locator('#request').inputValue();await page.locator('#request-file').setInputFiles({name:'bad.json',mimeType:'application/json',buffer:Buffer.from([0xc3,0x28])});await expect(page.locator('#error')).toContainText('valid UTF-8');expect(await download(page,'original-download')).toBe(before);await page.locator('#request-file').setInputFiles({name:'large.json',mimeType:'application/json',buffer:Buffer.alloc(131073,32)});await expect(page.locator('#error')).toContainText('128 KiB');expect(await download(page,'original-download')).toBe(before);
});
test('markup and controls render only as text',async({page})=>{
 await open(page);await supply(page,example('unicode-control'));await expect(page.locator('#distances')).toContainText('<img src=x onerror=alert(1)>');expect(await page.locator('#results img').count()).toBe(0);expect((await download(page,'report-download'))).toContain('Café');
});
test('edits invalidate visible results and exports',async({page})=>{
 await open(page);await page.locator('#request').fill('{broken');await expect(page.locator('#results')).toBeHidden();await expect(page.locator('#report-download')).toBeDisabled();await page.locator('#reset').click();await expect(page.locator('#request')).toBeFocused();await page.locator('#run').click();await expect(page.locator('#results')).toBeVisible();
});
test('late file cannot overwrite edit or reset',async({page})=>{
 await open(page);await page.evaluate(()=>{File.prototype.arrayBuffer=()=>new Promise(resolve=>window.finishRead=()=>resolve(new TextEncoder().encode('stale').buffer));});await page.locator('#request-file').setInputFiles({name:'held.json',mimeType:'application/json',buffer:Buffer.from('held')});await page.locator('#request').fill('newer draft');await page.evaluate(()=>window.finishRead());expect(await page.locator('#request').inputValue()).toBe('newer draft');await page.locator('#request-file').setInputFiles({name:'held.json',mimeType:'application/json',buffer:Buffer.from('held')});await page.locator('#reset').click();await page.evaluate(()=>window.finishRead());expect(await page.locator('#request').inputValue()).toBe(example('graph'));
});
test('file deadline retains draft and recovers inspection',async({page})=>{
 await open(page);await page.clock.install();const before=await page.locator('#request').inputValue();await page.evaluate(()=>{File.prototype.arrayBuffer=()=>new Promise(()=>{});});await page.locator('#request-file').setInputFiles({name:'held.json',mimeType:'application/json',buffer:Buffer.from('held')});await expect(page.locator('#run')).toBeDisabled();await page.clock.fastForward(5001);await expect(page.locator('#error')).toContainText('five seconds');await expect(page.locator('#run')).toBeEnabled();expect(await page.locator('#request').inputValue()).toBe(before);
});
test('startup deadline retains independent downloads',async({page})=>{
 await page.addInitScript(()=>{window.Worker=class{postMessage(){}terminate(){}};});await page.clock.install();await page.goto('./');await page.locator('#request').fill('retained');await page.clock.fastForward(12001);await expect(page.locator('#retry')).toBeVisible();expect(await download(page,'original-download')).toBe('retained');await expect(page.locator('a[href="./source.zip"]')).toBeVisible();
});
test('compute deadline fences late results',async({page})=>{
 await page.addInitScript(()=>{window.Worker=class{constructor(){window.fakeWorker=this;setTimeout(()=>this.onmessage?.({data:{type:'ready'}}),20);}postMessage(data){this.last=data;}terminate(){}};});await page.clock.install();await page.goto('./');await page.clock.fastForward(21);await expect(page.locator('#run')).toBeDisabled();await page.locator('#request').fill('retained');await page.clock.fastForward(10001);await expect(page.locator('#retry')).toBeVisible();await page.evaluate(()=>window.fakeWorker.onmessage({data:{id:window.fakeWorker.last.id,canonical:'{}'}}));await expect(page.locator('#results')).toBeHidden();expect(await download(page,'original-download')).toBe('retained');
});
test('stale computation cannot restore edited results',async({page})=>{
 await page.addInitScript(()=>{window.Worker=class{constructor(){window.fakeWorker=this;setTimeout(()=>this.onmessage?.({data:{type:'ready'}}),20);}postMessage(data){this.last=data;}terminate(){}};});await page.goto('./');await expect(page.locator('#status')).toContainText('Inspecting');await page.locator('#request').fill('newer');await page.evaluate(()=>window.fakeWorker.onmessage({data:{id:window.fakeWorker.last.id,canonical:JSON.stringify({ok:true})}}));await expect(page.locator('#results')).toBeHidden();await expect(page.locator('#run')).toBeEnabled();
});
test('failed WASM can retry retained input',async({page})=>{
 await page.route('**/*.wasm',route=>route.abort());await page.goto('./');await expect(page.locator('#retry')).toBeVisible();const before=await page.locator('#request').inputValue();await page.unroute('**/*.wasm');await page.locator('#retry').click();await expect(page.locator('#results')).toBeVisible();expect(await page.locator('#request').inputValue()).toBe(before);
});
test('no JavaScript retains example and source download',async({browser,baseURL})=>{
 const context=await browser.newContext({javaScriptEnabled:false});const page=await context.newPage();await page.goto(baseURL);await expect(page.locator('#static-help')).toBeVisible();await expect(page.locator('#request')).toContainText('north-east');expect((await(await page.request.get(new URL('source.zip',baseURL).href)).body()).subarray(0,2).toString()).toBe('PK');await context.close();
});
test('WASM MIME source and license routes work',async({request,baseURL})=>{
 const response=await request.get(new URL('pkg/rline_web_bg.wasm',baseURL).href);expect(response.headers()['content-type']).toContain('application/wasm');expect([...((await response.body()).subarray(0,4))]).toEqual([0,97,115,109]);for(const path of ['BUILD.md','LICENSE','THIRD-PARTY-NOTICES.txt','rust/COPYRIGHT-library.html','source/crates/rgraph-core/src/lib.rs','source/Cargo.toml','source/web/adapter/Cargo.lock'])expect((await request.get(new URL(path,baseURL).href)).ok(),path).toBeTruthy();expect(await(await request.get(new URL('LICENSE',baseURL).href)).text()).toContain('MIT License');expect(await(await request.get(new URL('THIRD-PARTY-NOTICES.txt',baseURL).href)).text()).toContain('thiserror');
});
test('record representative timings without performance certification',async({page,baseURL})=>{
 const began=performance.now();await open(page);const startup=performance.now()-began;const result=await page.evaluate(async raw=>{const module=await import('./pkg/rline_web.js');await module.default();const samples=[];for(let i=0;i<32;i++){const before=performance.now();JSON.parse(module.replay(raw));samples.push(performance.now()-before);}samples.sort((a,b)=>a-b);return {user_agent:navigator.userAgent,hardware_concurrency:navigator.hardwareConcurrency,compute_ms:{min:samples[0],median:samples[16],max:samples[31]},warm_repetitions:32};},example('graph'));expect(result.compute_ms.max).toBeLessThan(10000);await mkdir('.cache',{recursive:true});await writeFile('.cache/timings.json',JSON.stringify({schema:'rline.browser-timings.v1',base_url:baseURL,startup_navigation_to_first_result_ms:startup,graph:{nodes:5,edges:4},note:'One browser/context and authored graph; warm compute includes JSON report parsing. No throughput, downstream or hardware-independent claim.',...result},null,2));
});
for(const width of [1440,390])test('readable graph results at '+width,async({page})=>{
 await page.setViewportSize({width,height:1000});await open(page);await expect(page.locator('#graph .graph-node')).toHaveCount(5);expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBeTruthy();await mkdir('.cache/review',{recursive:true});await page.screenshot({path:`.cache/review/results-${width}.png`,fullPage:true});
});
