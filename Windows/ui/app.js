import {digits,name,shortName,color,halfPath,Gesture} from './geometry.js';
const $=id=>document.getElementById(id), native=!!window.__TAURI__, query=new URLSearchParams(location.search), detailWindow=query.get('view')==='details';
let state={quota:{windows:[],updatedAt:null,loading:false,error:null},hidden:false,dark:matchMedia('(prefers-color-scheme: dark)').matches,expanded:false,preview:!native},closeTimer;
let gesture=new Gesture();
const fixtures={dual:[{minutes:300,remaining:82,resetsAt:1893456000},{minutes:10080,remaining:47,resetsAt:1894060800}],single:[{minutes:10080,remaining:82,resetsAt:1894060800}],unknown:[{minutes:300,remaining:null,resetsAt:null}],zero:[{minutes:300,remaining:0,resetsAt:null}]};
async function action(command,args={}) {
  if(native){try{return await window.__TAURI__.core.invoke(command,args);}catch{ $('error').textContent='操作失败，请通过托盘重试';}return;}
  if(command==='toggle_details')state.expanded=!state.expanded;
  if(command==='close_details')state.expanded=false;
  if(command==='refresh'){state.quota.error=null;state.quota.updatedAt=Date.now();}
  if(command==='set_hidden'){state.hidden=args.hidden;state.expanded=false;localStorage.setItem('codex-hud-preview-hidden',String(args.hidden));}
  render();
}
function svg(tag,attrs){const node=document.createElementNS('http://www.w3.org/2000/svg',tag);for(const [key,value]of Object.entries(attrs))node.setAttribute(key,value);return node;}
function render(){
  document.documentElement.classList.toggle('dark',state.dark);
  const q=state.quota,windows=q.windows.length?q.windows:[{remaining:null,minutes:null,resetsAt:null}],dual=q.windows.length>1;
  $('orb').classList.toggle('dual',dual);$('orb').style.display=state.hidden||detailWindow?'none':'block';
  $('orb').setAttribute('aria-label',`打开额度详情，${windows.map(w=>`${name(w.minutes)}剩余 ${digits(w.remaining)}`).join('，')}`);
  $('rings').setAttribute('viewBox',`0 0 36 ${dual?72:36}`);$('rings').replaceChildren();$('numbers').replaceChildren();
  windows.slice(0,2).forEach((w,i)=>{
    const attrs={fill:'none','stroke-width':1.5,'stroke-linecap':'round'};
    if(dual){$('rings').append(svg('path',{...attrs,d:halfPath(1,!!i),stroke:'var(--track)'}));if(w.remaining>0)$('rings').append(svg('path',{...attrs,d:halfPath(w.remaining/100,!!i),stroke:color(w.remaining)}));}
    else{$('rings').append(svg('circle',{...attrs,cx:18,cy:18,r:15.5,stroke:'var(--track)'}));if(w.remaining>0)$('rings').append(svg('circle',{...attrs,cx:18,cy:18,r:15.5,stroke:color(w.remaining),'stroke-dasharray':`${w.remaining/100*97.389} 97.389`,transform:'rotate(-90 18 18)'}));}
    const span=document.createElement('span');span.className='reading';if(dual){const small=document.createElement('small');small.textContent=shortName(w.minutes);span.append(small);}span.append(document.createTextNode(digits(w.remaining)));$('numbers').append(span);
  });
  const stale=q.updatedAt&&Date.now()-q.updatedAt>330000;
  $('warning').classList.toggle('visible',!!q.error||!!stale);$('rows').replaceChildren();
  for(const w of q.windows){const row=document.createElement('article'),line=document.createElement('div'),label=document.createElement('span'),value=document.createElement('span'),reset=document.createElement('p');line.className='row-line';label.textContent=name(w.minutes);value.className='value';value.textContent=digits(w.remaining)+(w.remaining===null?'':'%');line.append(label,value);reset.className='reset';const date=w.resetsAt===null?null:new Date(w.resetsAt*1000);reset.textContent=date&&Number.isFinite(date.getTime())?`重置于 ${date.toLocaleString()}`:'重置时间未知';row.append(line,reset);$('rows').append(row);}
  if(!q.windows.length)$('rows').textContent='暂无可用额度窗口';
  $('heading').textContent=state.preview?'剩余额度 · 演示':'剩余额度';
  $('sync').textContent=(q.loading?'同步中 · ':'')+(q.updatedAt?`最近成功同步 ${new Date(q.updatedAt).toLocaleString()}`:'尚未成功同步');
  $('refresh').disabled=q.loading;$('error').textContent=q.error||state.platformWarning||(stale?'数据已过期，等待下次同步':'');
  clearTimeout(closeTimer);
  if(state.expanded&&(!native||detailWindow)){$('details').hidden=false;$('details').classList.remove('closing');}
  else if(!$('details').hidden){$('details').classList.add('closing');closeTimer=setTimeout(()=>{$('details').hidden=true;},160);}
  $('visibility').textContent=state.hidden?'显示浮窗':'隐藏浮窗';
  $('preview-status').textContent=state.hidden?'演示浮窗已隐藏；刷新仍可从宿主托盘触发。':'';
}
$('orb').addEventListener('pointerdown',e=>{if(e.button!==0)return;gesture.threshold=state.dragThreshold||4;gesture.down(e.screenX,e.screenY);$('orb').setPointerCapture(e.pointerId);});
$('orb').addEventListener('pointermove',e=>{if(gesture.move(e.screenX,e.screenY)){if(native)action('start_drag');}});
$('orb').addEventListener('pointerup',()=>{if(gesture.up())action('toggle_details');});
$('orb').addEventListener('pointercancel',()=>gesture.cancel());
$('orb').addEventListener('lostpointercapture',()=>gesture.cancel());
$('orb').addEventListener('keydown',e=>{if(e.key==='Enter'||e.key===' '){e.preventDefault();action('toggle_details');}});
document.addEventListener('keydown',e=>{if(e.key==='Escape')action('close_details');});
$('close').onclick=()=>action('close_details');$('refresh').onclick=()=>action('refresh');
if(native){
  document.body.classList.toggle('details',detailWindow);
  if(detailWindow){
    let lastHeight=0;
    new ResizeObserver(()=>{
      const height=$('details').offsetHeight;
      if(height>=64&&height!==lastHeight){lastHeight=height;action('resize_details',{height});}
    }).observe($('details'));
  }
  await window.__TAURI__.event.listen('hud-state',e=>{state=e.payload;render();});
  const initial=await action('get_state');if(initial){state=initial;gesture=new Gesture(initial.dragThreshold||4);render();}
}else{
  document.body.classList.add('preview');$('preview').hidden=false;state.hidden=localStorage.getItem('codex-hud-preview-hidden')==='true';
  state.quota={windows:fixtures.dual,updatedAt:Date.now(),loading:false,error:null};
  document.querySelectorAll('[data-fixture]').forEach(button=>button.onclick=()=>{const key=button.dataset.fixture;if(key==='error'){state.quota.error='演示：同步失败，保留上次成功数据和时间';}else{state.quota={windows:fixtures[key],updatedAt:Date.now(),loading:false,error:null};}render();});
  $('theme').onclick=()=>{state.dark=!state.dark;render();};$('visibility').onclick=()=>action('set_hidden',{hidden:!state.hidden});render();
}
