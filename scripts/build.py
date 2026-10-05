#!/usr/bin/env python3
"""One command builds the Rust helper, installer, and both extension packages."""
import argparse,base64,hashlib,json,os,pathlib,re,shutil,subprocess,sys,zipfile
import sign as signing
ROOT=pathlib.Path(__file__).resolve().parents[1]
def run(args,**kw):
 print('+',' '.join(map(str,args)),flush=True);subprocess.run(list(map(str,args)),check=True,cwd=ROOT,**kw)
def package():
 c=json.loads((ROOT/'project.json').read_text());out=ROOT/'dist';out.mkdir(exist_ok=True)
 installer=out/'NativeSnip-Helper-Setup.exe'
 if not installer.exists():raise SystemExit('Build the installer before packaging.')
 common={'manifest_version':3,'name':'NativeSnip — Windows screenshots','version':c['version'],'description':'Open Windows’ built-in screenshot bar from your toolbar and save captures using a small local helper. Windows only.','permissions':['nativeMessaging','downloads','contextMenus'],'action':{'default_title':'NativeSnip — Take a Windows screenshot','default_icon':{str(n):f'icons/icon-{n}.png' for n in [16,32,48]}},'icons':{str(n):f'icons/icon-{n}.png' for n in [16,32,48,128]},'options_ui':{'page':'setup.html','open_in_tab':True}}
 for browser in ['chromium','firefox']:
  dest=out/f'autosnips-{browser}';shutil.rmtree(dest,ignore_errors=True);shutil.copytree(ROOT/'extension',dest,ignore=shutil.ignore_patterns('chromium-public-key.der'))
  manifest=json.loads(json.dumps(common))
  if browser=='chromium':manifest.update({'key':c['chromium_key'],'minimum_chrome_version':'120','background':{'service_worker':'background.js'}})
  else:manifest.update({'background':{'scripts':['background.js']},'browser_specific_settings':{'gecko':{'id':c['firefox_id'],'strict_min_version':'140.0','data_collection_permissions':{'required':['none']}}}})
  (dest/'manifest.json').write_text(json.dumps(manifest,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
  (dest/'helper').mkdir(exist_ok=True);shutil.copy2(installer,dest/'helper/NativeSnip-Helper-Setup.exe')
  with zipfile.ZipFile(out/f'NativeSnip-{browser}-{c["version"]}.zip','w',zipfile.ZIP_DEFLATED) as z:
   for p in sorted(dest.rglob('*')):
    if p.is_file():z.write(p,p.relative_to(dest))
 run([sys.executable,'tests/package_test.py'])
 print('Built both extension ZIPs in dist/. Chromium ID:',c['chromium_id'])
def verify_signed_release(out,config):
 receipt=json.loads((out/'signing-record.json').read_text(encoding='utf-8'))
 if receipt.get('certificate_thumbprint')!=config['certificate_thumbprint']:raise ValueError('Signing record does not match the selected certificate.')
 if not receipt.get('uninstaller',{}).get('timestamp_required'):raise ValueError('Missing verified uninstaller signing record.')
 if receipt['uninstaller'].get('certificate_thumbprint')!=config['certificate_thumbprint']:raise ValueError('Uninstaller signer does not match.')
 for name in ['autosnips-host.exe','NativeSnip-Helper-Setup.exe']:
  verified=signing.verify(out/name,config)
  if verified!=receipt.get('files',{}).get(name):raise ValueError('Signed artifact changed since the signing build: '+name)

def main():
 p=argparse.ArgumentParser();p.add_argument('--target',default='x86_64-pc-windows-msvc' if sys.platform=='win32' else 'x86_64-pc-windows-gnu');p.add_argument('--package-only',action='store_true');p.add_argument('--sign',action='store_true',help='Sign helper, embedded uninstaller and installer; require external signing configuration.');a=p.parse_args()
 config=signing.load_config() if a.sign else None
 out=ROOT/'dist'
 if a.package_only and a.sign:verify_signed_release(out,config)
 c=json.loads((ROOT/'project.json').read_text());native_version=re.search(r'^version\s*=\s*"([^"]+)"',(ROOT/'native/Cargo.toml').read_text(),re.M).group(1);assert native_version==c['version'],'Native and extension versions differ';key=base64.b64decode(c['chromium_key']);id_=''.join(chr(97+int(x,16)) for x in hashlib.sha256(key).hexdigest()[:32]);assert id_==c['chromium_id'],'Chromium ID does not match public key'
 if not a.package_only:
  run([sys.executable,'-B','-m','unittest','discover','-s','tests','-p','signing_test.py'])
  run(["node","--test","tests/background.test.cjs","tests/setup.test.cjs"])
  env=os.environ.copy();env['RUSTFLAGS']=(env.get('RUSTFLAGS','')+' -C target-feature=+crt-static').strip()
  cargo=os.environ.get('CARGO','cargo');run([cargo,'test','--manifest-path','native/Cargo.toml','--locked'])
  run([cargo,'build','--release','--manifest-path','native/Cargo.toml','--target',a.target,'--locked'],env=env)
  host=ROOT/'native/target'/a.target/'release/autosnips-host.exe';out=ROOT/'dist';out.mkdir(exist_ok=True);shutil.copy2(host,out/'autosnips-host.exe')
  # Invalidate old signing evidence even when rebuilding an unsigned development release.
  receipt=out/'signing-record.json';receipt.unlink(missing_ok=True)
  uninstaller_record=out/'uninstaller-signature.json';uninstaller_record.unlink(missing_ok=True)
  makensis=os.environ.get('MAKENSIS','makensis');prefix='/' if sys.platform=='win32' else '-'
  args=[makensis,f'{prefix}DSOURCE={ROOT}',f'{prefix}DHOST_EXE={out/"autosnips-host.exe"}',f'{prefix}DOUTPUT={out/"NativeSnip-Helper-Setup.exe"}']
  if a.sign:
   helper_record=signing.sign(out/'autosnips-host.exe',config)
   args.extend([f'{prefix}DSIGN_PYTHON={sys.executable}',f'{prefix}DSIGN_RECORD={uninstaller_record}'])
  run(args+['installer/setup.nsi'])
  if a.sign:
   embedded=json.loads(uninstaller_record.read_text(encoding='utf-8'))
   if embedded.get('certificate_thumbprint')!=config['certificate_thumbprint'] or not embedded.get('timestamp_required'):raise ValueError('NSIS did not produce verified uninstaller signing evidence.')
   installer_record=signing.sign(out/'NativeSnip-Helper-Setup.exe',config)
   receipt.write_text(json.dumps({'certificate_thumbprint':config['certificate_thumbprint'],'uninstaller':embedded,'files':{'autosnips-host.exe':helper_record,'NativeSnip-Helper-Setup.exe':installer_record}},indent=2)+'\n',encoding='utf-8')
   verify_signed_release(out,config)
 package()
 run([sys.executable,"scripts/release.py"])
if __name__=='__main__':
 try:main()
 except (ValueError,OSError,subprocess.SubprocessError) as error:
  print('Build failed:',type(error).__name__,file=sys.stderr)
  if isinstance(error,ValueError):print(str(error),file=sys.stderr)
  sys.exit(1)
