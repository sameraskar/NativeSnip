import base64,hashlib,json,pathlib,unittest,zipfile
ROOT=pathlib.Path(__file__).resolve().parents[1]
class Packages(unittest.TestCase):
 def test_both_manifests_and_installer(self):
  c=json.loads((ROOT/'project.json').read_text())
  expected_hash=hashlib.sha256((ROOT/'dist/NativeSnip-Helper-Setup.exe').read_bytes()).digest()
  for browser in ['chromium','firefox']:
   with self.subTest(browser=browser),zipfile.ZipFile(ROOT/'dist'/f'NativeSnip-{browser}-{c["version"]}.zip') as z:
    self.assertIsNone(z.testzip());m=json.loads(z.read('manifest.json'))
    self.assertEqual(m['version'],c['version']);self.assertEqual(m['manifest_version'],3)
    self.assertEqual(set(m['permissions']),{'downloads','nativeMessaging','contextMenus'})
    self.assertNotIn('host_permissions',m);self.assertNotIn('content_scripts',m);self.assertNotIn('default_popup',m['action'])
    self.assertEqual(hashlib.sha256(z.read('helper/NativeSnip-Helper-Setup.exe')).digest(),expected_hash)
    self.assertEqual(z.read('helper/NativeSnip-Helper-Setup.exe')[:2],b'MZ')
    for f in ['setup.html','setup.js','setup.css','background.js','icons/icon-128.png']:self.assertIn(f,z.namelist())
    if browser=='chromium':
     ident=''.join(chr(97+int(x,16)) for x in hashlib.sha256(base64.b64decode(m['key'])).hexdigest()[:32]);self.assertEqual(ident,c['chromium_id']);self.assertIn('service_worker',m['background'])
    else:self.assertEqual(m['browser_specific_settings']['gecko']['id'],c['firefox_id']);self.assertIn('scripts',m['background'])
 def test_no_private_keys_or_user_data(self):
  version=json.loads((ROOT/'project.json').read_text())['version']
  for browser in ['chromium','firefox']:
   with zipfile.ZipFile(ROOT/'dist'/f'NativeSnip-{browser}-{version}.zip') as z:
    for f in z.namelist():self.assertFalse(f.endswith(('.pem','.pfx','.jsonl','.csv')),f)
if __name__=='__main__':unittest.main()
