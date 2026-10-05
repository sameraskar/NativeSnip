import contextlib

import io
import json
import os
import pathlib
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

ROOT = pathlib.Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'scripts'))
import sign
import build

THUMB = '0123456789ABCDEF0123456789ABCDEF01234567'
CONFIG = {'signtool': 'signtool.exe', 'certificate_thumbprint': THUMB,
          'timestamp_url': 'https://timestamp.example/rfc3161'}


class Signing(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = pathlib.Path(self.temp.name)
        self.file = self.root / 'file with spaces.exe'
        self.file.write_bytes(b'MZtest')
        self.config_file = self.root / 'signing.json'
        self.config_file.write_text(json.dumps(CONFIG), encoding='utf-8')

    def load(self, config=None):
        if config is not None:
            self.config_file.write_text(json.dumps(config), encoding='utf-8')
        with patch.object(sign.sys, 'platform', 'win32'), patch.dict(os.environ, {'NATIVESNIP_SIGN_CONFIG': str(self.config_file)}):
            return sign.load_config()

    def test_external_config(self):
        self.assertEqual(self.load(), CONFIG)

    def test_non_windows_rejected(self):
        with patch.object(sign.sys, 'platform', 'linux'), self.assertRaises(ValueError):
            sign.load_config()

    def test_missing_config_rejected(self):
        with patch.object(sign.sys, 'platform', 'win32'), patch.dict(os.environ, {}, clear=True), self.assertRaises(ValueError):
            sign.load_config()

    def test_config_inside_source_rejected(self):
        with patch.object(sign, 'ROOT', self.root), self.assertRaises(ValueError):
            self.load()

    def test_config_inside_unnormalized_source_rejected(self):
        alias = self.root / 'unused' / '..'
        with patch.object(sign, 'ROOT', alias), self.assertRaises(ValueError):
            self.load()

    def test_bad_thumbprint_rejected(self):
        with self.assertRaises(ValueError):
            self.load({**CONFIG, 'certificate_thumbprint': 'bad'})

    def test_unknown_keys_rejected(self):
        with self.assertRaises(ValueError):
            self.load({**CONFIG, 'password': 'must-not-be-stored'})

    def test_machine_store_requires_boolean(self):
        with self.assertRaises(ValueError):
            self.load({**CONFIG, 'machine_store': 'false'})

    def test_https_timestamp_required(self):
        for url in ['', 'http://timestamp.example', 'https:']:
            with self.subTest(url=url), self.assertRaises(ValueError):
                self.load({**CONFIG, 'timestamp_url': url})

    def test_direct_signing_flags(self):
        args = sign.signing_command({**CONFIG, 'machine_store': True}, self.file)
        self.assertEqual(args, ['signtool.exe', 'sign', '/sha1', THUMB, '/fd', 'SHA256',
                                '/tr', CONFIG['timestamp_url'], '/td', 'SHA256', '/sm', str(self.file)])

    def test_custom_command_preserves_argv_and_environment_syntax(self):
        config = {**CONFIG, 'sign_command': ['cloud.exe', '--file', '{file}', '%TOKEN%']}
        self.load(config)
        self.assertEqual(sign.signing_command(config, self.file),
                         ['cloud.exe', '--file', str(self.file), '%TOKEN%'])

    def test_custom_command_requires_one_file_placeholder(self):
        for command in ['cloud.exe {file}', [], ['cloud.exe'], ['cloud.exe', '{file}', '{file}'], ['cloud.exe', 1], ['sign.cmd', '{file}']]:
            with self.subTest(command=command), self.assertRaises(ValueError):
                self.load({**CONFIG, 'sign_command': command})

    def test_verify_requires_both_tools_and_records_hash(self):
        with patch.object(sign, 'invoke') as invoke:
            result = sign.verify(self.file, CONFIG)
        self.assertEqual(invoke.call_count, 2)
        self.assertEqual(invoke.call_args_list[0].args[0],
                         ['signtool.exe', 'verify', '/pa', '/all', '/v', '/tw', str(self.file.resolve(strict=True))])
        ps = invoke.call_args_list[1]
        import base64
        script = base64.b64decode(ps.args[0][-1]).decode('utf-16le')
        for check in ['Valid', 'Authenticode', 'SignerCertificate.Thumbprint', 'TimeStamperCertificate']:
            self.assertIn(check, script)
        self.assertEqual(ps.kwargs['env']['NATIVESNIP_VERIFY_THUMBPRINT'], THUMB)
        self.assertTrue(result['timestamp_required'])
        self.assertEqual(len(result['sha256']), 64)

    def test_signing_failure_skips_verification(self):
        with patch.object(sign, 'invoke', side_effect=subprocess.CalledProcessError(1, ['tool', 'secret'])), patch.object(sign, 'verify') as verify:
            with self.assertRaises(subprocess.CalledProcessError):
                sign.sign(self.file, CONFIG)
        verify.assert_not_called()

    def test_verification_failure_propagates(self):
        with patch.object(sign, 'invoke', side_effect=[None, subprocess.CalledProcessError(1, ['powershell'])]):
            with self.assertRaises(subprocess.CalledProcessError):
                sign.verify(self.file, CONFIG)

    def test_cli_does_not_echo_provider_arguments_on_failure(self):
        with patch.object(sys, 'argv', ['sign.py', 'sign', str(self.file)]), patch.object(sign, 'load_config', return_value=CONFIG), patch.object(sign, 'sign', side_effect=subprocess.CalledProcessError(1, ['tool', 'private-value'])), contextlib.redirect_stderr(io.StringIO()) as output:
            self.assertEqual(sign.main(), 1)
        self.assertNotIn('private-value', output.getvalue())

    def test_unsigned_build_stays_opt_in(self):
        text = (ROOT / 'installer/setup.nsi').read_text(encoding='utf-8')
        self.assertIn('!ifdef SIGN_PYTHON', text)
        self.assertIn('!uninstfinalize', text)
        self.assertIn('--record "${SIGN_RECORD}"\' = 0', text)


class SignedBuild(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = pathlib.Path(self.temp.name)
        (self.root / 'native').mkdir()
        (self.root / 'project.json').write_bytes((ROOT / 'project.json').read_bytes())
        (self.root / 'native/Cargo.toml').write_text('version = "0.3.1"\n')
        self.events = []
        self.evidence = {'sha256': 'a' * 64, 'certificate_thumbprint': THUMB, 'timestamp_required': True}

    def run_command(self, args, **kwargs):
        self.events.append(('run', list(map(str, args))))
        if str(args[0]) == 'makensis':
            (self.root / 'dist/uninstaller-signature.json').write_text(json.dumps(self.evidence))

    def sign_file(self, file, config):
        self.events.append(('sign', pathlib.Path(file).name))
        return self.evidence.copy()

    def invoke_build(self, args, package_failure=False):
        with patch.object(build, 'ROOT', self.root), patch.object(sys, 'argv', ['build.py'] + args), patch.object(sign, 'load_config', return_value=CONFIG), patch.object(build, 'run', side_effect=self.run_command), patch.object(build.shutil, 'copy2'), patch.object(sign, 'sign', side_effect=self.sign_file), patch.object(sign, 'verify', return_value=self.evidence), patch.object(build, 'package', side_effect=RuntimeError('package') if package_failure else lambda: self.events.append(('package', None))), patch.dict(os.environ, {'MAKENSIS': 'makensis'}):
            build.main()

    def test_signed_order_and_nsis_hook_arguments(self):
        self.invoke_build(['--sign'])
        important = [e for e in self.events if e[0] in {'sign', 'package'} or (e[0] == 'run' and e[1][0] == 'makensis')]
        self.assertEqual(important[0], ('sign', 'autosnips-host.exe'))
        self.assertEqual(important[1][1][0], 'makensis')
        self.assertTrue(any('DSIGN_PYTHON=' in arg for arg in important[1][1]))
        self.assertTrue(any('DSIGN_RECORD=' in arg for arg in important[1][1]))
        self.assertEqual(important[2], ('sign', 'NativeSnip-Helper-Setup.exe'))
        self.assertEqual(important[3][0], 'package')
        self.assertTrue((self.root / 'dist/signing-record.json').exists())

    def test_helper_sign_failure_stops_before_installer_and_packaging(self):
        with patch.object(self, 'sign_file', side_effect=subprocess.CalledProcessError(1, ['signer'])), self.assertRaises(subprocess.CalledProcessError):
            self.invoke_build(['--sign'])
        self.assertFalse(any(e[0] == 'package' or (e[0] == 'run' and e[1][0] == 'makensis') for e in self.events))

    def test_nsis_failure_stops_before_setup_signing_and_packaging(self):
        original = self.run_command
        def fail_nsis(args, **kwargs):
            if str(args[0]) == 'makensis':
                raise subprocess.CalledProcessError(1, args)
            original(args, **kwargs)
        with patch.object(self, 'run_command', side_effect=fail_nsis), self.assertRaises(subprocess.CalledProcessError):
            self.invoke_build(['--sign'])
        self.assertNotIn(('sign', 'NativeSnip-Helper-Setup.exe'), self.events)
        self.assertFalse(any(e[0] == 'package' for e in self.events))

    def test_missing_uninstaller_receipt_stops_setup_signing(self):
        def omit_receipt(args, **kwargs):
            self.events.append(('run', list(map(str, args))))
        with patch.object(self, 'run_command', side_effect=omit_receipt), self.assertRaises(FileNotFoundError):
            self.invoke_build(['--sign'])
        self.assertNotIn(('sign', 'NativeSnip-Helper-Setup.exe'), self.events)
        self.assertFalse(any(e[0] == 'package' for e in self.events))

    def test_package_only_does_not_resign_or_compile(self):
        self.invoke_build(['--sign'])
        self.events.clear()
        self.invoke_build(['--package-only', '--sign'])
        self.assertEqual(self.events[0][0], 'package')
        self.assertFalse(any(e[0] == 'sign' for e in self.events))
        self.assertFalse(any(e[0] == 'run' and e[1][0] == 'makensis' for e in self.events))

    def test_package_only_missing_evidence_stops(self):
        with self.assertRaises(FileNotFoundError):
            self.invoke_build(['--package-only', '--sign'])
        self.assertEqual(self.events, [])

    def test_changed_artifact_stops_packaging(self):
        self.invoke_build(['--sign'])
        with patch.object(sign, 'verify', return_value={**self.evidence, 'sha256': 'b' * 64}), self.assertRaises(ValueError):
            build.verify_signed_release(self.root / 'dist', CONFIG)

    def test_wrong_certificate_stops_packaging(self):
        self.invoke_build(['--sign'])
        with self.assertRaises(ValueError):
            build.verify_signed_release(self.root / 'dist', {**CONFIG, 'certificate_thumbprint': 'F' * 40})

    def test_unsigned_rebuild_invalidates_signing_record(self):
        self.invoke_build(['--sign'])
        self.events.clear()
        self.invoke_build([])
        self.assertFalse((self.root / 'dist/signing-record.json').exists())
        self.assertFalse(any(e[0] == 'sign' for e in self.events))


if __name__ == '__main__':
    unittest.main()
