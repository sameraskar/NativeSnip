'use strict';
const api=globalThis.browser||chrome;
const el=id=>document.getElementById(id);
let downloadId=null;
let connected=false;
async function request(type,extra={}){const r=await api.runtime.sendMessage({type,...extra});if(!r?.ok)throw new Error(r?.error||'The extension connection is unavailable.');return r.value;}
function message(text,kind=''){el('connection').textContent=text;el('connection').className=kind;}
async function check(){
 try{const r=await request('ping');connected=true;el('heading').textContent='Ready to capture';el('setup').hidden=true;el('ready').hidden=false;el('settings').hidden=false;
 el('folder').value=r.settings.save_folder;el('launch').value=r.settings.launch_method||'shortcut';el('diagnostics').checked=r.settings.diagnostics;el('broker').checked=r.settings.broker_candidates;el('resolved').textContent=`Current folder: ${r.resolved_folder}`;message(`Connected to Windows screenshots · helper ${r.version}`,'ready');
 const state=await request('state');render(state);
 }catch(e){connected=false;el('heading').textContent='Connect to Windows screenshots';el('setup').hidden=false;el('ready').hidden=true;el('settings').hidden=true;message(/not found|no such native application|not registered|specified native messaging host/i.test(e.message)?'The Windows screenshot connection is not installed yet. Save and run the helper installer, then check again.':e.message,'error');}
}
function render(s){el('cancel').hidden=s.status!=='capturing';el('capture').disabled=s.status==='capturing';if(['saved','error','cancelled','timed_out','capturing'].includes(s.status)){message(s.message||s.status,s.status==='error'||s.status==='timed_out'?'error':s.status==='saved'?'ready':'');}}
async function task(button,fn){button.disabled=true;try{await fn();}catch(e){message(e.message,'error');}finally{button.disabled=false;if(button.id==='capture'){try{render(await request('state'));}catch{}}}}
async function install(){const r=await request('install_helper');downloadId=r.downloadId;el('reveal').hidden=false;message('Choose where to save NativeSnip-Helper-Setup-0.3.1.exe. Explorer will open when it is ready. Double-click the selected installer, then return here.');}
el('install').onclick=()=>task(el('install'),install);el('repair').onclick=()=>task(el('repair'),install);
el('reveal').onclick=()=>task(el('reveal'),()=>request('reveal_installer',{downloadId}));
el('check').onclick=()=>task(el('check'),check);
el('capture').onclick=()=>task(el('capture'),async()=>render(await request('capture')));
el('cancel').onclick=()=>task(el('cancel'),()=>request('cancel'));
el('open').onclick=()=>task(el('open'),()=>request('open_folder'));
el('browse').onclick=()=>task(el('browse'),async()=>{const r=await request('choose_folder');if(r.status==='chosen')el('folder').value=r.folder;});
el('save').onclick=()=>task(el('save'),async()=>{const r=await request('save_settings',{settings:{save_folder:el('folder').value.trim(),launch_method:el('launch').value,diagnostics:el('diagnostics').checked,broker_candidates:el('broker').checked}});el('resolved').textContent=`Current folder: ${r.resolved_folder}`;message('Settings saved.','ready');});
el('export').onclick=()=>task(el('export'),async()=>{await request('export_diagnostics');message('Diagnostic report exported and opened in Explorer. Review it before sharing.','ready');});
api.runtime.onMessage.addListener(msg=>{if(msg.type==='state_changed')render(msg.state);else if(msg.type==='installer_progress')message(msg.message);});
let lastCheck=0;
window.addEventListener('focus',()=>{if(!connected&&Date.now()-lastCheck>2000){lastCheck=Date.now();check();}});
check();
