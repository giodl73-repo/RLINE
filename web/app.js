const $=id=>document.getElementById(id),examples=JSON.parse($('examples-data').textContent);$('static-help').hidden=true;
let focusSource=false;
const outputs=['report-download','submitted-download','boundary-download'];
let worker,epoch=0,revision=0,serial=0,active=null,ready=false,busy=false,reading=false,startupTimer,computeTimer,fileTimer,fileToken=0,original=null,result=null,canonical=null;
function controls(){$('run').disabled=!ready||busy||reading;for(const id of outputs)$(id).disabled=!result?.ok;$('original-download').disabled=!$('request').value&&original===null;}
function clear(){canonical=result=null;$('results').hidden=true;$('error').hidden=true;controls();}
function cancelFile(){fileToken++;reading=false;clearTimeout(fileTimer);}
function invalidate(){revision++;clear();if(ready)$('status').textContent=busy?'Inputs changed; waiting for previous work.':'Inputs changed. Inspect again.';}
function edited(){focusSource=false;cancelFile();original=null;invalidate();}
function diagnostic(detail){$('error').hidden=false;$('error-message').textContent=detail.message;$('diagnostic').textContent=JSON.stringify(detail,null,2);}
function engineError(message){epoch++;worker?.terminate();worker=null;ready=busy=false;active=null;clearTimeout(startupTimer);clearTimeout(computeTimer);clear();diagnostic({message});$('retry').hidden=false;$('status').textContent='Engine unavailable. Input and source downloads remain available.';controls();}
function replay(){
 if(!ready||busy||reading)return;clear();const text=$('request').value;if(new TextEncoder().encode(text).length>131072){diagnostic({message:'Request exceeds 128 KiB UTF-8.'});return;}
 busy=true;active={id:++serial,revision};const id=active.id,generation=epoch;controls();$('status').textContent='Inspecting in Rust…';computeTimer=setTimeout(()=>{if(generation===epoch&&active?.id===id)engineError('Inspect exceeded ten seconds. Retry with retained input.');},10000);
 try{worker.postMessage({type:'replay',id,request:original??text});}catch{engineError('The worker could not accept this request.');}
}
function render(){
 $('summary').textContent=`${result.graph.nodes.length} nodes · ${result.graph.edges.length} declared edges · ${result.components.length} components · ${result.bridge_edge_ids.length} bridge edges`;
 $('distances').textContent=JSON.stringify(result.graph.nodes.map((label,i)=>({node:i,label,distance:result.distances[i]??'unreachable',selected:result.graph.selected?.includes(i)??false})),null,2);
 $('topology').textContent=JSON.stringify({reachable:result.reachable,components:result.components,selected_components:result.selected_components,bridge_edge_ids:result.bridge_edge_ids,articulation_points:result.articulation_points},null,2);
 $('boundary').textContent=result.boundary_json;$('notes').replaceChildren();for(const note of result.notes){const item=document.createElement('li');item.textContent=note;$('notes').append(item);}
 $('native').textContent=JSON.stringify({predecessors:result.predecessors,native_path_counts:result.native_path_counts,visit_order:result.visit_order,bridge_records:result.bridge_records},null,2);
 $('edges').textContent=JSON.stringify(result.graph.edges,null,2);$('source-node').replaceChildren();result.graph.nodes.forEach((label,i)=>{const option=document.createElement('option');option.value=String(i);option.textContent=i+': '+label;$('source-node').append(option);});$('source-node').value=String(result.graph.source);draw();$('results').hidden=false;if(focusSource){focusSource=false;$('source-node').focus();}
}
function chooseSource(i){try{const request=JSON.parse($('request').value);request.source=i;$('request').value=JSON.stringify(request,null,2);edited();focusSource=true;replay();}catch{diagnostic({message:'Edit the graph as valid JSON before changing source.'});}}
function draw(){
 const svg=$('graph'),ns='http://www.w3.org/2000/svg';svg.replaceChildren();const n=result.graph.nodes.length,points=result.graph.nodes.map((_,i)=>({x:360+165*Math.cos(2*Math.PI*i/n-Math.PI/2),y:240+165*Math.sin(2*Math.PI*i/n-Math.PI/2)}));
 function element(tag,attrs,text){const e=document.createElementNS(ns,tag);for(const [key,value] of Object.entries(attrs))e.setAttribute(key,value);if(text!==undefined)e.textContent=text;return e;}
 for(const edge of result.graph.edges){const a=points[edge.from],b=points[edge.to],blocked=result.graph.blocked_edges?.includes(edge.id),bridge=result.bridge_edge_ids.includes(edge.id);const line=edge.from===edge.to?element('circle',{cx:a.x,cy:a.y-18,r:23,fill:'none'}):element('line',{x1:a.x,y1:a.y,x2:b.x,y2:b.y});line.setAttribute('class',blocked?'blocked':bridge?'bridge':'edge');line.append(element('title',{},`${edge.id}: ${edge.from} ↔ ${edge.to}, cost ${edge.weight}${blocked?' (blocked)':''}`));svg.append(line);}
 result.graph.nodes.forEach((label,i)=>{const p=points[i],group=element('g',{role:'button',tabindex:0,'aria-label':`Use node ${i} ${label} as source`,class:'graph-node'});group.append(element('circle',{cx:p.x,cy:p.y,r:20,class:i===result.graph.source?'source':result.graph.selected?.includes(i)?'selected':'node'}),element('text',{x:p.x,y:p.y+5,'text-anchor':'middle'},String(i)),element('text',{x:p.x,y:p.y+38,'text-anchor':'middle',class:'node-label'},label.length>18?label.slice(0,16)+'…':label),element('title',{},`${label}; distance ${result.distances[i]??'unreachable'}`));const choose=()=>chooseSource(i);group.addEventListener('click',choose);group.addEventListener('keydown',e=>{if(e.key==='Enter'||e.key===' '){e.preventDefault();choose();}});svg.append(group);});
}
function start(){
 epoch++;const generation=epoch;worker?.terminate();clearTimeout(startupTimer);clearTimeout(computeTimer);ready=busy=false;active=null;clear();$('retry').hidden=true;$('status').textContent='Loading Rust engine…';startupTimer=setTimeout(()=>{if(generation===epoch&&!ready)engineError('Rust did not start within twelve seconds. Retry to load it again.');},12000);
 try{worker=new Worker(new URL('./worker.js',import.meta.url),{type:'module'});}catch{engineError('This browser could not create the Rust worker.');return;}
 worker.onerror=e=>{e.preventDefault();if(generation===epoch)engineError('The Rust worker failed.');};worker.onmessage=({data})=>{
  if(generation!==epoch)return;if(data.type==='startup-error'){engineError('Rust could not be downloaded or initialized.');return;}if(data.type==='ready'){clearTimeout(startupTimer);ready=true;$('status').textContent='Rust engine ready.';controls();if(revision===0)replay();return;}
  if(!active||data.id!==active.id)return;clearTimeout(computeTimer);const submitted=active.revision;busy=false;active=null;
  if(data.type==='compute-error'){engineError('The worker could not finish replay.');return;}if(submitted!==revision){$('status').textContent='Inputs changed. Inspect again.';controls();return;}
  try{result=JSON.parse(data.canonical);}catch{engineError('The worker returned an unreadable report.');return;}canonical=data.canonical;if(!result.ok){diagnostic(result.diagnostic);$('status').textContent='Fix the request and inspect again.';controls();return;}render();$('status').textContent='Graph inspected. Review native results and exact input.';controls();
 };
}
function download(name,text,type='application/json'){const url=URL.createObjectURL(new Blob([text],{type:type+';charset=utf-8'}));const link=document.createElement('a');link.href=url;link.download=name;link.click();setTimeout(()=>URL.revokeObjectURL(url),1000);}
function example(name){cancelFile();original=null;$('request').value=examples[name];$('request-file').value='';invalidate();$('request').focus();$('request').setSelectionRange(0,0);$('request').scrollTop=0;}
$('source-node').addEventListener('change',()=>chooseSource(Number($('source-node').value)));$('request').addEventListener('input',edited);$('workbench').addEventListener('submit',e=>{e.preventDefault();replay();});$('reset').addEventListener('click',()=>example('graph'));$('retry').addEventListener('click',start);for(const button of document.querySelectorAll('[data-example]'))button.addEventListener('click',()=>example(button.dataset.example));
$('original-download').addEventListener('click',()=>download('rline-input.json',original??$('request').value));$('report-download').addEventListener('click',()=>{if(result?.ok)download('rline-graph-report.json',canonical);});$('submitted-download').addEventListener('click',()=>{if(result?.ok)download('rline-request.json',result.request_json);});$('boundary-download').addEventListener('click',()=>{if(result?.ok)download('rline-boundary.json',result.boundary_json);});
$('request-file').addEventListener('change',async()=>{
 const file=$('request-file').files[0];if(!file)return;cancelFile();invalidate();const token=fileToken;if(file.size>131072){diagnostic({message:'Choose valid UTF-8 JSON no larger than 128 KiB.'});$('request-file').value='';return;}
 reading=true;controls();fileTimer=setTimeout(()=>{if(token===fileToken){cancelFile();diagnostic({message:'Local read exceeded five seconds. Previous draft retained.'});controls();}},5000);
 try{const text=new TextDecoder('utf-8',{fatal:true,ignoreBOM:true}).decode(await file.arrayBuffer());if(token===fileToken){$('request').value=text;$('request-file').value='';edited();original=text;}}
 catch{if(token===fileToken)diagnostic({message:'File could not be read as valid UTF-8. Previous draft retained.'});}finally{if(token===fileToken){reading=false;clearTimeout(fileTimer);$('request-file').value='';}controls();}
});start();
