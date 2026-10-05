#!/usr/bin/env python3
"""Package tested build outputs, source, docs and checksums for distribution."""
import hashlib,json,pathlib,shutil,zipfile
ROOT=pathlib.Path(__file__).resolve().parents[1]
def main():
 version=json.loads((ROOT/'project.json').read_text())['version'];out=ROOT/'dist'
 source=out/f'NativeSnip-Source-{version}.zip'
 with zipfile.ZipFile(source,'w',zipfile.ZIP_DEFLATED) as z:
  for p in sorted(ROOT.rglob('*')):
   rel=p.relative_to(ROOT)
   if p.is_file() and not any(x in {'dist','target','.git','__pycache__'} for x in rel.parts):z.write(p,pathlib.Path('NativeSnip')/rel)
 helper=out/f'NativeSnip-Helper-Setup-{version}.exe';shutil.copy2(out/'NativeSnip-Helper-Setup.exe',helper)
 stage=out/f'release-{version}';shutil.rmtree(stage,ignore_errors=True);stage.mkdir()
 files=[out/f'NativeSnip-{b}-{version}.zip' for b in ['chromium','firefox']]+[helper,source]+[ROOT/n for n in ['START-HERE.md','PRIVACY.md','RELEASE-NOTES.md','RELEASE-CHECKLIST.md','LICENSE','THIRD-PARTY-LICENSES.txt']]
 for p in files:shutil.copy2(p,stage/p.name)
 sums=''.join(hashlib.sha256(p.read_bytes()).hexdigest()+'  '+p.name+'\n' for p in sorted(stage.iterdir()))
 (stage/'SHA256SUMS.txt').write_text(sums,encoding='utf-8');shutil.copy2(stage/'SHA256SUMS.txt',out/f'NativeSnip-SHA256SUMS-{version}.txt')
 release=out/f'NativeSnip-Release-{version}.zip'
 with zipfile.ZipFile(release,'w',zipfile.ZIP_DEFLATED) as z:
  for p in sorted(stage.iterdir()):z.write(p,pathlib.Path(f'NativeSnip-{version}')/p.name)
 with zipfile.ZipFile(release) as z:assert z.testzip() is None
 for line in sums.splitlines():
  digest,name=line.split('  ',1);assert hashlib.sha256((stage/name).read_bytes()).hexdigest()==digest
 print('Final release:',release)
if __name__=='__main__':main()
