'use strict';
const api = globalThis.browser || chrome;
const HOST = 'com.autosnips.capture';
const VERSION = '0.3.1';
const PROTOCOL = 1;
let capturePort = null;
let launchPending = false;
let state = { status: 'idle', message: 'Click the toolbar button to take a Windows screenshot.' };
function missingHost(error) {
  return /not found|no such native application|not registered|specified native messaging host/i.test(String(error?.message || error));
}
async function setState(next) {
  state = next;
  const badge = { capturing: '…', saved: '✓', error: '!', setup_required: 'SET', timed_out: '!', cancelled: '×' }[next.status] || '';
  await api.action.setBadgeText({ text: badge });
  await api.action.setBadgeBackgroundColor({ color: next.status === 'error' ? '#b42318' : '#087f8c' });
  await api.action.setTitle({ title: `NativeSnip — ${next.message || next.status}` });
  api.runtime.sendMessage({ type: 'state_changed', state }).catch(() => {});
  if (next.status === 'saved' || next.status === 'cancelled') {
    setTimeout(() => { if (state === next) { api.action.setBadgeText({text:''}).catch(()=>{});api.action.setTitle({title:'NativeSnip — Take a Windows screenshot'}).catch(()=>{}); } }, 2500);
  }
}
async function host(op, extra = {}) {
  const result = await api.runtime.sendNativeMessage(HOST, { op, ...extra });
  if (result.status === 'error') throw new Error(result.message || 'The Windows helper reported an error.');
  if (op === 'ping' && (result.protocol !== PROTOCOL || result.version !== VERSION)) {
    throw new Error(`Helper ${result.version || 'unknown'} needs updating. This extension requires ${VERSION}. Open Settings to install the matching helper.`);
  }
  return result;
}
async function page(name) { await api.runtime.openOptionsPage(); }

async function startCapture() {
  if (capturePort) { await setState({ status: 'capturing', message: 'A capture is already in progress. Press Escape in the Windows bar to cancel.' }); return state; }
  if (launchPending) return state;
  launchPending = true;
  try {
  const clickedAt = Date.now();
  setState({status:'capturing',message:'Opening Windows screenshot bar…'}).catch(()=>{});
  const platform = await api.runtime.getPlatformInfo();
  if (platform.os !== 'win') { await setState({status:'error',message:'NativeSnip currently supports Windows only.'}); await page('setup.html'); return state; }
  const port = api.runtime.connectNative(HOST); capturePort = port;
  let finished = false;
  port.onMessage.addListener(result => {
    if (finished) return;
    if (result.status === 'ready') {
      if (result.protocol !== PROTOCOL || result.version !== VERSION) {
        finished = true; capturePort = null; port.disconnect();
        setState({status:'error',message:`Install helper ${VERSION} from Settings to update the screenshot connection.`}).catch(()=>{});
        page('setup.html').catch(()=>{});
      } else port.postMessage({op:'capture',browser_ready_ms:Date.now()-clickedAt});
      return;
    }
    if (result.status === 'capturing') { setState({status:'capturing',message:result.message}).catch(()=>{}); return; }
    finished = true; capturePort = null;
    const message = result.status === 'saved' ? `Saved ${result.file}` : result.message || 'Capture cancelled. Nothing was saved.';
    setState({ ...result, message }).catch(()=>{});
    port.disconnect();
    if (result.status === 'error' || result.status === 'timed_out') page('setup.html').catch(()=>{});
  });
  port.onDisconnect.addListener(() => {
    const error = api.runtime.lastError;
    if (capturePort === port) capturePort = null;
    if (!finished) {
      finished = true;
      setState({ status:missingHost(error) ? 'setup_required' : 'error', message:error?.message || 'The screenshot connection closed before completion. Nothing has been confirmed saved.' }).catch(()=>{});
      page('setup.html').catch(()=>{});
    }
  });
  port.postMessage({ op:'hello' });
  return state;
  } finally { launchPending = false; }
}
async function installHelper() {
  const platform = await api.runtime.getPlatformInfo();
  if (platform.os !== 'win') throw new Error('This helper is for Windows.');
  const response = await fetch(api.runtime.getURL('helper/NativeSnip-Helper-Setup.exe'));
  if (!response.ok) throw new Error('The bundled installer could not be read. Extract the complete extension ZIP and reload it.');
  const bytes = new Uint8Array(await response.arrayBuffer());
  if (bytes.length < 2 || bytes[0] !== 0x4d || bytes[1] !== 0x5a) throw new Error('The bundled installer is missing or damaged. Extract the complete extension ZIP and reload it.');
  let binary = '';
  for (let offset = 0; offset < bytes.length; offset += 16384) binary += String.fromCharCode(...bytes.subarray(offset, offset + 16384));
  const url = 'data:application/octet-stream;base64,' + btoa(binary);
  return api.downloads.download({url,filename:'NativeSnip-Helper-Setup-0.3.1.exe',saveAs:true,conflictAction:'uniquify'});
}
// Chromium can assign a generic name to data-URL downloads. Set it at the final naming stage.
if (api.downloads.onDeterminingFilename) {
  api.downloads.onDeterminingFilename.addListener((item, suggest) => {
    if (item.byExtensionId === api.runtime.id && item.url.startsWith('data:application/octet-stream;base64,')) {
      suggest({filename:'NativeSnip-Helper-Setup-0.3.1.exe', conflictAction:'uniquify'});
    } else suggest();
  });
}
// Completion may happen after a setup tab closes or a background worker restarts.
api.downloads.onChanged.addListener(delta => {
  if (!delta.state && !delta.danger) return;
  api.downloads.search({id:delta.id}).then(items => {
    const file=items[0];
    if (file?.url !== api.runtime.getURL('helper/NativeSnip-Helper-Setup.exe') && !(file?.byExtensionId === api.runtime.id && file.url?.startsWith('data:application/octet-stream;base64,'))) return;
    let message = '';
    if (delta.state?.current === 'complete') {
      message = 'Installer saved. Double-click the selected installer in Explorer, finish setup, then check the connection.';
      api.downloads.show(delta.id);
    } else if (delta.state?.current === 'interrupted') message = `Installer saving was interrupted (${delta.error?.current || file.error || 'unknown error'}). Check your browser Downloads for a confirmation or error, then try again.`;
    else if (delta.danger?.current && delta.danger.current !== 'safe' && delta.danger.current !== 'accepted') message = 'Your browser may need confirmation before saving the unsigned installer. Check its Downloads panel.';
    if (message) api.runtime.sendMessage({type:'installer_progress',message}).catch(()=>{});
  }).catch(()=>{});
});
async function dispatch(msg) {
  switch (msg.type) {
    case 'state': return state;
    case 'capture': return startCapture();
    case 'cancel': if (capturePort) capturePort.postMessage({op:'cancel'}); return {status:'cancelling'};
    case 'ping': return host('ping');
    case 'install_helper': return {downloadId:await installHelper()};
    case 'reveal_installer': await api.downloads.show(msg.downloadId); return {status:'shown'};
    case 'choose_folder': return host('choose_folder');
    case 'save_settings': return host('set_settings',{settings:msg.settings});
    case 'open_folder': return host('open_folder');
    case 'export_diagnostics': return host('export_diagnostics');
    default: throw new Error('Unknown extension request.');
  }
}
api.runtime.onMessage.addListener((msg, sender, respond) => {
  if (msg.type === 'state_changed' || msg.type === 'installer_progress') return false;
  if (sender.id !== api.runtime.id) return false;
  dispatch(msg).then(value => respond({ok:true,value}), e => respond({ok:false,error:String(e.message||e)}));
  return true;
});
async function menus() {
  await api.contextMenus.removeAll();
  api.contextMenus.create({id:'settings',title:'Settings and screenshot connection',contexts:['action']});
  api.contextMenus.create({id:'folder',title:'Open screenshots folder',contexts:['action']});
  api.contextMenus.create({id:'cancel',title:'Cancel current saving session',contexts:['action']});
}
api.runtime.onInstalled.addListener(()=>{menus().catch(()=>{});});
api.contextMenus.onClicked.addListener(info => {
  if (info.menuItemId === 'settings') page('setup.html').catch(()=>{});
  else if (info.menuItemId === 'folder') host('open_folder').catch(e=>{setState({status:'error',message:e.message});page('setup.html');});
  else if (info.menuItemId === 'cancel' && capturePort) capturePort.postMessage({op:'cancel'});
});
api.action.onClicked.addListener(()=>{startCapture().catch(e=>{setState({status:'error',message:String(e.message||e)});page('setup.html');});});
