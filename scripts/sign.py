#!/usr/bin/env python3
"""Opt-in Authenticode signing; configuration and credentials stay outside source."""
import argparse
import base64
import hashlib
import json
import os
import pathlib
import re
import subprocess
import sys
from urllib.parse import urlparse

ROOT = pathlib.Path(__file__).resolve().parents[1]
TIMEOUT = 300


def load_config():
    if sys.platform != 'win32':
        raise ValueError('Authenticode release signing requires Windows.')
    name = os.environ.get('NATIVESNIP_SIGN_CONFIG')
    if not name:
        raise ValueError('Set NATIVESNIP_SIGN_CONFIG to an external signing JSON file.')
    path = pathlib.Path(name).resolve(strict=True)
    if path.is_relative_to(ROOT):
        raise ValueError('Signing configuration must be outside the source tree.')
    config = json.loads(path.read_text(encoding='utf-8-sig'))
    allowed = {'signtool', 'certificate_thumbprint', 'timestamp_url', 'machine_store', 'sign_command'}
    if not isinstance(config, dict) or set(config) - allowed:
        raise ValueError('Unknown signing configuration keys; never store credentials here.')
    thumb = config.get('certificate_thumbprint', '')
    if not isinstance(thumb, str) or not re.fullmatch(r'[0-9a-fA-F]{40}', thumb):
        raise ValueError('certificate_thumbprint must be the 40-hex public certificate thumbprint.')
    config['certificate_thumbprint'] = thumb.upper()
    if not isinstance(config.get('signtool'), str) or not config['signtool']:
        raise ValueError('Specify the Windows SDK signtool executable.')
    if not isinstance(config.get('machine_store', False), bool):
        raise ValueError('machine_store must be a boolean.')
    command = config.get('sign_command')
    if command is not None:
        if (not isinstance(command, list) or not command
                or not all(isinstance(arg, str) and arg for arg in command)
                or sum(arg.count('{file}') for arg in command) != 1):
            raise ValueError('sign_command must be an argv list with exactly one {file} placeholder.')
        if pathlib.Path(command[0]).suffix.lower() in {'.bat', '.cmd'}:
            raise ValueError('Use an executable or script interpreter, not a shell batch command.')
    else:
        url = config.get('timestamp_url')
        if not isinstance(url, str) or urlparse(url).scheme != 'https' or not urlparse(url).netloc:
            raise ValueError('Specify the provider HTTPS RFC 3161 timestamp_url.')
    return config


def invoke(args, **kwargs):
    # Do not echo provider arguments: credentials belong in secure environment/storage.
    subprocess.run(args, check=True, timeout=TIMEOUT, **kwargs)


def signing_command(config, file):
    if 'sign_command' in config:
        return [arg.replace('{file}', str(file)) for arg in config['sign_command']]
    args = [config['signtool'], 'sign', '/sha1', config['certificate_thumbprint'],
            '/fd', 'SHA256', '/tr', config['timestamp_url'], '/td', 'SHA256']
    if config.get('machine_store', False):
        args.append('/sm')
    return args + [str(file)]


def verify(file, config):
    file = pathlib.Path(file).resolve(strict=True)
    invoke([config['signtool'], 'verify', '/pa', '/all', '/v', '/tw', str(file)])
    # /tw is only a warning in SignTool. Independently require a timestamp and
    # the selected publisher, and reject catalog-only or untrusted signatures.
    script = """$ErrorActionPreference = 'Stop'
$s = Get-AuthenticodeSignature -LiteralPath $env:NATIVESNIP_VERIFY_FILE
if ($s.Status.ToString() -ne 'Valid') { throw 'Signature is not valid' }
if ($s.SignatureType.ToString() -ne 'Authenticode') { throw 'Embedded signature required' }
if ($null -eq $s.SignerCertificate -or $s.SignerCertificate.Thumbprint -ne $env:NATIVESNIP_VERIFY_THUMBPRINT) { throw 'Unexpected signing certificate' }
if ($null -eq $s.TimeStamperCertificate) { throw 'Trusted timestamp required' }
"""
    encoded = base64.b64encode(script.encode('utf-16le')).decode('ascii')
    env = os.environ.copy()
    env['NATIVESNIP_VERIFY_FILE'] = str(file)
    env['NATIVESNIP_VERIFY_THUMBPRINT'] = config['certificate_thumbprint']
    invoke(['powershell.exe', '-NoLogo', '-NoProfile', '-NonInteractive',
            '-EncodedCommand', encoded], env=env)
    return {'sha256': hashlib.sha256(file.read_bytes()).hexdigest(),
            'certificate_thumbprint': config['certificate_thumbprint'],
            'timestamp_required': True}


def sign(file, config):
    file = pathlib.Path(file).resolve(strict=True)
    print('Signing and verifying:', file.name, flush=True)
    invoke(signing_command(config, file))
    return verify(file, config)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('operation', choices=['sign', 'verify'])
    parser.add_argument('file', type=pathlib.Path)
    parser.add_argument('--record', type=pathlib.Path)
    args = parser.parse_args()
    try:
        config = load_config()
        record = sign(args.file, config) if args.operation == 'sign' else verify(args.file, config)
        if args.record:
            args.record.write_text(json.dumps(record, indent=2) + '\n', encoding='utf-8')
    except (ValueError, OSError, subprocess.SubprocessError) as error:
        # CalledProcessError includes argv; avoid exposing provider arguments.
        print('Signing/verification failed:', type(error).__name__, file=sys.stderr)
        if isinstance(error, ValueError):
            print(str(error), file=sys.stderr)
        return 1
    return 0


if __name__ == '__main__':
    sys.exit(main())
